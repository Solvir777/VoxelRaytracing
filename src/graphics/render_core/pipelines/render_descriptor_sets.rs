use crate::graphics::buffers::Buffers;
use std::sync::Arc;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::Image;
use vulkano::image::view::ImageView;
use vulkano::pipeline::{ComputePipeline, Pipeline};

pub fn create_raytrace_descriptor_sets(
    images: &Vec<Arc<Image>>,
    descriptor_set_allocator: &StandardDescriptorSetAllocator,
    pipeline: &Arc<ComputePipeline>,
    buffers: &Buffers,
) -> Vec<Arc<PersistentDescriptorSet>> {
    images
        .iter()
        .map(|x| {
            PersistentDescriptorSet::new(
                descriptor_set_allocator,
                pipeline.layout().set_layouts()[0].clone(),
                [
                    WriteDescriptorSet::image_view(0, ImageView::new_default(x.clone()).unwrap()),
                    WriteDescriptorSet::image_view(
                        1,
                        ImageView::new_default(buffers.block_data_buffer.clone()).unwrap(),
                    ),
                ],
                [],
            )
            .unwrap()
        })
        .collect()
}
