use crate::arenas::ArenaName;
use code_gen::Type;

pub trait Lifespan: Default {
    fn id_type(arena: &ArenaName) -> Type;
    fn valid_id_type(arena: &ArenaName) -> Type;
    fn allocator(arena: &ArenaName) -> Type;
    fn is_deletable() -> bool;
}

#[derive(Debug, Default)]
pub struct Permanent;

impl Lifespan for Permanent {
    fn id_type(arena: &ArenaName) -> Type {
        format!("Id<{}>", arena).parse().unwrap()
    }

    fn valid_id_type(arena: &ArenaName) -> Type {
        format!("Id<{}>", arena).parse().unwrap()
    }

    fn allocator(arena: &ArenaName) -> Type {
        format!("FixedAllocator<{}>", arena).parse().unwrap()
    }

    fn is_deletable() -> bool { false }
}

#[derive(Debug, Default)]
pub struct Transient;

impl Lifespan for Transient {
    fn id_type(arena: &ArenaName) -> Type {
        format!("GenId<{}>", arena).parse().unwrap()
    }

    fn valid_id_type(arena: &ArenaName) -> Type {
        format!("Valid<{}>", arena).parse().unwrap()
    }

    fn allocator(arena: &ArenaName) -> Type {
        format!("GenAllocator<{}>", arena).parse().unwrap()
    }

    fn is_deletable() -> bool { true }
}
