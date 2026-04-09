# OpenCode TUI Plugin SDK

TypeScript SDK for developing TUI plugins for OpenCode.

## Installation

```bash
npm install @opencode-ai/plugin-tui
```

## Usage

```typescript
import { createTuiPluginModule, TuiPlugin } from '@opencode-ai/plugin-tui';

const module = createTuiPluginModule();

// Register a command
module.commands.register({
  id: 'my-command',
  label: 'My Command',
  shortcut: 'Ctrl+M',
  action: () => {
    console.log('Command executed!');
  },
});

// Register a route
module.routes.register({
  id: 'my-route',
  path: '/my-route',
  handler: (params) => {
    console.log('Navigated to:', params);
  },
});

// Register a dialog
module.dialogs.register({
  id: 'my-dialog',
  title: 'My Dialog',
  content: 'Hello World',
  onConfirm: () => console.log('Confirmed'),
  onCancel: () => console.log('Cancelled'),
});

// Register a slot
module.slots.register({
  id: 'my-slot',
  render: () => {
    console.log('Rendering slot');
  },
});

// Install a theme
module.themes.install({
  id: 'my-theme',
  name: 'My Theme',
  colors: {
    background: '#000000',
    foreground: '#ffffff',
  },
});

// Set the current theme
module.themes.set('my-theme');

// Listen to events
module.events.on('session:start', (data) => {
  console.log('Session started:', data);
});

// State management
module.state.set('user-pref', { theme: 'dark' });
const pref = module.state.get('user-pref');

// Cleanup on dispose
module.onDispose = () => {
  console.log('Plugin disposed');
};
```

## API Reference

### Commands API

- `commands.register(definition)` - Register a new command
- `commands.unregister(id)` - Unregister a command
- `commands.list()` - List all registered commands

### Routes API

- `routes.register(definition)` - Register a new route
- `routes.unregister(id)` - Unregister a route
- `routes.navigate(path, params)` - Navigate to a route

### Dialogs API

- `dialogs.register(definition)` - Register a new dialog
- `dialogs.unregister(id)` - Unregister a dialog
- `dialogs.show(id)` - Show a dialog
- `dialogs.hide(id)` - Hide a dialog

### Slots API

- `slots.register(definition)` - Register a new slot
- `slots.unregister(id)` - Unregister a slot
- `slots.update(id)` - Update/render a slot

### Themes API

- `themes.install(theme)` - Install a theme
- `themes.uninstall(id)` - Uninstall a theme
- `themes.set(id)` - Set the current theme
- `themes.getCurrent()` - Get the current theme
- `themes.list()` - List all installed themes

### Events API

- `events.on(event, callback)` - Subscribe to an event (returns subscription ID)
- `events.off(id)` - Unsubscribe from an event
- `events.emit(event, data)` - Emit an event

### State API

- `state.get(key)` - Get a value from state
- `state.set(key, value)` - Set a value in state
- `state.delete(key)` - Delete a value from state
- `state.clear()` - Clear all state

## License

MIT
