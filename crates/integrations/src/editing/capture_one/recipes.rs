use super::scripting::RecipeProps;

pub const SRGB_PROFILE: &str = "sRGB Color Space Profile";

#[derive(Clone, Copy)]
pub struct RecipeSpec {
    pub long_edge: i64,
    pub jpeg_quality: i64,
}
impl RecipeSpec {
    pub const PREVIEW: Self = Self {
        long_edge: 1600,
        jpeg_quality: 80,
    };
    pub const WEB: Self = Self {
        long_edge: 2048,
        jpeg_quality: 90,
    };
    pub fn props(&self) -> RecipeProps {
        RecipeProps {
            enabled: false,
            output_format: "JPEG".into(),
            color_profile: SRGB_PROFILE.into(),
            scaling_method: "Long_Edge".into(),
            scaling_unit: "pixels".into(),
            primary_scaling_value: self.long_edge,
            jpeg_quality: self.jpeg_quality,
            existing_files: "overwrite".into(),
            output_sub_folder: "_session_manager".into(),
            root_folder_type: None,
        }
    }
}
