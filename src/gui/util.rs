use bevy::prelude::*;

pub struct MsecTimer<const MSEC: usize>(Timer);

impl<const MSEC: usize> MsecTimer<MSEC> {
    pub fn tick(&mut self, time: &Time) -> bool {
        self.0.tick(time.delta()).just_finished()
    }
}

impl<const MSEC: usize> Default for MsecTimer<MSEC> {
    fn default() -> Self {
        let sec = MSEC as f32 / 1000.0;
        Self(Timer::from_seconds(sec, TimerMode::Repeating))
    }
}

#[allow(unused)]
pub fn print_hierarchy(
    entity: Entity,
    query_names: &Query<&Name>,
    query_children: &Query<&Children>,
) {
    fn print_entity_tree(
        entity: Entity,
        depth: usize,
        query_names: &Query<&Name>,
        query_children: &Query<&Children>,
    ) {
        // インデント
        let indent = "  ".repeat(depth);

        // 名前があれば表示
        if let Ok(name) = query_names.get(entity) {
            println!("{}- {:?} ({})", indent, entity, name.as_str(),);
        } else {
            println!("{}- {:?}", indent, entity);
        }

        // 子を再帰的に表示
        if let Ok(children) = query_children.get(entity) {
            for child in children.iter() {
                print_entity_tree(child, depth + 1, query_names, query_children);
            }
        }
    }

    print_entity_tree(entity, 0, query_names, query_children);
}
