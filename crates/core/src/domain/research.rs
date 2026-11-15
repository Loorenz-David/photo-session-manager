use serde::{Deserialize, Serialize};

use super::attribute::AiConfidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackAction {
    Accepted,
    Cleared,
    Edited,
    KeptPrevious,
}

impl FeedbackAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Cleared => "cleared",
            Self::Edited => "edited",
            Self::KeptPrevious => "kept_previous",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        [
            Self::Accepted,
            Self::Cleared,
            Self::Edited,
            Self::KeptPrevious,
        ]
        .into_iter()
        .find(|action| action.as_str() == s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feedback {
    pub id: i64,
    pub item_id: i64,
    pub key: String,
    pub suggested_value: String,
    pub confidence: AiConfidence,
    pub final_value: Option<String>,
    pub action: FeedbackAction,
    pub at: i64,
    pub step_token: Option<String>,
}
