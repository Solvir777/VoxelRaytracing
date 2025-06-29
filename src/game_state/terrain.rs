use crate::game_state::terrain::block::Block;
use crate::graphics::Graphics;
use nalgebra::Vector3;
use std::collections::HashMap;
use vulkano::buffer::Subbuffer;
use crate::graphics;

pub mod block;
pub struct Terrain {
    pub chunks: HashMap<Vector3<i32>, ChunkBuffer>,
}
pub type ChunkBuffer = Subbuffer<[u16; Graphics::CHUNK_VOLUME as usize]>;

impl Terrain {
    pub fn empty() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
    /// Checks whether the chunk is present in the terrain struct and generates it otherwise.
    pub fn upload_chunk(&mut self, graphics: &mut Graphics, chunk_position: Vector3<i32>) {
        if !self.chunks.contains_key(&chunk_position) {
            let chunk = graphics.generate_chunk(chunk_position);

            self.chunks.insert(chunk_position, chunk);
        }

        let chunk = self.chunks.get(&chunk_position).unwrap();

        let chunk_index = graphics::chunk_buffer_index(chunk_position, &graphics.settings);
        graphics.copy_buffer_to_image(
            chunk.clone(),
            graphics.render_core.buffers.block_data_buffers[chunk_index].clone(),
            None,
        );
        graphics.generate_distance_field(chunk_position);
    }
    pub fn place_block(
        &mut self,
        graphics: &mut Graphics,
        block_position: Vector3<i32>,
        block_type: Block,
    ) {
        graphics.wait_and_reset_last_frame_end();
        let block_chunk =
            block_position.map(|x| (x as f32 / Graphics::CHUNK_SIZE as f32).floor() as i32);
        let chunk = self.chunks.get_mut(&block_chunk).unwrap();
        let mut guard = chunk.write().unwrap();
        guard[graphics::block_in_chunk_index(block_position)] = block_type.as_u16();
        drop(guard);
        let index = graphics::chunk_buffer_index(block_chunk, &graphics.settings);

        graphics.copy_buffer_to_image(
            chunk.clone(),
            graphics.render_core.buffers.block_data_buffers[index].clone(),
            Some(block_position),
        );
        graphics.wait_and_reset_last_frame_end();
        graphics.generate_distance_field(block_chunk);
    }
}

