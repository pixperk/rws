use rws_common::Message;
use tokio_tungstenite::connect_async;
use futures_util::{SinkExt};
use url::Url;

#[tokio::main]
async fn main(){
    let (mut ws_stream, _) = connect_async(Url::parse("ws://localhost:3000").unwrap()).await.unwrap();

    println!("Connected to the server");

    // Example of sending a message
    let msg = Message{
        event : "chat".to_string(),
         data: serde_json::json!({ "user": "yashaswi", "msg": "yo server" }),
    };

    let raw = serde_json::to_string(&msg).unwrap();
    ws_stream.send(tungstenite::Message::Text(raw)).await.unwrap();


}