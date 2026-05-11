use anyhow::Result;
use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, ToNsName, tokio::Stream, traits::tokio::Listener as _,
};
use pawmodoro::timer;
use tokio::io::{AsyncBufReadExt as _, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> Result<()> {
    let timer = timer::start();

    let name = "pawmodoro.sock".to_ns_name::<GenericNamespaced>()?;

    let listener = ListenerOptions::new()
        .name(name)
        .try_overwrite(true)
        .create_tokio()?;

    loop {
        let conn = listener.accept().await?;
        let timer = timer.clone();
        tokio::spawn(async move { handle_client(conn, timer).await });
    }
}

async fn handle_client(conn: Stream, timer: timer::Timer) -> Result<()> {
    let (read_half, mut write_half) = tokio::io::split(conn);
    let mut reader = BufReader::new(read_half);
    let mut buffer = String::with_capacity(128);

    eprintln!("Client connected");
    loop {
        let bytes = reader.read_line(&mut buffer).await?;
        if bytes == 0 {
            eprintln!("Client disconnected");
            break Ok(());
        }

        let command: timer::Command = serde_json::from_str(buffer.trim())?;
        let state = timer.execute(command).await?;

        let serialized = serde_json::to_string(&state)?;
        write_half.write_all(serialized.as_bytes()).await?;
        write_half.write_all(b"\n").await?;

        buffer.clear();
    }
}
