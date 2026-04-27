use std::collections::HashMap;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::warn;
use walkdir::WalkDir;

use crate::config::WatcherConfig;

const DEFAULT_IGNORE_PATTERNS: [&str; 6] = [
    ".git/**",
    "node_modules/**",
    "dist/**",
    "build/**",
    "target/**",
    ".next/**",
];
const MAX_FILES_PER_DIRECTORY: usize = 10_000;

pub struct IgnoreMatcher {
    patterns: Vec<String>,
    globset: GlobSet,
}

impl IgnoreMatcher {
    pub fn new(user_patterns: Option<&[String]>) -> Result<Self, crate::OpenCodeError> {
        let mut patterns = DEFAULT_IGNORE_PATTERNS
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>();

        if let Some(user) = user_patterns {
            patterns.extend(user.iter().cloned());
        }

        let mut builder = GlobSetBuilder::new();
        for pattern in &patterns {
            let glob = Glob::new(pattern).map_err(|e| {
                crate::OpenCodeError::Config(format!("Invalid ignore pattern '{}': {}", pattern, e))
            })?;
            builder.add(glob);
        }

        let globset = builder.build().map_err(|e| {
            crate::OpenCodeError::Config(format!("Failed to build ignore matcher: {}", e))
        })?;

        Ok(Self { patterns, globset })
    }

    pub fn should_ignore(&self, path: &Path) -> bool {
        self.globset.is_match(path)
    }

    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }
}

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    _watch_roots: Vec<PathBuf>,
    pub ignore_matcher: IgnoreMatcher,
}

impl FileWatcher {
    pub fn start(watch_path: &Path, config: &WatcherConfig) -> Option<FileWatcher> {
        match Self::try_start(watch_path, config) {
            Ok(watcher) => Some(watcher),
            Err(err) => {
                warn!(
                    "File watcher failed to start at '{}': {}. Continuing without file watcher.",
                    watch_path.display(),
                    err
                );
                None
            }
        }
    }

    fn try_start(
        watch_path: &Path,
        config: &WatcherConfig,
    ) -> Result<FileWatcher, crate::OpenCodeError> {
        if !watch_path.exists() {
            return Err(crate::OpenCodeError::Config(format!(
                "Watch path '{}' does not exist",
                watch_path.display()
            )));
        }

        let ignore_matcher = IgnoreMatcher::new(config.ignore.as_deref())?;
        enforce_file_count_limit(watch_path, &ignore_matcher)?;

        let mut watcher = RecommendedWatcher::new(move |_event| {}, NotifyConfig::default())
            .map_err(|e| {
                crate::OpenCodeError::Config(format!("Failed to initialize watcher: {}", e))
            })?;

        let targets = collect_watch_targets(watch_path, &ignore_matcher);
        if targets.is_empty() {
            return Err(crate::OpenCodeError::Config(format!(
                "No watchable paths remain after ignore filtering for '{}'",
                watch_path.display()
            )));
        }

        for target in &targets {
            watcher
                .watch(target, RecursiveMode::NonRecursive)
                .map_err(|e| {
                    crate::OpenCodeError::Config(format!(
                        "Failed to watch '{}': {}",
                        target.display(),
                        e
                    ))
                })?;
        }

        Ok(FileWatcher {
            _watcher: watcher,
            _watch_roots: targets,
            ignore_matcher,
        })
    }
}

fn collect_watch_targets(root: &Path, matcher: &IgnoreMatcher) -> Vec<PathBuf> {
    let mut targets = vec![];
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !matcher.should_ignore(e.path()))
        .flatten()
    {
        if entry.path().is_dir() {
            targets.push(entry.path().to_path_buf());
        }
    }
    targets
}

fn enforce_file_count_limit(
    root: &Path,
    matcher: &IgnoreMatcher,
) -> Result<(), crate::OpenCodeError> {
    let mut counts: HashMap<PathBuf, usize> = HashMap::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !matcher.should_ignore(e.path()))
        .flatten()
    {
        if entry.path().is_file() {
            if let Some(parent) = entry.path().parent() {
                let count = counts.entry(parent.to_path_buf()).or_insert(0);
                *count += 1;
                if *count > MAX_FILES_PER_DIRECTORY {
                    return Err(crate::OpenCodeError::Config(format!(
                        "Directory '{}' exceeds max watched files ({} > {})",
                        parent.display(),
                        count,
                        MAX_FILES_PER_DIRECTORY
                    )));
                }
            }
        }
    }

    Ok(())
}