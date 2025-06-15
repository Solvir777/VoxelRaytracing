use nalgebra::{Rotation3, Vector3};
use winit::event::VirtualKeyCode;
use crate::input_state::{InputState, KeyState};
use crate::settings::Settings;

pub struct Player{
    pub position: Vector3<f32>,
    pub rotation: Rotation,
}

impl Player{
    const SPEED: f32 = 15.;
    pub fn new() -> Self {
        Self{
            position: Vector3::<f32>::new(16., 16., 16.),
            rotation: Rotation::zero(),
        }
    }

    pub fn get_rotation_mat(&self) -> nalgebra::Rotation<f32, 3> {
        self.rotation.to_rotation_matrix()
    }

    pub fn movement(&mut self, input_state: &InputState, delta_time: f32) {
        let (w, a, s, d, up_key, down_key) =
            (
                input_state.is_key_pressed(VirtualKeyCode::W, KeyState::Held),
                input_state.is_key_pressed(VirtualKeyCode::A, KeyState::Held),
                input_state.is_key_pressed(VirtualKeyCode::S, KeyState::Held),
                input_state.is_key_pressed(VirtualKeyCode::D, KeyState::Held),
                input_state.is_key_pressed(VirtualKeyCode::Space, KeyState::Held),
                input_state.is_key_pressed(VirtualKeyCode::LShift, KeyState::Held),
            );

        let up = Vector3::new(0., 1., 0.);
        let forward = Vector3::new(self.rotation.yaw.sin(), 0., self.rotation.yaw.cos()); //Z is default forward
        let right = Vector3::new(self.rotation.yaw.cos(), 0., -self.rotation.yaw.sin());

        let mut movement = Vector3::<f32>::zeros();
        if w {movement += forward}
        if s {movement -= forward}
        if d {movement += right}
        if a {movement -= right}
        if up_key {movement += up}
        if down_key {movement -= up}

        movement = movement.cap_magnitude(1.);
        movement *= delta_time * Player::SPEED;
        self.position += movement;
    }
    pub fn pan(&mut self, input_state: &InputState, settings: &Settings) {
        let sensitivity = settings.input_settings.mouse_sensitivity;
        self.rotation.yaw += input_state.mouse.delta_x as f32 * sensitivity;
        self.rotation.pitch = (self.rotation.pitch + input_state.mouse.delta_y as f32 * sensitivity).clamp(-1.7, 1.7);
    }
}

pub struct Rotation {
    pitch: f32,
    yaw: f32,
}

impl Rotation {
    fn to_rotation_matrix(&self) -> nalgebra::Rotation<f32, 3> {
        Rotation3::from_scaled_axis(Vector3::new(0., self.yaw, 0.)) * Rotation3::from_scaled_axis(Vector3::new(self.pitch, 0., 0.))
    }

    fn zero() -> Self {
        Rotation {
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}