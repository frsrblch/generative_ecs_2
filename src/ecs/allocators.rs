use crate::ecs::ids::*;
use bit_set::BitSet;
use std::marker::PhantomData;

#[derive(Debug, Default, Clone)]
pub struct FixedAllocator<T> {
    next_index: usize,
    marker: PhantomData<T>,
}

impl<T> FixedAllocator<T> {
    pub fn create(&mut self) -> Id<T> {
        let id = Id::new(self.next_index);
        self.next_index += 1;
        id
    }
}

#[derive(Debug, Default, Clone)]
pub struct GenAllocator<T> {
    generation: Vec<Generation>,
    dead: Vec<usize>,
    living: BitSet,
    marker: PhantomData<T>,
}

impl<T> GenAllocator<T> {
    pub fn create(&mut self) -> Valid<T> {
        if let Some(index) = self.dead.pop() {
            let gen = self.generation[index];

            self.living.insert(index);

            let id = GenId::new(index, gen);
            Valid::new(id)
        } else {
            let index = self.generation.len();
            let gen = Generation::default();

            self.generation.push(gen);
            self.living.insert(index);

            let id = GenId::new(index, gen);
            Valid::new(id)
        }
    }

    pub fn validate(&self, id: GenId<T>) -> Option<Valid<T>> {
        if self.is_alive(id) {
            Some(Valid::new(id))
        } else {
            None
        }
    }

    pub fn is_alive(&self, id: GenId<T>) -> bool {
        self.generation
            .get(id.index)
            .map(|current_gen| *current_gen == id.gen)
            .unwrap_or(false)
    }

    pub fn kill(&mut self, id: GenId<T>) {
        if self.is_alive(id) {
            let gen = &mut self.generation[id.index];
            *gen = gen.next();

            self.living.remove(id.index);
            self.dead.push(id.index);
        }
    }

    pub fn ids<'a>(&'a self) -> impl Iterator<Item=Valid<T>> + 'a {
        self.living
            .iter()
            .map(move |index| {
                let gen = self.generation[index];
                let id = GenId::new(index, gen);
                Valid::new(id)
            })
    }
}
