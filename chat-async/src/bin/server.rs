use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Sender, channel};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};
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

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
    client_count: Arc<AtomicUsize>,
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
                                let timestamp = get_timestamp();
                                println!("[{}] [MESSAGE] From client {}: \"{}\"", timestamp, addr, text);
                                let _ = bcast_tx.send(format!("[{}] [{}]: {}", timestamp, addr, text));
                            }
                        } else if msg.is_close() {
                            let timestamp = get_timestamp();
                            client_count.fetch_sub(1, Ordering::SeqCst);
                            let current_clients = client_count.load(Ordering::SeqCst);
                            println!("[{}] [SYSTEM] Client {} disconnected (Active clients: {})", timestamp, addr, current_clients);
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        let timestamp = get_timestamp();
                        eprintln!("[{}] [ERROR] Error from client {}: {}", timestamp, addr, e);
                        break;
                    }
                    None => {
                        let timestamp = get_timestamp();
                        client_count.fetch_sub(1, Ordering::SeqCst);
                        let current_clients = client_count.load(Ordering::SeqCst);
                        println!("[{}] [SYSTEM] Client {} closed connection (Active clients: {})", timestamp, addr, current_clients);
                        break;
                    }
                }
            }
            // Receive from broadcast and send to client
            msg = bcast_rx.recv() => {
                match msg {
                    Ok(text) => {
                        if let Err(e) = ws_stream.send(Message::text(text)).await {
                            let timestamp = get_timestamp();
                            eprintln!("[{}] [ERROR] Error sending to client {}: {}", timestamp, addr, e);
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
    let client_count = Arc::new(AtomicUsize::new(0));

    // Modified to 8080
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let timestamp = get_timestamp();
    println!("[{}] [SYSTEM] Server started - listening on port 8080", timestamp);

    loop {
        let (socket, addr) = listener.accept().await?;
        client_count.fetch_add(1, Ordering::SeqCst);
        let current_clients = client_count.load(Ordering::SeqCst);
        let timestamp = get_timestamp();
        println!("[{}] [SYSTEM] New connection from {} (Active clients: {})", timestamp, addr, current_clients);
        let bcast_tx = bcast_tx.clone();
        let client_count = Arc::clone(&client_count);
        tokio::spawn(async move {
            // Wrap the raw TCP stream into a websocket.
            let (_req, ws_stream) = ServerBuilder::new().accept(socket).await?;

            handle_connection(addr, ws_stream, bcast_tx, client_count).await
        });
    }
}