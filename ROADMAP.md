# raps-mock Evolution Roadmap

## Executive Summary

raps-mock is an APS API mock server that enables offline development, testing, and CI/CD integration without requiring live Autodesk Platform Services credentials. This document outlines the strategic evolution from the current v0.2.0 to a comprehensive, production-grade mock server.

---

## Current State Assessment (v0.2.0)

### What Exists Today

**Core Architecture:**
- Two modes: Stateless (fixed OpenAPI examples) and Stateful (in-memory state)
- Auto-generated routes from OpenAPI 3.0 specifications
- Axum-based HTTP server with tower middleware
- Custom handler registry for extending behavior
- Test server helper (`TestServer`) for integration tests

**State Managers:**
| State Module | Status | Coverage |
|--------------|--------|----------|
| Auth | ✅ Implemented | Token generation, validation |
| Buckets | ✅ Implemented | CRUD operations |
| Objects | ✅ Implemented | Upload metadata, listing |
| Projects | ✅ Implemented | Hubs, projects hierarchy |
| Translations | ✅ Implemented | Job status, manifests |
| Issues | ✅ Implemented | ACC Issues CRUD |
| Webhooks | ✅ Implemented | Subscription management |

**Known Gaps:**
- State persistence (marked TODO)
- Missing ACC modules: Submittals, Checklists, RFIs, Assets
- No Design Automation support
- No Reality Capture support
- No file upload simulation (binary handling)
- No translation progress simulation
- No webhook delivery simulation
- No rate limiting simulation
- No error injection/chaos testing
- No request recording/playback
- No Docker/container support
- No fixtures/scenario preloading

---

## Evolution Phases

### Phase 1: Complete ACC Module Coverage (v0.3.0)

**Goal:** Achieve feature parity with raps CLI for ACC APIs.

**New State Managers:**

```
src/state/
├── submittals.rs   # ACC Submittals CRUD
├── checklists.rs   # ACC Checklists + Templates
├── rfis.rs         # ACC RFIs CRUD
├── assets.rs       # ACC Assets + Categories
├── folders.rs      # Data Management folders
└── items.rs        # Data Management items/versions
```

**Implementation Tasks:**

1. **Submittals State** (`submittals.rs`)
   ```rust
   pub struct SubmittalInfo {
       pub id: String,
       pub title: String,
       pub number: Option<String>,
       pub status: String,
       pub spec_section: Option<String>,
       pub due_date: Option<String>,
       pub created_at: String,
       pub updated_at: String,
   }
   ```
   - List submittals by project
   - Create/update submittals
   - Status transitions

2. **Checklists State** (`checklists.rs`)
   ```rust
   pub struct ChecklistInfo {
       pub id: String,
       pub title: String,
       pub template_id: Option<String>,
       pub status: String,
       pub location: Option<String>,
       pub assignee_id: Option<String>,
       pub due_date: Option<String>,
   }
   
   pub struct ChecklistTemplate {
       pub id: String,
       pub title: String,
       pub description: Option<String>,
       pub item_count: u32,
   }
   ```
   - Template management
   - Checklist creation from templates
   - Status tracking

3. **RFIs State** (`rfis.rs`)
   ```rust
   pub struct RfiInfo {
       pub id: String,
       pub title: String,
       pub number: Option<String>,
       pub status: String,
       pub question: Option<String>,
       pub answer: Option<String>,
       pub due_date: Option<String>,
   }
   ```

4. **Assets State** (`assets.rs`)
   ```rust
   pub struct AssetInfo {
       pub id: String,
       pub description: Option<String>,
       pub barcode: Option<String>,
       pub category_id: Option<String>,
       pub status_id: Option<String>,
   }
   ```

5. **Folders/Items State** (`folders.rs`, `items.rs`)
   - Hierarchical folder structure
   - Item versions with lineage tracking
   - Storage quota simulation

**Deliverables:**
- All ACC module state managers
- Router handlers for each endpoint
- Unit tests for each state manager
- Integration tests against raps CLI

---

### Phase 2: State Persistence & Fixtures (v0.4.0)

**Goal:** Enable reproducible test scenarios and state persistence across server restarts.

**Features:**

1. **State Persistence**
   ```rust
   impl StateManager {
       /// Save complete state to JSON file
       pub fn save_to_file(&self, path: &Path) -> Result<()>;
       
       /// Load state from JSON file
       pub fn load_from_file(&self, path: &Path) -> Result<()>;
       
       /// Export state as portable fixture
       pub fn export_fixture(&self) -> Result<Fixture>;
   }
   ```

2. **Fixture Format**
   ```yaml
   # fixtures/hospital-project.yaml
   version: "1.0"
   name: "Hospital Project Scenario"
   description: "Pre-configured state for hospital construction project"
   
   buckets:
     - bucket_key: "hospital-models"
       policy_key: "persistent"
       objects:
         - object_key: "hospital-L1.rvt"
           size: 152000000
           sha1: "abc123..."
   
   projects:
     - id: "project-001"
       name: "Central Hospital"
       hub_id: "hub-001"
       folders:
         - id: "folder-001"
           name: "Project Files"
           parent_id: null
   
   issues:
     - id: "issue-001"
       project_id: "project-001"
       title: "Structural clash at Level 3"
       status: "open"
   
   translations:
     - urn: "dXJuOmFkc2sub2JqZWN0cy..."
       status: "success"
       progress: "complete"
       derivatives:
         - type: "svf2"
           status: "success"
   ```

3. **CLI Commands for Fixtures**
   ```bash
   # Load a fixture at startup
   raps-mock --fixture fixtures/hospital-project.yaml
   
   # Export current state to fixture
   raps-mock export-state --output current-state.yaml
   
   # List available fixtures
   raps-mock fixtures list
   ```

4. **Programmatic Fixture Loading**
   ```rust
   let server = MockServer::new(config).await?;
   server.load_fixture("fixtures/hospital-project.yaml").await?;
   ```

**Deliverables:**
- JSON/YAML state serialization
- Fixture file format specification
- CLI commands for fixture management
- Sample fixtures for common scenarios
- Documentation for fixture creation

---

### Phase 3: Simulation Capabilities (v0.5.0)

**Goal:** Realistic simulation of APS behavior including latency, failures, and async operations.

**Features:**

1. **Translation Progress Simulation**
   ```rust
   pub struct TranslationSimulator {
       /// Simulate translation taking time
       pub async fn start_translation(&self, urn: &str, config: TranslationConfig) -> String;
       
       /// Progress advances over time
       pub fn get_progress(&self, job_id: &str) -> TranslationProgress;
   }
   
   pub struct TranslationConfig {
       /// Duration in seconds to complete
       pub duration_secs: u64,
       /// Probability of failure (0.0 - 1.0)
       pub failure_probability: f64,
       /// Error type if failure
       pub failure_type: Option<TranslationError>,
   }
   ```

2. **Webhook Delivery Simulation**
   ```rust
   pub struct WebhookSimulator {
       /// Actually POST to registered webhook URLs
       pub async fn deliver_event(&self, event: WebhookEvent) -> DeliveryResult;
       
       /// Simulate retry behavior
       pub async fn simulate_delivery_with_retries(&self, event: WebhookEvent);
   }
   ```
   - Real HTTP delivery to registered URLs
   - Configurable retry behavior
   - Delivery status tracking

3. **Latency Injection**
   ```rust
   pub struct LatencyConfig {
       /// Base latency for all requests
       pub base_latency_ms: u64,
       /// Per-endpoint latency overrides
       pub endpoint_latencies: HashMap<String, u64>,
       /// Random jitter range
       pub jitter_ms: u64,
   }
   ```
   ```bash
   # Start with simulated latency
   raps-mock --latency 200 --jitter 50
   ```

4. **Error Injection**
   ```rust
   pub struct ChaosConfig {
       /// Probability of 500 errors (0.0 - 1.0)
       pub error_rate: f64,
       /// Specific endpoints to fail
       pub failing_endpoints: Vec<String>,
       /// Error codes to return
       pub error_codes: Vec<u16>,
   }
   ```
   ```bash
   # Simulate 5% random failures
   raps-mock --chaos-error-rate 0.05
   
   # Fail specific endpoint
   raps-mock --fail-endpoint "/oss/v2/buckets/{bucket_key}"
   ```

5. **Rate Limiting Simulation**
   ```rust
   pub struct RateLimitConfig {
       /// Requests per minute per endpoint
       pub limits: HashMap<String, u32>,
       /// Default limit
       pub default_limit: u32,
   }
   ```

**Deliverables:**
- Translation progress simulation with configurable timing
- Webhook delivery with actual HTTP calls
- Latency injection middleware
- Error injection middleware
- Rate limiting middleware
- Configuration via CLI and programmatic API

---

### Phase 4: Design Automation & Reality Capture (v0.6.0)

**Goal:** Complete coverage of all APS APIs supported by raps CLI.

**New State Managers:**

1. **Design Automation** (`da/`)
   ```
   src/state/da/
   ├── engines.rs      # AutoCAD, Revit, Inventor, 3ds Max, Fusion
   ├── appbundles.rs   # App bundle management
   ├── activities.rs   # Activity definitions
   ├── workitems.rs    # Work item execution simulation
   └── mod.rs
   ```
   
   ```rust
   pub struct WorkItemInfo {
       pub id: String,
       pub activity_id: String,
       pub status: WorkItemStatus,
       pub progress: String,
       pub report_url: Option<String>,
       pub stats: WorkItemStats,
   }
   
   pub enum WorkItemStatus {
       Pending,
       InProgress,
       Success,
       Failed,
       Cancelled,
   }
   ```
   
   - Engine listing (static data)
   - App bundle upload simulation
   - Activity CRUD
   - Work item execution simulation with progress
   - Report URL generation

2. **Reality Capture** (`reality.rs`)
   ```rust
   pub struct PhotosceneInfo {
       pub id: String,
       pub name: String,
       pub scene_type: String,
       pub status: PhotosceneStatus,
       pub progress: f64,
       pub output_formats: Vec<String>,
       pub result_url: Option<String>,
   }
   ```
   - Photoscene creation
   - Photo upload tracking
   - Processing simulation
   - Result URL generation

**Deliverables:**
- Full Design Automation API simulation
- Reality Capture photogrammetry simulation
- Work item execution with progress
- Engine/bundle/activity management
- Integration tests with raps DA commands

---

### Phase 5: Developer Experience (v0.7.0)

**Goal:** Make raps-mock the best-in-class APS development tool.

**Features:**

1. **Request Recording & Playback**
   ```rust
   pub struct RecordingConfig {
       pub output_dir: PathBuf,
       pub record_bodies: bool,
       pub anonymize_tokens: bool,
   }
   ```
   ```bash
   # Record all requests
   raps-mock --record ./recordings/
   
   # Playback recorded session
   raps-mock --playback ./recordings/session-001/
   ```

2. **Admin Dashboard**
   - Web UI for server management
   - Real-time request log
   - State inspection
   - Fixture management
   - Chaos controls
   
   ```bash
   raps-mock --admin-port 3001
   # Dashboard at http://localhost:3001
   ```

3. **OpenAPI Validation Mode**
   ```bash
   # Validate requests against OpenAPI schema
   raps-mock --validate-requests
   
   # Validate responses against OpenAPI schema
   raps-mock --validate-responses
   ```

4. **Multi-Region Simulation**
   ```rust
   pub struct RegionConfig {
       pub default_region: Region,
       pub enforce_region_constraints: bool,
   }
   ```
   - US/EMEA bucket isolation
   - Region mismatch errors
   - Cross-region operation blocking

5. **File Upload Simulation**
   ```rust
   pub struct UploadConfig {
       /// Store uploaded files to disk
       pub persist_uploads: bool,
       /// Directory for persisted files
       pub upload_dir: PathBuf,
       /// Max file size (for rejection testing)
       pub max_file_size: u64,
   }
   ```
   - Accept actual file uploads
   - Store to temporary directory
   - Chunked upload support
   - Resumable upload tracking

**Deliverables:**
- Request recording/playback
- Admin dashboard (web UI)
- OpenAPI validation middleware
- Multi-region simulation
- File upload handling
- Comprehensive documentation

---

### Phase 6: Deployment & Integration (v1.0.0)

**Goal:** Production-ready release with full ecosystem integration.

**Features:**

1. **Docker Support**
   ```dockerfile
   FROM rust:1.88-slim AS builder
   WORKDIR /app
   COPY . .
   RUN cargo build --release
   
   FROM debian:bookworm-slim
   COPY --from=builder /app/target/release/raps-mock /usr/local/bin/
   EXPOSE 3000
   CMD ["raps-mock"]
   ```
   
   ```bash
   # Pre-built image
   docker pull ghcr.io/dmytro-yemelianov/raps-mock:latest
   
   # Run with fixtures
   docker run -v ./fixtures:/fixtures \
     -p 3000:3000 \
     raps-mock --fixture /fixtures/project.yaml
   ```

2. **GitHub Actions Integration**
   ```yaml
   # .github/actions/raps-mock/action.yml
   name: 'raps-mock'
   description: 'Start APS mock server for testing'
   inputs:
     port:
       description: 'Server port'
       default: '3000'
     fixture:
       description: 'Fixture file to load'
       required: false
   runs:
     using: 'composite'
     steps:
       - run: |
           curl -L https://github.com/.../raps-mock/releases/latest/download/raps-mock-linux-x64 -o /usr/local/bin/raps-mock
           chmod +x /usr/local/bin/raps-mock
           raps-mock --port ${{ inputs.port }} --fixture ${{ inputs.fixture }} &
   ```
   
   Usage:
   ```yaml
   jobs:
     test:
       runs-on: ubuntu-latest
       steps:
         - uses: dmytro-yemelianov/raps-mock-action@v1
           with:
             fixture: ./test-fixtures/scenario.yaml
         
         - name: Run tests against mock
           run: npm test
           env:
             APS_BASE_URL: http://localhost:3000
   ```

3. **Testcontainers Support**
   ```rust
   use testcontainers::{clients::Cli, RunnableImage};
   use raps_mock_testcontainers::RapsMock;
   
   #[tokio::test]
   async fn test_with_container() {
       let docker = Cli::default();
       let container = docker.run(RapsMock::default());
       let port = container.get_host_port_ipv4(3000);
       
       // Test against http://localhost:{port}
   }
   ```

4. **VS Code Extension Integration**
   - Provide mock server configuration for VS Code REST Client
   - Generate `.env` files pointing to mock server
   - Fixture browser in VS Code

5. **Documentation Site**
   - Comprehensive user guide
   - API reference
   - Fixture format specification
   - Example repository
   - Video tutorials

**Deliverables:**
- Docker image on ghcr.io
- GitHub Action for CI/CD
- Testcontainers module (Rust)
- Pre-built binaries for all platforms
- Comprehensive documentation
- Example repository with common scenarios
- v1.0.0 release

---

## Implementation Timeline

| Phase | Version | Target | Duration |
|-------|---------|--------|----------|
| Phase 1: ACC Coverage | v0.3.0 | Q1 2026 | 4 weeks |
| Phase 2: Persistence | v0.4.0 | Q1 2026 | 3 weeks |
| Phase 3: Simulation | v0.5.0 | Q2 2026 | 5 weeks |
| Phase 4: DA & Reality | v0.6.0 | Q2 2026 | 4 weeks |
| Phase 5: DevEx | v0.7.0 | Q3 2026 | 6 weeks |
| Phase 6: Deployment | v1.0.0 | Q3 2026 | 4 weeks |

---

## Success Metrics

1. **Adoption**
   - GitHub stars
   - Crates.io downloads
   - Docker pulls

2. **Coverage**
   - % of APS APIs supported
   - % of raps CLI commands testable against mock

3. **Quality**
   - Test coverage
   - Issue response time
   - Documentation completeness

4. **Community**
   - External contributors
   - Fixture contributions
   - Integration examples

---

## Technical Decisions

### State Storage
- Use `DashMap` for concurrent access (current approach)
- JSON/YAML serialization via serde
- Optional SQLite backend for large state (future consideration)

### Async Model
- Tokio runtime (current)
- Axum for HTTP handling (current)
- Tower middleware for cross-cutting concerns

### Configuration
- CLI flags for common options
- YAML config file for complex scenarios
- Environment variables for CI/CD

### Testing Strategy
- Unit tests for each state manager
- Integration tests against raps CLI
- Smoke tests against OpenAPI specs
- Property-based tests for state consistency

---

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| OpenAPI spec drift | Medium | High | CI job to validate against upstream |
| Scope creep | Medium | Medium | Strict phase boundaries |
| Maintenance burden | Low | Medium | Community contributions, clear docs |
| Performance issues | Low | Low | Benchmarking, profiling |

---

## Next Actions

1. **Immediate (This Week)**
   - Create GitHub issues for Phase 1 tasks
   - Set up project board
   - Clean up temporary files in repo

2. **Short-term (This Month)**
   - Implement Submittals state manager
   - Implement Checklists state manager
   - Add integration tests

3. **Medium-term (This Quarter)**
   - Complete Phase 1 & 2
   - Begin Phase 3 simulation work
   - Publish blog post about raps-mock

---

## Appendix: Fixture Schema (Draft)

```yaml
# JSON Schema for fixtures
$schema: "http://json-schema.org/draft-07/schema#"
type: object
properties:
  version:
    type: string
    pattern: "^\\d+\\.\\d+$"
  name:
    type: string
  description:
    type: string
  
  auth:
    type: object
    properties:
      tokens:
        type: array
        items:
          type: object
          properties:
            access_token: { type: string }
            token_type: { type: string }
            expires_in: { type: integer }
            scope: { type: string }
  
  buckets:
    type: array
    items:
      type: object
      required: [bucket_key]
      properties:
        bucket_key: { type: string }
        policy_key: { type: string }
        region: { type: string, enum: [US, EMEA] }
        objects:
          type: array
          items:
            type: object
            required: [object_key]
            properties:
              object_key: { type: string }
              size: { type: integer }
              sha1: { type: string }
  
  projects:
    type: array
    items:
      type: object
      required: [id, name]
      properties:
        id: { type: string }
        name: { type: string }
        hub_id: { type: string }
  
  issues:
    type: array
    items:
      type: object
      required: [id, project_id, title]
      properties:
        id: { type: string }
        project_id: { type: string }
        title: { type: string }
        status: { type: string }
        description: { type: string }
  
  translations:
    type: array
    items:
      type: object
      required: [urn, status]
      properties:
        urn: { type: string }
        status: { type: string, enum: [pending, inprogress, success, failed] }
        progress: { type: string }
        derivatives:
          type: array
          items:
            type: object
            properties:
              type: { type: string }
              status: { type: string }

required: [version]
```
