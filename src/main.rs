use futures::StreamExt as _;
use iced::{
    Element, Program, Subscription, Theme,
    widget::{button, column, text},
};
use pawmodoro::timer::{self, Timer};

pub fn main() -> iced::Result {
    env_logger::init();

    iced::application(State::default, update, view)
        .subscription(subscription)
        .theme(Theme::Dark)
        .centered()
        .run()
}

enum Time {
    Running(u64),
    Paused(u64),
}

impl Default for Time {
    fn default() -> Self {
        Time::Paused(30)
    }
}

#[derive(Default)]
struct State {
    timer: Option<timer::Timer>,
    time: Time,
}

#[derive(Debug, Clone)]
enum Message {
    Timer(timer::Event),
    Start,
    Pause,
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::Timer(timer::Event::Init(timer)) => {
            state.timer = Some(timer);
        }
        Message::Timer(timer::Event::Tick(remaining)) => {
            state.time = Time::Running(remaining);
            println!("Time remaining: {} seconds", remaining);
        }
        Message::Timer(timer::Event::Paused(remaining)) => {
            state.time = Time::Paused(remaining);
            println!("Timer paused with {} seconds remaining", remaining);
        }
        Message::Start => {
            if let Some(ref timer) = state.timer {
                let duration = match state.time {
                    Time::Running(remaining) | Time::Paused(remaining) => remaining,
                };
                timer.start(duration);
            }
        }
        Message::Pause => {
            if let Some(ref timer) = state.timer {
                timer.pause();
            }
        }
    }
}

fn subscription(_: &State) -> Subscription<Message> {
    Subscription::run(|| timer::start()).map(Message::Timer)
}

fn view(state: &State) -> Element<'_, Message> {
    let time = {
        let value = match state.time {
            Time::Running(remaining) | Time::Paused(remaining) => remaining,
        };
        let minutes = value / 60;
        let seconds = value % 60;
        let time = format!("{:02}:{:02}", minutes, seconds);

        text(time)
    };

    let button = match state.time {
        Time::Running(_) => button("Pause").on_press(Message::Pause),
        Time::Paused(_) => button("Start").on_press(Message::Start),
    };

    column![time, button].into()
}
