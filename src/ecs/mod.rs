mod allocators;
mod components;
mod ids;

pub use allocators::*;
pub use components::*;
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