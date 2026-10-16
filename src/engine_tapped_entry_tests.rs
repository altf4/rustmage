use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
const CARDS: [&str; 5] = [
    "Diregraf Ghoul",
    "Piranha Fly",
    "Daring Thunder-Thief",
    "Sleep-Cursed Faerie",
    "Eddymurk Crab",
];
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(expected, fork.snapshot());
    let rev = g.revision();
    let p = g.actor().unwrap_or(Player::P0);
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
}
fn cast_named(g: &mut Game, id: CardId) {
    g.state.players[g.actor().unwrap().index()].mana = [20; 6];
    cast(g, id, None);
}

#[test]
fn tapped_creature_replacements_and_initial_stun_exist_on_every_entry_path() {
    for owner in [Player::P0, Player::P1] {
        for active in [Player::P0, Player::P1] {
            for name in CARDS {
                let mut g = scenario();
                g.state.active = active;
                g.state.priority = owner;
                let id = named(&mut g, owner, name, Zone::Exile);
                let before = g.snapshot();
                let mut fork = g.clone();
                g.move_object(id, Zone::Battlefield);
                fork.move_object(id, Zone::Battlefield);
                assert_eq!(g.snapshot(), fork.snapshot());
                let expected = g.snapshot();
                let o = g.object_view(id);
                assert_eq!(o.tapped, name != "Eddymurk Crab" || owner != active);
                assert_eq!(
                    o.counters.stun,
                    if name == "Sleep-Cursed Faerie" { 3 } else { 0 }
                );
                assert_eq!(
                    (o.power, o.toughness),
                    match name {
                        "Diregraf Ghoul" => (2, 2),
                        "Piranha Fly" => (2, 1),
                        "Daring Thunder-Thief" => (4, 4),
                        "Sleep-Cursed Faerie" => (3, 3),
                        _ => (5, 5),
                    }
                );
                g.restore(&before).unwrap();
                g.move_object(id, Zone::Battlefield);
                assert_eq!(g.snapshot(), expected);
            }
        }
    }
}

#[test]
fn cast_entry_waits_for_resolution_and_flash_respects_turn_and_spell_timing() {
    for owner in [Player::P0, Player::P1] {
        for active in [Player::P0, Player::P1] {
            for name in CARDS {
                let mut g = scenario();
                g.state.active = active;
                g.state.priority = owner;
                let id = named(&mut g, owner, name, Zone::Hand);
                g.state.players[owner.index()].mana = [20; 6];
                let flash = matches!(name, "Daring Thunder-Thief" | "Eddymurk Crab");
                if owner != active && !flash {
                    reject(&mut g, Action::AnnounceCast { card: id });
                    continue;
                }
                cast(&mut g, id, None);
                assert_eq!(g.object_view(id).zone, Zone::Stack);
                assert!(!g.object_view(id).tapped);
                assert_eq!(g.object_view(id).counters.stun, 0);
                replay(&mut g, Action::Pass);
                replay(&mut g, Action::Pass);
                let o = g.object_view(id);
                assert_eq!(o.zone, Zone::Battlefield);
                assert_eq!(o.tapped, name != "Eddymurk Crab" || owner != active);
                assert_eq!(
                    o.counters.stun,
                    if name == "Sleep-Cursed Faerie" { 3 } else { 0 }
                );
            }
        }
    }
}

#[test]
fn stun_replaces_each_natural_or_paid_untap_then_new_incarnation_restores_initial_counters() {
    for paid in [false, true] {
        let mut g = scenario();
        let id = named(&mut g, Player::P0, "Sleep-Cursed Faerie", Zone::Hand);
        cast_named(&mut g, id);
        pass_pair(&mut g);
        assert!(g.keywords(id).contains(Keywords::FLYING));
        assert_eq!(
            g.state.objects[id as usize].card.triggers()[0].condition,
            TriggerCondition::Ward
        );
        for (step, remaining) in [2, 1, 0, 0].into_iter().enumerate() {
            if paid {
                g.state.players[0].mana = [20; 6];
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a,Action::Activate{card,..}if *card==id))
                    .unwrap();
                replay(&mut g, a);
                replay(&mut g, Action::Pass);
                replay(&mut g, Action::Pass);
            } else {
                g.state.active = Player::P0;
                g.begin_turn();
            }
            let o = g.object_view(id);
            assert_eq!(o.counters.stun, remaining);
            // Three replacements remove counters while retaining tapped; fourth untaps.
            assert_eq!(o.tapped, step < 3);
        }
        assert!(!g.object_view(id).tapped);
        g.state.active = Player::P0;
        g.state.step = Step::Main1;
        g.give_priority();
        let old = g.object_ref(id);
        let bounce = named(&mut g, Player::P0, "Unsummon", Zone::Hand);
        g.state.players[0].mana = [20; 6];
        cast(&mut g, bounce, Some(Target::Card(id)));
        pass_pair(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Hand);
        assert_eq!(g.object_view(id).counters.stun, 0);
        cast_named(&mut g, id);
        pass_pair(&mut g);
        assert_ne!(old, g.object_ref(id));
        assert!(g.object_view(id).tapped);
        assert_eq!(g.object_view(id).counters.stun, 3);
    }
}

#[test]
fn crab_locks_exact_graveyard_reduction_and_etb_has_zero_to_two_distinct_targets() {
    for count in [0, 1, 2] {
        let mut g = scenario();
        let crab = named(&mut g, Player::P0, "Eddymurk Crab", Zone::Hand);
        for name in ["Shock", "Divination", "Unsummon"] {
            named(&mut g, Player::P0, name, Zone::Graveyard);
        }
        named(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
        named(&mut g, Player::P1, "Shock", Zone::Graveyard);
        let a = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let b = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let land = named(&mut g, Player::P1, "Forest", Zone::Battlefield);
        g.state.players[0].mana = [0, 2, 0, 0, 0, 2];
        replay(&mut g, Action::AnnounceCast { card: crab });
        assert!(
            g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::PayCosts{payment}if *payment==[0,2,0,0,0,2]))
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, pay);
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert!(!g.object_view(crab).tapped);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        if count > 0 {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
        }
        if count > 1 {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(b),
                },
            );
        }
        if g.state.announcing.is_some() {
            replay(&mut g, Action::FinishTargets);
        }
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert_eq!(g.object_view(a).tapped, count > 0);
        assert_eq!(g.object_view(b).tapped, count > 1);
    }
}

#[test]
fn countered_tapped_creature_spell_never_receives_entry_state() {
    let mut g = scenario();
    let faerie = named(&mut g, Player::P0, "Sleep-Cursed Faerie", Zone::Hand);
    cast_named(&mut g, faerie);
    replay(&mut g, Action::Pass);
    let counter = named(&mut g, Player::P1, "Counterspell", Zone::Hand);
    g.state.players[1].mana = [20; 6];
    cast(&mut g, counter, Some(Target::Spell(faerie)));
    pass_pair(&mut g);
    let o = g.object_view(faerie);
    assert_eq!(o.zone, Zone::Graveyard);
    assert!(!o.tapped);
    assert_eq!(o.counters.stun, 0);
}

#[test]
fn real_noncasting_recovery_and_simultaneous_batches_apply_entry_replacements() {
    for owner in [Player::P0, Player::P1] {
        let mut g = scenario();
        g.state.active = owner;
        g.state.priority = owner;
        let faerie = named(&mut g, owner, "Sleep-Cursed Faerie", Zone::Graveyard);
        let spell = named(&mut g, owner, "Zombify", Zone::Hand);
        g.state.players[owner.index()].mana = [20; 6];
        cast(&mut g, spell, Some(Target::Card(faerie)));
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert_eq!(g.object_view(faerie).zone, Zone::Battlefield);
        assert!(g.object_view(faerie).tapped);
        assert_eq!(g.object_view(faerie).counters.stun, 3);
    }
    let mut g = scenario();
    let f = named(&mut g, Player::P0, "Sleep-Cursed Faerie", Zone::Exile);
    let own = named(&mut g, Player::P0, "Eddymurk Crab", Zone::Graveyard);
    let enemy = named(&mut g, Player::P1, "Eddymurk Crab", Zone::Exile);
    let before = g.snapshot();
    let mut fork = g.clone();
    let events = g.move_objects_raw(&[f, own, enemy], Zone::Battlefield);
    g.finish_zone_events(&events, &[]);
    let events = fork.move_objects_raw(&[f, own, enemy], Zone::Battlefield);
    fork.finish_zone_events(&events, &[]);
    assert_eq!(g.snapshot(), fork.snapshot());
    assert!(g.object_view(f).tapped);
    assert_eq!(g.object_view(f).counters.stun, 3);
    assert!(!g.object_view(own).tapped);
    assert!(g.object_view(enemy).tapped);
    assert_eq!(
        g.state
            .pending_triggers
            .iter()
            .filter(|t| t.card == own || t.card == enemy)
            .count(),
        2
    );
    g.restore(&before).unwrap();
    assert_eq!(g.object_view(f).zone, Zone::Exile);
}
