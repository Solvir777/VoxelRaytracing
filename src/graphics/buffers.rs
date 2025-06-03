use crate::graphics::Graphics;
use crate::settings::graphics_settings::GraphicsSettings;
use crate::graphics::vulkano_core::VulkanoCore;
use std::sync::Arc;
use vulkano::format::Format;
use vulkano::image::{Image, ImageCreateInfo, ImageUsage};
use vulkano::memory::allocator::AllocationCreateInfo;

pub struct Buffers {
    pub block_data_buffer: Arc<Image>,
}

impl Buffers {
    pub(crate) fn new(vulkano_core: &VulkanoCore, graphics_settings: &GraphicsSettings) -> Self {
        let block_data_buffer = create_block_data_buffer(vulkano_core, &graphics_settings);

        Self { block_data_buffer }
    }
}

fn create_block_data_buffer(
    vulkano_core: &VulkanoCore,
    graphics_settings: &GraphicsSettings,
) -> Arc<Image> {
    let loaded_voxels_size =
        (graphics_settings.render_distance as u32 * 2 + 1) * Graphics::CHUNK_SIZE;

    let image_create_info = ImageCreateInfo {
        image_type: vulkano::image::ImageType::Dim3d,
        format: Format::R16_UINT,
        extent: [loaded_voxels_size; 3],
        usage: ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
        ..Default::default()
    };

    Image::new(
        vulkano_core.allocators.memory.clone(),
        image_create_info,
        AllocationCreateInfo::default(),
    )
    .unwrap()
}
