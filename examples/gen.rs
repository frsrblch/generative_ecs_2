use generative_ecs_2::arenas::Arena;
use generative_ecs_2::entities::Entity;
use generative_ecs_2::lifetimes::*;
use generative_ecs_2::worlds::World;

fn main() {
    let target = "./examples/target.rs";

    let addition =
r#"#[derive(Debug, Default, Copy, Clone)]
pub struct Population;
#[derive(Debug, Default, Copy, Clone)]
pub struct Albedo(f64);

fn main() {
    let mut world = World::default();

    let sol = world.allocators.system.create();
    world.state.system.insert(&sol, get_sol("Sol"));

    let earth = world.create_body(get_earth(sol));
    let earth_orbit = world.state.body.orbit[&earth].unwrap();
    let _moon = world.create_body(get_moon(sol, earth_orbit));
}

fn get_sol(name: &str) -> SystemRow {
    SystemRow {
        name: name.to_string().into(),
        position: Default::default(),
        temperature: Temperature::in_kelvin(5778.0),
        radius: Length::in_meters(696340e3),
    }
}

fn get_surface(area: Area, albedo: Albedo) -> SurfaceRow {
    SurfaceRow { area, albedo }
}

fn get_orbit(radius: Length, period: Time, parent: Option<Id<Orbit>>) -> OrbitRow {
    OrbitRow {
        parent,
        period,
        radius,
    }
}

fn get_earth(system: Id<System>) -> BodyEntity {
    BodyEntity {
        body: BodyRow {
            system,
            name: "Earth".to_string().into(),
            mass: Mass::in_kilograms(5.972e24),
            radius: Length::in_meters(6371e3),
        },
        orbit: get_orbit(Length::in_meters(149.6e9), Time::in_days(365.25), None).into(),
        surface: get_surface(Area::in_meters_squared(510.1e12), Albedo(0.30)).into(),
    }
}

fn get_moon(system: Id<System>, earth_orbit: Id<Orbit>) -> BodyEntity {
    BodyEntity {
        body: BodyRow {
            system,
            name: None,
            mass: Default::default(),
            radius: Default::default()
        },
        orbit: get_orbit(Length::in_meters(384748e3), Time::in_days(27.32), Some(earth_orbit)).into(),
        surface: get_surface(Area::in_meters_squared(38e12), Albedo(0.12)).into(),
    }
}"#;

    let world = get_world();

    std::fs::write(target, world.to_string() + addition).ok();
}

pub fn get_world() -> World {
    let mut system = Arena::<Permanent>::new("System");
    system.add_optional_component_with_field("name", "String");
    system.add_required_component("Position");
    system.add_required_component("Temperature");
    system.add_required_component_with_field("radius", "Length");

    let mut body = Arena::<Permanent>::new("Body");
    body.add_reference(&system);
    body.add_optional_component_with_field("name", "String");
    body.add_required_component("Mass");
    body.add_required_component_with_field("radius", "Length");
    body.add_default_component("Position");
    body.add_default_component("Velocity");

    let mut orbit = Arena::<Permanent>::new("Orbit");
    orbit.add_optional_self_link("parent");
    orbit.add_required_component_with_field("period", "Time");
    orbit.add_required_component_with_field("radius", "Length");
    orbit.add_default_component_with_field("relative_position", "Position");

    let mut surface = Arena::<Permanent>::new("Surface");
    surface.add_required_component("Area");
    surface.add_required_component("Albedo");
    surface.add_default_component("Temperature");

    let mut nation = Arena::<Transient>::new("Nation");
    nation.add_required_component_with_field("name", "String");
    nation.add_default_component("Population");

    let mut colony = Arena::<Transient>::new("Colony");
    colony.add_reference(&body);
    colony.add_reference(&nation);
    colony.add_required_component_with_field("name", "String");
    colony.add_required_component("Population");

    let mut vessel = Arena::<Transient>::new("Vessel");
    vessel.add_required_component_with_field("name", "String");
    vessel.add_required_component("Mass");
    vessel.add_required_component("Speed");

    let mut transit = Arena::<Transient>::new("Transit");
    transit.add_required_component_with_field("departure", "Time");
    transit.add_required_component_with_field("arrival", "Time");
    transit.add_default_component("Position");
    transit.add_reference(&vessel);
    transit.add_reference_with_field(&body, "from");
    transit.add_reference_with_field(&body, "to");

    let mut planet = Entity::new(&body);
    planet.add_child(&orbit);
    planet.add_child(&surface);

    let mut world = World::new();

    world.use_statements.push("use physics::*;".to_string());

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
