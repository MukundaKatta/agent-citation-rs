use agent_citation::{validate, Citation, CitationStore, JsonlSink};

#[test]
fn full_rag_turn_capture_then_validate() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("citations.jsonl");
    let store = CitationStore::with_sink(Box::new(JsonlSink::new(&path).unwrap()));

    let text = "The policy expires 2026-12-31 [1]. \
                Renewal requires 60 days notice [2]. \
                Joint policies may renew earlier under section 4 [1, 3].";
    let citations = vec![
        Citation::new("1", "docs://policy/v3")
            .unwrap()
            .with_span("2026-12-31 expiration")
            .with_page(4)
            .with_confidence(0.91)
            .unwrap(),
        Citation::new("2", "docs://renewal/v1")
            .unwrap()
            .with_span("60 days advance notice required")
            .with_page(2)
            .with_confidence(0.87)
            .unwrap(),
        Citation::new("3", "docs://policy/v3")
            .unwrap()
            .with_span("section 4 joint renewals")
            .with_page(11)
            .with_confidence(0.82)
            .unwrap(),
    ];

    let record = store.attach("turn_42", text, citations.clone()).unwrap();
    assert_eq!(record.turn_id, "turn_42");

    let report = validate(text, &citations);
    assert_eq!(report.facts_with_citations, 3);
    assert_eq!(report.facts_missing_citations, 0);
    assert!(report.dangling_citation_ids.is_empty());
    assert!(report.duplicate_citation_ids.is_empty());
    assert!((report.coverage_ratio - 1.0).abs() < f64::EPSILON);
    assert!(report.is_clean());

    let raw = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 1);
    let parsed: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(parsed["turn_id"], "turn_42");
    assert_eq!(parsed["citations"].as_array().unwrap().len(), 3);
}

#[test]
fn detects_hallucinated_marker() {
    let store = CitationStore::new();
    let text = "Real fact [1]. Made-up fact [9].";
    let citations = vec![Citation::new("1", "docs://a").unwrap()];
    store.attach("turn_1", text, citations.clone()).unwrap();
    let report = validate(text, &citations);
    assert_eq!(report.facts_missing_citations, 1);
    assert!(report.markers_in_order.contains(&"9".to_string()));
    assert!(report.as_markdown().contains("needs review"));
}
