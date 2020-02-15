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

    pub fn create_body_entity(&mut self, entity: BodyEntity) -> Id<Body> {
        let (alloc, state) = self.split();
        
        let id = state.body.create(&mut alloc.body, entity.body);
        
        if let Some(orbit) = entity.orbit {
            let child = state.orbit.create(&mut alloc.orbit, orbit);
        }
        
        if let Some(surface) = entity.surface {
            let child = state.surface.create(&mut alloc.surface, surface);
        }
        
        id
    }
}

#[derive(Debug, Default, Clone)]
pub struct Allocators {
    pub system: FixedAllocator<System>,
    pub body: FixedAllocator<Body>,
    pub orbit: FixedAllocator<Orbit>,
    pub surface: FixedAllocator<Surface>,
}

#[derive(Debug, Default, Clone)]
pub struct State {
    pub system: System,
    pub body: Body,
    pub orbit: Orbit,
    pub surface: Surface,
}

#[derive(Debug, Default, Clone)]
pub struct System {
    pub name: Component<Self, Option<String>>,
    pub position: Component<Self, Position>,
}

impl System {
    fn insert(&mut self, id: &Id<System>, row: SystemRow) {
        self.name.insert(id, row.name);
        self.position.insert(id, row.position);
    }

    pub fn create(
        &mut self,
        allocator: &mut FixedAllocator<System>,
        row: SystemRow
    ) -> Id<System> {
        let id = allocator.create();
        self.insert(&id, row);
        id
    }
}

#[derive(Debug, Clone)]
pub struct SystemRow {
    pub name: Option<String>,
    pub position: Position,
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
    fn insert(&mut self, id: &Id<Body>, row: BodyRow) {
        self.system.insert(id, row.system);
        self.name.insert(id, row.name);
        self.mass.insert(id, row.mass);
        self.radius.insert(id, row.radius);
    }

    pub fn create(
        &mut self,
        allocator: &mut FixedAllocator<Body>,
        row: BodyRow
    ) -> Id<Body> {
        let id = allocator.create();
        self.insert(&id, row);
        id
    }

    pub fn link_to_orbit(&mut self, id: &Id<Body>, child: &Id<Orbit>) {
        self.orbit.insert(id, Some(child.id()));
    }

    pub fn link_to_surface(&mut self, id: &Id<Body>, child: &Id<Surface>) {
        self.surface.insert(id, Some(child.id()));
    }
}

#[derive(Debug, Clone)]
pub struct BodyRow {
    pub system: Id<System>,
    pub name: Option<String>,
    pub mass: Mass,
    pub radius: Length,
}

#[derive(Debug, Default, Clone)]
pub struct Orbit {
    pub body: Component<Self, Id<Body>>,
    pub parent: Component<Self, Option<Id<Orbit>>>,
    pub period: Component<Self, Duration>,
    pub radius: Component<Self, Length>,
    pub relative_position: Component<Self, Position>,
}

impl Orbit {
    fn insert(&mut self, id: &Id<Orbit>, row: OrbitRow) {
        self.parent.insert(id, row.parent);
        self.period.insert(id, row.period);
        self.radius.insert(id, row.radius);
    }

    pub fn create(
        &mut self,
        allocator: &mut FixedAllocator<Orbit>,
        row: OrbitRow
    ) -> Id<Orbit> {
        let id = allocator.create();
        self.insert(&id, row);
        id
    }

    pub fn link_to_body(&mut self, parent: &Id<Body>, child: &Id<Orbit>) {
        self.body.insert(child, parent.id());
    }
}

#[derive(Debug, Clone)]
pub struct OrbitRow {
    pub parent: Option<Id<Orbit>>,
    pub period: Duration,
    pub radius: Length,
}

#[derive(Debug, Default, Clone)]
pub struct Surface {
    pub body: Component<Self, Id<Body>>,
    pub area: Component<Self, Area>,
    pub albedo: Component<Self, Albedo>,
    pub temperature: Component<Self, Temperature>,
}

impl Surface {
    fn insert(&mut self, id: &Id<Surface>, row: SurfaceRow) {
        self.area.insert(id, row.area);
        self.albedo.insert(id, row.albedo);
    }

    pub fn create(
        &mut self,
        allocator: &mut FixedAllocator<Surface>,
        row: SurfaceRow
    ) -> Id<Surface> {
        let id = allocator.create();
        self.insert(&id, row);
        id
    }

    pub fn link_to_body(&mut self, parent: &Id<Body>, child: &Id<Surface>) {
        self.body.insert(child, parent.id());
    }
}

#[derive(Debug, Clone)]
pub struct SurfaceRow {
    pub area: Area,
    pub albedo: Albedo,
}

#[derive(Debug, Clone)]
pub struct BodyEntity {
    pub body: BodyRow,
    pub orbit: Option<OrbitRow>,
    pub surface: Option<SurfaceRow>,
}

#[derive(Debug, Default, Copy, Clone)] pub struct Position;
#[derive(Debug, Default, Copy, Clone)] pub struct Velocity;
#[derive(Debug, Default, Copy, Clone)] pub struct Temperature;
#[derive(Debug, Default, Copy, Clone)] pub struct Population;
#[derive(Debug, Default, Copy, Clone)] pub struct Time;
#[derive(Debug, Default, Copy, Clone)] pub struct Area;
#[derive(Debug, Default, Copy, Clone)] pub struct Albedo;
#[derive(Debug, Default, Copy, Clone)] pub struct Mass;
#[derive(Debug, Default, Copy, Clone)] pub struct Length;
#[derive(Debug, Default, Copy, Clone)] pub struct Duration;

fn main() {}
