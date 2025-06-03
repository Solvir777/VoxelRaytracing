mod raytrace_pipeline;
mod render_descriptor_sets;

use crate::graphics::render_core::pipelines::raytrace_pipeline::create_raytrace_pipeline;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::pipeline::ComputePipeline;

pub struct Pipelines {
    pub raytrace_pipeline: Arc<ComputePipeline>,
}

impl Pipelines {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            raytrace_pipeline: create_raytrace_pipeline(device.clone()),
        }
    }
}
