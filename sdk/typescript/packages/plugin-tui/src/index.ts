export interface CommandDefinition {
  id: string;
  label: string;
  shortcut?: string;
  action: () => void | Promise<void>;
}

export interface RouteDefinition {
  id: string;
  path: string;
  handler: (params: Record<string, string>) => void | Promise<void>;
}

export interface DialogDefinition {
  id: string;
  title: string;
  content: string;
  onConfirm?: () => void | Promise<void>;
  onCancel?: () => void | Promise<void>;
}

export interface SlotDefinition {
  id: string;
  render: () => void | Promise<void>;
}

export interface ThemeDefinition {
  id: string;
  name: string;
  colors: Record<string, string>;
}

export interface EventCallback {
  id: string;
  event: string;
  callback: (data: unknown) => void | Promise<void>;
}

export interface StateStore {
  get<T = unknown>(key: string): T | undefined;
  set<T = unknown>(key: string, value: T): void;
  delete(key: string): void;
  clear(): void;
}

export interface TuiPluginModule {
  commands: CommandsAPI;
  routes: RoutesAPI;
  dialogs: DialogsAPI;
  slots: SlotsAPI;
  themes: ThemesAPI;
  events: EventsAPI;
  state: StateAPI;
  onDispose?: () => void | Promise<void>;
}

export interface CommandsAPI {
  register(definition: CommandDefinition): void;
  unregister(id: string): void;
  list(): CommandDefinition[];
}

export interface RoutesAPI {
  register(definition: RouteDefinition): void;
  unregister(id: string): void;
  navigate(path: string, params?: Record<string, string>): void;
}

export interface DialogsAPI {
  register(definition: DialogDefinition): void;
  unregister(id: string): void;
  show(id: string): void;
  hide(id: string): void;
}

export interface SlotsAPI {
  register(definition: SlotDefinition): void;
  unregister(id: string): void;
  update(id: string): void;
}

export interface ThemesAPI {
  install(theme: ThemeDefinition): void;
  uninstall(id: string): void;
  set(id: string): void;
  getCurrent(): ThemeDefinition | undefined;
  list(): ThemeDefinition[];
}

export interface EventsAPI {
  on(event: string, callback: (data: unknown) => void | Promise<void>): string;
  off(id: string): void;
  emit(event: string, data: unknown): void;
}

export interface StateAPI {
  get<T = unknown>(key: string): T | undefined;
  set<T = unknown>(key: string, value: T): void;
  delete(key: string): void;
  clear(): void;
}

export interface TuiPlugin<TState = unknown> {
  id: string;
  name: string;
  version: string;
  module: TuiPluginModule;
}

export type PluginDisposeCallback = () => void | Promise<void>;

export function createCommandsAPI(): CommandsAPI {
  const commands = new Map<string, CommandDefinition>();
  
  return {
    register(definition: CommandDefinition): void {
      commands.set(definition.id, definition);
    },
    unregister(id: string): void {
      commands.delete(id);
    },
    list(): CommandDefinition[] {
      return Array.from(commands.values());
    },
  };
}

export function createRoutesAPI(): RoutesAPI {
  const routes = new Map<string, RouteDefinition>();
  
  return {
    register(definition: RouteDefinition): void {
      routes.set(definition.id, definition);
    },
    unregister(id: string): void {
      routes.delete(id);
    },
    navigate(path: string, params?: Record<string, string>): void {
      const route = Array.from(routes.values()).find(r => r.path === path);
      if (route) {
        route.handler(params || {});
      }
    },
  };
}

export function createDialogsAPI(): DialogsAPI {
  const dialogs = new Map<string, DialogDefinition>();
  
  return {
    register(definition: DialogDefinition): void {
      dialogs.set(definition.id, definition);
    },
    unregister(id: string): void {
      dialogs.delete(id);
    },
    show(id: string): void {
      const dialog = dialogs.get(id);
      if (dialog) {
        console.log(`Showing dialog: ${dialog.title}`);
      }
    },
    hide(id: string): void {
      console.log(`Hiding dialog: ${id}`);
    },
  };
}

export function createSlotsAPI(): SlotsAPI {
  const slots = new Map<string, SlotDefinition>();
  
  return {
    register(definition: SlotDefinition): void {
      slots.set(definition.id, definition);
    },
    unregister(id: string): void {
      slots.delete(id);
    },
    update(id: string): void {
      const slot = slots.get(id);
      if (slot) {
        slot.render();
      }
    },
  };
}

export function createThemesAPI(): ThemesAPI {
  const themes = new Map<string, ThemeDefinition>();
  let currentTheme: string | undefined;
  
  return {
    install(theme: ThemeDefinition): void {
      themes.set(theme.id, theme);
    },
    uninstall(id: string): void {
      themes.delete(id);
      if (currentTheme === id) {
        currentTheme = undefined;
      }
    },
    set(id: string): void {
      if (themes.has(id)) {
        currentTheme = id;
      }
    },
    getCurrent(): ThemeDefinition | undefined {
      return currentTheme ? themes.get(currentTheme) : undefined;
    },
    list(): ThemeDefinition[] {
      return Array.from(themes.values());
    },
  };
}

export function createEventsAPI(): EventsAPI {
  const events = new Map<string, EventCallback>();
  let idCounter = 0;
  
  return {
    on(event: string, callback: (data: unknown) => void | Promise<void>): string {
      const id = `event_${idCounter++}`;
      events.set(id, { id, event, callback });
      return id;
    },
    off(id: string): void {
      events.delete(id);
    },
    emit(event: string, data: unknown): void {
      for (const [, callback] of events) {
        if (callback.event === event) {
          callback.callback(data);
        }
      }
    },
  };
}

export function createStateAPI(): StateAPI {
  const state = new Map<string, unknown>();
  
  return {
    get<T = unknown>(key: string): T | undefined {
      return state.get(key) as T | undefined;
    },
    set<T = unknown>(key: string, value: T): void {
      state.set(key, value);
    },
    delete(key: string): void {
      state.delete(key);
    },
    clear(): void {
      state.clear();
    },
  };
}

export function createTuiPluginModule(): TuiPluginModule {
  return {
    commands: createCommandsAPI(),
    routes: createRoutesAPI(),
    dialogs: createDialogsAPI(),
    slots: createSlotsAPI(),
    themes: createThemesAPI(),
    events: createEventsAPI(),
    state: createStateAPI(),
  };
}
