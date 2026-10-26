//! First qualifying declaration is player history, not a per-source activation cap.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers
            )
        });
    }
    panic!(
        "Were-Worm decision budget: {:?} {:?}",
        g.state.step, g.state.pending
    );
}
fn drive(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn board(p: Player) -> Game {
    let mut g = scenario(p);
    g.state.players[p.other().index()].life = 500;
    g
}
fn source(g: &mut Game, p: Player) -> CardId {
    put(g, p, "Desert Were-Worm", Zone::Battlefield)
}
fn combat(g: &mut Game) {
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    until(g, |g| g.state.pending == Pending::Attackers);
}
fn declare(g: &mut Game, ids: &[CardId]) {
    for &card in ids {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
}
fn next_combat(g: &mut Game) {
    until(g, |g| {
        g.state.step == Step::BeginCombat && g.state.pending == Pending::Priority
    });
    until(g, |g| g.state.pending == Pending::Attackers);
}
fn giants(g: &mut Game, p: Player) -> [CardId; 2] {
    std::array::from_fn(|_| put(g, p, "Hamlet Glutton", Zone::Battlefield))
}
fn maximum(g: &Game, p: Player) -> Option<i128> {
    g.state.players[p.index()].greatest_declared_attack_power_this_turn
}
#[test]
fn hob_desert_worm_paid_body_live_mountains_nonbasic_control_and_layer7c() {
    for p in [Player::P0, Player::P1] {
        let mut g = board(p);
        let id = put(&mut g, p, "Desert Were-Worm", Zone::Hand);
        let m = put(&mut g, p, "Mountain", Zone::Battlefield);
        put(&mut g, p, "Raucous Theater", Zone::Battlefield);
        put(&mut g, p, "Forest", Zone::Battlefield);
        for _ in 0..3 {
            put(&mut g, p.other(), "Mountain", Zone::Battlefield);
        }
        assert_eq!(g.object_view(id).power, 0, "not a CDA in hand");
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 4];
        cast(&mut g, id, None);
        drive(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(stats(&g, id), (4, 5));
        assert!(g.has_subtype(id, "Dragon") && g.has_subtype(id, "Wurm"));
        let recipient = g.object_ref(id);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .base_stats
            .push(BaseStatsEffectView {
                recipient,
                source: recipient,
                power: Some(1),
                toughness: Some(3),
                timestamp,
                static_source: false,
                permanent: false,
            });
        assert_eq!(
            stats(&g, id),
            (5, 3),
            "Mountain modifier is 7c after the later 7b setter"
        );
        g.add_counter(id, CounterKind::PlusOne, 1);
        assert_eq!(stats(&g, id), (6, 4));
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, growth, Some(Target::Card(id)));
        drive(&mut g);
        assert_eq!(stats(&g, id), (9, 7));
        g.move_object(m, Zone::Graveyard);
        assert_eq!(stats(&g, id), (7, 7));
        g.change_controller(g.object_ref(id), p.other());
        assert_eq!(stats(&g, id), (11, 7));
        g.move_object(id, Zone::Hand);
        assert_eq!(g.object_view(id).power, 0);
    }
}
#[test]
fn hob_desert_worm_paid_source_cannot_attack_immediately_and_attack_selection_is_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = board(p);
        let w = put(&mut g, p, "Desert Were-Worm", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 4];
        cast(&mut g, w, None);
        drive(&mut g);
        let foreign = put(&mut g, p.other(), "Hamlet Glutton", Zone::Battlefield);
        let tapped = put(&mut g, p, "Hamlet Glutton", Zone::Battlefield);
        g.state.objects[tapped as usize].tapped = true;
        combat(&mut g);
        reject(&mut g, Action::Attack { card: w });
        reject(&mut g, Action::Attack { card: foreign });
        reject(&mut g, Action::Attack { card: tapped });
        assert_eq!(maximum(&g, p), None);
        act(&mut g, Action::FinishAttackers);
        assert_eq!(maximum(&g, p), None, "empty declaration is not an attack");
    }
}
#[test]
fn hob_desert_worm_first_qualifying_attack_can_be_second_combat_and_never_third_same_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = board(p);
        source(&mut g, p);
        let aurelia = put(&mut g, p, "Aurelia, the Warleader", Zone::Battlefield);
        let dragon = put(&mut g, p, "Hamlet Glutton", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        combat(&mut g);
        declare(&mut g, &[aurelia, dragon, bear]);
        drive(&mut g);
        assert_eq!(maximum(&g, p), Some(11));
        assert_eq!(
            g.observe(p).additional_combats,
            1,
            "only Aurelia's extra combat"
        );
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, growth, Some(Target::Card(bear)));
        drive(&mut g);
        assert_eq!(
            g.observe(p).additional_combats,
            1,
            "post-declaration boost cannot retroactively qualify"
        );
        next_combat(&mut g);
        declare(&mut g, &[aurelia, dragon, bear]);
        drive(&mut g);
        assert_eq!(maximum(&g, p), Some(14));
        assert_eq!(
            g.observe(p).additional_combats,
            1,
            "first qualifying attack grants its combat"
        );
        assert!(!g.object_view(dragon).tapped && !g.object_view(bear).tapped);
        next_combat(&mut g);
        declare(&mut g, &[aurelia, dragon, bear]);
        drive(&mut g);
        assert_eq!(g.observe(p).additional_combats, 0);
        assert!(g.object_view(dragon).tapped && g.object_view(bear).tapped);
        until(&mut g, |g| g.state.step == Step::Main2);
    }
}
#[test]
fn hob_desert_worm_signed_power_eleven_twelve_and_above_are_frozen_at_declaration() {
    for p in [Player::P0, Player::P1] {
        for total in [11, 12, 13] {
            let mut g = board(p);
            source(&mut g, p);
            let a = giants(&mut g, p);
            let small = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[small as usize].growth.power = total - 14;
            assert_eq!(stats(&g, small).0, total - 12);
            combat(&mut g);
            declare(&mut g, &[a[0], a[1], small]);
            assert_eq!(maximum(&g, p), Some(total));
            // Drop below threshold before resolving. This is an event, not an intervening-if.
            g.move_object(a[1], Zone::Graveyard);
            drive(&mut g);
            assert_eq!(g.observe(p).additional_combats, u64::from(total >= 12));
            assert_eq!(g.object_view(a[0]).tapped, total < 12);
        }
    }
}
#[test]
fn hob_desert_worm_history_precedes_provider_and_is_shared_by_all_sources() {
    for p in [Player::P0, Player::P1] {
        let mut g = board(p);
        let a = put(&mut g, p, "Aurelia, the Warleader", Zone::Battlefield);
        let d = put(&mut g, p, "Hamlet Glutton", Zone::Battlefield);
        let h = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        combat(&mut g);
        declare(&mut g, &[a, d, h]);
        drive(&mut g);
        assert_eq!(maximum(&g, p), Some(12));
        source(&mut g, p);
        next_combat(&mut g);
        declare(&mut g, &[a, d, h]);
        drive(&mut g);
        assert_eq!(
            g.observe(p).additional_combats,
            0,
            "later provider cannot see a new first qualifying attack"
        );
        let mut g = board(p);
        source(&mut g, p);
        source(&mut g, p);
        let ds = giants(&mut g, p);
        combat(&mut g);
        declare(&mut g, &ds);
        drive(&mut g);
        assert_eq!(
            g.observe(p).additional_combats,
            2,
            "both sources observe the same first event"
        );
    }
}
#[test]
fn hob_desert_worm_ability_suppression_does_not_erase_history_or_retroactively_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = board(p);
        let w = source(&mut g, p);
        put(&mut g, p, "Mountain", Zone::Battlefield);
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, aura, Some(Target::Card(w)));
        drive(&mut g);
        assert_eq!(stats(&g, w), (1, 1));
        let a = put(&mut g, p, "Aurelia, the Warleader", Zone::Battlefield);
        let d = put(&mut g, p, "Hamlet Glutton", Zone::Battlefield);
        let h = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        combat(&mut g);
        declare(&mut g, &[a, d, h]);
        drive(&mut g);
        assert_eq!(
            g.observe(p).additional_combats,
            1,
            "suppressed Wurm has no trigger"
        );
        assert_eq!(maximum(&g, p), Some(12));
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(stats(&g, w), (2, 5));
        next_combat(&mut g);
        declare(&mut g, &[a, d, h]);
        drive(&mut g);
        assert_eq!(g.observe(p).additional_combats, 0);
    }
}
#[test]
fn hob_desert_worm_source_departure_suppression_or_control_change_cannot_cancel_packet() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = board(p);
            let w = source(&mut g, p);
            let ds = giants(&mut g, p);
            let idle = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[idle as usize].tapped = true;
            combat(&mut g);
            declare(&mut g, &ds);
            match mode {
                0 => {
                    g.move_object(w, Zone::Exile);
                }
                1 => {
                    let aura = put(&mut g, p, "Eaten by Piranhas", Zone::Hand);
                    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
                    cast(&mut g, aura, Some(Target::Card(w)));
                }
                _ => g.change_controller(g.object_ref(w), p.other()),
            }
            drive(&mut g);
            assert_eq!(g.observe(p).additional_combats, 1);
            assert!(ds.into_iter().all(|id| !g.object_view(id).tapped));
            assert!(g.object_view(idle).tapped, "only actual attackers untap");
        }
    }
}
#[test]
fn hob_desert_worm_untaps_current_attackers_including_put_attacking_but_not_stale_incarnations() {
    for p in [Player::P0, Player::P1] {
        let mut g = board(p);
        let w = source(&mut g, p);
        g.state.objects[w as usize].tapped = true;
        let ds = giants(&mut g, p);
        combat(&mut g);
        declare(&mut g, &ds);
        let original = g.object_ref(ds[1]);
        g.move_object(ds[1], Zone::Exile);
        g.move_object(ds[1], Zone::Battlefield);
        assert_ne!(g.object_ref(ds[1]), original);
        g.state.objects[ds[1] as usize].tapped = true;
        let created = g.create_token_batch_capture(
            p,
            Card::from_name("@token/hob-dragon").unwrap(),
            1,
            true,
            Some(&[CombatDefender::Player(p.other())]),
            true,
        );
        let token = created[0].card;
        assert!(g.state.attackers.iter().any(|a| a.card == token));
        drive(&mut g);
        assert!(!g.object_view(token).tapped && !g.object_view(ds[0]).tapped);
        assert!(g.object_view(ds[1]).tapped && g.object_view(w).tapped);
        assert_eq!(
            maximum(&g, p),
            Some(12),
            "put-attacking token did not change declaration history"
        );
    }
}
#[test]
fn hob_desert_worm_put_attacking_creatures_never_trigger_or_qualify_a_small_declaration() {
    for p in [Player::P0, Player::P1] {
        let mut g = board(p);
        source(&mut g, p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        combat(&mut g);
        declare(&mut g, &[bear]);
        assert_eq!(maximum(&g, p), Some(2));
        g.create_token_batch_capture(
            p,
            Card::from_name("@token/hob-dragon").unwrap(),
            2,
            true,
            Some(&[
                CombatDefender::Player(p.other()),
                CombatDefender::Player(p.other()),
            ]),
            false,
        );
        drive(&mut g);
        assert_eq!(g.observe(p).additional_combats, 0);
        assert_eq!(maximum(&g, p), Some(2));
        assert!(g.object_view(bear).tapped);
    }
}
#[test]
fn hob_desert_worm_source_controller_filters_event_and_turn_reset_reopens_qualification() {
    for p in [Player::P0, Player::P1] {
        let mut g = board(p);
        let w = source(&mut g, p.other());
        let ds = giants(&mut g, p);
        combat(&mut g);
        declare(&mut g, &ds);
        drive(&mut g);
        assert_eq!(g.observe(p).additional_combats, 0);
        assert_eq!(maximum(&g, p), Some(12));
        assert_eq!(maximum(&g, p.other()), None);
        g.change_controller(g.object_ref(w), p);
        until(&mut g, |g| g.state.step == Step::Main2);
        // Real turn initialization resets both players' history, not provider entry/control.
        g.state.active = p.other();
        g.finish_cleanup();
        assert_eq!(g.state.active, p);
        assert_eq!(maximum(&g, p), None);
        assert_eq!(maximum(&g, p.other()), None);
        for id in ds {
            g.state.objects[id as usize].tapped = false;
        }
        combat(&mut g);
        declare(&mut g, &ds);
        drive(&mut g);
        assert_eq!(g.observe(p).additional_combats, 1);
    }
}
#[test]
fn hob_desert_worm_owned_finish_attackers_raw_index_restore_and_overflow_rollback() {
    for p in [Player::P0, Player::P1] {
        let mut g = board(p);
        source(&mut g, p);
        let ds = giants(&mut g, p);
        combat(&mut g);
        for card in ds {
            act(&mut g, Action::Attack { card });
        }
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == Action::FinishAttackers)
            .unwrap();
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        act(&mut raw, Action::FinishAttackers);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        drive(&mut raw);
        drive(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(maximum(&raw, p), Some(12));
        let expected = raw.snapshot();
        raw.restore(&saved).unwrap();
        act(&mut raw, Action::FinishAttackers);
        drive(&mut raw);
        assert_eq!(raw.snapshot(), expected);
        let mut g = board(p);
        source(&mut g, p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[a as usize].growth.power = i128::MAX - 2;
        g.state.objects[b as usize].growth.power = -1;
        combat(&mut g);
        for card in [a, b] {
            act(&mut g, Action::Attack { card });
        }
        let saved = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(p, Action::FinishAttackers),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        assert_eq!(maximum(&g, p), None);
        assert!(!g.object_view(a).tapped && !g.object_view(b).tapped);
    }
}
#[test]
fn hob_desert_worm_itself_can_supply_all_twelve_power_from_current_mountains() {
    for p in [Player::P0, Player::P1] {
        for n in [5, 6] {
            let mut g = board(p);
            let w = source(&mut g, p);
            for _ in 0..n {
                put(&mut g, p, "Mountain", Zone::Battlefield);
            }
            combat(&mut g);
            declare(&mut g, &[w]);
            drive(&mut g);
            assert_eq!(maximum(&g, p), Some(2 * n));
            assert_eq!(g.observe(p).additional_combats, u64::from(n == 6));
            assert_eq!(g.object_view(w).tapped, n == 5);
        }
    }
}
