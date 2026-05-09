use iced::{
    Alignment::Center,
    Element, Subscription, Task,
    widget::{button, column, row, text},
};
use pawmodoro::{
    config::Config,
    timer::{self},
};

pub fn main() -> iced::Result {
    env_logger::init();

    iced::application(State::new, State::update, State::view)
        .subscription(State::subscription)
        .centered()
        .run()
}

enum Time {
    Running(u64),
    Paused(u64),
}

impl Time {
    fn remaining(&self) -> u64 {
        match self {
            Time::Running(remaining) | Time::Paused(remaining) => *remaining,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Work,
    ShortBreak,
    LongBreak,
}

struct State {
    config: Config,
    mode: Mode,
    timer: Option<timer::Timer>,
    time: Time,
    rounds: u64,
}

#[derive(Debug, Clone)]
enum Message {
    Timer(timer::Event),
    Start,
    Pause,
    Switch(Mode),
}

impl State {
    fn new() -> Self {
        let config = Config::default();

        State {
            config: Default::default(),
            mode: Mode::Work,
            timer: None,
            time: Time::Paused(config.work_duration),
            rounds: 0,
        }
    }

    fn next_mode(&self) -> Mode {
        match self.mode {
            Mode::Work => {
                if self.rounds % self.config.rounds_before_long_break == 0 {
                    Mode::LongBreak
                } else {
                    Mode::ShortBreak
                }
            }
            Mode::ShortBreak => Mode::Work,
            Mode::LongBreak => Mode::Work,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Timer(timer::Event::Init(timer)) => {
                self.timer = Some(timer);

                Task::none()
            }
            Message::Timer(timer::Event::Tick(remaining)) => {
                self.time = Time::Running(remaining);

                Task::none()
            }
            Message::Timer(timer::Event::Stopped) => {
                self.time = Time::Paused(0);

                if self.mode == Mode::Work {
                    self.rounds += 1;
                }

                let mode = self.next_mode();

                Task::done(Message::Switch(mode))
            }
            Message::Start => {
                if let Some(ref timer) = self.timer {
                    let duration = self.time.remaining();
                    timer.start(duration);
                }

                Task::none()
            }
            Message::Pause => {
                if let Some(ref timer) = self.timer {
                    timer.stop();
                }

                Task::none()
            }
            Message::Switch(mode) => {
                if let Some(ref timer) = self.timer {
                    timer.stop();
                }

                let duration = match mode {
                    Mode::Work => self.config.work_duration,
                    Mode::ShortBreak => self.config.short_break_duration,
                    Mode::LongBreak => self.config.long_break_duration,
                };

                self.mode = mode;
                self.time = Time::Paused(duration);

                Task::none()
            }
        }
    }

    fn subscription(_: &Self) -> Subscription<Message> {
        Subscription::run(|| timer::start()).map(Message::Timer)
    }

    fn view(&self) -> Element<'_, Message> {
        let modes = {
            let work = mode_button(&self.mode, Mode::Work);
            let short_break = mode_button(&self.mode, Mode::ShortBreak);
            let long_break = mode_button(&self.mode, Mode::LongBreak);

            row![work, short_break, long_break].spacing(8)
        };

        let time = {
            let value = self.time.remaining();
            let minutes = value / 60;
            let seconds = value % 60;
            let time = format!("{:02}:{:02}", minutes, seconds);

            text(time).size(80)
        };

        let button = match self.time {
            Time::Running(_) => button("Pause").on_press(Message::Pause),
            Time::Paused(_) => button("Start").on_press(Message::Start),
        };

        column![modes, time, button]
            .align_x(Center)
            .spacing(16)
            .padding(32)
            .into()
    }
}

fn mode_button<'a>(current_mode: &'a Mode, mode: Mode) -> Element<'a, Message> {
    let label = match mode {
        Mode::Work => "Work",
        Mode::ShortBreak => "Short Break",
        Mode::LongBreak => "Long Break",
    };

    if *current_mode == mode {
        button(label).into()
    } else {
        button(label).on_press(Message::Switch(mode)).into()
    }
}
