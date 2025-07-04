use crate::settings::graphics_settings::GraphicsSettings;
use crate::shaders;
use crate::shaders::rendering::PushConstants;
use std::sync::Arc;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Device;
use vulkano::image::Image;
use vulkano::image::view::ImageView;
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::{
    PipelineDescriptorSetLayoutCreateInfo, PipelineLayoutCreateInfo, PushConstantRange,
};
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::shader::ShaderStages;
use crate::graphics::buffers::Buffers;
use crate::graphics::vulkano_core::VulkanoCore;

pub struct RaytracePipeline {
    pub pipeline: Arc<ComputePipeline>,
    pub descriptor_sets: Vec<Arc<PersistentDescriptorSet>>,
}

impl RaytracePipeline {
    pub fn new(
        device: Arc<Device>,
        graphics_settings: &GraphicsSettings,
        images: &Vec<Arc<Image>>,
        vulkano: &VulkanoCore,
        buffers: &Buffers,
    ) -> Self {
        let compute_shader = shaders::rendering::load(device.clone()).unwrap();

        let entry_point = compute_shader
            .specialize(
                [(0, (graphics_settings.render_distance as i32).into())]
                    .into_iter()
                    .collect(),
            )
            .unwrap()
            .entry_point("main")
            .unwrap();
        let stage_info = PipelineShaderStageCreateInfo::new(entry_point);

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineLayoutCreateInfo {
                push_constant_ranges: vec![PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    offset: 0,
                    size: size_of::<PushConstants>() as u32,
                }],
                ..PipelineDescriptorSetLayoutCreateInfo::from_stages(&[stage_info.clone()])
                    .into_pipeline_layout_create_info(device.clone())
                    .unwrap()
            },
        )
            .unwrap();

        let pipeline = ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage_info, layout),
        )
            .unwrap();
        
        let mut ret = Self{
            pipeline,
            descriptor_sets: vec!()
        };
        ret.recreate_raytrace_descriptor_sets(images, vulkano, buffers);
        
        ret
    }
    
    
    
    pub fn recreate_raytrace_descriptor_sets(
        &mut self,
        images: &Vec<Arc<Image>>,
        vulkano: &VulkanoCore,
        buffers: &Buffers,
    ) {
        self.descriptor_sets = images
            .iter()
            .map(|x| {
                PersistentDescriptorSet::new(
                    &vulkano.allocators.descriptor_set,
                    self.pipeline.layout().set_layouts()[0].clone(),
                    [
                        WriteDescriptorSet::image_view(0, ImageView::new_default(x.clone()).unwrap()),
                        WriteDescriptorSet::buffer(1, buffers.player_raycast_buffer.clone()),
                        WriteDescriptorSet::buffer(2, buffers.gpu_graphics_settings_buffer.clone()),
                        WriteDescriptorSet::image_view_array(3, 0, buffers.get_chunk_image_views()),
                        WriteDescriptorSet::image_view_array(4, 0, buffers.get_distance_image_views()),
                        WriteDescriptorSet::image_view(5, buffers.textures.image_view.clone()),
                        WriteDescriptorSet::sampler(6, buffers.textures.sampler.clone()),
                    ],
                    [],
                )
                    .unwrap()
            })
            .collect()
    }

}
