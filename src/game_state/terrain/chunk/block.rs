use crate::game_state::terrain::chunk::block::solid_block::SolidBlock;
use crate::game_state::terrain::chunk::block::transparent_block::TransparentBlock;

mod solid_block;
mod transparent_block;

pub enum Block {
    SolidBlock(SolidBlock),
    TransparentBlock(TransparentBlock),
    Air
}

impl Block {
    pub fn as_u16(&self) -> u16 {
        match self {
            Block::SolidBlock(_) => {1}
            Block::TransparentBlock(_) => {2}
            Block::Air => {0}
        }
    }
    pub fn from_u16(value: u16) -> Self {
        match value {
            1 => Block::SolidBlock(SolidBlock::Grass),
            2 => Block::TransparentBlock(TransparentBlock::Glass),
            _ => Block::Air
        }
    }
}