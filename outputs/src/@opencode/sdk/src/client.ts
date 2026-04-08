/**
 * OpenCode SDK Client.
 *
 * Main client for interacting with the OpenCode REST API.
 */

import { ApiKeyAuth } from './auth';
import { SdkError } from './error';
import type { ToolDefinition, ToolResult } from './tools';
import { ToolCall } from './tools';
import type {
  SessionInfo,
  CreateSessionRequest,
  CreateSessionResponse,
  ForkSessionRequest,
  ForkSessionResponse,
  AddMessageRequest,
  AddMessageResponse,
} from './session';
import { SdkSession } from './session';

/**
 * Client configuration options.
 */
export interface ClientConfig {
  /** Base URL of the OpenCode API server. Defaults to http://localhost:8080/api */
  baseUrl?: string;
  /** API authentication. */
  auth?: ApiKeyAuth;
  /** Request timeout in milliseconds. Defaults to 30000 (30 seconds). */
  timeout?: number;
  /** Whether to skip TLS verification (for development only). */
  skipTlsVerification?: boolean;
}

/**
 * Builder for creating an OpenCodeClient.
 */
export class ClientBuilder {
  private config: Required<ClientConfig>;

  /**
   * Creates a new client builder with default configuration.
   */
  constructor() {
    this.config = {
      baseUrl: process.env.OPENCODE_BASE_URL ?? 'http://localhost:8080/api',
      auth: ApiKeyAuth.fromEnv(),
      timeout: 30000,
      skipTlsVerification: false,
    };
  }

  /**
   * Sets the base URL of the OpenCode API server.
   */
  baseUrl(url: string): this {
    this.config.baseUrl = url;
    return this;
  }

  /**
   * Sets the API key for authentication.
   */
  apiKey(key: string): this {
    this.config.auth = new ApiKeyAuth(key);
    return this;
  }

  /**
   * Sets the request timeout.
   */
  timeout(timeoutMs: number): this {
    this.config.timeout = timeoutMs;
    return this;
  }

  /**
   * Sets whether to skip TLS verification (development only).
   */
  skipTlsVerification(skip: boolean): this {
    this.config.skipTlsVerification = skip;
    return this;
  }

  /**
   * Builds the OpenCodeClient.
   */
  build(): OpenCodeClient {
    if (!this.config.auth.isSet) {
      throw SdkError.missingConfig('apiKey');
    }
    return new OpenCodeClient(this.config);
  }
}

/**
 * OpenCode SDK Client for programmatic access to OpenCode.
 *
 * @example
 * ```typescript
 * import { OpenCodeClient } from '@opencode/sdk';
 *
 * const client = new OpenCodeClient({
 *   apiKey: 'sk-your-api-key',
 * });
 *
 * const session = await client.createSession('Hello!');
 * console.log('Created session:', session.sessionId);
 * ```
 */
export class OpenCodeClient {
  private readonly _config: Required<ClientConfig>;
  private localSession: SdkSession | null = null;

  /**
   * Creates a new OpenCodeClient with the given configuration.
   */
  constructor(config: ClientConfig) {
    this._config = {
      baseUrl: config.baseUrl ?? process.env.OPENCODE_BASE_URL ?? 'http://localhost:8080/api',
      auth: config.auth ?? ApiKeyAuth.fromEnv(),
      timeout: config.timeout ?? 30000,
      skipTlsVerification: config.skipTlsVerification ?? false,
    };
  }

  /**
   * Creates a new client builder.
   */
  static builder(): ClientBuilder {
    return new ClientBuilder();
  }

  /**
   * Returns the client configuration.
   */
  get config(): Readonly<Required<ClientConfig>> {
    return this._config;
  }

  // ==================== Session API ====================

  /**
   * Creates a new session.
   *
   * @param initialPrompt - Optional initial prompt to start the session with.
   * @returns The created session response.
   */
  async createSession(initialPrompt?: string): Promise<CreateSessionResponse> {
    const request: CreateSessionRequest = {
      initialPrompt,
    };

    const response = await this.request<CreateSessionResponse>(
      'POST',
      '/sessions',
      request
    );

    return response;
  }

  /**
   * Gets a session by ID.
   *
   * @param sessionId - The session ID to retrieve.
   * @returns The full session object.
   */
  async getSession(sessionId: string): Promise<SdkSession> {
    const response = await this.request<SdkSession>(
      'GET',
      `/sessions/${sessionId}`
    );

    return response;
  }

  /**
   * Lists all sessions with pagination.
   *
   * @param limit - Maximum number of sessions to return.
   * @param offset - Number of sessions to skip.
   * @returns List of session info objects.
   */
  async listSessions(
    limit?: number,
    offset?: number
  ): Promise<SessionInfo[]> {
    const params = new URLSearchParams();
    if (limit !== undefined) params.set('limit', String(limit));
    if (offset !== undefined) params.set('offset', String(offset));

    const query = params.toString();
    const url = `/sessions${query ? `?${query}` : ''}`;

    interface ListResponse {
      items: SessionInfo[];
    }

    const response = await this.request<ListResponse>('GET', url);
    return response.items;
  }

  /**
   * Forks a session at the given message index.
   *
   * @param sessionId - The session ID to fork.
   * @param forkAtMessageIndex - The message index to fork at.
   * @returns The forked session response.
   */
  async forkSession(
    sessionId: string,
    forkAtMessageIndex: number
  ): Promise<ForkSessionResponse> {
    const request: ForkSessionRequest = {
      forkAtMessageIndex,
    };

    const response = await this.request<ForkSessionResponse>(
      'POST',
      `/sessions/${sessionId}/fork`,
      request
    );

    return response;
  }

  /**
   * Adds a message to a session.
   *
   * @param sessionId - The session ID.
   * @param content - The message content.
   * @param role - Optional message role (defaults to 'user').
   * @returns The add message response.
   */
  async addMessage(
    sessionId: string,
    content: string,
    role?: string
  ): Promise<AddMessageResponse> {
    const request: AddMessageRequest = {
      role: role ?? 'user',
      content,
    };

    const response = await this.request<AddMessageResponse>(
      'POST',
      `/sessions/${sessionId}/messages`,
      request
    );

    return response;
  }

  /**
   * Aborts a session.
   *
   * @param sessionId - The session ID to abort.
   */
  async abortSession(sessionId: string): Promise<void> {
    await this.request('POST', `/sessions/${sessionId}/abort`);
  }

  /**
   * Deletes a session.
   *
   * @param sessionId - The session ID to delete.
   */
  async deleteSession(sessionId: string): Promise<void> {
    await this.request('DELETE', `/sessions/${sessionId}`);
  }

  // ==================== Tools API ====================

  /**
   * Lists all available tools.
   *
   * @returns List of tool definitions.
   */
  async listTools(): Promise<ToolDefinition[]> {
    interface ListToolsResponse {
      items: ToolDefinition[];
    }

    const response = await this.request<ListToolsResponse>('GET', '/tools');
    return response.items;
  }

  /**
   * Executes a tool.
   *
   * @param toolCall - The tool call request.
   * @returns The tool execution result.
   */
  async executeTool(toolCall: ToolCall): Promise<ToolResult> {
    const response = await this.request<{
      id: string;
      toolName: string;
      success: boolean;
      result?: string;
      error?: string;
    }>('POST', '/tools/execute', {
      name: toolCall.name,
      arguments: toolCall.arguments,
    });

    const now = new Date();
    return {
      id: response.id,
      toolName: response.toolName,
      success: response.success,
      result: response.result,
      error: response.error,
      startedAt: now,
      completedAt: now,
    };
  }

  // ==================== Local Session (Offline Mode) ====================

  /**
   * Creates a local session (offline mode, no server required).
   *
   * @param initialPrompt - Optional initial prompt to start the session with.
   */
  async createLocalSession(initialPrompt?: string): Promise<void> {
    this.localSession = new SdkSession(crypto.randomUUID());

    if (initialPrompt) {
      this.localSession.addUserMessage(initialPrompt);
    }
  }

  /**
   * Gets the current local session.
   *
   * @returns The local session or null if not created.
   */
  async getLocalSession(): Promise<SdkSession | null> {
    return this.localSession;
  }

  /**
   * Adds a message to the local session.
   *
   * @param role - Message role ('user', 'assistant', 'system').
   * @param content - Message content.
   */
  async addLocalMessage(role: string, content: string): Promise<void> {
    if (!this.localSession) {
      throw SdkError.sessionError('no local session');
    }

    switch (role.toLowerCase()) {
      case 'user':
        this.localSession.addUserMessage(content);
        break;
      case 'assistant':
        this.localSession.addAssistantMessage(content);
        break;
      case 'system':
        this.localSession.addSystemMessage(content);
        break;
      default:
        this.localSession.messages.push({
          role: role as 'user' | 'assistant' | 'system',
          content,
        });
        this.localSession.updatedAt = new Date();
    }
  }

  // ==================== HTTP Request Helper ====================

  /**
   * Makes an HTTP request to the OpenCode API.
   */
  private async request<T>(
    method: string,
    path: string,
    body?: unknown
  ): Promise<T> {
    const url = `${this._config.baseUrl}${path}`;

    const headers: Record<string, string> = {
      Authorization: this._config.auth.authorizationHeader,
      'Content-Type': 'application/json',
    };

    const options: RequestInit = {
      method,
      headers,
    };

    if (body !== undefined) {
      options.body = JSON.stringify(body);
    }

    let response: Response;

    try {
      response = await fetch(url, {
        ...options,
        signal: AbortSignal.timeout(this._config.timeout),
      });
    } catch (error) {
      if (error instanceof Error) {
        if (error.name === 'TimeoutError') {
          throw SdkError.networkError(`Request timeout after ${this._config.timeout}ms`);
        }
        throw SdkError.networkError(`Request failed: ${error.message}`);
      }
      throw SdkError.networkError('Unknown network error');
    }

    if (response.status === 204) {
      return undefined as T;
    }

    let responseData: unknown;

    try {
      responseData = await response.json();
    } catch {
      const text = await response.text();
      throw SdkError.fromHttpStatus(
        response.status,
        text || response.statusText
      );
    }

    if (!response.ok) {
      const errorData = responseData as { error?: string; message?: string };
      const message = errorData?.error ?? errorData?.message ?? response.statusText;
      throw SdkError.fromHttpStatus(response.status, message);
    }

    return responseData as T;
  }
}
