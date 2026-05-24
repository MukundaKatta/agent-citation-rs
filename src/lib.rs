//! WHERE-layer structured citations for AI agent outputs.
//!
//! Sibling to `agent-decision-log` (WHY), `agentsnap` (CALLS), and `agenttrace` (COST).
//!
//! # Quick start
//!
//! Validate that every bracketed marker in an agent reply has a backing citation.
//!
//! ```
//! use agent_citation::{Citation, validate};
//!
//! let text = "The policy expires 2026-12-31 [1].";
//! let citations = vec![Citation::new("1", "docs://policy/v3").unwrap()];
//! let report = validate(text, &citations);
//! assert_eq!(report.facts_with_citations, 1);
//! assert!(report.is_clean());
//! ```
//!
//! Capture a turn-scoped attribution record into the default in-memory store.
//!
//! ```
//! use agent_citation::{Citation, CitationStore};
//!
//! let store = CitationStore::new();
//! let record = store
//!     .attach(
//!         "turn_1",
//!         "Effective immediately [1].",
//!         vec![Citation::new("1", "docs://policy/v3").unwrap()],
//!     )
//!     .unwrap();
//! assert_eq!(record.turn_id, "turn_1");
//! ```

mod attribute;
mod citation;
mod report;
mod store;
mod validate;

pub use attribute::{attribute, unique_marker_ids, Marker};
pub use citation::{Citation, CitationError};
pub use report::ValidationReport;
pub use store::{
    AttributionRecord, CitationStore, InMemorySink, JsonlSink, Sink, StoreError,
};
pub use validate::validate;
