use crate::game_state::terrain::block::solid_block::SolidBlock;
use crate::graphics::vulkano_core::VulkanoCore;
use std::mem;
use std::sync::Arc;
use vulkano::DeviceSize;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{CopyBufferToImageInfo, PrimaryCommandBufferAbstract};
use vulkano::format::Format;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::sync::GpuFuture;

const TEXTURE_SIZE: u32 = 16;

pub fn create_block_texture_view(vulkano_core: &VulkanoCore) -> Arc<ImageView> {
    let texture_image = create_texture_image(vulkano_core);
    ImageView::new_default(texture_image).unwrap()
}

fn create_texture_image(vulkano_core: &VulkanoCore) -> Arc<Image> {
    let format = Format::R8G8B8A8_SRGB;
    const ARRAY_LAYERS: u32 = mem::variant_count::<SolidBlock>() as u32;

    let buffer_size = format.block_size()
        * (TEXTURE_SIZE * TEXTURE_SIZE) as DeviceSize
        * ARRAY_LAYERS as DeviceSize;

    let upload_buffer = Buffer::new_slice(
        vulkano_core.allocators.memory.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        buffer_size,
    )
    .unwrap();

    {
        let mut image_data = &mut *upload_buffer.write().unwrap();

        for png_bytes in [
            include_bytes!("blocks/grass_side.png").as_slice(),
            include_bytes!("blocks/grass_side.png").as_slice(),
            include_bytes!("blocks/grass_side.png").as_slice(),
        ] {
            let decoder = png::Decoder::new(png_bytes);
            let mut reader = decoder.read_info().unwrap();
            reader.next_frame(image_data).unwrap();
            let info = reader.info();
            image_data = &mut image_data[(info.width * info.height * 4) as usize..];
        }
    }

    let texture_image = Image::new(
        vulkano_core.allocators.memory.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format,
            extent: [TEXTURE_SIZE, TEXTURE_SIZE, 1],
            array_layers: ARRAY_LAYERS,
            usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )
    .unwrap();
    
    copy_to_image(vulkano_core, upload_buffer.clone(), texture_image.clone());
    texture_image    
}

fn copy_to_image(vulkano_core: &VulkanoCore, buffer: Subbuffer<[u8]>, image: Arc<Image>) {
    let copy_info = CopyBufferToImageInfo::buffer_image(buffer.clone(), image.clone());
    let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
        &vulkano_core.allocators.commmand_buffer,
        vulkano_core.queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
    )
        .unwrap();

    builder.copy_buffer_to_image(copy_info).unwrap();

    let command_buffer = builder.build().unwrap();
    let future = command_buffer.execute(vulkano_core.queue.clone()).unwrap();
    future.then_signal_fence_and_flush().unwrap().wait(None).unwrap();
    
}