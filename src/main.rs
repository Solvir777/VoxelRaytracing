mod game_state;
mod graphics;
mod input_state;
mod settings;
pub mod shaders;

use crate::game_state::GameState;
use crate::game_state::terrain::block::Block;
use crate::game_state::terrain::block::solid_block::SolidBlock;
use crate::input_state::{InputState, PressState};
use crate::settings::Settings;
use graphics::Graphics;
use nalgebra::Vector3;
use std::time::Instant;
use winit::event::MouseButton;

fn main() {
    let settings = Settings::new();

    let (graphics, event_loop) = Graphics::new(settings);
    let game_state = GameState::new();
    let input_state = InputState::new();

    let mut last_frame = Instant::now();

    let update = move |game_state: &mut GameState,
                       input_state: &InputState,
                       graphics: &mut Graphics,
                       control_flow: &mut winit::event_loop::ControlFlow| {
        let delta_time = last_frame.elapsed().as_secs_f32();
        last_frame = Instant::now();

        if input_state.is_key_pressed(winit::event::VirtualKeyCode::Escape, PressState::Down) {
            *control_flow = winit::event_loop::ControlFlow::Exit;
        }
        if input_state.is_key_pressed(winit::event::VirtualKeyCode::Tab, PressState::Down) {
            println!("toggling confined state");
            graphics.toggle_confine();
        }
        player_actions(game_state, graphics, input_state);
        
        if input_state.is_key_pressed(winit::event::VirtualKeyCode::Up, PressState::Held) {
            graphics.add_pov(0.5);
        }
        if input_state.is_key_pressed(winit::event::VirtualKeyCode::Down, PressState::Held) {
            graphics.add_pov(- 0.5);
        }
        game_state.update(&input_state, &graphics.settings, delta_time);
    };

    graphics.run(game_state, input_state, event_loop, update);
}

fn player_actions(game_state: &mut GameState, graphics: &mut Graphics, input_state: &InputState) {
    if input_state.is_mouse_pressed(MouseButton::Left, PressState::Held)
        && let Some(block_hit) = graphics.what_is_bro_looking_at()
    {
        let pos = ((block_hit.hit_point + block_hit.hit_normal * 0.2).map(|x| x.floor() as i32))
            as Vector3<i32>;
        game_state
            .terrain
            .place_block(graphics, pos, Block::SolidBlock(SolidBlock::Grass))
    }
    if input_state.is_mouse_pressed(MouseButton::Right, PressState::Held)
        && let Some(block_hit) = graphics.what_is_bro_looking_at()
    {
        let pos = (block_hit.hit_point - block_hit.hit_normal * 0.2).map(|x| x.floor() as i32)
            as Vector3<i32>;
        game_state.terrain.place_block(graphics, pos, Block::Air)
    }
}