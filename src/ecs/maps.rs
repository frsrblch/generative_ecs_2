use super::{Id, Valid, Get};
use rustc_hash::FxHashMap;
use std::hash::Hash;
use crate::ecs::{GenId, Insert};

#[derive(Debug, Clone)]
pub struct ComponentMap<ID: Hash + Eq, T> {
    values: FxHashMap<ID, T>,
}

impl<ID: Hash + Eq, T> Default for ComponentMap<ID, T> {
    fn default() -> Self {
        Self {
            values: Default::default(),
        }
    }
}

impl<ID: Hash + Eq, T> ComponentMap<ID, T> {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline(always)]
    pub fn remove(&mut self, id: &ID) -> Option<T> {
        self.values.remove(id)
    }
}

impl<ID, T> Get<Id<ID>, T> for ComponentMap<Id<ID>, T> {
    #[inline(always)]
    fn get(&self, id: Id<ID>) -> Option<&T> {
        self.values.get(&id)
    }

    #[inline(always)]
    fn get_mut(&mut self, id: Id<ID>) -> Option<&mut T> {
        self.values.get_mut(&id)
    }
}

impl<ID, T> Get<&Id<ID>, T> for ComponentMap<Id<ID>, T> {
    #[inline(always)]
    fn get(&self, id: &Id<ID>) -> Option<&T> {
        self.values.get(id)
    }

    #[inline(always)]
    fn get_mut(&mut self, id: &Id<ID>) -> Option<&mut T> {
        self.values.get_mut(id)
    }
}

impl<ID, T> Get<Valid<'_, ID>, T> for ComponentMap<GenId<ID>, T> {
    #[inline(always)]
    fn get(&self, id: Valid<ID>) -> Option<&T> {
        self.values.get(&id.id)
    }

    #[inline(always)]
    fn get_mut(&mut self, id: Valid<ID>) -> Option<&mut T> {
        self.values.get_mut(&id.id)
    }
}

impl<ID, T> Get<&Valid<'_, ID>, T> for ComponentMap<GenId<ID>, T> {
    #[inline(always)]
    fn get(&self, id: &Valid<ID>) -> Option<&T> {
        self.values.get(&id.id)
    }

    #[inline(always)]
    fn get_mut(&mut self, id: &Valid<ID>) -> Option<&mut T> {
        self.values.get_mut(&id.id)
    }
}

impl<ID, T> Insert<Id<ID>, T> for ComponentMap<Id<ID>, T> {
    #[inline(always)]
    fn insert(&mut self, id: &Id<ID>, value: T) {
        self.values.insert(*id, value);
    }
}

impl<ID, T> Insert<Valid<'_, ID>, T> for ComponentMap<GenId<ID>, T> {
    #[inline(always)]
    fn insert(&mut self, id: &Valid<ID>, value: T) {
        self.values.insert(id.id, value);
    }
}