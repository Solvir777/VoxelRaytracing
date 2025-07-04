use std::sync::Arc;
use vulkano::device::DeviceOwned;
use vulkano::image::sampler::{Sampler, SamplerCreateInfo};
use vulkano::image::view::ImageView;
use crate::graphics::vulkano_core::VulkanoCore;
use crate::textures::create_block_texture_view;

pub struct Textures {
    pub sampler: Arc<Sampler>,
    pub image_view: Arc<ImageView>,
}

impl Textures {
    pub fn new(vulkano_core: &VulkanoCore) -> Self {
        let textures = create_block_texture_view(vulkano_core);
        let sampler = Sampler::new(
            vulkano_core.device.clone(),
            SamplerCreateInfo::default(),
        ).unwrap();
        Self{
            image_view: textures,
            sampler
        }
    }
}