use crate::graphics::Graphics;
use crate::settings::graphics_settings::GraphicsSettings;
use crate::graphics::vulkano_core::VulkanoCore;
use std::sync::Arc;
use vulkano::format::Format;
use vulkano::image::{Image, ImageCreateInfo, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo};

pub struct Buffers {
    pub block_data_buffers: Box<[Arc<Image>]>,
}

impl Buffers {
    pub fn new(vulkano_core: &VulkanoCore, graphics_settings: &GraphicsSettings) -> Self {
        let block_data_buffers = create_block_data_buffers(vulkano_core, &graphics_settings);
        Self { block_data_buffers }
    }
    
    pub fn get_chunk_image_views(&self) -> Vec<Arc<ImageView>>{
        self.block_data_buffers.iter().map(
            |x|
                ImageView::new_default(x.clone()).unwrap()
        ).collect::<Vec<_>>()
    }
}

fn create_block_data_buffers(
    vulkano_core: &VulkanoCore,
    graphics_settings: &GraphicsSettings,
) -> Box<[Arc<Image>]> {
    
    let image_create_info = ImageCreateInfo {
        image_type: vulkano::image::ImageType::Dim3d,
        format: Format::R16_UINT,
        extent: [Graphics::CHUNK_SIZE; 3],
        usage: ImageUsage::STORAGE | ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC,
        ..Default::default()
    };

    let loaded_chunks_size = graphics_settings.render_distance as usize * 2 + 1;
    vec![(); loaded_chunks_size * loaded_chunks_size * loaded_chunks_size].iter().map(
        |_| {
            Image::new(
                vulkano_core.allocators.memory.clone(),
                image_create_info.clone(),
                AllocationCreateInfo::default(),
            )
                .unwrap()
        }
    ).collect::<Vec<_>>().into_boxed_slice()
}
