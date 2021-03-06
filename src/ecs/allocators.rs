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

    pub fn ids(&self) -> impl Iterator<Item = Id<T>> {
        (0..self.next_index).into_iter().map(|i| Id::new(i))
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
            let gen = self
                .generation
                .get(index)
                .copied()
                .unwrap_or_else(Default::default);

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
        if let Some(gen) = self.generation.get(id.index) {
            *gen == id.gen
        } else {
            false
        }
    }

    pub fn kill(&mut self, id: GenId<T>) {
        if let Some(gen) = self.generation.get_mut(id.index) {
            if *gen == id.gen {
                *gen = gen.next();

                self.living.remove(id.index);
                self.dead.push(id.index);
            }
        }
    }

    pub fn ids<'a>(&'a self) -> impl Iterator<Item = Valid<T>> + 'a {
        self.living.iter().filter_map(move |index| {
            let gen = self.generation.get(index)?;
            let id = GenId::new(index, *gen);
            Some(Valid::new(id))
        })
    }
}
