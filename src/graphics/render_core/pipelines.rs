mod raytrace_pipeline;
mod terrain_generator_pipeline;

use crate::graphics::buffers::Buffers;
use crate::graphics::render_core::pipelines::raytrace_pipeline::create_raytrace_pipeline;
use crate::graphics::render_core::pipelines::terrain_generator_pipeline::create_terrain_generator_pipeline;
use crate::graphics::vulkano_core::VulkanoCore;
use crate::settings::graphics_settings::GraphicsSettings;
use std::sync::Arc;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Device;
use vulkano::image::Image;
use vulkano::image::view::ImageView;
use vulkano::pipeline::{ComputePipeline, Pipeline};

pub struct RaytracePipeline {
    pub pipeline: Arc<ComputePipeline>,
    pub descriptor_sets: Vec<Arc<PersistentDescriptorSet>>,
}
pub struct GeneratorPipeline {
    pub pipeline: Arc<ComputePipeline>,
}

pub struct Pipelines {
    pub raytrace_pipeline: RaytracePipeline,
    pub terrain_generator_pipeline: GeneratorPipeline,
}

impl Pipelines {
    pub fn new(
        device: Arc<Device>,
        swapchain_images: &Vec<Arc<Image>>,
        vulkano: &VulkanoCore,
        buffers: &Buffers,
        graphics_settings: &GraphicsSettings,
    ) -> Self {
        let raytrace_pipeline = create_raytrace_pipeline(device.clone(), graphics_settings);
        let raytrace_descriptor_sets = create_raytrace_descriptor_sets(
            &swapchain_images,
            vulkano,
            raytrace_pipeline.clone(),
            buffers,
        );

        let terrain_generator_pipeline = create_terrain_generator_pipeline(device.clone());
        Self {
            raytrace_pipeline: RaytracePipeline {
                pipeline: raytrace_pipeline,
                descriptor_sets: raytrace_descriptor_sets,
            },
            terrain_generator_pipeline: GeneratorPipeline {
                pipeline: terrain_generator_pipeline,
            },
        }
    }

    pub fn recreate_image_descriptor_sets(
        &mut self,
        images: &Vec<Arc<Image>>,
        vulkano: &VulkanoCore,
        pipeline: Arc<ComputePipeline>,
        buffers: &Buffers,
    ) {
        self.raytrace_pipeline.descriptor_sets =
            create_raytrace_descriptor_sets(images, vulkano, pipeline, buffers);
    }
}

fn create_raytrace_descriptor_sets(
    images: &Vec<Arc<Image>>,
    vulkano: &VulkanoCore,
    render_pipeline: Arc<ComputePipeline>,
    buffers: &Buffers,
) -> Vec<Arc<PersistentDescriptorSet>> {
    images
        .iter()
        .map(|x| {
            PersistentDescriptorSet::new(
                &vulkano.allocators.descriptor_set,
                render_pipeline.layout().set_layouts()[0].clone(),
                [
                    WriteDescriptorSet::image_view(0, ImageView::new_default(x.clone()).unwrap()),
                    WriteDescriptorSet::buffer(1, buffers.player_raycast_buffer.clone()),
                    WriteDescriptorSet::buffer(2, buffers.gpu_graphics_settings_buffer.clone()),
                    WriteDescriptorSet::image_view_array(3, 0, buffers.get_chunk_image_views()),
                ],
                [],
            )
            .unwrap()
        })
        .collect()
}
