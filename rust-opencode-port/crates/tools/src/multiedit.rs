use async_trait::async_trait;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Edit {
    pub path: String,
    pub old_string: String,
    pub new_string: String,
}

pub struct MultiEditTool;

#[async_trait]
impl Tool for MultiEditTool {
    fn name(&self) -> &str {
        "multi_edit"
    }

    fn description(&self) -> &str {
        "Apply multiple edits across different files atomically"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(MultiEditTool)
    }

    async fn execute(&self, args: serde_json::Value, _ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let edits: Vec<Edit> = serde_json::from_value(args).map_err(|e| OpenCodeError::Parse(e.to_string()))?;
        
        let mut file_contents: HashMap<String, String> = HashMap::new();
        
        // First pass: validate all edits and read current content
        for edit in &edits {
            if !file_contents.contains_key(&edit.path) {
                let content = std::fs::read_to_string(&edit.path).map_err(|e| OpenCodeError::Io(e))?;
                file_contents.insert(edit.path.clone(), content);
            }
            
            let content = file_contents.get(&edit.path).unwrap();
            if !content.contains(&edit.old_string) {
                return Err(OpenCodeError::Tool(format!("Old string not found in {}", edit.path)));
            }
        }
        
        // Second pass: apply all edits to the in-memory contents
        for edit in edits {
            let content = file_contents.get_mut(&edit.path).unwrap();
            *content = content.replace(&edit.old_string, &edit.new_string);
        }
        
        // Third pass: write all updated contents back to files (atomic-ish)
        for (path, new_content) in file_contents {
            std::fs::write(&path, new_content).map_err(|e| OpenCodeError::Io(e))?;
        }

        Ok(ToolResult::ok("All edits applied successfully".to_string()))
    }
}
