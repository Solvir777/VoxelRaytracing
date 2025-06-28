mod raytrace_pipeline;
mod terrain_generator_pipeline;
mod terrain_distance_pipeline;

use crate::graphics::buffers::Buffers;
use crate::graphics::render_core::pipelines::raytrace_pipeline::RaytracePipeline;
use crate::graphics::render_core::pipelines::terrain_generator_pipeline::TerrainGeneratorPipeline;
use crate::graphics::vulkano_core::VulkanoCore;
use crate::settings::graphics_settings::GraphicsSettings;
use std::sync::Arc;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Device;
use vulkano::image::Image;
use vulkano::image::view::ImageView;
use vulkano::pipeline::{ComputePipeline, Pipeline};
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
