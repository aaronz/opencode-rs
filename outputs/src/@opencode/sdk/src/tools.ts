/**
 * Tool management for OpenCode SDK.
 *
 * Provides types and operations for executing and listing tools.
 */

/**
 * Tool parameter definition.
 */
export interface ToolParameter {
  /** Parameter name. */
  name: string;
  /** Parameter description. */
  description: string;
  /** Whether the parameter is required. */
  required: boolean;
  /** JSON Schema for the parameter. */
  schema?: Record<string, unknown>;
}

/**
 * Tool definition from the registry.
 */
export interface ToolDefinition {
  /** Tool name. */
  name: string;
  /** Tool description. */
  description: string;
  /** Tool parameters. */
  parameters: ToolParameter[];
}

/**
 * Tool execution result.
 */
export interface ToolResult {
  /** Result ID. */
  id: string;
  /** Name of the executed tool. */
  toolName: string;
  /** Whether the execution was successful. */
  success: boolean;
  /** Result content (if successful). */
  result?: string;
  /** Error message (if failed). */
  error?: string;
  /** Execution start time. */
  startedAt: Date;
  /** Execution completion time. */
  completedAt: Date;
}

export namespace ToolResult {
  /**
   * Creates a successful tool result.
   */
  export function success(
    toolName: string,
    result: string,
    id?: string
  ): ToolResult {
    const now = new Date();
    return {
      id: id ?? crypto.randomUUID(),
      toolName,
      success: true,
      result,
      error: undefined,
      startedAt: now,
      completedAt: now,
    };
  }

  /**
   * Creates a failed tool result.
   */
  export function failure(
    toolName: string,
    error: string,
    id?: string
  ): ToolResult {
    const now = new Date();
    return {
      id: id ?? crypto.randomUUID(),
      toolName,
      success: false,
      result: undefined,
      error,
      startedAt: now,
      completedAt: now,
    };
  }
}

/**
 * Tool call request.
 */
export interface ToolCall {
  /** Tool name to execute. */
  name: string;
  /** Tool arguments as JSON object. */
  arguments: Record<string, unknown>;
}

export namespace ToolCall {
  /**
   * Creates a new tool call with the given name and arguments.
   */
  export function create(
    name: string,
    arguments_: Record<string, unknown> = {}
  ): ToolCall {
    return { name, arguments: arguments_ };
  }

  /**
   * Creates a tool call with a single argument.
   */
  export function withArg(
    name: string,
    key: string,
    value: unknown
  ): ToolCall {
    return { name, arguments: { [key]: value } };
  }
}

/**
 * Tool execution response from the API.
 */
export interface ToolExecutionResponse {
  /** Execution result ID. */
  id: string;
  /** Tool name. */
  toolName: string;
  /** Whether successful. */
  success: boolean;
  /** Result or error. */
  result?: string;
  /** Error message if failed. */
  error?: string;
}
