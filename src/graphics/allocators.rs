use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::Device;
use vulkano::memory::allocator::StandardMemoryAllocator;

pub struct Allocators {
    pub memory: Arc<StandardMemoryAllocator>,
    pub commmand_buffer: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set: Arc<StandardDescriptorSetAllocator>,
}

impl Allocators {
    pub fn new(device: Arc<Device>) -> Self {
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Self {
            memory: memory_allocator,
            commmand_buffer: command_buffer_allocator,
            descriptor_set: descriptor_set_allocator,
        }
    }
}
