# RWS - Rust WebSocket Framework

A modular WebSocket chat framework built with Rust, featuring real-time messaging, room management, and an extensible event hook system.

## Features

- **Real-time messaging** with delivery tracking
- **Room-based chat** with join/leave functionality
- **Extensible event system** similar to Socket.IO
- **Modular architecture** with reusable core library
- **Terminal UI client** with message history
- **Type-safe message protocol** using structured events

## Architecture

The project is split into multiple crates:

- `rws-core` - Core WebSocket protocol and server logic
- `rws-common` - Shared message types and data structures
- `rws-server` - Server binary with event hook registration
- `rws-client` - Terminal-based chat client with TUI

## Quick Start

### Running the Server

```bash
cargo run --bin rws-server
```

Server starts on `ws://localhost:3000`

### Running the Client

```bash
cargo run --bin rws-client -- --username alice --server ws://localhost:3000
```

### Basic Usage

- Type messages and press Enter to send
- Use `/create <room-name>` to create a room
- Use `/join <room-id>` to join a room
- Use `/leave` to leave current room
- Press Ctrl+Q to quit

## Event System

The framework uses a hook-based event system where you can register custom handlers:

```rust
let mut handlers = EventHandlers::default();
handlers.on_join = Some(handle_user_join);
handlers.on_chat = Some(handle_chat_message);

let server = Server { handlers, /* ... */ };
```

## Message Protocol

All communication uses JSON-serialized events:

- `Join` - User connects with username
- `Chat` - Send/receive chat messages
- `CreateRoom` - Create a new chat room
- `JoinRoom` / `LeaveRoom` - Room management
- `AssignedId` - Server assigns UUID to client

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Project Structure

```
rws/
├── rws-core/          # Core WebSocket framework
├── rws-common/        # Shared types and protocols
├── rws-server/        # Server binary
├── rws-client/        # Terminal client
└── Cargo.toml         # Workspace configuration
```

## Dependencies

- `tokio` - Async runtime
- `tokio-tungstenite` - WebSocket implementation
- `serde` - JSON serialization
- `ratatui` - Terminal UI (client only)
- `uuid` - Unique identifiers

## License

MIT License
