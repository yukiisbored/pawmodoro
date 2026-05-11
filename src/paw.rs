use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use inquire::Select;
use interprocess::local_socket::{
    GenericNamespaced, ToNsName, tokio::Stream, traits::tokio::Stream as _,
};
use pawmodoro::timer::{Command, Mode, State, TimerState};
use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader};

#[derive(Parser)]
#[command(name = "paw", about = "Control the pawmodoro timer daemon")]
struct Cli {
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Start the timer
    Start,
    /// Pause the timer
    Pause,
    /// Change the mode
    Switch {
        #[arg(value_enum)]
        mode: Option<ModeArg>,
    },
    /// Go to the next mode
    Next,
    /// Show the latest state
    Status,
}

#[derive(Clone, Copy, ValueEnum)]
enum ModeArg {
    Pomodoro,
    ShortBreak,
    LongBreak,
}

impl From<ModeArg> for Mode {
    fn from(value: ModeArg) -> Self {
        match value {
            ModeArg::Pomodoro => Mode::Pomodoro,
            ModeArg::ShortBreak => Mode::ShortBreak,
            ModeArg::LongBreak => Mode::LongBreak,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let command = match cli.command.unwrap_or(Cmd::Status) {
        Cmd::Start => Command::Start,
        Cmd::Pause => Command::Pause,
        Cmd::Switch { mode } => Command::Switch(match mode {
            Some(mode) => mode.into(),
            None => prompt_mode()?,
        }),
        Cmd::Next => Command::Next,
        Cmd::Status => Command::Status,
    };

    let state = send(&command).await?;
    print_state(&state);

    Ok(())
}

fn prompt_mode() -> Result<Mode> {
    let options = vec!["Pomodoro", "Short Break", "Long Break"];
    let choice = Select::new("Select mode:", options).prompt()?;
    Ok(match choice {
        "Pomodoro" => Mode::Pomodoro,
        "Short Break" => Mode::ShortBreak,
        "Long Break" => Mode::LongBreak,
        _ => unreachable!(),
    })
}

async fn send(command: &Command) -> Result<State> {
    let name = "pawmodoro.sock".to_ns_name::<GenericNamespaced>()?;
    let conn = Stream::connect(name)
        .await
        .context("could not connect to pawmodoro daemon — is it running?")?;
    let (read_half, mut write_half) = tokio::io::split(conn);
    let mut reader = BufReader::new(read_half);

    let request = serde_json::to_string(command)?;
    write_half.write_all(request.as_bytes()).await?;
    write_half.write_all(b"\n").await?;

    let mut buffer = String::with_capacity(256);
    let bytes = reader.read_line(&mut buffer).await?;
    if bytes == 0 {
        anyhow::bail!("daemon closed the connection without responding");
    }

    let state = serde_json::from_str(buffer.trim())?;
    Ok(state)
}

fn print_state(state: &State) {
    let mode = match state.mode {
        Mode::Pomodoro => "Pomodoro".red().bold(),
        Mode::ShortBreak => "Short Break".green().bold(),
        Mode::LongBreak => "Long Break".blue().bold(),
    };

    let (remaining, status) = match state.timer {
        TimerState::Running(remaining) => (remaining, "running".green()),
        TimerState::Paused(remaining) => (remaining, "paused".yellow()),
    };

    let minutes = remaining / 60;
    let seconds = remaining % 60;
    let time = format!("{minutes:02}:{seconds:02}").bold();

    println!("{}  {}  {}", mode, time, status);
    println!("{} {}", "Rounds completed:".dimmed(), state.rounds);
}
