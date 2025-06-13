use crate::game_state::terrain::chunk::block::solid_block::SolidBlock;
use crate::game_state::terrain::chunk::block::transparent_block::TransparentBlock;

mod solid_block;
mod transparent_block;

pub enum Block {
    SolidBlock(SolidBlock),
    TransparentBlock(TransparentBlock),
    Air
}