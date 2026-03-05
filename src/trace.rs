// SPDX-License-Identifier: Apache-2.0
// Copyright 2024-2025 Dmytro Yemelianov

//! API call trace recorder for use in integration tests.

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

/// A single recorded API call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiCall {
    pub method: String,
    pub path: String,
}

/// Thread-safe recorder that captures API calls made against the mock server.
/// Clone it freely — all clones share the same underlying list.
#[derive(Debug, Clone, Default)]
pub struct TraceRecorder {
    calls: Arc<Mutex<Vec<ApiCall>>>,
}

impl TraceRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one API call. Called by middleware on every write request.
    pub fn record(&self, call: ApiCall) {
        self.calls.lock().unwrap().push(call);
    }

    /// Return a snapshot of all calls recorded so far.
    pub fn calls(&self) -> Vec<ApiCall> {
        self.calls.lock().unwrap().clone()
    }

    /// Clear the recorded calls (useful between test phases).
    pub fn clear(&self) {
        self.calls.lock().unwrap().clear();
    }

    /// Assert exactly `count` calls have been recorded.
    pub fn assert_call_count(&self, count: usize) {
        let calls = self.calls();
        assert_eq!(
            calls.len(),
            count,
            "Expected {count} calls, got {}.\nCalls: {calls:#?}",
            calls.len()
        );
    }

    /// Assert at least one call was made to `method` on a path containing `path_fragment`.
    pub fn assert_called_with(&self, method: &str, path_fragment: &str) {
        let calls = self.calls();
        assert!(
            calls.iter().any(|c| {
                c.method.eq_ignore_ascii_case(method) && c.path.contains(path_fragment)
            }),
            "Expected {method} {path_fragment} — not found.\nCalls: {calls:#?}"
        );
    }

    /// Assert NO call was made to `method` on a path containing `path_fragment`.
    pub fn assert_not_called_with(&self, method: &str, path_fragment: &str) {
        let calls = self.calls();
        assert!(
            !calls.iter().any(|c| {
                c.method.eq_ignore_ascii_case(method) && c.path.contains(path_fragment)
            }),
            "Expected {method} {path_fragment} NOT to be called.\nCalls: {calls:#?}"
        );
    }

    /// Return all POST calls whose path contains `path_fragment`.
    pub fn post_calls_to(&self, path_fragment: &str) -> Vec<ApiCall> {
        self.calls()
            .into_iter()
            .filter(|c| c.method.eq_ignore_ascii_case("POST") && c.path.contains(path_fragment))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_query() {
        let rec = TraceRecorder::new();
        rec.record(ApiCall { method: "POST".into(), path: "/projects/p1/users".into() });
        rec.record(ApiCall { method: "GET".into(), path: "/projects/p1/users/u1".into() });

        rec.assert_call_count(2);
        rec.assert_called_with("POST", "/projects/p1/users");
        rec.assert_called_with("GET", "/users/u1");
    }

    #[test]
    fn test_assert_not_called() {
        let rec = TraceRecorder::new();
        rec.assert_not_called_with("DELETE", "/projects");
    }

    #[test]
    fn test_post_calls_to() {
        let rec = TraceRecorder::new();
        rec.record(ApiCall { method: "POST".into(), path: "/projects/p1/users".into() });
        rec.record(ApiCall { method: "POST".into(), path: "/projects/p2/users".into() });
        rec.record(ApiCall { method: "PATCH".into(), path: "/projects/p1/users/u1".into() });

        assert_eq!(rec.post_calls_to("/users").len(), 2);
        assert_eq!(rec.post_calls_to("p2").len(), 1);
    }

    #[test]
    fn test_clear() {
        let rec = TraceRecorder::new();
        rec.record(ApiCall { method: "POST".into(), path: "/foo".into() });
        rec.clear();
        rec.assert_call_count(0);
    }

    #[test]
    fn test_clone_shares_state() {
        let rec = TraceRecorder::new();
        let rec2 = rec.clone();
        rec2.record(ApiCall { method: "DELETE".into(), path: "/bar".into() });
        rec.assert_call_count(1);
    }
}
