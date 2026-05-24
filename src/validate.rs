use std::collections::{HashMap, HashSet};

use crate::attribute::{attribute, unique_marker_ids};
use crate::citation::Citation;
use crate::report::ValidationReport;

pub fn validate(text: &str, citations: &[Citation]) -> ValidationReport {
    let citation_ids: Vec<String> = citations.iter().map(|c| c.id.clone()).collect();
    let citation_id_set: HashSet<String> = citation_ids.iter().cloned().collect();

    let mut counts: HashMap<String, usize> = HashMap::new();
    for cid in &citation_ids {
        *counts.entry(cid.clone()).or_insert(0) += 1;
    }
    let mut duplicates: Vec<String> = counts
        .into_iter()
        .filter_map(|(k, v)| if v > 1 { Some(k) } else { None })
        .collect();
    duplicates.sort();

    let markers = attribute(text);
    let ordered_ids = unique_marker_ids(&markers);
    let marker_id_set: HashSet<String> = ordered_ids.iter().cloned().collect();

    let matched: Vec<String> = ordered_ids
        .iter()
        .filter(|m| citation_id_set.contains(*m))
        .cloned()
        .collect();
    let missing: Vec<String> = ordered_ids
        .iter()
        .filter(|m| !citation_id_set.contains(*m))
        .cloned()
        .collect();
    let mut dangling: Vec<String> = citation_id_set
        .difference(&marker_id_set)
        .cloned()
        .collect();
    dangling.sort();

    let coverage = if ordered_ids.is_empty() {
        1.0
    } else {
        matched.len() as f64 / ordered_ids.len() as f64
    };

    ValidationReport {
        facts_with_citations: matched.len(),
        facts_missing_citations: missing.len(),
        dangling_citation_ids: dangling,
        duplicate_citation_ids: duplicates,
        coverage_ratio: coverage,
        markers_in_order: ordered_ids,
    }
}
