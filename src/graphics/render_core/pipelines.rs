mod raytrace_pipeline;
mod terrain_generator_pipeline;
pub mod terrain_distance_pipeline;

use crate::graphics::buffers::Buffers;
use crate::graphics::render_core::pipelines::raytrace_pipeline::RaytracePipeline;
use crate::graphics::render_core::pipelines::terrain_generator_pipeline::TerrainGeneratorPipeline;
use crate::graphics::vulkano_core::VulkanoCore;
use crate::settings::graphics_settings::GraphicsSettings;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::image::Image;
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::shader::ShaderModule;
use crate::graphics::render_core::pipelines::terrain_distance_pipeline::TerrainDistancePipeline;

pub struct Pipelines {
    pub raytrace_pipeline: RaytracePipeline,
    pub terrain_generator_pipeline: TerrainGeneratorPipeline,
    pub terrain_distance_pipeline: TerrainDistancePipeline,
}

impl Pipelines {
    pub fn new(
        device: Arc<Device>,
        swapchain_images: &Vec<Arc<Image>>,
        vulkano: &VulkanoCore,
        buffers: &Buffers,
        graphics_settings: &GraphicsSettings,
    ) -> Self {
        Self {
            raytrace_pipeline: RaytracePipeline::new(device.clone(), graphics_settings, swapchain_images, vulkano, buffers),
            terrain_generator_pipeline: TerrainGeneratorPipeline::new(device.clone()),
            terrain_distance_pipeline: TerrainDistancePipeline::new(device.clone()),
        }
    }

    pub fn recreate_image_descriptor_sets(
        &mut self,
        images: &Vec<Arc<Image>>,
        vulkano: &VulkanoCore,
        buffers: &Buffers,
    ) {
        self.raytrace_pipeline.recreate_raytrace_descriptor_sets(images, vulkano, buffers);
    }
}


pub fn default_pipeline_from_shader_module(device: Arc<Device>, module: Arc<ShaderModule>) -> Arc<ComputePipeline> {
    let entry_point = module
        .entry_point("main")
        .unwrap();
    let stage_info = PipelineShaderStageCreateInfo::new(entry_point);

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&[stage_info.clone()])
            .into_pipeline_layout_create_info(device.clone())
            .unwrap()
    )
        .unwrap();

    ComputePipeline::new(
        device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage_info, layout),
    )
        .unwrap()
}