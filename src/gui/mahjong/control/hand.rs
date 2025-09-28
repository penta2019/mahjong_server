use rand::{prelude::IndexedRandom, rng};

use super::*;

pub type IsDrawn = bool;

#[derive(Debug)]
pub struct GuiHand {
    entity: Entity,
    tiles: Vec<GuiTile>,
    drawn_tile: Option<GuiTile>,
}

impl GuiHand {
    pub fn new() -> Self {
        let entity = param()
            .commands
            .spawn((Name::new("Hand"), Transform::default()))
            .id();
        Self {
            entity,
            tiles: vec![],
            drawn_tile: None,
        }
    }

    pub fn init(&mut self, m_tiles: &[Tile]) {
        for t in m_tiles {
            let tile = GuiTile::new(*t);
            param()
                .commands
                .entity(tile.entity())
                .insert((ChildOf(self.entity), self.tf_tile(false)));
            self.tiles.push(tile);
        }
    }

    pub fn deal_tile(&mut self, m_tile: Tile) {
        let tile = GuiTile::new(m_tile);
        param()
            .commands
            .entity(tile.entity())
            .insert((ChildOf(self.entity), self.tf_tile(true)));
        self.drawn_tile = Some(tile);
    }

    pub fn take_tile(
        &mut self,
        m_tile: Tile,
        is_drawn: bool,
        preferred_tile: Option<Entity>,
    ) -> GuiTile {
        let mut tile = if is_drawn {
            // 牌を明示的にツモ切り
            self.drawn_tile.take().unwrap()
        } else if let Some(e_tile) = preferred_tile
            && let Some(pos) = self
                .tiles
                .iter()
                .position(|t| t.tile() == m_tile && t.entity() == e_tile)
        {
            // 手牌が見える状態かつ牌が存在し,Discard Actionの打牌として選択している場合
            self.tiles.remove(pos)
        } else if let Some(pos) = self.tiles.iter().position(|t| t.tile() == m_tile) {
            // 手牌が見える状態かつ牌が存在する場合
            self.tiles.remove(pos)
        } else if let Some(drawn_tile) = &self.drawn_tile
            && drawn_tile.tile() == m_tile
        {
            // ツモ牌を暗黙に手牌から取り除く場合 (加槓,暗槓など)
            self.drawn_tile.take().unwrap()
        } else {
            // 対戦モードで他家の手牌にZ8(基本的には全てZ8)が含まれる場合
            // 手牌のZ8の中からランダムに選択
            let pos_candidates = self
                .tiles
                .iter()
                .enumerate()
                .filter(|(_, t)| t.tile() == Z8)
                .map(|(i, _)| i)
                .collect::<Vec<usize>>();
            if let Some(pos) = pos_candidates.choose(&mut rng()) {
                self.tiles.remove(*pos)
            } else {
                panic!("{} not found in hand", m_tile);
            }
        };

        if tile.tile() == Z8 {
            tile.mutate(m_tile);
        } else {
            assert!(tile.tile() == m_tile);
        }

        if let Some(drawn_tile) = self.drawn_tile.take() {
            self.tiles.push(drawn_tile);
        }

        tile
    }

    pub fn find_tile_from_entity(&self, e_tile: Entity) -> Option<(&GuiTile, IsDrawn)> {
        for tile in &self.tiles {
            if e_tile == tile.entity() {
                return Some((tile, false));
            }
        }
        if let Some(tile) = &self.drawn_tile
            && tile.entity() == e_tile
        {
            return Some((tile, true));
        }
        None
    }

    pub fn align(&mut self) {
        self.tiles.sort_by_key(|t| t.tile());
        for (i, tile) in self.tiles.iter().enumerate() {
            param()
                .commands
                .entity(tile.entity())
                .insert(MoveAnimation::new(Vec3::new(
                    GuiTile::WIDTH * i as f32,
                    GuiTile::HEIGHT / 2.,
                    GuiTile::DEPTH / 2.,
                )));
        }
    }

    fn tf_tile(&self, is_drawn: bool) -> Transform {
        Transform::from_xyz(
            GuiTile::WIDTH * self.tiles.len() as f32 + if is_drawn { 0.005 } else { 0. },
            GuiTile::HEIGHT / 2.,
            GuiTile::DEPTH / 2.,
        )
    }
}

impl HasEntity for GuiHand {
    fn entity(&self) -> Entity {
        self.entity
    }
}
