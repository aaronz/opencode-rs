use crate::events::{Event, EventBus};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct WorkspaceManager {
    root: PathBuf,
    event_bus: Arc<EventBus>,
}

impl WorkspaceManager {
    pub fn new(root: PathBuf, event_bus: Arc<EventBus>) -> Self {
        Self { root, event_bus }
    }

    pub fn start_watcher(&self) -> notify::Result<RecommendedWatcher> {
        let event_bus = self.event_bus.clone();
        let root = self.root.clone();

        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                            for path in event.paths {
                                if let Ok(rel_path) = path.strip_prefix(&root) {
                                    event_bus.publish(Event::FileChanged(
                                        rel_path.to_string_lossy().to_string(),
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })?;

        watcher.watch(&self.root, RecursiveMode::Recursive)?;
        Ok(watcher)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}
