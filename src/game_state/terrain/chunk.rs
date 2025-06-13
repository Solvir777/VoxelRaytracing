mod block;

use nalgebra::Vector3;
use crate::game_state::terrain::chunk::block::Block;
use crate::graphics::Graphics;

pub struct Chunk {
    data: [[[Block; Graphics::CHUNK_SIZE as usize]; Graphics::CHUNK_SIZE as usize]; Graphics::CHUNK_SIZE as usize]
}


impl Chunk {
}