use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionKind {
    Normal,
    Split,
}
impl CompositionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Split => "split",
        }
    }
    pub fn slots(self) -> &'static [OutputSlot] {
        match self {
            Self::Normal => &[OutputSlot::Single],
            Self::Split => &[OutputSlot::Left, OutputSlot::Right],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    #[serde(rename = "original")]
    Original,
    #[serde(rename = "1:1")]
    Square,
    #[serde(rename = "4:5")]
    FourFive,
    #[serde(rename = "3:2")]
    ThreeTwo,
    #[serde(rename = "2:3")]
    TwoThree,
    #[serde(rename = "16:9")]
    SixteenNine,
    #[serde(rename = "free")]
    Free,
}
impl Shape {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "original" => Some(Self::Original),
            "1:1" => Some(Self::Square),
            "4:5" => Some(Self::FourFive),
            "3:2" => Some(Self::ThreeTwo),
            "2:3" => Some(Self::TwoThree),
            "16:9" => Some(Self::SixteenNine),
            "free" => Some(Self::Free),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Square => "1:1",
            Self::FourFive => "4:5",
            Self::ThreeTwo => "3:2",
            Self::TwoThree => "2:3",
            Self::SixteenNine => "16:9",
            Self::Free => "free",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    Named,
    Turned,
}
impl Orientation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Named => "named",
            Self::Turned => "turned",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Flip {
    None,
    Horizontal,
}
impl Flip {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Horizontal => "horizontal",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Default,
    User,
}
impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::User => "user",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionState {
    Ok,
    NeedsReview,
}
impl CompositionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::NeedsReview => "needs_review",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewReason {
    Geometry,
    Frame,
}
impl ReviewReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Geometry => "geometry",
            Self::Frame => "frame",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameSource {
    Camera,
    Preview,
    Master,
}
impl FrameSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Camera => "camera",
            Self::Preview => "preview",
            Self::Master => "master",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputSlot {
    Single,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorGeometry {
    pub rotation: f64,
    pub flip: String,
    pub keystone_vertical: f64,
    pub keystone_horizontal: f64,
    pub keystone_amount: i64,
}
impl Default for EditorGeometry {
    fn default() -> Self {
        Self {
            rotation: 0.0,
            flip: "none".into(),
            keystone_vertical: 0.0,
            keystone_horizontal: 0.0,
            keystone_amount: 100,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    pub w: i64,
    pub h: i64,
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelRect {
    pub left: i64,
    pub top: i64,
    pub width: i64,
    pub height: i64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fingerprint {
    pub frame: Frame,
    pub source: FrameSource,
    pub geometry: EditorGeometry,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionValues {
    pub kind: CompositionKind,
    pub shape: Shape,
    pub orientation: Orientation,
    pub quarter_turns: u8,
    pub flip: Flip,
    pub straighten: f64,
    pub rect: Rect,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Composition {
    pub photo_id: i64,
    pub kind: CompositionKind,
    pub shape: Shape,
    pub orientation: Orientation,
    pub quarter_turns: u8,
    pub flip: Flip,
    pub straighten: f64,
    pub rect: Option<Rect>,
    pub fingerprint: Option<Fingerprint>,
    pub origin: Origin,
    pub revision: i64,
    pub state: CompositionState,
    pub review_reason: Option<ReviewReason>,
    pub updated_at: i64,
}
impl Composition {
    pub fn frame(&self) -> Option<Frame> {
        self.fingerprint.as_ref().map(|f| f.frame)
    }
    pub fn values(&self) -> Option<CompositionValues> {
        self.rect.map(|rect| CompositionValues {
            kind: self.kind,
            shape: self.shape,
            orientation: self.orientation,
            quarter_turns: self.quarter_turns,
            flip: self.flip,
            straighten: self.straighten,
            rect,
        })
    }
}
