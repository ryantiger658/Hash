# #ash REST API

All endpoints require an `Authorization: Bearer <api_key>` header.

Base URL: `http://<host>:3535/api`

---

## Authentication

```
Authorization: Bearer your-api-key-here
```

Returns `401 Unauthorized` if the key is missing or incorrect.

---

## Files

### List all files

```
GET /api/files
```

Returns metadata for every file in the vault (no content).

**Response `200 OK`:**
```json
[
  {
    "path": "notes/setup.md",
    "checksum": "a3f1...",
    "modified": 1710000000,
    "size": 1024
  }
]
```

---

### Get file contents

```
GET /api/files/{path}
```

Returns the raw file bytes. `path` is the vault-relative path with `/` separators.

**Response:** `200 OK` with raw bytes, or `404 Not Found`.

---

### Create or update a file

```
PUT /api/files/{path}
```

**Body:** raw file bytes (any content type).

**Response:** `204 No Content` on success.

---

### Delete a file

```
DELETE /api/files/{path}
```

**Response:** `204 No Content` on success, `404 Not Found` if the file doesn't exist.

---

## Sync

### Get vault snapshot

```
GET /api/sync/snapshot
```

Returns the server's current file list plus a server timestamp. Desktop clients use this to compute what needs to be pushed or pulled.

**Response `200 OK`:**
```json
{
  "server_time": 1710000000,
  "files": [
    {
      "path": "notes/setup.md",
      "checksum": "a3f1...",
      "modified": 1710000000,
      "size": 1024
    }
  ]
}
```

---

### Push changes

```
POST /api/sync/push
Content-Type: application/json
```

**Body:**
```json
{
  "upsert": [
    {
      "path": "notes/new-note.md",
      "content": "<base64-encoded file contents>",
      "modified": 1710000001
    }
  ],
  "delete": [
    { "path": "notes/old-note.md" }
  ]
}
```

**Response `200 OK`:**
```json
{
  "accepted": ["notes/new-note.md", "notes/old-note.md"],
  "rejected": []
}
```

A rejected item includes a `reason` string explaining why the write failed (e.g. path traversal attempt, disk error).
