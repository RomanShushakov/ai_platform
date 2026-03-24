use anyhow::{Context, Result};
use serde_json::{Value, json};
use shared_types::{ToolDefinition, ToolResult};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "search_docs".to_string(),
        description: "Search internal company documentation. Use this when the user asks about company policies, \
        onboarding, vacation, expenses, access, or internal processes.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query for internal documentation"
                }
            },
            "required": ["query"]
        }),
    }
}

pub async fn execute(arguments: Value) -> Result<ToolResult> {
    let query = arguments
        .get("query")
        .and_then(|v| v.as_str())
        .context("missing required string argument: query")?;

    let results = search_mock_docs(query);

    Ok(ToolResult {
        name: "search_docs".to_string(),
        content: json!({
            "query": query,
            "results": results
        }),
    })
}

fn search_mock_docs(query: &str) -> Vec<Value> {
    let docs = vec![
        (
            "vacation_policy",
            "Employees can request vacation in the HR portal. Manager approval is required \
            for absences longer than 3 business days.",
        ),
        (
            "onboarding_guide",
            "New employees should complete onboarding in the first week, including account setup, \
            security training, and team introduction meetings.",
        ),
        (
            "expense_policy",
            "Business expenses must be submitted within 30 days with receipts attached. \
            Travel expenses require manager approval.",
        ),
    ];

    let q = query.to_lowercase();

    docs.into_iter()
        .filter(|(_, content)| {
            let c = content.to_lowercase();
            c.contains(&q) || q.split_whitespace().any(|word| c.contains(word))
        })
        .map(|(id, snippet)| {
            json!({
                "doc_id": id,
                "snippet": snippet
            })
        })
        .collect()
}
