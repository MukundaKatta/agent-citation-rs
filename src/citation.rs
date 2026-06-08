use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Citation {
    pub id: String,
    pub source_uri: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub span: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CitationError {
    BlankId,
    BlankSourceUri,
    NegativePage,
    ConfidenceOutOfRange,
}

impl std::fmt::Display for CitationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CitationError::BlankId => f.write_str("Citation.id must be a non-empty string"),
            CitationError::BlankSourceUri => {
                f.write_str("Citation.source_uri must be a non-empty string")
            }
            CitationError::NegativePage => f.write_str("Citation.page must be >= 0 when provided"),
            CitationError::ConfidenceOutOfRange => {
                f.write_str("Citation.confidence must be within [0.0, 1.0]")
            }
        }
    }
}

impl std::error::Error for CitationError {}

impl Citation {
    pub fn new(
        id: impl Into<String>,
        source_uri: impl Into<String>,
    ) -> Result<Self, CitationError> {
        let c = Citation {
            id: id.into(),
            source_uri: source_uri.into(),
            span: None,
            page: None,
            confidence: None,
            metadata: BTreeMap::new(),
        };
        c.validate()?;
        Ok(c)
    }

    pub fn with_span(mut self, span: impl Into<String>) -> Self {
        self.span = Some(span.into());
        self
    }

    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Result<Self, CitationError> {
        if !(0.0..=1.0).contains(&confidence) {
            return Err(CitationError::ConfidenceOutOfRange);
        }
        self.confidence = Some(confidence);
        Ok(self)
    }

    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn validate(&self) -> Result<(), CitationError> {
        if self.id.trim().is_empty() {
            return Err(CitationError::BlankId);
        }
        if self.source_uri.trim().is_empty() {
            return Err(CitationError::BlankSourceUri);
        }
        if let Some(c) = self.confidence {
            if !(0.0..=1.0).contains(&c) {
                return Err(CitationError::ConfidenceOutOfRange);
            }
        }
        Ok(())
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("id".into(), serde_json::Value::String(self.id.clone()));
        map.insert(
            "source_uri".into(),
            serde_json::Value::String(self.source_uri.clone()),
        );
        if let Some(ref s) = self.span {
            map.insert("span".into(), serde_json::Value::String(s.clone()));
        }
        if let Some(p) = self.page {
            map.insert("page".into(), serde_json::Value::Number(p.into()));
        }
        if let Some(c) = self.confidence {
            if let Some(n) = serde_json::Number::from_f64(c) {
                map.insert("confidence".into(), serde_json::Value::Number(n));
            }
        }
        if !self.metadata.is_empty() {
            let m: serde_json::Map<String, serde_json::Value> = self
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            map.insert("metadata".into(), serde_json::Value::Object(m));
        }
        serde_json::Value::Object(map)
    }

    pub fn from_json_value(value: &serde_json::Value) -> Result<Self, CitationError> {
        let obj = value.as_object().ok_or(CitationError::BlankSourceUri)?;
        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let source_uri = obj
            .get("source_uri")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let span = obj
            .get("span")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let page = obj.get("page").and_then(|v| v.as_u64()).map(|p| p as u32);
        let confidence = obj.get("confidence").and_then(|v| v.as_f64());
        let metadata: BTreeMap<String, serde_json::Value> = obj
            .get("metadata")
            .and_then(|v| v.as_object())
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        let c = Citation {
            id,
            source_uri,
            span,
            page,
            confidence,
            metadata,
        };
        c.validate()?;
        Ok(c)
    }
}
