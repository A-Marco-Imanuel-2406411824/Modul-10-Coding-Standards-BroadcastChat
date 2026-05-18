use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use http::Uri;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};
use std::time::SystemTime;

fn get_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Add 7 hours for GMT+7 timezone
    let adjusted_time = now + (7 * 3600);
    let hours = (adjusted_time % 86400) / 3600;
    let minutes = (adjusted_time % 3600) / 60;
    let seconds = adjusted_time % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    // Modified to 8080
    let (mut ws_stream, _) =
        ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8080"))
            .connect()
            .await?;

    let timestamp = get_timestamp();
    println!("[{}] [SYSTEM] Connected to server on port 8080", timestamp);

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            // Read from stdin and send to server
            line = stdin.next_line() => {
                match line {
                    Ok(Some(msg)) => {
                        let timestamp = get_timestamp();
                        println!("[{}] [YOU] {}", timestamp, msg);
                        ws_stream.send(Message::text(msg)).await?;
                    }
                    Ok(None) => {
                        // EOF reached
                        break;
                    }
                    Err(e) => {
                        let timestamp = get_timestamp();
                        eprintln!("[{}] [ERROR] Error reading from stdin: {}", timestamp, e);
                        break;
                    }
                }
            }
            // Receive from server and display
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            if let Some(text) = msg.as_text() {
                                let timestamp = get_timestamp();
                                println!("[{}] [RECEIVED] {}", timestamp, text);
                            }
                        } else if msg.is_binary() {
                            let timestamp = get_timestamp();
                            println!("[{}] [SYSTEM] Received binary data", timestamp);
                        } else if msg.is_close() {
                            let timestamp = get_timestamp();
                            println!("[{}] [SYSTEM] Server closed connection", timestamp);
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        let timestamp = get_timestamp();
                        eprintln!("[{}] [ERROR] Error receiving message: {}", timestamp, e);
                        break;
                    }
                    None => {
                        let timestamp = get_timestamp();
                        println!("[{}] [SYSTEM] Connection closed", timestamp);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}