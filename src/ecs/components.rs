use crate::ecs::ids::{Id, Valid};
use rayon::iter::*;
use std::any::type_name;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

pub trait Insert<ID, T> {
    fn insert(&mut self, id: &ID, value: T);
}

#[derive(Debug, Default, Clone)]
pub struct Component<ID, T> {
    values: Vec<T>,
    marker: PhantomData<ID>,
}

impl<ID, T> Component<ID, T> {
    fn insert_unchecked(&mut self, index: usize, value: T) {
        match self.values.len() {
            len if index < len => self.values[index] = value,
            len if index == len => self.values.push(value),
            len => panic!(format!(
                "Component<{},{}>: Invalid index ({}) for insert given current length ({})",
                type_name::<ID>(),
                type_name::<T>(),
                index,
                len
            )),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        (&self.values).into_iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        (&mut self.values).into_iter()
    }
}

impl<ID, T> Index<&Id<ID>> for Component<ID, T> {
    type Output = T;

    fn index(&self, index: &Id<ID>) -> &Self::Output {
        &self.values[index.index]
    }
}

impl<ID, T> IndexMut<&Id<ID>> for Component<ID, T> {
    fn index_mut(&mut self, index: &Id<ID>) -> &mut Self::Output {
        &mut self.values[index.index]
    }
}

impl<ID, T> Insert<Id<ID>, T> for Component<ID, T> {
    fn insert(&mut self, id: &Id<ID>, value: T) {
        self.insert_unchecked(id.index, value);
    }
}

impl<ID, T> Index<&Valid<'_, ID>> for Component<ID, T> {
    type Output = T;

    fn index(&self, index: &Valid<ID>) -> &Self::Output {
        &self.values[index.id.index]
    }
}

impl<ID, T> IndexMut<&Valid<'_, ID>> for Component<ID, T> {
    fn index_mut(&mut self, index: &Valid<ID>) -> &mut Self::Output {
        &mut self.values[index.id.index]
    }
}

impl<ID, T> Insert<Valid<'_, ID>, T> for Component<ID, T> {
    fn insert(&mut self, id: &Valid<ID>, value: T) {
        self.insert_unchecked(id.id.index, value);
    }
}

impl<'a, ID: Send, T: Send + Sync> IntoParallelIterator for &'a Component<ID, T> {
    type Iter = rayon::slice::Iter<'a, T>;
    type Item = &'a T;

    fn into_par_iter(self) -> Self::Iter {
        self.values.par_iter()
    }
}

impl<'a, ID: Send, T: Send + Sync> IntoParallelIterator for &'a mut Component<ID, T> {
    type Iter = rayon::slice::IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_par_iter(self) -> Self::Iter {
        self.values.par_iter_mut()
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
            a.insert_unchecked(i, i);
            b.insert_unchecked(i, 20 - i);
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
            a.insert_unchecked(i, i);
            b.insert_unchecked(i, 20 - i);
        }

        a.par_iter_mut().zip(b.par_iter()).for_each(|(a, b)| {
            *a += *b;
        });

        a.iter().for_each(|a| assert_eq!(20, *a));
    }
}
