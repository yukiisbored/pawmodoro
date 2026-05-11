use anyhow::Result;
use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, ToNsName, tokio::Stream, traits::tokio::Listener as _,
};
use pawmodoro::timer::{self, Event};
use tokio::{
    io::{AsyncBufReadExt as _, AsyncWriteExt, BufReader},
    select,
    sync::broadcast,
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut timer_events = timer::start();

    let timer = loop {
        let event = timer_events.recv().await?;
        match event {
            timer::Event::Init(timer) => break timer,
            _ => continue,
        }
    };

    let name = "pawmodoro.sock".to_ns_name::<GenericNamespaced>()?;

    let listener = ListenerOptions::new()
        .name(name)
        .try_overwrite(true)
        .create_tokio()?;

    loop {
        let conn = listener.accept().await?;

        let events = timer_events.resubscribe();
        let timer = timer.clone();
        tokio::spawn(async move { handle_client(conn, events, timer).await });
    }
}

async fn handle_client(
    conn: Stream,
    mut events: broadcast::Receiver<timer::Event>,
    timer: timer::Timer,
) -> Result<()> {
    let (read_half, mut write_half) = tokio::io::split(conn);
    let mut reader = BufReader::new(read_half);
    let mut buffer = String::with_capacity(128);

    eprintln!("Client connected");
    loop {
        select! {
            Ok(bytes) = reader.read_line(&mut buffer) => {
                if bytes == 0 {
                    eprintln!("Client disconnected");
                    break Ok(());
                }

                let command: timer::Command = serde_json::from_str(buffer.trim())?;
                timer.send(command);

                buffer.clear();
            },
            Ok(event) = events.recv() => {
                let Event::Tick(state) = event else {
                    continue;
                };

                let serialized = serde_json::to_string(&state)?;
                write_half.write_all(serialized.as_bytes()).await?;
                write_half.write_all(b"\n").await?;
            },
        }
    }
}
