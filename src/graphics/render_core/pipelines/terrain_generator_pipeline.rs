use crate::shaders;
use crate::shaders::terrain_gen::PushConstants;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::{
    PipelineDescriptorSetLayoutCreateInfo, PipelineLayoutCreateInfo, PushConstantRange,
};
use vulkano::pipeline::{ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::shader::ShaderStages;

pub struct TerrainGeneratorPipeline {
    pub pipeline: Arc<ComputePipeline>,
}
impl TerrainGeneratorPipeline {
    pub fn new(device: Arc<Device>) -> Self{
        let compute_shader = shaders::terrain_gen::load(device.clone()).unwrap();
        let entry_point = compute_shader.entry_point("main").unwrap();
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
        Self{
            pipeline
        }
    }
}