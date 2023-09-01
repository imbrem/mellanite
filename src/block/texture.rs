use std::collections::BTreeSet;

use bevy::{
    asset::HandleId,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bytemuck::{Pod, Zeroable};
use slab::Slab;

const TEXTURE_HEIGHT: usize = 16;
const LOG_SHEET_SIZE: u32 = 8;
const SHEET_HEIGHT: usize = 1 << LOG_SHEET_SIZE;
const SHEET_SIZE: usize = SHEET_HEIGHT * SHEET_HEIGHT;

#[derive(Resource, Default)]
pub struct BlockMaterials {
    sheet_materials: Vec<Handle<StandardMaterial>>,
    blocks: Vec<Vec<BlockData>>,
    materials: Slab<MaterialData>,
    textures: BTreeSet<(HandleId, BlockTextureId)>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct MaterialData {
    template: Handle<StandardMaterial>,
    curr_texture: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct BlockData {
    texture: Option<Handle<Image>>,
    material: BlockMaterialId,
}

impl BlockMaterials {
    #[inline]
    pub fn new_material(
        &mut self,
        template: Handle<StandardMaterial>,
    ) -> Result<BlockMaterialId, ()> {
        if self.materials.len() >= u16::MAX as usize {
            return Err(());
        }
        let ix = self.materials.insert(MaterialData {
            template,
            curr_texture: usize::MAX,
        });
        Ok(BlockMaterialId(ix as u16))
    }

    #[inline]
    pub fn new_texture(
        &mut self,
        material: BlockMaterialId,
        materials: &mut Assets<StandardMaterial>,
    ) -> Result<BlockTextureId, ()> {
        let curr_material = self.materials.get_mut(material.0 as usize).ok_or(())?;
        let l = self.blocks.len();
        let blocks = match self.blocks.get_mut(curr_material.curr_texture) {
            Some(blocks) if blocks.len() <= SHEET_SIZE => blocks,
            _ if l < u32::MAX as usize / SHEET_SIZE => {
                curr_material.curr_texture = l;
                self.blocks.push(Vec::new());
                let template = materials
                    .get_mut(&curr_material.template)
                    .ok_or(())?
                    .clone();
                self.sheet_materials.push(materials.add(template));
                self.blocks.last_mut().unwrap()
            }
            _ => return Err(()),
        };
        let block_ix = blocks.len();
        blocks.push(BlockData {
            texture: None,
            material,
        });
        let texture_ix = curr_material.curr_texture;
        let id = BlockTextureId((texture_ix * SHEET_HEIGHT * SHEET_HEIGHT + block_ix) as u32);
        Ok(id)
    }

    #[inline]
    pub fn set_block_texture(
        &mut self,
        block: BlockTextureId,
        texture: Handle<Image>,
        images: &mut Assets<Image>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Result<Option<Handle<Image>>, ()> {
        let data = self
            .blocks
            .get_mut(block.sheet().0 as usize)
            .ok_or(())?
            .get_mut(block.coords().0 as usize)
            .ok_or(())?;
        'precomp: {
            if let Some(old_texture) = &data.texture {
                if old_texture == &texture {
                    break 'precomp;
                }
                self.textures.remove(&(old_texture.id(), block));
            }

            self.textures.insert((texture.id(), block));
            if let Some(data) = images.get(&texture) {
                Self::blit_texture_inner(
                    &mut self.sheet_materials,
                    block,
                    &data.convert(TextureFormat::Rgba8UnormSrgb).unwrap(),
                    images,
                    materials,
                )
            }
        }
        Ok(data.texture.replace(texture))
    }

    #[inline]
    pub fn get_sheet_material(&self, sheet: SheetId) -> Handle<StandardMaterial> {
        self.sheet_materials[sheet.0 as usize].clone()
    }

    #[inline]
    fn blit_texture_inner(
        texture_sheets: &mut [Handle<StandardMaterial>],
        block: BlockTextureId,
        texture: &Image,
        images: &mut Assets<Image>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        let sheet = &mut texture_sheets[block.sheet().0 as usize];
        let target_mat = materials.get_mut(sheet).unwrap();
        let base_color_texture = target_mat.base_color_texture.get_or_insert_with(|| {
            let image = Image::new_fill(
                Extent3d {
                    width: (TEXTURE_HEIGHT * SHEET_HEIGHT) as u32,
                    height: (TEXTURE_HEIGHT * SHEET_HEIGHT) as u32,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &[0, 0, 0, 255],
                TextureFormat::Rgba8UnormSrgb,
            );
            images.add(image)
        });
        let target_data = images.get_mut(base_color_texture).unwrap();
        let coords = block.coords();
        let start = coords.x_ix() as usize * 4 * TEXTURE_HEIGHT
            + coords.y_ix() as usize * 4 * TEXTURE_HEIGHT * SHEET_HEIGHT;
        for y in 0..TEXTURE_HEIGHT {
            for x in 0..TEXTURE_HEIGHT {
                for c in 0..4 {
                    target_data.data[start + c + x * 4 + y * 4 * TEXTURE_HEIGHT * SHEET_HEIGHT] =
                        texture.data[c + x * 4 + y * 4 * TEXTURE_HEIGHT]
                }
            }
        }
    }

    // #[inline]
    // pub fn blit_texture(&mut self, block: BlockId, texture: &Image, images: &mut Assets<Image>) {
    //     if self.blocks.contains(block.0 as usize) {
    //         Self::blit_texture_inner(
    //             &mut self.texture_sheets,
    //             block,
    //             &texture.convert(TextureFormat::Rgba8Unorm).unwrap(),
    //             images,
    //         )
    //     }
    // }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable)]
#[repr(transparent)]
pub struct BlockTextureId(u32);

impl BlockTextureId {
    /// Get this block's associated texture sheet
    #[inline]
    pub fn sheet(&self) -> SheetId {
        SheetId((self.0 / (SHEET_HEIGHT * SHEET_HEIGHT) as u32) as u16)
    }

    /// Get this block's coordinates in the texture sheet
    #[inline]
    pub fn coords(&self) -> SheetCoords {
        SheetCoords((self.0 % (SHEET_HEIGHT * SHEET_HEIGHT) as u32) as u16)
    }
}

impl Default for BlockTextureId {
    fn default() -> Self {
        Self(u32::MAX)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable)]
#[repr(transparent)]
pub struct SheetCoords(u16);

pub const BORDER_WIDTH: f32 = 0.1 / (TEXTURE_HEIGHT * SHEET_HEIGHT) as f32;

impl SheetCoords {
    pub fn x_ix(&self) -> u8 {
        (self.0 % SHEET_HEIGHT as u16) as u8
    }

    pub fn y_ix(&self) -> u8 {
        (self.0 / SHEET_HEIGHT as u16) as u8
    }

    pub fn top_left(&self) -> [f32; 2] {
        [
            self.x_ix() as f32 / (SHEET_HEIGHT as f32) + BORDER_WIDTH,
            self.y_ix() as f32 / (SHEET_HEIGHT as f32) + BORDER_WIDTH,
        ]
    }

    pub fn top_right(&self) -> [f32; 2] {
        [
            (self.x_ix() as f32 + 1.0) / (SHEET_HEIGHT as f32) - BORDER_WIDTH,
            self.y_ix() as f32 / (SHEET_HEIGHT as f32) + BORDER_WIDTH,
        ]
    }

    pub fn bottom_left(&self) -> [f32; 2] {
        [
            self.x_ix() as f32 / (SHEET_HEIGHT as f32) + BORDER_WIDTH,
            (self.y_ix() as f32 + 1.0) / (SHEET_HEIGHT as f32) - BORDER_WIDTH,
        ]
    }

    pub fn bottom_right(&self) -> [f32; 2] {
        [
            (self.x_ix() as f32 + 1.0) / (SHEET_HEIGHT as f32) - BORDER_WIDTH,
            (self.y_ix() as f32 + 1.0) / (SHEET_HEIGHT as f32) - BORDER_WIDTH,
        ]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable)]
#[repr(transparent)]
pub struct BlockMaterialId(u16);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable)]
#[repr(transparent)]
pub struct SheetId(u16);

pub fn blit_loaded_textures(
    mut events: EventReader<AssetEvent<Image>>,
    mut blocks: ResMut<BlockMaterials>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let id: HandleId = handle.id();
                let blocks = &mut *blocks;
                for (lowest_id, block) in blocks
                    .textures
                    .range((id, BlockTextureId(0))..=(id, BlockTextureId(u32::MAX)))
                {
                    debug_assert_eq!(id, *lowest_id);
                    if let Some(data) = images.get(handle) {
                        BlockMaterials::blit_texture_inner(
                            &mut blocks.sheet_materials,
                            *block,
                            &data.convert(TextureFormat::Rgba8UnormSrgb).unwrap(),
                            &mut images,
                            &mut materials,
                        )
                    }
                }
            }
            AssetEvent::Removed { handle } => {
                //TODO: optimize
                let id = handle.id();
                while let Some(&(lowest_id, block)) = blocks
                    .textures
                    .range((id, BlockTextureId(0))..=(id, BlockTextureId(u32::MAX)))
                    .next()
                {
                    debug_assert_eq!(id, lowest_id);
                    let removed = blocks.textures.remove(&(lowest_id, block));
                    debug_assert!(removed);
                }
            }
        }
    }
}
