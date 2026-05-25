/*!
agent-citation: WHERE-layer structured citations for AI agent outputs.

Attach source attribution to every claim your agent makes. `attribute()`
creates a `Citation` and appends it to a `CitationStore`. `validate()` checks
required fields. Pairs with `agent-decision-log` (WHY), `agentsnap` (CALLS),
and `agenttrace` (COST) in the agent observability stack.

```rust
use agent_citation::{CitationStore, Citation};

let mut store = CitationStore::new();
let c = store.attribute("Wikipedia", "Water boils at 100°C", None, None);
assert_eq!(store.all().len(), 1);
assert!(!c.id.is_empty());
```
*/

use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("cit_{:016x}_{:04x}", ts.wrapping_mul(2654435761), n)
}

// ---- Citation -------------------------------------------------------------

/// A single source attribution record.
#[derive(Debug, Clone, PartialEq)]
pub struct Citation {
    pub id: String,
    pub source: String,
    pub text: String,
    pub url: Option<String>,
    pub metadata: Map<String, Value>,
    pub timestamp: f64,
}

impl Citation {
    /// Serialize to a JSON `Value`.
    pub fn to_json(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("id".to_owned(), Value::String(self.id.clone()));
        m.insert("source".to_owned(), Value::String(self.source.clone()));
        m.insert("text".to_owned(), Value::String(self.text.clone()));
        if let Some(ref url) = self.url {
            m.insert("url".to_owned(), Value::String(url.clone()));
        }
        if !self.metadata.is_empty() {
            m.insert("metadata".to_owned(), Value::Object(self.metadata.clone()));
        }
        m.insert("timestamp".to_owned(), Value::from(self.timestamp));
        Value::Object(m)
    }
}

// ---- CitationError --------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CitationError {
    MissingSource,
    MissingText,
    DuplicateId(String),
}

impl std::fmt::Display for CitationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CitationError::MissingSource => write!(f, "CitationError: source is empty"),
            CitationError::MissingText => write!(f, "CitationError: text is empty"),
            CitationError::DuplicateId(id) => write!(f, "CitationError: duplicate id {}", id),
        }
    }
}

impl std::error::Error for CitationError {}

// ---- CitationStore --------------------------------------------------------

/// Append-only store for citations.
#[derive(Debug, Default, Clone)]
pub struct CitationStore {
    citations: Vec<Citation>,
    index: HashMap<String, usize>,
}

impl CitationStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create and store a citation. Returns the created `Citation`.
    pub fn attribute(
        &mut self,
        source: &str,
        text: &str,
        url: Option<&str>,
        metadata: Option<Map<String, Value>>,
    ) -> Citation {
        let c = Citation {
            id: new_id(),
            source: source.to_owned(),
            text: text.to_owned(),
            url: url.map(|s| s.to_owned()),
            metadata: metadata.unwrap_or_default(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
        };
        let idx = self.citations.len();
        self.index.insert(c.id.clone(), idx);
        self.citations.push(c.clone());
        c
    }

    /// Validate a citation (check required fields and no duplicate id).
    pub fn validate(&self, c: &Citation) -> Result<(), CitationError> {
        if c.source.trim().is_empty() {
            return Err(CitationError::MissingSource);
        }
        if c.text.trim().is_empty() {
            return Err(CitationError::MissingText);
        }
        if self.index.contains_key(&c.id) {
            return Err(CitationError::DuplicateId(c.id.clone()));
        }
        Ok(())
    }

    /// Look up a citation by id.
    pub fn find(&self, id: &str) -> Option<&Citation> {
        let idx = self.index.get(id)?;
        self.citations.get(*idx)
    }

    /// All citations in insertion order.
    pub fn all(&self) -> &[Citation] {
        &self.citations
    }

    /// Number of citations.
    pub fn len(&self) -> usize {
        self.citations.len()
    }

    /// True if no citations have been added.
    pub fn is_empty(&self) -> bool {
        self.citations.is_empty()
    }

    /// Citations from a specific source.
    pub fn by_source(&self, source: &str) -> Vec<&Citation> {
        self.citations.iter().filter(|c| c.source == source).collect()
    }

    /// Serialize all citations to a JSON array.
    pub fn to_json(&self) -> Value {
        Value::Array(self.citations.iter().map(|c| c.to_json()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribute_creates_citation() {
        let mut s = CitationStore::new();
        let c = s.attribute("Wikipedia", "Water boils at 100C", None, None);
        assert_eq!(c.source, "Wikipedia");
        assert_eq!(c.text, "Water boils at 100C");
        assert!(c.url.is_none());
        assert!(!c.id.is_empty());
    }

    #[test]
    fn attribute_with_url() {
        let mut s = CitationStore::new();
        let c = s.attribute("Wiki", "text", Some("https://en.wikipedia.org"), None);
        assert_eq!(c.url, Some("https://en.wikipedia.org".to_string()));
    }

    #[test]
    fn store_len_increments() {
        let mut s = CitationStore::new();
        assert_eq!(s.len(), 0);
        s.attribute("A", "text", None, None);
        s.attribute("B", "text", None, None);
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn find_by_id() {
        let mut s = CitationStore::new();
        let c = s.attribute("Source", "content", None, None);
        let found = s.find(&c.id).unwrap();
        assert_eq!(found.id, c.id);
    }

    #[test]
    fn find_missing_returns_none() {
        let s = CitationStore::new();
        assert!(s.find("nonexistent").is_none());
    }

    #[test]
    fn all_returns_slice() {
        let mut s = CitationStore::new();
        s.attribute("A", "1", None, None);
        s.attribute("B", "2", None, None);
        assert_eq!(s.all().len(), 2);
    }

    #[test]
    fn by_source_filters() {
        let mut s = CitationStore::new();
        s.attribute("Wiki", "a", None, None);
        s.attribute("Wiki", "b", None, None);
        s.attribute("News", "c", None, None);
        assert_eq!(s.by_source("Wiki").len(), 2);
        assert_eq!(s.by_source("News").len(), 1);
        assert_eq!(s.by_source("Other").len(), 0);
    }

    #[test]
    fn validate_ok() {
        let s = CitationStore::new();
        let c = Citation {
            id: "fresh-id".to_string(),
            source: "Source".to_string(),
            text: "Text".to_string(),
            url: None,
            metadata: serde_json::Map::new(),
            timestamp: 0.0,
        };
        assert!(s.validate(&c).is_ok());
    }

    #[test]
    fn validate_empty_source_fails() {
        let s = CitationStore::new();
        let c = Citation {
            id: "id1".to_string(),
            source: "  ".to_string(),
            text: "Text".to_string(),
            url: None,
            metadata: serde_json::Map::new(),
            timestamp: 0.0,
        };
        assert_eq!(s.validate(&c), Err(CitationError::MissingSource));
    }

    #[test]
    fn validate_empty_text_fails() {
        let s = CitationStore::new();
        let c = Citation {
            id: "id1".to_string(),
            source: "Source".to_string(),
            text: "".to_string(),
            url: None,
            metadata: serde_json::Map::new(),
            timestamp: 0.0,
        };
        assert_eq!(s.validate(&c), Err(CitationError::MissingText));
    }

    #[test]
    fn validate_duplicate_id_fails() {
        let mut s = CitationStore::new();
        let c = s.attribute("S", "T", None, None);
        let c2 = Citation { id: c.id.clone(), ..c.clone() };
        assert!(matches!(s.validate(&c2), Err(CitationError::DuplicateId(_))));
    }

    #[test]
    fn is_empty_initially() {
        let s = CitationStore::new();
        assert!(s.is_empty());
    }

    #[test]
    fn to_json_is_array() {
        let mut s = CitationStore::new();
        s.attribute("S", "T", None, None);
        let j = s.to_json();
        assert!(j.is_array());
        assert_eq!(j.as_array().unwrap().len(), 1);
    }

    #[test]
    fn citation_to_json_has_fields() {
        let mut s = CitationStore::new();
        let c = s.attribute("Source", "Text", Some("https://example.com"), None);
        let j = c.to_json();
        assert_eq!(j["source"], "Source");
        assert_eq!(j["text"], "Text");
        assert_eq!(j["url"], "https://example.com");
        assert!(!j["id"].as_str().unwrap().is_empty());
    }

    #[test]
    fn unique_ids() {
        let mut s = CitationStore::new();
        let a = s.attribute("A", "1", None, None);
        let b = s.attribute("B", "2", None, None);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn with_metadata() {
        let mut s = CitationStore::new();
        let mut meta = serde_json::Map::new();
        meta.insert("page".to_string(), serde_json::json!(42));
        let c = s.attribute("Book", "quote", None, Some(meta));
        assert_eq!(c.metadata["page"], 42);
    }

    #[test]
    fn error_display() {
        assert!(CitationError::MissingSource.to_string().contains("source"));
        assert!(CitationError::MissingText.to_string().contains("text"));
        assert!(CitationError::DuplicateId("x".to_string()).to_string().contains("x"));
    }
}
