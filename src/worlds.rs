use crate::arenas::*;
use crate::entities::{Entity, EntityCore};
use crate::lifespans::*;
use code_gen::Visibility::Pub;
use code_gen::*;
use std::collections::HashMap;
use std::fmt::*;
use std::str::FromStr;

// TODO EntityEnum: add state transition functions

#[derive(Debug, Default)]
pub struct World {
    pub use_statements: Vec<String>,

    pub fields: Vec<Field>,
    pub arenas: Vec<ArenaCore>,
    pub entities: Vec<EntityCore>,

    pub allocator: HashMap<ArenaName, Type>,
    pub id: HashMap<ArenaName, Type>,
    pub valid_id: HashMap<ArenaName, Type>,
}

impl Display for World {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for u in self.use_statements.iter() {
            writeln!(f, "{}", u).ok();
        }

        writeln!(f, "use generative_ecs_2::ecs::*;\n").ok();

        writeln!(f, "{}", self.generate_world()).ok();

        writeln!(f, "{}", self.generate_allocators()).ok();

        writeln!(f, "{}", self.generate_state()).ok();

        for arena in self.generate_arenas() {
            writeln!(f, "{}", arena).ok();
        }

        for row in self.generate_arena_rows() {
            writeln!(f, "{}", row).ok();
        }

        for entity in self.generate_entities() {
            writeln!(f, "{}", entity).ok();
        }

        for row_enum in self.generate_entity_row_enums() {
            writeln!(f, "{}", row_enum).ok();
        }

        for id_enum in self.generate_entity_id_enums() {
            writeln!(f, "{}", id_enum).ok();
        }

        Ok(())
    }
}

impl World {
    pub fn new() -> Self {
        Default::default()
    }

    fn contains_arena(&self, arena_name: &ArenaName) -> bool {
        self.arenas.iter().any(|a| a.name.eq(arena_name))
    }

    pub fn add_state_field(&mut self, field_name: &str, field_type: &str) {
        self.fields.push(Field::new(field_name, field_type));
    }

    pub fn add_state_field_by_type(&mut self, type_name: &str) {
        let name = CamelCase::from_str(type_name)
            .map(|cc| cc.into_snake_case())
            .or_else(|_| SnakeCase::from_str(type_name))
            .expect(&format!(
                "Given type cannot be formatted as snake_case: {}",
                type_name
            ));

        let field_type = Type::new(type_name);

        let field = Field {
            visibility: Pub,
            name,
            field_type,
        };

        self.fields.push(field);
    }

    pub fn insert_arena<L: Lifespan>(&mut self, arena: Arena<L>) {
        if self.contains_arena(&arena.arena.name) {
            panic!(format!("Duplicate arena name: {}", arena.arena.name));
        }

        self.allocator.insert(arena.name(), arena.allocator());

        self.id.insert(arena.name(), arena.id_type());

        self.valid_id.insert(arena.name(), arena.valid_id_type());

        self.arenas.push(arena.arena);
    }

    pub fn insert_entity<L: Lifespan>(&mut self, entity: Entity<L>) {
        if !entity.get_arenas().all(|a| self.contains_arena(a)) {
            panic!(
                "Arena must be inserted before Entity: {}",
                entity.entity.base
            );
        }

        self.entities.push(entity.entity);
    }

    pub fn generate_world(&self) -> StructType {
        StructType {
            base: self.generate_world_struct(),
            enum_impl: Some(self.generate_world_impl()),
            enum_traits: vec![]
        }
    }

    pub fn generate_world_struct(&self) -> Struct {
        Struct::new(WORLD)
            .with_derives(Derives::with_debug_default_clone())
            .add_field(Field::from_type(Type::new(ALLOCATORS)))
            .add_field(Field::from_type(Type::new(STATE)))
    }

    pub fn generate_world_impl(&self) -> Impl {
        let world_impl = Impl::new(WORLD).add_function(Self::get_split_function());

        let entity_methods = self
            .entities
            .iter()
            .map(|e| self.generate_entity_function(e));

        let arena_functions = self
            .arenas
            .iter()
            .filter_map(|a| self.generate_non_entity_arena_function(a));

        entity_methods
            .chain(arena_functions)
            .fold(world_impl, |world, f| world.add_function(f))
    }

    fn get_split_function() -> Function {
        Function::new("split")
            .with_parameters("&mut self")
            .with_return(format!("(&mut {}, &mut {})", ALLOCATORS, STATE))
            .add_line(CodeLine::new(
                0,
                &format!(
                    "(&mut self.{}, &mut self.{})",
                    Field::from_type(Type::new(ALLOCATORS)).name,
                    Field::from_type(Type::new(STATE)).name,
                ),
            ))
    }

    fn generate_non_entity_arena_function(&self, arena: &ArenaCore) -> Option<Function> {
        if self.entities.iter().any(|e| e.owns_arena(&arena.name)) {
            return None;
        }

        let func = Function::new(&format!("create_{}", arena.name.as_field_name()))
            .with_parameters(&format!(
                "&mut self, row: {}",
                self.generate_arena_row(arena).typ
            ))
            .with_return(self.get_valid_id(&arena.name).to_string())
            .add_line(CodeLine::new(
                0,
                &format!(
                    "let id = self.allocators.{e}.create();",
                    e = arena.name.as_field_name(),
                ),
            ))
            .add_line(CodeLine::new(
                0,
                &format!(
                    "self.state.{e}.insert(&id, row);",
                    e = arena.name.as_field_name(),
                ),
            ))
            .add_line(CodeLine::new(0, "id"));

        func.into()
    }

    fn generate_entity_function(&self, entity: &EntityCore) -> Function {
        let e = entity.base.as_field_name();

        let func = Function::new(&format!("create_{}", entity.base.as_field_name()))
            .with_parameters(&format!("&mut self, entity: {}", entity.name()))
            .with_return(self.get_valid_id(&entity.base).to_string())
            .add_line(CodeLine::new(0, "let (alloc, state) = self.split();"))
            .add_line(CodeLine::new(0, ""))
            .add_line(CodeLine::new(0, &format!("let id = state.{e}.create(entity.{e}, &mut alloc.{e});", e = e)));

        let func = entity
            .children
            .iter()
            .map(|child| self.get_arena(child))
            .fold(func, |func, child| {
                let c = child.name.as_field_name();

                func.add_line(CodeLine::new(0, ""))
                    .add_line(CodeLine::new(0, &format!("if let Some({c}) = entity.{c} {{", c=c)))
                    .add_line(CodeLine::new(1, &format!("let {c} = state.{c}.create({c}, &mut alloc.{c});", c=c)))
                    .add_line(CodeLine::new(1, &format!("state.link_{e}_to_{c}(&id, &{c});", e=e, c=c)))
                    .add_line(CodeLine::new(0, "}"))
            });

        let func = entity
            .enums
            .iter()
            .fold(func, |func, entity_enum| {
                let func = func
                    .add_line(CodeLine::new(0, &format!("match entity.{} {{", entity_enum.name.into_snake_case())));

                entity_enum.options.iter().fold(func, |func, opt| {
                    func.add_line(CodeLine::new(1, &format!("{}Row::{}(row) => {{", entity_enum.name, opt)))
                        .add_line(CodeLine::new(2, &format!("let {c} = state.{c}.create(row, &mut alloc.{c});", c=opt.as_field_name())))
                        .add_line(CodeLine::new(2, &format!("state.link_{e}_to_{c}(&id, &{c});", e=e, c=opt.as_field_name())))
                        .add_line(CodeLine::new(1, "}"))
                })
                    .add_line(CodeLine::new(0, "}"))
            });

        func.add_line(CodeLine::new(0, ""))
            .add_line(CodeLine::new(0, "id"))
    }

    pub fn generate_allocators(&self) -> Struct {
        let fields = self
            .arenas
            .iter()
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

    pub fn generate_arenas(&self) -> Vec<StructType> {
        self.arenas
            .iter()
            .map(|a| {
                StructType {
                    base: self.generate_arena(a),
                    enum_impl: Some(self.generate_arena_impl(a)),
                    enum_traits: vec![]
                }
            })
            .collect()
    }

    pub fn generate_arena_rows(&self) -> Vec<Struct> {
        self.arenas
            .iter()
            .map(|a| self.generate_arena_row(a))
            .collect()
    }

    pub fn generate_entities(&self) -> Vec<StructType> {
        self.entities
            .iter()
            .map(|e| {
                StructType {
                    base: e.generate_struct(),
                    enum_impl: None,
                    enum_traits: vec![]
                }
            })
            .collect()
    }

    pub fn generate_entity_id_enums(&self) -> Vec<EnumType> {
        self.entities
            .iter()
            .flat_map(|e| {
                e
                    .enums
                    .iter()
                    .map(|e| e.get_id_enum(self))
            })
            .collect()
    }

    pub fn generate_entity_row_enums(&self) -> Vec<EnumType> {
        self.entities
            .iter()
            .flat_map(|e| {
                e
                    .enums
                    .iter()
                    .map(|e| e.get_row_enum(self))
            })
            .collect()
    }

    pub fn generate_arena(&self, arena: &ArenaCore) -> Struct {
        let component_fields = arena.components.iter().map(|comp| Field {
            visibility: Pub,
            name: comp.field_name.clone(),
            field_type: comp.get_component_type(),
        });

        let entity_enums = self
            .entities
            .iter()
            .filter(|e| e.base.eq(&arena.name))
            .flat_map(|e| e.enums.iter())
            .map(|e| Field {
                visibility: Pub,
                name: e.name.into_snake_case(),
                field_type: e.get_component_type()
            });

        let own_links = self
            .entities
            .iter()
            .filter(|e| e.base.eq(&arena.name))
            .flat_map(|e| e.children.iter())
            .map(|c| Field {
                visibility: Pub,
                name: c.as_field_name(),
                field_type: Type::new(&format!("Component<Self,Option<{}>>", self.get_id(c))),
            });

        let entity_links = self
            .entities
            .iter()
            .flat_map(|e| {
                e.children
                    .iter()
                    .chain(e.collections.iter())
                    .chain(e.enums.iter().flat_map(|e| e.options.iter()))
                    .map(move |c| (e, c))
            })
            .filter(|(_e, c)| arena.name.eq(c))
            .map(|(e, _c)| Field {
                visibility: Pub,
                name: e.base.as_field_name(),
                field_type: Type::new(&format!("Component<Self,{}>", self.get_id(&e.base))),
            });

        let fields = entity_links
            .chain(component_fields)
            .chain(entity_enums)
            .chain(own_links)
            .collect();

        Struct::new(arena.name.as_str())
            .with_fields(fields)
            .with_derives(Derives::with_debug_default_clone())
    }

    pub fn generate_state(&self) -> StructType {
        StructType {
            base: self.generate_state_struct(),
            enum_impl: Some(self.generate_state_impl()),
            enum_traits: vec![]
        }
    }

    pub fn generate_state_struct(&self) -> Struct {
        let arena_fields = self.arenas.iter().map(|a| Field {
            visibility: Pub,
            name: a.name.as_field_name(),
            field_type: a.name.as_type(),
        });

        let fields = self.fields.iter().cloned().chain(arena_fields).collect();

        Struct::new(STATE)
            .with_derives(Derives::with_debug_default_clone())
            .with_fields(fields)
    }

    pub fn generate_state_impl(&self) -> Impl {
        let state_impl = Impl::new(STATE);

        // link entity children
        let entity_child_links = self.generate_entity_child_link_functions();

        // link entity enums
        let child_enum_links = self.generate_entity_enum_link_functions();

        entity_child_links
            .chain(child_enum_links)
            .fold(state_impl, |state_impl, f| state_impl.add_function(f))
    }

    fn generate_entity_child_link_functions(&self) -> impl Iterator<Item=Function> + '_ {
        self.entities
            .iter()
            .flat_map(|e| e.children.iter().map(move |c| (e, c)))
            .map(move |(e, c)| {
                let parent = e.base.as_field_name();
                let child = c.as_field_name();
                Function::new(&format!("link_{}_to_{}", &parent, &child))
                    .with_parameters(&format!(
                        "&mut self, {p}: &{p_id}, {c}: &{c_id}",
                        p=&parent,
                        p_id=self.get_valid_id(&e.base),
                        c=&child,
                        c_id=self.get_valid_id(c),
                    ))
                    .add_line(CodeLine::new(0, &format!(
                        "self.{p}.{c}.insert({p}, {c}.id().into());",
                        p=&parent,
                        c=&child,
                    )))
                    .add_line(CodeLine::new(0, &format!(
                        "self.{c}.{p}.insert({c}, {p}.id());",
                        p=&parent,
                        c=&child,
                    )))
            })
    }

    fn generate_entity_enum_link_functions(&self) -> impl Iterator<Item=Function> + '_ {
        self.entities
            .iter()
            .flat_map(move |e| {
                e.enums.iter().flat_map(move |ent_enum| {
                    ent_enum.options.iter().map(move |c| (e, ent_enum, c))
                })
            })
            .map(move |(e, ent_enum, c)| {
                let parent = e.base.as_field_name();
                let child = c.as_field_name();

                Function::new(&format!("link_{}_to_{}", &parent, &child))
                    .with_parameters(&format!(
                        "&mut self, {p}: &{p_id}, {c}: &{c_id}",
                        p=&parent,
                        p_id=self.get_valid_id(&e.base),
                        c=&child,
                        c_id=self.get_valid_id(c),
                    ))
                    .add_line(CodeLine::new(0, &format!(
                        "self.{p}.{e}.insert({p}, {c}.id().into());",
                        p=&parent,
                        e=ent_enum.name.into_snake_case(),
                        c=&child,
                    )))
                    .add_line(CodeLine::new(0, &format!(
                        "self.{c}.{p}.insert({c}, {p}.id());",
                        p=&parent,
                        c=&child,
                    )))
            })
    }

    pub(crate) fn generate_arena_row(&self, arena: &ArenaCore) -> Struct {
        let component_fields = arena
            .components
            .iter()
            .filter(|c| c.source == Source::ByValue)
            .filter_map(|c| c.get_row_field());

        let fields = component_fields.collect();

        Struct::new(&format!("{}Row", arena.name))
            .with_derives(Derives::with_debug_clone())
            .with_fields(fields)
    }

    fn generate_arena_impl(&self, arena: &ArenaCore) -> Impl {
        Impl::from(&Type::new(arena.name.as_str()))
            .add_function(self.get_insert_function(arena))
            .add_function(self.get_create_function(arena))
    }

    fn get_insert_function(&self, arena: &ArenaCore) -> Function {
        let mut func = Function::new("insert").with_parameters(&format!(
            "&mut self, id: &{}, row: {}",
            self.get_valid_id(&arena.name),
            self.generate_arena_row(arena).typ,
        ));

        for field in self.generate_arena_row(arena).fields {
            func = func.add_line(CodeLine::new(
                0,
                &format!("self.{}.insert(id, row.{});", field.name, field.name),
            ));
        }

        let mut func = arena
            .components
            .iter()
            .filter(|c| c.source == Source::ByDefault)
            .fold(func, |func, comp| {
                func.add_line(CodeLine::new(
                    0,
                    &format!("self.{}.insert(id, Default::default());", comp.field_name),
                ))
            });

        for (field, _arena) in arena.optional_refs.iter() {
            func = func.add_line(CodeLine::new(
                0,
                &format!("self.{}.insert(id, None);", field),
            ));
        }

        if let Some(entity) = self.get_entity(&arena.name) {
            for c in entity.children.iter() {
                func = func.add_line(CodeLine::new(
                    0,
                    &format!("self.{}.insert(id, None);", c.as_field_name()),
                ));
            }
        }

        func
    }

    fn get_create_function(&self, arena: &ArenaCore) -> Function {
        Function::new("create")
            .with_parameters(&format!(
                "&mut self, row: {}, alloc: &'a mut {}",
                self.generate_arena_row(arena).typ,
                self.get_allocator(&arena.name)
            ))
            .with_generics(Generics::one("'a"))
            .with_return(self.get_valid_id_with_lifetime(&arena.name).to_string())
            .add_line(CodeLine::new(0, "let id = alloc.create();"))
            .add_line(CodeLine::new(0, "self.insert(&id, row);"))
            .add_line(CodeLine::new(0, "id"))
    }

    pub fn get_arena(&self, arena: &ArenaName) -> &ArenaCore {
        self.arenas.iter().find(|a| a.name == *arena).unwrap()
    }

    pub fn get_entity(&self, arena: &ArenaName) -> Option<&EntityCore> {
        self.entities.iter().find(|e| e.base == *arena)
    }

    pub fn get_parent_entities<'a>(
        &'a self,
        arena: &'a ArenaName,
    ) -> impl Iterator<Item = &EntityCore> + 'a {
        self.entities
            .iter()
            .filter(move |e| e.children.contains(arena))
    }

    pub fn get_allocator(&self, arena: &ArenaName) -> Type {
        self.allocator.get(arena).unwrap().clone()
    }

    pub fn get_id(&self, arena: &ArenaName) -> Type {
        self.id.get(arena).unwrap().clone()
    }

    pub fn get_valid_id(&self, arena: &ArenaName) -> Type {
        self.valid_id.get(arena).unwrap().clone()
    }

    pub fn get_valid_id_with_lifetime(&self, arena: &ArenaName) -> Type {
        let mut id = self.valid_id.get(arena).unwrap().clone();
        if id.name.as_str() == "Valid" {
            id.types.push_front("'a");
        }
        id
    }
}

const WORLD: &'static str = "World";
const ALLOCATORS: &'static str = "Allocators";
const STATE: &'static str = "State";

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn generate_world() {
        let s = World::new().generate_world_struct();
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
        let s = get_world().generate_state_struct();
        println!("{}", s);
        //        assert!(false);
    }

    #[test]
    fn generate_arena() {
        let world = get_world();

        world
            .generate_arenas()
            .iter()
            .for_each(|a| println!("{}", a));

        //        assert!(false);
    }

    #[test]
    fn generate_entity() {
        let world = get_world();

        world
            .generate_entities()
            .iter()
            .for_each(|a| println!("{}", a));

        //        assert!(false);
    }

    #[test]
    fn generate_arena_rows() {
        let world = get_world();

        world
            .generate_arena_rows()
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

    pub fn get_world() -> World {
        Default::default()

        // let mut system = Arena::<Permanent>::new("System");
        // system.add_optional_component_with_field::<String>("name");
        // system.add_required_component("Position");
        //
        // let mut body = Arena::<Permanent>::new("Body");
        // body.add_reference(&system);
        // body.add_optional_component_with_field::<String>("name");
        // body.add_required_component::<Mass>();
        // body.add_required_component_with_field::<Length>("radius");
        // body.add_default_component::<Position>();
        // body.add_default_component::<Velocity>();
        //
        // let mut orbit = Arena::<Permanent>::new("Orbit");
        // orbit.add_optional_self_link("parent");
        // orbit.add_required_component_with_field::<Duration>("period");
        // orbit.add_required_component_with_field::<Length>("radius");
        // orbit.add_default_component_with_field::<Position>("relative_position");
        //
        // let mut surface = Arena::<Permanent>::new("Surface");
        // surface.add_required_component::<Area>();
        // surface.add_required_component::<Albedo>();
        // surface.add_default_component::<Temperature>();
        //
        // let mut nation = Arena::<Transient>::new("Nation");
        // nation.add_required_component_with_field::<String>("name");
        // nation.add_default_component::<Population>();
        //
        // let mut colony = Arena::<Transient>::new("Colony");
        // colony.add_reference(&body);
        // colony.add_reference(&nation);
        // colony.add_required_component_with_field::<String>("name");
        // colony.add_required_component::<Population>();
        //
        // let mut vessel = Arena::<Transient>::new("Vessel");
        // vessel.add_required_component_with_field::<String>("name");
        // vessel.add_required_component::<Mass>();
        //
        // let mut transit = Arena::<Transient>::new("Transit");
        // transit.add_required_component_with_field::<Time>("departure");
        // transit.add_required_component_with_field::<Time>("arrival");
        // transit.add_default_component::<Position>();
        // transit.add_reference(&vessel);
        // transit.add_reference_with_field(&body, "from");
        // transit.add_reference_with_field(&body, "to");
        //
        // let mut planet = Entity::new(&body);
        // planet.add_child(&orbit);
        // planet.add_child(&surface);
        //
        // let mut world = World::new();
        //
        // world.insert_arena(system);
        // world.insert_arena(body);
        // world.insert_arena(orbit);
        // world.insert_arena(surface);
        // world.insert_arena(nation);
        // world.insert_arena(colony);
        // world.insert_arena(vessel);
        // world.insert_arena(transit);
        //
        // world.insert_entity(planet);
        //
        // world
    }
}
