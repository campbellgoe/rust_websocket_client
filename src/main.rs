use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    let url = Url::parse("ws://127.0.0.1:8080")?;
    let (mut ws_stream, _) = connect_async(url.as_str()).await.expect("Failed to connect");
    println!("WebSocket client connected");

    // Sending a message to the server
    let message = "Hello, Server!";
    ws_stream.send(Message::Text(message.into())).await?;

    // Receiving messages from the server
    while let Some(msg) = ws_stream.next().await {
        match msg? {
            Message::Text(text) => {
                println!("Received message from server: {}", text);
            }
            _ => {}
        }
    }

    Ok(())
}