use dotenv::dotenv;
use rustpilot::{Agent, Result};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let api_key =
        env::var("ANTHROPIC_API_KEY").expect("Please set ANTHROPIC_API_KEY environment variable");

    let model = env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| rustpilot::models::select_model());

    let mut agent = Agent::with_model(api_key, model)?;
    agent.run().await?;

    Ok(())
}
