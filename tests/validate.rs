use agent_citation::{validate, Citation, ValidationReport};

#[test]
fn full_coverage() {
    let text = "Expires 2026-12-31 [1]. Notice 60 days [2].";
    let citations = vec![
        Citation::new("1", "docs://policy/v3").unwrap(),
        Citation::new("2", "docs://renewal/v1").unwrap(),
    ];
    let report = validate(text, &citations);
    assert_eq!(report.facts_with_citations, 2);
    assert_eq!(report.facts_missing_citations, 0);
    assert!((report.coverage_ratio - 1.0).abs() < f64::EPSILON);
    assert!(report.is_clean());
}

#[test]
fn flags_orphan_markers() {
    let text = "Fact A [1]. Fact B [99].";
    let citations = vec![Citation::new("1", "docs://a").unwrap()];
    let report = validate(text, &citations);
    assert_eq!(report.facts_with_citations, 1);
    assert_eq!(report.facts_missing_citations, 1);
    assert!((report.coverage_ratio - 0.5).abs() < f64::EPSILON);
    assert!(!report.is_clean());
}

#[test]
fn flags_dangling_citations() {
    let text = "Fact A [1].";
    let citations = vec![
        Citation::new("1", "docs://a").unwrap(),
        Citation::new("2", "docs://b").unwrap(),
    ];
    let report = validate(text, &citations);
    assert_eq!(report.dangling_citation_ids, vec!["2".to_string()]);
    assert!(!report.is_clean());
}

#[test]
fn flags_duplicate_citation_ids() {
    let text = "Fact A [1].";
    let citations = vec![
        Citation::new("1", "docs://a").unwrap(),
        Citation::new("1", "docs://a-mirror").unwrap(),
    ];
    let report = validate(text, &citations);
    assert_eq!(report.duplicate_citation_ids, vec!["1".to_string()]);
    assert!(!report.is_clean());
}

#[test]
fn empty_text_is_clean() {
    let report = validate("", &[]);
    assert!((report.coverage_ratio - 1.0).abs() < f64::EPSILON);
    assert!(report.is_clean());
}

#[test]
fn no_markers_with_dangling_citations() {
    let report = validate(
        "Plain answer with no markers.",
        &[Citation::new("1", "docs://a").unwrap()],
    );
    assert_eq!(report.facts_missing_citations, 0);
    assert_eq!(report.dangling_citation_ids, vec!["1".to_string()]);
    assert!((report.coverage_ratio - 1.0).abs() < f64::EPSILON);
    assert!(!report.is_clean());
}

#[test]
fn unique_dedupes_repeated_markers() {
    let text = "A [1] and again [1] and once more [1].";
    let citations = vec![Citation::new("1", "docs://a").unwrap()];
    let report = validate(text, &citations);
    assert_eq!(report.facts_with_citations, 1);
    assert_eq!(report.markers_in_order, vec!["1".to_string()]);
}

#[test]
fn report_default_is_clean() {
    let r = ValidationReport::default();
    assert!(r.is_clean());
}

#[test]
fn markdown_clean_mentions_status_clean() {
    let r = ValidationReport {
        facts_with_citations: 1,
        coverage_ratio: 1.0,
        ..Default::default()
    };
    let md = r.as_markdown();
    assert!(md.contains("Status: clean"));
    assert!(md.contains("Coverage ratio: **1.00**"));
}

#[test]
fn markdown_dirty_mentions_needs_review() {
    let r = ValidationReport {
        facts_with_citations: 1,
        facts_missing_citations: 1,
        dangling_citation_ids: vec!["7".to_string()],
        duplicate_citation_ids: vec!["1".to_string()],
        coverage_ratio: 0.5,
        ..Default::default()
    };
    let md = r.as_markdown();
    assert!(md.contains("Status: needs review"));
    assert!(md.contains("Dangling citation ids: `7`"));
    assert!(md.contains("Duplicate citation ids: `1`"));
}
