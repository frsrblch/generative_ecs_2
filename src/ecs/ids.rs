use std::any::{type_name, Any};
use std::cmp::Ordering;
use std::fmt::*;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::num::NonZeroU32;

#[derive(Debug)]
pub struct Id<T> {
    pub(crate) index: usize,
    marker: PhantomData<T>,
}

impl<T: Any> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Id::<{}>::({})", type_name::<T>(), self.index)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self::new(self.index)
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index.eq(&other.index)
    }
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> Id<T> {
    pub(crate) fn new(index: usize) -> Self {
        Self {
            index,
            marker: PhantomData,
        }
    }

    pub fn id(&self) -> Self {
        *self
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Generation(NonZeroU32);

impl Display for Generation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0.get())
    }
}

impl Generation {
    pub fn next(self) -> Self {
        let next_gen = NonZeroU32::new(self.0.get() + 1).unwrap();
        Generation(next_gen)
    }

    pub fn value(self) -> u32 {
        self.0.get()
    }
}

impl Default for Generation {
    fn default() -> Self {
        Generation(NonZeroU32::new(1).unwrap())
    }
}

#[derive(Debug)]
pub struct GenId<T> {
    pub(crate) index: usize,
    pub(crate) gen: Generation,
    marker: PhantomData<T>,
}

impl<T: Any> Display for GenId<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "GenId::<{}>::({},{})",
            type_name::<T>(),
            self.index,
            self.gen
        )
    }
}

impl<T> Clone for GenId<T> {
    fn clone(&self) -> Self {
        Self::new(self.index, self.gen)
    }
}

impl<T> Copy for GenId<T> {}

impl<T> GenId<T> {
    pub(crate) fn new(index: usize, gen: Generation) -> Self {
        Self {
            index,
            gen,
            marker: PhantomData,
        }
    }
}

impl<T> PartialEq for GenId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.gen == other.gen
    }
}

impl<T> Eq for GenId<T> {}

impl<T> Hash for GenId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> PartialOrd for GenId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for GenId<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index).then(self.gen.cmp(&other.gen))
    }
}

#[derive(Debug)]
pub struct Valid<'a, T> {
    pub (crate) id: GenId<T>,
    marker: PhantomData<&'a ()>,
}

impl<'a, T> Valid<'a, T> {
    pub fn new(id: GenId<T>) -> Self {
        Self {
            id,
            marker: PhantomData,
        }
    }

    pub fn id(&self) -> GenId<T> {
        self.id
    }
}
