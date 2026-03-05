// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! Admin state: account users, project users, projects, companies, and jobs.

use crate::state::db::Db;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Account-level user
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub first_name: String,
    pub last_name: String,
    pub status: String,
    pub role: String,
    pub company_id: Option<String>,
}

/// Admin-level project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminProject {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub status: String,
}

/// Project-level user assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUser {
    pub id: String,
    pub project_id: String,
    pub email: String,
    pub name: String,
    pub status: String,
    pub role_id: String,
}

/// HQ Company
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub trade: String,
}

/// Async job status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    pub id: String,
    pub status: String,
    pub progress: String,
    pub result: String,
}

/// Project template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTemplate {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub status: String,
}

pub struct AdminState {
    db: Arc<Db>,
}

impl AdminState {
    pub fn new(db: Arc<Db>) -> Self {
        let state = Self { db };
        state.seed();
        state
    }

    fn seed(&self) {
        let account = "mock-account-001";
        let conn = self.db.conn();

        // Seed account users
        let users = [
            ("user-001", "alice@example.com", "Alice Johnson", "Alice", "Johnson", "active", "project_admin", Some("comp-001")),
            ("user-002", "bob@example.com", "Bob Smith", "Bob", "Smith", "active", "project_user", Some("comp-002")),
        ];
        for (id, email, name, first, last, status, role, company_id) in users {
            conn.execute(
                "INSERT OR IGNORE INTO admin_users (id, account_id, email, name, first_name, last_name, status, role, company_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![id, account, email, name, first, last, status, role, company_id],
            )
            .expect("failed to seed admin user");
        }

        // Seed admin projects
        let projects = [
            ("proj-001", "Mock Project Alpha", "active"),
            ("proj-002", "Mock Project Beta", "active"),
        ];
        for (id, name, status) in projects {
            conn.execute(
                "INSERT OR IGNORE INTO admin_projects (id, account_id, name, status)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![id, account, name, status],
            )
            .expect("failed to seed admin project");
        }

        // Seed a project user assignment
        conn.execute(
            "INSERT OR IGNORE INTO project_users (id, project_id, email, name, status, role_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["user-001", "proj-001", "alice@example.com", "Alice Johnson", "active", "role-admin"],
        )
        .expect("failed to seed project user");

        // Seed companies
        let companies = [
            ("comp-001", "Mock Construction Co", "General Contractor"),
            ("comp-002", "Mock Engineering Ltd", "Electrical"),
        ];
        for (id, name, trade) in companies {
            conn.execute(
                "INSERT OR IGNORE INTO companies (id, account_id, name, trade)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![id, account, name, trade],
            )
            .expect("failed to seed company");
        }

        // Seed project templates
        let templates = [
            ("tmpl-001", "Default Template", "active"),
            ("tmpl-002", "Construction Template", "active"),
        ];
        for (id, name, status) in templates {
            conn.execute(
                "INSERT OR IGNORE INTO project_templates (id, account_id, name, status)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![id, account, name, status],
            )
            .expect("failed to seed project template");
        }
    }

    // ---- Account Users ----

    pub fn list_users(&self, account_id: &str) -> crate::error::Result<Vec<AccountUser>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, email, name, first_name, last_name, status, role, company_id
             FROM admin_users WHERE account_id = ?1",
        )?;
        let items = stmt.query_map(rusqlite::params![account_id], Self::row_to_user)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn add_user(
        &self,
        account_id: &str,
        email: String,
        name: String,
        role: String,
    ) -> crate::error::Result<AccountUser> {
        let id = format!("user-{}", uuid::Uuid::new_v4());
        let (first_name, last_name) = split_name(&name);
        let user = AccountUser {
            id: id.clone(),
            email,
            name,
            first_name,
            last_name,
            status: "active".to_string(),
            role,
            company_id: None,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO admin_users (id, account_id, email, name, first_name, last_name, status, role, company_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![user.id, account_id, user.email, user.name, user.first_name, user.last_name, user.status, user.role, user.company_id],
        )?;
        Ok(user)
    }

    pub fn search_user(&self, account_id: &str, email: &str) -> crate::error::Result<Option<AccountUser>> {
        let conn = self.db.conn();
        Ok(conn
            .query_row(
                "SELECT id, email, name, first_name, last_name, status, role, company_id
                 FROM admin_users WHERE account_id = ?1 AND email = ?2",
                rusqlite::params![account_id, email],
                Self::row_to_user,
            )
            .optional()?)
    }

    pub fn update_user(
        &self,
        account_id: &str,
        user_id: &str,
        name: Option<String>,
        role: Option<String>,
        status: Option<String>,
    ) -> crate::error::Result<Option<AccountUser>> {
        let conn = self.db.conn();
        let current = match conn
            .query_row(
                "SELECT id, email, name, first_name, last_name, status, role, company_id
                 FROM admin_users WHERE id = ?1 AND account_id = ?2",
                rusqlite::params![user_id, account_id],
                Self::row_to_user,
            )
            .optional()?
        {
            Some(c) => c,
            None => return Ok(None),
        };

        let new_name = name.unwrap_or(current.name.clone());
        let (first, last) = split_name(&new_name);
        let new_role = role.unwrap_or(current.role);
        let new_status = status.unwrap_or(current.status);

        conn.execute(
            "UPDATE admin_users SET name = ?1, first_name = ?2, last_name = ?3, role = ?4, status = ?5
             WHERE id = ?6 AND account_id = ?7",
            rusqlite::params![new_name, first, last, new_role, new_status, user_id, account_id],
        )?;

        Ok(Some(AccountUser {
            id: current.id,
            email: current.email,
            name: new_name,
            first_name: first,
            last_name: last,
            status: new_status,
            role: new_role,
            company_id: current.company_id,
        }))
    }

    pub fn delete_user(&self, account_id: &str, user_id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM admin_users WHERE id = ?1 AND account_id = ?2",
            rusqlite::params![user_id, account_id],
        )?;
        Ok(rows > 0)
    }

    pub fn import_users(
        &self,
        account_id: &str,
        users: Vec<(String, String, String)>,
    ) -> crate::error::Result<(Vec<AccountUser>, Vec<String>)> {
        let mut success = Vec::new();
        let mut failures = Vec::new();
        for (email, name, role) in users {
            match self.add_user(account_id, email.clone(), name, role) {
                Ok(user) => success.push(user),
                Err(_) => failures.push(email),
            }
        }
        Ok((success, failures))
    }

    // ---- Admin Projects ----

    pub fn list_projects(&self, account_id: &str) -> crate::error::Result<Vec<AdminProject>> {
        let conn = self.db.conn();
        
        let count: i64 = conn.query_row(
            "SELECT count(*) FROM admin_projects WHERE account_id = ?1",
            rusqlite::params![account_id],
            |row| row.get(0),
        )?;
        
        if count == 0 && account_id != "mock-account-001" {
            return Err(crate::error::MockError::NotFound(format!("Account {} not found", account_id)));
        }

        let mut stmt = conn.prepare(
            "SELECT id, account_id, name, status FROM admin_projects WHERE account_id = ?1",
        )?;
        let items = stmt.query_map(rusqlite::params![account_id], |row| {
            Ok(AdminProject {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                status: row.get(3)?,
            })
        })?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn create_project(
        &self,
        account_id: &str,
        name: String,
    ) -> crate::error::Result<AdminProject> {
        let id = format!("proj-{}", uuid::Uuid::new_v4());
        let project = AdminProject {
            id,
            account_id: account_id.to_string(),
            name,
            status: "active".to_string(),
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO admin_projects (id, account_id, name, status) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![project.id, project.account_id, project.name, project.status],
        )?;
        Ok(project)
    }

    pub fn get_project(&self, account_id: &str, project_id: &str) -> crate::error::Result<Option<AdminProject>> {
        let conn = self.db.conn();
        Ok(conn
            .query_row(
                "SELECT id, account_id, name, status FROM admin_projects WHERE id = ?1 AND account_id = ?2",
                rusqlite::params![project_id, account_id],
                |row| {
                    Ok(AdminProject {
                        id: row.get(0)?,
                        account_id: row.get(1)?,
                        name: row.get(2)?,
                        status: row.get(3)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn update_project(
        &self,
        account_id: &str,
        project_id: &str,
        name: Option<String>,
        status: Option<String>,
    ) -> crate::error::Result<Option<AdminProject>> {
        let current = match self.get_project(account_id, project_id)? {
            Some(c) => c,
            None => return Ok(None),
        };
        let new_name = name.unwrap_or(current.name);
        let new_status = status.unwrap_or(current.status);
        let conn = self.db.conn();
        conn.execute(
            "UPDATE admin_projects SET name = ?1, status = ?2 WHERE id = ?3 AND account_id = ?4",
            rusqlite::params![new_name, new_status, project_id, account_id],
        )?;
        Ok(Some(AdminProject {
            id: current.id,
            account_id: current.account_id,
            name: new_name,
            status: new_status,
        }))
    }

    // ---- Project Users ----

    pub fn list_project_users(&self, project_id: &str) -> crate::error::Result<Vec<ProjectUser>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, email, name, status, role_id FROM project_users WHERE project_id = ?1",
        )?;
        let items = stmt.query_map(rusqlite::params![project_id], Self::row_to_project_user)?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn get_project_user(&self, project_id: &str, user_id: &str) -> crate::error::Result<Option<ProjectUser>> {
        let conn = self.db.conn();
        Ok(conn
            .query_row(
                "SELECT id, project_id, email, name, status, role_id FROM project_users WHERE id = ?1 AND project_id = ?2",
                rusqlite::params![user_id, project_id],
                Self::row_to_project_user,
            )
            .optional()?)
    }

    pub fn add_project_user(
        &self,
        project_id: &str,
        user_id: String,
        email: String,
        name: String,
        role_id: String,
    ) -> crate::error::Result<ProjectUser> {
        let pu = ProjectUser {
            id: user_id,
            project_id: project_id.to_string(),
            email,
            name,
            status: "active".to_string(),
            role_id,
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT OR REPLACE INTO project_users (id, project_id, email, name, status, role_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![pu.id, pu.project_id, pu.email, pu.name, pu.status, pu.role_id],
        )?;
        Ok(pu)
    }

    pub fn update_project_user(
        &self,
        project_id: &str,
        user_id: &str,
        role_id: Option<String>,
    ) -> crate::error::Result<Option<ProjectUser>> {
        let current = match self.get_project_user(project_id, user_id)? {
            Some(c) => c,
            None => return Ok(None),
        };
        let new_role = role_id.unwrap_or(current.role_id);
        let conn = self.db.conn();
        conn.execute(
            "UPDATE project_users SET role_id = ?1 WHERE id = ?2 AND project_id = ?3",
            rusqlite::params![new_role, user_id, project_id],
        )?;
        Ok(Some(ProjectUser {
            role_id: new_role,
            ..current
        }))
    }

    pub fn delete_project_user(&self, project_id: &str, user_id: &str) -> crate::error::Result<bool> {
        let conn = self.db.conn();
        let rows = conn.execute(
            "DELETE FROM project_users WHERE id = ?1 AND project_id = ?2",
            rusqlite::params![user_id, project_id],
        )?;
        Ok(rows > 0)
    }

    // ---- Jobs ----

    pub fn create_job(&self, status: &str) -> crate::error::Result<JobInfo> {
        let id = format!("job-{}", uuid::Uuid::new_v4());
        let job = JobInfo {
            id: id.clone(),
            status: status.to_string(),
            progress: "100%".to_string(),
            result: "success".to_string(),
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO admin_jobs (id, status, progress, result) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![job.id, job.status, job.progress, job.result],
        )?;
        Ok(job)
    }

    pub fn get_job(&self, job_id: &str) -> crate::error::Result<Option<JobInfo>> {
        let conn = self.db.conn();
        Ok(conn
            .query_row(
                "SELECT id, status, progress, result FROM admin_jobs WHERE id = ?1",
                rusqlite::params![job_id],
                |row| {
                    Ok(JobInfo {
                        id: row.get(0)?,
                        status: row.get(1)?,
                        progress: row.get(2)?,
                        result: row.get(3)?,
                    })
                },
            )
            .optional()?)
    }

    // ---- Companies ----

    pub fn list_companies(&self, account_id: &str) -> crate::error::Result<Vec<Company>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, account_id, name, trade FROM companies WHERE account_id = ?1",
        )?;
        let items = stmt.query_map(rusqlite::params![account_id], |row| {
            Ok(Company {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                trade: row.get(3)?,
            })
        })?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    // ---- Project Templates ----

    pub fn list_templates(&self, account_id: &str) -> crate::error::Result<Vec<ProjectTemplate>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, account_id, name, status FROM project_templates WHERE account_id = ?1",
        )?;
        let items = stmt.query_map(rusqlite::params![account_id], |row| {
            Ok(ProjectTemplate {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                status: row.get(3)?,
            })
        })?;
        Ok(items.filter_map(|r| r.ok()).collect())
    }

    pub fn create_template(
        &self,
        account_id: &str,
        name: String,
    ) -> crate::error::Result<ProjectTemplate> {
        let id = format!("tmpl-{}", uuid::Uuid::new_v4());
        let t = ProjectTemplate {
            id,
            account_id: account_id.to_string(),
            name,
            status: "active".to_string(),
        };
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO project_templates (id, account_id, name, status) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![t.id, t.account_id, t.name, t.status],
        )?;
        Ok(t)
    }

    pub fn get_template(&self, account_id: &str, template_id: &str) -> crate::error::Result<Option<ProjectTemplate>> {
        let conn = self.db.conn();
        Ok(conn
            .query_row(
                "SELECT id, account_id, name, status FROM project_templates WHERE id = ?1 AND account_id = ?2",
                rusqlite::params![template_id, account_id],
                |row| {
                    Ok(ProjectTemplate {
                        id: row.get(0)?,
                        account_id: row.get(1)?,
                        name: row.get(2)?,
                        status: row.get(3)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn update_template(
        &self,
        account_id: &str,
        template_id: &str,
        name: Option<String>,
        status: Option<String>,
    ) -> crate::error::Result<Option<ProjectTemplate>> {
        let current = match self.get_template(account_id, template_id)? {
            Some(c) => c,
            None => return Ok(None),
        };
        let new_name = name.unwrap_or(current.name);
        let new_status = status.unwrap_or(current.status);
        let conn = self.db.conn();
        conn.execute(
            "UPDATE project_templates SET name = ?1, status = ?2 WHERE id = ?3 AND account_id = ?4",
            rusqlite::params![new_name, new_status, template_id, account_id],
        )?;
        Ok(Some(ProjectTemplate {
            id: current.id,
            account_id: current.account_id,
            name: new_name,
            status: new_status,
        }))
    }

    // ---- Row helpers ----

    fn row_to_user(row: &rusqlite::Row<'_>) -> rusqlite::Result<AccountUser> {
        Ok(AccountUser {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            first_name: row.get(3)?,
            last_name: row.get(4)?,
            status: row.get(5)?,
            role: row.get(6)?,
            company_id: row.get(7)?,
        })
    }

    fn row_to_project_user(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectUser> {
        Ok(ProjectUser {
            id: row.get(0)?,
            project_id: row.get(1)?,
            email: row.get(2)?,
            name: row.get(3)?,
            status: row.get(4)?,
            role_id: row.get(5)?,
        })
    }
}

fn split_name(name: &str) -> (String, String) {
    let parts: Vec<&str> = name.splitn(2, ' ').collect();
    let first = parts.first().unwrap_or(&"").to_string();
    let last = parts.get(1).unwrap_or(&"").to_string();
    (first, last)
}
