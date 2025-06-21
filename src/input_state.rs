use winit::event::{ElementState, KeyboardInput, MouseButton};
use crate::input_state::keyboard_input_state::KeyboardInputState;
use crate::input_state::mouse_input_state::MouseInputState;
mod keyboard_input_state;
mod mouse_input_state;

#[derive(Debug)]
pub struct InputState {
    pub keyboard: KeyboardInputState,
    pub mouse: MouseInputState,
}

pub enum PressState {
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
    pub fn is_key_pressed(&self, key_code: winit::event::VirtualKeyCode, key_state: PressState) -> bool {
        match (self.keyboard.held_keys.contains(&key_code), self.keyboard.held_keys_last_frame.contains(&key_code), key_state)  {
            (true, true, PressState::Held) => true,
            (true, false, PressState::Down) => true,
            (false, true, PressState::Up) => true,
            _ => false
        }
    }
    pub fn is_mouse_pressed(&self, button: MouseButton, mouse_state: PressState) -> bool {
        match (self.mouse.pressed_buttons.contains(&button), self.mouse.last_frame_pressed_buttons.contains(&button), mouse_state)  {
            (true, true, PressState::Held) => true,
            (true, false, PressState::Down) => true,
            (false, true, PressState::Up) => true,
            _ => false
        }
    }

    pub fn update_mouse_press(&mut self, state: ElementState, button: MouseButton) {
        match state {
            ElementState::Pressed => {
                self.mouse.pressed_buttons.insert(button);
            }
            ElementState::Released => {
                self.mouse.pressed_buttons.remove(&button);
            }
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