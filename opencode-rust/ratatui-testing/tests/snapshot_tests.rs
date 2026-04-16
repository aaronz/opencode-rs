use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_buffer(width: u16, height: u16, content: &[&str]) -> Buffer {
    let area = Rect::new(0, 0, width, height);
    let mut buffer = Buffer::empty(area);
    for (y, line) in content.iter().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let idx = y * width as usize + x;
            if idx < buffer.content.len() {
                buffer.content[idx].set_symbol(c.to_string().as_str());
            }
        }
    }
    buffer
}

#[test]
fn test_snapshot_with_custom_directory() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let custom_path = temp_dir.path().to_str().unwrap();

    env::set_var("RATATUI_TESTING_SNAPSHOT_DIR", custom_path);

    let buffer = create_buffer(3, 2, &["abc", "def"]);
    let name = "custom_dir_snapshot";

    ratatui_testing::save_snapshot(name, &buffer).expect("Should save snapshot to custom dir");

    let expected_path: PathBuf = PathBuf::from(custom_path).join(format!("{}.json", name));
    assert!(
        expected_path.exists(),
        "Snapshot file should exist in custom directory"
    );

    let loaded =
        ratatui_testing::load_snapshot(name).expect("Should load snapshot from custom dir");
    assert_eq!(loaded.area.width, buffer.area.width);
    assert_eq!(loaded.area.height, buffer.area.height);

    for i in 0..buffer.content.len() {
        assert_eq!(loaded.content[i].symbol(), buffer.content[i].symbol());
    }

    fs::remove_file(&expected_path).ok();
    env::remove_var("RATATUI_TESTING_SNAPSHOT_DIR");
}
