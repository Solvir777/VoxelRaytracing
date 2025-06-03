use std::any::Any;
use crate::graphics::render_core::RenderCore;
use crate::graphics::vulkano_core::VulkanoCore;
use vulkano::pipeline::Pipeline;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::EventLoop;
use crate::game_state::GameState;
use crate::graphics::render_core::swapchain_resources::SwapchainResources;
use crate::input_state::InputState;
use crate::settings::Settings;

mod allocators;
mod buffers;
mod render_core;
mod vulkano_core;

pub(crate) struct Graphics {
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    vulkano_core: VulkanoCore,
    render_core: RenderCore,
}

impl Graphics {
    pub const CHUNK_SIZE: u32 = 32;
    pub fn new(settings: &Settings) -> (Self, EventLoop<()>) {
        let (vulkano_core, event_loop) = VulkanoCore::new();

        let previous_frame_end: Option<Box<dyn GpuFuture>> = Some(Box::new(sync::now(vulkano_core.device.clone())));

        let render_core = RenderCore::new(&vulkano_core, &settings);

        (
            Self {
                previous_frame_end,
                vulkano_core,
                render_core,
            },
            event_loop
        )
    }

    pub fn run(
        mut self,
        mut game_state: GameState,
        mut input_state: InputState,
        settings: Settings,
        event_loop: EventLoop<()>,
    ) -> !{
        event_loop.run(move |event, elwt, control_flow|
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
                    if input_state.is_key_pressed(winit::event::VirtualKeyCode::Escape) {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    game_state.update(&input_state, &settings);

                    self.draw_frame(&game_state);
                    input_state.refresh();
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
            .bind_pipeline_compute(self.render_core.pipelines.raytrace_pipeline.clone())
            .unwrap()
            .push_constants(
                self.render_core
                    .pipelines
                    .raytrace_pipeline
                    .layout()
                    .clone(),
                0,
                game_state.get_push_constants(),
            )
            .unwrap()
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Compute,
                self.render_core.pipelines.raytrace_pipeline.layout().clone(),
                0,
                self.render_core.swapchain_ressources.descriptor_sets[image_index as usize].clone(),
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
        if!self.render_core.swapchain_ressources.recreate_swapchain {
            return;
        }
        SwapchainResources::recreate_swapchain(self);
    }
}
