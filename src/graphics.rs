use std::sync::Arc;

use crate::game_state::terrain::{block_in_chunk_index, ChunkBuffer};
use crate::game_state::GameState;
use crate::graphics::render_core::swapchain_resources::SwapchainResources;
use crate::graphics::render_core::RenderCore;
use crate::graphics::vulkano_core::VulkanoCore;
use crate::input_state::InputState;
use crate::settings::Settings;
use crate::shaders::rendering::LookingAtBlock;
use crate::shaders::terrain_gen;

use nalgebra::Vector3;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{BufferImageCopy, CommandBufferUsage, CopyBufferToImageInfo};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::{Image, ImageAspects, ImageSubresourceLayers};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::Pipeline;
use vulkano::sync::GpuFuture;
use vulkano::{sync, DeviceSize};
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

mod allocators;
mod buffers;
mod render_core;
mod vulkano_core;

pub struct Graphics {
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub vulkano_core: VulkanoCore,
    pub render_core: RenderCore,
    pub settings: Settings,
    cursor_confined: bool,
}
impl Graphics {
    pub const CHUNK_SIZE: u32 = 64;
    pub const CHUNK_VOLUME: u32 =
        Graphics::CHUNK_SIZE * Graphics::CHUNK_SIZE * Graphics::CHUNK_SIZE;
    pub fn new(settings: Settings) -> (Self, EventLoop<()>) {
        let (vulkano_core, event_loop) = VulkanoCore::new();

        let previous_frame_end: Option<Box<dyn GpuFuture>> =
            Some(Box::new(sync::now(vulkano_core.device.clone())));

        let render_core = RenderCore::new(&vulkano_core, &settings);

        (
            Self {
                previous_frame_end,
                vulkano_core,
                render_core,
                settings,
                cursor_confined: false,
            },
            event_loop,
        )
    }

    pub fn run<F>(
        mut self,
        mut game_state: GameState,
        mut input_state: InputState,
        event_loop: EventLoop<()>,
        mut update: F,
    ) where
        F: FnMut(&mut GameState, &InputState, &mut Self, &mut ControlFlow) + 'static,
    {
        self.update_chunks(&mut game_state, None);
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(..) => {
                    self.render_core.swapchain_ressources.recreate_swapchain = true;
                }
                WindowEvent::CloseRequested { .. } => {
                    println!("Close Requested!");
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    input_state.update_mouse_press(state, button);
                }
                _ => {}
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    input_state.update_mouse(delta);
                }
                DeviceEvent::MouseWheel { .. } => {}
                DeviceEvent::Key(input) => {
                    input_state.update_keys(input);
                }
                _ => {}
            },
            Event::RedrawEventsCleared => {
                let old_chunk_pos = game_state.get_player_chunk();
                update(&mut game_state, &input_state, &mut self, control_flow);
                self.update_chunks(&mut game_state, Some(old_chunk_pos));

                self.draw_frame(&game_state);
                input_state.refresh();
                unsafe { self.vulkano_core.device.clone().wait_idle().unwrap() }
            }
            _ => {}
        });
    }
    fn recreate_swapchain(&mut self) {
        self.render_core.swapchain_ressources.recreate_swapchain = true;
    }
    pub fn draw_frame(&mut self, game_state: &GameState) {
        let image_extent: [u32; 2] = self.vulkano_core.window.inner_size().into();

        if image_extent.contains(&0) {
            return;
        }

        self.recreate_swapchain_if_needed();

        let (image_index, suboptimal, acquire_future) =
            match vulkano::swapchain::acquire_next_image(
                self.render_core.swapchain_ressources.swapchain.clone(),
                None,
            )
            .map_err(vulkano::Validated::unwrap)
            {
                Ok(r) => r,
                Err(vulkano::VulkanError::OutOfDate) => {
                    self.recreate_swapchain();
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {:?}", e),
            };

        if suboptimal {
            self.recreate_swapchain();
        }

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
                    .raytrace_pipeline
                    .pipeline
                    .clone(),
            )
            .unwrap()
            .push_constants(
                self.render_core
                    .pipelines
                    .raytrace_pipeline
                    .pipeline
                    .layout()
                    .clone(),
                0,
                game_state.get_push_constants(),
            )
            .unwrap()
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Compute,
                self.render_core
                    .pipelines
                    .raytrace_pipeline
                    .pipeline
                    .layout()
                    .clone(),
                0,
                self.render_core.pipelines.raytrace_pipeline.descriptor_sets[image_index as usize]
                    .clone(),
            )
            .unwrap()
            .dispatch([image_extent[0] / 16 + 1, image_extent[1] / 16 + 1, 1])
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.vulkano_core.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.vulkano_core.queue.clone(),
                vulkano::swapchain::SwapchainPresentInfo::swapchain_image_index(
                    self.render_core.swapchain_ressources.swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush();

        match future.map_err(vulkano::Validated::unwrap) {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(vulkano::VulkanError::OutOfDate) => {
                self.recreate_swapchain();
                self.previous_frame_end = Some(sync::now(self.vulkano_core.device.clone()).boxed());
            }
            Err(e) => {
                panic!("failed to flush future: {:?}", e);
            }
        }
    }

    fn recreate_swapchain_if_needed(&mut self) {
        if !self.render_core.swapchain_ressources.recreate_swapchain {
            return;
        }
        SwapchainResources::recreate_swapchain(self);
    }

    fn update_chunks(&mut self, game_state: &mut GameState, old_chunk_pos: Option<Vector3<i32>>) {
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

    pub fn wait_and_reset_last_frame_end(&mut self) {
        if let Some(future) = self.previous_frame_end.take()
            && future.queue().is_some()
        {
            future
                .then_signal_fence_and_flush()
                .unwrap()
                .wait(None)
                .unwrap();
        }

        self.previous_frame_end = Some(sync::now(self.vulkano_core.device.clone()).boxed());
    }
    pub fn toggle_confine(&mut self) {
        self.cursor_confined = !self.cursor_confined;
        self.vulkano_core
            .window
            .set_cursor_grab(if self.cursor_confined {
                winit::window::CursorGrabMode::Confined
            } else {
                winit::window::CursorGrabMode::None
            })
            .unwrap();
        self.vulkano_core
            .window
            .set_cursor_visible(!self.cursor_confined);
    }

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

    pub fn what_is_bro_looking_at(&mut self) -> Option<LookingAtBlock> {
        self.wait_and_reset_last_frame_end();
        let buffer = self.render_core.buffers.player_raycast_buffer.clone();
        let looking_at_guard = buffer.read().unwrap();

        if looking_at_guard.block_id > 0 {
            return Some(LookingAtBlock {
                hit_point: looking_at_guard.hit_point,
                block_id: looking_at_guard.block_id,
                hit_normal: looking_at_guard.hit_normal,
            });
        }
        None
    }
    
    pub fn add_pov(&mut self, difference: f32) {
        self.wait_and_reset_last_frame_end();
        self.settings.graphics_settings.field_of_view = (self.settings.graphics_settings.field_of_view + difference).clamp(50., 160.);
        self.render_core.buffers.gpu_graphics_settings_buffer.write().unwrap().fov = self.settings.graphics_settings.field_of_view.to_radians();
    }
}
