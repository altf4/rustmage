use super::*;
use super::*;
pub use rustmage_core::engine::additional_costs::*;
#[cfg(test)]
mod tap_source_tests {
    use super::*;

    #[test]
    fn tap_symbol_reserves_the_source_but_a_group_cost_alone_does_not() {
        let source = ObjectRef {
            card: 0,
            generation: 7,
        };
        let other = ObjectRef {
            card: 1,
            generation: 7,
        };
        let goal = |name: &str| FundingGoal {
            actor: Player::P0,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: Some((
                source,
                0,
                Counters::default(),
                Card::from_name(name).unwrap(),
                None,
            )),
        };
        let tethermage = goal("Tenured Tethermage");
        assert!(!tap_cost_reserves_source(&tethermage, source));
        let lathril = goal("Lathril, Blade of the Elves");
        assert!(tap_cost_reserves_source(&lathril, source));
        assert!(!tap_cost_reserves_source(&lathril, other));
    }
}
