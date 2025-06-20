use std::time::Instant;
use nalgebra::Vector3;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo, CopyImageToBufferInfo};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use crate::graphics::render_core::RenderCore;
use crate::graphics::vulkano_core::VulkanoCore;
use vulkano::pipeline::Pipeline;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use crate::game_state::GameState;
use crate::game_state::terrain::chunk::Chunk;
use crate::graphics::render_core::swapchain_resources::SwapchainResources;
use crate::input_state::{InputState, KeyState};
use crate::settings::Settings;
use crate::shaders::terrain_gen;

mod allocators;
mod buffers;
mod render_core;
mod vulkano_core;

pub struct Graphics {
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub vulkano_core: VulkanoCore,
    render_core: RenderCore,
    pub(crate) settings: Settings,
    cursor_confined: bool
}
impl Graphics {
    pub const CHUNK_SIZE: u32 = 32;
    pub fn new(settings: Settings) -> (Self, EventLoop<()>) {
        let (vulkano_core, event_loop) = VulkanoCore::new();

        let previous_frame_end: Option<Box<dyn GpuFuture>> = Some(Box::new(sync::now(vulkano_core.device.clone())));

        let render_core = RenderCore::new(&vulkano_core, &settings);

        (
            Self {
                previous_frame_end,
                vulkano_core,
                render_core,
                settings,
                cursor_confined: false,
            },
            event_loop
        )
    }

    pub fn run<F>(
        mut self,
        mut game_state: GameState,
        mut input_state: InputState,
        event_loop: EventLoop<()>,
        mut update: F,
    )
    where F: FnMut(&mut GameState, &InputState, &mut Self, &mut ControlFlow) + 'static
    {
        self.update_chunks(&mut game_state, None);
        event_loop.run(move |event, _, control_flow|
            {
            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::Resized(..) => {
                            self.render_core.swapchain_ressources.recreate_swapchain = true;
                        }
                        WindowEvent::CloseRequested{ .. } => {
                            println!("Close Requested!");
                            *control_flow = winit::event_loop::ControlFlow::Exit;
                        }
                        _ => {}
                    }
                }
                Event::DeviceEvent { event, .. } => {
                    match event {
                        DeviceEvent::MouseMotion { delta } => {
                            input_state.update_mouse(delta);
                        }
                        DeviceEvent::MouseWheel { .. } => {}
                        DeviceEvent::Key(input) => {
                            input_state.update_keys(input);
                        }
                        _ => {}
                    }
                }
                Event::RedrawEventsCleared => {
                    let old_chunk_pos = game_state.get_player_chunk();
                    update(&mut game_state, &input_state, &mut self, control_flow);
                    self.update_chunks(&mut game_state, Some(old_chunk_pos));

                    
                    self.draw_frame(&game_state);
                    input_state.refresh();
                    unsafe {self.vulkano_core.device.clone().wait_idle().unwrap()}
                }
                _ => {}
            }
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
            match vulkano::swapchain::acquire_next_image(self.render_core.swapchain_ressources.swapchain.clone(), None)
                .map_err(vulkano::Validated::unwrap)
            {
                Ok(r) => r,
                Err(vulkano::VulkanError::OutOfDate) => {
                    self.recreate_swapchain();
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            self.recreate_swapchain();
        }

        let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
            &self.vulkano_core.allocators.commmand_buffer,
            self.vulkano_core.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();


        builder
            .bind_pipeline_compute(self.render_core.pipelines.raytrace_pipeline.pipeline.clone())
            .unwrap()
            .push_constants(
                self.render_core
                    .pipelines.raytrace_pipeline
                    .pipeline
                    .layout()
                    .clone(),
                0,
                game_state.get_push_constants(),
            )
            .unwrap()
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Compute,
                self.render_core.pipelines.raytrace_pipeline.pipeline.layout().clone(),
                0,
                self.render_core.pipelines.raytrace_pipeline.descriptor_sets[image_index as usize].clone(),
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
                panic!("failed to flush future: {e}");
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
        if let Some(old_pos) = old_chunk_pos {
            if old_pos == game_state.get_player_chunk() {
                return;
            }
        }
        let timer = Instant::now();
        self.wait_and_reset_last_frame_end();
        println!("waited {} secs for rendering to finish", timer.elapsed().as_secs_f32());
        
        let gen_dist = self.settings.graphics_settings.render_distance as i32;
        for x in (-gen_dist)..=gen_dist {
            for y in (-gen_dist)..=gen_dist {
                for z in (-gen_dist)..=gen_dist {
                    let chunk_pos = (game_state.get_player_chunk() + Vector3::new(x, y, z)) as Vector3<i32>;
                    //update this chunk if there was no previous chunk or if it left the players chunk range
                    let update_chunk = match old_chunk_pos {
                        None => {true}
                        Some(old_pos) => {(chunk_pos - old_pos).amax() > gen_dist}
                    };

                    if update_chunk {
                        self.upload_chunk(game_state, chunk_pos);
                    }
                }
            }
        }
        println!("time to generate all chunks: {}", timer.elapsed().as_secs_f32());
        self.previous_frame_end = Some(sync::now(self.vulkano_core.device.clone()).boxed());
    }
    /// Checks whether the chunk is contained in the Terrain struct, if not creates and inserts it. TODO
    /// Then the chunk is uploaded to the gpu
    fn upload_chunk(&mut self, game_state: &mut GameState, chunk_position: Vector3<i32>) {
       if let Some(chunk) = game_state.terrain.chunks.get(&chunk_position) {
           self.chunk_to_gpu(chunk_position, chunk);
           return;
       } 
        self.generate_chunk(chunk_position);
        self.chunk_from_gpu(game_state, chunk_position);
    }
    /// Send a stored chunk to the gpu
    fn chunk_to_gpu(&mut self, chunk_position: Vector3<i32>, chunk_data: &Chunk) {
        
        let staging_buffer = Buffer::from_data(
            self.vulkano_core.allocators.memory.clone(),
            BufferCreateInfo{
                usage: BufferUsage::TRANSFER_SRC | BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo{
                memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            chunk_data.to_raw_data()
        ).unwrap();
        
        let chunk_index = chunk_buffer_index(chunk_position, &self.settings);
        
        let buffer_to_image = CopyBufferToImageInfo::buffer_image(
            staging_buffer,
            self.render_core.buffers.block_data_buffers[chunk_index].clone(),
        );
        
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.vulkano_core.allocators.commmand_buffer,
            self.vulkano_core.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();
        
        builder
            .copy_buffer_to_image(buffer_to_image)
            .unwrap();
        
        let command_buffer = builder.build().unwrap();
        
        let future = self.previous_frame_end.take().unwrap()
            .then_execute(self.vulkano_core.queue.clone(), command_buffer)
            .unwrap();
        
        self.previous_frame_end = Some(future.boxed());
    }
    /// Read back a chunk from the gpu and fill it into Terrain struct
    fn chunk_from_gpu(&mut self, game_state: &mut GameState, chunk_position: Vector3<i32>) {
        let staging_buffer = Buffer::from_data(
            self.vulkano_core.allocators.memory.clone(),
            BufferCreateInfo{
                usage: BufferUsage::TRANSFER_DST | BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo{
                memory_type_filter: MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            [0u16; Graphics::CHUNK_SIZE as usize * Graphics::CHUNK_SIZE as usize * Graphics::CHUNK_SIZE as usize]
        ).unwrap();

        let chunk_index = chunk_buffer_index(chunk_position, &self.settings);

        let image_to_buffer = CopyImageToBufferInfo::image_buffer(
            self.render_core.buffers.block_data_buffers[chunk_index].clone(),
            staging_buffer.clone()
        );

        let mut builder = AutoCommandBufferBuilder::primary(
            &self.vulkano_core.allocators.commmand_buffer,
            self.vulkano_core.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();

        builder
            .copy_image_to_buffer(image_to_buffer)
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = self.previous_frame_end.take().unwrap()
            .then_execute(self.vulkano_core.queue.clone(), command_buffer)
            .unwrap();

        future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
        
        let chunk_data = staging_buffer.read().unwrap();
        
        let chunk = Chunk::from_raw_data(chunk_data.as_slice());
        
        game_state.terrain.chunks.insert(chunk_position, chunk);
        
        self.previous_frame_end = Some(sync::now(self.vulkano_core.device.clone()).boxed());
    }

    //noinspection RsUnresolvedPath
    fn generate_chunk(&mut self, chunk_position: Vector3<i32>) {
        let push_constants = terrain_gen::PushConstants{
            chunk_position: chunk_position.into(),
        };
        
        let chunk_index = chunk_buffer_index(chunk_position, &self.settings);

        let descriptor_set = PersistentDescriptorSet::new(
            &self.vulkano_core.allocators.descriptor_set,
            self.render_core.pipelines.terrain_generator_pipeline.pipeline.layout().set_layouts()[0].clone(),
            [
                WriteDescriptorSet::image_view(0, ImageView::new_default(self.render_core.buffers.block_data_buffers[chunk_index].clone()).unwrap()),
            ],
            [],
        ).unwrap();
        
        let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
            &self.vulkano_core.allocators.commmand_buffer,
            self.vulkano_core.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
            .unwrap();
        
        builder
            .bind_pipeline_compute(self.render_core.pipelines.terrain_generator_pipeline.pipeline.clone())
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
                self.render_core.pipelines.terrain_generator_pipeline.pipeline.layout().clone(),
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
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        
        
        self.previous_frame_end = Some(future.boxed());
    }
    
    fn wait_and_reset_last_frame_end(&mut self) {
        if let Some(future) = self.previous_frame_end.take() && future.queue().is_some() { 
            future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
        }

        self.previous_frame_end = Some(sync::now(self.vulkano_core.device.clone()).boxed());
    }
    pub fn toggle_confine(&mut self) {
        self.cursor_confined = !self.cursor_confined;
        self.vulkano_core.window.set_cursor_grab(if self.cursor_confined { winit::window::CursorGrabMode::Confined } else { winit::window::CursorGrabMode::None }).unwrap();
        self.vulkano_core.window.set_cursor_visible(!self.cursor_confined);
    }
}



fn chunk_buffer_index(chunk_position: Vector3<i32>, settings: &Settings) -> usize {
    let render_sl = 2 * settings.graphics_settings.render_distance as i32 + 1;
    chunk_position.map(
        |x|
            x.rem_euclid(render_sl)
    ).dot(&Vector3::new(1, render_sl, render_sl * render_sl)) as usize
}