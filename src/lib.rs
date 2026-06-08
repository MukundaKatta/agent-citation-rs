/*!
agent-citation: WHERE-layer structured citations for AI agent outputs.

Given an LLM-generated answer with bracketed markers like `[1]`, `[2, 3]`,
this crate lets you attach source-of-truth [`Citation`] records, capture each
turn into a pluggable [`Sink`] via a [`CitationStore`], and structurally
[`validate`] that every marker has a backing citation with no dangling or
duplicate ids.

Pairs with `agent-decision-log` (WHY), `agentsnap` (CALLS), and `agenttrace`
(COST) in the agent observability stack.

```rust
use agent_citation::{validate, Citation};

let text = "The policy expires 2026-12-31 [1]. Renewal needs 60 days notice [2].";
let citations = vec![
    Citation::new("1", "docs://policy/v3").unwrap(),
    Citation::new("2", "docs://renewal/v1").unwrap(),
];

let report = validate(text, &citations);
assert!(report.is_clean());
assert_eq!(report.facts_with_citations, 2);
```
*/

pub mod attribute;
pub mod citation;
pub mod report;
pub mod store;
pub mod validate;

pub use attribute::{attribute, unique_marker_ids, Marker};
pub use citation::{Citation, CitationError};
pub use report::ValidationReport;
pub use store::{AttributionRecord, CitationStore, InMemorySink, JsonlSink, Sink, StoreError};
pub use validate::validate;
