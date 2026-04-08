/**
 * @opencode/sdk - TypeScript SDK for OpenCode RS
 *
 * @example
 * ```typescript
 * import { OpenCodeClient } from '@opencode/sdk';
 *
 * const client = new OpenCodeClient({
 *   apiKey: 'your-api-key',
 * });
 *
 * const session = await client.createSession('Hello, world!');
 * console.log('Created session:', session.sessionId);
 * ```
 */

// Core exports
export { OpenCodeClient, ClientConfig, ClientBuilder } from './client';
export { SdkError, SdkErrorCode, ErrorCategory } from './error';

// Session exports
export {
  SessionInfo,
  SdkSession,
  SessionMessage,
  CreateSessionRequest,
  CreateSessionResponse,
  ForkSessionRequest,
  ForkSessionResponse,
  AddMessageRequest,
  AddMessageResponse,
  parseRole,
} from './session';
export type { SessionState } from './session';

// Tools exports
export {
  ToolDefinition,
  ToolParameter,
  ToolResult,
  ToolCall,
  ToolExecutionResponse,
} from './tools';

// Auth exports
export { ApiKeyAuth } from './auth';

// Re-export Message type for convenience
export type { Message } from './types';
