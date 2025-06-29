use std::sync::Arc;
use nalgebra::Vector3;
use vulkano::command_buffer::CommandBufferUsage;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Device;
use vulkano::image::view::ImageView;
use vulkano::pipeline::{ComputePipeline, Pipeline};
use vulkano::sync::GpuFuture;
use crate::graphics::{chunk_buffer_index, Graphics};
use crate::graphics::render_core::pipelines::default_pipeline_from_shader_module;
use crate::shaders;

pub struct TerrainDistancePipeline {
    pub sweep_pipeline: Arc<ComputePipeline>,
    pub setup_pipeline: Arc<ComputePipeline>,
}

impl TerrainDistancePipeline {
    pub fn new(device: Arc<Device>) -> Self {
        TerrainDistancePipeline{
            sweep_pipeline: sweep_pipeline(device.clone()),
            setup_pipeline: setup_pipeline(device.clone()),
        }
    }
}

fn sweep_pipeline(device: Arc<Device>) -> Arc<ComputePipeline> {
    let compute_shader = shaders::distance_gen::load(device.clone()).unwrap();
    
    default_pipeline_from_shader_module(device.clone(), compute_shader)
}


fn setup_pipeline(device: Arc<Device>) -> Arc<ComputePipeline> {
    let compute_shader = shaders::distance_setup::load(device.clone()).unwrap();

    default_pipeline_from_shader_module(device.clone(), compute_shader)
}


pub fn execute(graphics: &mut Graphics, chunk_position: Vector3<i32>) {
    graphics.wait_and_reset_last_frame_end();
    execute_setup(graphics, chunk_position);
    execute_sweeps(graphics, chunk_position);
}

fn execute_setup(graphics: &mut Graphics, chunk_position: Vector3<i32>) {
    let descriptor_set = PersistentDescriptorSet::new(
        &graphics.vulkano_core.allocators.descriptor_set,
        graphics.render_core
            .pipelines
            .terrain_distance_pipeline
            .setup_pipeline
            .layout()
            .set_layouts()[0]
            .clone(),
        [
            WriteDescriptorSet::image_view(
                0,
                ImageView::new_default(
                    graphics.render_core.buffers.block_data_buffers[chunk_buffer_index(chunk_position, &graphics.settings)].clone()
                ).unwrap()
            ),
            WriteDescriptorSet::image_view(
                1,
                ImageView::new_default(
                    graphics.render_core.buffers.distance_data_buffers[chunk_buffer_index(chunk_position, &graphics.settings)].clone()
                ).unwrap()
            )
        ],
        [],
    )
        .unwrap();

    let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
        &graphics.vulkano_core.allocators.commmand_buffer,
        graphics.vulkano_core.queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
        .unwrap();

    builder
        .bind_pipeline_compute(
            graphics.render_core
                .pipelines
                .terrain_distance_pipeline
                .setup_pipeline
                .clone(),
        )
        .unwrap()
        .bind_descriptor_sets(
            vulkano::pipeline::PipelineBindPoint::Compute,
            graphics.render_core
                .pipelines
                .terrain_distance_pipeline
                .setup_pipeline
                .layout()
                .clone(),
            0,
            descriptor_set,
        )
        .unwrap()
        .dispatch([Graphics::CHUNK_SIZE / 8; 3])
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = graphics
        .previous_frame_end
        .take()
        .unwrap()
        .then_execute(graphics.vulkano_core.queue.clone(), command_buffer)
        .unwrap();

    graphics.previous_frame_end = Some(future.boxed());
}

fn execute_sweeps(graphics: &mut Graphics, chunk_position: Vector3<i32>) {
    let descriptor_set = PersistentDescriptorSet::new(
        &graphics.vulkano_core.allocators.descriptor_set,
        graphics.render_core
            .pipelines
            .terrain_distance_pipeline
            .sweep_pipeline
            .layout()
            .set_layouts()[0]
            .clone(),
        [
            WriteDescriptorSet::image_view(
                0,
                ImageView::new_default(
                    graphics.render_core.buffers.distance_data_buffers[chunk_buffer_index(chunk_position, &graphics.settings)].clone()
                ).unwrap()
            )
        ],
        [],
    )
        .unwrap();

    let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
        &graphics.vulkano_core.allocators.commmand_buffer,
        graphics.vulkano_core.queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
        .unwrap();

    builder
        .bind_pipeline_compute(
            graphics.render_core
                .pipelines
                .terrain_distance_pipeline
                .sweep_pipeline
                .clone(),
        )
        .unwrap()
        .bind_descriptor_sets(
            vulkano::pipeline::PipelineBindPoint::Compute,
            graphics.render_core
                .pipelines
                .terrain_distance_pipeline
                .sweep_pipeline
                .layout()
                .clone(),
            0,
            descriptor_set,
        )
        .unwrap();

    const DIRECTIONS: [Vector3<i32>; 8] = [
        Vector3::new(1, 1, 1),
        Vector3::new(-1, -1, -1),
        Vector3::new(1, 1, -1),
        Vector3::new(-1, -1, 1),
        Vector3::new(1, -1, 1),
        Vector3::new(-1, 1, -1),
        Vector3::new(1, -1, -1),
        Vector3::new(-1, 1, 1),
    ];

    for dir in DIRECTIONS.iter().chain(DIRECTIONS.iter()) {
        builder
            .push_constants(
                graphics.render_core.pipelines.terrain_distance_pipeline.sweep_pipeline.layout().clone(),
                0,
                *dir,
            )
            .unwrap()
            .dispatch(
                [Graphics::CHUNK_SIZE / 16, Graphics::CHUNK_SIZE / 16, 1]
            ).unwrap();
    }

    let command_buffer = builder.build().unwrap();

    let future = graphics
        .previous_frame_end
        .take()
        .unwrap()
        .then_execute(graphics.vulkano_core.queue.clone(), command_buffer)
        .unwrap();

    graphics.previous_frame_end = Some(future.boxed());
}