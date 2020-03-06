mod allocators;
mod components;
mod maps;
mod ids;

pub use allocators::*;
pub use components::*;
pub use maps::*;
pub use ids::*;

pub trait Insert<ID, T> {
    fn insert(&mut self, id: &ID, value: T);
}

pub trait Get<ID, T> {
    fn get(&self, id: ID) -> Option<&T>;
    fn get_mut(&mut self, id: ID) -> Option<&mut T>;
}

pub trait GetOpt<ID, T> {
    fn get_opt(&self, id: ID) -> Option<&T>;
    fn get_opt_mut(&mut self, id: ID) -> Option<&mut T>;
}

pub trait GetTuple2<ID, T1, T2> {
    fn get(&self, id: ID) -> Option<(&T1, &T2)>;
}

impl<'a, ID: 'a, T1, T2, A, B> GetTuple2<ID, T1, T2> for (&'a A, &'a B)
where
    ID: Copy,
    A: Get<ID, T1>,
    B: Get<ID, T2>,
{
    fn get(&self, id: ID) -> Option<(&T1, &T2)> {
        let t1 = self.0;
        let t2 = self.1;
        t1.get(id).and_then(move |t1| t2.get(id).map(|t2| (t1, t2)))
    }
}

pub trait GetTuple3<ID, T1, T2, T3> {
    fn get(&self, id: ID) -> Option<(&T1, &T2, &T3)>;
}

impl<'a, ID: 'a, T1, T2, T3, A, B, C> GetTuple3<ID, T1, T2, T3> for (&'a A, &'a B, &'a C)
where
    ID: Copy,
    A: Get<ID, T1>,
    B: Get<ID, T2>,
    C: Get<ID, T3>,
{
    fn get(&self, id: ID) -> Option<(&T1, &T2, &T3)> {
        let t1 = self.0.get(id)?;
        let t2 = self.1.get(id)?;
        let t3 = self.2.get(id)?;
        Some((t1, t2, t3))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuple2_test() {
        let mut alloc = FixedAllocator::<u32>::default();
        let mut c1 = Component::<u32, u8>::default();
        let mut c2 = Component::<u32, char>::default();

        let id = alloc.create();
        c1.insert(&id, 1);
        c2.insert(&id, 'a');

        assert_eq!(Some((&1u8, &'a')), (&c1, &c2).get(id));
    }

    #[test]
    fn tuple3_test() {
        let mut alloc = FixedAllocator::<u32>::default();
        let mut c1 = Component::<u32, u8>::default();
        let mut c2 = Component::<u32, char>::default();
        let mut c3 = Component::<u32, String>::default();

        let id = alloc.create();
        c1.insert(&id, 1);
        c2.insert(&id, 'a');
        c3.insert(&id, String::from("b"));

        assert_eq!(Some((&1u8, &'a', &String::from("b"))), (&c1, &c2, &c3).get(id));
    }
}