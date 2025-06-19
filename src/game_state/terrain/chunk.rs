mod block;

use std::array;
use crate::game_state::terrain::chunk::block::Block;
use crate::graphics::Graphics;

pub struct Chunk {
    data: [[[Block; Graphics::CHUNK_SIZE as usize]; Graphics::CHUNK_SIZE as usize]; Graphics::CHUNK_SIZE as usize]
}


impl Chunk {
    pub fn to_raw_data(&self) -> [u16; Graphics::CHUNK_SIZE as usize * Graphics::CHUNK_SIZE as usize * Graphics::CHUNK_SIZE as usize] {
        let cs = Graphics::CHUNK_SIZE as usize;
        array::from_fn(
            |i| {
                let x = i % cs;
                let y = (i / cs) % cs;
                let z = i / (cs * cs);

                self.data[x][y][z].as_u16()
            }

        )
    }

    pub fn from_raw_data(guard: &[u16]) -> Self {
        let cs = Graphics::CHUNK_SIZE as usize;
        let blocks = array::from_fn(
            |x| {
                array::from_fn(
                    |y|
                        array::from_fn(
                            |z| {
                                Block::from_u16(guard[x + y * cs + z * cs * cs])
                            }
                        )
                )
            }
        );
        Self {
            data: blocks
        }
    }
}