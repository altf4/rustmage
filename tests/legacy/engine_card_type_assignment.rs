use super::*;
pub use rustmage_core::engine::card_type_assignment::*;

#[cfg(test)]
mod tests {
    use super::distinct_type_assignment;
    #[test]
    fn assigns_multi_type_objects_without_greedy_order_dependence() {
        let artifact = 1 << 4;
        let creature = 1;
        let enchantment = 1 << 5;
        assert!(distinct_type_assignment(&[]));
        assert!(distinct_type_assignment(&[artifact | creature, creature]));
        assert!(distinct_type_assignment(&[creature, artifact | creature]));
        assert!(distinct_type_assignment(&[
            artifact | creature,
            creature | enchantment,
            artifact
        ]));
        assert!(!distinct_type_assignment(&[
            artifact | creature,
            artifact | creature,
            creature
        ]));
        assert!(!distinct_type_assignment(&[artifact, 0]));
        assert!(!distinct_type_assignment(&[u16::MAX; 17]));
    }
}
