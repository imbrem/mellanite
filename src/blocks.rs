use std::collections::BTreeSet;

use bevy::{
    asset::HandleId,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bytemuck::{Pod, Zeroable};
use slab::Slab;

#[derive(Resource, Default)]
pub struct Blocks {
    texture_materials: Vec<Handle<StandardMaterial>>,
    blocks: Slab<BlockData>,
    textures: BTreeSet<(HandleId, BlockId)>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct BlockData {
    texture: Option<Handle<Image>>,
}

impl Blocks {
    #[inline]
    pub fn new_block(
        &mut self,
        images: &mut Assets<Image>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Result<BlockId, ()> {
        let ix = self.blocks.insert(BlockData { texture: None });
        if ix >= u32::MAX as usize {
            Err(())
        } else {
            let id = BlockId(ix as u32);
            while self.texture_materials.len() <= id.sheet().0 as usize {
                let image = Image::new_fill(
                    Extent3d {
                        width: 16 * 256,
                        height: 16 * 256,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    &[0, 0, 255, 255],
                    TextureFormat::Rgba8UnormSrgb,
                );
                let image_id = images.add(image);
                let material = StandardMaterial {
                    base_color_texture: Some(image_id.clone()), //TODO: allow null images later...
                    perceptual_roughness: 1.0,
                    reflectance: 0.0,
                    unlit: false,
                    ..default()
                };
                let mat_id = materials.add(material);
                self.texture_materials.push(mat_id)
            }
            Ok(id)
        }
    }

    #[inline]
    pub fn set_block_texture(
        &mut self,
        block: BlockId,
        texture: Handle<Image>,
        images: &mut Assets<Image>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Result<Option<Handle<Image>>, Handle<Image>> {
        if let Some(data) = self.blocks.get_mut(block.0 as usize) {
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
                        &mut self.texture_materials,
                        block,
                        &data.convert(TextureFormat::Rgba8UnormSrgb).unwrap(),
                        images,
                        materials,
                    )
                }
            }
            Ok(data.texture.replace(texture))
        } else {
            Err(texture)
        }
    }

    #[inline]
    pub fn get_sheet_material(&self, sheet: SheetId) -> Handle<StandardMaterial> {
        self.texture_materials[sheet.0 as usize].clone()
    }

    #[inline]
    fn blit_texture_inner(
        texture_sheets: &mut [Handle<StandardMaterial>],
        block: BlockId,
        texture: &Image,
        images: &mut Assets<Image>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        let sheet = &mut texture_sheets[block.sheet().0 as usize];
        let target_mat = materials.get_mut(sheet).unwrap();
        let target_data = images
            .get_mut(target_mat.base_color_texture.as_ref().unwrap())
            .unwrap();
        let coords = block.coords();
        let start = coords.x_ix() as usize * 4 * 16 + coords.y_ix() as usize * 4 * 16 * 256;
        for y in 0..16 {
            for x in 0..16 {
                for c in 0..4 {
                    target_data.data[start + c + x * 4 + y * 4 * 16 * 256] =
                        texture.data[c + x * 4 + y * 4 * 16]
                }
            }
        }
        // for data in target_data.data.iter_mut() {
        //     *data = 50;
        // }
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
pub struct BlockId(pub u32);

impl BlockId {
    /// Get this block's associated texture sheet
    #[inline]
    pub fn sheet(&self) -> SheetId {
        SheetId((self.0 >> 16) as u16)
    }

    /// Get this block's coordinates in the texture sheet
    #[inline]
    pub fn coords(&self) -> SheetCoords {
        SheetCoords(self.0 as u16)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable)]
#[repr(transparent)]
pub struct SheetCoords(pub u16);

impl SheetCoords {
    pub fn x_ix(&self) -> u8 {
        self.0 as u8
    }

    pub fn y_ix(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn top_left(&self) -> [f32; 2] {
        [self.x_ix() as f32 / 256.0, self.y_ix() as f32 / 256.0]
    }

    pub fn top_right(&self) -> [f32; 2] {
        [
            (self.x_ix() as f32 + 1.0) / 256.0,
            self.y_ix() as f32 / 256.0,
        ]
    }

    pub fn bottom_left(&self) -> [f32; 2] {
        [
            self.x_ix() as f32 / 256.0,
            (self.y_ix() as f32 + 1.0) / 256.0,
        ]
    }

    pub fn bottom_right(&self) -> [f32; 2] {
        [
            (self.x_ix() as f32 + 1.0) / 256.0,
            (self.y_ix() as f32 + 1.0) / 256.0,
        ]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable)]
#[repr(transparent)]
pub struct SheetId(pub u16);

pub fn blit_loaded_textures(
    mut events: EventReader<AssetEvent<Image>>,
    mut blocks: ResMut<Blocks>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let id = handle.id();
                let blocks = &mut *blocks;
                for (lowest_id, block) in blocks
                    .textures
                    .range((id, BlockId(0))..=(id, BlockId(u32::MAX)))
                {
                    debug_assert_eq!(id, *lowest_id);
                    if let Some(data) = images.get(handle) {
                        Blocks::blit_texture_inner(
                            &mut blocks.texture_materials,
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
                    .range((id, BlockId(0))..=(id, BlockId(u32::MAX)))
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
