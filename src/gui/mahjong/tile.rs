use super::*;

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
        self.tile_tag().mutate(m_tile);
    }

    pub fn set_emissive(&self, color: LinearRgba) {
        self.tile_tag().set_emissive(&mut param().materials, color);
    }

    fn tile_tag(&self) -> Mut<'_, TileTag> {
        param().tile_tags.get_mut(self.entity).unwrap()
    }
}

impl HasEntity for GuiTile {
    fn entity(&self) -> Entity {
        self.entity
    }
}
