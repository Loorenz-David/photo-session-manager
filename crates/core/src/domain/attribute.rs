//! The generic attribute model (ch08): core-typed definitions, values in canonical form with
//! provenance (05A MC-9), presets per item type and step. Nothing platform-specific (H7).

use serde::{Deserialize, Serialize};

/// The 18 core types of ch08 §Schema. Integrations translate their own types into these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreType {
    Text,
    Textarea,
    RichText,
    Integer,
    Decimal,
    Money,
    Dimension,
    Weight,
    Boolean,
    Date,
    Color,
    Rating,
    Url,
    Tags,
    Choice,
    MultiChoice,
    MediaList,
    Reference,
}

impl CoreType {
    pub const ALL: [CoreType; 18] = [
        Self::Text,
        Self::Textarea,
        Self::RichText,
        Self::Integer,
        Self::Decimal,
        Self::Money,
        Self::Dimension,
        Self::Weight,
        Self::Boolean,
        Self::Date,
        Self::Color,
        Self::Rating,
        Self::Url,
        Self::Tags,
        Self::Choice,
        Self::MultiChoice,
        Self::MediaList,
        Self::Reference,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Textarea => "textarea",
            Self::RichText => "rich_text",
            Self::Integer => "integer",
            Self::Decimal => "decimal",
            Self::Money => "money",
            Self::Dimension => "dimension",
            Self::Weight => "weight",
            Self::Boolean => "boolean",
            Self::Date => "date",
            Self::Color => "color",
            Self::Rating => "rating",
            Self::Url => "url",
            Self::Tags => "tags",
            Self::Choice => "choice",
            Self::MultiChoice => "multi_choice",
            Self::MediaList => "media_list",
            Self::Reference => "reference",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == text)
    }
}

/// The three form steps (ch08 §Groups and presets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeGroup {
    Details,
    Condition,
    Product,
}

impl AttributeGroup {
    pub const ALL: [AttributeGroup; 3] = [Self::Details, Self::Condition, Self::Product];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Details => "details",
            Self::Condition => "condition",
            Self::Product => "product",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|g| g.as_str() == text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Human,
    Ai,
    Default,
}

impl Provenance {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Ai => "ai",
            Self::Default => "default",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [Self::Human, Self::Ai, Self::Default]
            .into_iter()
            .find(|p| p.as_str() == text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiConfidence {
    High,
    Medium,
    Low,
}

impl AiConfidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [Self::High, Self::Medium, Self::Low]
            .into_iter()
            .find(|c| c.as_str() == text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiStatus {
    Suggested,
    Accepted,
    Cleared,
}

impl AiStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::Accepted => "accepted",
            Self::Cleared => "cleared",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [Self::Suggested, Self::Accepted, Self::Cleared]
            .into_iter()
            .find(|s| s.as_str() == text)
    }
}

/// One attribute definition (ch08 §Schema; table `attribute_definitions`). `validations` is a JSON
/// object; the core reads `units` (dimension, weight: the adapter's unit codes) from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    pub integration: String,
    pub key: String,
    pub label: String,
    pub core_type: CoreType,
    pub choices: Vec<String>,
    pub validations: serde_json::Value,
    pub group: AttributeGroup,
    pub description: Option<String>,
    pub writable: bool,
}

impl Definition {
    /// The unit codes listed in `validations.units`, if any.
    pub fn units(&self) -> Option<Vec<&str>> {
        self.validations
            .get("units")
            .and_then(|u| u.as_array())
            .map(|units| units.iter().filter_map(|u| u.as_str()).collect())
    }
}

/// One stored value (table `attribute_values`). `value` is canonical JSON text (MC-9) when `valid`,
/// else the value as received, verbatim. A row with `ai_status = cleared` is not a value of the item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Value {
    pub item_id: i64,
    pub key: String,
    pub value: String,
    pub valid: bool,
    pub provenance: Provenance,
    pub ai_confidence: Option<AiConfidence>,
    pub ai_status: Option<AiStatus>,
    pub ai_reason: Option<String>,
    pub ai_sources: Option<Vec<String>>,
    pub updated_at: i64,
}

impl Value {
    /// False once the value was cleared (ch08: cleared values are no longer the item's values).
    pub fn is_live(&self) -> bool {
        self.ai_status != Some(AiStatus::Cleared)
    }

    /// The stored JSON, parsed.
    pub fn json(&self) -> Option<serde_json::Value> {
        serde_json::from_str(&self.value).ok()
    }
}

/// One field of a preset layout: a definition key and an optional default (canonical JSON value).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetField {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

/// Customize fields for one item type and one step (table `attribute_presets`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preset {
    pub item_type: String,
    pub group: AttributeGroup,
    pub layout: Vec<PresetField>,
}

/// One field of the form (`form_get`): its definition, the item's live value if any, and for a
/// `media_list` field the selected condition photo ids (07A MC-14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormField {
    pub definition: Definition,
    pub value: Option<Value>,
    /// The value is a preset default nobody has touched (provenance `default`).
    pub is_default: bool,
    /// The definition is a `choice` over the severity set (05A MC-9 field kind).
    pub is_severity: bool,
    pub selected: Option<Vec<i64>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormGroup {
    pub group: AttributeGroup,
    pub fields: Vec<FormField>,
    /// Fields holding a valid value.
    pub filled: usize,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Form {
    pub item_id: i64,
    pub groups: Vec<FormGroup>,
}
