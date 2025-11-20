use super::super::prelude::*;
use crate::move_animation::MoveAnimation;

#[derive(Debug)]
pub struct GuiDiscard {
    entity: Entity,
    // 捨て牌のVec3は(0.0, 0.0, 0.0)から開始し横(x)にTILE_WIDTHずつスライドしていく
    // 各行の捨て牌は6個までなのでpos_tiles.len() % 6 == 0のときは
    // xを0.にリセットしてyをTILE_HEIGHTだけマイナス方向にスライド
    // 例外的にリーチ宣言牌とその次の牌(行の先頭は例外)の場合は
    // xのスライドは(TILE_WIDHT + TILE_HEIGHT) / 2.になることに注意
    tiles: Vec<(GuiTile, Vec3)>,
    riichi_index: Option<usize>,
}
crate::impl_has_entity!(GuiDiscard);

impl GuiDiscard {
    const TILES_IN_ROW: usize = 6;

    pub fn new() -> Self {
        let entity = cmd()
            .spawn((Name::new("Discard"), Transform::IDENTITY))
            .id();
        Self {
            entity,
            tiles: vec![],
            riichi_index: None,
        }
    }

    pub fn set_riichi(&mut self) {
        assert!(self.riichi_index.is_none());
        self.riichi_index = Some(self.tiles.len());
    }

    pub fn push_tile(&mut self, tile: GuiTile) {
        let i_tile = self.tiles.len();
        let mut pos = if let Some((_, last_pos)) = self.tiles.last() {
            if i_tile.is_multiple_of(GuiDiscard::TILES_IN_ROW) {
                let y = -GuiTile::HEIGHT * (i_tile / GuiDiscard::TILES_IN_ROW) as f32;
                Vec3::new(0.0, y, 0.0)
            } else {
                last_pos + Vec3::new(GuiTile::WIDTH, 0.0, 0.0)
            }
        } else {
            Vec3::ZERO
        };
        let mut rot = Quat::IDENTITY;

        // event.is_riichi == trueの時以外に直前のリーチ宣言牌が鳴きで他家の副露に移動した場合なども含む
        if let Some(riichi_index) = self.riichi_index {
            // リーチ宣言牌とその次の牌は位置が少しずれる (リーチ宣言牌の次の打牌で行が変わるタイミングは除く)
            if i_tile == riichi_index
                || (i_tile == riichi_index + 1 && !i_tile.is_multiple_of(GuiDiscard::TILES_IN_ROW))
            {
                pos += Vec3::new((GuiTile::HEIGHT - GuiTile::WIDTH) / 2.0, 0.0, 0.0);
            }
            // リーチ宣言牌を横に倒す
            if i_tile == riichi_index {
                rot = Quat::from_rotation_z(FRAC_PI_2);
            }
        }

        let mut tf = tile.transform_from(self.entity);
        tf.rotation = rot;

        // 捨て牌が通る(鳴きやロンが入らない)まで少しずらしておく
        let move_to = pos + Vec3::new(GuiTile::WIDTH / 2.0, -GuiTile::WIDTH / 4.0, 0.0);
        tile.insert((ChildOf(self.entity), tf, MoveAnimation::new(move_to)));
        self.tiles.push((tile, pos));
    }

    pub fn confirm_last_tile(&mut self) {
        if let Some((tile, pos)) = self.tiles.last().as_ref() {
            tile.insert(MoveAnimation::new(*pos));
        }
    }

    pub fn take_last_tile(&mut self) -> GuiTile {
        self.tiles.pop().unwrap().0
    }
}
