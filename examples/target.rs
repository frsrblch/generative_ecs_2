use physics::*;
use generative_ecs_2::ecs::*;

#[derive(Debug, Default, Clone)]
pub struct World {
    pub allocators: Allocators,
    pub state: State,
}

impl World {
    pub fn split(&mut self) -> (&mut Allocators, &mut State) {
        (&mut self.allocators, &mut self.state)
    }

    pub fn create_body(&mut self, entity: BodyEntity) -> Id<Body> {
        let (alloc, state) = self.split();
        
        let id = state.body.create(entity.body, &mut alloc.body);

        if let Some(orbit) = entity.orbit {
            let orbit = state.orbit.create(orbit, &mut alloc.orbit);
            state.link_body_to_orbit(&id, &orbit);
        }

        if let Some(surface) = entity.surface {
            let surface = state.surface.create(surface, &mut alloc.surface);
            state.link_body_to_surface(&id, &surface);
        }

        
        id
    }

    pub fn create_vessel(&mut self, entity: VesselEntity) -> Valid<Vessel> {
        let (alloc, state) = self.split();
        
        let id = state.vessel.create(entity.vessel, &mut alloc.vessel);

        if let Some(engine) = entity.engine {
            let engine = state.engine.create(engine, &mut alloc.engine);
            state.link_vessel_to_engine(&id, &engine);
        }

        match entity.vessel_location {
            VesselLocationRow::VesselOrbit(row) => {
                let vessel_orbit = state.vessel_orbit.create(row, &mut alloc.vessel_orbit);
                state.link_vessel_to_vessel_orbit(&id, &vessel_orbit);
            }
            VesselLocationRow::VesselTransit(row) => {
                let vessel_transit = state.vessel_transit.create(row, &mut alloc.vessel_transit);
                state.link_vessel_to_vessel_transit(&id, &vessel_transit);
            }
        }
        
        id
    }

    pub fn delete_vessel(&mut self, id: GenId<Vessel>) {
        let (alloc, state) = self.split();

        if let Some(id) = alloc.vessel.validate(id) {
            if let Some(child) = state.vessel.engine.get_opt(&id) {
                alloc.engine.kill(*child);
            }

            match state.vessel.vessel_location.get(&id) {
                Some(VesselLocation::VesselOrbit(child)) => alloc.vessel_orbit.kill(*child),
                Some(VesselLocation::VesselTransit(child)) => alloc.vessel_transit.kill(*child),
                None => {},
            }
        }

        alloc.vessel.kill(id);
    }

    pub fn create_system(&mut self, row: SystemRow) -> Id<System> {
        let id = self.allocators.system.create();
        self.state.system.insert(&id, row);
        id
    }

    pub fn create_nation(&mut self, row: NationRow) -> Valid<Nation> {
        let id = self.allocators.nation.create();
        self.state.nation.insert(&id, row);
        id
    }

    pub fn create_colony(&mut self, row: ColonyRow) -> Valid<Colony> {
        let id = self.allocators.colony.create();
        self.state.colony.insert(&id, row);
        id
    }
}


#[derive(Debug, Default, Clone)]
pub struct Allocators {
    pub system: FixedAllocator<System>,
    pub body: FixedAllocator<Body>,
    pub orbit: FixedAllocator<Orbit>,
    pub surface: FixedAllocator<Surface>,
    pub nation: GenAllocator<Nation>,
    pub colony: GenAllocator<Colony>,
    pub vessel: GenAllocator<Vessel>,
    pub engine: GenAllocator<Engine>,
    pub vessel_transit: GenAllocator<VesselTransit>,
    pub vessel_orbit: GenAllocator<VesselOrbit>,
}

#[derive(Debug, Default, Clone)]
pub struct State {
    pub starfield: Starfield,
    pub system: System,
    pub body: Body,
    pub orbit: Orbit,
    pub surface: Surface,
    pub nation: Nation,
    pub colony: Colony,
    pub vessel: Vessel,
    pub engine: Engine,
    pub vessel_transit: VesselTransit,
    pub vessel_orbit: VesselOrbit,
}

impl State {
    pub fn link_body_to_orbit(&mut self, body: &Id<Body>, orbit: &Id<Orbit>) {
        self.body.orbit.insert(body, orbit.id().into());
        self.orbit.body.insert(orbit, body.id());
    }

    pub fn link_body_to_surface(&mut self, body: &Id<Body>, surface: &Id<Surface>) {
        self.body.surface.insert(body, surface.id().into());
        self.surface.body.insert(surface, body.id());
    }

    pub fn link_vessel_to_engine(&mut self, vessel: &Valid<Vessel>, engine: &Valid<Engine>) {
        self.vessel.engine.insert(vessel, engine.id().into());
        self.engine.vessel.insert(engine, vessel.id());
    }

    pub fn link_vessel_to_vessel_orbit(&mut self, vessel: &Valid<Vessel>, vessel_orbit: &Valid<VesselOrbit>) {
        self.vessel.vessel_location.insert(vessel, vessel_orbit.id().into());
        self.vessel_orbit.vessel.insert(vessel_orbit, vessel.id());
    }

    pub fn link_vessel_to_vessel_transit(&mut self, vessel: &Valid<Vessel>, vessel_transit: &Valid<VesselTransit>) {
        self.vessel.vessel_location.insert(vessel, vessel_transit.id().into());
        self.vessel_transit.vessel.insert(vessel_transit, vessel.id());
    }
}


#[derive(Debug, Default, Clone)]
pub struct System {
    pub name: Component<Self, Option<String>>,
    pub position: Component<Self, Position>,
    pub temperature: Component<Self, Temperature>,
    pub radius: Component<Self, Length>,
}

impl System {
    pub fn insert(&mut self, id: &Id<System>, row: SystemRow) {
        self.name.insert(id, row.name);
        self.position.insert(id, row.position);
        self.temperature.insert(id, row.temperature);
        self.radius.insert(id, row.radius);
    }

    pub fn create<'a>(&mut self, row: SystemRow, alloc: &'a mut FixedAllocator<System>) -> Id<System> {
        let id = alloc.create();
        self.insert(&id, row);
        id
    }
}


#[derive(Debug, Default, Clone)]
pub struct Body {
    pub system: Component<Self, Id<System>>,
    pub name: Component<Self, Option<String>>,
    pub mass: Component<Self, Mass>,
    pub radius: Component<Self, Length>,
    pub position: Component<Self, Position>,
    pub velocity: Component<Self, Velocity>,
    pub orbit: Component<Self, Option<Id<Orbit>>>,
    pub surface: Component<Self, Option<Id<Surface>>>,
}

impl Body {
    pub fn insert(&mut self, id: &Id<Body>, row: BodyRow) {
        self.system.insert(id, row.system);
        self.name.insert(id, row.name);
        self.mass.insert(id, row.mass);
        self.radius.insert(id, row.radius);
        self.position.insert(id, Default::default());
        self.velocity.insert(id, Default::default());
        self.orbit.insert(id, None);
        self.surface.insert(id, None);
    }

    pub fn create<'a>(&mut self, row: BodyRow, alloc: &'a mut FixedAllocator<Body>) -> Id<Body> {
        let id = alloc.create();
        self.insert(&id, row);
        id
    }
}


#[derive(Debug, Default, Clone)]
pub struct Orbit {
    pub body: Component<Self, Id<Body>>,
    pub parent: Component<Self, Option<Id<Orbit>>>,
    pub period: Component<Self, Time>,
    pub radius: Component<Self, Length>,
    pub relative_position: Component<Self, Position>,
}

impl Orbit {
    pub fn insert(&mut self, id: &Id<Orbit>, row: OrbitRow) {
        self.parent.insert(id, row.parent);
        self.period.insert(id, row.period);
        self.radius.insert(id, row.radius);
        self.relative_position.insert(id, Default::default());
    }

    pub fn create<'a>(&mut self, row: OrbitRow, alloc: &'a mut FixedAllocator<Orbit>) -> Id<Orbit> {
        let id = alloc.create();
        self.insert(&id, row);
        id
    }
}


#[derive(Debug, Default, Clone)]
pub struct Surface {
    pub body: Component<Self, Id<Body>>,
    pub area: Component<Self, Area>,
    pub albedo: Component<Self, Albedo>,
    pub temperature: Component<Self, Temperature>,
}

impl Surface {
    pub fn insert(&mut self, id: &Id<Surface>, row: SurfaceRow) {
        self.area.insert(id, row.area);
        self.albedo.insert(id, row.albedo);
        self.temperature.insert(id, Default::default());
    }

    pub fn create<'a>(&mut self, row: SurfaceRow, alloc: &'a mut FixedAllocator<Surface>) -> Id<Surface> {
        let id = alloc.create();
        self.insert(&id, row);
        id
    }
}


#[derive(Debug, Default, Clone)]
pub struct Nation {
    pub name: Component<Self, String>,
    pub population: Component<Self, Population>,
}

impl Nation {
    pub fn insert(&mut self, id: &Valid<Nation>, row: NationRow) {
        self.name.insert(id, row.name);
        self.population.insert(id, Default::default());
    }

    pub fn create<'a>(&mut self, row: NationRow, alloc: &'a mut GenAllocator<Nation>) -> Valid<'a, Nation> {
        let id = alloc.create();
        self.insert(&id, row);
        id
    }
}


#[derive(Debug, Default, Clone)]
pub struct Colony {
    pub body: Component<Self, Id<Body>>,
    pub nation: Component<Self, GenId<Nation>>,
    pub name: Component<Self, String>,
    pub population: Component<Self, Population>,
}

impl Colony {
    pub fn insert(&mut self, id: &Valid<Colony>, row: ColonyRow) {
        self.body.insert(id, row.body);
        self.nation.insert(id, row.nation);
        self.name.insert(id, row.name);
        self.population.insert(id, row.population);
    }

    pub fn create<'a>(&mut self, row: ColonyRow, alloc: &'a mut GenAllocator<Colony>) -> Valid<'a, Colony> {
        let id = alloc.create();
        self.insert(&id, row);
        id
    }
}


#[derive(Debug, Default, Clone)]
pub struct Vessel {
    pub name: Component<Self, String>,
    pub mass: Component<Self, Mass>,
    pub speed: Component<Self, Speed>,
    pub vessel_location: Component<Self, VesselLocation>,
    pub engine: Component<Self, Option<GenId<Engine>>>,
}

impl Vessel {
    pub fn insert(&mut self, id: &Valid<Vessel>, row: VesselRow) {
        self.name.insert(id, row.name);
        self.mass.insert(id, row.mass);
        self.speed.insert(id, row.speed);
        self.engine.insert(id, None);
    }

    pub fn create<'a>(&mut self, row: VesselRow, alloc: &'a mut GenAllocator<Vessel>) -> Valid<'a, Vessel> {
        let id = alloc.create();
        self.insert(&id, row);
        id
    }
}


#[derive(Debug, Default, Clone)]
pub struct Engine {
    pub vessel: Component<Self, GenId<Vessel>>,
    pub thrust: Component<Self, Force>,
}

impl Engine {
    pub fn insert(&mut self, id: &Valid<Engine>, row: EngineRow) {
        self.thrust.insert(id, row.thrust);
    }

    pub fn create<'a>(&mut self, row: EngineRow, alloc: &'a mut GenAllocator<Engine>) -> Valid<'a, Engine> {
        let id = alloc.create();
        self.insert(&id, row);
        id
    }
}


#[derive(Debug, Default, Clone)]
pub struct VesselTransit {
    pub vessel: Component<Self, GenId<Vessel>>,
    pub departure: Component<Self, Time>,
    pub arrival: Component<Self, Time>,
    pub position: Component<Self, Position>,
    pub from: Component<Self, Id<Body>>,
    pub to: Component<Self, Id<Body>>,
}

impl VesselTransit {
    pub fn insert(&mut self, id: &Valid<VesselTransit>, row: VesselTransitRow) {
        self.departure.insert(id, row.departure);
        self.arrival.insert(id, row.arrival);
        self.from.insert(id, row.from);
        self.to.insert(id, row.to);
        self.position.insert(id, Default::default());
    }

    pub fn create<'a>(&mut self, row: VesselTransitRow, alloc: &'a mut GenAllocator<VesselTransit>) -> Valid<'a, VesselTransit> {
        let id = alloc.create();
        self.insert(&id, row);
        id
    }
}


#[derive(Debug, Default, Clone)]
pub struct VesselOrbit {
    pub vessel: Component<Self, GenId<Vessel>>,
    pub parent: Component<Self, Option<Id<Body>>>,
    pub period: Component<Self, Time>,
}

impl VesselOrbit {
    pub fn insert(&mut self, id: &Valid<VesselOrbit>, row: VesselOrbitRow) {
        self.parent.insert(id, row.parent);
        self.period.insert(id, row.period);
    }

    pub fn create<'a>(&mut self, row: VesselOrbitRow, alloc: &'a mut GenAllocator<VesselOrbit>) -> Valid<'a, VesselOrbit> {
        let id = alloc.create();
        self.insert(&id, row);
        id
    }
}


#[derive(Debug, Clone)]
pub struct SystemRow {
    pub name: Option<String>,
    pub position: Position,
    pub temperature: Temperature,
    pub radius: Length,
}

#[derive(Debug, Clone)]
pub struct BodyRow {
    pub system: Id<System>,
    pub name: Option<String>,
    pub mass: Mass,
    pub radius: Length,
}

#[derive(Debug, Clone)]
pub struct OrbitRow {
    pub parent: Option<Id<Orbit>>,
    pub period: Time,
    pub radius: Length,
}

#[derive(Debug, Clone)]
pub struct SurfaceRow {
    pub area: Area,
    pub albedo: Albedo,
}

#[derive(Debug, Clone)]
pub struct NationRow {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ColonyRow {
    pub body: Id<Body>,
    pub nation: GenId<Nation>,
    pub name: String,
    pub population: Population,
}

#[derive(Debug, Clone)]
pub struct VesselRow {
    pub name: String,
    pub mass: Mass,
    pub speed: Speed,
}

#[derive(Debug, Clone)]
pub struct EngineRow {
    pub thrust: Force,
}

#[derive(Debug, Clone)]
pub struct VesselTransitRow {
    pub departure: Time,
    pub arrival: Time,
    pub from: Id<Body>,
    pub to: Id<Body>,
}

#[derive(Debug, Clone)]
pub struct VesselOrbitRow {
    pub parent: Option<Id<Body>>,
    pub period: Time,
}

#[derive(Debug, Clone)]
pub struct BodyEntity {
    pub body: BodyRow,
    pub orbit: Option<OrbitRow>,
    pub surface: Option<SurfaceRow>,
}


#[derive(Debug, Clone)]
pub struct VesselEntity {
    pub vessel: VesselRow,
    pub engine: Option<EngineRow>,
    pub vessel_location: VesselLocationRow,
}


#[derive(Debug, Clone)]
pub enum VesselLocationRow {
    VesselOrbit(VesselOrbitRow),
    VesselTransit(VesselTransitRow),
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum VesselLocation {
    VesselOrbit(GenId<VesselOrbit>),
    VesselTransit(GenId<VesselTransit>),
}

impl From<GenId<VesselOrbit>> for VesselLocation {
    fn from(value: GenId<VesselOrbit>) -> Self {
        VesselLocation::VesselOrbit(value)
    }
}

impl From<GenId<VesselTransit>> for VesselLocation {
    fn from(value: GenId<VesselTransit>) -> Self {
        VesselLocation::VesselTransit(value)
    }
}


#[derive(Debug, Default, Copy, Clone)]
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

