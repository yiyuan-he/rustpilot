use rustpilot::{Agent, Result};
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let api_key =
        env::var("ANTHROPIC_API_KEY").expect("Please set ANTHROPIC_API_KEY environment variable");

    let mut agent = Agent::new(api_key)?;
    agent.run().await?;

    Ok(())
}
