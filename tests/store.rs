use agent_citation::{
    AttributionRecord, Citation, CitationStore, InMemorySink, JsonlSink, Sink, StoreError,
};
use std::sync::{Arc, Mutex};

#[test]
fn defaults_to_memory_sink() {
    let store = CitationStore::new();
    let summary = store.render_text_summary().unwrap();
    assert!(summary.is_empty());
}

#[test]
fn attach_and_export_roundtrip() {
    let store = CitationStore::new();
    store
        .attach(
            "turn_1",
            "A [1].",
            vec![Citation::new("1", "docs://a")
                .unwrap()
                .with_span("A fact")],
        )
        .unwrap();
    let out = store.export().unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0]["turn_id"], "turn_1");
    assert_eq!(out[0]["citations"][0]["id"], "1");
}

#[test]
fn attach_accepts_iterator_of_citations() {
    let store = CitationStore::new();
    let cites = vec![
        Citation::new("1", "docs://a").unwrap(),
        Citation::new("2", "docs://b").unwrap(),
    ];
    store.attach("turn_1", "A [1]. B [2].", cites).unwrap();
    let out = store.export().unwrap();
    let ids: Vec<String> = out[0]["citations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap().to_string())
        .collect();
    assert!(ids.contains(&"1".to_string()));
    assert!(ids.contains(&"2".to_string()));
}

#[test]
fn attach_rejects_blank_turn_id() {
    let store = CitationStore::new();
    let r = store.attach("", "text", Vec::<Citation>::new());
    assert!(matches!(r, Err(StoreError::BlankTurnId)));
}

#[test]
fn in_memory_sink_clear() {
    let sink = InMemorySink::new();
    sink.write(AttributionRecord::new(
        "turn_1".into(),
        "A [1].".into(),
        vec![Citation::new("1", "docs://a").unwrap()],
    ))
    .unwrap();
    assert_eq!(sink.len(), 1);
    sink.clear();
    assert!(sink.is_empty());
}

#[test]
fn render_text_summary_lists_each_turn() {
    let store = CitationStore::new();
    store
        .attach(
            "turn_1",
            "A [1].",
            vec![Citation::new("1", "docs://a").unwrap()],
        )
        .unwrap();
    store
        .attach(
            "turn_2",
            "B [2].",
            vec![Citation::new("2", "docs://b").unwrap()],
        )
        .unwrap();
    let summary = store.render_text_summary().unwrap();
    assert!(summary.contains("turn_1"));
    assert!(summary.contains("turn_2"));
    assert!(summary.contains("cites=1"));
}

#[test]
fn jsonl_sink_writes_and_reads_back() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("citations.jsonl");
    let sink = JsonlSink::new(&path).unwrap();
    let store = CitationStore::with_sink(Box::new(sink));
    store
        .attach(
            "turn_1",
            "A [1].",
            vec![Citation::new("1", "docs://a").unwrap().with_page(2)],
        )
        .unwrap();
    let raw = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = raw.lines().collect();
    assert_eq!(lines.len(), 1);
    let parsed: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(parsed["turn_id"], "turn_1");
    assert_eq!(parsed["citations"][0]["page"], 2);
}

#[test]
fn jsonl_sink_appends_across_calls() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("citations.jsonl");
    let sink = JsonlSink::new(&path).unwrap();
    let store = CitationStore::with_sink(Box::new(sink));
    store
        .attach("t1", "A [1].", vec![Citation::new("1", "docs://a").unwrap()])
        .unwrap();
    store
        .attach("t2", "B [2].", vec![Citation::new("2", "docs://b").unwrap()])
        .unwrap();
    let sink2 = JsonlSink::new(&path).unwrap();
    let records = sink2.read_all().unwrap();
    let ids: Vec<String> = records.iter().map(|r| r.turn_id.clone()).collect();
    assert_eq!(ids, vec!["t1".to_string(), "t2".to_string()]);
}

#[test]
fn jsonl_sink_read_all_on_missing_file_is_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("nope.jsonl");
    let sink = JsonlSink::new(&path).unwrap();
    assert!(sink.read_all().unwrap().is_empty());
}

#[test]
fn jsonl_sink_creates_parent_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("nested/deeper/audit.jsonl");
    let sink = JsonlSink::new(&path).unwrap();
    let store = CitationStore::with_sink(Box::new(sink));
    store
        .attach("t1", "A [1].", vec![Citation::new("1", "docs://a").unwrap()])
        .unwrap();
    assert!(path.exists());
}

#[test]
fn jsonl_sink_skips_blank_lines() {
    use std::io::Write;
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("audit.jsonl");
    let sink = JsonlSink::new(&path).unwrap();
    let store = CitationStore::with_sink(Box::new(sink));
    store
        .attach("t1", "A [1].", vec![Citation::new("1", "docs://a").unwrap()])
        .unwrap();
    let mut fh = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    fh.write_all(b"\n   \n").unwrap();
    let sink2 = JsonlSink::new(&path).unwrap();
    let records = sink2.read_all().unwrap();
    assert_eq!(records.len(), 1);
}

struct CountingSink {
    records: Arc<Mutex<Vec<AttributionRecord>>>,
}

impl Sink for CountingSink {
    fn write(&self, record: AttributionRecord) -> Result<(), StoreError> {
        self.records
            .lock()
            .map_err(|e| StoreError::Io(e.to_string()))?
            .push(record);
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<AttributionRecord>, StoreError> {
        Ok(self
            .records
            .lock()
            .map_err(|e| StoreError::Io(e.to_string()))?
            .clone())
    }
}

#[test]
fn custom_sink_trait_works() {
    let shared = Arc::new(Mutex::new(Vec::<AttributionRecord>::new()));
    let sink = CountingSink {
        records: shared.clone(),
    };
    let store = CitationStore::with_sink(Box::new(sink));
    store
        .attach("t1", "A [1].", vec![Citation::new("1", "docs://a").unwrap()])
        .unwrap();
    store
        .attach("t2", "B [2].", vec![Citation::new("2", "docs://b").unwrap()])
        .unwrap();
    let snap = shared.lock().unwrap();
    assert_eq!(snap.len(), 2);
    let ids: std::collections::HashSet<String> = snap.iter().map(|r| r.turn_id.clone()).collect();
    assert!(ids.contains("t1"));
    assert!(ids.contains("t2"));
}
