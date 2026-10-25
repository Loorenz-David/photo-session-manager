use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "hook", rename_all = "snake_case")]
pub enum HookEvent {
    CaptureDone {
        argv: Vec<String>,
        #[serde(default)]
        last_captured_file: Option<String>,
    },
    ProcessingDone {
        job: String,
        raw: String,
        outputs: Vec<String>,
    },
    VariantAdjusted {
        variant_id: String,
        raw: String,
    },
}

impl HookEvent {
    pub fn hook(&self) -> &'static str {
        match self {
            Self::CaptureDone { .. } => "capture_done",
            Self::ProcessingDone { .. } => "processing_done",
            Self::VariantAdjusted { .. } => "variant_adjusted",
        }
    }
    pub fn candidates(&self) -> Vec<String> {
        match self {
            Self::CaptureDone {
                argv,
                last_captured_file,
            } => argv
                .iter()
                .cloned()
                .chain(last_captured_file.iter().cloned())
                .collect(),
            _ => Vec::new(),
        }
    }
}

pub fn parse(json: &[u8]) -> Result<HookEvent> {
    serde_json::from_slice(json).map_err(|e| Error::InvalidData(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mc15_capture_all_argv_and_last_file() {
        let event = parse(
            br#"{"hook":"capture_done","argv":["/a.ARW","/b.ARW"],"last_captured_file":"/c.ARW"}"#,
        )
        .unwrap();
        assert_eq!(event.candidates(), ["/a.ARW", "/b.ARW", "/c.ARW"]);
    }
    #[test]
    fn mc15_processing_and_adjusted_shapes() {
        assert_eq!(
            parse(br#"{"hook":"processing_done","job":"j","raw":"/a.ARW","outputs":["/a.jpg"]}"#)
                .unwrap(),
            HookEvent::ProcessingDone {
                job: "j".into(),
                raw: "/a.ARW".into(),
                outputs: vec!["/a.jpg".into()]
            }
        );
        assert_eq!(
            parse(br#"{"hook":"variant_adjusted","variant_id":"v","raw":"/a.ARW"}"#).unwrap(),
            HookEvent::VariantAdjusted {
                variant_id: "v".into(),
                raw: "/a.ARW".into()
            }
        );
    }
}
