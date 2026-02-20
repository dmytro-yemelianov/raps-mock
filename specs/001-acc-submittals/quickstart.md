# Quickstart: ACC Submittals Mock

**Feature**: 001-acc-submittals
**Date**: 2026-01-15

This guide shows you how to use the ACC Submittals mock endpoints.

---

## Prerequisites

- raps-mock running in stateful mode (default)
- An HTTP client (curl, Postman, or your application)

---

## Start the Mock Server

```bash
raps-mock --port 3000

# Or with verbose logging
raps-mock --port 3000 --verbose
```

---

## Using the Submittals API

### List Submittals

```bash
curl -X GET "http://localhost:3000/construction/submittals/v1/projects/project-001/submittals" \
  -H "Authorization: Bearer mock-token"
```

**Response** (empty project):
```json
{
  "results": []
}
```

### Create a Submittal

```bash
curl -X POST "http://localhost:3000/construction/submittals/v1/projects/project-001/submittals" \
  -H "Authorization: Bearer mock-token" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "HVAC Shop Drawings",
    "number": "SUB-001",
    "description": "Shop drawings for Level 1 HVAC systems",
    "spec_section": "23 00 00",
    "due_date": "2026-02-15"
  }'
```

**Response**:
```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "project_id": "project-001",
  "title": "HVAC Shop Drawings",
  "number": "SUB-001",
  "status": "open",
  "description": "Shop drawings for Level 1 HVAC systems",
  "spec_section": "23 00 00",
  "due_date": "2026-02-15",
  "created_at": 1736931600000,
  "updated_at": 1736931600000
}
```

### Get a Submittal

```bash
curl -X GET "http://localhost:3000/construction/submittals/v1/projects/project-001/submittals/a1b2c3d4-e5f6-7890-abcd-ef1234567890" \
  -H "Authorization: Bearer mock-token"
```

### Update a Submittal

```bash
curl -X PATCH "http://localhost:3000/construction/submittals/v1/projects/project-001/submittals/a1b2c3d4-e5f6-7890-abcd-ef1234567890" \
  -H "Authorization: Bearer mock-token" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "submitted"
  }'
```

### Delete a Submittal

```bash
curl -X DELETE "http://localhost:3000/construction/submittals/v1/projects/project-001/submittals/a1b2c3d4-e5f6-7890-abcd-ef1234567890" \
  -H "Authorization: Bearer mock-token"
```

---

## Using with raps CLI

Configure raps CLI to use the mock server:

```bash
export APS_BASE_URL=http://localhost:3000
export APS_CLIENT_ID=mock-client
export APS_CLIENT_SECRET=mock-secret

# Then use raps commands (when submittal commands are implemented)
raps submittal list --project project-001
raps submittal create --project project-001 --title "New Submittal"
```

---

## Library Usage (Tests)

```rust
use raps_mock::TestServer;

#[tokio::test]
async fn test_submittal_workflow() {
    // Start mock server
    let server = TestServer::start_default().await.unwrap();
    let client = reqwest::Client::new();

    // Create a submittal
    let create_resp = client
        .post(&format!(
            "{}/construction/submittals/v1/projects/project-001/submittals",
            server.url
        ))
        .header("Authorization", "Bearer mock-token")
        .json(&serde_json::json!({
            "title": "Test Submittal",
            "spec_section": "01 00 00"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(create_resp.status(), 201);

    let submittal: serde_json::Value = create_resp.json().await.unwrap();
    let submittal_id = submittal["id"].as_str().unwrap();

    // Update status
    let update_resp = client
        .patch(&format!(
            "{}/construction/submittals/v1/projects/project-001/submittals/{}",
            server.url, submittal_id
        ))
        .header("Authorization", "Bearer mock-token")
        .json(&serde_json::json!({
            "status": "approved"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(update_resp.status(), 200);

    let updated: serde_json::Value = update_resp.json().await.unwrap();
    assert_eq!(updated["status"], "approved");
}
```

---

## Common Workflows

### 1. Submit for Review Workflow

```bash
# Create submittal
curl -X POST ".../submittals" -d '{"title": "Steel Details"}'

# Update to submitted
curl -X PATCH ".../submittals/{id}" -d '{"status": "submitted"}'

# Later: Approve
curl -X PATCH ".../submittals/{id}" -d '{"status": "approved"}'
```

### 2. Batch Creation

```bash
# Create multiple submittals
for i in 1 2 3; do
  curl -X POST "http://localhost:3000/construction/submittals/v1/projects/project-001/submittals" \
    -H "Authorization: Bearer mock-token" \
    -H "Content-Type: application/json" \
    -d "{\"title\": \"Submittal $i\"}"
done

# List all
curl "http://localhost:3000/construction/submittals/v1/projects/project-001/submittals" \
  -H "Authorization: Bearer mock-token"
```

---

## Notes

- **State is in-memory**: Submittals are lost when the server stops
- **Any project ID works**: The mock doesn't validate project existence
- **Status is flexible**: Any string is accepted as a status value
- **Authentication**: The mock accepts any Bearer token

---

## Next Steps

- See [data-model.md](./data-model.md) for the complete data structure
- See [contracts/api.md](./contracts/api.md) for full API documentation
- Run `/speckit.tasks` to generate implementation tasks
