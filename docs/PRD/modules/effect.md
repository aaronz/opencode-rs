# PRD: effect Module

## Module Overview

**Module Name:** `effect`
**Type:** Utility
**Source:** `/packages/opencode/src/effect/`

## Purpose

Effect-based functional programming utilities. Provides Effect monad for async operations with proper error handling, similar to ZIO in Scala.

## Functionality

### Core Features

1. **Effect Monad**
   - Composable async operations
   - Error handling with typed errors
   - Resource management
   - Concurrent operations

2. **Service Pattern**
   - Dependency injection
   - Layer composition
   - Context management

3. **Key Types**

   ```typescript
   // Effect - represents an async operation that may fail
   type Effect<A, E, R> = (context: Context<R>) => Promise<A>

   // Service - Effect-based service definition
   class Service<Interface> {
     readonly _tag: string
     readonly [key: symbol]: Interface
   }

   // Layer - Effect-based dependency layer
   type Layer<Out, E, In> = (context: Context<In>) => Effect<Context<Out>, E, never>
   ```

### Service Interface Pattern

```typescript
// Define service interface
interface DatabaseService {
  query(sql: string): Effect<Row[], Error, never>
  execute(sql: string): Effect<void, Error, never>
}

// Implement service
class DatabaseServiceImpl implements DatabaseService {
  query(sql: string): Effect<Row[], Error, never> {
    return Effect.gen(function* () {
      const db = yield* Database
      return db.query(sql)
    })
  }
}

// Create layer
const DatabaseLayer = Layer.effect(
  DatabaseService,
  Effect.gen(function* () {
    const db = yield* Effect.promise(() => new Database())
    return new DatabaseServiceImpl(db)
  })
)
```

### Effect Operations

| Operation | Description |
|-----------|-------------|
| `Effect.succeed` | Create successful effect |
| `Effect.fail` | Create failed effect |
| `Effect.gen` | Generator-based effect |
| `Effect.promise` | Convert Promise to Effect |
| `Effect.orDie` | Convert error to fatal |
| `Effect.map` | Map success value |
| `Effect.flatMap` | Chain effects |

## Dependencies

- `effect` - Effect library

## Acceptance Criteria

1. Effect operations work correctly
2. Services are properly composed
3. Errors are handled typed
4. Resources are managed properly

## Rust Implementation Guidance

The Rust equivalent should:
- Use `tokio` for async runtime
- Use `anyhow` for error handling
- Use traits for service interfaces
- Consider using `async_trait`

## Test Design

### Unit Tests
- `monad_composition`: Test chaining of successful and failing effects.
- `service_injection`: Test that a service can be registered and accessed from context.

### Rust Specifics
- In Rust, this usually maps to testing generic trait bounds and async result chaining. Test `Result<T, E>` combinators and custom context injection structs.
