use super::{Get, GetOpt, Insert};
use crate::ecs::ids::{Id, Valid};
use rayon::iter::*;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use simd_vecs::vecs::Vec1;

#[derive(Debug, Clone)]
pub struct Component<ID, T> {
    values: Vec1<T>,
    marker: PhantomData<ID>,
}

impl<ID, T> Default for Component<ID, T> {
    fn default() -> Self {
        Self {
            values: Vec1::new(),
            marker: PhantomData,
        }
    }
}

impl<ID, T> Component<ID, T> {
    fn insert_at(&mut self, index: usize, value: T) {
        self.values.insert(value, index)
    }

    #[inline(always)]
    pub fn iter(&self) -> std::slice::Iter<T> {
        (&self.values.values).into_iter()
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        (&mut self.values.values).into_iter()
    }
}

impl<ID, T> GetOpt<Id<ID>, T> for Component<ID, Option<T>> {
    #[inline(always)]
    fn get_opt(&self, id: Id<ID>) -> Option<&T> {
        self.values.values.get(id.index).and_then(|o| o.as_ref())
    }

    #[inline(always)]
    fn get_opt_mut(&mut self, id: Id<ID>) -> Option<&mut T> {
        self.values.values.get_mut(id.index).and_then(|o| o.as_mut())
    }
}

impl<ID, T> GetOpt<&Id<ID>, T> for Component<ID, Option<T>> {
    #[inline(always)]
    fn get_opt(&self, id: &Id<ID>) -> Option<&T> {
        self.values.get(id.index).and_then(|o| o.as_ref())
    }

    #[inline(always)]
    fn get_opt_mut(&mut self, id: &Id<ID>) -> Option<&mut T> {
        self.values.get_mut(id.index).and_then(|o| o.as_mut())
    }
}

impl<ID, T> GetOpt<Valid<'_, ID>, T> for Component<ID, Option<T>> {
    #[inline(always)]
    fn get_opt(&self, id: Valid<ID>) -> Option<&T> {
        self.values.get(id.id.index).and_then(|o| o.as_ref())
    }

    #[inline(always)]
    fn get_opt_mut(&mut self, id: Valid<ID>) -> Option<&mut T> {
        self.values.get_mut(id.id.index).and_then(|o| o.as_mut())
    }
}

impl<ID, T> GetOpt<&Valid<'_, ID>, T> for Component<ID, Option<T>> {
    #[inline(always)]
    fn get_opt(&self, id: &Valid<ID>) -> Option<&T> {
        self.values.get(id.id.index).and_then(|o| o.as_ref())
    }

    #[inline(always)]
    fn get_opt_mut(&mut self, id: &Valid<ID>) -> Option<&mut T> {
        self.values.get_mut(id.id.index).and_then(|o| o.as_mut())
    }
}

impl<ID, T> Get<Id<ID>, T> for Component<ID, T> {
    #[inline(always)]
    fn get(&self, id: Id<ID>) -> Option<&T> {
        self.values.get(id.index)
    }

    #[inline(always)]
    fn get_mut(&mut self, id: Id<ID>) -> Option<&mut T> {
        self.values.get_mut(id.index)
    }
}

impl<ID, T> Get<&Id<ID>, T> for Component<ID, T> {
    #[inline(always)]
    fn get(&self, id: &Id<ID>) -> Option<&T> {
        self.values.get(id.index)
    }

    #[inline(always)]
    fn get_mut(&mut self, id: &Id<ID>) -> Option<&mut T> {
        self.values.get_mut(id.index)
    }
}

impl<ID, T> Get<Valid<'_, ID>, T> for Component<ID, T> {
    #[inline(always)]
    fn get(&self, id: Valid<ID>) -> Option<&T> {
        self.values.get(id.id.index)
    }

    #[inline(always)]
    fn get_mut(&mut self, id: Valid<ID>) -> Option<&mut T> {
        self.values.get_mut(id.id.index)
    }
}

impl<ID, T> Get<&Valid<'_, ID>, T> for Component<ID, T> {
    #[inline(always)]
    fn get(&self, id: &Valid<ID>) -> Option<&T> {
        self.values.get(id.id.index)
    }

    #[inline(always)]
    fn get_mut(&mut self, id: &Valid<ID>) -> Option<&mut T> {
        self.values.get_mut(id.id.index)
    }
}

impl<ID, T> Index<&Id<ID>> for Component<ID, T> {
    type Output = T;

    fn index(&self, index: &Id<ID>) -> &Self::Output {
        &self.values.values[index.index]
    }
}

impl<ID, T> IndexMut<&Id<ID>> for Component<ID, T> {
    fn index_mut(&mut self, index: &Id<ID>) -> &mut Self::Output {
        &mut self.values.values[index.index]
    }
}

impl<ID, T> Insert<Id<ID>, T> for Component<ID, T> {
    #[inline(always)]
    fn insert(&mut self, id: &Id<ID>, value: T) {
        self.insert_at(id.index, value);
    }
}

impl<ID, T> Index<Id<ID>> for Component<ID, T> {
    type Output = T;

    fn index(&self, index: Id<ID>) -> &Self::Output {
        &self.values.values[index.index]
    }
}

impl<ID, T> IndexMut<Id<ID>> for Component<ID, T> {
    fn index_mut(&mut self, index: Id<ID>) -> &mut Self::Output {
        &mut self.values.values[index.index]
    }
}

impl<ID, T> Index<&Valid<'_, ID>> for Component<ID, T> {
    type Output = T;

    fn index(&self, index: &Valid<ID>) -> &Self::Output {
        &self.values.values[index.id.index]
    }
}

impl<ID, T> IndexMut<&Valid<'_, ID>> for Component<ID, T> {
    fn index_mut(&mut self, index: &Valid<ID>) -> &mut Self::Output {
        &mut self.values.values[index.id.index]
    }
}

impl<ID, T> Index<Valid<'_, ID>> for Component<ID, T> {
    type Output = T;

    fn index(&self, index: Valid<ID>) -> &Self::Output {
        &self.values.values[index.id.index]
    }
}

impl<ID, T> IndexMut<Valid<'_, ID>> for Component<ID, T> {
    fn index_mut(&mut self, index: Valid<ID>) -> &mut Self::Output {
        &mut self.values.values[index.id.index]
    }
}

impl<ID, T> Insert<Valid<'_, ID>, T> for Component<ID, T> {
    #[inline(always)]
    fn insert(&mut self, id: &Valid<ID>, value: T) {
        self.insert_at(id.id.index, value);
    }
}

impl<'a, ID: Send, T: Send + Sync> IntoParallelIterator for &'a Component<ID, T> {
    type Iter = rayon::slice::Iter<'a, T>;
    type Item = &'a T;

    #[inline(always)]
    fn into_par_iter(self) -> Self::Iter {
        self.values.values.par_iter()
    }
}

impl<'a, ID: Send, T: Send + Sync> IntoParallelIterator for &'a mut Component<ID, T> {
    type Iter = rayon::slice::IterMut<'a, T>;
    type Item = &'a mut T;

    #[inline(always)]
    fn into_par_iter(self) -> Self::Iter {
        self.values.values.par_iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        let mut a = Component::<(), usize>::default();
        let mut b = Component::<(), usize>::default();

        for i in 0..10 {
            a.insert_at(i, i);
            b.insert_at(i, 20 - i);
        }

        a.iter_mut().zip(b.iter()).for_each(|(a, b)| {
            *a += *b;
        });

        a.iter().for_each(|a| assert_eq!(20, *a));
    }

    #[test]
    fn par_iter() {
        let mut a = Component::<(), usize>::default();
        let mut b = Component::<(), usize>::default();

        for i in 0..10 {
            a.insert_at(i, i);
            b.insert_at(i, 20 - i);
        }

        a.par_iter_mut().zip(b.par_iter()).for_each(|(a, b)| {
            *a += *b;
        });

        a.iter().for_each(|a| assert_eq!(20, *a));
    }
}
