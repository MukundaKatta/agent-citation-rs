/*!
`agent-citation`: WHERE-layer structured citations for AI agent outputs.

Given an LLM-generated answer with bracketed markers like `[1]`, `[2, 3]`, this
crate lets you:

- attach source-of-truth [`Citation`] records (uri, span, page, confidence,
  metadata) to an agent turn,
- capture each turn into a pluggable [`Sink`] ([`InMemorySink`] for tests,
  [`JsonlSink`] for audit, or bring your own backend),
- structurally [`validate`] the answer: every marker has a backing citation, no
  dangling citations, no duplicate ids.

The check is *structural*, not semantic: it verifies that markers and citations
line up, not that the cited source actually says what the agent claims.

Pairs with `agent-decision-log` (WHY), `agentsnap` (CALLS), and `agenttrace`
(COST) in the agent observability stack.

# Validate an answer

```
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

# Attach + audit a turn

```
use agent_citation::{Citation, CitationStore};

let store = CitationStore::new(); // defaults to an in-memory sink
store
    .attach(
        "turn_42",
        "Expires 2026-12-31 [1].",
        vec![Citation::new("1", "docs://policy/v3")
            .unwrap()
            .with_span("expires 2026-12-31")
            .with_page(4)
            .with_confidence(0.91)
            .unwrap()],
    )
    .unwrap();

let exported = store.export().unwrap();
assert_eq!(exported.len(), 1);
assert_eq!(exported[0]["turn_id"], "turn_42");
```
*/

#![forbid(unsafe_code)]

mod attribute;
mod citation;
mod report;
mod store;
mod validate;

pub use attribute::{attribute, unique_marker_ids, Marker};
pub use citation::{Citation, CitationError};
pub use report::ValidationReport;
pub use store::{AttributionRecord, CitationStore, InMemorySink, JsonlSink, Sink, StoreError};
pub use validate::validate;
