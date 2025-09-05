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
    model: String,
}

impl Agent {
    pub fn new(api_key: String) -> Result<Self> {
        Self::with_model(api_key, "claude-opus-4-1-20250805".to_string())
    }

    pub fn with_model(api_key: String, model: String) -> Result<Self> {
        let client = AnthropicClient::new(api_key);

        let mut registry = ToolRegistry::new();
        crate::tools::file_tools::register(&mut registry);

        let system_prompt = "You are an expert coding assistant with deep knowledge across multiple programming languages and frameworks. Your primary goal is to help users with coding tasks effectively and accurately.

Key capabilities:
- Read, analyze, and understand code in any programming language
- Write, edit, and refactor code following best practices and conventions
- Debug issues and suggest optimizations
- Explain complex concepts clearly and provide examples
- Use available file system tools to read/edit files directly

Guidelines:
1. Always strive for code quality: write clean, maintainable, and well-documented code
2. Consider edge cases and potential errors in your solutions
3. Follow language-specific conventions and idioms
4. When editing files, preserve existing code style and formatting
5. Provide clear explanations for your changes and recommendations
6. Ask clarifying questions when requirements are ambiguous
7. Be explicit about assumptions you're making
8. Test your understanding by reading files before making changes

When working with files:
- Always read the current content before editing to understand context
- Make targeted, precise edits rather than rewriting entire files
- Preserve important comments and documentation
- Be careful with indentation and formatting to match the existing style

Remember: You have access to file reading and editing tools. Use them to provide direct, practical assistance rather than just theoretical advice.".to_string();

        Ok(Self {
            client,
            registry,
            conversation: Vec::new(),
            system_prompt,
            model,
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
        self.conversation.push(msg);

        self.reasoning_loop().await
    }

    async fn reasoning_loop(&mut self) -> Result<String> {
        let mut final_response = String::new();

        loop {
            let tools = self
                .registry
                .definitions()
                .into_iter()
                .map(|def| self.to_tool(def))
                .collect();

            let request = MessagesRequest {
                model: self.model.clone(),
                max_tokens: 4096,
                messages: self.conversation.clone(),
                system: Some(self.system_prompt.clone()),
                tools: Some(tools),
                tool_choice: Some(ToolChoice::Auto),
                ..Default::default()
            };

            let response = self.client.messages(request).await?;

            let mut text = String::new();
            let mut tool_uses = Vec::new();

            for block in &response.content {
                match block {
                    ContentBlock::Text { text: t } => {
                        text.push_str(t);
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

            if !text.is_empty() {
                final_response = text;
            }

            self.conversation.push(Message {
                role: Role::Assistant,
                content: response.content,
            });

            if tool_uses.is_empty() {
                // no more tools to use, we're done
                break;
            }

            // execute tools
            let mut tool_results = Vec::new();
            for tool_use in tool_uses {
                println!("→ {}", tool_use.name);

                let result = self.registry.execute(&tool_use).await?;

                if result.is_error {
                    eprintln!("failed: {}", result.content);
                }

                tool_results.push(result);
            }

            self.conversation.push(Message {
                role: Role::User,
                content: tool_results
                    .into_iter()
                    .map(|result| ContentBlock::ToolResult {
                        tool_use_id: result.tool_use_id,
                        content: result.content,
                        is_error: Some(result.is_error),
                    })
                    .collect(),
            });

            // loop continues to get next response
        }

        Ok(final_response)
    }

    fn to_tool(&self, def: ToolDefinition) -> Tool {
        Tool {
            name: def.name,
            description: Some(def.description),
            input_schema: def.input_schema,
        }
    }
}
