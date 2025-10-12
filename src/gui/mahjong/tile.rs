use super::{
    prelude::*,
    tile_plugin::{TileControl, create_tile},
};

pub const TILE_NORMAL: LinearRgba = LinearRgba::new(0.0, 0.0, 0.0, 0.0); // ハイライトなし
pub const TILE_ACTIVE: LinearRgba = LinearRgba::new(0.0, 1.0, 0.0, 0.15); // ハイライト (打牌可)
pub const TILE_INACTIVE: LinearRgba = LinearRgba::new(0.0, 0.0, 0.0, 0.4); // ハイライト (打牌不可)

#[derive(Debug)]
pub struct GuiTile {
    entity: Entity,
    tile: Tile,
}

impl GuiTile {
    pub const WIDTH: f32 = 0.020;
    pub const HEIGHT: f32 = 0.028;
    pub const DEPTH: f32 = 0.016;

    pub fn new(tile: Tile) -> Self {
        let param = param();
        let entity = create_tile(&mut param.commands, &param.asset_server, tile);
        Self { entity, tile }
    }

    pub fn tile(&self) -> Tile {
        self.tile
    }

    pub fn mutate(&mut self, m_tile: Tile) {
        self.tile = m_tile;
        self.tile_control().mutate(m_tile);
    }

    pub fn blend(&mut self, color: LinearRgba) {
        self.tile_control().blend(color);
    }

    fn tile_control(&self) -> Mut<'_, TileControl> {
        param().tile_controls.get_mut(self.entity).unwrap()
    }
}

impl HasEntity for GuiTile {
    fn entity(&self) -> Entity {
        self.entity
    }
}
