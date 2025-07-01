use clap::Parser;

#[derive(Parser)]
#[command(name = "rws-client")]
#[command(about = "A beautiful WebSocket chat client")]
#[command(version = "1.0")]
pub struct Args {
    /// Your username
    #[arg(short, long, default_value = "anonymous")]
    pub username: String,

    /// Server URL
    #[arg(short, long, default_value = "ws://localhost:3000")]
    pub server: String,
}
