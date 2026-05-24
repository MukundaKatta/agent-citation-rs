use agent_citation::{attribute, unique_marker_ids};

#[test]
fn finds_single_marker() {
    let text = "The policy expires 2026-12-31 [1].";
    let markers = attribute(text);
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].id, "1");
    assert_eq!(&text[markers[0].start..markers[0].end], "[1]");
}

#[test]
fn finds_multiple_markers_in_order() {
    let text = "A [1]. B [2]. C [3].";
    let ids: Vec<String> = attribute(text).into_iter().map(|m| m.id).collect();
    assert_eq!(ids, vec!["1", "2", "3"]);
}

#[test]
fn expands_grouped_markers() {
    let text = "Both apply here [1, 2].";
    let markers = attribute(text);
    let ids: Vec<String> = markers.iter().map(|m| m.id.clone()).collect();
    assert_eq!(ids, vec!["1", "2"]);
    assert_eq!(markers[0].start, markers[1].start);
    assert_eq!(markers[0].end, markers[1].end);
}

#[test]
fn ignores_non_numeric_brackets() {
    let text = "See [Appendix A] for details.";
    assert!(attribute(text).is_empty());
}

#[test]
fn empty_for_clean_text() {
    assert!(attribute("No citations here at all.").is_empty());
}

#[test]
fn unique_marker_ids_dedupes_in_order() {
    let text = "A [1]. B [2]. C [1]. D [3].";
    let markers = attribute(text);
    assert_eq!(unique_marker_ids(&markers), vec!["1", "2", "3"]);
}

#[test]
fn double_digit_ids() {
    let text = "Big id [12] and [101].";
    let ids: Vec<String> = attribute(text).into_iter().map(|m| m.id).collect();
    assert_eq!(ids, vec!["12", "101"]);
}

#[test]
fn open_bracket_without_close_is_ignored() {
    assert!(attribute("Unclosed [1 still text").is_empty());
}

#[test]
fn empty_brackets_are_ignored() {
    assert!(attribute("Empty [] noted.").is_empty());
}
