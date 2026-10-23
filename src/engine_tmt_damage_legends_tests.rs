//! Paid damage-card scenarios with owned raw/index rollback and both observations.
use super::ability_timeline_tests::{checked, named, next_turn, settle, settle_on, setup};
use super::*;
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
            .unwrap();
        checked(g, a);
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    let original = g.state.announcing.as_ref().map(|a| a.item.id);
    for _ in 0..8 {
        if g.state.announcing.as_ref().map(|a| a.item.id) != original
            || g.state.announcing.is_none()
        {
            break;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    id
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(before, g.snapshot());
    assert_eq!(revision, g.revision());
}
fn activate_ability(g: &mut Game, id: CardId, ability: u16, targets: &[Target]) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid activation");
}
fn until(g: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..100 {
        if condition(g) {
            return;
        }
        g.prepare_triggers();
        if condition(g) {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("condition not reached {:?}", g.state.pending);
}
fn combat(g: &mut Game, id: CardId) {
    until(g, |g| g.state.pending == Pending::Attackers);
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    until(g, |g| g.state.pending == Pending::Blockers);
}
fn own_main(g: &mut Game, p: Player) {
    until(g, |g| {
        g.state.active == p
            && g.state.step == Step::Main1
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
    });
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
}
fn activate(g: &mut Game, id: CardId, targets: &[Target]) {
    activate_ability(g, id, 0, targets);
}

fn activate_discard_color(
    g: &mut Game,
    p: Player,
    id: CardId,
    discard: CardId,
    color: crate::cards::ChoiceColor,
) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            break;
        }
        let wanted = Action::ChooseAdditionalCost {
            object: g.object_ref(discard),
        };
        let actions = g.legal_actions();
        let a = if actions.contains(&wanted) {
            wanted
        } else {
            actions
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap()
        };
        checked(g, a);
    }
    assert_eq!(g.state.objects[discard as usize].zone, Zone::Graveyard);
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseColor { color })
    });
    assert!(g.observe(p.other()).actions.is_empty());
    checked(g, Action::ChooseColor { color });
    settle(g);
}
#[test]
fn tmt_damage_legends_venus_nonlethal_damage_counters_and_lethal_no_rebinding() {
    for lethal in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let v = paid(&mut g, p, "Venus, Torn Between Worlds", &[]);
        settle(&mut g);
        paid(
            &mut g,
            p,
            if lethal { "Bot Bashing Time" } else { "Shock" },
            &[Target::Card(v)],
        );
        settle(&mut g);
        if lethal {
            assert!(!g.on_battlefield(v));
            assert_eq!(g.counter_count(v, CounterKind::PlusOne), 0);
        } else {
            assert_eq!(g.counter_count(v, CounterKind::PlusOne), 2);
            assert_eq!((g.stats(v).power, g.stats(v).toughness), (7, 7));
        }
    }
}
#[test]
fn tmt_damage_legends_venus_any_counter_dealer_optional_actual_blue_payment() {
    let p = Player::P0;
    let mut g = setup(p);
    paid(&mut g, p, "Venus, Torn Between Worlds", &[]);
    settle(&mut g);
    let island = named(&mut g, p, "Island", Zone::Battlefield);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    g.set_counters(bear, Counters::of(CounterKind::Dream, 1));
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    combat(&mut g, bear);
    checked(&mut g, Action::FinishBlockers);
    g.state.players[p.index()].mana = [30; 6];
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
    let hand = g.state.players[p.index()].library.len();
    checked(&mut g, Action::TapMana { card: island });
    checked(
        &mut g,
        Action::PayResolutionMana {
            payment: [0, 1, 0, 0, 0, 0],
        },
    );
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].library.len(), hand - 1);
    assert_eq!(g.state.players[p.index()].mana[1], 0);
}
#[test]
fn tmt_damage_legends_leatherhead_paid_entry_hexproof_then_any_counter_reflexive_destroy() {
    let p = Player::P0;
    let mut g = setup(p);
    let l = paid(&mut g, p, "Leatherhead, Swamp Stalker", &[]);
    settle(&mut g);
    assert_eq!(g.counter_count(l, CounterKind::Hexproof), 1);
    assert!(g.keywords(l).contains(Keywords::HEXPROOF));
    assert!(g.keywords(l).contains(Keywords::TRAMPLE));
    let own = named(&mut g, p, "Skateboard", Zone::Battlefield);
    let enemy = named(&mut g, p.other(), "Skateboard", Zone::Battlefield);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    combat(&mut g, l);
    checked(&mut g, Action::FinishBlockers);
    until(&mut g, |g| g.counter_removal_domain().is_some());
    checked(
        &mut g,
        Action::ChooseCounterRemoval {
            counters: Counters::of(CounterKind::Hexproof, 1),
        },
    );
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(enemy),
        })
    });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    assert_eq!(g.counter_count(l, CounterKind::Hexproof), 0);
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    settle(&mut g);
    assert!(!g.on_battlefield(enemy));
    assert!(!g.keywords(l).contains(Keywords::HEXPROOF));
}
#[test]
fn tmt_damage_legends_mondo_repeated_choices_fix_protection_colors_independently() {
    let p = Player::P0;
    let mut g = setup(p);
    let m = paid(&mut g, p, "Mondo Gecko", &[]);
    settle(&mut g);
    for color in [
        crate::cards::ChoiceColor::Red,
        crate::cards::ChoiceColor::Blue,
    ] {
        let d = named(&mut g, p, "Forest", Zone::Hand);
        activate_discard_color(&mut g, p, m, d, color);
    }
    assert_eq!(ability_world::CharacteristicWorld::colors(&g, m), 2);
    let red = named(&mut g, p.other(), "Shock", Zone::Hand);
    let blue = named(&mut g, p.other(), "Unsummon", Zone::Hand);
    assert!(!g.cast_target_matches(
        TargetRule::Any,
        p.other(),
        g.object_ref(red),
        Target::Card(m)
    ));
    assert!(!g.cast_target_matches(
        TargetRule::Creature,
        p.other(),
        g.object_ref(blue),
        Target::Card(m)
    ));
    assert!(g.cast_target_matches(TargetRule::Any, p, g.object_ref(red), Target::Card(m)));
    next_turn(&mut g);
    assert!(g.cast_target_matches(
        TargetRule::Any,
        p.other(),
        g.object_ref(red),
        Target::Card(m)
    ));
}
#[test]
fn tmt_damage_legends_mondo_temporary_hexproof_obeys_layer_six_removal_and_restore() {
    let p = Player::P0;
    let mut g = setup(p);
    let m = paid(&mut g, p, "Mondo Gecko", &[]);
    settle(&mut g);
    let d = named(&mut g, p, "Forest", Zone::Hand);
    activate_discard_color(&mut g, p, m, d, crate::cards::ChoiceColor::Red);
    let red = named(&mut g, p.other(), "Shock", Zone::Hand);
    assert!(!g.cast_target_matches(
        TargetRule::Any,
        p.other(),
        g.object_ref(red),
        Target::Card(m)
    ));
    let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(m)]);
    settle(&mut g);
    assert!(g.cast_target_matches(
        TargetRule::Any,
        p.other(),
        g.object_ref(red),
        Target::Card(m)
    ));
    paid(&mut g, p, "Disenchant", &[Target::Card(aura)]);
    settle(&mut g);
    assert!(!g.cast_target_matches(
        TargetRule::Any,
        p.other(),
        g.object_ref(red),
        Target::Card(m)
    ));
}
#[test]
fn tmt_damage_legends_mondo_combat_draws_distinct_current_permanent_colors() {
    let p = Player::P0;
    let mut g = setup(p);
    let m = paid(&mut g, p, "Mondo Gecko", &[]);
    settle(&mut g);
    named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    named(&mut g, p, "Buzz Bots", Zone::Battlefield);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    let before = g.state.players[p.index()].library.len();
    combat(&mut g, m);
    checked(&mut g, Action::FinishBlockers);
    until(&mut g, |g| {
        g.state.players[p.index()].library.len() == before - 2
    });
    assert_eq!(g.state.players[p.index()].library.len(), before - 2);
}
#[test]
fn tmt_damage_legends_raph_mikey_haste_reveals_first_creature_tapped_attacking() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raph & Mikey, Troublemakers", &[]);
    settle(&mut g);
    let c = named(&mut g, p, "Grizzly Bears", Zone::Library);
    let a = named(&mut g, p, "Shock", Zone::Library);
    let b = named(&mut g, p, "Forest", Zone::Library);
    combat(&mut g, r);
    assert!(g.on_battlefield(c));
    assert!(g.state.objects[c as usize].tapped);
    assert!(g.state.attackers.iter().any(|a| a.card == c));
    assert!(g.state.pending_triggers.is_empty());
    assert_eq!(g.state.objects[a as usize].zone, Zone::Library);
    assert_eq!(g.state.objects[b as usize].zone, Zone::Library);
    assert!(g.state.players[p.index()].library[..2].contains(&a));
    assert!(g.state.players[p.index()].library[..2].contains(&b));
}
#[test]
fn tmt_damage_legends_old_hob_actual_hasty_token_and_source_independent_destroy() {
    let p = Player::P0;
    let mut g = setup(p);
    let h = paid(&mut g, p, "Old Hob, Alleycat Blues", &[]);
    settle(&mut g);
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    let token = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.is_token() && o.zone == Zone::Battlefield)
        .unwrap()
        .0 as CardId;
    assert_eq!((g.stats(token).power, g.stats(token).toughness), (2, 2));
    assert!(g.keywords(token).contains(Keywords::HASTE));
    checked(&mut g, Action::Attack { card: token });
    checked(&mut g, Action::FinishAttackers);
    until(&mut g, |g| g.state.pending == Pending::Blockers);
    checked(&mut g, Action::FinishBlockers);
    g.state.players[p.index()].mana = [30; 6];
    paid(&mut g, p, "Unsummon", &[Target::Card(h)]);
    settle(&mut g);
    until(&mut g, |g| {
        g.state.step == Step::End && g.state.pending == Pending::Priority
    });
    settle(&mut g);
    assert!(!g.on_battlefield(token));
}
#[test]
fn tmt_damage_legends_old_hob_attacking_token_indestructible_survives_delayed_destroy() {
    let p = Player::P0;
    let mut g = setup(p);
    let h = paid(&mut g, p, "Old Hob, Alleycat Blues", &[]);
    settle(&mut g);
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    let token = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.is_token() && o.zone == Zone::Battlefield)
        .unwrap()
        .0 as CardId;
    checked(&mut g, Action::Attack { card: token });
    checked(&mut g, Action::FinishAttackers);
    until(&mut g, |g| g.state.pending == Pending::Blockers);
    checked(&mut g, Action::FinishBlockers);
    g.state.players[p.index()].mana = [30; 6];
    activate(&mut g, h, &[Target::Card(token)]);
    settle(&mut g);
    assert!(g.keywords(token).contains(Keywords::INDESTRUCTIBLE));
    until(&mut g, |g| {
        g.state.step == Step::End && g.state.pending == Pending::Priority
    });
    settle(&mut g);
    assert!(g.on_battlefield(token));
    next_turn(&mut g);
    assert!(g.on_battlefield(token));
    assert!(!g.keywords(token).contains(Keywords::INDESTRUCTIBLE));
}
#[test]
fn tmt_damage_legends_leatherhead_can_remove_other_kind_and_decline_without_targeting() {
    for remove in [true, false] {
        let p = Player::P0;
        let mut g = setup(p);
        let l = paid(&mut g, p, "Leatherhead, Swamp Stalker", &[]);
        settle(&mut g);
        let mut counters = g.counter_amounts(l).to_owned();
        counters.add(CounterKind::Charge, 1);
        g.set_counters(l, counters);
        let artifact = named(&mut g, p.other(), "Skateboard", Zone::Battlefield);
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        combat(&mut g, l);
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.counter_removal_domain().is_some());
        let mut too_many = Counters::of(CounterKind::Charge, 1);
        too_many.add(CounterKind::Hexproof, 1);
        reject(&mut g, Action::ChooseCounterRemoval { counters: too_many });
        if remove {
            checked(
                &mut g,
                Action::ChooseCounterRemoval {
                    counters: Counters::of(CounterKind::Charge, 1),
                },
            );
            settle_on(&mut g, Some(Target::Card(artifact)));
            assert!(!g.on_battlefield(artifact));
            assert_eq!(g.counter_count(l, CounterKind::Charge), 0);
        } else {
            checked(&mut g, Action::DeclineEffectPayment);
            settle(&mut g);
            assert!(g.on_battlefield(artifact));
            assert_eq!(g.counter_count(l, CounterKind::Charge), 1);
        }
        assert_eq!(g.counter_count(l, CounterKind::Hexproof), 1);
        assert!(g.keywords(l).contains(Keywords::HEXPROOF));
    }
}
#[test]
fn tmt_damage_legends_leatherhead_keyword_counter_removal_order_and_restore() {
    let p = Player::P0;
    let mut g = setup(p);
    let l = paid(&mut g, p, "Leatherhead, Swamp Stalker", &[]);
    settle(&mut g);
    let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(l)]);
    settle(&mut g);
    assert_eq!(g.counter_count(l, CounterKind::Hexproof), 1);
    assert!(!g.keywords(l).contains(Keywords::HEXPROOF));
    let mut counters = g.counter_amounts(l).to_owned();
    counters.add(CounterKind::Hexproof, 1);
    g.set_counters(l, counters);
    assert!(g.keywords(l).contains(Keywords::HEXPROOF));
    paid(&mut g, p, "Disenchant", &[Target::Card(aura)]);
    settle(&mut g);
    assert!(g.keywords(l).contains(Keywords::HEXPROOF));
    assert!(g.keywords(l).contains(Keywords::TRAMPLE));
}
#[test]
fn tmt_damage_legends_mondo_temporary_grants_and_colors_are_not_copiable() {
    let p = Player::P0;
    let mut g = setup(p);
    let m = paid(&mut g, p, "Mondo Gecko", &[]);
    settle(&mut g);
    let discard = named(&mut g, p, "Forest", Zone::Hand);
    activate_discard_color(&mut g, p, m, discard, crate::cards::ChoiceColor::Red);
    let rite = named(&mut g, p, "Rite of Replication", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: rite });
    checked(&mut g, Action::ChooseKicker { pay: false });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(m),
        },
    );
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::Legend { .. })
    });
    let copy = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.is_token() && o.zone == Zone::Battlefield)
        .unwrap()
        .0 as CardId;
    let object = g.object_ref(copy);
    checked(&mut g, Action::KeepLegend { object });
    settle(&mut g);
    assert_eq!(ability_world::CharacteristicWorld::colors(&g, copy), 2);
    let red = named(&mut g, p.other(), "Shock", Zone::Hand);
    assert!(g.cast_target_matches(
        TargetRule::Any,
        p.other(),
        g.object_ref(red),
        Target::Card(copy)
    ));
}
#[test]
fn tmt_damage_legends_raph_mikey_entry_attack_does_not_trigger_declared_attack_ability() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raph & Mikey, Troublemakers", &[]);
    settle(&mut g);
    let sibling = named(&mut g, p, "Raph & Leo, Sibling Rivals", Zone::Library);
    combat(&mut g, r);
    assert!(g.on_battlefield(sibling));
    assert!(g.state.attackers.iter().any(|a| a.card == sibling));
    assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 0);
}
#[test]
fn tmt_damage_legends_raph_mikey_attacking_entry_chooses_different_exact_walker() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raph & Mikey, Troublemakers", &[]);
    settle(&mut g);
    let target = named(
        &mut g,
        p.other(),
        "Liliana, Dreadhorde General",
        Zone::Battlefield,
    );
    let own = named(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
    g.set_counters(target, Counters::of(CounterKind::Loyalty, 6));
    g.set_counters(own, Counters::of(CounterKind::Loyalty, 6));
    let c = named(&mut g, p, "Grizzly Bears", Zone::Library);
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    checked(&mut g, Action::Attack { card: r });
    checked(&mut g, Action::FinishAttackers);
    let defender = CombatDefender::Planeswalker(g.object_ref(target));
    until(&mut g, |g| {
        g.legal_actions()
            .contains(&Action::ChooseHandDefender { defender })
    });
    let bad = CombatDefender::Planeswalker(g.object_ref(own));
    reject(&mut g, Action::ChooseHandDefender { defender: bad });
    checked(&mut g, Action::ChooseHandDefender { defender });
    until(&mut g, |g| g.state.pending == Pending::Blockers);
    assert_eq!(
        g.state
            .attackers
            .iter()
            .find(|a| a.card == r)
            .unwrap()
            .defender,
        Some(CombatDefender::Player(p.other()))
    );
    assert_eq!(
        g.state
            .attackers
            .iter()
            .find(|a| a.card == c)
            .unwrap()
            .defender,
        Some(defender)
    );
}
#[test]
fn tmt_damage_legends_raph_mikey_no_creature_reveals_and_randomizes_entire_library() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raph & Mikey, Troublemakers", &[]);
    settle(&mut g);
    let before = g.state.players[p.index()]
        .library
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    combat(&mut g, r);
    let after = g.state.players[p.index()]
        .library
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(before, after);
    assert_eq!(g.state.attackers.len(), 1);
    assert!(
        g.state.players[p.index()]
            .library
            .iter()
            .all(|id| !g.state.objects[*id as usize].card.is_creature())
    );
}
#[test]
fn tmt_damage_legends_mondo_actual_opponent_spell_rejects_fixed_color_target() {
    let p = Player::P0;
    let mut g = setup(p);
    let m = paid(&mut g, p, "Mondo Gecko", &[]);
    settle(&mut g);
    let d = named(&mut g, p, "Forest", Zone::Hand);
    activate_discard_color(&mut g, p, m, d, crate::cards::ChoiceColor::Red);
    let red = named(&mut g, p.other(), "Shock", Zone::Hand);
    if g.actor() == Some(p) {
        checked(&mut g, Action::Pass);
    }
    checked(&mut g, Action::AnnounceCast { card: red });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(m),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    until(&mut g, |g| g.state.announcing.is_none());
    settle(&mut g);
    assert_eq!(g.state.objects[m as usize].damage, 0);
    assert_eq!(g.state.objects[red as usize].zone, Zone::Graveyard);
}
#[test]
fn tmt_damage_legends_fixed_color_projection_uses_existing_padding() {
    assert_eq!(std::mem::size_of::<ability_order::LayerSixObject>(), 56);
    assert_eq!(std::mem::size_of::<AbilityAvailabilityView>(), 32);
    assert_eq!(std::mem::size_of::<Object>(), 144);
}
