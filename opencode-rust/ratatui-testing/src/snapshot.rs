use anyhow::{Context, Result};
use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_SNAPSHOT_DIR: &str = "snapshots";
const SNAPSHOT_DIR_ENV_VAR: &str = "RATATUI_TESTING_SNAPSHOT_DIR";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializedCell {
    symbol: String,
    fg: String,
    bg: String,
    modifier_bits: u16,
}

impl SerializedCell {
    fn from_cell(cell: &Cell) -> Self {
        SerializedCell {
            symbol: cell.symbol().to_string(),
            fg: format!("{:?}", cell.fg),
            bg: format!("{:?}", cell.bg),
            modifier_bits: cell.modifier.bits(),
        }
    }

    fn to_cell(&self) -> Cell {
        let mut cell = Cell::default();
        cell.set_symbol(&self.symbol);
        if let Ok(fg) = self.fg.parse() {
            cell.set_fg(fg);
        }
        if let Ok(bg) = self.bg.parse() {
            cell.set_bg(bg);
        }
        cell.modifier = Modifier::from_bits(self.modifier_bits).unwrap_or_default();
        cell
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializedBuffer {
    area_x: u16,
    area_y: u16,
    area_width: u16,
    area_height: u16,
    cells: Vec<SerializedCell>,
}

impl From<&Buffer> for SerializedBuffer {
    fn from(buffer: &Buffer) -> Self {
        let cells: Vec<SerializedCell> = buffer
            .content
            .iter()
            .map(SerializedCell::from_cell)
            .collect();
        SerializedBuffer {
            area_x: buffer.area.x,
            area_y: buffer.area.y,
            area_width: buffer.area.width,
            area_height: buffer.area.height,
            cells,
        }
    }
}

impl SerializedBuffer {
    fn to_buffer(&self) -> Buffer {
        let area = Rect::new(self.area_x, self.area_y, self.area_width, self.area_height);
        let mut buffer = Buffer::empty(area);
        for (i, cell) in self.cells.iter().enumerate() {
            if i < buffer.content.len() {
                buffer.content[i] = cell.to_cell();
            }
        }
        buffer
    }
}

fn get_snapshot_dir() -> Result<PathBuf> {
    let dir_name =
        env::var(SNAPSHOT_DIR_ENV_VAR).unwrap_or_else(|_| DEFAULT_SNAPSHOT_DIR.to_string());
    let dir = PathBuf::from(dir_name);
    if !dir.exists() {
        fs::create_dir_all(&dir).context("Failed to create snapshot directory")?;
    }
    Ok(dir)
}

#[allow(clippy::collapsible_str_replace)]
fn get_snapshot_path(name: &str) -> Result<PathBuf> {
    let dir = get_snapshot_dir()?;
    let sanitized_name = name.replace('/', "_").replace('\\', "_").replace("..", "_");
    Ok(dir.join(format!("{}.json", sanitized_name)))
}

pub fn load_snapshot(name: &str) -> Result<Buffer> {
    let path = get_snapshot_path(name).context("Failed to get snapshot path")?;
    let content = fs::read_to_string(&path)
        .context(format!("Failed to read snapshot file: {}", path.display()))?;
    let serialized: SerializedBuffer =
        serde_json::from_str(&content).context("Failed to parse snapshot file")?;
    Ok(serialized.to_buffer())
}

pub fn save_snapshot(name: &str, buffer: &Buffer) -> Result<()> {
    let path = get_snapshot_path(name).context("Failed to get snapshot path")?;
    let serialized = SerializedBuffer::from(buffer);
    let content =
        serde_json::to_string_pretty(&serialized).context("Failed to serialize snapshot")?;
    fs::write(&path, content)
        .context(format!("Failed to write snapshot file: {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Modifier};

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
    fn test_save_and_load_snapshot() {
        let buffer = create_buffer(3, 2, &["abc", "def"]);

        let name = "test_snapshot";
        save_snapshot(name, &buffer).unwrap();

        let loaded = load_snapshot(name).unwrap();
        assert_eq!(loaded.area.width, buffer.area.width);
        assert_eq!(loaded.area.height, buffer.area.height);

        for i in 0..buffer.content.len() {
            assert_eq!(loaded.content[i].symbol(), buffer.content[i].symbol());
        }

        std::fs::remove_file(get_snapshot_path(name).unwrap()).ok();
    }

    #[test]
    fn test_load_nonexistent_snapshot() {
        let result = load_snapshot("nonexistent_snapshot_12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_organized_directory_structure() {
        let dir = get_snapshot_dir().unwrap();
        assert!(dir.exists());
        assert_eq!(dir.file_name().unwrap().to_str().unwrap(), "snapshots");
    }

    #[test]
    fn test_snapshot_with_colors() {
        let area = Rect::new(0, 0, 2, 1);
        let mut buffer = Buffer::empty(area);
        buffer.content[0].set_symbol("H");
        buffer.content[0].set_fg(Color::Green);
        buffer.content[0].set_bg(Color::Black);
        buffer.content[0].modifier = Modifier::BOLD;

        let name = "color_snapshot";
        save_snapshot(name, &buffer).unwrap();

        let loaded = load_snapshot(name).unwrap();
        assert_eq!(loaded.content[0].symbol(), "H");

        std::fs::remove_file(get_snapshot_path(name).unwrap()).ok();
    }

    #[test]
    fn test_default_snapshot_dir() {
        env::remove_var(SNAPSHOT_DIR_ENV_VAR);
        let dir = get_snapshot_dir().unwrap();
        assert_eq!(
            dir.file_name().unwrap().to_str().unwrap(),
            DEFAULT_SNAPSHOT_DIR
        );
    }

    #[test]
    fn test_custom_snapshot_dir_from_env() {
        let custom_dir = "custom_test_snapshots";
        env::set_var(SNAPSHOT_DIR_ENV_VAR, custom_dir);
        let dir = get_snapshot_dir().unwrap();
        assert_eq!(dir.file_name().unwrap().to_str().unwrap(), custom_dir);
        env::remove_var(SNAPSHOT_DIR_ENV_VAR);
        let default_dir = get_snapshot_dir().unwrap();
        assert_eq!(
            default_dir.file_name().unwrap().to_str().unwrap(),
            DEFAULT_SNAPSHOT_DIR
        );
    }
}
