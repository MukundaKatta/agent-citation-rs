# agent-citation

[![CI](https://github.com/MukundaKatta/agent-citation-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/MukundaKatta/agent-citation-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

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

## Marker syntax

`validate` and the underlying `attribute` parser recognize bracketed numeric
markers only:

- `[1]` -- a single citation marker.
- `[1, 2]` -- a group; expands to markers `1` and `2`, both pointing at the same
  text span.
- `[12]`, `[101]` -- multi-digit ids are fine.

Anything that is not a comma-separated list of digits is ignored, so prose like
`[Appendix A]`, empty brackets `[]`, and unclosed `[1` are left untouched.
Repeated markers (`[1] ... [1]`) are de-duplicated when computing coverage.

## API at a glance

| Item | Purpose |
| --- | --- |
| `Citation::new(id, source_uri)` | Build a validated citation (rejects blank id / uri). |
| `.with_span` / `.with_page` / `.with_confidence` / `.with_metadata` | Builder methods for optional fields (confidence must be in `[0.0, 1.0]`). |
| `validate(text, &citations) -> ValidationReport` | Structurally check markers against citations. |
| `ValidationReport::is_clean()` / `as_markdown()` | Inspect or render the result. |
| `CitationStore::new()` | Capture turns into the default in-memory sink. |
| `CitationStore::with_sink(Box<dyn Sink>)` | Capture turns into a custom backend. |
| `store.attach(turn_id, text, citations)` | Record one agent turn. |
| `store.export()` / `store.render_text_summary()` | Read captured turns back as JSON or a text digest. |
| `InMemorySink`, `JsonlSink`, `Sink` | Built-in and pluggable sinks. |

`ValidationReport` carries `facts_with_citations`, `facts_missing_citations`,
`dangling_citation_ids`, `duplicate_citation_ids`, `coverage_ratio`, and
`markers_in_order`.

## Development

```sh
cargo test                 # unit, integration, and doc tests
cargo fmt --all -- --check  # formatting
cargo clippy --all-targets -- -D warnings
```

CI runs the same checks on every push and pull request.

## License

MIT.
