use rws_server::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = Server::bind("127.0.0.1:3000").await?;
    server.run().await
}
