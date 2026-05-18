## Experiment 2.1:
Image from server:
![server](img.png)

Image from three consecutive clients:
![typed: i](img_1.png)
![typed: am](img_2.png)
![typed: here](img_3.png)

How to run:
- Generate a server with cargo run --bin server
- Generate a client with cargo run --bin client (generate three of them)
- Type something on each created client

Explanation: 

This experiment demonstrates a broadcast chat system built with Rust's async/await patterns and WebSocket communication. The server (visible in the first image) initializes a TCP listener on port 2000 and uses Tokio's broadcast channel to handle multiple concurrent client connections. When the server starts, it prints "listening on port 2000" and waits for incoming WebSocket connections, managing them with async tasks spawned for each new client. Each subsequent image shows a different client connecting to the server and typing messages ("i", "am", and "here") sequentially, with each message being received and processed by the server. The server's `handle_connection` function uses `tokio::select!` macro to concurrently manage two operations: receiving messages from clients via stdin and broadcasting them to all other connected clients through the broadcast channel. When a client sends a message, the server prefixes it with the client's IP address and port, then broadcasts this formatted message to all subscribed clients in real-time. The three client screenshots demonstrate the system's ability to maintain multiple persistent connections simultaneously, where each client can both send and receive messages, creating a true broadcast chat experience where all connected clients see messages from all other participants. This showcases async Rust concepts including concurrent I/O multiplexing, multi-producer broadcast channels, and proper resource cleanup when clients disconnect.

## Experiment 2.2: Changing the Port Configuration

In order to change the port to 8000 (or any other desired port), you need to modify the port number in two locations within the codebase. First, open the `chat-async/src/bin/server.rs` file and locate line 67 where `TcpListener::bind("127.0.0.1:8080")` is defined, then change `8080` to your desired port number (e.g., `8000`). Similarly, update the client configuration in `chat-async/src/bin/client.rs` at line 11 where the connection URI is specified as `Uri::from_static("ws://127.0.0.1:8080")`, changing `8080` to match your new port. After making these changes in both files, rebuild the project using `cargo build --release` or simply run the binaries with `cargo run --bin server` and `cargo run --bin client` commands, which will automatically recompile with the new port settings.

### WebSocket Protocol Details

Yes, both sides of this chat system (Server and client) uses the same **WebSocket protocol**, which is defined and managed by the `tokio-websockets` crate (version 0.13.2) specified in `Cargo.toml`. The WebSocket protocol is a standardized protocol (RFC 6455) that enables full-duplex bidirectional communication over a single TCP connection, which is ideal for real-time applications like this chat system. In the server code (`server.rs`), the WebSocket connection is established at line 76 using `ServerBuilder::new().accept(socket).await?`, which upgrades the initial TCP connection to a WebSocket connection by performing the necessary HTTP upgrade handshake. The client similarly initiates a WebSocket connection at lines 10-13 in `client.rs` using `ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8080")).connect().await?`, where the `ws://` URI scheme indicates it's using an unencrypted WebSocket protocol (as opposed to `wss://` for secure WebSocket connections over TLS). The actual message handling using the WebSocket protocol is abstracted by the `tokio-websockets` crate, which provides convenient methods like `Message::text()` for sending text frames (line 24 in client and line 46 in server) and `msg.is_text()`, `msg.is_close()` checks for receiving different message types, all following the WebSocket frame format specification. 

