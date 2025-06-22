use crate::settings::graphics_settings::GraphicsSettings;
use crate::shaders::rendering::GpuGraphicsSettings;

impl GpuGraphicsSettings {
    pub fn new(graphics_settings: &GraphicsSettings) -> Self{
        Self{
            fov: graphics_settings.field_of_view.to_radians()
        }
    }
}