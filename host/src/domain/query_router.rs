use shared_types::{HybridAnalysis, HybridLiveStatus, QueryRoute, ToolDefinition};

pub fn route_query(query: &str) -> QueryRoute {
    let q = query.to_lowercase();

    let tool_first_keywords = [
        "weather",
        "temperature",
        "current",
        "status",
        "check",
        "create",
        "update",
        "delete",
        "restart",
        "run",
        "deploy",
        "live",
        "today",
        "now",
    ];

    let retrieval_first_keywords = [
        "policy",
        "how do i",
        "how to",
        "onboarding",
        "guide",
        "documentation",
        "docs",
        "process",
        "architecture",
        "faq",
        "vacation",
        "expense",
    ];

    let tool_hits = tool_first_keywords
        .iter()
        .filter(|kw| q.contains(**kw))
        .count();

    let retrieval_hits = retrieval_first_keywords
        .iter()
        .filter(|kw| q.contains(**kw))
        .count();

    match (tool_hits, retrieval_hits) {
        (t, r) if t > 0 && r == 0 => QueryRoute::ToolFirst,
        (t, r) if r > 0 && t == 0 => QueryRoute::RetrievalFirst,
        (t, r) if t > 0 && r > 0 => QueryRoute::Hybrid,
        _ => QueryRoute::Hybrid,
    }
}

pub fn route_name(route: &QueryRoute) -> &'static str {
    match route {
        QueryRoute::RetrievalFirst => "retrieval_first",
        QueryRoute::ToolFirst => "tool_first",
        QueryRoute::Hybrid => "hybrid",
    }
}

fn has_live_tool_for_query(query: &str, tools: &[ToolDefinition]) -> bool {
    if query.contains("weather") {
        return tools.iter().any(|t| t.name == "get_weather");
    }

    if query.contains("reimbursement") {
        return tools.iter().any(|t| t.name == "get_reimbursement_status");
    }

    false
}

pub fn analyze_hybrid_query(query: &str, tools: &[ToolDefinition]) -> HybridAnalysis {
    let q = query.to_lowercase();

    let doc_keywords = [
        "policy",
        "how do i",
        "how to",
        "guide",
        "documentation",
        "docs",
        "process",
        "architecture",
        "faq",
        "vacation",
        "expense",
    ];

    let live_keywords = [
        "current status",
        "status of",
        "my reimbursement",
        "my request",
        "live",
        "today",
        "now",
        "current",
        "check",
    ];

    let has_doc_intent = doc_keywords.iter().any(|kw| q.contains(kw));
    let has_live_intent = live_keywords.iter().any(|kw| q.contains(kw));

    let live_status = if !has_live_intent {
        HybridLiveStatus::NotNeeded
    } else if has_live_tool_for_query(&q, tools) {
        HybridLiveStatus::ToolAvailable
    } else {
        HybridLiveStatus::MissingTool
    };

    HybridAnalysis {
        has_doc_intent,
        has_live_intent,
        live_status,
    }
}
