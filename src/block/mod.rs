use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use slab::Slab;

use self::texture::BlockTextureId;

pub mod texture;

#[derive(Resource, Default)]
pub struct Blocks {
    blocks: Slab<()>,
    meshing_data: Vec<MeshingData>,
}

impl Blocks {
    #[inline]
    pub fn new_block(
        &mut self,
        texture: BlockTextureId,
        opacity: u32,
    ) -> Result<BlockId, ()> {
        let ix = self.blocks.insert(());
        if ix >= u32::MAX as usize {
            Err(())
        } else {
            let id = BlockId(ix as u32);
            while self.meshing_data.len() <= ix {
                self.meshing_data.push(default())
            }
            self.meshing_data[ix] = MeshingData { texture, opacity };
            Ok(id)
        }
    }

    #[inline]
    pub fn get_meshing_data(&self, id: BlockId) -> MeshingData {
        self.meshing_data
            .get(id.0 as usize)
            .copied()
            .unwrap_or_default()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct MeshingData {
    pub texture: BlockTextureId,
    pub opacity: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Zeroable, Pod)]
#[repr(C)]
pub struct BlockId(u32);

impl Default for BlockId {
    #[inline]
    fn default() -> Self {
        Self(u32::MAX)
    }
}
