use std::sync::Arc;
use vulkano::device::Device;
use vulkano::pipeline::{ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::{PipelineDescriptorSetLayoutCreateInfo, PipelineLayoutCreateInfo, PushConstantRange};
use vulkano::shader::ShaderStages;
use crate::shaders;
use crate::shaders::rendering::PushConstants;

pub struct TerrainDistancePipeline {
    pub pipeline: Arc<ComputePipeline>,
}

impl TerrainDistancePipeline {
    pub fn new(device: Arc<Device>) -> Self {
        let compute_shader = shaders::distance_gen::load(device.clone()).unwrap();

        let entry_point = compute_shader
            .entry_point("main")
            .unwrap();
        let stage_info = PipelineShaderStageCreateInfo::new(entry_point);

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&[stage_info.clone()])
                .into_pipeline_layout_create_info(device.clone())
                .unwrap()
        )
            .unwrap();

        let pipeline = ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage_info, layout),
        )
            .unwrap();
        
        TerrainDistancePipeline{
            pipeline,
        }
        
    }
}