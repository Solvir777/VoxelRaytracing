use crate::graphics::buffers::Buffers;
use crate::graphics::render_core::pipelines::Pipelines;
use crate::graphics::render_core::swapchain_resources::SwapchainResources;
use crate::graphics::vulkano_core::VulkanoCore;
use crate::settings::Settings;

mod pipelines;
pub mod swapchain_resources;

pub struct RenderCore {
    pub swapchain_ressources: SwapchainResources,
    pub pipelines: Pipelines,
    pub buffers: Buffers,
}

impl RenderCore {
    pub(crate) fn new(vulkano_core: &VulkanoCore, settings: &Settings) -> Self {
        let graphics_settings = &settings.graphics_settings;
        let buffers = Buffers::new(vulkano_core, &graphics_settings);
        let swapchain = SwapchainResources::new(vulkano_core);
        let pipelines = Pipelines::new(vulkano_core.device.clone(), &swapchain.swapchain_images, vulkano_core, &buffers, &settings.graphics_settings);
        Self {
            swapchain_ressources: swapchain,
            buffers,
            pipelines,
        }
    }
}
