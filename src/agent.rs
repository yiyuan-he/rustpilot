use std::io::{self, Write};

use crate::anthropic::{
    AnthropicClient, ContentBlock, Message, MessagesRequest, Role, Tool, ToolChoice,
};
use crate::error::Result;
use crate::tools::{ToolDefinition, ToolRegistry, ToolUse};

pub struct Agent {
    client: AnthropicClient,
    registry: ToolRegistry,
    conversation: Vec<Message>,
    system_prompt: String,
}

impl Agent {
    pub fn new(api_key: String) -> Result<Self> {
        let client = AnthropicClient::new(api_key);

        let mut registry = ToolRegistry::new();
        crate::tools::file_tools::register(&mut registry);

        let system_prompt = "You're a coding assistant. You can read, list, and edit files.".to_string();

        Ok(Self {
            client,
            registry,
            conversation: Vec::new(),
            system_prompt,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("rustpilot");
        println!();

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            if input == "exit" || input == "quit" {
                println!("bye");
                break;
            }

            if input.is_empty() {
                continue;
            }

            match self.process_message(input).await {
                Ok(response) => println!("{}", response),
                Err(e) => eprintln!("error: {}", e),
            }
        }

        Ok(())
    }

    async fn process_message(&mut self, input: &str) -> Result<String> {
        let msg = Message {
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: input.to_string(),
            }],
        };
        self.conversation.push(msg.clone());

        let tools = self
            .registry
            .definitions()
            .into_iter()
            .map(|def| self.to_tool(def))
            .collect();

        let request = MessagesRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            messages: self.conversation.clone(),
            system: Some(self.system_prompt.clone()),
            tools: Some(tools),
            tool_choice: Some(ToolChoice::Auto),
            ..Default::default()
        };

        let response = self.client.messages(request).await?;

        let mut response_text = String::new();
        let mut tool_uses = Vec::new();

        for block in &response.content {
            match block {
                ContentBlock::Text { text } => {
                    response_text.push_str(text);
                }
                ContentBlock::ToolUse { id, name, input } => {
                    tool_uses.push(ToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                    });
                }
                _ => {}
            }
        }

        if !tool_uses.is_empty() {
            let mut tool_results = Vec::new();

            for tool_use in tool_uses {
                println!("[{}]", tool_use.name);

                let result = self.registry.execute(&tool_use).await?;

                if result.is_error {
                    eprintln!("failed: {}", result.content);
                }

                tool_results.push(result);
            }

            self.conversation.push(Message {
                role: Role::Assistant,
                content: response.content,
            });

            let tool_result_message = Message {
                role: Role::User,
                content: tool_results
                    .into_iter()
                    .map(|result| ContentBlock::ToolResult {
                        tool_use_id: result.tool_use_id,
                        content: result.content,
                        is_error: Some(result.is_error),
                    })
                    .collect(),
            };
            self.conversation.push(tool_result_message);

            return self.get_follow_up_response().await;
        }

        self.conversation.push(Message {
            role: Role::Assistant,
            content: response.content,
        });

        Ok(response_text)
    }

    async fn get_follow_up_response(&mut self) -> Result<String> {
        let request = MessagesRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            messages: self.conversation.clone(),
            system: Some(self.system_prompt.clone()),
            ..Default::default()
        };

        let response = self.client.messages(request).await?;

        let mut text = String::new();
        for block in &response.content {
            if let ContentBlock::Text { text: t } = block {
                text.push_str(t);
            }
        }

        self.conversation.push(Message {
            role: Role::Assistant,
            content: response.content,
        });
        Ok(text)
    }

    fn to_tool(&self, def: ToolDefinition) -> Tool {
        Tool {
            name: def.name,
            description: Some(def.description),
            input_schema: def.input_schema,
        }
    }
}

