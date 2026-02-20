# Quickstart: Request Recording and Playback

**Feature**: 004-recording-mode
**Date**: 2026-01-15

This guide shows you how to record APS API interactions and replay them for testing.

---

## Prerequisites

- raps-mock v0.3.0 or later
- Valid APS credentials (for recording only)
- An existing project that makes APS API calls

---

## Recording API Interactions

### Step 1: Start the mock server in record mode

```bash
# Record interactions from the official APS API
raps-mock --record https://developer.api.autodesk.com --output ./recordings/

# With verbose logging to see what's being captured
raps-mock --record https://developer.api.autodesk.com --output ./recordings/ --verbose
```

### Step 2: Point your application to the mock server

```bash
# Set environment variables (example for raps CLI)
export APS_BASE_URL=http://localhost:3000
export APS_CLIENT_ID=your-client-id
export APS_CLIENT_SECRET=your-client-secret

# Or for other applications, update the API base URL to http://localhost:3000
```

### Step 3: Run your application workflow

```bash
# Example: Execute a typical APS workflow through raps CLI
raps auth test                           # Authentication flow
raps bucket list                         # OSS operations
raps bucket create --key test-bucket     # More operations
raps translate start <urn>               # Model Derivative
```

### Step 4: Stop recording

Press `Ctrl+C` to stop the mock server. Your recordings are saved in `./recordings/`.

**Result**: You now have a `./recordings/` directory with JSON files for each API interaction.

---

## Replaying Recorded Sessions

### Step 1: Start the mock server in playback mode

```bash
# Play back previously recorded interactions
raps-mock --playback ./recordings/

# The server starts without needing APS credentials
```

### Step 2: Point your application to the mock server

```bash
export APS_BASE_URL=http://localhost:3000
# No credentials needed for playback!
```

### Step 3: Run your tests or application

```bash
# Your API calls now return recorded responses
raps bucket list    # Returns the recorded bucket list
```

---

## Library Usage

### Recording in Code

```rust
use raps_mock::{MockServer, MockServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server in record mode
    let config = MockServerConfig::record_mode(
        "https://developer.api.autodesk.com",
        "./recordings/my-session",
    );

    let server = MockServer::new(config).await?;
    server.start("0.0.0.0:3000").await?;

    Ok(())
}
```

### Playback in Tests

```rust
use raps_mock::TestServer;

#[tokio::test]
async fn test_bucket_workflow() {
    // Start server with recorded data
    let server = TestServer::start_playback("./fixtures/bucket-workflow")
        .await
        .expect("Failed to start playback server");

    // Make requests - they return recorded responses
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/oss/v2/buckets", server.url))
        .header("Authorization", "Bearer any-token") // Token doesn't matter in playback
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["items"].is_array());
}
```

---

## Configuration Options

### Recording Options

| Option | CLI Flag | Default | Description |
|--------|----------|---------|-------------|
| Backend URL | `--record <url>` | (required) | APS API to proxy to |
| Output directory | `--output <dir>` | `./recordings` | Where to save recordings |
| Anonymize | `--anonymize` | true | Redact credentials |
| Skip bodies | `--no-bodies` | false | Don't record request/response bodies |
| Max body size | `--max-body <bytes>` | 10MB | Skip bodies larger than this |

### Playback Options

| Option | CLI Flag | Default | Description |
|--------|----------|---------|-------------|
| Recordings directory | `--playback <dir>` | (required) | Directory with recordings |
| Matching mode | `--match <mode>` | strict | `strict`, `path-only`, or `flexible` |
| Ignore headers | `--ignore-header <name>` | none | Headers to ignore in matching |
| Sequential | `--sequential` | false | Match in recorded order |

---

## Anonymization

By default, recordings have sensitive data redacted:

**Before anonymization**:
```json
{
  "headers": {
    "Authorization": "Bearer eyJhbGciOiJSUzI1NiIs..."
  }
}
```

**After anonymization**:
```json
{
  "headers": {
    "Authorization": "[REDACTED]"
  }
}
```

### Default anonymized patterns

- Headers: `Authorization`, `Cookie`, `X-Ads-Token`
- Body fields: `access_token`, `refresh_token`, `client_secret`
- Query params: `access_token`, `api_key`

### Disable anonymization (not recommended)

```bash
raps-mock --record https://developer.api.autodesk.com --no-anonymize
```

---

## Flexible Matching

When requests have dynamic fields (timestamps, request IDs), use flexible matching:

```bash
# Ignore varying headers during playback
raps-mock --playback ./recordings/ \
  --match flexible \
  --ignore-header X-Request-Id \
  --ignore-header Date
```

---

## Sequential Playback

For workflows where request order matters:

```bash
# Enable sequential matching
raps-mock --playback ./recordings/ --sequential
```

This ensures:
1. First matching request returns first recorded response
2. Second matching request returns second recorded response
3. Out-of-order requests return an error

---

## Best Practices

### 1. Organize recordings by workflow

```
recordings/
├── auth-flow/           # Authentication recordings
├── bucket-crud/         # Bucket operations
├── translation-job/     # Model Derivative workflow
└── issues-workflow/     # ACC Issues operations
```

### 2. Commit recordings to version control

Recordings are portable and can be shared:
```bash
git add recordings/
git commit -m "Add APS API recordings for integration tests"
```

### 3. Use in CI/CD

```yaml
# GitHub Actions example
jobs:
  test:
    steps:
      - uses: actions/checkout@v4

      - name: Start mock server
        run: raps-mock --playback ./fixtures/api-recordings/ &

      - name: Run tests
        run: npm test
        env:
          APS_BASE_URL: http://localhost:3000
```

### 4. Keep recordings up to date

When APS APIs change, re-record:
```bash
# Re-record with latest API responses
raps-mock --record https://developer.api.autodesk.com --output ./recordings-new/
# Compare and update fixtures
```

---

## Troubleshooting

### "No matching recording found"

1. Check that the request method and path match exactly
2. Try `--match path-only` to relax matching
3. Inspect recordings to see what was captured

### "Recording file is invalid"

1. Check JSON syntax in the recording file
2. Verify the file wasn't corrupted during transfer
3. Re-record if necessary

### Requests are slow during recording

Recording adds minimal overhead (<50ms). If requests are slow:
1. Check network connectivity to the backend
2. The backend itself may be slow
3. Try `--no-bodies` to skip large body recording

---

## Next Steps

- See [data-model.md](./data-model.md) for the recording file format
- See [contracts/internal-api.md](./contracts/internal-api.md) for library API details
- Run `/speckit.tasks` to generate implementation tasks
