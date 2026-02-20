# Quickstart: State Persistence and Fixtures

**Feature**: 002-state-persistence
**Date**: 2026-01-15

This guide shows you how to use fixtures to save and restore mock server state.

---

## What are Fixtures?

Fixtures are YAML or JSON files that define pre-configured server state. They let you:
- Start the server with test data already loaded
- Share test scenarios with your team
- Create reproducible test environments

---

## Quick Start

### 1. Create a Fixture File

Create `fixtures/my-project.yaml`:

```yaml
version: "1.0"
metadata:
  name: "My Test Project"
  description: "Sample data for testing"

buckets:
  - bucket_key: "test-bucket"
    policy_key: "transient"

projects:
  - id: "project-001"
    name: "Test Project"
    hub_id: "hub-001"

issues:
  - id: "issue-001"
    project_id: "project-001"
    title: "First Issue"
    status: "open"
    created_at: 1736931600000
```

### 2. Start Server with Fixture

```bash
raps-mock --fixture fixtures/my-project.yaml --port 3000
```

### 3. Verify Data is Loaded

```bash
# List buckets - your test-bucket is there
curl http://localhost:3000/oss/v2/buckets

# List issues - your issue is there
curl http://localhost:3000/construction/issues/v1/projects/project-001/issues
```

---

## Creating Fixtures

### Manual Creation

Write YAML by hand for custom scenarios:

```yaml
version: "1.0"
buckets:
  - bucket_key: "my-bucket"
    policy_key: "persistent"
```

### Export from Running Server

1. Start the server and create resources via API
2. Export the current state:

```bash
# Future: via CLI command
raps-mock export-state --output my-fixture.yaml
```

---

## Using Fixtures in Tests

### Rust Integration Tests

```rust
use raps_mock::TestServer;

#[tokio::test]
async fn test_with_fixture() {
    // Start server with fixture
    let server = TestServer::start_with_fixture("fixtures/test-data.yaml")
        .await
        .expect("Failed to start server");

    // Data from fixture is immediately available
    let client = reqwest::Client::new();
    let resp = client
        .get(&format!("{}/oss/v2/buckets", server.url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["items"].as_array().unwrap().len() > 0);
}
```

### Inline Fixture in Tests

```rust
use raps_mock::{TestServer, Fixture};

#[tokio::test]
async fn test_with_inline_data() {
    // Create fixture programmatically
    let mut fixture = Fixture::empty();
    fixture.buckets.push(BucketInfo {
        bucket_key: "inline-bucket".to_string(),
        policy_key: "transient".to_string(),
        region: None,
        created_date: None,
    });

    let server = TestServer::start_with(fixture).await.unwrap();
    // ... test with inline-bucket
}
```

---

## Fixture Format Reference

### Required Fields

```yaml
version: "1.0"  # Required: schema version
```

### Optional Metadata

```yaml
metadata:
  name: "Fixture Name"
  description: "What this fixture provides"
  author: "Your Name"
  created_at: "2026-01-15T10:00:00Z"
```

### Resource Collections

All collections are optional. Include only what you need:

```yaml
buckets:
  - bucket_key: string      # Required
    policy_key: string      # Required
    region: string          # Optional: US, EMEA

objects:
  - bucket_key: string      # Required: must match a bucket
    object_key: string      # Required
    size: integer           # Required
    sha1: string            # Required
    content_type: string    # Optional

projects:
  - id: string              # Required
    name: string            # Required
    hub_id: string          # Required

issues:
  - id: string              # Required
    project_id: string      # Required
    title: string           # Required
    status: string          # Required
    description: string     # Optional
    created_at: integer     # Required: Unix ms

translations:
  - urn: string             # Required
    status: string          # Required: pending|inprogress|success|failed
    progress: string        # Optional

webhooks:
  - id: string              # Required
    callback_url: string    # Required
    event: string           # Required
    scope: object           # Required
```

---

## Sample Fixtures

### Empty (Reset State)

```yaml
version: "1.0"
```

### OSS Testing

```yaml
version: "1.0"
metadata:
  name: "OSS Test Data"

buckets:
  - bucket_key: "test-transient"
    policy_key: "transient"
  - bucket_key: "test-persistent"
    policy_key: "persistent"
    region: "US"

objects:
  - bucket_key: "test-persistent"
    object_key: "model.rvt"
    size: 1024000
    sha1: "abc123"
    content_type: "application/octet-stream"
```

### Full ACC Project

```yaml
version: "1.0"
metadata:
  name: "ACC Project"
  description: "Complete ACC setup with issues"

projects:
  - id: "acc-project-001"
    name: "Construction Project"
    hub_id: "b.hub-001"

issues:
  - id: "issue-001"
    project_id: "acc-project-001"
    title: "Foundation issue"
    status: "open"
    created_at: 1736931600000

  - id: "issue-002"
    project_id: "acc-project-001"
    title: "Electrical review"
    status: "in_review"
    created_at: 1736932000000
```

---

## Best Practices

### 1. Organize by Test Scenario

```
fixtures/
├── oss/
│   ├── empty-buckets.yaml
│   └── with-objects.yaml
├── acc/
│   ├── issues-workflow.yaml
│   └── submittals-workflow.yaml
└── full-project.yaml
```

### 2. Use Descriptive Metadata

```yaml
metadata:
  name: "Issue Workflow Test"
  description: "Contains issues in various statuses for workflow testing"
```

### 3. Keep Fixtures Small

Focus each fixture on one test scenario rather than loading everything.

### 4. Version Control Your Fixtures

```bash
git add fixtures/
git commit -m "Add fixtures for OSS testing"
```

---

## Troubleshooting

### "Version mismatch" Error

Your fixture is from an incompatible version. Update the `version` field or recreate the fixture.

### "File not found" Error

Check the path is correct relative to where you run raps-mock.

### Data Not Appearing

1. Check YAML syntax (use a YAML validator)
2. Ensure required fields are present
3. Check server logs with `--verbose`

---

## Next Steps

- See [data-model.md](./data-model.md) for the complete fixture schema
- See [contracts/api.md](./contracts/api.md) for the library API
- Run `/speckit.tasks` to generate implementation tasks
