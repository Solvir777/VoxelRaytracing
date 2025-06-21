use std::collections::HashMap;
use nalgebra::Vector3;
use vulkano::buffer::Subbuffer;
use crate::graphics::Graphics;
use crate::settings::Settings;

pub struct Terrain {
    pub chunks: HashMap<Vector3<i32>, ChunkBuffer>
}
pub type ChunkBuffer = Subbuffer<[u16; Graphics::CHUNK_VOLUME as usize]>;


impl Terrain {
    pub fn empty() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
    /// Checks whether the chunk is present in the terrain struct and generates it otherwise.
    /// if hard_generate, it will be copied to the gpu
    pub fn upload_chunk(&mut self, graphics: &mut Graphics, chunk_position: Vector3<i32>) {
        if !self.chunks.contains_key(&chunk_position) {
            let chunk = graphics.generate_chunk(chunk_position);

            self.chunks.insert(chunk_position, chunk);
        }

        // if hard generate
        let chunk = self.chunks.get(&chunk_position).unwrap();
        
        let chunk_index = chunk_buffer_index(chunk_position, &graphics.settings);
        graphics.copy_buffer_to_image(chunk.clone(), graphics.render_core.buffers.block_data_buffers[chunk_index].clone())
        // end
    }
}


fn chunk_buffer_index(chunk_position: Vector3<i32>, settings: &Settings) -> usize {
    let render_sl = 2 * settings.graphics_settings.render_distance as i32 + 1;
    chunk_position.map(
        |x|
            x.rem_euclid(render_sl)
    ).dot(&Vector3::new(1, render_sl, render_sl * render_sl)) as usize
}