use crate::game_state::terrain::block::Block;
use crate::graphics::Graphics;
use nalgebra::Vector3;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use vulkano::buffer::Subbuffer;
use crate::game_state::load_store::Serializeable;
use crate::graphics;

pub mod block;
pub struct Terrain {
    pub chunks: HashMap<Vector3<i32>, ChunkBuffer>,
}
pub type ChunkData = [u16; Graphics::CHUNK_VOLUME as usize];
pub type ChunkBuffer = Subbuffer<ChunkData>;

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
    
    pub fn save_terrain(&self, name: &str) {
        let mut file = File::create(name).unwrap();
        let bytes: Vec<u8> = self.save();
        file.write(&bytes).unwrap();
    }

    fn save(&self) -> Vec<u8> {
        let mut bytes = vec!();
        self.chunks.iter().for_each(|(position, chunk)| {
            println!("Storing chunk at {}", position);
            println!("as: {}", Vector3::<i32>::deserialize(&*position.serialize()));
            bytes.extend_from_slice(&position.serialize());
            bytes.extend_from_slice(&ChunkData::serialize(&*chunk.read().unwrap()));
        });
        bytes
    }
    pub fn load(graphics: &mut Graphics, file_name: &str) -> Self{
        const POS_BYTES: usize = size_of::<Vector3<i32>>();
        const CHUNK_BYTES: usize = size_of::<ChunkData>();
        
        let bytes = std::fs::read(file_name).unwrap();
        let mut terrain = Terrain::empty();
        
        bytes.chunks(POS_BYTES + CHUNK_BYTES).for_each(|chunk| {
            let position = Vector3::deserialize(&chunk[0..POS_BYTES]);
            let chunk_bytes = &chunk[POS_BYTES..POS_BYTES + CHUNK_BYTES];
            let chunk_data = ChunkData::deserialize(chunk_bytes);
            terrain.chunks.insert(
                position,
                graphics.chunk_from_data(chunk_data)
            );
        });
        
        terrain
    }
}





impl Serializeable for ChunkData {
    fn serialize(&self) -> Vec<u8> {
        let data = self.as_slice().iter().flat_map(
            |x| x.to_be_bytes()
        ).collect();
        data
    }

    fn deserialize(data: &[u8]) -> Self {
        let a = data.chunks_exact(2).map(
            |c|
                u16::from_be_bytes([c[0], c[1]])
        ).collect::<Vec<u16>>().try_into().unwrap();
        a
    }
}

impl Serializeable for Vector3<i32> {
    fn serialize(&self) -> Vec<u8> {
        let mut ret = vec!();
        ret.extend(self.x.to_le_bytes().iter());
        ret.extend(self.y.to_le_bytes().iter());
        ret.extend(self.z.to_le_bytes().iter());
        ret
    }

    fn deserialize(data: &[u8]) -> Self {
        let mut v = Vector3::zeros();
        v.x = i32::from_le_bytes(data[0..4].try_into().unwrap());
        v.y = i32::from_le_bytes(data[4..8].try_into().unwrap());
        v.z = i32::from_le_bytes(data[8..12].try_into().unwrap());
        v
    }
}