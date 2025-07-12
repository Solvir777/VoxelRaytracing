use crate::game_state::terrain::block::solid_block::SolidBlock;
use crate::game_state::terrain::block::transparent_block::TransparentBlock;

pub mod solid_block;
pub mod transparent_block;

pub enum Block {
    SolidBlock(SolidBlock),
    TransparentBlock(TransparentBlock),
    Air,
}

impl Block {
    pub fn as_u16(&self) -> u16 {
        match self {
            Block::SolidBlock(SolidBlock::Grass) => 1,
            Block::SolidBlock(SolidBlock::Stone) => 2,
            Block::TransparentBlock(_) => 3,
            _ => 0
        }
    }
    pub fn from_u16(value: u16) -> Self {
        match value {
            1 => Block::SolidBlock(SolidBlock::Grass),
            2 => Block::SolidBlock(SolidBlock::Stone),
            _ => Block::Air,
        }
    }
}
