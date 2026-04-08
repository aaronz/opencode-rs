# @opencode/sdk

TypeScript SDK for OpenCode RS - Programmatic access to OpenCode capabilities.

## Features

- **Session Management**: Create, load, save, fork, and abort sessions
- **Tool Execution**: Execute and list available tools
- **Auth Integration**: API key authentication
- **Error Handling**: Structured error types with error codes (1xxx-9xxx)
- **Cross-Platform**: Works in Node.js and browser environments

## Installation

```bash
npm install @opencode/sdk
```

## Quick Start

```typescript
import { OpenCodeClient } from '@opencode/sdk';

const client = new OpenCodeClient({
  apiKey: 'your-api-key',
  baseUrl: 'http://localhost:8080/api',
});

const session = await client.createSession('Hello, world!');
console.log('Created session:', session.sessionId);
```

## Error Codes

| Range | Category |
|-------|----------|
| 1xxx | Authentication |
| 2xxx | Authorization |
| 3xxx | Provider |
| 4xxx | Tool |
| 5xxx | Session |
| 6xxx | Config |
| 7xxx | Validation |
| 9xxx | Internal |

## License

MIT
