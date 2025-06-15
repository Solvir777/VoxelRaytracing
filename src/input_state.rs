use winit::event::KeyboardInput;
use crate::input_state::keyboard_input_state::KeyboardInputState;
use crate::input_state::mouse_input_state::MouseInputState;
mod keyboard_input_state;
mod mouse_input_state;

#[derive(Debug)]
pub struct InputState {
    keyboard: KeyboardInputState,
    pub mouse: MouseInputState,
}

pub enum KeyState {
    Down,
    Up,
    Held,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardInputState::new(),
            mouse: MouseInputState::new(),
        }
    }
    pub fn is_key_pressed(&self, key_code: winit::event::VirtualKeyCode, key_state: KeyState) -> bool {
        match key_state {
            KeyState::Down => {self.keyboard.key_down(key_code)}
            KeyState::Up => {self.keyboard.key_up(key_code)}
            KeyState::Held => {self.keyboard.key_held(key_code)}
        }
    }

    pub fn update_keys(&mut self, keyboard_input: KeyboardInput) {
        self.keyboard.update(keyboard_input);
    }

    pub fn update_mouse(&mut self, delta: (f64, f64)) {
        self.mouse.update(delta);
    }

    pub fn refresh(&mut self) {
        self.keyboard.refresh();
        self.mouse.refresh();
    }
}