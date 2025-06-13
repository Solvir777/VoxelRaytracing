mod raytrace_pipeline;
mod terrain_generator_pipeline;

use crate::graphics::render_core::pipelines::raytrace_pipeline::create_raytrace_pipeline;
use crate::graphics::render_core::pipelines::terrain_generator_pipeline::create_terrain_generator_pipeline;
use std::sync::Arc;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Device;
use vulkano::image::Image;
use vulkano::image::view::ImageView;
use vulkano::pipeline::{ComputePipeline, Pipeline};
use crate::graphics::buffers::Buffers;
use crate::graphics::vulkano_core::VulkanoCore;

pub struct RaytracePipeline {
    pub pipeline: Arc<ComputePipeline>,
    pub descriptor_sets: Vec<Arc<PersistentDescriptorSet>>,
}
pub struct GeneratorPipeline {
    pub pipeline: Arc<ComputePipeline>,
    pub descriptor_set: Arc<PersistentDescriptorSet>,
}

pub struct Pipelines {
    pub raytrace_pipeline: RaytracePipeline,
    pub terrain_generator_pipeline: GeneratorPipeline,
}

impl Pipelines {
    pub fn new(device: Arc<Device>, swapchain_images: &Vec<Arc<Image>>, vulkano: &VulkanoCore, buffers: &Buffers) -> Self {
        let raytrace_pipeline = create_raytrace_pipeline(device.clone());
        let raytrace_descriptor_sets = create_raytrace_descriptor_sets(&swapchain_images, vulkano, raytrace_pipeline.clone(), buffers);

        let terrain_generator_pipeline = create_terrain_generator_pipeline(device.clone());
        let terrain_generator_descriptor_set = create_terrain_descriptor_set(vulkano, terrain_generator_pipeline.clone(), buffers);
        Self {
            raytrace_pipeline: RaytracePipeline {
                pipeline: raytrace_pipeline,
                descriptor_sets: raytrace_descriptor_sets,
            },
            terrain_generator_pipeline: GeneratorPipeline {
                pipeline: terrain_generator_pipeline,
                descriptor_set: terrain_generator_descriptor_set,
            },
        }
    }

    pub fn recreate_image_descriptor_sets(&mut self, images: &Vec<Arc<Image>>, vulkano: &VulkanoCore, pipeline: Arc<ComputePipeline>, buffers: &Buffers) {
        self.raytrace_pipeline.descriptor_sets = create_raytrace_descriptor_sets(images, vulkano, pipeline, buffers);
    }
}

fn create_terrain_descriptor_set(vulkano: &VulkanoCore, pipeline: Arc<ComputePipeline>, buffers: &Buffers) -> Arc<PersistentDescriptorSet> {
    PersistentDescriptorSet::new(
        &vulkano.allocators.descriptor_set,
        pipeline.layout().set_layouts()[0].clone(),
        [
            WriteDescriptorSet::image_view(0, ImageView::new_default(buffers.block_data_buffer.clone()).unwrap()),
        ],
        [],
    ).unwrap()
}

fn create_raytrace_descriptor_sets
(
    images: &Vec<Arc<Image>>,
    vulkano: &VulkanoCore,
    render_pipeline: Arc<ComputePipeline>,
    buffers: &Buffers,
) -> Vec<Arc<PersistentDescriptorSet>> {
    images
        .iter()
        .map(|x|
            PersistentDescriptorSet::new(
                &vulkano.allocators.descriptor_set,
                render_pipeline.layout().set_layouts()[0].clone(),
                [
                    WriteDescriptorSet::image_view(0, ImageView::new_default(x.clone()).unwrap()),
                    WriteDescriptorSet::image_view(1, ImageView::new_default(buffers.block_data_buffer.clone()).unwrap()),
                ],
                [],
            )
                .unwrap()
        ).collect()
}