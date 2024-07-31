use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use url::Url;
use tokio::sync::{mpsc, Mutex};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let url = Url::parse("ws://127.0.0.1:8080")?;
    let (ws_stream, _) = connect_async(url.as_str()).await.expect("Failed to connect");
    let ws_stream = Arc::new(Mutex::new(ws_stream));
    println!("WebSocket client connected");

    let (tx, mut rx) = mpsc::unbounded_channel();

    // Task to handle user input
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            if !line.trim().is_empty() {
                println!("Read line: {}", line); // Debugging statement
                tx_clone.send(line.trim().to_string()).unwrap();
            }
        }
    });

    // Task to handle WebSocket messages
    let ws_stream_clone = ws_stream.clone();
    let ws_task = tokio::spawn(async move {
        let mut ws_stream = ws_stream_clone.lock().await;
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received message from server: {}", text);
                }
                Ok(_) => {}
                Err(e) => {
                    println!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Process user input
    while let Some(message) = rx.recv().await {
        println!("Processing message: {}", message); // Debugging statement
        if let Err(e) = handle_user_input(message, ws_stream.clone()).await {
            println!("Error processing message: {}", e);
        }
    }

    // Send LEAVE message when program exits
    let mut ws_stream = ws_stream.lock().await;
    ws_stream.send(Message::Text("LEAVE general".to_string())).await?;
    println!("Sent LEAVE request to room: general");

    ws_task.await?;

    Ok(())
}

async fn handle_user_input(message: String, ws_stream: Arc<Mutex<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>) -> Result<()> {
    let mut ws_stream = ws_stream.lock().await;
    if message.starts_with("/JOIN ") {
        let room = message.strip_prefix("/JOIN ").unwrap();
        ws_stream.send(Message::Text(format!("JOIN {}", room))).await?;
        println!("Sent JOIN request to room: {}", room);
    } else if message.starts_with("/MSG ") {
        let parts: Vec<&str> = message.splitn(3, ' ').collect();
        if parts.len() == 3 {
            let room = parts[1];
            let text = parts[2];
            ws_stream.send(Message::Text(format!("MSG {} {}", room, text))).await?;
            println!("Sent message to room {}: {}", room, text);
        } else {
            println!("Invalid /MSG command format. Use: /MSG room_name message");
        }
    } else {
        println!("Unknown command: {}", message);
    }
    Ok(())
}