use mahjong_core::rand::prelude::*;

use super::super::prelude::*;
use crate::move_animation::MoveAnimation;

pub type IsDrawn = bool;

#[derive(Debug)]
pub struct GuiHand {
    entity: Entity,
    tiles: Vec<GuiTile>, // 一番左の牌の下側中心奥が基準位置
    drawn_tile: Option<Entity>,
    preferred_tile: Option<Entity>,
    do_sort: bool,
}
crate::impl_has_entity!(GuiHand);

impl GuiHand {
    pub fn new() -> Self {
        let entity = cmd().spawn((Name::new("Hand"), Transform::default())).id();
        Self {
            entity,
            tiles: vec![],
            drawn_tile: None,
            preferred_tile: None,
            do_sort: true,
        }
    }

    // 一番左の牌の下側左奥が基準位置から右端までの長さ
    pub fn width(&self) -> f32 {
        GuiTile::WIDTH
            * (self.tiles.len() as f32 + if self.drawn_tile.is_some() { 0.5 } else { 0.0 })
    }

    pub fn init(&mut self, tiles: Vec<GuiTile>) {
        for tile in &tiles {
            tile.insert((ChildOf(self.entity), self.tf_new_tile(false)));
        }
        self.tiles = tiles;
    }

    pub fn deal_tile(&mut self, tile: GuiTile, animate: bool) {
        self.drawn_tile = Some(tile.entity());

        if animate {
            let mut tf_from = tile.transform_from(self.entity);
            let tf_to = self.tf_new_tile(true);
            tf_from.rotation = tf_to.rotation;
            tile.insert((
                ChildOf(self.entity),
                tf_from,
                MoveAnimation::new(tf_to.translation),
            ));
        } else {
            let tf_to = self.tf_new_tile(true);
            tile.insert((ChildOf(self.entity), tf_to));
        }

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
            if let Some(pos) = pos_candidates.choose(&mut ThreadRng::default()) {
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
            self.align(true);
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
        self.align(true);
    }

    pub fn set_preferred_tile(&mut self, e_tile: Entity) {
        self.preferred_tile = Some(e_tile);
    }

    pub fn align(&mut self, animate: bool) {
        if self.do_sort {
            self.tiles.sort_by_key(|t| t.tile());
        }

        for (i, tile) in self.tiles.iter().enumerate() {
            let pos = Vec3::new(
                GuiTile::WIDTH * i as f32,
                GuiTile::HEIGHT / 2.0,
                GuiTile::DEPTH / 2.0,
            );
            if animate {
                tile.insert(MoveAnimation::new(pos).with_frame(6));
            } else {
                tile.insert(Transform::from_translation(pos));
            }
        }
    }

    fn tf_new_tile(&self, is_drawn: bool) -> Transform {
        Transform::from_xyz(
            GuiTile::WIDTH * self.tiles.len() as f32 + if is_drawn { 0.005 } else { 0. },
            GuiTile::HEIGHT / 2.0,
            GuiTile::DEPTH / 2.0,
        )
    }
}
