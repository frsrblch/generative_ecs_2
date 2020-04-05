pub mod arenas;
pub mod ecs;
pub mod entities;
pub mod lifespans;
pub mod worlds;

pub mod prelude {
    pub use crate::arenas::Arena;
    pub use crate::entities::Entity;
    pub use crate::lifespans::*;
    pub use crate::worlds::{World, Insert};
}
