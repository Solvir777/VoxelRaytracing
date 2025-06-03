mod player;
use crate::game_state::player::Player;
use crate::input_state::InputState;
use crate::settings::Settings;
use crate::shaders::rendering::PushConstants;

pub struct GameState {
    pub player: Player,
}

impl GameState {
    pub fn get_push_constants(&self) -> PushConstants {
        let pos = self.player.position;
        let rot = self.player.get_rotation_mat();
        let cam_transform = nalgebra::Matrix4::new_translation(&pos) * nalgebra::Matrix4::from(rot).transpose();

        PushConstants{
            cam_transform: cam_transform.into(),
        }
    }

    pub fn new() -> Self {
        Self {
            player: Player::new(),
        }
    }
    pub fn update(&mut self, input_state: &InputState, settings: &Settings) {
        self.player.pan(input_state, settings);
        self.player.movement(input_state);

    }
}