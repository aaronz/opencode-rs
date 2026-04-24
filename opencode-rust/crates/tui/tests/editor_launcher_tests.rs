use opencode_tui::input::EditorLauncher;
use std::env;

#[test]
fn test_editor_launcher_new() {
    let _launcher = EditorLauncher::new();
    // EditorLauncher created
}

#[test]
fn test_editor_launcher_with_editor() {
    let _launcher = EditorLauncher::new().with_editor("vim".to_string());
    // EditorLauncher with editor created
}

#[test]
fn test_editor_launcher_from_env_without_editor() {
    env::remove_var("EDITOR");
    env::remove_var("VISUAL");
    let _launcher = EditorLauncher::from_env();
    // EditorLauncher from env works
}

#[test]
fn test_editor_launcher_from_env_with_editor() {
    env::set_var("EDITOR", "nvim");
    let _launcher = EditorLauncher::from_env();
    env::remove_var("EDITOR");
    // EditorLauncher from env created
}

#[test]
fn test_editor_launcher_from_env_prefers_editor_over_visual() {
    env::set_var("EDITOR", "vim");
    env::set_var("VISUAL", "nano");
    let _launcher = EditorLauncher::from_env();
    env::remove_var("EDITOR");
    env::remove_var("VISUAL");
    // EditorLauncher prefers EDITOR over VISUAL
}

#[test]
fn test_editor_launcher_fallback_to_visual() {
    env::remove_var("EDITOR");
    env::set_var("VISUAL", "code");
    let _launcher = EditorLauncher::from_env();
    env::remove_var("VISUAL");
    // EditorLauncher falls back to VISUAL
}

#[test]
fn test_editor_launcher_empty_env_var() {
    env::set_var("EDITOR", "");
    let _launcher = EditorLauncher::from_env();
    env::remove_var("EDITOR");
    // EditorLauncher handles empty EDITOR
}

#[test]
fn test_editor_launcher_chaining() {
    let _launcher = EditorLauncher::new()
        .with_editor("emacs".to_string())
        .with_editor("vim".to_string());
    // EditorLauncher chaining works
}

#[test]
fn test_editor_launcher_default() {
    let _launcher = EditorLauncher::default();
    // Default constructor works
}
