use super::{super::util::reparent_tranform, *};

#[derive(Debug)]
pub struct GuiDiscard {
    entity: Entity,
    // 捨て牌のVec3は(0., 0., 0.)から開始し横(x)にTILE_WIDTHずつスライドしていく
    // 各行の捨て牌は6個までなのでpos_tiles.len() % 6 == 0のときは
    // xを0.にリセットしてyをTILE_HEIGHTだけマイナス方向にスライド
    // 例外的にリーチ宣言牌とその次の牌(行の先頭は例外)の場合は
    // xのスライドは(TILE_WIDHT + TILE_HEIGHT) / 2.になることに注意
    tiles: Vec<(GuiTile, Vec3)>,
    riichi_index: Option<usize>,
}

impl GuiDiscard {
    const TILES_IN_ROW: usize = 6;

    pub fn new(param: &mut StageParam, parent: Entity, seat: Seat) -> Self {
        let e_discard = param
            .commands
            .spawn((
                Name::new(format!("Discard[{seat}]")),
                ChildOf(parent),
                Transform {
                    translation: Vec3::new(-0.05, GuiTile::DEPTH / 2., 0.074),
                    rotation: Quat::from_rotation_x(-FRAC_PI_2),
                    scale: Vec3::ONE,
                },
            ))
            .id();
        Self {
            entity: e_discard,
            tiles: vec![],
            riichi_index: None,
        }
    }

    pub fn set_riichi(&mut self) {
        self.riichi_index = Some(self.tiles.len());
    }

    pub fn push_tile(&mut self, param: &mut StageParam, gui_tile: GuiTile) {
        let i_tile = self.tiles.len();
        let mut pos = if let Some((_, last_pos)) = self.tiles.last() {
            if i_tile % GuiDiscard::TILES_IN_ROW == 0 {
                let y = -GuiTile::HEIGHT * (i_tile / GuiDiscard::TILES_IN_ROW) as f32;
                Vec3::new(0., y, 0.)
            } else {
                last_pos + Vec3::new(GuiTile::WIDTH, 0., 0.)
            }
        } else {
            Vec3::default()
        };
        let mut rot = Quat::IDENTITY;

        // event.is_riichi == trueの時以外に直前のリーチ宣言牌が鳴きで他家の副露に移動した場合なども含む
        if let Some(riichi_index) = self.riichi_index {
            // リーチ宣言牌とその次の牌は位置が少しずれる
            if i_tile == riichi_index || i_tile == riichi_index + 1 {
                pos += Vec3::new((GuiTile::HEIGHT - GuiTile::WIDTH) / 2., 0., 0.);
            }
            // リーチ宣言牌を横に倒す
            if i_tile == riichi_index {
                rot = Quat::from_axis_angle(Vec3::Z, FRAC_PI_2);
            }
        }

        let mut tf = reparent_tranform(gui_tile.entity, self.entity, &param.globals);
        tf.rotation = rot;

        param
            .commands
            .entity(gui_tile.entity)
            .insert((ChildOf(self.entity), tf, MoveTo::new(pos)));
        self.tiles.push((gui_tile, pos));
    }
}
