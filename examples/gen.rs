use generative_ecs_2::arenas::Arena;
use generative_ecs_2::entities::{Entity, EntityEnum};
use generative_ecs_2::lifespans::*;
use generative_ecs_2::worlds::{World, Insert};

// cargo run --example gen && cargo check --example target

fn main() {
    let target = "./examples/target.rs";

    let addition = r#"#[derive(Debug, Default, Copy, Clone)]
pub struct Population;
#[derive(Debug, Default, Copy, Clone)]
pub struct Albedo(f64);
#[derive(Debug, Default, Copy, Clone)]
pub struct Starfield;

fn main() {
    let mut world = World::default();

    let sol = world.create_system(get_sol());
    let earth = world.create_body(get_earth(sol));
    let _moon = world.create_body(get_luna(&world.state, earth));
}

fn get_sol() -> SystemRow {
    SystemRow {
        name: "Sol".to_string().into(),
        position: Default::default(),
        temperature: Temperature::in_kelvin(5778.0),
        radius: Length::in_meters(696340e3),
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
        orbit: Some(OrbitRow {
            radius: Length::in_meters(149.6e9),
            period: Time::in_days(365.25),
            parent: None,
        }),
        surface: Some(SurfaceRow {
            area: Area::in_meters_squared(510.1e12),
            albedo: Albedo(0.30),
        }),
    }
}

fn get_luna(state: &State, earth: Id<Body>) -> BodyEntity {
    let system = state.body.system[&earth];
    let parent = state.body.orbit[&earth];
    BodyEntity {
        body: BodyRow {
            system,
            name: "Luna".to_string().into(),
            mass: Default::default(),
            radius: Default::default(),
        },
        orbit: Some(OrbitRow {
            radius: Length::in_meters(384748e3),
            period: Time::in_days(27.32),
            parent,
        }),
        surface: Some(SurfaceRow {
            area: Area::in_meters_squared(38e12),
            albedo: Albedo(0.12),
        }),
    }
}

pub struct BodyPosition;

impl BodyPosition {
    pub fn update(state: &mut State) {
        let position = &mut state.body.position;
        let orbit = &state.body.orbit;

        let relative_position = &state.orbit.relative_position;
        let parent = &state.orbit.parent;

        position
            .iter_mut()
            .zip(orbit.iter())
            .filter_map(|(pos, orbit)| orbit.map(|orbit| (pos, orbit)))
            .for_each(|(pos, orbit)| {
                let mut orbit = orbit;
                *pos = relative_position[orbit];
                while let Some(parent) = parent[orbit] {
                    orbit = parent;
                    *pos += relative_position[orbit];
                }
            })
    }
}

"#;

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

    let mut engine = Arena::<Transient>::new("Engine");
    engine.add_required_component_with_field("thrust", "Force");

    let mut vessel_transit = Arena::<Transient>::new("VesselTransit");
    vessel_transit.add_required_component_with_field("departure", "Time");
    vessel_transit.add_required_component_with_field("arrival", "Time");
    vessel_transit.add_default_component("Position");
    vessel_transit.add_reference_with_field("from", &body);
    vessel_transit.add_reference_with_field("to", &body);

    let mut vessel_orbit = Arena::<Transient>::new("VesselOrbit");
    vessel_orbit.add_optional_reference_with_field("parent", &body);
    vessel_orbit.add_required_component_with_field("period", "Time");

    let mut planet = Entity::new(&body);
    planet.add_child(&orbit);
    planet.add_child(&surface);

    let mut vessel_entity = Entity::new(&vessel);
    vessel_entity.add_child(&engine);
    vessel_entity.add_enum(EntityEnum::new("VesselLocation", vec![&vessel_orbit, &vessel_transit]));

    let mut world = World::new();

    world.add_state_field_by_type("Starfield");

    world.use_statements.push("use physics::*;".to_string());

    world.insert(system);
    world.insert(body);
    world.insert(orbit);
    world.insert(surface);
    world.insert(nation);
    world.insert(colony);
    world.insert(vessel);
    world.insert(engine);
    world.insert(vessel_transit);
    world.insert(vessel_orbit);

    world.insert(planet);
    world.insert(vessel_entity);

    world
}
