use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::citation::Citation;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributionRecord {
    pub turn_id: String,
    pub text: String,
    pub citations: Vec<Citation>,
    pub captured_at: f64,
}

impl AttributionRecord {
    pub fn new(turn_id: String, text: String, citations: Vec<Citation>) -> Self {
        let captured_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        AttributionRecord {
            turn_id,
            text,
            citations,
            captured_at,
        }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        let cites: Vec<serde_json::Value> =
            self.citations.iter().map(|c| c.to_json_value()).collect();
        let mut map = serde_json::Map::new();
        map.insert(
            "turn_id".into(),
            serde_json::Value::String(self.turn_id.clone()),
        );
        map.insert("text".into(), serde_json::Value::String(self.text.clone()));
        map.insert("citations".into(), serde_json::Value::Array(cites));
        if let Some(n) = serde_json::Number::from_f64(self.captured_at) {
            map.insert("captured_at".into(), serde_json::Value::Number(n));
        }
        serde_json::Value::Object(map)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    BlankTurnId,
    Io(String),
    Parse(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::BlankTurnId => f.write_str("turn_id must be a non-empty string"),
            StoreError::Io(s) => write!(f, "io error: {}", s),
            StoreError::Parse(s) => write!(f, "parse error: {}", s),
        }
    }
}

impl std::error::Error for StoreError {}

pub trait Sink: Send + Sync {
    fn write(&self, record: AttributionRecord) -> Result<(), StoreError>;
    fn read_all(&self) -> Result<Vec<AttributionRecord>, StoreError>;
}

#[derive(Default)]
pub struct InMemorySink {
    records: Arc<Mutex<Vec<AttributionRecord>>>,
}

impl InMemorySink {
    pub fn new() -> Self {
        InMemorySink {
            records: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn clear(&self) {
        if let Ok(mut g) = self.records.lock() {
            g.clear();
        }
    }

    pub fn len(&self) -> usize {
        self.records.lock().map(|g| g.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Sink for InMemorySink {
    fn write(&self, record: AttributionRecord) -> Result<(), StoreError> {
        let mut g = self
            .records
            .lock()
            .map_err(|e| StoreError::Io(e.to_string()))?;
        g.push(record);
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<AttributionRecord>, StoreError> {
        let g = self
            .records
            .lock()
            .map_err(|e| StoreError::Io(e.to_string()))?;
        Ok(g.clone())
    }
}

pub struct JsonlSink {
    path: PathBuf,
}

impl JsonlSink {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| StoreError::Io(e.to_string()))?;
            }
        }
        Ok(JsonlSink { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Sink for JsonlSink {
    fn write(&self, record: AttributionRecord) -> Result<(), StoreError> {
        let payload = serde_json::to_string(&record.to_json_value())
            .map_err(|e| StoreError::Parse(e.to_string()))?;
        let mut fh = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)
            .map_err(|e| StoreError::Io(e.to_string()))?;
        fh.write_all(payload.as_bytes())
            .map_err(|e| StoreError::Io(e.to_string()))?;
        fh.write_all(b"\n").map_err(|e| StoreError::Io(e.to_string()))?;
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<AttributionRecord>, StoreError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let fh = std::fs::File::open(&self.path).map_err(|e| StoreError::Io(e.to_string()))?;
        let rdr = BufReader::new(fh);
        let mut out: Vec<AttributionRecord> = Vec::new();
        for line in rdr.lines() {
            let raw = line.map_err(|e| StoreError::Io(e.to_string()))?;
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            let v: serde_json::Value =
                serde_json::from_str(trimmed).map_err(|e| StoreError::Parse(e.to_string()))?;
            let turn_id = v
                .get("turn_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let text = v
                .get("text")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let captured_at = v
                .get("captured_at")
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0);
            let citations: Vec<Citation> = v
                .get("citations")
                .and_then(|x| x.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|c| Citation::from_json_value(c).ok())
                        .collect()
                })
                .unwrap_or_default();
            out.push(AttributionRecord {
                turn_id,
                text,
                citations,
                captured_at,
            });
        }
        Ok(out)
    }
}

pub struct CitationStore {
    sink: Box<dyn Sink>,
}

impl Default for CitationStore {
    fn default() -> Self {
        CitationStore {
            sink: Box::new(InMemorySink::new()),
        }
    }
}

impl CitationStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sink(sink: Box<dyn Sink>) -> Self {
        CitationStore { sink }
    }

    pub fn sink(&self) -> &dyn Sink {
        self.sink.as_ref()
    }

    pub fn attach(
        &self,
        turn_id: impl Into<String>,
        text: impl Into<String>,
        citations: impl IntoIterator<Item = Citation>,
    ) -> Result<AttributionRecord, StoreError> {
        let turn_id = turn_id.into();
        if turn_id.trim().is_empty() {
            return Err(StoreError::BlankTurnId);
        }
        let record = AttributionRecord::new(turn_id, text.into(), citations.into_iter().collect());
        self.sink.write(record.clone())?;
        Ok(record)
    }

    pub fn export(&self) -> Result<Vec<serde_json::Value>, StoreError> {
        Ok(self
            .sink
            .read_all()?
            .iter()
            .map(|r| r.to_json_value())
            .collect())
    }

    pub fn render_text_summary(&self) -> Result<String, StoreError> {
        let mut out = String::new();
        for record in self.sink.read_all()? {
            let cite_ids: String = if record.citations.is_empty() {
                "-".to_string()
            } else {
                record
                    .citations
                    .iter()
                    .map(|c| c.id.clone())
                    .collect::<Vec<_>>()
                    .join(",")
            };
            out.push_str(&format!(
                "[{}] cites={} len={}\n",
                record.turn_id,
                cite_ids,
                record.text.len()
            ));
        }
        Ok(out)
    }
}
