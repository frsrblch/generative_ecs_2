use crate::arenas::*;
use crate::lifespans::*;
use code_gen::*;
use std::marker::PhantomData;
use crate::worlds::World;
use code_gen::Visibility::Pub;

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
            || self.collections.contains(&arena)
            || self.enums.iter().any(|e| e.options.contains(&arena));
    }

    pub fn generate_struct(&self) -> Struct {
        let mut fields = vec![Field {
            visibility: Pub,
            name: self.base.as_field_name(),
            field_type: self.base.get_row_type(),
        }];

        let child_fields = self.children.iter().map(|c| Field {
            visibility: Pub,
            name: c.as_field_name(),
            field_type: Type::new(&format!("Option<{}>", c.get_row_type())),
        });
        fields.extend(child_fields);

        let enum_fields = self.enums.iter().map(|e| Field {
            visibility: Default::default(),
            name: e.name.into_snake_case(),
            field_type: e.get_row_type(),
        });
        fields.extend(enum_fields);

        Struct::new(self.name().as_str())
            .with_derives(Derives::with_debug_clone())
            .with_fields(fields)
    }
}

#[derive(Debug)]
pub struct Entity<L: Lifespan> {
    pub entity: EntityCore,
    marker: PhantomData<L>,
}

impl<L: Lifespan> Entity<L> {
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
    pub fn add_child(&mut self, child: &Arena<impl Lifespan>) {
        self.entity.children.push(child.name());
    }

    // 1 to [0..]
    pub fn add_collection(&mut self, child: &Arena<impl Lifespan>) {
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

// TODO impl From<VALID_ID> and From<Row> for entity enums
// TODO add function World::create_ENTITY_ENUM_ROW
// TODO for create_ENTITY, create and link to ENTITY_ENUM_ID

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

    pub fn get_row_enum(&self, world: &World) -> EnumType {
        let typ = self.get_row_type();
        let row_enum = Enum::new(&typ.to_string())
            .with_derives(Derives::with_debug_clone());

        let base = self.options
            .iter()
            .fold(row_enum, |row_enum, o| {
                let arena_row = world.generate_arena_row(&world.get_arena(o));
                row_enum.add_option(EnumOption::new(o.as_str(), vec![&arena_row.typ.to_string()]))
            });
        
        EnumType {
            base,
            enum_impl: None,
            enum_traits: vec![]
        }
    }

    pub fn get_id_enum(&self, world: &World) -> EnumType {
        let typ = self.get_id_enum_type();
        let id_enum = Enum::new(&typ.to_string()).with_derives(Self::id_derives());
        
        let base = self.options
            .iter()
            .fold(id_enum, |id_enum, o| {
                let arena_id = world.get_id(o);
                id_enum.add_option(EnumOption::new(o.as_str(), vec![&arena_id.to_string()]))
            });

        let enum_traits: Vec<TraitImpl> = self
            .options
            .iter()
            .map(|opt| {
                let id = world.get_id(opt);
                Self::from_trait()
                    .impl_for(&typ)
                    .with_generics(Generics::one(&id.to_string()))
                    .add_function(
                        Self::from_trait_function()
                            .with_parameters(&format!("value: {}", id))
                            .add_line(CodeLine::new(0, &format!("{}::{}(value)", typ, opt))))
            })
            .collect();

        EnumType {
            base,
            enum_impl: None,
            enum_traits
        }
    }

    fn from_trait() -> Trait {
        Trait::new("From")
            .with_generics(Generics::one("T"))
            .add_function_definition(Self::from_trait_function())
    }

    fn from_trait_function() -> TraitFunction {
        TraitFunction::new("from")
            .with_parameters("value: T")
            .with_return("Self")
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
        let mut parent = Arena::<Permanent>::new("Fleet");
        parent.add_required_component_with_field("ships", "u16");

        let mut orbit = Arena::<Transient>::new("FleetOrbit");
        orbit.add_required_component_with_field("period", "Time");

        let mut transit = Arena::<Transient>::new("FleetTransit");
        transit.add_required_component_with_field("arrival", "Time");

        let mut parent_entity = Entity::new(&parent);

        // enum
        let states = EntityEnum::new("FleetLocation", vec![&orbit, &transit]);
        parent_entity.add_enum(states);

        // // child
        // parent_entity.add_child(&orbit);
        // parent_entity.add_child(&transit);

        let mut world = World::default();

        world.insert_arena(parent);
        world.insert_arena(orbit);
        world.insert_arena(transit);

        world.insert_entity(parent_entity);


        println!("{}", world);

        assert!(false);
    }
}