#[derive(Default, Copy, Clone, Debug)]
pub struct BlockData {
    pub block_type: BlockType,
}

#[repr(u32)]
#[derive(Eq, PartialEq, Default, Copy, Clone, Debug)]
pub enum BlockType {
    #[default]
    Air,
    Grass,
    Dirt,
}

impl BlockType {
    pub fn is_solid(&self) -> bool {
        match self {
            BlockType::Air => false,
            BlockType::Grass => true,
            BlockType::Dirt => true,
        }
    }
}
