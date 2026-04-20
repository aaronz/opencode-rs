# PRD: acp Module

## Module Overview

**Module Name:** `acp`
**Type:** Integration
**Source:** `/packages/opencode/src/acp/`

## Purpose

Agent Communication Protocol - inter-agent messaging. Enables communication between multiple OpenCode agents.

## Functionality

### Core Features

1. **ACP Protocol**
   - Message passing between agents
   - Session sharing
   - Remote agent control

2. **ACP Commands**

   | Command | Description |
   |---------|-------------|
   | `status` | Get ACP status |
   | `handshake` | Perform handshake |
   | `connect` | Connect to agent |
   | `ack` | Acknowledge handshake |

3. **API Surface**

   ```typescript
   interface ACPClient {
     status(): Promise<ACPStatus>
     handshake(clientId: string, capabilities: string[]): Promise<HandshakeResponse>
     connect(url: string): Promise<void>
     ack(handshakeId: string): Promise<void>
   }

   interface ACPStatus {
     connected: boolean
     clientId?: string
     capabilities: string[]
   }
   ```

## Dependencies

- Server module for HTTP communication

## Acceptance Criteria

1. ACP client connects to agents
2. Handshake completes successfully
3. Messages are passed correctly
4. Status is reported accurately

## Rust Implementation Guidance

The Rust equivalent should:
- Use `reqwest` for HTTP
- Use `tokio` for async
- Implement proper JSON handling

## Test Design

### Unit Tests
- `protocol_messages`: Verify formatting of handshake, ack, and status messages.
- `state_machine`: Test the transition from disconnected -> handshaking -> connected.

### Integration Tests
- `local_agent_communication`: Spin up two instances of the ACP server locally and test end-to-end handshake and message passing.

### Rust Specifics
- Use `tokio::net::TcpStream` or IPC sockets for integration testing.
- Test task cancellation on disconnect.
