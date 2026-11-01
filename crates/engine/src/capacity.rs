//! Private typed unwinds abort a numeric transaction, never a Magic game.
use serde::Serialize;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CapacityExceeded {
    pub resource: &'static str,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Debug)]
pub(crate) struct CapacitySignal(pub CapacityExceeded);

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) trait CapacityExt<T> {
    fn capacity(self, resource: &'static str) -> T;
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn exhausted(resource: &'static str) -> ! {
    // Unlike panic!, this does not run the process panic hook. The Game action
    // boundary catches this private payload and restores its complete checkpoint.
    std::panic::resume_unwind(Box::new(CapacitySignal(CapacityExceeded { resource })))
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<T> CapacityExt<T> for Option<T> {
    #[inline]
    fn capacity(self, resource: &'static str) -> T {
        match self {
            Some(value) => value,
            None => exhausted(resource),
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<T, E> CapacityExt<T> for Result<T, E> {
    #[inline]
    fn capacity(self, resource: &'static str) -> T {
        match self {
            Ok(value) => value,
            Err(_) => exhausted(resource),
        }
    }
}
