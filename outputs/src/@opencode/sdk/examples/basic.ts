/**
 * @opencode/sdk - Basic Usage Example
 *
 * This example demonstrates basic usage of the OpenCode TypeScript SDK.
 */

import {
  OpenCodeClient,
  SdkError,
  ToolCall,
} from '../src/index';

/**
 * Example: Basic session creation and tool execution.
 */
async function basicExample(): Promise<void> {
  console.log('=== OpenCode SDK Basic Example ===\n');

  // Create client with API key
  const client = new OpenCodeClient({
    baseUrl: process.env.OPENCODE_BASE_URL ?? 'http://localhost:8080/api',
    auth: { key: process.env.OPENCODE_API_KEY ?? 'sk-your-api-key' },
    timeout: 30000,
  });

  try {
    // Create a new session
    console.log('1. Creating session...');
    const session = await client.createSession('Hello, OpenCode!');
    console.log(`   Created session: ${session.sessionId}`);
    console.log(`   Status: ${session.status}`);
    console.log(`   Message count: ${session.messageCount}\n`);

    // Add a message
    console.log('2. Adding message...');
    const addResult = await client.addMessage(
      session.sessionId,
      'Can you read the file at /tmp/test.txt?',
      'user'
    );
    console.log(`   Session: ${addResult.sessionId}`);
    console.log(`   Total messages: ${addResult.messageCount}\n`);

    // List sessions
    console.log('3. Listing sessions...');
    const sessions = await client.listSessions(10, 0);
    console.log(`   Found ${sessions.length} sessions`);
    sessions.forEach((s) => {
      console.log(`   - ${s.id}: ${s.messageCount} messages`);
    });
    console.log();

    // List available tools
    console.log('4. Listing available tools...');
    const tools = await client.listTools();
    console.log(`   Found ${tools.length} tools`);
    tools.slice(0, 5).forEach((t) => {
      console.log(`   - ${t.name}: ${t.description.slice(0, 50)}...`);
    });
    if (tools.length > 5) {
      console.log(`   ... and ${tools.length - 5} more`);
    }
    console.log();

    // Execute a tool
    console.log('5. Executing tool (read file)...');
    const toolCall: ToolCall = {
      name: 'read',
      arguments: {
        path: '/tmp/test.txt',
      },
    };

    try {
      const result = await client.executeTool(toolCall);
      if (result.success) {
        console.log(`   Success! Result: ${result.result?.slice(0, 100)}...`);
      } else {
        console.log(`   Failed: ${result.error}`);
      }
    } catch (error) {
      if (error instanceof SdkError) {
        console.log(`   SDK Error [${error.code}]: ${error.message}`);
      } else {
        throw error;
      }
    }
    console.log();

    // Fork session
    console.log('6. Forking session...');
    const fork = await client.forkSession(session.sessionId, 1);
    console.log(`   Forked session: ${fork.id}`);
    console.log(`   Parent: ${fork.parentSessionId}`);
    console.log(`   Messages: ${fork.messageCount}\n`);

    console.log('=== Example completed successfully! ===');
  } catch (error) {
    if (error instanceof SdkError) {
      console.error(`SDK Error [${error.code}] (${error.category}): ${error.message}`);
      process.exit(1);
    }
    throw error;
  }
}

/**
 * Example: Local session (offline mode).
 */
async function localSessionExample(): Promise<void> {
  console.log('\n=== Local Session Example ===\n');

  const client = OpenCodeClient.builder()
    .apiKey('sk-local-key')
    .build();

  // Create local session (no server required)
  console.log('1. Creating local session...');
  await client.createLocalSession('Working offline');
  const localSession = await client.getLocalSession();
  console.log(`   Local session ID: ${localSession?.id ?? 'none'}`);
  console.log(`   Messages: ${localSession?.messageCount ?? 0}\n`);

  // Add messages locally
  console.log('2. Adding local messages...');
  await client.addLocalMessage('assistant', 'Hello! How can I help you?');
  await client.addLocalMessage('user', 'Just testing local mode.');
  const updated = await client.getLocalSession();
  console.log(`   Messages: ${updated?.messageCount ?? 0}\n`);

  console.log('=== Local session example completed! ===');
}

// Run examples
basicExample().catch(console.error);
localSessionExample().catch(console.error);
