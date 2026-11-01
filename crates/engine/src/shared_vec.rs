//! Snapshot-owned buffers share storage until an independent branch mutates it.
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SharedVec<T>(Arc<Vec<T>>);
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<T> SharedVec<T> {
    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<T> Default for SharedVec<T> {
    fn default() -> Self {
        Self(Arc::new(Vec::new()))
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<T> From<Vec<T>> for SharedVec<T> {
    fn from(values: Vec<T>) -> Self {
        Self(Arc::new(values))
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<T> FromIterator<T> for SharedVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Vec::from_iter(iter).into()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<T> Deref for SharedVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<T: Clone> DerefMut for SharedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<'a, T> IntoIterator for &'a SharedVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<'a, T: Clone> IntoIterator for &'a mut SharedVec<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<T: Clone> IntoIterator for SharedVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        Arc::unwrap_or_clone(self.0).into_iter()
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<T: std::hash::Hash> std::hash::Hash for SharedVec<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
