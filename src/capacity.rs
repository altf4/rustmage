//! Private typed unwinds abort a numeric transaction, never a Magic game.
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CapacityExceeded {
    pub resource: &'static str,
}
#[derive(Debug)]
pub(crate) struct CapacitySignal(pub CapacityExceeded);

pub(crate) trait CapacityExt<T> {
    fn capacity(self, resource: &'static str) -> T;
}
fn exhausted(resource: &'static str) -> ! {
    // Unlike panic!, this does not run the process panic hook. The Game action
    // boundary catches this private payload and restores its complete checkpoint.
    std::panic::resume_unwind(Box::new(CapacitySignal(CapacityExceeded { resource })))
}
impl<T> CapacityExt<T> for Option<T> {
    #[inline]
    fn capacity(self, resource: &'static str) -> T {
        match self {
            Some(value) => value,
            None => exhausted(resource),
        }
    }
}
impl<T, E> CapacityExt<T> for Result<T, E> {
    #[inline]
    fn capacity(self, resource: &'static str) -> T {
        match self {
            Ok(value) => value,
            Err(_) => exhausted(resource),
        }
    }
}
