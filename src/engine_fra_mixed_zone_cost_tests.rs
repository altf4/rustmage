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

#[test]
fn mixed_cost_waits_for_real_entry_life_choice_and_replays_owned_state() {
    for player in [Player::P0, Player::P1] {
        for pay_life in [false, true] {
            let mut g = scenario(player);
            let land = put(&mut g, player, "Watery Grave", Zone::Graveyard);
            let other = put(&mut g, player, "Bear Cub", Zone::Graveyard);
            let moves = [(g.object_ref(land), Zone::Exile), (g.object_ref(other), Zone::Exile)];
            let mut plans = g.plan_atomic_zone_cost(&moves).unwrap();
            plans[0].1.destination = Zone::Battlefield;
            let before = g.snapshot();
            g.stage_atomic_zone_cost(plans);
            assert!(g.advance_entries());
            assert!(matches!(g.state.pending, Pending::EntryLife { player: actor } if actor == player));
            assert_eq!(g.object_view(land).zone, Zone::Graveyard);
            assert_eq!(g.object_view(other).zone, Zone::Graveyard);
            let pending = g.snapshot();
            let action = Action::ChooseEntryLife { pay: pay_life };
            let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
            let mut raw = g.clone();
            let mut indexed = g.clone();
            assert!(raw.apply(player.other(), action.clone()).is_err());
            assert_eq!(raw.snapshot(), pending);
            raw.apply(player, action.clone()).unwrap();
            indexed.apply_index(player, g.revision(), index).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(raw.object_view(land).zone, Zone::Battlefield);
            assert_eq!(raw.object_view(other).zone, Zone::Exile);
            assert_eq!(raw.object_view(land).tapped, !pay_life);
            let after = raw.snapshot();
            raw.restore(&pending).unwrap();
            act(&mut raw, action);
            assert_eq!(raw.snapshot(), after);
            g.restore(&before).unwrap();
            assert_eq!(g.snapshot(), before);
        }
    }
}

#[test]
fn cannot_exile_prohibition_precedes_ordinary_prevention_for_cost() {
    use ability_world::TEST_EXILE_PROHIBITOR;
    use zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let provider = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let card = put(&mut g, player, "Shock", Zone::Graveyard);
        let reference = g.object_ref(card);
        TEST_EXILE_PROHIBITOR.with(|p| p.set(Some(g.object_ref(provider))));
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(reference)));
        let before = g.snapshot();
        assert!(g.plan_atomic_zone_cost(&[(reference, Zone::Exile)]).is_none());
        assert_eq!(g.snapshot(), before);
        TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
        let plans = g.plan_atomic_zone_cost(&[(reference, Zone::Exile)]).unwrap();
        assert!(plans[0].1.prevented);
        let events = g.commit_atomic_zone_cost(plans).unwrap();
        assert!(!events[0].moved);
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}
