use generative_ecs_2::worlds::World;
use generative_ecs_2::arenas::Arena;
use generative_ecs_2::lifetimes::*;
use generative_ecs_2::entities::Entity;

fn main() {
    let target = "./examples/target.rs";

    let types = "#[derive(Debug, Default, Copy, Clone)] pub struct Position;
#[derive(Debug, Default, Copy, Clone)] pub struct Velocity;
#[derive(Debug, Default, Copy, Clone)] pub struct Temperature;
#[derive(Debug, Default, Copy, Clone)] pub struct Population;
#[derive(Debug, Default, Copy, Clone)] pub struct Time;
#[derive(Debug, Default, Copy, Clone)] pub struct Area;
#[derive(Debug, Default, Copy, Clone)] pub struct Albedo;
#[derive(Debug, Default, Copy, Clone)] pub struct Mass;
#[derive(Debug, Default, Copy, Clone)] pub struct Length;
#[derive(Debug, Default, Copy, Clone)] pub struct Duration;

fn main() {}\n";

    let world = get_world();

    std::fs::write(target, world.to_string() + types).ok();

}

#[derive(Debug, Default, Copy, Clone)] struct Position;
#[derive(Debug, Default, Copy, Clone)] struct Velocity;
#[derive(Debug, Default, Copy, Clone)] struct Temperature;
#[derive(Debug, Default, Copy, Clone)] struct Population;
#[derive(Debug, Default, Copy, Clone)] struct Time;
#[derive(Debug, Default, Copy, Clone)] struct Area;
#[derive(Debug, Default, Copy, Clone)] struct Albedo;
#[derive(Debug, Default, Copy, Clone)] struct Mass;
#[derive(Debug, Default, Copy, Clone)] struct Length;
#[derive(Debug, Default, Copy, Clone)] struct Duration;

pub fn get_world() -> World
{
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
//    world.insert_arena(nation);
//    world.insert_arena(colony);
//    world.insert_arena(vessel);
//    world.insert_arena(transit);

    world.insert_entity(planet);

    world
}