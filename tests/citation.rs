use agent_citation::{Citation, CitationError};

#[test]
fn minimal_fields_only() {
    let c = Citation::new("1", "docs://policy/v3").unwrap();
    assert_eq!(c.id, "1");
    assert_eq!(c.source_uri, "docs://policy/v3");
    assert!(c.span.is_none());
    assert!(c.page.is_none());
    assert!(c.confidence.is_none());
    assert!(c.metadata.is_empty());
}

#[test]
fn full_fields_roundtrip() {
    let c = Citation::new("policy-v3-p4", "https://example.com/policy.pdf")
        .unwrap()
        .with_span("expires 2026-12-31")
        .with_page(4)
        .with_confidence(0.92)
        .unwrap()
        .with_metadata("retriever", "bge-large")
        .with_metadata("score", 0.81);
    let v = c.to_json_value();
    let back = Citation::from_json_value(&v).unwrap();
    assert_eq!(back, c);
}

#[test]
fn to_dict_drops_empty_fields() {
    let c = Citation::new("1", "docs://x").unwrap();
    let v = c.to_json_value();
    let obj = v.as_object().unwrap();
    assert!(!obj.contains_key("span"));
    assert!(!obj.contains_key("page"));
    assert!(!obj.contains_key("confidence"));
    assert!(!obj.contains_key("metadata"));
}

#[test]
fn rejects_blank_id() {
    let r = Citation::new("   ", "docs://x");
    assert_eq!(r, Err(CitationError::BlankId));
}

#[test]
fn rejects_blank_source_uri() {
    let r = Citation::new("1", "");
    assert_eq!(r, Err(CitationError::BlankSourceUri));
}

#[test]
fn rejects_out_of_range_confidence() {
    let c = Citation::new("1", "docs://x").unwrap();
    let r = c.with_confidence(1.5);
    assert!(matches!(r, Err(CitationError::ConfidenceOutOfRange)));
}

#[test]
fn metadata_roundtrip_preserves_arbitrary_json() {
    let c = Citation::new("1", "docs://x")
        .unwrap()
        .with_metadata("nested", serde_json::json!({"k": [1, 2, 3]}));
    let v = c.to_json_value();
    let back = Citation::from_json_value(&v).unwrap();
    assert_eq!(
        back.metadata.get("nested"),
        Some(&serde_json::json!({"k": [1, 2, 3]}))
    );
}
