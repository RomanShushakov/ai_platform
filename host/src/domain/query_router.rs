use shared_types::QueryRoute;

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
