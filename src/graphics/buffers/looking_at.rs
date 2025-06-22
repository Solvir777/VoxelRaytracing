use crate::shaders::rendering::LookingAtBlock;

impl LookingAtBlock {
    pub fn new() -> Self {
        Self{
            hit_point: Default::default(),
            block_id: 0,
            hit_normal: Default::default(),
        }
    }
}