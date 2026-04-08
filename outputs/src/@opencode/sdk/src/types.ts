/**
 * Shared types for the OpenCode SDK.
 */

/**
 * Message role in a conversation.
 */
export type MessageRole = 'user' | 'assistant' | 'system';

/**
 * A message in a session.
 */
export interface Message {
  role: MessageRole;
  content: string;
}

/**
 * Session state enumeration.
 */
export type SessionState = 'idle' | 'processing' | 'aborted' | 'completed' | 'error';

/**
 * Pagination parameters.
 */
export interface PaginationParams {
  limit?: number;
  offset?: number;
}

/**
 * List response wrapper.
 */
export interface ListResponse<T> {
  items: T[];
  limit?: number;
  offset?: number;
  count?: number;
  total?: number;
}
