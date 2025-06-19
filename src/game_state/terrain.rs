pub mod chunk;

use std::collections::HashMap;
use nalgebra::Vector3;
use crate::game_state::terrain::chunk::Chunk;

pub struct Terrain {
    pub chunks: HashMap<Vector3<i32>, Chunk>
}


impl Terrain {
    pub fn empty() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
}