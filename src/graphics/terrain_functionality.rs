use std::sync::Arc;
use nalgebra::Vector3;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{BufferImageCopy, CommandBufferUsage, CopyBufferToImageInfo};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::{Image, ImageAspects, ImageSubresourceLayers};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::Pipeline;
use vulkano::{sync, DeviceSize};
use vulkano::image::view::ImageView;
use vulkano::sync::GpuFuture;
use crate::game_state::GameState;
use crate::game_state::terrain::ChunkBuffer;
use crate::graphics::{block_in_chunk_index, chunk_buffer_index, Graphics};
use crate::shaders::terrain_gen;

impl Graphics {
    pub fn copy_buffer_to_image(
        &mut self,
        buffer: ChunkBuffer,
        image: Arc<Image>,
        only_copy_single_block: Option<Vector3<i32>>,
    ) {
        let copy_info = match only_copy_single_block {
            None => CopyBufferToImageInfo::buffer_image(buffer.clone(), image.clone()),
            Some(block_position) => {
                let block_in_chunk = block_in_chunk_index(block_position);
                let regions = vec![BufferImageCopy {
                    buffer_offset: (block_in_chunk * size_of::<u16>()) as DeviceSize,
                    image_subresource: ImageSubresourceLayers {
                        aspects: ImageAspects::COLOR,
                        mip_level: 0,
                        array_layers: 0..1,
                    },
                    image_offset: block_position
                        .map(|x| x.rem_euclid(Self::CHUNK_SIZE as i32) as u32)
                        .data
                        .0[0],
                    image_extent: [1; 3],
                    ..Default::default()
                }]
                    .into();

                CopyBufferToImageInfo {
                    regions,
                    ..CopyBufferToImageInfo::buffer_image(buffer.clone(), image.clone())
                }
            }
        };

        let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
            &self.vulkano_core.allocators.commmand_buffer,
            self.vulkano_core.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
            .unwrap();

        builder.copy_buffer_to_image(copy_info).unwrap();

        let command_buffer = builder.build().unwrap();

        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .then_execute(self.vulkano_core.queue.clone(), command_buffer)
            .unwrap();

        self.previous_frame_end = Some(future.boxed());
    }

    pub fn update_chunks(&mut self, game_state: &mut GameState, old_chunk_pos: Option<Vector3<i32>>) {
        if let Some(old_pos) = old_chunk_pos
            && old_pos == game_state.get_player_chunk()
        {
            return;
        }
        self.wait_and_reset_last_frame_end();

        let gen_dist = self.settings.graphics_settings.render_distance as i32;
        for x in (-gen_dist)..=gen_dist {
            for y in (-gen_dist)..=gen_dist {
                for z in (-gen_dist)..=gen_dist {
                    self.wait_and_reset_last_frame_end();
                    let chunk_pos =
                        (game_state.get_player_chunk() + Vector3::new(x, y, z)) as Vector3<i32>;
                    //update this chunk if there was no previous chunk or if it left the players chunk range
                    let update_chunk = match old_chunk_pos {
                        None => true,
                        Some(old_pos) => (chunk_pos - old_pos).amax() > gen_dist,
                    };

                    if update_chunk {
                        game_state.terrain.upload_chunk(self, chunk_pos);
                    }
                }
            }
        }
        self.previous_frame_end = Some(sync::now(self.vulkano_core.device.clone()).boxed());
    }

    /// Generates a chunk and returns a host-mapped Buffer containing its Data
    pub fn generate_chunk(&mut self, chunk_position: Vector3<i32>) -> ChunkBuffer {
        let cpu_buffer: ChunkBuffer = Buffer::new_sized(
            self.vulkano_core.allocators.memory.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC
                    | BufferUsage::TRANSFER_DST
                    | BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
        )
            .unwrap();

        let push_constants = terrain_gen::PushConstants {
            chunk_position: chunk_position.into(),
        };

        let descriptor_set = PersistentDescriptorSet::new(
            &self.vulkano_core.allocators.descriptor_set,
            self.render_core
                .pipelines
                .terrain_generator_pipeline
                .pipeline
                .layout()
                .set_layouts()[0]
                .clone(),
            [WriteDescriptorSet::buffer(0, cpu_buffer.clone())],
            [],
        )
            .unwrap();

        let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
            &self.vulkano_core.allocators.commmand_buffer,
            self.vulkano_core.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
            .unwrap();

        builder
            .bind_pipeline_compute(
                self.render_core
                    .pipelines
                    .terrain_generator_pipeline
                    .pipeline
                    .clone(),
            )
            .unwrap()
            .push_constants(
                self.render_core
                    .pipelines
                    .terrain_generator_pipeline
                    .pipeline
                    .layout()
                    .clone(),
                0,
                push_constants,
            )
            .unwrap()
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Compute,
                self.render_core
                    .pipelines
                    .terrain_generator_pipeline
                    .pipeline
                    .layout()
                    .clone(),
                0,
                descriptor_set,
            )
            .unwrap()
            .dispatch([Self::CHUNK_SIZE / 8; 3])
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .then_execute(self.vulkano_core.queue.clone(), command_buffer)
            .unwrap();

        self.previous_frame_end = Some(future.boxed());
        cpu_buffer
    }
    
    pub fn generate_distance_field(&mut self, chunk_position: Vector3<i32>) {
        let descriptor_set = PersistentDescriptorSet::new(
            &self.vulkano_core.allocators.descriptor_set,
            self.render_core
                .pipelines
                .terrain_distance_pipeline
                .pipeline
                .layout()
                .set_layouts()[0]
                .clone(),
            [
                WriteDescriptorSet::image_view(
                    0, 
                    ImageView::new_default(
                        self.render_core.buffers.block_data_buffers[chunk_buffer_index(chunk_position, &self.settings)].clone()
                    ).unwrap()
                ),
                WriteDescriptorSet::image_view(
                    1,
                    ImageView::new_default(
                        self.render_core.buffers.distance_data_buffers[chunk_buffer_index(chunk_position, &self.settings)].clone()
                    ).unwrap()
                )
            ],
            [],
        )
            .unwrap();

        let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
            &self.vulkano_core.allocators.commmand_buffer,
            self.vulkano_core.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
            .unwrap();

        builder
            .bind_pipeline_compute(
                self.render_core
                    .pipelines
                    .terrain_distance_pipeline
                    .pipeline
                    .clone(),
            )
            .unwrap()
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Compute,
                self.render_core
                    .pipelines
                    .terrain_distance_pipeline
                    .pipeline
                    .layout()
                    .clone(),
                0,
                descriptor_set,
            )
            .unwrap()
            .dispatch([Self::CHUNK_SIZE / 8; 3])
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .then_execute(self.vulkano_core.queue.clone(), command_buffer)
            .unwrap();

        self.previous_frame_end = Some(future.boxed());
    }
}