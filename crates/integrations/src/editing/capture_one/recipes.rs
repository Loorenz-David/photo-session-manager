use super::scripting::RecipeProps;

pub const SRGB_PROFILE: &str = "sRGB Color Space Profile";
pub const MASTER_TIFF_COMPRESSION: &str = "Uncompressed";

#[derive(Clone, Copy)]
pub struct RecipeSpec {
    pub output_format: &'static str,
    pub bits: i64,
    pub scaling_method: &'static str,
    pub scaling_unit: &'static str,
    pub long_edge: i64,
    pub jpeg_quality: i64,
    pub export_crop_method: &'static str,
}
impl RecipeSpec {
    pub const PREVIEW: Self = Self {
        output_format: "JPEG",
        bits: 8,
        scaling_method: "Long_Edge",
        scaling_unit: "pixels",
        long_edge: 2560,
        jpeg_quality: 85,
        export_crop_method: "ignore",
    };
    pub const MASTER: Self = Self {
        output_format: "TIFF",
        bits: 16,
        scaling_method: "Fixed",
        scaling_unit: "percent",
        long_edge: 100,
        jpeg_quality: 100,
        export_crop_method: "ignore",
    };
    pub fn props(&self) -> RecipeProps {
        RecipeProps {
            enabled: false,
            output_format: self.output_format.into(),
            bits: self.bits,
            tiff_compression: MASTER_TIFF_COMPRESSION.into(),
            export_crop_method: self.export_crop_method.into(),
            color_profile: SRGB_PROFILE.into(),
            scaling_method: self.scaling_method.into(),
            scaling_unit: self.scaling_unit.into(),
            primary_scaling_value: self.long_edge,
            jpeg_quality: self.jpeg_quality,
            existing_files: "overwrite".into(),
            output_sub_folder: "_session_manager".into(),
            root_folder_type: None,
        }
    }
}
