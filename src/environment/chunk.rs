use std::sync::Arc;

use bevy::{
    math::{IVec3, ivec3},
    platform::collections::HashMap,
};

use super::{
    block::BlockData,
    quad::Direction,
    utils::{index_to_ivec3_bounds, vec3_to_index},
};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_P: usize = CHUNK_SIZE + 2;
pub const CHUNK_SIZE_P2: usize = CHUNK_SIZE_P * CHUNK_SIZE_P;
pub const CHUNK_SIZE_P3: usize = CHUNK_SIZE_P * CHUNK_SIZE_P * CHUNK_SIZE_P;
pub const CHUNK_SIZE2: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_SIZE2_I32: i32 = CHUNK_SIZE2 as i32;
pub const CHUNK_SIZE3: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone)]
pub struct ChunkData {
    pub voxels: Vec<BlockData>,
}

impl ChunkData {
    #[inline]
    pub fn get_block(&self, index: usize) -> &BlockData {
        if self.voxels.len() == 1 {
            &self.voxels[0]
        } else {
            &self.voxels[index]
        }
    }

    // returns the block type if all voxels are the same
    #[inline]
    pub fn get_block_if_filled(&self) -> Option<&BlockData> {
        if self.voxels.len() == 1 {
            Some(&self.voxels[0])
        } else {
            None
        }
    }
}

// pointers to chunk data, a middle one with all their neighbours
#[derive(Clone)]
pub struct ChunksRefs {
    pub chunks: Vec<Arc<ChunkData>>,
}

impl ChunksRefs {
    ///! construct a ChunkRefs at middle_chunk position
    ///! safety: panics if ChunkData doesn't exist in input world_data
    pub fn try_new(
        world_data: &HashMap<IVec3, Arc<ChunkData>>,
        middle_chunk: IVec3,
    ) -> Option<Self> {
        let mut chunks = vec![];
        for i in 0..3 * 3 * 3 {
            let offset = index_to_ivec3_bounds(i, 3) + IVec3::splat(-1);
            chunks.push(Arc::clone(
                world_data.get(&(middle_chunk + offset)).unwrap(),
            ))
        }
        Some(Self { chunks })
    }

    // returns if all the voxels are the same
    // this is an incredibly fast approximation (1 sample per chunk) all = voxels[0]
    // so may be inacurate, but the odds are incredibly low
    pub fn is_all_voxels_same(&self) -> bool {
        let first_block = self.chunks[0].get_block_if_filled();

        let Some(block) = first_block else {
            return false;
        };

        for chunk in self.chunks[1..].iter() {
            let option = chunk.get_block_if_filled();
            if let Some(v) = option {
                if block.block_type != v.block_type {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    ///! helper function to get block data that may exceed the bounds of the middle chunk
    ///! input position is local pos to middle chunk
    pub fn get_block(&self, pos: IVec3) -> &BlockData {
        let x = (pos.x + 32) as u32;
        let y = (pos.y + 32) as u32;
        let z = (pos.z + 32) as u32;
        let (x_chunk, x) = ((x / 32) as i32, (x % 32) as i32);
        let (y_chunk, y) = ((y / 32) as i32, (y % 32) as i32);
        let (z_chunk, z) = ((z / 32) as i32, (z % 32) as i32);

        let chunk_index = vec3_to_index(IVec3::new(x_chunk, y_chunk, z_chunk), 3);
        let chunk_data = &self.chunks[chunk_index];
        let i = vec3_to_index(IVec3::new(x, y, z), 32);
        chunk_data.get_block(i)
    }

    ///! helper function to get voxels
    ///! panics if the local pos is outside the middle chunk
    pub fn get_block_no_neighbour(&self, pos: IVec3) -> &BlockData {
        let chunk_data = &self.chunks[13];
        let i = vec3_to_index(pos, 32);
        chunk_data.get_block(i)
    }

    ///! helper function to sample adjacent(back,left,down) voxels
    pub fn get_adjacent_blocks(
        &self,
        pos: IVec3,
        // current back, left, down
    ) -> (&BlockData, &BlockData, &BlockData, &BlockData) {
        let current = self.get_block(pos);
        let back = self.get_block(pos + ivec3(0, 0, -1));
        let left = self.get_block(pos + ivec3(-1, 0, 0));
        let down = self.get_block(pos + ivec3(0, -1, 0));
        (current, back, left, down)
    }

    ///! helper function to sample adjacent voxels, von neuman include all facing planes
    pub fn get_von_neumann(&self, pos: IVec3) -> Option<Vec<(Direction, &BlockData)>> {
        let mut result = vec![];
        result.push((Direction::Back, self.get_block(pos + ivec3(0, 0, -1))));
        result.push((Direction::Forward, self.get_block(pos + ivec3(0, 0, 1))));
        result.push((Direction::Down, self.get_block(pos + ivec3(0, -1, 0))));
        result.push((Direction::Up, self.get_block(pos + ivec3(0, 1, 0))));
        result.push((Direction::Left, self.get_block(pos + ivec3(-1, 0, 0))));
        result.push((Direction::Right, self.get_block(pos + ivec3(1, 0, 0))));
        Some(result)
    }

    pub fn get_2(&self, pos: IVec3, offset: IVec3) -> (&BlockData, &BlockData) {
        let first = self.get_block(pos);
        let second = self.get_block(pos + offset);
        (first, second)
    }
}
