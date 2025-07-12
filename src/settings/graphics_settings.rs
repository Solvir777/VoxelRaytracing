pub struct GraphicsSettings {
    pub field_of_view: f32,
    pub render_distance: u8,
    pub level_of_detail_layers: u8,
}

impl GraphicsSettings {
    pub fn standard() -> Self {
        Self {
            field_of_view: 90.,
            render_distance: 3,
            level_of_detail_layers: 1,
        }
    }
}
