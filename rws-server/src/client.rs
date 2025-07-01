use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use futures_util::stream::SplitSink;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub type Tx = Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, WsMessage>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub id: Uuid,
    pub username : Option<String>,
    pub tx: Tx,
}

pub type Clients = Arc<Mutex<HashMap<Uuid, Client>>>;