use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

use super::Tool;
use crate::error::{AgentError, Result};

#[derive(Deserialize)]
struct ReadInput {
    path: String,
}

pub struct Read;

#[async_trait]
impl Tool for Read {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "read a file"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The file path to read"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, input: Value) -> Result<String> {
        let p: ReadInput = serde_json::from_value(input)?;
        let contents = fs::read_to_string(&p.path).map_err(AgentError::FileError)?;
        Ok(contents)
    }
}

#[derive(Deserialize)]
struct ListInput {
    path: String,
}

pub struct List;

#[async_trait]
impl Tool for List {
    fn name(&self) -> &str {
        "ls"
    }

    fn description(&self) -> &str {
        "list directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The directory path to list"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, input: Value) -> Result<String> {
        let p: ListInput = serde_json::from_value(input)?;
        let path = Path::new(&p.path);

        if !path.exists() {
            return Err(AgentError::ToolError(format!("not found: {}", p.path)));
        }

        let mut entries = Vec::new();

        if path.is_dir() {
            let dir_entries = fs::read_dir(path).map_err(AgentError::FileError)?;

            for entry in dir_entries {
                let entry = entry.map_err(AgentError::FileError)?;
                let path = entry.path();
                let metadata = entry.metadata().map_err(AgentError::FileError)?;

                let entry_type = if metadata.is_dir() { "DIR" } else { "FILE" };
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

                entries.push(format!("[{}] {}", entry_type, name));
            }
        } else {
            entries.push(format!("[FILE] {}", path.display()));
        }

        Ok(entries.join("\n"))
    }
}

#[derive(Deserialize)]
struct EditInput {
    path: String,
    old_content: String,
    new_content: String,
}

pub struct Edit;

#[async_trait]
impl Tool for Edit {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "edit file"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The file path to edit"
                },
                "old_content": {
                    "type": "string",
                    "description": "The content to replace"
                },
                "new_content": {
                    "type": "string",
                    "description": "The new content"
                }
            },
            "required": ["path", "old_content", "new_content"]
        })
    }

    async fn execute(&self, input: Value) -> Result<String> {
        let p: EditInput = serde_json::from_value(input)?;
        let current = fs::read_to_string(&p.path).map_err(AgentError::FileError)?;
        
        if !current.contains(&p.old_content) {
            return Err(AgentError::ToolError(format!("not found in {}", p.path)));
        }
        
        let new = current.replacen(&p.old_content, &p.new_content, 1);
        fs::write(&p.path, &new).map_err(AgentError::FileError)?;

        let diff = TextDiff::from_lines(&current, &new);
        let mut out = String::new();
        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            out.push_str(&format!("{}{}", sign, change));
        }
        Ok(out)
    }
}

pub fn register(reg: &mut super::ToolRegistry) {
    reg.register(Box::new(Read));
    reg.register(Box::new(List));
    reg.register(Box::new(Edit));
}
