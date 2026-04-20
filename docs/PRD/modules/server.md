# PRD: server Module

## Module Overview

**Module Name:** `server`
**Type:** Core
**Source:** `/packages/opencode/src/server/`

## Purpose

HTTP server and API routes for desktop/web modes and remote agent access. Provides REST API for controlling the agent remotely.

## Functionality

### Core Features

1. **HTTP Server**
   - Server initialization and lifecycle
   - Request routing
   - Middleware support
   - CORS handling

2. **API Routes**

   | Route | Handler | Description |
   |-------|---------|-------------|
   | `GET /health` | Health check | Server health status |
   | `POST /session` | Create session | Create new session |
   | `GET /session/:id` | Get session | Retrieve session |
   | `POST /session/:id/message` | Send message | Send message to agent |
   | `GET /session/:id/messages` | Get messages | Get session messages |
   | `POST /session/:id/compact` | Compact session | Trigger compaction |

3. **Workspace Management**
   - Workspace creation and deletion
   - Workspace metadata
   - Workspace file access

4. **Proxy Support**
   - HTTP proxy for API calls
   - Proxy authentication
   - Proxy retry logic

5. **Security**
   - `fence.ts` - Security fencing
   - Request validation
   - Rate limiting

6. **Discovery**
   - `mdns.ts` - mDNS service discovery
   - Local network discovery

### Key Files

| File | Purpose |
|------|---------|
| `server.ts` | Main server implementation |
| `routes/` | API route handlers |
| `proxy.ts` | HTTP proxy |
| `middleware.ts` | Server middleware |
| `workspace.ts` | Workspace management |
| `fence.ts` | Security fencing |
| `mdns.ts` | mDNS discovery |
| `error.ts` | Server errors |
| `event.ts` | Event handling |
| `projectors.ts` | Response projection |

### Server Configuration

```typescript
interface ServerConfig {
  hostname: string
  port: number
  cors: boolean
  auth?: {
    type: 'bearer' | 'basic'
    token: string
  }
  proxy?: {
    url: string
    auth?: { username: string, password: string }
  }
}
```

### Middleware Stack

1. Request logging
2. CORS headers
3. Authentication
4. Rate limiting
5. Request validation
6. Error handling

### Adapter Support

| Adapter | File | Purpose |
|---------|------|---------|
| Bun | `adapter.bun.ts` | Bun runtime |
| Node | `adapter.node.ts` | Node.js runtime |
| Generic | `adapter.ts` | Common interface |

## Dependencies

- `router` - HTTP router
- `middleware` - Middleware utilities
- `workspace` - Workspace module
- `session` - Session management

## Acceptance Criteria

1. Server starts and listens on configured port
2. All routes are functional
3. CORS handling works correctly
4. Authentication is enforced
5. Proxy support works
6. mDNS discovery functions

## Rust Implementation Guidance

The Rust equivalent should:
- Use `axum` or `actix-web` for HTTP
- Use `tower` for middleware
- Use `tokio` for async runtime
- Implement proper error handling
- Consider using `serde` for JSON
