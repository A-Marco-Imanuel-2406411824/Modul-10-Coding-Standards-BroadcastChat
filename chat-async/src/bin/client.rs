use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use http::Uri;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    let (mut ws_stream, _) =
        ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:2000"))
            .connect()
            .await?;

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            // Read from stdin and send to server
            line = stdin.next_line() => {
                match line {
                    Ok(Some(msg)) => {
                        ws_stream.send(Message::text(msg)).await?;
                    }
                    Ok(None) => {
                        // EOF reached
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error reading from stdin: {}", e);
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
                                println!("{}", text);
                            }
                        } else if msg.is_binary() {
                            println!("Received binary data");
                        } else if msg.is_close() {
                            println!("Server closed connection");
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    }
                    None => {
                        println!("Connection closed");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}