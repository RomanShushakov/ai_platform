mod adapters;

use adapters::tools_client::ToolsClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ToolsClient::new("http://localhost:3001");

    let tools = client.list_tools().await?;
    println!("TOOLS: {:#?}", tools);

    let result = client
        .call_tool(
            "get_weather".to_string(),
            serde_json::json!({ "city": "Berlin" }),
        )
        .await?;

    println!("TOOL RESULT: {:#?}", result);

    Ok(())
}
