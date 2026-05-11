use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::{mpsc, oneshot},
    time,
};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Timer(mpsc::Sender<Request>);

impl Timer {
    pub async fn execute(&self, cmd: Command) -> Result<State> {
        let (tx, rx) = oneshot::channel();
        self.0.send(Request(cmd, tx)).await?;
        Ok(rx.await?)
    }
}

struct Request(Command, oneshot::Sender<State>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimerState {
    Running(u64),
    Paused(u64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    Pomodoro,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Start,
    Pause,
    Switch(Mode),
    Next,
    Status,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub config: Config,
    pub mode: Mode,
    pub timer: TimerState,
    pub rounds: u64,
}

impl State {
    fn new(config: Config) -> Self {
        let work_duration = config.work_duration;

        State {
            config,
            mode: Mode::Pomodoro,
            timer: TimerState::Paused(work_duration),
            rounds: 0,
        }
    }

    fn next_mode(&self) -> Mode {
        match self.mode {
            Mode::Pomodoro => {
                if self
                    .rounds
                    .is_multiple_of(self.config.rounds_before_long_break)
                {
                    Mode::LongBreak
                } else {
                    Mode::ShortBreak
                }
            }
            Mode::ShortBreak | Mode::LongBreak => Mode::Pomodoro,
        }
    }

    fn pause(&mut self) {
        if let TimerState::Running(remaining) = self.timer {
            self.timer = TimerState::Paused(remaining);
        }
    }

    fn update(&mut self, cmd: Command) {
        match cmd {
            Command::Status => {}
            Command::Start => {
                if let TimerState::Paused(remaining) = self.timer {
                    self.timer = TimerState::Running(remaining);
                }
            }
            Command::Pause => self.pause(),

            Command::Switch(mode) => {
                self.pause();

                let duration = match mode {
                    Mode::Pomodoro => self.config.work_duration,
                    Mode::ShortBreak => self.config.short_break_duration,
                    Mode::LongBreak => self.config.long_break_duration,
                };

                self.mode = mode;
                self.timer = TimerState::Paused(duration);
            }
            Command::Next => {
                if self.mode == Mode::Pomodoro {
                    self.rounds += 1;
                }

                let mode = self.next_mode();

                self.update(Command::Switch(mode))
            }
        }
    }
}

pub fn start() -> Timer {
    let (tx, mut rx) = mpsc::channel::<Request>(100);

    tokio::spawn(async move {
        let mut state = State::new(Config::default());
        let mut interval: Option<time::Interval> = None;

        loop {
            select! {
                Some(Request(cmd, reply)) = rx.recv() => {
                    state.update(cmd);
                    let _ = reply.send(state.clone());
                }
                _ = async {
                    if let TimerState::Paused(_) = state.timer {
                        return std::future::pending::<()>().await;
                    }

                    let interval = match interval {
                        Some(ref mut interval) => interval,
                        None => {
                            interval = Some(new_interval());
                            interval.as_mut().unwrap()
                        }
                    };

                    interval.tick().await;
                } => {
                    if let TimerState::Running(ref mut remaining) = state.timer {
                        if *remaining > 0 {
                            *remaining -= 1;
                        } else {
                            state.update(Command::Next);
                        }
                    }
                }
            }
        }
    });

    Timer(tx)
}

/// Start one second from now so the first tick fires after 1s, not immediately.
fn new_interval() -> time::Interval {
    time::interval_at(
        time::Instant::now() + Duration::from_secs(1),
        Duration::from_secs(1),
    )
}
