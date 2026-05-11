use anyhow::Result;
use interprocess::local_socket::{
    GenericNamespaced, ToNsName, tokio::Stream, traits::tokio::Stream as _,
};
use tokio::io::{AsyncBufReadExt as _, BufReader};

#[tokio::main]
async fn main() -> Result<()> {
    let name = "pawmodoro.sock".to_ns_name::<GenericNamespaced>()?;
    let mut buffer = String::with_capacity(128);
    let conn = Stream::connect(name).await?;
    let mut reader = BufReader::new(&conn);

    loop {
        let Ok(bytes) = reader.read_line(&mut buffer).await else {
            break Ok(());
        };

        if bytes == 0 {
            break Ok(());
        }

        println!("Received: {}", buffer.trim());
    }
}
