mod player;
pub mod terrain;

use nalgebra::Vector3;
use crate::game_state::player::Player;
use crate::game_state::terrain::Terrain;
use crate::graphics::Graphics;
use crate::input_state::InputState;
use crate::settings::Settings;
use crate::shaders::rendering::PushConstants;

pub struct GameState {
    pub player: Player,
    pub terrain: Terrain,
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
            terrain: Terrain::empty(),
        }
    }
    pub fn update(&mut self, input_state: &InputState, settings: &Settings, delta_time: f32) {
        self.player.pan(input_state, settings);
        self.player.movement(input_state, delta_time);
    }

    pub fn get_player_chunk(&self) -> Vector3<i32> {
        self.player.position.map(|x: f32| (x / Graphics::CHUNK_SIZE as f32).floor() as i32)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_player_chunk() {
        let mut game_state = GameState::new();
        // Test case 1: Player at origin
        game_state.player.position = Vector3::new(0.0, 0.0, 0.0);
        assert_eq!(game_state.get_player_chunk(), Vector3::new(0, 0, 0));

        // Test case 2: Player inside a chunk
        game_state.player.position = Vector3::new(10.0, 20.0, 30.0);
        assert_eq!(game_state.get_player_chunk(), Vector3::new(0, 0, 0));

        // Test case 3: Player at the edge of a chunk
        game_state.player.position = Vector3::new(31.9, 31.9, 31.9);
        assert_eq!(game_state.get_player_chunk(), Vector3::new(0, 0, 0));

        // Test case 4: Player in the next chunk
        game_state.player.position = Vector3::new(32.0, 32.0, 32.0);
        assert_eq!(game_state.get_player_chunk(), Vector3::new(1, 1, 1));

        // Test case 5: Player in a negative chunk
        game_state.player.position = Vector3::new(-1.0, -1.0, -1.0);
        assert_eq!(game_state.get_player_chunk(), Vector3::new(-1, -1, -1));
    }
}