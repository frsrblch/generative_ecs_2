use code_gen::*;
use crate::arenas::*;
use crate::entities::{Entity, EntityCore};
use crate::lifetimes::*;
use code_gen::Visibility::Pub;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct World {
    pub arenas: Vec<ArenaCore>,
    pub entities: Vec<EntityCore>,

    pub allocator: HashMap<ArenaName, Type>,
    pub id_type: HashMap<ArenaName, Type>,
    pub valid_id_type: HashMap<ArenaName, Type>,
}

impl World {
    pub fn new() -> Self {
        Default::default()
    }

    fn contains_arena(&self, arena_name: &ArenaName) -> bool {
        self.arenas.iter().any(|a| a.name.eq(arena_name))
    }

    pub fn insert_arena<L: Lifetime>(&mut self, arena: Arena<L>) {
        if self.contains_arena(&arena.arena.name) {
            panic!(format!("Duplicate arena name: {}", arena.arena.name));
        }

        self.allocator.insert(arena.name(), arena.allocator());
        self.id_type.insert(arena.name(), arena.id_type());
        self.valid_id_type.insert(arena.name(), arena.valid_id_type());
        self.arenas.push(arena.arena);
    }

    pub fn insert_entity<L: Lifetime>(&mut self, entity: Entity<L>) {
        if !entity.get_arenas().all(|a| self.contains_arena(a)) {
            panic!("Arena must be inserted before Entity: {}", entity.entity.base);
        }

        self.entities.push(entity.entity);
    }

    pub fn generate_world(&self) -> Struct {
        Struct::new(WORLD)
            .with_derives(Derives::with_debug_default_clone())
            .add_field(Field::from_type(Type::new(ALLOCATORS)))
            .add_field(Field::from_type(Type::new(STATE)))
    }

    pub fn generate_world_impl(&self) -> Impl {
        let i = Impl::new(WORLD).add_function(Self::get_split_function());

        self.entities.iter()
            .map(|e| self.generate_entity_function(e))
            .fold(i, |w, f| w.add_function(f))
    }

    fn get_split_function() -> Function {
        Function::new(SPLIT)
            .with_parameters("&mut self")
            .with_return(format!("(&mut {}, &mut {})", ALLOCATORS, STATE))
            .add_line(CodeLine::new(0, &format!(
                "(&mut self.{}, &mut self.{})",
                Field::from_type(Type::new(ALLOCATORS)).name,
                Field::from_type(Type::new(STATE)).name,
            )))
    }

    fn generate_entity_function(&self, entity: &EntityCore) -> Function {
        let func = Function::new(&format!("create_{}_entity", entity.base.as_field_name()))
            .with_parameters(&format!("&mut self, entity: {}", entity.name()))
            .with_return(self.get_valid_id_type(&entity.base).to_string())
            .add_line(CodeLine::new(0, "let (alloc, state) = self.split();"))
            .add_line(CodeLine::new(0, ""))
            .add_line(CodeLine::new(0, &format!(
                "let id = state.{e}.create(&mut alloc.{e}, entity.{e});",
                e=entity.base.as_field_name(),
            )));

        entity.children.iter()
            .map(|child| self.get_arena(child))
            .fold(func, |mut func, child| {
                let c = child.name.as_field_name();
                func = func.add_line(CodeLine::new(0, ""));
                func = func.add_line(CodeLine::new(0, &format!("if let Some({c}) = entity.{c} {{", c=c)));
                func = func.add_line(CodeLine::new(1, &format!("let child = state.{c}.create(&mut alloc.{c}, entity.{c})", c=c)));
                func.add_line(CodeLine::new(0, "}"))
            })
    }

    pub fn generate_allocators(&self) -> Struct {
        let fields = self.arenas.iter()
            .map(|a| (&a.name, self.get_allocator(&a.name)))
            .map(|(name, field_type)| Field {
                visibility: Pub,
                name: name.as_field_name(),
                field_type: field_type.clone(),
            })
            .collect();

        Struct::new(ALLOCATORS)
            .with_derives(Derives::with_debug_default_clone())
            .with_fields(fields)
    }

    pub fn generate_arenas(&self) -> Vec<(Struct, Impl)> {
        self.arenas.iter()
            .map(|a| (self.generate_arena(a), self.generate_arena_impl(a)))
            .collect()
    }

    pub fn generate_entities(&self) -> Vec<Struct> {
        self.entities.iter()
            .map(|e| self.generate_entity(e))
            .collect()
    }

    pub fn generate_entity(&self, entity: &EntityCore) -> Struct {
        let mut fields = vec![
            Field {
                visibility: Pub,
                name: entity.base.as_field_name(),
                field_type: entity.base.get_row_type(),
            }
        ];

        let child_fields = entity.children
            .iter()
            .map(|c| {
                Field {
                    visibility: Pub,
                    name: c.as_field_name(),
                    field_type: Type::new(&format!("Option<{}>", c.get_row_type()))
                }
            });
        fields.extend(child_fields);

        Struct::new(entity.name().as_str())
            .with_derives(Derives::with_debug_clone())
            .with_fields(fields)
    }

    pub fn generate_arena(&self, arena: &ArenaCore) -> Struct {
        let component_fields = arena.components.iter()
            .map(|comp| {
                Field {
                    visibility: Pub,
                    name: comp.field_name.clone(),
                    field_type: comp.get_component_type(),
                }
            });

        let own_links = self.entities.iter()
            .filter(|e| e.base.eq(&arena.name))
            .flat_map(|e| e.children.iter())
            .map(|c| {
                Field {
                    visibility: Pub,
                    name: c.as_field_name(),
                    field_type: Type::new(&format!("Component<Self,Option<{}>>", self.get_id_type(c))),
                }
            });

        let entity_links = self.entities.iter()
            .flat_map(|e| {
                e.children.iter()
                    .chain(e.collections.iter())
                    .map(move |c| (e, c))
            })
            .filter(|(_e, c)| arena.name.eq(c))
            .map(|(e, _c)| {
                Field {
                    visibility: Pub,
                    name: e.base.as_field_name(),
                    field_type: Type::new(&format!("Component<Self,{}>", self.get_id_type(&e.base))),
                }
            });

        let fields = entity_links
            .chain(component_fields)
            .chain(own_links)
            .collect();

        Struct::new(arena.name.as_str())
            .with_fields(fields)
            .with_derives(Derives::with_debug_default_clone())
    }

    pub fn generate_state(&self) -> Struct {
        let fields = self.arenas.iter()
            .map(|a| {
                Field {
                    visibility: Pub,
                    name: a.name.as_field_name(),
                    field_type: a.name.as_type()
                }
            })
            .collect();

        Struct::new(STATE)
            .with_derives(Derives::with_debug_default_clone())
            .with_fields(fields)
    }

    pub fn generate_arena_rows(&self) -> Vec<Struct> {
        self.arenas.iter()
            .map(|a| self.generate_arena_row(a))
            .collect()
    }

    fn generate_arena_row(&self, arena: &ArenaCore) -> Struct {
        let component_fields = arena.components.iter()
            .filter(|c| c.source == Source::ByValue)
            .filter_map(|c| c.get_row_field());

        let fields = component_fields.collect();

        Struct::new(&format!("{}Row", arena.name))
            .with_derives(Derives::with_debug_clone())
            .with_fields(fields)
    }

    pub fn generate_arena_impls(&self) -> Vec<Impl> {
        self.arenas.iter()
            .map(|a| self.generate_arena_impl(a))
            .collect()
    }

    fn generate_arena_impl(&self, arena: &ArenaCore) -> Impl {
        let arena_impl = Impl::from(&self.generate_arena(arena).typ)
            .add_function(self.get_insert_function(arena))
            .add_function(self.get_create_function(arena));

        self.get_link_functions(&arena.name)
            .into_iter()
            .fold(arena_impl, |i, f| i.add_function(f))
    }

    fn get_insert_function(&self, arena: &ArenaCore) -> Function {
        let mut func = Function::new("insert")
            .with_visibility(Visibility::Private)
            .with_parameters(&format!(
                "&mut self, id: &{}, row: {}",
                self.get_valid_id_type(&arena.name),
                self.generate_arena_row(arena).typ,
            ));

        for field in self.generate_arena_row(arena).fields {
            func = func.add_line(CodeLine::new(0, &format!("self.{}.insert(id, row.{})", field.name, field.name)));
        }

        func
    }

    fn get_create_function(&self, arena: &ArenaCore) -> Function {
        Function::new("create")
            .with_parameters(&format!(
                "\n        &mut self,\n        allocator: &mut {},\n        row: {}\n    ",
                self.get_allocator(&arena.name),
                self.generate_arena_row(arena).typ)
            )
            .with_return(self.get_valid_id_type(&arena.name).to_string())
            .add_line(CodeLine::new(0, "let id = allocator.create();"))
            .add_line(CodeLine::new(0, "self.insert(&id, row);"))
            .add_line(CodeLine::new(0, "id"))
    }

    fn get_link_functions(&self, arena: &ArenaName) -> Vec<Function> {
        if let Some(entity) = self.get_entity(arena) {
            entity.children.iter()
                .map(|c| {
                    Function::new(&format!("link_to_{}", c.as_field_name()))
                        .with_parameters(&format!(
                            "&mut self, id: &{}, child: &{}",
                            self.get_valid_id_type(arena),
                            self.get_valid_id_type(c)
                        ))
                        .add_line(CodeLine::new(0, &format!("self.{}.insert(id, Some(child.id()));", c.as_field_name())))
                })
                .collect()
        } else {
            self.get_parent_entities(arena)
                .map(|e| {
                    let parent_field = e.base.as_field_name();
                    let parent_id = self.get_valid_id_type(&e.base);
                    let self_id = self.get_valid_id_type(arena);

                    Function::new(&format!("link_to_{}", parent_field))
                        .with_parameters(&format!(
                            "&mut self, parent: &{}, child: &{}",
                            parent_id,
                            self_id
                        ))
                        .add_line(CodeLine::new(0, &format!(
                            "self.{}.insert(child, parent.id());",
                            parent_field,
                        )))
                })
                .collect()
        }
    }

    pub fn get_arena(&self, arena: &ArenaName) -> &ArenaCore {
        self.arenas.iter().find(|a| a.name == *arena).unwrap()
    }

    pub fn get_entity(&self, arena: &ArenaName) -> Option<&EntityCore> {
        self.entities.iter().find(|e| e.base == *arena)
    }

    pub fn get_parent_entities<'a>(&'a self, arena: &'a ArenaName) -> impl Iterator<Item=&EntityCore> + 'a {
        self.entities.iter().filter(move |e| e.children.contains(arena))
    }

    pub fn get_allocator(&self, arena: &ArenaName) -> Type {
        self.allocator.get(arena).unwrap().clone()
    }

    pub fn get_id_type(&self, arena: &ArenaName) -> Type {
        self.id_type.get(arena).unwrap().clone()
    }

    pub fn get_valid_id_type(&self, arena: &ArenaName) -> Type {
        self.valid_id_type.get(arena).unwrap().clone()
    }
}

const WORLD: &'static str = "World";
const ALLOCATORS: &'static str = "Allocators";
const STATE: &'static str = "State";
const SPLIT: &'static str = "split";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_world() {
        let s = World::new().generate_world();
        println!("{}", s);
//        assert!(false);
    }

    #[test]
    fn generate_allocators() {
        let s = get_world().generate_allocators();
        println!("{}", s);
//        assert!(false);
    }

    #[test]
    fn generate_state() {
        let s = get_world().generate_state();
        println!("{}", s);
//        assert!(false);
    }

    #[test]
    fn generate_arena() {
        let world = get_world();

        world.generate_arenas()
            .iter()
            .for_each(|a| println!("{}\n{}", a.0, a.1));

        assert!(false);
    }

    #[test]
    fn generate_entity() {
        let world = get_world();

        world.generate_entities()
            .iter()
            .for_each(|a| println!("{}", a));

//        assert!(false);
    }

    #[test]
    fn generate_arena_rows() {
        let world = get_world();

        world.generate_arena_rows()
            .iter()
            .for_each(|a| println!("{}", a));

//        assert!(false);
    }

    #[test]
    fn generate_world_impls() {
        let world = get_world().generate_world_impl();

        println!("{}", world);

//        assert!(false);
    }

    #[test]
    #[should_panic]
    fn insert_duplicate_arena() {
        let mut world = World::new();
        world.insert_arena(Arena::<Permanent>::new("Test"));
        world.insert_arena(Arena::<Transient>::new("Test"));
    }

    #[derive(Default)] struct Position;
    #[derive(Default)] struct Velocity;
    #[derive(Default)] struct Temperature;
    #[derive(Default)] struct Population;
    struct Time;
    struct Area;
    struct Albedo;
    struct Mass;
    struct Length;
    struct Duration;

    pub fn get_world() -> World {
        let mut system = Arena::<Permanent>::new("System");
        system.add_optional_component_with_field::<String>("name");
        system.add_required_component::<Position>();

        let mut body = Arena::<Permanent>::new("Body");
        body.add_reference(&system);
        body.add_optional_component_with_field::<String>("name");
        body.add_required_component::<Mass>();
        body.add_required_component_with_field::<Length>("radius");
        body.add_default_component::<Position>();
        body.add_default_component::<Velocity>();

        let mut orbit = Arena::<Permanent>::new("Orbit");
        orbit.add_optional_self_link("parent");
        orbit.add_required_component_with_field::<Duration>("period");
        orbit.add_required_component_with_field::<Length>("radius");
        orbit.add_default_component_with_field::<Position>("relative_position");

        let mut surface = Arena::<Permanent>::new("Surface");
        surface.add_required_component::<Area>();
        surface.add_required_component::<Albedo>();
        surface.add_default_component::<Temperature>();

        let mut nation = Arena::<Transient>::new("Nation");
        nation.add_required_component_with_field::<String>("name");
        nation.add_default_component::<Population>();

        let mut colony = Arena::<Transient>::new("Colony");
        colony.add_reference(&body);
        colony.add_reference(&nation);
        colony.add_required_component_with_field::<String>("name");
        colony.add_required_component::<Population>();

        let mut vessel = Arena::<Transient>::new("Vessel");
        vessel.add_required_component_with_field::<String>("name");
        vessel.add_required_component::<Mass>();

        let mut transit = Arena::<Transient>::new("Transit");
        transit.add_required_component_with_field::<Time>("departure");
        transit.add_required_component_with_field::<Time>("arrival");
        transit.add_default_component::<Position>();
        transit.add_reference(&vessel);
        transit.add_reference_with_field(&body, "from");
        transit.add_reference_with_field(&body, "to");

        let mut planet = Entity::new(&body);
        planet.add_child(&orbit);
        planet.add_child(&surface);

        let mut world = World::new();

        world.insert_arena(system);
        world.insert_arena(body);
        world.insert_arena(orbit);
        world.insert_arena(surface);
        world.insert_arena(nation);
        world.insert_arena(colony);
        world.insert_arena(vessel);
        world.insert_arena(transit);

        world.insert_entity(planet);

        world
    }
}