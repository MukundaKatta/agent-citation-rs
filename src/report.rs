use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub facts_with_citations: usize,
    pub facts_missing_citations: usize,
    pub dangling_citation_ids: Vec<String>,
    pub duplicate_citation_ids: Vec<String>,
    pub coverage_ratio: f64,
    pub markers_in_order: Vec<String>,
}

impl Default for ValidationReport {
    fn default() -> Self {
        ValidationReport {
            facts_with_citations: 0,
            facts_missing_citations: 0,
            dangling_citation_ids: Vec::new(),
            duplicate_citation_ids: Vec::new(),
            coverage_ratio: 1.0,
            markers_in_order: Vec::new(),
        }
    }
}

impl ValidationReport {
    pub fn is_clean(&self) -> bool {
        self.facts_missing_citations == 0
            && self.dangling_citation_ids.is_empty()
            && self.duplicate_citation_ids.is_empty()
    }

    pub fn as_markdown(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("# Citation validation report".to_string());
        lines.push(String::new());
        lines.push(format!(
            "- Facts with citations: **{}**",
            self.facts_with_citations
        ));
        lines.push(format!(
            "- Facts missing citations: **{}**",
            self.facts_missing_citations
        ));
        lines.push(format!(
            "- Coverage ratio: **{:.2}**",
            self.coverage_ratio
        ));
        if !self.dangling_citation_ids.is_empty() {
            lines.push(format!(
                "- Dangling citation ids: `{}`",
                self.dangling_citation_ids.join(", ")
            ));
        }
        if !self.duplicate_citation_ids.is_empty() {
            lines.push(format!(
                "- Duplicate citation ids: `{}`",
                self.duplicate_citation_ids.join(", ")
            ));
        }
        lines.push(String::new());
        if self.is_clean() {
            lines.push("Status: clean".to_string());
        } else {
            lines.push("Status: needs review".to_string());
        }
        lines.join("\n")
    }
}
