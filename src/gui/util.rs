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

pub fn print_hierarchy(entity: Entity, names: &Query<&Name>, childrens: &Query<&Children>) {
    fn print_entity_tree(
        entity: Entity,
        depth: usize,
        names: &Query<&Name>,
        childrens: &Query<&Children>,
    ) {
        // インデント
        let indent = "  ".repeat(depth);

        // 名前があれば表示
        if let Ok(name) = names.get(entity) {
            println!("{}- {:?} ({})", indent, entity, name.as_str(),);
        } else {
            println!("{}- {:?}", indent, entity);
        }

        // 子を再帰的に表示
        if let Ok(children) = childrens.get(entity) {
            for child in children.iter() {
                print_entity_tree(child, depth + 1, names, childrens);
            }
        }
    }

    print_entity_tree(entity, 0, names, childrens);
}

pub fn reparent_tranform(
    child: Entity,
    new_parent: Entity,
    globals: &Query<&'static mut GlobalTransform>,
) -> Transform {
    let child_global = *globals.get(child).unwrap();
    let parent_global = *globals.get(new_parent).unwrap();
    let new_local = parent_global.affine().inverse() * child_global.affine();
    Transform::from_matrix(new_local.into())
}
