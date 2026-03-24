use anyhow::{Context, Result};
use serde_json::{Value, json};
use shared_types::{ToolDefinition, ToolResult};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "get_weather".to_string(),
        description:
            "Get current weather for a city. Use this when the user asks about weather or \
        temperature in a location."
                .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "Name of the city"
                }
            },
            "required": ["city"]
        }),
    }
}

pub async fn execute(arguments: Value) -> Result<ToolResult> {
    let city = arguments
        .get("city")
        .and_then(|v| v.as_str())
        .context("missing required string argument: city")?;

    let content = mock_weather_for(city);

    Ok(ToolResult {
        name: "get_weather".to_string(),
        content,
    })
}

fn mock_weather_for(city: &str) -> Value {
    let normalized = city.trim().to_lowercase();

    let (temperature_c, condition) = match normalized.as_str() {
        "berlin" => (18, "Cloudy"),
        "hamburg" => (16, "Windy"),
        "munich" | "münchen" => (20, "Sunny"),
        _ => (17, "Partly cloudy"),
    };

    json!({
        "city": city,
        "temperature_c": temperature_c,
        "condition": condition
    })
}
