use rand::{prelude::IndexedRandom, rng};

use super::prelude::*;
use crate::gui::move_animation::MoveAnimation;

pub type IsDrawn = bool;

#[derive(Debug)]
pub struct GuiHand {
    entity: Entity,
    tiles: Vec<GuiTile>,
    drawn_tile: Option<Entity>,
    preferred_tile: Option<Entity>,
    do_sort: bool,
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
            preferred_tile: None,
            do_sort: true,
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
        self.drawn_tile = Some(tile.entity());

        param()
            .commands
            .entity(tile.entity())
            .insert((ChildOf(self.entity), self.tf_tile(true)));
        self.tiles.push(tile);
    }

    pub fn take_tile(&mut self, m_tile: Tile, is_drawn: bool) -> GuiTile {
        let preferred_tile = self.preferred_tile.take();
        let pos = if is_drawn {
            // 牌を明示的にツモ切り
            self.tiles
                .iter()
                .position(|t| self.is_drawn_tile(t))
                .unwrap()
        } else if let Some(e_tile) = preferred_tile
            && let Some(pos) = self
                .tiles
                .iter()
                .position(|t| t.tile() == m_tile && t.entity() == e_tile)
        {
            // 手牌が見える状態かつ牌が存在し,Discard Actionの打牌として選択している場合
            pos
        } else if let Some(pos) = self
            .tiles
            .iter()
            .position(|t| t.tile() == m_tile && !self.is_drawn_tile(t))
        {
            // 手牌が見える状態かつ牌が存在する場合 (ツモ牌以外)
            pos
        } else if let Some(pos) = self
            .tiles
            .iter()
            .position(|t| t.tile() == m_tile && self.is_drawn_tile(t))
        {
            // 手牌が見える状態かつ牌が存在する場合 (ツモ牌)
            pos
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
                *pos
            } else {
                panic!("{} not found in hand", m_tile);
            }
        };

        let mut tile = self.tiles.remove(pos);

        if tile.tile() == Z8 {
            tile.mutate(m_tile);
        } else {
            assert!(tile.tile() == m_tile);
        }

        self.drawn_tile = None;

        tile
    }

    pub fn tiles(&mut self) -> &mut [GuiTile] {
        &mut self.tiles
    }

    pub fn is_drawn_tile(&self, tile: &GuiTile) -> bool {
        Some(tile.entity()) == self.drawn_tile
    }

    pub fn drawn_tile(&self) -> Option<&GuiTile> {
        let e_tile = self.drawn_tile?;
        self.tiles.iter().find(|t| t.entity() == e_tile)
    }

    pub fn set_sort(&mut self, flag: bool) {
        self.do_sort = flag;
        if self.do_sort {
            self.align();
        }
    }

    pub fn move_tile(&mut self, e_from: Entity, e_to: Entity) {
        if let Some(from) = self.tiles.iter().position(|t| t.entity() == e_from)
            && let Some(to) = self.tiles.iter().position(|t| t.entity() == e_to)
        {
            let tile = self.tiles.remove(from);
            // from < to の場合,先のremoveで位置が一つ左にずれる
            // let to = if from < to { to - 1 } else { to };
            self.tiles.insert(to, tile);
        }
        self.set_sort(false);
        self.align();
    }

    pub fn set_preferred_tile(&mut self, e_tile: Entity) {
        self.preferred_tile = Some(e_tile);
    }

    pub fn align(&mut self) {
        if self.do_sort {
            self.tiles.sort_by_key(|t| t.tile());
        }

        for (i, tile) in self.tiles.iter().enumerate() {
            param()
                .commands
                .entity(tile.entity())
                .insert(MoveAnimation::with_frame(
                    Vec3::new(
                        GuiTile::WIDTH * i as f32,
                        GuiTile::HEIGHT / 2.0,
                        GuiTile::DEPTH / 2.0,
                    ),
                    6,
                ));
        }
    }

    fn tf_tile(&self, is_drawn: bool) -> Transform {
        Transform::from_xyz(
            GuiTile::WIDTH * self.tiles.len() as f32 + if is_drawn { 0.005 } else { 0. },
            GuiTile::HEIGHT / 2.0,
            GuiTile::DEPTH / 2.0,
        )
    }
}

impl HasEntity for GuiHand {
    fn entity(&self) -> Entity {
        self.entity
    }
}
