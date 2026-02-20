# API Contracts: ACC Submittals

**Feature**: 001-acc-submittals
**Date**: 2026-01-15
**Type**: REST API (HTTP Endpoints)

## Overview

This document defines the HTTP API contracts for ACC Submittals mock endpoints. These endpoints match the ACC Submittals API structure.

---

## Base Path

```
/construction/submittals/v1/projects/{project_id}/submittals
```

---

## Endpoints

### List Submittals

**GET** `/construction/submittals/v1/projects/{project_id}/submittals`

List all submittals for a project.

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string | Yes | Project identifier |

**Response 200** (Success):
```json
{
  "results": [
    {
      "id": "submittal-uuid",
      "project_id": "project-001",
      "title": "HVAC Shop Drawings",
      "number": "SUB-001",
      "status": "open",
      "description": "...",
      "spec_section": "23 00 00",
      "due_date": "2026-02-15",
      "created_at": 1736931600000,
      "updated_at": 1736931600000
    }
  ]
}
```

**Response 404** (Project not found):
```json
{
  "code": "NOT_FOUND",
  "message": "Project not found"
}
```

---

### Create Submittal

**POST** `/construction/submittals/v1/projects/{project_id}/submittals`

Create a new submittal in a project.

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string | Yes | Project identifier |

**Request Body**:
```json
{
  "title": "Structural Steel Details",
  "number": "SUB-002",
  "description": "Detailed drawings for steel framing",
  "spec_section": "05 12 00",
  "due_date": "2026-03-01"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | Yes | Submittal title |
| `number` | string | No | User reference number |
| `description` | string | No | Detailed description |
| `spec_section` | string | No | Specification section |
| `due_date` | string | No | ISO 8601 date |

**Response 201** (Created):
```json
{
  "id": "new-submittal-uuid",
  "project_id": "project-001",
  "title": "Structural Steel Details",
  "number": "SUB-002",
  "status": "open",
  "description": "Detailed drawings for steel framing",
  "spec_section": "05 12 00",
  "due_date": "2026-03-01",
  "created_at": 1736931600000,
  "updated_at": 1736931600000
}
```

**Response 400** (Validation error):
```json
{
  "code": "BAD_REQUEST",
  "message": "title is required"
}
```

---

### Get Submittal

**GET** `/construction/submittals/v1/projects/{project_id}/submittals/{submittal_id}`

Get a single submittal by ID.

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string | Yes | Project identifier |
| `submittal_id` | string | Yes | Submittal identifier |

**Response 200** (Success):
```json
{
  "id": "submittal-uuid",
  "project_id": "project-001",
  "title": "HVAC Shop Drawings",
  "number": "SUB-001",
  "status": "submitted",
  "description": "Shop drawings for Level 1 HVAC systems",
  "spec_section": "23 00 00",
  "due_date": "2026-02-15",
  "created_at": 1736931600000,
  "updated_at": 1736935200000
}
```

**Response 404** (Not found):
```json
{
  "code": "NOT_FOUND",
  "message": "Submittal not found"
}
```

---

### Update Submittal

**PATCH** `/construction/submittals/v1/projects/{project_id}/submittals/{submittal_id}`

Update a submittal's fields.

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string | Yes | Project identifier |
| `submittal_id` | string | Yes | Submittal identifier |

**Request Body** (all fields optional):
```json
{
  "title": "Updated Title",
  "status": "approved",
  "description": "Updated description"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | No | New title |
| `number` | string | No | New reference number |
| `status` | string | No | New status |
| `description` | string | No | New description |
| `spec_section` | string | No | New spec section |
| `due_date` | string | No | New due date |

**Response 200** (Success):
```json
{
  "id": "submittal-uuid",
  "project_id": "project-001",
  "title": "Updated Title",
  "status": "approved",
  "description": "Updated description",
  "created_at": 1736931600000,
  "updated_at": 1736940000000
}
```

**Response 404** (Not found):
```json
{
  "code": "NOT_FOUND",
  "message": "Submittal not found"
}
```

---

### Delete Submittal

**DELETE** `/construction/submittals/v1/projects/{project_id}/submittals/{submittal_id}`

Delete a submittal.

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string | Yes | Project identifier |
| `submittal_id` | string | Yes | Submittal identifier |

**Response 204** (Success):
No content.

**Response 404** (Not found):
```json
{
  "code": "NOT_FOUND",
  "message": "Submittal not found"
}
```

---

## HTTP Headers

### Required Headers

| Header | Value | Description |
|--------|-------|-------------|
| `Content-Type` | `application/json` | For POST/PATCH requests |
| `Authorization` | `Bearer {token}` | Authentication (handled by middleware) |

### Response Headers

| Header | Value | Description |
|--------|-------|-------------|
| `Content-Type` | `application/json` | Response format |

---

## Error Responses

All error responses follow this format:

```json
{
  "code": "ERROR_CODE",
  "message": "Human-readable message"
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `BAD_REQUEST` | 400 | Invalid request body or missing required field |
| `UNAUTHORIZED` | 401 | Missing or invalid authentication |
| `NOT_FOUND` | 404 | Project or submittal not found |
| `INTERNAL_ERROR` | 500 | Server error |
