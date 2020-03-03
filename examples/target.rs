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

    pub fn create_vessel(&mut self, row: VesselRow) -> Valid<Vessel> {
        let id = self.allocators.vessel.create();
        self.state.vessel.insert(&id, row);
        id
    }

    pub fn create_transit(&mut self, row: TransitRow) -> Valid<Transit> {
        let id = self.allocators.transit.create();
        self.state.transit.insert(&id, row);
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
    pub transit: GenAllocator<Transit>,
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
    pub transit: Transit,
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
}

impl Vessel {
    pub fn insert(&mut self, id: &Valid<Vessel>, row: VesselRow) {
        self.name.insert(id, row.name);
        self.mass.insert(id, row.mass);
        self.speed.insert(id, row.speed);
    }

    pub fn create<'a>(&mut self, row: VesselRow, alloc: &'a mut GenAllocator<Vessel>) -> Valid<'a, Vessel> {
        let id = alloc.create();
        self.insert(&id, row);
        id
    }
}


#[derive(Debug, Default, Clone)]
pub struct Transit {
    pub departure: Component<Self, Time>,
    pub arrival: Component<Self, Time>,
    pub position: Component<Self, Position>,
    pub vessel: Component<Self, GenId<Vessel>>,
    pub from: Component<Self, Id<Body>>,
    pub to: Component<Self, Id<Body>>,
}

impl Transit {
    pub fn insert(&mut self, id: &Valid<Transit>, row: TransitRow) {
        self.departure.insert(id, row.departure);
        self.arrival.insert(id, row.arrival);
        self.vessel.insert(id, row.vessel);
        self.from.insert(id, row.from);
        self.to.insert(id, row.to);
        self.position.insert(id, Default::default());
    }

    pub fn create<'a>(&mut self, row: TransitRow, alloc: &'a mut GenAllocator<Transit>) -> Valid<'a, Transit> {
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
pub struct TransitRow {
    pub departure: Time,
    pub arrival: Time,
    pub vessel: GenId<Vessel>,
    pub from: Id<Body>,
    pub to: Id<Body>,
}

#[derive(Debug, Clone)]
pub struct BodyEntity {
    pub body: BodyRow,
    pub orbit: Option<OrbitRow>,
    pub surface: Option<SurfaceRow>,
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

