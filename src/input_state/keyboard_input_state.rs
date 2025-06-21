use std::collections::HashSet;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Debug)]
pub struct KeyboardInputState {
    pub held_keys: HashSet<VirtualKeyCode>,
    pub held_keys_last_frame: HashSet<VirtualKeyCode>,
}

impl KeyboardInputState {
    pub fn new() -> Self {
        Self {
            held_keys: HashSet::new(),
            held_keys_last_frame: HashSet::new(),
        }
    }

    pub fn update(&mut self, keyboard_input: KeyboardInput) {
        // ignore unknown keypresses
        if keyboard_input.virtual_keycode.is_none() {
            return;
        }

        match keyboard_input.state {
            ElementState::Pressed => {
                self.held_keys
                    .insert(keyboard_input.virtual_keycode.unwrap());
            }
            ElementState::Released => {
                self.held_keys
                    .remove(&keyboard_input.virtual_keycode.unwrap());
            }
        }
    }

    pub fn refresh(&mut self) {
        self.held_keys_last_frame = self.held_keys.clone();
    }
}
