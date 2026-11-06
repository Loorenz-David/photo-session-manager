use psm_core::{
    Result,
    domain::camera::{CameraCapabilities, CameraStatus, CaptureId, Link},
    ports::camera::CameraProvider,
};
pub struct FakeDirectProvider;
impl CameraProvider for FakeDirectProvider {
    fn raw_extensions(&self) -> &'static [&'static str] {
        &["arw"]
    }
    fn status(&self) -> Result<CameraStatus> {
        Ok(CameraStatus::Connected {
            name: "X".into(),
            link: Link::Wireless,
        })
    }
    fn capture(&self) -> Result<CaptureId> {
        Ok(CaptureId("fake-capture".into()))
    }
    fn capabilities(&self) -> CameraCapabilities {
        CameraCapabilities {
            status: true,
            capture: true,
            transfer: true,
            exposure_events: true,
            settings_read: true,
            settings_write_store_destination: true,
            depends_on_editing_provider: false,
        }
    }
}
