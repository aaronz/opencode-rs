/**
 * Session management for OpenCode SDK.
 *
 * Provides types and operations for managing OpenCode sessions.
 */

import type { MessageRole, SessionState } from './types';
export type { SessionState } from './types';

/**
 * Session information returned by the API.
 */
export interface SessionInfo {
  /** Unique session identifier. */
  id: string;
  /** Session creation timestamp (ISO 8601). */
  createdAt: string;
  /** Last update timestamp (ISO 8601). */
  updatedAt: string;
  /** Number of messages in the session. */
  messageCount: number;
  /** Preview of the last message (if available). */
  preview?: string;
}

/**
 * Session creation request.
 */
export interface CreateSessionRequest {
  /** Optional initial prompt to start the session with. */
  initialPrompt?: string;
}

/**
 * Session creation response from the API.
 */
export interface CreateSessionResponse {
  /** The created session ID. */
  sessionId: string;
  /** Creation timestamp. */
  createdAt: string;
  /** Session status. */
  status: string;
  /** Number of messages. */
  messageCount: number;
}

/**
 * Fork session request.
 */
export interface ForkSessionRequest {
  /** Message index to fork at. */
  forkAtMessageIndex: number;
}

/**
 * Fork session response.
 */
export interface ForkSessionResponse {
  /** New session ID after fork. */
  id: string;
  /** Parent session ID. */
  parentSessionId?: string;
  /** Message count in forked session. */
  messageCount: number;
}

/**
 * Add message request.
 */
export interface AddMessageRequest {
  /** Message role (user, assistant, system). */
  role?: string;
  /** Message content. */
  content: string;
}

/**
 * Add message response.
 */
export interface AddMessageResponse {
  /** Session ID. */
  sessionId: string;
  /** Total message count after adding. */
  messageCount: number;
}

/**
 * Session message type.
 */
export interface SessionMessage {
  /** Message role. */
  role: MessageRole;
  /** Message content. */
  content: string;
}

/**
 * Session summary information.
 */
export interface SessionSummary {
  /** Summary text. */
  summary: string;
  /** Summary creation timestamp. */
  createdAt: string;
}

/**
 * SDK Session representation for local use.
 */
export class SdkSession {
  /** Unique session identifier. */
  public id: string;
  /** Session messages. */
  public messages: SessionMessage[];
  /** Creation timestamp. */
  public createdAt: Date;
  /** Last update timestamp. */
  public updatedAt: Date;
  /** Current session state. */
  public state: SessionState;

  /**
   * Creates a new SDK session with the given ID.
   */
  constructor(id: string) {
    this.id = id;
    this.messages = [];
    this.createdAt = new Date();
    this.updatedAt = new Date();
    this.state = 'idle';
  }

  /**
   * Adds a user message to the session.
   */
  addUserMessage(content: string): void {
    this.messages.push({ role: 'user', content });
    this.updatedAt = new Date();
  }

  /**
   * Adds an assistant message to the session.
   */
  addAssistantMessage(content: string): void {
    this.messages.push({ role: 'assistant', content });
    this.updatedAt = new Date();
  }

  /**
   * Adds a system message to the session.
   */
  addSystemMessage(content: string): void {
    this.messages.push({ role: 'system', content });
    this.updatedAt = new Date();
  }

  /**
   * Returns the number of messages.
   */
  get messageCount(): number {
    return this.messages.length;
  }
}

/**
 * Parse role string to MessageRole.
 */
export function parseRole(role?: string): MessageRole {
  switch (role?.toLowerCase()) {
    case 'assistant':
      return 'assistant';
    case 'system':
      return 'system';
    default:
      return 'user';
  }
}
