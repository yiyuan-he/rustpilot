use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub mod file_tools;

use crate::error::Result;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, input: Value) -> Result<String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

impl ToolDefinition {
    pub fn from_tool(tool: &dyn Tool) -> Self {
        Self {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            input_schema: tool.input_schema(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUse {
    pub id: String,
    pub name: String,
    pub input: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.get(name)
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| ToolDefinition::from_tool(tool.as_ref()))
            .collect()
    }

    pub async fn execute(&self, tool_use: &ToolUse) -> Result<ToolResult> {
        match self.get(&tool_use.name) {
            Some(tool) => match tool.execute(tool_use.input.clone()).await {
                Ok(content) => Ok(ToolResult {
                    tool_use_id: tool_use.id.clone(),
                    content,
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    tool_use_id: tool_use.id.clone(),
                    content: format!("Error: {}", e),
                    is_error: true,
                }),
            },
            None => Ok(ToolResult {
                tool_use_id: tool_use.id.clone(),
                content: format!("Tool '{}' not found", tool_use.name),
                is_error: true,
            }),
        }
    }
}
