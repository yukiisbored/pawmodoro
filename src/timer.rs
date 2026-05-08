use std::time::Duration;

use futures::{Stream, channel::mpsc::Sender};
use iced::stream;
use tokio::{select, sync::mpsc, time};

#[derive(Debug, Clone)]
pub struct Timer(mpsc::Sender<Command>);

impl Timer {
    pub fn start(&self, duration: u64) {
        self.0.try_send(Command::Start(duration)).unwrap();
    }

    pub fn pause(&self) {
        self.0.try_send(Command::Pause).unwrap();
    }
}

enum TimerState {
    Running(time::Interval, u64),
    Paused,
}

#[derive(Debug, Clone)]
pub enum Event {
    Init(Timer),
    Tick(u64),
    Paused(u64),
}

#[derive(Debug, Clone)]
pub enum Command {
    Start(u64),
    Pause,
}

pub fn start() -> impl Stream<Item = Event> {
    stream::channel(100, |mut output: Sender<Event>| async move {
        let mut state = TimerState::Paused;
        let (tx, mut rx) = mpsc::channel(100);
        let timer = Timer(tx);
        output.try_send(Event::Init(timer)).unwrap();

        loop {
            select! {
                Some(cmd) = rx.recv() => {
                    match cmd {
                        Command::Start(duration) => {
                            state = TimerState::Running(new_interval(), duration);
                        },
                        Command::Pause => {
                            if let TimerState::Running(_, remaining) = state {
                                state = TimerState::Paused;
                                output.try_send(Event::Paused(remaining)).unwrap();
                            }
                        },
                    }
                }
                _ = async {
                    if let TimerState::Running(ref mut interval, _) = state {
                        interval.tick().await;
                    } else {
                        futures::future::pending::<()>().await;
                    }
                } => {
                    if let TimerState::Running(_, ref mut remaining) = state {
                        if *remaining > 0 {
                            *remaining -= 1;
                            output.try_send(Event::Tick(*remaining)).unwrap();
                        } else {
                            state = TimerState::Paused;
                            output.try_send(Event::Paused(0)).unwrap();
                        }
                    }
                }
            }
        }
    })
}

/// Start one second from now so the first tick fires after 1s, not immediately.
fn new_interval() -> time::Interval {
    time::interval_at(
        time::Instant::now() + Duration::from_secs(1),
        Duration::from_secs(1),
    )
}
