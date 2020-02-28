use crate::arenas::*;
use crate::lifetimes::*;
use code_gen::*;
use std::marker::PhantomData;
use crate::worlds::World;

//	From	    To	        Relationsh	Use Case	                                        Example
//	Permanent	Permanent	MaybeOwns	A -> Opt<B>	                                        not all bodies have an atmosphere
//	Permanent	Transient	MaybeOwns	A -> Opt<B>	                                        ??
//	Transient	Permanent	MaybeOwns	INVALID, child entity will leak if parent removed	-
//	Transient	Transient	MaybeOwns	A -> Opt<B>	                                        optional or shared component, only deleted by the owner
//	Permanent	Permanent	ManyOwns	A -> [B]	                                        planet owns several mine sites
//	Permanent	Transient	ManyOwns	A -> [B]	                                        NEW
//	Transient	Permanent	ManyOwns	INVALID, child entity will leak if parent removed	-
//	Transient	Transient	ManyOwns	A -> [B]	                                        NEW
//	Permanent	Permanent	Ref     	A -- B	                                            all bodies reference a system
//	Permanent	Permanent	MaybeRef	A -- Opt<B>	                                        ??
//	Permanent	Permanent	ManyRef	    A -- [B]	                                        NEW
//	Permanent	Transient	Ref	        INVALID, cannot be unlinked if child removed	    -
//	Permanent	Transient	MaybeRef	A -- Opt<B>	                                        ??
//	Permanent	Transient	ManyRef 	A -- [B]	                                        Systems lists bodies contained
//	Transient	Permanent	Ref	        A -- B	                                            colony references the body it's built upon
//	Transient	Permanent	MaybeRef	A -- Opt<B>	                                        ships can reference a system, but may not be in one
//	Transient	Permanent	ManyRef	    A -- [B]	                                        NEW
//	Transient	Transient	Ref	        MAYBE INVALID, must point at owner so that it can be deleted with it
//	Transient	Transient	MaybeRef	A -- Opt<B>                                         ship refers to its controller
//	Transient	Transient	ManyRef	    A -- [B]                                            NEW

#[derive(Debug)]
pub struct EntityCore {
    pub base: ArenaName,
    pub children: Vec<ArenaName>,    // one to maybe one
    pub collections: Vec<ArenaName>, // one to many
    pub enums: Vec<EntityEnum>,
}

impl EntityCore {
    pub fn new(arena: &ArenaCore) -> Self {
        Self {
            base: arena.name.clone(),
            children: vec![],
            collections: vec![],
            enums: vec![],
        }
    }

    pub fn name(&self) -> CamelCase {
        CamelCase::new(&format!("{}Entity", self.base))
    }

    pub(crate) fn owns_arena(&self, arena: &ArenaName) -> bool {
        return self.base == *arena
            || self.children.contains(&arena)
            || self.collections.contains(&arena);
    }
}

#[derive(Debug)]
pub struct Entity<L: Lifetime> {
    pub entity: EntityCore,
    marker: PhantomData<L>,
}

impl<L: Lifetime> Entity<L> {
    pub fn new(arena: &Arena<L>) -> Self {
        Entity {
            entity: EntityCore::new(&arena.arena),
            marker: PhantomData,
        }
    }

    pub fn get_arenas(&self) -> impl Iterator<Item = &ArenaName> {
        std::iter::once(&self.entity.base)
            .chain(self.entity.children.iter())
            .chain(self.entity.collections.iter())
    }

    pub fn add_enum(&mut self, entity_enum: EntityEnum) {
        self.entity.enums.push(entity_enum);
    }
}

impl Entity<Permanent> {
    // 1 to Option
    pub fn add_child(&mut self, child: &Arena<impl Lifetime>) {
        self.entity.children.push(child.name());
    }

    // 1 to [0..]
    pub fn add_collection(&mut self, child: &Arena<impl Lifetime>) {
        self.entity.collections.push(child.name());
    }
}

// transient cannot own permanent or it would leak when the parent is deleted
impl Entity<Transient> {
    // 1 to Option
    pub fn add_child(&mut self, child: &Arena<Transient>) {
        self.entity.children.push(child.name());
    }

    // 1 to [0..]
    pub fn add_collection(&mut self, child: &Arena<Transient>) {
        self.entity.collections.push(child.name());
    }
}

#[derive(Debug, Clone)]
pub struct EntityEnum {
    pub name: CamelCase,
    pub options: Vec<ArenaName>
}

impl EntityEnum {
    pub fn new(enum_type: &str, options: Vec<&Arena<Transient>>) -> Self {
        let options = options
            .into_iter()
            .map(|a| a.name())
            .collect();

        Self {
            name: CamelCase::new(enum_type),
            options
        }
    }

    pub fn get_row_type(&self) -> Type {
        Type::new(&format!("{}Row", self.name))
    }

    pub fn get_row_enum(&self, world: &World) -> Enum {
        let typ = self.get_row_type();
        let row_enum = Enum::new(&typ.to_string())
            .with_derives(Derives::with_debug_clone());

        self.options
            .iter()
            .fold(row_enum, |row_enum, o| {
                let arena_row = world.generate_arena_row(&world.get_arena(o));
                row_enum.add_option(EnumOption::new(o.as_str(), vec![&arena_row.typ.to_string()]))
            })
    }

    pub fn get_id_enum(&self, world: &World) -> Enum {
        let typ = self.get_id_enum_type();
        let id_enum = Enum::new(&typ.to_string()).with_derives(Self::id_derives());
        self.options
            .iter()
            .fold(id_enum, |id_enum, o| {
                let arena_id = world.get_id(o);
                id_enum.add_option(EnumOption::new(o.as_str(), vec![&arena_id.to_string()]))
            })
    }

    fn id_derives() -> Derives {
        let mut derives = Derives::with_debug_clone();
        derives.insert(Derive::Copy);
        derives.insert(Derive::Hash);
        derives.insert(Derive::Eq);
        derives.insert(Derive::Ord);
        derives
    }

    fn get_id_enum_type(&self) -> Type {
        Type::new(self.name.as_str())
    }

    pub fn get_component_type(&self) -> Type {
        Type::new(&format!("Component<Self,{}>", self.get_id_enum_type()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{World};

    #[test]
    fn simple_enum() {
        let parent = Arena::<Permanent>::new("Fleet");

        let state_1 = Arena::<Transient>::new("FleetOrbit");
        let state_2 = Arena::<Transient>::new("FleetTransit");

        let mut parent_entity = Entity::new(&parent);
        let states = EntityEnum::new("FleetLocation", vec![&state_1, &state_2]);
        parent_entity.add_enum(states);

        let mut world = World::default();

        world.insert_arena(parent);
        world.insert_arena(state_1);
        world.insert_arena(state_2);

        world.insert_entity(parent_entity);


        println!("{}", world);

        assert!(false);
    }
}