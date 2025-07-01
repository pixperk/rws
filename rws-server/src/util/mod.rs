use crate::client::Clients;

pub async fn get_username_from_client(
    clients: &Clients,
    client_id: uuid::Uuid,
) -> Option<String> {
    let clients = clients.lock().await;
    clients.get(&client_id).and_then(|c| c.username.clone())
}