use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Sender, channel};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            // Receive from client and broadcast
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            if let Some(text) = msg.as_text() {
                                println!("[{}] {}", addr, text);
                                let _ = bcast_tx.send(format!("[{}] {}", addr, text));
                            }
                        } else if msg.is_close() {
                            println!("Client {} disconnected", addr);
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Error from client {}: {}", addr, e);
                        break;
                    }
                    None => {
                        println!("Client {} closed connection", addr);
                        break;
                    }
                }
            }
            // Receive from broadcast and send to client
            msg = bcast_rx.recv() => {
                match msg {
                    Ok(text) => {
                        if let Err(e) = ws_stream.send(Message::text(text)).await {
                            eprintln!("Error sending to client {}: {}", addr, e);
                            break;
                        }
                    }
                    Err(_) => {
                        // Broadcast sender dropped, continue
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);

    // Modified to 8080
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let bcast_tx = bcast_tx.clone();
        tokio::spawn(async move {
            // Wrap the raw TCP stream into a websocket.
            let (_req, ws_stream) = ServerBuilder::new().accept(socket).await?;

            handle_connection(addr, ws_stream, bcast_tx).await
        });
    }
}