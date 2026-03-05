use raps_mock::TestServer;
use reqwest::Client;

#[tokio::main]
async fn main() {
    let server = TestServer::start_default().await.unwrap();
    let client = Client::new();
    
    let url = format!("{}/construction/admin/v1/projects/proj-001/users", server.url);
    println!("Requesting: {}", url);
    
    let resp = client.post(url)
        .bearer_auth("mock-3leg-token")
        .json(&serde_json::json!({
            "email": "test@example.com",
            "roleId": "role-admin"
        }))
        .send()
        .await
        .unwrap();
        
    println!("Status: {}", resp.status());
    let body = resp.text().await.unwrap();
    println!("Body: {}", body);
}
