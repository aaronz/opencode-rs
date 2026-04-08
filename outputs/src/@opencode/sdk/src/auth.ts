/**
 * Authentication module for OpenCode SDK.
 *
 * Supports API key authentication for the OpenCode REST API.
 */

/**
 * API Key authentication credentials.
 */
export class ApiKeyAuth {
  private readonly _key: string;
  private readonly _prefix?: string;

  /**
   * Creates a new API key authentication with the given key.
   */
  constructor(key: string) {
    this._key = key;
    this._prefix = key.startsWith('sk-') ? 'sk-***' : undefined;
  }

  /**
   * Creates a new API key authentication from environment or given key.
   */
  static fromEnv(envKey?: string): ApiKeyAuth {
    return new ApiKeyAuth(envKey ?? process.env.OPENCODE_API_KEY ?? '');
  }

  /**
   * Returns the API key.
   */
  get key(): string {
    return this._key;
  }

  /**
   * Returns a masked version of the API key for logging.
   */
  get maskedKey(): string {
    return this._prefix ?? '***';
  }

  /**
   * Returns the Authorization header value.
   */
  get authorizationHeader(): string {
    return `Bearer ${this._key}`;
  }

  /**
   * Returns true if the API key is set.
   */
  get isSet(): boolean {
    return this._key.length > 0;
  }

  /**
   * Returns a string representation of the auth.
   */
  toString(): string {
    return `ApiKeyAuth(${this.maskedKey})`;
  }
}
