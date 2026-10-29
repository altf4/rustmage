//! Isolated shared-zone draft tests; linked only for grouped validation.
use super::*;

#[test]
fn mixed_zone_exile_prohibition_is_atomic_and_replays() {
    use ability_world::TEST_EXILE_PROHIBITOR;
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { TEST_EXILE_PROHIBITOR.with(|p| p.set(None)); } }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let provider = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let source = put(&mut g, player, "Shock", Zone::Hand);
        let grave = put(&mut g, player, "Bear Cub", Zone::Graveyard);
        let moves = [(g.object_ref(source), Zone::Exile), (g.object_ref(grave), Zone::Exile)];
        TEST_EXILE_PROHIBITOR.with(|p| p.set(Some(g.object_ref(provider))));
        let before = g.snapshot();
        for _ in 0..2 {
            assert!(g.plan_atomic_zone_cost(&moves).is_none());
            assert_eq!(g.snapshot(), before);
            g.restore(&before).unwrap();
        }
        TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
        assert!(g.plan_atomic_zone_cost(&moves).is_some());
    }
}

#[test]
fn frozen_mixed_replacement_redirects_stage_together_and_prevented_cost_is_paid() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let redirect = put(&mut g, player, "Bear Cub", Zone::Graveyard);
        let exile = put(&mut g, player, "Hill Giant", Zone::Graveyard);
        let prevented = put(&mut g, player, "Shock", Zone::Hand);
        let inputs = [(g.object_ref(redirect), Zone::Exile), (g.object_ref(exile), Zone::Exile), (g.object_ref(prevented), Zone::Exile)];
        let before = g.snapshot();
        for _ in 0..2 {
            let mut plans = g.plan_atomic_zone_cost(&inputs).unwrap();
            // A synthetic frozen outcome exercises the staged mixed-destination
            // commit independent of any currently supported redirect provider.
            plans[0].1.destination = Zone::Battlefield;
            plans[2].1.prevented = true;
            g.stage_atomic_zone_cost(plans);
            settle(&mut g);
            assert_eq!(g.object_view(redirect).zone, Zone::Battlefield);
            assert_eq!(g.object_view(exile).zone, Zone::Exile);
            assert_eq!(g.object_view(prevented).zone, Zone::Hand);
            g.restore(&before).unwrap();
            assert_eq!(g.snapshot(), before);
        }
    }
}
