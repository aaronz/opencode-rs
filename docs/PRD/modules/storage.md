# PRD: storage Module

## Module Overview

**Module Name:** `storage`
**Type:** Core
**Source:** `/packages/opencode/src/storage/`

## Purpose

Persistent storage using SQLite with Drizzle ORM. Stores sessions, messages, settings, and other persistent data.

## Functionality

### Core Features

1. **Database Schema**
   - Sessions table
   - Messages table
   - Settings table
   - Auth credentials table

2. **Migration System**
   - `JsonMigration` - JSON-based migrations
   - Progress tracking
   - Migration rollback

3. **Query Interface**
   - Session queries
   - Message queries
   - Settings queries

### Database Schema

```sql
-- Sessions
CREATE TABLE sessions (
  id TEXT PRIMARY KEY,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  title TEXT,
  directory TEXT,
  provider TEXT,
  model TEXT,
  status TEXT DEFAULT 'active'
);

-- Messages
CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  tool_call_id TEXT,
  tool_name TEXT,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Settings
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);

-- Auth credentials
CREATE TABLE auth (
  provider TEXT PRIMARY KEY,
  type TEXT NOT NULL,
  key TEXT,
  access TEXT,
  refresh TEXT,
  metadata TEXT,
  expires_at INTEGER
);
```

### Key Files

- Database schema and definitions
- Migration system
- Query builders

### Database Client

```typescript
// Using Drizzle ORM with Bun-SQLite
import { drizzle } from "drizzle-orm/bun-sqlite"

const db = drizzle({ client: Database.Client() })

// Query sessions
const sessions = await db.select().from(sessionsTable)

// Insert message
await db.insert(messagesTable).values(message)
```

### Migration System

```typescript
// Migration runner
await JsonMigration.run(db, {
  progress: (event) => {
    console.log(`${event.current}/${event.total} - ${event.label}`)
  }
})

interface MigrationEvent {
  current: number
  total: number
  label: string
}
```

### Dependencies

- `drizzle-orm` - ORM
- `drizzle-orm/bun-sqlite` - Bun SQLite driver
- `better-sqlite3` - SQLite bindings

## Acceptance Criteria

1. Database schema is properly defined
2. Migrations run successfully on first start
3. Sessions are persisted correctly
4. Messages are stored with proper relations
5. Auth credentials are securely stored
6. Query performance is adequate

## Rust Implementation Guidance

The Rust equivalent should:
- Use `rusqlite` or `sqlx` for database
- Use `rusqlite` or `diesel` for ORM
- Implement migration system similar to JSON migration
- Use proper async/await patterns
- Consider using `serde` for serialization

## Test Design

### Unit Tests
- `sql_generation`: Validate that queries for session fetching, message insertion, and auth updates produce expected SQL.
- `migration_logic`: Test that migrations apply sequentially and fail safely on malformed updates.

### Integration Tests
- `sqlite_crud`: Create an in-memory SQLite database, apply migrations, and run full CRUD cycles for sessions, messages, and settings.
- `persistence_check`: Write to a file-backed SQLite database, close the connection, reopen it, and verify data integrity.

### Rust Specifics
- Use `rusqlite` with in-memory databases (`sqlite::memory:`) for fast, isolated tests.
- Use `tempfile` to test file-backed SQLite databases.
