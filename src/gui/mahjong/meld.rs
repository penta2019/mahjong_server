use super::*;

#[derive(Debug)]
pub struct GuiMeld {
    entity: Entity,
    items: Vec<GuiMeldItem>,
}

impl GuiMeld {
    pub fn new() -> Self {
        let entity = param().commands.spawn(Name::new("Meld")).id();
        Self {
            entity,
            items: vec![],
        }
    }
}

impl HasEntity for GuiMeld {
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Debug)]
pub struct GuiMeldItem {
    entity: Entity,
    tiles: Vec<GuiTile>,
    meld_index: Option<usize>,
}

impl GuiMeldItem {
    pub fn new(mut tiles: Vec<GuiTile>, meld_index: Option<usize>) -> Self {
        let entity = param().commands.spawn(Name::new("MeldItem")).id();

        for gui_tile in tiles.iter_mut().enumerate() {}

        Self {
            entity,
            tiles,
            meld_index,
        }
    }
}

impl HasEntity for GuiMeldItem {
    fn entity(&self) -> Entity {
        self.entity
    }
}
