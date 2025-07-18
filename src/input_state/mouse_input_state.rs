use std::collections::HashSet;
use winit::event::MouseButton;

#[derive(Debug)]
pub struct MouseInputState {
    pub delta_x: f64,
    pub delta_y: f64,
    pub pressed_buttons: HashSet<MouseButton>,
    pub last_frame_pressed_buttons: HashSet<MouseButton>,
}
impl MouseInputState {
    pub fn new() -> MouseInputState {
        Self {
            delta_x: 0.0,
            delta_y: 0.0,
            pressed_buttons: HashSet::new(),
            last_frame_pressed_buttons: HashSet::new(),
        }
    }
    pub fn update(&mut self, (delta_x, delta_y): (f64, f64)) {
        self.delta_x += delta_x;
        self.delta_y += delta_y;
    }
    pub fn refresh(&mut self) {
        self.delta_x = 0.;
        self.delta_y = 0.;
        self.last_frame_pressed_buttons = self.pressed_buttons.clone();
    }
}
