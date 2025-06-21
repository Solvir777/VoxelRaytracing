use crate::settings::graphics_settings::GraphicsSettings;
use crate::settings::input_settings::InputSettings;

pub mod graphics_settings;
mod input_settings;

pub struct Settings {
    pub(crate) graphics_settings: GraphicsSettings,
    pub(crate) input_settings: InputSettings,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            graphics_settings: GraphicsSettings::standard(),
            input_settings: InputSettings::standard(),
        }
    }
}
