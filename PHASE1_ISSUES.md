# Phase 1 Implementation Issues

## Overview
This file tracks the implementation tasks for raps-mock Phase 1: Complete ACC Module Coverage (v0.3.0)

---

## Issue 1: Implement Submittals State Manager

**Labels:** `enhancement`, `phase-1`, `acc`

**Description:**
Add state management for ACC Submittals API to enable mock responses for submittal operations.

**Tasks:**
- [ ] Create `src/state/submittals.rs`
- [ ] Define `SubmittalInfo` struct with fields: id, title, number, status, spec_section, due_date, created_at, updated_at
- [ ] Implement `SubmittalState` with methods:
  - `list_submittals(project_id: &str) -> Vec<SubmittalInfo>`
  - `get_submittal(project_id: &str, submittal_id: &str) -> Option<SubmittalInfo>`
  - `create_submittal(project_id: &str, request: CreateSubmittalRequest) -> SubmittalInfo`
  - `update_submittal(project_id: &str, submittal_id: &str, request: UpdateSubmittalRequest) -> Option<SubmittalInfo>`
- [ ] Add to `StateManager`
- [ ] Create router handlers
- [ ] Add unit tests
- [ ] Add integration tests with raps CLI `acc submittal` commands

**Acceptance Criteria:**
- `raps acc submittal list <project-id>` works against mock server
- `raps acc submittal create` creates submittal in mock state
- `raps acc submittal update` updates submittal in mock state

---

## Issue 2: Implement Checklists State Manager

**Labels:** `enhancement`, `phase-1`, `acc`

**Description:**
Add state management for ACC Checklists API including template support.

**Tasks:**
- [ ] Create `src/state/checklists.rs`
- [ ] Define structs:
  - `ChecklistInfo`: id, title, template_id, status, location, assignee_id, due_date
  - `ChecklistTemplate`: id, title, description, item_count
- [ ] Implement `ChecklistState` with methods:
  - `list_checklists(project_id: &str) -> Vec<ChecklistInfo>`
  - `get_checklist(project_id: &str, checklist_id: &str) -> Option<ChecklistInfo>`
  - `create_checklist(project_id: &str, request: CreateChecklistRequest) -> ChecklistInfo`
  - `update_checklist(project_id: &str, checklist_id: &str, request: UpdateChecklistRequest) -> Option<ChecklistInfo>`
  - `list_templates(project_id: &str) -> Vec<ChecklistTemplate>`
- [ ] Pre-populate with sample templates
- [ ] Add to `StateManager`
- [ ] Create router handlers
- [ ] Add unit tests
- [ ] Add integration tests

**Acceptance Criteria:**
- `raps acc checklist list` works against mock server
- `raps acc checklist templates` returns pre-populated templates
- `raps acc checklist create --template-id` works with mock templates

---

## Issue 3: Implement RFIs State Manager

**Labels:** `enhancement`, `phase-1`, `acc`

**Description:**
Add state management for ACC RFIs (Requests for Information) API.

**Tasks:**
- [ ] Create `src/state/rfis.rs`
- [ ] Define `RfiInfo` struct: id, title, number, status, question, answer, due_date, created_at, updated_at
- [ ] Implement `RfiState` with CRUD methods
- [ ] Add to `StateManager`
- [ ] Create router handlers
- [ ] Add unit tests
- [ ] Add integration tests with raps CLI `rfi` commands

**Acceptance Criteria:**
- `raps rfi list` works against mock server
- `raps rfi create` creates RFI in mock state
- `raps rfi update` can answer/close RFIs

---

## Issue 4: Implement Assets State Manager

**Labels:** `enhancement`, `phase-1`, `acc`

**Description:**
Add state management for ACC Assets API including categories.

**Tasks:**
- [ ] Create `src/state/assets.rs`
- [ ] Define structs:
  - `AssetInfo`: id, description, barcode, category_id, status_id, created_at, updated_at
  - `AssetCategory`: id, name, description
  - `AssetStatus`: id, name
- [ ] Implement `AssetState` with CRUD methods
- [ ] Pre-populate with sample categories and statuses
- [ ] Add to `StateManager`
- [ ] Create router handlers
- [ ] Add unit tests
- [ ] Add integration tests

**Acceptance Criteria:**
- `raps acc asset list` works against mock server
- `raps acc asset create` creates asset with category reference
- `raps acc asset update` can change status

---

## Issue 5: Implement Folders State Manager

**Labels:** `enhancement`, `phase-1`, `data-management`

**Description:**
Add state management for Data Management folders hierarchy.

**Tasks:**
- [ ] Create `src/state/folders.rs`
- [ ] Define `FolderInfo` struct: id, name, parent_id, project_id, created_at, updated_at
- [ ] Implement hierarchical folder structure
- [ ] Support root folder concept
- [ ] Implement `FolderState` with methods:
  - `list_folders(project_id: &str, folder_id: Option<&str>) -> Vec<FolderInfo>`
  - `get_folder(project_id: &str, folder_id: &str) -> Option<FolderInfo>`
  - `create_folder(project_id: &str, parent_id: Option<&str>, name: &str) -> FolderInfo`
- [ ] Add to `StateManager`
- [ ] Create router handlers
- [ ] Add unit tests

**Acceptance Criteria:**
- `raps folder list` navigates folder hierarchy
- Root folders are accessible
- Parent-child relationships are maintained

---

## Issue 6: Implement Items/Versions State Manager

**Labels:** `enhancement`, `phase-1`, `data-management`

**Description:**
Add state management for Data Management items and versions.

**Tasks:**
- [ ] Create `src/state/items.rs`
- [ ] Define structs:
  - `ItemInfo`: id, name, folder_id, project_id, type, created_at
  - `VersionInfo`: id, item_id, version_number, created_at, storage_urn
- [ ] Implement version lineage tracking
- [ ] Implement `ItemState` with methods:
  - `list_items(project_id: &str, folder_id: &str) -> Vec<ItemInfo>`
  - `get_item(project_id: &str, item_id: &str) -> Option<ItemInfo>`
  - `list_versions(project_id: &str, item_id: &str) -> Vec<VersionInfo>`
  - `get_version(project_id: &str, item_id: &str, version_id: &str) -> Option<VersionInfo>`
- [ ] Add to `StateManager`
- [ ] Create router handlers
- [ ] Add unit tests

**Acceptance Criteria:**
- `raps item versions` returns version history
- Items linked to folders correctly
- Versions linked to storage URNs

---

## Issue 7: Update StateManager Integration

**Labels:** `enhancement`, `phase-1`, `infrastructure`

**Description:**
Integrate all new state managers into the central StateManager.

**Tasks:**
- [ ] Update `src/state/manager.rs` to include:
  - `submittals: Arc<submittals::SubmittalState>`
  - `checklists: Arc<checklists::ChecklistState>`
  - `rfis: Arc<rfis::RfiState>`
  - `assets: Arc<assets::AssetState>`
  - `folders: Arc<folders::FolderState>`
  - `items: Arc<items::ItemState>`
- [ ] Update `src/state/mod.rs` exports
- [ ] Update state initialization in `StateManager::new()`
- [ ] Test all state managers work together

---

## Issue 8: Add Router Handlers for ACC Endpoints

**Labels:** `enhancement`, `phase-1`, `handlers`

**Description:**
Create Axum router handlers for all new ACC API endpoints.

**Tasks:**
- [ ] Create handlers in `src/handlers/`:
  - Submittals: GET/POST/PATCH `/construction/submittals/v1/projects/{project_id}/submittals`
  - Checklists: GET/POST/PATCH `/construction/checklists/v1/projects/{project_id}/checklists`
  - Templates: GET `/construction/checklists/v1/projects/{project_id}/templates`
  - RFIs: GET/POST/PATCH `/construction/rfis/v1/projects/{project_id}/rfis`
  - Assets: GET/POST/PATCH `/construction/assets/v1/projects/{project_id}/assets`
- [ ] Wire handlers to router in `src/server/router.rs`
- [ ] Handle path parameters correctly
- [ ] Return appropriate status codes

---

## Issue 9: Create Integration Test Suite

**Labels:** `testing`, `phase-1`

**Description:**
Create comprehensive integration tests that verify raps CLI commands work against raps-mock.

**Tasks:**
- [ ] Create `tests/integration/acc_submittals.rs`
- [ ] Create `tests/integration/acc_checklists.rs`
- [ ] Create `tests/integration/acc_rfis.rs`
- [ ] Create `tests/integration/acc_assets.rs`
- [ ] Create `tests/integration/data_management.rs`
- [ ] Use `TestServer` helper for all tests
- [ ] Test full CRUD lifecycle for each resource

---

## Issue 10: Update Documentation

**Labels:** `documentation`, `phase-1`

**Description:**
Update README and docs for new API coverage.

**Tasks:**
- [ ] Update README.md with new supported APIs
- [ ] Add API coverage table
- [ ] Document new CLI options (if any)
- [ ] Add usage examples for new endpoints
- [ ] Update library documentation in lib.rs

---

## Cleanup Tasks

### Remove Temporary Files

**Description:**
Remove temporary files left in repository root.

**Files to remove:**
- `tmpclaude-*.cwd` (all files matching this pattern)

```bash
rm -f tmpclaude-*-cwd
```

---

## Phase 1 Milestone Checklist

- [ ] All 6 new state managers implemented
- [ ] All handlers wired to router
- [ ] Unit tests passing (>80% coverage for new code)
- [ ] Integration tests passing
- [ ] Documentation updated
- [ ] README reflects new API coverage
- [ ] Version bumped to 0.3.0
- [ ] Changelog updated
- [ ] Release created

**Target Completion:** Q1 2026
