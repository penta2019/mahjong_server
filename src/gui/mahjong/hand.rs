use super::*;

#[derive(Debug)]
pub struct GuiHand {
    entity: Entity,
    tiles: Vec<GuiTile>,
    drawn_tile: Option<GuiTile>,
}

impl GuiHand {
    pub fn new() -> Self {
        let entity = param().commands.spawn(Name::new("Hand")).id();
        Self {
            entity,
            tiles: vec![],
            drawn_tile: None,
        }
    }

    pub fn init(&mut self, tiles: &[Tile]) {
        for t in tiles {
            let tile = GuiTile::new(*t);
            param()
                .commands
                .entity(tile.entity())
                .insert((ChildOf(self.entity), self.tf_tile(false)));
            self.tiles.push(tile);
        }
    }

    pub fn deal_tile(&mut self, tile: Tile) {
        let tile = GuiTile::new(tile);
        param()
            .commands
            .entity(tile.entity())
            .insert((ChildOf(self.entity), self.tf_tile(true)));
        self.drawn_tile = Some(tile);
    }

    pub fn take_tile(&mut self, tile: Tile, is_drawn: bool) -> GuiTile {
        let gui_tile = if is_drawn {
            // 牌を明示的にツモ切り
            self.drawn_tile.take().unwrap()
        } else if let Some(pos) = self.tiles.iter().position(|t| t.tile() == tile) {
            self.tiles.remove(pos)
        } else if let Some(drawn_tile) = &self.drawn_tile
            && drawn_tile.tile() == tile
        {
            // ツモ牌を暗黙に手牌から取り除く場合 (加槓,暗槓など)
            self.drawn_tile.take().unwrap()
        } else {
            panic!("{} not found in hand", tile);
        };
        assert!(gui_tile.tile() == tile);

        if let Some(drawn_tile) = self.drawn_tile.take() {
            self.tiles.push(drawn_tile);
        }

        gui_tile
    }

    pub fn align(&mut self) {
        self.tiles.sort_by_key(|t| t.tile());
        for (i, tile) in self.tiles.iter().enumerate() {
            param()
                .commands
                .entity(tile.entity())
                .insert(MoveTo::new(Vec3::new(
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
