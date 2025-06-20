use crate::shaders;
use crate::shaders::rendering::PushConstants;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::{
    PipelineDescriptorSetLayoutCreateInfo, PipelineLayoutCreateInfo, PushConstantRange,
};
use vulkano::pipeline::{ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::shader::ShaderStages;
use crate::settings::graphics_settings::GraphicsSettings;

pub fn create_raytrace_pipeline(device: Arc<Device>, graphics_settings: &GraphicsSettings) -> Arc<ComputePipeline> {
    let compute_shader = shaders::rendering::load(device.clone()).unwrap();
    
    let entry_point = compute_shader
        .specialize(
            [(0, (graphics_settings.render_distance as i32).into())]
                .into_iter()
                .collect(),
        )
        .unwrap()
        .entry_point("main").unwrap();
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

    ComputePipeline::new(
        device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage_info, layout),
    )
    .unwrap()
}
