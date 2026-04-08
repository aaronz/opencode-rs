/**
 * SDK Error types with error code mapping (FR-222, ERR-001-012).
 *
 * Error codes follow the unified numbering scheme:
 * - 1xxx: Authentication errors
 * - 2xxx: Authorization errors
 * - 3xxx: Provider errors
 * - 4xxx: Tool errors
 * - 5xxx: Session errors
 * - 6xxx: Config errors
 * - 7xxx: Validation errors
 * - 9xxx: Internal errors
 */

/**
 * Error codes by category.
 */
export enum SdkErrorCode {
  // Authentication (1xxx)
  AuthenticationFailed = 1001,
  InvalidApiKey = 1002,
  TokenExpired = 1003,

  // Authorization (2xxx)
  AccessDenied = 2001,
  InsufficientPermissions = 2002,

  // Provider (3xxx)
  ProviderError = 3001,
  ProviderNotFound = 3002,
  RateLimited = 3003,

  // Tool (4xxx)
  ToolNotFound = 4001,
  ToolExecutionFailed = 4002,
  ToolTimeout = 4003,
  InvalidToolArgs = 4004,

  // Session (5xxx)
  SessionNotFound = 5001,
  SessionError = 5002,
  SessionExpired = 5003,

  // Config (6xxx)
  ConfigError = 6001,
  MissingConfig = 6002,

  // Validation (7xxx)
  ValidationError = 7001,
  InvalidRequest = 7002,

  // Internal (9xxx)
  InternalError = 9001,
  NetworkError = 9002,
  IoError = 9003,
  ApiError = 9004,
}

/**
 * Error categories corresponding to error code ranges.
 */
export type ErrorCategory =
  | 'Authentication'
  | 'Authorization'
  | 'Provider'
  | 'Tool'
  | 'Session'
  | 'Config'
  | 'Validation'
  | 'Internal'
  | 'Unknown';

/**
 * Get error category from error code.
 */
export function getErrorCategory(code: number): ErrorCategory {
  if (code >= 1000 && code < 2000) return 'Authentication';
  if (code >= 2000 && code < 3000) return 'Authorization';
  if (code >= 3000 && code < 4000) return 'Provider';
  if (code >= 4000 && code < 5000) return 'Tool';
  if (code >= 5000 && code < 6000) return 'Session';
  if (code >= 6000 && code < 7000) return 'Config';
  if (code >= 7000 && code < 8000) return 'Validation';
  if (code >= 9000 && code < 10000) return 'Internal';
  return 'Unknown';
}

/**
 * Base SDK Error class.
 */
export class SdkError extends Error {
  public readonly code: number;
  public readonly category: ErrorCategory;
  public readonly details?: Record<string, unknown>;

  constructor(
    message: string,
    code: number,
    category?: ErrorCategory,
    details?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'SdkError';
    this.code = code;
    this.category = category ?? getErrorCategory(code);
    this.details = details;

    // Maintains proper stack trace for where error was thrown
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, SdkError);
    }
  }

  /**
   * Returns a string representation of the error.
   */
  toString(): string {
    return `[${this.code}] ${this.category}: ${this.message}`;
  }

  /**
   * Convert to JSON-serializable object.
   */
  toJSON(): Record<string, unknown> {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      category: this.category,
      details: this.details,
    };
  }

  // Factory methods for common errors

  /**
   * Creates an authentication failed error.
   */
  static authenticationFailed(detail: string): SdkError {
    return new SdkError(
      `Authentication failed: ${detail}`,
      SdkErrorCode.AuthenticationFailed,
      'Authentication'
    );
  }

  /**
   * Creates an invalid API key error.
   */
  static invalidApiKey(detail?: string): SdkError {
    return new SdkError(
      'Invalid API key',
      SdkErrorCode.InvalidApiKey,
      'Authentication',
      detail ? { detail } : undefined
    );
  }

  /**
   * Creates a token expired error.
   */
  static tokenExpired(detail?: string): SdkError {
    return new SdkError(
      'Token expired',
      SdkErrorCode.TokenExpired,
      'Authentication',
      detail ? { detail } : undefined
    );
  }

  /**
   * Creates an access denied error.
   */
  static accessDenied(detail: string): SdkError {
    return new SdkError(
      `Access denied: ${detail}`,
      SdkErrorCode.AccessDenied,
      'Authorization'
    );
  }

  /**
   * Creates an insufficient permissions error.
   */
  static insufficientPermissions(
    detail: string,
    requiredRole?: string
  ): SdkError {
    return new SdkError(
      'Insufficient permissions',
      SdkErrorCode.InsufficientPermissions,
      'Authorization',
      requiredRole ? { detail, requiredRole } : { detail }
    );
  }

  /**
   * Creates a tool not found error.
   */
  static toolNotFound(name: string): SdkError {
    return new SdkError(
      `Tool not found: ${name}`,
      SdkErrorCode.ToolNotFound,
      'Tool',
      { name }
    );
  }

  /**
   * Creates a tool execution failed error.
   */
  static toolExecutionFailed(tool: string, detail: string): SdkError {
    return new SdkError(
      `Tool execution failed: ${detail}`,
      SdkErrorCode.ToolExecutionFailed,
      'Tool',
      { tool, detail }
    );
  }

  /**
   * Creates a tool timeout error.
   */
  static toolTimeout(tool: string, timeoutMs: number): SdkError {
    return new SdkError(
      `Tool timeout: ${tool} (${timeoutMs}ms)`,
      SdkErrorCode.ToolTimeout,
      'Tool',
      { tool, timeoutMs }
    );
  }

  /**
   * Creates an invalid tool arguments error.
   */
  static invalidToolArgs(detail: string): SdkError {
    return new SdkError(
      `Invalid tool arguments: ${detail}`,
      SdkErrorCode.InvalidToolArgs,
      'Tool',
      { detail }
    );
  }

  /**
   * Creates a session not found error.
   */
  static sessionNotFound(id: string): SdkError {
    return new SdkError(
      `Session not found: ${id}`,
      SdkErrorCode.SessionNotFound,
      'Session',
      { id }
    );
  }

  /**
   * Creates a session error.
   */
  static sessionError(detail: string): SdkError {
    return new SdkError(
      `Session error: ${detail}`,
      SdkErrorCode.SessionError,
      'Session',
      { detail }
    );
  }

  /**
   * Creates a session expired error.
   */
  static sessionExpired(id: string): SdkError {
    return new SdkError(
      `Session expired: ${id}`,
      SdkErrorCode.SessionExpired,
      'Session',
      { id }
    );
  }

  /**
   * Creates a config error.
   */
  static configError(detail: string): SdkError {
    return new SdkError(
      `Configuration error: ${detail}`,
      SdkErrorCode.ConfigError,
      'Config',
      { detail }
    );
  }

  /**
   * Creates a missing config error.
   */
  static missingConfig(field: string): SdkError {
    return new SdkError(
      `Missing configuration: ${field}`,
      SdkErrorCode.MissingConfig,
      'Config',
      { field }
    );
  }

  /**
   * Creates a validation error.
   */
  static validationError(field: string, message: string): SdkError {
    return new SdkError(
      `Validation error: ${field} - ${message}`,
      SdkErrorCode.ValidationError,
      'Validation',
      { field, message }
    );
  }

  /**
   * Creates an invalid request error.
   */
  static invalidRequest(detail: string): SdkError {
    return new SdkError(
      `Invalid request: ${detail}`,
      SdkErrorCode.InvalidRequest,
      'Validation',
      { detail }
    );
  }

  /**
   * Creates a network error.
   */
  static networkError(detail: string): SdkError {
    return new SdkError(
      `Network error: ${detail}`,
      SdkErrorCode.NetworkError,
      'Internal',
      { detail }
    );
  }

  /**
   * Creates an internal error.
   */
  static internalError(detail: string): SdkError {
    return new SdkError(
      `Internal error: ${detail}`,
      SdkErrorCode.InternalError,
      'Internal',
      { detail }
    );
  }

  /**
   * Creates an API error from HTTP status.
   */
  static fromHttpStatus(status: number, message: string): SdkError {
    switch (status) {
      case 400:
        return new SdkError(
          `Invalid request: ${message}`,
          SdkErrorCode.InvalidRequest,
          'Validation',
          { status }
        );
      case 401:
      case 403:
        return new SdkError(
          `Authentication failed: ${message}`,
          SdkErrorCode.AuthenticationFailed,
          'Authentication',
          { status }
        );
      case 404:
        return new SdkError(
          `Session not found: ${message}`,
          SdkErrorCode.SessionNotFound,
          'Session',
          { status }
        );
      case 422:
        return new SdkError(
          `Validation error: ${message}`,
          SdkErrorCode.ValidationError,
          'Validation',
          { status }
        );
      case 429:
        return new SdkError(
          `Rate limited: ${message}`,
          SdkErrorCode.RateLimited,
          'Provider',
          { status }
        );
      case 500:
      case 502:
      case 503:
      case 504:
        return new SdkError(
          `Internal error: ${message}`,
          SdkErrorCode.InternalError,
          'Internal',
          { status }
        );
      default:
        return new SdkError(
          `API error: ${status} - ${message}`,
          SdkErrorCode.ApiError,
          'Internal',
          { status }
        );
    }
  }
}

/**
 * Result type alias for SDK operations.
 */
export type SdkResult<T> = T | SdkError;
