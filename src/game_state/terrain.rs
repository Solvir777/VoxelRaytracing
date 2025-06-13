mod chunk;

use std::collections::HashMap;
use nalgebra::Vector3;
use crate::game_state::terrain::chunk::Chunk;

pub struct Terrain {
    chunks: HashMap<Vector3<i32>, Chunk>
}