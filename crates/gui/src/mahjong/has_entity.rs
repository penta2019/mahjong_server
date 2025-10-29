use super::{super::util::reparent_transform, prelude::*};

// model構造体に対して実装するトレイト
pub trait HasEntity {
    fn entity(&self) -> Entity;

    #[inline]
    fn cmd(&self) -> EntityCommands<'_> {
        cmd().entity(self.entity())
    }

    #[inline]
    fn insert(&self, bundle: impl Bundle) {
        self.cmd().insert(bundle);
    }

    #[inline]
    fn despawn(&self) {
        self.cmd().despawn();
    }

    #[inline]
    fn transform_from(&self, target: Entity) -> Transform {
        reparent_transform(self.entity(), target, &param().globals)
    }
}

#[macro_export]
macro_rules! impl_has_entity {
    ($ty:ty) => {
        impl HasEntity for $ty {
            #[inline]
            fn entity(&self) -> Entity {
                self.entity
            }
        }
    };
}
