pub struct InputSettings {
    pub mouse_sensitivity: f32,
}

impl InputSettings {
    pub fn standard() -> Self {
        Self {
            mouse_sensitivity: 0.002,
        }
    }
}