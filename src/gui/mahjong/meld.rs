use super::*;

#[derive(Debug)]
pub struct GuiMeld {
    entity: Entity,
}

impl GuiMeld {
    pub fn new(parent: Entity, seat: Seat) -> Self {
        let e_meld = param()
            .commands
            .spawn((Name::new(format!("Meld[{seat}]")), ChildOf(parent)))
            .id();
        Self { entity: e_meld }
    }
}

#[derive(Debug)]
pub struct GuiMeldItem {
    entity: Entity,
}
