# agent-citation

WHERE-layer structured citations for AI agent outputs.

Given an LLM-generated answer with bracketed markers like `[1]`, `[2, 3]`, this crate lets you:

- attach source-of-truth `Citation` records (uri, span, page, confidence, metadata)
- capture each turn into a pluggable `Sink` (in-memory for tests, JSONL for audit, BYO for a database)
- structurally validate the answer: every marker has a backing citation, no dangling citations, no duplicates

Sibling crates in the agent observability quadrant:

- `agent-decision-log` -- WHY layer
- `agentsnap` -- CALLS layer
- `agenttrace` -- COST layer

## Install

```toml
[dependencies]
agent-citation = "0.1"
```

## Validate

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
println!("{}", report.as_markdown());
```

The check is structural, not semantic. It does not try to verify whether the cited source actually says what the agent claims. It only checks that markers and citations match up.

## Attach + audit

```rust
use agent_citation::{Citation, CitationStore, JsonlSink};

let sink = JsonlSink::new("/tmp/citations.jsonl").unwrap();
let store = CitationStore::with_sink(Box::new(sink));

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
```

Each `attach` call writes one JSON line. Tail-friendly. Greppable. Restartable.

## Custom sink

Implement `Sink` for any backend.

```rust
use agent_citation::{AttributionRecord, Sink, StoreError};
use std::sync::{Arc, Mutex};

struct CountingSink {
    records: Arc<Mutex<Vec<AttributionRecord>>>,
}

impl Sink for CountingSink {
    fn write(&self, record: AttributionRecord) -> Result<(), StoreError> {
        self.records.lock().unwrap().push(record);
        Ok(())
    }
    fn read_all(&self) -> Result<Vec<AttributionRecord>, StoreError> {
        Ok(self.records.lock().unwrap().clone())
    }
}
```

## License

MIT.
