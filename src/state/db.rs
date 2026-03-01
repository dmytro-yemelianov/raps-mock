// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! SQLite database wrapper for persistent state storage.

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

/// SQLite database wrapper shared across all state modules.
pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    /// Open an in-memory SQLite database (default, same behavior as DashMap).
    pub fn open_in_memory() -> Self {
        let conn = Connection::open_in_memory().expect("failed to open in-memory SQLite");
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init();
        db
    }

    /// Open a file-backed SQLite database for persistent storage.
    pub fn open_file(path: &Path) -> Self {
        let conn = Connection::open(path).expect("failed to open SQLite file");
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init();
        db
    }

    /// Get a lock on the underlying connection.
    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("db mutex poisoned")
    }

    fn init(&self) {
        let conn = self.conn();
        // Performance pragmas
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )
        .expect("failed to set pragmas");

        // Create all tables
        conn.execute_batch(SCHEMA)
            .expect("failed to initialize schema");
    }
}

const SCHEMA: &str = "
-- Auth
CREATE TABLE IF NOT EXISTS tokens (
    client_id TEXT PRIMARY KEY,
    access_token TEXT NOT NULL UNIQUE,
    token_type TEXT NOT NULL DEFAULT 'Bearer',
    expires_in INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    refresh_token TEXT,
    scope TEXT
);

-- OSS Buckets
CREATE TABLE IF NOT EXISTS buckets (
    bucket_key TEXT PRIMARY KEY,
    bucket_owner TEXT NOT NULL DEFAULT '',
    created_date INTEGER NOT NULL,
    policy_key TEXT NOT NULL,
    permissions TEXT NOT NULL DEFAULT '[]'
);

-- OSS Objects
CREATE TABLE IF NOT EXISTS objects (
    bucket_key TEXT NOT NULL,
    object_key TEXT NOT NULL,
    object_id TEXT NOT NULL,
    sha1 TEXT NOT NULL,
    size INTEGER NOT NULL,
    content_type TEXT NOT NULL,
    location TEXT NOT NULL,
    PRIMARY KEY (bucket_key, object_key)
);

-- Data Management
CREATE TABLE IF NOT EXISTS hubs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    region TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    hub_id TEXT NOT NULL,
    name TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_projects_hub_id ON projects(hub_id);

-- Model Derivative
CREATE TABLE IF NOT EXISTS translations (
    urn TEXT PRIMARY KEY,
    status TEXT NOT NULL,
    progress TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    metadata_json TEXT,
    object_tree_json TEXT,
    properties_json TEXT
);

-- ACC Issues
CREATE TABLE IF NOT EXISTS issues (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_issues_project_id ON issues(project_id);

CREATE TABLE IF NOT EXISTS issue_comments (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_issue_comments_issue_id ON issue_comments(issue_id);

-- Webhooks
CREATE TABLE IF NOT EXISTS webhooks (
    hook_id TEXT PRIMARY KEY,
    tenant TEXT NOT NULL,
    callback_url TEXT NOT NULL,
    event TEXT NOT NULL,
    system TEXT NOT NULL,
    scope_folder TEXT,
    scope_workflow TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_date TEXT NOT NULL
);

-- Design Automation
CREATE TABLE IF NOT EXISTS app_bundles (
    id TEXT PRIMARY KEY,
    engine TEXT NOT NULL,
    description TEXT NOT NULL,
    version INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS activities (
    id TEXT PRIMARY KEY,
    engine TEXT NOT NULL,
    description TEXT,
    version INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS work_items (
    id TEXT PRIMARY KEY,
    status TEXT NOT NULL,
    progress TEXT,
    activity_id TEXT NOT NULL
);

-- Reality Capture
CREATE TABLE IF NOT EXISTS photoscenes (
    photoscene_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    scene_type TEXT NOT NULL,
    convert_format TEXT NOT NULL,
    status TEXT NOT NULL,
    progress TEXT NOT NULL,
    progress_msg TEXT,
    scene_link TEXT
);

-- ACC RFIs
CREATE TABLE IF NOT EXISTS rfis (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_rfis_project_id ON rfis(project_id);

-- ACC Assets
CREATE TABLE IF NOT EXISTS assets (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_assets_project_id ON assets(project_id);

-- ACC Submittals
CREATE TABLE IF NOT EXISTS submittals (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_submittals_project_id ON submittals(project_id);

-- ACC Checklists
CREATE TABLE IF NOT EXISTS checklists (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_checklists_project_id ON checklists(project_id);

-- Data Management: Folders
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    parent_folder_id TEXT,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_modified_time TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_folders_project_id ON folders(project_id);
CREATE INDEX IF NOT EXISTS idx_folders_parent_id ON folders(parent_folder_id);

-- Data Management: Items
CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    folder_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_modified_time TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_items_project_id ON items(project_id);
CREATE INDEX IF NOT EXISTS idx_items_folder_id ON items(folder_id);

-- Data Management: Item Versions
CREATE TABLE IF NOT EXISTS item_versions (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL,
    version_number INTEGER NOT NULL,
    display_name TEXT NOT NULL,
    storage_size INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_item_versions_item_id ON item_versions(item_id);
";
