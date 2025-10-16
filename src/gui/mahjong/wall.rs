use std::collections::VecDeque;

use super::prelude::*;

// 牌山の基準位置 (一番右上の牌の中心座標)
const TF_WALL: Transform = Transform::from_xyz(0.177, GuiTile::DEPTH * 1.5, 0.174);
const WIDTH: usize = 17; // 牌山の横幅(枚)
const HEIGHT: usize = 2; // 牌山の高さ(枚)

// 牌山のそれぞれの牌とその管理情報
#[derive(Debug)]
struct Entry {
    tile: GuiTile,      // Tileの実態
    tf: Transform,      // 牌山非表示(通常状態)の配置
    tf_show: Transform, // 牌山表示時の配置
}

#[derive(Debug)]
pub struct Wall {
    entity: Entity,
    tiles: VecDeque<Entry>,        // ツモ山
    replacements: VecDeque<Entry>, // 嶺上牌
    doras: Vec<Entry>,             // ドラ表示牌
    ura_doras: Vec<Entry>,         // 裏ドラ
    dora_count: usize,             // ドラ表示牌の枚数
}

impl Wall {
    pub fn new() -> Self {
        let entity = cmd()
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

    pub fn init(&mut self, event: &EventNew) {
        let p = param();
        let cmd = &mut p.cmd;

        // 起家の一番左上の牌を先頭に時計回りに牌を積む
        for s in 0..SEAT {
            let wall = cmd
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
                    let tf_show = Transform {
                        translation: Vec3::new(
                            -GuiTile::WIDTH * w as f32 + GuiTile::HEIGHT * 0.5,
                            -GuiTile::DEPTH,
                            GuiTile::HEIGHT * (h as f32 - 0.5),
                        ),
                        rotation: Quat::from_rotation_x(-FRAC_PI_2),
                        scale: Vec3::ONE,
                    };
                    tile.insert((ChildOf(wall), tf_show));
                    self.tiles.push_back(Entry {
                        tile,
                        tf,
                        tf_show,
                        // wall_seat: s,
                    });
                }
            }
        }

        // 基準位置を親の牌山の右上の牌に移動
        let wall_seat = (event.dealer + event.dice - 1) % SEAT;
        self.tiles.rotate_right(WIDTH * HEIGHT * wall_seat);
        // サイコロの目の分基準位置を移動
        self.tiles.rotate_left(event.dice * HEIGHT);

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
            if let Some(entry) = self.tiles.pop_front() {
                entry.tile.despawn();
                // tileはここでDropされる
            }
        }

        // 牌山の中身がわかる場合(牌譜,観戦,デバッグ等)はテクスチャを張り替える
        mutate_tiles(self.tiles.iter_mut(), &event.wall);
        mutate_tiles(self.doras.iter_mut(), &event.dora_wall);
        mutate_tiles(self.ura_doras.iter_mut(), &event.ura_dora_wall);
        mutate_tiles(self.replacements.iter_mut(), &event.replacement_wall);
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
        entry.tile.insert(entry.tf_show);
        self.dora_count += 1;
    }
}

impl HasEntity for Wall {
    fn entity(&self) -> Entity {
        self.entity
    }
}

fn mutate_tiles<'a, T: Iterator<Item = &'a mut Entry>>(target: T, source: &[Tile]) {
    for (entry, tile) in target.zip(source.iter()) {
        entry.tile.mutate(*tile);
    }
}
