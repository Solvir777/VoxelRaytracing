use crate::graphics::Graphics;
use crate::graphics::vulkano_core::VulkanoCore;
use crate::settings::graphics_settings::GraphicsSettings;
use crate::shaders::rendering::LookingAtBlock;
use nalgebra::Vector3;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};

pub struct Buffers {
    pub block_data_buffers: Box<[Arc<Image>]>,
    pub player_raycast_buffer: Subbuffer<LookingAtBlock>,
}

impl Buffers {
    pub fn new(vulkano_core: &VulkanoCore, graphics_settings: &GraphicsSettings) -> Self {
        let player_raycast_buffer = create_looking_at_buffer(vulkano_core);
        let block_data_buffers = create_block_data_buffers(vulkano_core, &graphics_settings);
        Self {
            block_data_buffers,
            player_raycast_buffer,
        }
    }

    pub fn get_chunk_image_views(&self) -> Vec<Arc<ImageView>> {
        self.block_data_buffers
            .iter()
            .map(|x| ImageView::new_default(x.clone()).unwrap())
            .collect::<Vec<_>>()
    }
}

fn create_looking_at_buffer(vulkano_core: &VulkanoCore) -> Subbuffer<LookingAtBlock> {
    let content = LookingAtBlock {
        hit_point: Vector3::zeros(),
        block_id: 0,
        hit_normal: Vector3::zeros(),
    };
    Buffer::from_data(
        vulkano_core.allocators.memory.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        content,
    )
    .unwrap()
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
    vec![(); loaded_chunks_size * loaded_chunks_size * loaded_chunks_size]
        .iter()
        .map(|_| {
            Image::new(
                vulkano_core.allocators.memory.clone(),
                image_create_info.clone(),
                AllocationCreateInfo::default(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
