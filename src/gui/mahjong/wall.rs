use std::collections::VecDeque;

use super::prelude::*;

// 牌山の基準位置 (一番右上の牌の中心座標)
const TF_WALL: Transform = Transform::from_xyz(0.177, GuiTile::DEPTH * 1.5, 0.174);
const WIDTH: usize = 17; // 牌山の横幅(枚)
const HEIGHT: usize = 2; // 牌山の高さ(枚)

#[derive(Debug)]
struct TileEntry {
    tile: GuiTile,      // Tileの実態
    tf: Transform,      // 牌山非表示(通常状態)の配置
    tf_show: Transform, // 牌山表示時の配置
    wall_seat: Seat,    // 牌が属している牌山の座席
}

#[derive(Debug)]
pub struct Wall {
    entity: Entity,
    tiles: VecDeque<TileEntry>,        // ツモ山
    replacements: VecDeque<TileEntry>, // 嶺上牌
    doras: Vec<TileEntry>,             // ドラ表示牌
    ura_doras: Vec<TileEntry>,         // 裏ドラ
    dora_count: usize,                 // ドラ表示牌の枚数
}

impl Wall {
    pub fn new() -> Self {
        let entity = param()
            .commands
            .spawn((Name::new("Wall".to_string()), Transform::IDENTITY))
            .id();

        Self {
            entity,
            tiles: VecDeque::with_capacity(WIDTH * HEIGHT * SEAT), // (=136)
            replacements: VecDeque::new(),
            doras: vec![],
            ura_doras: vec![],
            dora_count: 0,
        }
    }

    pub fn init(&mut self, dealer: Seat, dice: usize) {
        let param = param();

        // 起家の一番左上の牌を先頭に時計回りに牌を積む
        for s in 0..SEAT {
            let wall = param
                .commands
                .spawn((
                    ChildOf(self.entity),
                    Transform::from_rotation(Quat::from_rotation_y(-FRAC_PI_2 * s as f32))
                        * TF_WALL,
                ))
                .id();

            for w in 0..WIDTH {
                for h in 0..HEIGHT {
                    let tile = GuiTile::new(Z8);
                    let tf = Transform {
                        translation: Vec3::new(
                            -GuiTile::WIDTH * w as f32,
                            -GuiTile::DEPTH * h as f32,
                            0.0,
                        ),
                        rotation: Quat::from_rotation_x(FRAC_PI_2),
                        scale: Vec3::ONE,
                    };
                    param
                        .commands
                        .entity(tile.entity())
                        .insert((ChildOf(wall), tf));
                    self.tiles.push_back(TileEntry {
                        tile,
                        tf,
                        tf_show: Transform::IDENTITY,
                        wall_seat: s,
                    });
                }
            }
        }

        // 基準位置を親の牌山の右上の牌に移動
        let wall_seat = (dealer + dice - 1) % SEAT;
        self.tiles.rotate_right(WIDTH * HEIGHT * wall_seat);
        // サイコロの目の分基準位置を移動
        self.tiles.rotate_left(dice * HEIGHT);

        // 嶺上牌
        let tile_1 = self.tiles.pop_back().unwrap();
        let tile_0 = self.tiles.pop_back().unwrap();
        let tile_3 = self.tiles.pop_back().unwrap();
        let tile_2 = self.tiles.pop_back().unwrap();
        self.replacements = VecDeque::from_iter([tile_0, tile_1, tile_2, tile_3]);

        // ドラ表示牌, 裏ドラ
        let mut doras = vec![];
        let mut ura_doras = vec![];
        for _ in 0..5 {
            ura_doras.push(self.tiles.pop_back().unwrap());
            doras.push(self.tiles.pop_back().unwrap());
        }
        self.doras = doras;
        self.ura_doras = ura_doras;

        // TODO
        for _ in 0..13 * 4 {
            if let Some(tile) = self.tiles.pop_front() {
                param.commands.entity(tile.tile.entity()).despawn();
                // tileはここでDropされる
            }
        }
    }

    pub fn take_tile(&mut self, is_replacement: bool) -> GuiTile {
        let entry = if is_replacement {
            self.replacements.pop_front()
        } else {
            self.tiles.pop_front()
        };
        entry.unwrap().tile
    }

    pub fn add_dora(&mut self, m_tile: Tile) {
        let entry = &mut self.doras[self.dora_count];
        entry.tile.mutate(m_tile);
        entry.tf = entry.tf * Transform::from_rotation(Quat::from_rotation_x(PI));
        param()
            .commands
            .entity(entry.tile.entity())
            .insert(entry.tf);
        self.dora_count += 1;
    }
}

impl HasEntity for Wall {
    fn entity(&self) -> Entity {
        self.entity
    }
}
