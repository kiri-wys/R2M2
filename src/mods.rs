//! Models related to Rimworld
//!
//! [`Mod`] contains all the relevant data to a Rimworld mod.
//! [`OrderedItems<T>`] is a container of <T> that is internally ordered
//! [`TagSpans`] may be used to get a display representation to
//! a collection of [`Tag`]. It blends background_color with each [`Tag`]'s color.
pub mod app_mod;
pub mod game;
pub mod tag;

pub use app_mod::Mod;
pub use tag::Tag;

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

pub trait Item {
    fn identifier(&self) -> &str;
    fn patch(&mut self, other: Self);
    fn vec_order(&self, other: &Self) -> Ordering;
}

pub use private::OrderedItems;
mod private {
    use super::*;

    #[derive(Default, Clone, Debug, Serialize)]
    pub struct OrderedItems<T: Item> {
        data: Vec<T>,
    }
    impl<'de, T> Deserialize<'de> for OrderedItems<T>
    where
        T: Item + Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct UnorderedVec<T> {
                data: Vec<T>,
            }
            let res: UnorderedVec<T> = UnorderedVec::deserialize(deserializer)?;
            Ok(res.data.into())
        }
    }
    impl<T: Item> OrderedItems<T> {
        pub fn get_by_name(&self, name: &str) -> Option<&T> {
            self.data.iter().find(|&item| item.identifier() == name)
        }
        fn get_mut_by_name(&mut self, name: &str) -> Option<&mut T> {
            self.data.iter_mut().find(|item| item.identifier() == name)
        }
        pub(super) fn sort(&mut self) {
            self.data.sort_by(|a, b| a.vec_order(b));
        }

        pub fn upsert(&mut self, other: T) {
            if let Some(existing) = self.get_mut_by_name(other.identifier()) {
                existing.patch(other);
                return;
            }
            let idx = self
                .data
                .binary_search_by(|curr| curr.vec_order(&other))
                .unwrap_or_else(|i| i);
            self.data.insert(idx, other);
        }
        pub const fn len(&self) -> usize {
            self.data.len()
        }

        pub const fn is_empty(&self) -> bool {
            self.data.is_empty()
        }

        pub fn get<I>(&self, index: I) -> Option<&I::Output>
        where
            I: std::slice::SliceIndex<[T]>,
        {
            self.data.get(index)
        }
        pub(super) fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
        where
            I: std::slice::SliceIndex<[T]>,
        {
            self.data.get_mut(index)
        }

        pub fn iter(&self) -> std::slice::Iter<'_, T> {
            self.data.iter()
        }

        pub fn first(&self) -> Option<&T> {
            self.data.first()
        }
    }

    impl<T> From<Vec<T>> for OrderedItems<T>
    where
        T: Item,
    {
        fn from(mut value: Vec<T>) -> Self {
            value.sort_by(|a, b| a.vec_order(b));
            Self { data: value }
        }
    }
}
