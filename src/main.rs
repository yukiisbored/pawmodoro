use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Subscription, Task,
    widget::{button, column, row, space, text},
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
        .window_size((420, 264))
        .resizable(false)
        .title("Pawmodoro")
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
    Pomodoro,
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
    Next,
}

impl State {
    fn new() -> Self {
        let config = Config::default();

        State {
            config: Default::default(),
            mode: Mode::Pomodoro,
            timer: None,
            time: Time::Paused(config.work_duration),
            rounds: 0,
        }
    }

    fn next_mode(&self) -> Mode {
        match self.mode {
            Mode::Pomodoro => {
                if self.rounds % self.config.rounds_before_long_break == 0 {
                    Mode::LongBreak
                } else {
                    Mode::ShortBreak
                }
            }
            Mode::ShortBreak => Mode::Pomodoro,
            Mode::LongBreak => Mode::Pomodoro,
        }
    }

    fn stop_timer(&mut self) {
        if let Some(ref timer) = self.timer {
            timer.stop();
            self.time = Time::Paused(self.time.remaining());
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

                Task::done(Message::Next)
            }
            Message::Start => {
                if let Some(ref timer) = self.timer {
                    let duration = self.time.remaining();
                    self.time = Time::Running(duration);
                    timer.start(duration);
                }

                Task::none()
            }
            Message::Pause => {
                self.stop_timer();

                Task::none()
            }
            Message::Switch(mode) => {
                self.stop_timer();

                let duration = match mode {
                    Mode::Pomodoro => self.config.work_duration,
                    Mode::ShortBreak => self.config.short_break_duration,
                    Mode::LongBreak => self.config.long_break_duration,
                };

                self.mode = mode;
                self.time = Time::Paused(duration);

                Task::none()
            }
            Message::Next => {
                if self.mode == Mode::Pomodoro {
                    self.rounds += 1;
                }

                let mode = self.next_mode();

                Task::done(Message::Switch(mode))
            }
        }
    }

    fn subscription(_: &Self) -> Subscription<Message> {
        Subscription::run(|| timer::start()).map(Message::Timer)
    }

    fn view(&self) -> Element<'_, Message> {
        let modes = {
            let work = mode_button(&self.mode, Mode::Pomodoro);
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

        let is_running = matches!(self.time, Time::Running(_));

        let bottom = {
            let primary_button = {
                let label = if is_running { "Pause" } else { "Start" };
                let message = if is_running {
                    Message::Pause
                } else {
                    Message::Start
                };

                button(text(label).width(Fill).center())
                    .on_press(message)
                    .width(72)
            };

            let skip_button: Element<'_, Message> = {
                let button = button(text("Skip").width(Fill).center()).width(72);

                if is_running {
                    button.on_press(Message::Next).into()
                } else {
                    button.into()
                }
            };

            row![space().width(72), primary_button, skip_button].spacing(8)
        };

        column![modes, time, bottom]
            .align_x(Center)
            .spacing(16)
            .padding(32)
            .into()
    }
}

fn mode_button<'a>(current_mode: &'a Mode, mode: Mode) -> Element<'a, Message> {
    let label = match mode {
        Mode::Pomodoro => "Pomodoro",
        Mode::ShortBreak => "Short Break",
        Mode::LongBreak => "Long Break",
    };

    let button = button(text(label).width(Fill).center());

    if *current_mode == mode {
        button.into()
    } else {
        button.on_press(Message::Switch(mode)).into()
    }
}
