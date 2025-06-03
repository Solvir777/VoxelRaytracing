use crate::graphics::vulkano_core::VulkanoCore;
use std::sync::Arc;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::{Image, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::pipeline::{ComputePipeline, Pipeline};
use vulkano::swapchain::{PresentMode, Swapchain, SwapchainCreateInfo};
use crate::graphics::buffers::Buffers;
use crate::graphics::Graphics;

pub struct SwapchainResources {
    pub swapchain: Arc<Swapchain>,
    pub swapchain_images: Vec<Arc<Image>>,
    pub descriptor_sets: Vec<Arc<PersistentDescriptorSet>>,
    pub recreate_swapchain: bool,
}

impl SwapchainResources {
    pub fn new(vulkano: &VulkanoCore, render_pipeline: Arc<ComputePipeline>, buffers: &Buffers) -> Self {
        let (swapchain, swapchain_images) = create_swapchain(vulkano);
        let descriptor_sets = create_image_descriptor_sets(&swapchain_images, vulkano, render_pipeline, buffers);
        Self{
            swapchain,
            swapchain_images,
            descriptor_sets,
            recreate_swapchain: true,
        }
    }

    pub fn recreate_swapchain(graphics: &mut Graphics) {
        let swapchain_resources = &mut graphics.render_core.swapchain_ressources;

        let (new_swapchain, new_images) =
            swapchain_resources
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: graphics.vulkano_core.window.inner_size().into(),
                ..swapchain_resources.swapchain.create_info()
            })
            .expect("failed to recreate swapchain");

        swapchain_resources.descriptor_sets = create_image_descriptor_sets(
            &new_images,
            &graphics.vulkano_core,
            graphics.render_core.pipelines.raytrace_pipeline.clone(),
            &graphics.render_core.buffers);

        swapchain_resources.swapchain = new_swapchain;
        swapchain_resources.recreate_swapchain = false;
    }
}

fn create_image_descriptor_sets
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

fn create_swapchain(vulkano: &VulkanoCore) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let surface_capabilities = vulkano
        .device
        .physical_device()
        .surface_capabilities(&vulkano.surface, Default::default())
        .unwrap();

    Swapchain::new(
        vulkano.device.clone(),
        vulkano.surface.clone(),
        SwapchainCreateInfo {
            min_image_count: surface_capabilities.min_image_count.max(2),
            image_format: Format::B8G8R8A8_UNORM,
            image_extent: vulkano.window.inner_size().into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::STORAGE,
            present_mode: PresentMode::Fifo,
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .into_iter()
                .next()
                .unwrap(),

            ..Default::default()
        },
    )
    .unwrap()
}
