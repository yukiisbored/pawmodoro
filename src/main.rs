use iced::{
    Element, Subscription, Task, Theme,
    widget::{button, column, row, text},
};
use pawmodoro::{
    config::Config,
    timer::{self},
};

pub fn main() -> iced::Result {
    env_logger::init();

    iced::application(new, update, view)
        .subscription(subscription)
        .theme(Theme::Dark)
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

fn new() -> State {
    let config = Config::default();

    State {
        config: Default::default(),
        mode: Mode::Work,
        timer: None,
        time: Time::Paused(config.work_duration),
        rounds: 0,
    }
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Timer(timer::Event::Init(timer)) => {
            state.timer = Some(timer);

            Task::none()
        }
        Message::Timer(timer::Event::Tick(remaining)) => {
            state.time = Time::Running(remaining);

            Task::none()
        }
        Message::Timer(timer::Event::Stopped) => {
            state.time = Time::Paused(0);

            if state.mode == Mode::Work {
                state.rounds += 1;
            }

            let mode = match state.mode {
                Mode::Work => {
                    if state.rounds % state.config.rounds_before_long_break == 0 {
                        Mode::LongBreak
                    } else {
                        Mode::ShortBreak
                    }
                }
                Mode::ShortBreak => Mode::Work,
                Mode::LongBreak => Mode::Work,
            };

            Task::done(Message::Switch(mode))
        }
        Message::Start => {
            if let Some(ref timer) = state.timer {
                let duration = state.time.remaining();
                timer.start(duration);
            }

            Task::none()
        }
        Message::Pause => {
            if let Some(ref timer) = state.timer {
                timer.stop();
            }

            Task::none()
        }
        Message::Switch(mode) => {
            if let Some(ref timer) = state.timer {
                timer.stop();
            }

            let duration = match mode {
                Mode::Work => state.config.work_duration,
                Mode::ShortBreak => state.config.short_break_duration,
                Mode::LongBreak => state.config.long_break_duration,
            };

            state.mode = mode;
            state.time = Time::Paused(duration);

            Task::none()
        }
    }
}

fn subscription(_: &State) -> Subscription<Message> {
    Subscription::run(|| timer::start()).map(Message::Timer)
}

fn view(state: &State) -> Element<'_, Message> {
    let modes = {
        let work = mode_button(&state.mode, Mode::Work);
        let short_break = mode_button(&state.mode, Mode::ShortBreak);
        let long_break = mode_button(&state.mode, Mode::LongBreak);

        row![work, short_break, long_break]
    };

    let time = {
        let value = state.time.remaining();
        let minutes = value / 60;
        let seconds = value % 60;
        let time = format!("{:02}:{:02}", minutes, seconds);

        text(time)
    };

    let button = match state.time {
        Time::Running(_) => button("Pause").on_press(Message::Pause),
        Time::Paused(_) => button("Start").on_press(Message::Start),
    };

    column![modes, time, button].into()
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
