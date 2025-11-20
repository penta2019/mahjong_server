use super::super::prelude::*;
use crate::move_animation::MoveAnimation;

// 一番右の牌の右端が基準位置
#[derive(Debug)]
pub struct GuiMeld {
    entity: Entity,
    items: Vec<GuiMeldItem>,
    item_ofsset_x: f32,
}
crate::impl_has_entity!(GuiMeld);

impl GuiMeld {
    pub fn new() -> Self {
        let entity = cmd().spawn((Name::new("Meld"), Transform::IDENTITY)).id();
        Self {
            entity,
            items: vec![],
            item_ofsset_x: 0.0,
        }
    }

    pub fn width(&self) -> f32 {
        -self.item_ofsset_x
    }

    pub fn meld(
        &mut self,
        self_tiles: Vec<GuiTile>,
        meld_tile: Option<(GuiTile, usize)>,
        animate: bool,
    ) {
        let mut meld_item_width = GuiTile::WIDTH * self_tiles.len() as f32;
        if meld_tile.is_some() {
            meld_item_width += GuiTile::HEIGHT;
        }

        let mut tiles = self_tiles;
        let mut meld_index = None;
        let mut tfs = [Transform::IDENTITY; 4];
        if let Some((meld_tile, meld_offset)) = meld_tile {
            meld_index = Some(3 - meld_offset);
            if tiles.len() == 3 && meld_offset == 1 {
                // 下家から大明槓する場合は鳴いた牌を一番右に移動
                meld_index = Some(3);
            }
            tiles.insert(meld_index.unwrap(), meld_tile);
        } else {
            match tiles.len() {
                1 => {
                    // 加槓: すでに存在するGuiMeldItemに牌を加えるので例外的な処理が必要
                    self.meld_kakan(tiles.pop().unwrap(), animate);
                    return;
                }
                4 => {
                    // 暗槓
                    // 両端の牌を裏返す
                    for i in [0, 3] {
                        tfs[i].rotation = Quat::from_rotation_x(PI);
                    }
                }
                _ => panic!("invalid meld"),
            }
        }

        let mut meld_item = GuiMeldItem::new();
        self.item_ofsset_x -= meld_item_width + GuiTile::WIDTH * 0.25;
        meld_item.insert((
            ChildOf(self.entity),
            Transform::from_xyz(self.item_ofsset_x, 0.0, 0.0),
        ));

        if let Some(i) = meld_index {
            tfs[i].rotation = Quat::from_rotation_z(if i < 2 {
                // 対面,上家からの鳴きは左向き
                FRAC_PI_2
            } else {
                // 下家からの鳴きは右向き
                -FRAC_PI_2
            });
        }

        let mut offset_x = 0.0; // 次の牌の基準位置
        for (i, tile) in tiles.iter().enumerate() {
            let mut move_to = Vec3::new(offset_x, 0.0, 0.0);
            if Some(i) == meld_index {
                let frac_diff_2 = (GuiTile::HEIGHT - GuiTile::WIDTH) / 2.0;
                move_to.x += frac_diff_2;
                move_to.y -= frac_diff_2;
                offset_x += GuiTile::HEIGHT;
            } else {
                offset_x += GuiTile::WIDTH;
            }

            if animate {
                tfs[i].translation = tile.transform_from(self.entity).translation;
                tile.insert((
                    ChildOf(meld_item.entity()),
                    tfs[i],
                    MoveAnimation::new(move_to),
                ));
            } else {
                tfs[i].translation = move_to;
                tile.insert((ChildOf(meld_item.entity()), tfs[i]));
            }
        }

        meld_item.tiles = tiles;
        meld_item.meld_index = meld_index;
        self.items.push(meld_item);
    }

    fn meld_kakan(&mut self, tile: GuiTile, animate: bool) {
        let normal = tile.tile().to_normal();
        let meld_item = self
            .items
            .iter_mut()
            .find(|item| item.tiles.iter().all(|t| t.tile().to_normal() == normal))
            .unwrap();
        let meld_index = meld_item.meld_index.unwrap();

        // 加槓と対象となるポンをどこから鳴いているかで向きを決定
        let e_meld_item = meld_item.entity();
        let mut tf = Transform::IDENTITY;
        tf.rotation = Quat::from_rotation_z(if meld_index == 2 {
            // 下家からなら右向き
            -FRAC_PI_2
        } else {
            // 対面,上家からなら左向き
            FRAC_PI_2
        });

        let frac_diff_2 = (GuiTile::HEIGHT - GuiTile::WIDTH) / 2.0;
        let move_to = Vec3::new(
            GuiTile::WIDTH * meld_index as f32 + frac_diff_2,
            GuiTile::WIDTH - frac_diff_2,
            0.0,
        );

        if animate {
            tf.translation = tile.transform_from(e_meld_item).translation;
            tile.insert((ChildOf(e_meld_item), tf, MoveAnimation::new(move_to)));
        } else {
            tf.translation = move_to;
            tile.insert((ChildOf(e_meld_item), tf));
        }

        meld_item.tiles.push(tile);
    }
}

#[derive(Debug)]
pub struct GuiMeldItem {
    entity: Entity,
    tiles: Vec<GuiTile>,
    meld_index: Option<usize>,
}
crate::impl_has_entity!(GuiMeldItem);

impl GuiMeldItem {
    pub fn new() -> Self {
        let entity = cmd().spawn(Name::new("MeldItem")).id();
        Self {
            entity,
            tiles: vec![],
            meld_index: None,
        }
    }
}
