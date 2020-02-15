pub mod arenas;
pub mod ecs;
pub mod entities;
pub mod lifetimes;
pub mod worlds;

pub mod prelude {
    pub use crate::arenas::Arena;
    pub use crate::entities::Entity;
    pub use crate::lifetimes::*;
    pub use crate::worlds::World;
}
