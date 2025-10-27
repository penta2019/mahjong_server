use super::super::{
    prelude::*,
    tile_plugin::{TileBlend, TileMutate, create_tile},
};

#[derive(Debug)]
pub struct GuiTile {
    entity: Entity,
    tile: Tile,
}
crate::impl_has_entity!(GuiTile);

impl GuiTile {
    pub const WIDTH: f32 = 0.020;
    pub const HEIGHT: f32 = 0.0256;
    pub const DEPTH: f32 = 0.016;

    pub const NORMAL: LinearRgba = LinearRgba::new(0.0, 0.0, 0.0, 0.0); // ハイライトなし
    pub const ACTIVE: LinearRgba = LinearRgba::new(0.0, 1.0, 0.0, 0.15); // ハイライト (打牌可)
    pub const INACTIVE: LinearRgba = LinearRgba::new(0.0, 0.0, 0.0, 0.4); // ハイライト (打牌不可)

    pub fn new(tile: Tile) -> Self {
        let p = param();
        let entity = create_tile(&mut p.cmd, &p.asset_server, tile);
        Self { entity, tile }
    }

    pub fn tile(&self) -> Tile {
        self.tile
    }

    pub fn mutate(&mut self, m_tile: Tile) {
        self.tile = m_tile;
        self.insert(TileMutate(m_tile));
    }

    pub fn blend(&mut self, color: LinearRgba) {
        self.insert(TileBlend(color));
    }
}
