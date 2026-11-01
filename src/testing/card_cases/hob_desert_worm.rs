//! Focused printed outcomes; shared engine invariants live in core tests.
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
    {
        let p = Player::P0;
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
fn hob_desert_worm_first_qualifying_attack_can_be_second_combat_and_never_third_same_turn() {
    {
        let p = Player::P0;
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
fn hob_desert_worm_history_precedes_provider_and_is_shared_by_all_sources() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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

// This nested module avoids changing the centrally managed assurance registry.
#[path = "hob_dain.rs"]
mod hob_dain;

#[test]
fn hob_desert_worm_dain_tax_payment_records_live_attack_power_once() {
    {
        let p = Player::P0;
        for (mountain_count, sacrifice_attacker) in [(5, false), (6, false), (6, true)] {
            let mut g = board(p);
            let worm = source(&mut g, p);
            put(
                &mut g,
                p.other(),
                "Dáin, Lord of the Iron Hills",
                Zone::Battlefield,
            );
            put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            g.refresh_enduring_stories();
            let mountains: Vec<_> = (0..mountain_count)
                .map(|_| put(&mut g, p, "Mountain", Zone::Battlefield))
                .collect();
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let mana_creature = if sacrifice_attacker {
                bear
            } else {
                let mountain = mountains[0];
                g.change_object_form(
                    g.object_ref(mountain),
                    crate::cards::AnimationSpec {
                        base: Some((1, 1)),
                        artifact: false,
                        creature_types: crate::cards::AnimationCreatureTypes::Named(&[]),
                        keywords: Keywords::NONE,
                        duration: crate::cards::AnimationDuration::Permanent,
                    },
                    characteristic_layers::AnimationTypeMode::AddCreature,
                );
                mountain
            };
            if sacrifice_attacker {
                g.add_counter(bear, CounterKind::Vigilance, 1);
            }
            let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(mana_creature)));
            g.state.players[p.index()].mana = [0; 6];
            combat(&mut g);
            act(&mut g, Action::Attack { card: worm });
            act(&mut g, Action::Attack { card: bear });
            act(&mut g, Action::FinishAttackers);
            assert_eq!(
                maximum(&g, p),
                None,
                "proposed attackers are not a declaration"
            );

            act(&mut g, Action::AbortAttackPayment);
            assert_eq!(
                maximum(&g, p),
                None,
                "aborting cannot consume the first attack"
            );
            act(&mut g, Action::FinishAttackers);
            let grantor = g.object_ref(ring);
            act(
                &mut g,
                Action::TapGrantedMana {
                    card: mana_creature,
                    grantor,
                    production: [0, 0, 3, 0, 0, 0],
                },
            );
            act(
                &mut g,
                Action::PayAttackCost {
                    payment: [0, 0, 2, 0, 0, 0],
                },
            );
            let expected = if sacrifice_attacker {
                12
            } else {
                (mountain_count as i128 - 1) * 2 + 2
            };
            assert_eq!(maximum(&g, p), Some(expected));
            assert_eq!(
                g.state.attackers.iter().any(|a| a.card == bear),
                !sacrifice_attacker
            );
            drive(&mut g);
            assert_eq!(g.observe(p).additional_combats, u64::from(expected >= 12));
        }
    }
}
