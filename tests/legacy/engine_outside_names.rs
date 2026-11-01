use super::*;
use super::*;
pub use rustmage_core::engine::outside_names::*;
use std::collections::HashMap;
use std::sync::OnceLock;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oracle_faces_supply_split_and_front_face_names() {
        let split = Card::from_name("Bottomless Pool // Locker Room").unwrap();
        assert_eq!(names(split), Some(&["Bottomless Pool", "Locker Room"][..]));
        assert_eq!(different(split, split), Some(false));
        let dfc = Card::from_name("Fatehold Chronologist // Peer Review").unwrap();
        assert_eq!(names(dfc), Some(&["Fatehold Chronologist"][..]));
        assert_eq!(different(split, dfc), Some(true));
    }
}
