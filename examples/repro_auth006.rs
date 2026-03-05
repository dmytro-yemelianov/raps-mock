use raps_mock::TestServer;
use reqwest::Client;

#[tokio::main]
async fn main() {
    let server = TestServer::start_default().await.unwrap();
    let client = Client::new();
    
    // This is the endpoint that failed in CLI tests
    let url = format!("{}/construction/admin/v1/accounts/acc/projects", server.url);
    println!("Requesting: {}", url);
    
    let resp = client.get(url)
        .bearer_auth("invalid-token")
        .send()
        .await
        .unwrap();
        
    println!("Status: {}", resp.status());
    let body: serde_json::Value = resp.json().await.unwrap();
    println!("Body: {:#}", body);
}
