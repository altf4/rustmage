use super::tests::{act, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn offered(g: &Game, id: CardId, index: u16) -> bool {
    g.legal_actions()
        .iter()
        .any(|a| matches!(a,Action::AnnounceActivate{card,ability,..}if *card==id&&*ability==index))
}
fn announce(g: &mut Game, id: CardId, index: u16) {
    replay(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: index,
            counter_payment: Counters::default(),
        },
    );
}
fn target(g: &mut Game, id: CardId) {
    replay(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
#[test]
fn own_turn_allows_priority_in_combat_and_with_stack_but_rejects_inactive_turn() {
    for (name, index) in [
        ("Professor Zei, Anthropologist", 1),
        ("Rescue Girl, First Responder", 0),
    ] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let grave = named(&mut g, Player::P0, "Shock", Zone::Graveyard);
        let land = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
        g.state.players[0].mana = [0, 0, 0, 2, 0, 0];
        let extra = named(&mut g, Player::P0, "Shock", Zone::Hand);
        replay(
            &mut g,
            Action::Cast {
                card: extra,
                target: Some(Target::Player(Player::P1)),
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        assert!(!g.state.stack.is_empty());
        for step in [
            Step::Upkeep,
            Step::Draw,
            Step::Main1,
            Step::BeginCombat,
            Step::Blockers,
            Step::CombatDamage,
            Step::Main2,
            Step::End,
        ] {
            g.state.step = step;
            assert!(offered(&g, source, index), "{name} {step:?}");
            g.state.active = Player::P1;
            assert!(!offered(&g, source, index));
            let direct = Action::Activate {
                card: source,
                ability: index,
                target: Some(Target::Card(if index == 1 { grave } else { land })),
                payment: if index == 1 {
                    [0, 0, 0, 1, 0, 0]
                } else {
                    [0; 6]
                },
                counter_payment: Counters::default(),
            };
            reject(&mut g, direct);
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: index,
                    counter_payment: Counters::default(),
                },
            );
            g.state.active = Player::P0;
        }
        let s = g.snapshot();
        announce(&mut g, source, index);
        assert_eq!(g.observe(Player::P0).decision, "targets");
        g.restore(&s).unwrap();
    }
}
#[test]
fn rescue_targets_another_controlled_permanent_including_lands_and_rejects_other_zones() {
    let mut g = scenario();
    let source = named(
        &mut g,
        Player::P0,
        "Rescue Girl, First Responder",
        Zone::Battlefield,
    );
    let mut good: Vec<_> = [
        "Forest",
        "Murmuring Volume",
        "Grizzly Bears",
        "Banishing Light",
        "Professor Dellian Fel",
    ]
    .into_iter()
    .map(|n| named(&mut g, Player::P0, n, Zone::Battlefield))
    .collect();
    g.state.objects[good[4] as usize].counters.loyalty = 5;
    let token = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.name == "Treasure Token"))
        .unwrap();
    good.push(spawn(&mut g, Player::P0, token, Zone::Battlefield));
    let enemy = named(&mut g, Player::P1, "Forest", Zone::Battlefield);
    let hand = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let grave = named(&mut g, Player::P0, "Shock", Zone::Graveyard);
    announce(&mut g, source, 0);
    for bad in [source, enemy, hand, grave] {
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bad),
            },
        );
    }
    let s = g.snapshot();
    for id in good {
        g.restore(&s).unwrap();
        target(&mut g, id);
        pay(&mut g);
        assert!(g.object_view(source).tapped);
        passes(&mut g);
        assert_eq!(
            g.object_view(id).zone,
            if g.object_view(id).card.is_token() {
                Zone::Outside
            } else {
                Zone::Hand
            }
        );
    }
}
#[test]
fn tap_sickness_and_haste_are_preserved_while_zei_loot_is_available_on_opponents_turn() {
    let mut g = scenario();
    let rescue = named(
        &mut g,
        Player::P0,
        "Rescue Girl, First Responder",
        Zone::Battlefield,
    );
    named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    g.state.objects[rescue as usize].summoning_sick = true;
    assert!(!offered(&g, rescue, 0));
    g.state.objects[rescue as usize].gained_keywords = Keywords::HASTE;
    assert!(offered(&g, rescue, 0));
    g.state.objects[rescue as usize].tapped = true;
    assert!(!offered(&g, rescue, 0));
    let zei = named(
        &mut g,
        Player::P0,
        "Professor Zei, Anthropologist",
        Zone::Battlefield,
    );
    let discard = named(&mut g, Player::P0, "Forest", Zone::Hand);
    named(&mut g, Player::P0, "Shock", Zone::Graveyard);
    g.state.players[0].mana = [0, 0, 0, 1, 0, 0];
    g.state.active = Player::P1;
    assert!(offered(&g, zei, 0));
    assert!(!offered(&g, zei, 1));
    announce(&mut g, zei, 0);
    let object = g.object_ref(discard);
    replay(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    passes(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 1);
}
#[test]
fn zei_recovers_only_own_instant_or_sorcery_and_sacrifices_source_once() {
    for name in ["Shock", "Divination"] {
        let mut g = scenario();
        let source = named(
            &mut g,
            Player::P0,
            "Professor Zei, Anthropologist",
            Zone::Battlefield,
        );
        let spell = named(&mut g, Player::P0, name, Zone::Graveyard);
        let creature = named(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
        let opp = named(&mut g, Player::P1, "Shock", Zone::Graveyard);
        let exile = named(&mut g, Player::P0, "Shock", Zone::Exile);
        let hand = named(&mut g, Player::P0, "Shock", Zone::Hand);
        g.state.players[0].mana = [0, 0, 0, 1, 0, 0];
        announce(&mut g, source, 1);
        for bad in [source, creature, opp, exile, hand] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
        }
        target(&mut g, spell);
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        passes(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        assert_eq!(g.state.creatures_died_this_turn, 1);
    }
}
#[test]
fn stale_recovery_target_fizzles_without_refunding_and_returned_land_choices_resume_ability() {
    let mut g = scenario();
    let source = named(
        &mut g,
        Player::P0,
        "Professor Zei, Anthropologist",
        Zone::Battlefield,
    );
    let spell = named(&mut g, Player::P0, "Shock", Zone::Graveyard);
    g.state.players[0].mana = [0, 0, 0, 1, 0, 0];
    let held = named(&mut g, Player::P1, "Temple Garden", Zone::Exile);
    // Future link IR; Zei itself does not create exile links.
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(source),
        exiled: g.object_ref(held),
        return_zone: Zone::Battlefield,
    });
    announce(&mut g, source, 1);
    target(&mut g, spell);
    pay(&mut g);
    assert_eq!(g.observe(Player::P1).decision, "entry_life");
    assert!(g.state.stack.is_empty());
    assert!(g.observe(Player::P1).payment_request.is_none());
    replay(&mut g, Action::ChooseEntryLife { pay: false });
    assert_eq!(g.state.stack.len(), 1);
    assert!(g.state.announcing.is_none());
    let old = g.object_ref(spell);
    g.move_object(spell, Zone::Hand);
    g.move_object(spell, Zone::Graveyard);
    assert_ne!(old, g.object_ref(spell));
    passes(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}
#[test]
fn typed_recovery_entry_bodies_and_dowser_prowess_are_complete() {
    for name in ["Shipwreck Dowser", "Zealous Lorecaster"] {
        let mut g = scenario();
        let instant = named(&mut g, Player::P0, "Shock", Zone::Graveyard);
        let source = named(&mut g, Player::P0, name, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        g.prepare_triggers();
        target(&mut g, instant);
        passes(&mut g);
        assert_eq!(g.object_view(instant).zone, Zone::Hand);
        if name == "Shipwreck Dowser" {
            g.state.players[0].mana = [0, 0, 0, 1, 0, 0];
            replay(
                &mut g,
                Action::Cast {
                    card: instant,
                    target: Some(Target::Player(Player::P1)),
                    payment: [0, 0, 0, 1, 0, 0],
                },
            );
            assert!(matches!(
                g.state.stack.last().unwrap().kind,
                StackKind::TriggeredAbility { .. }
            ));
            passes(&mut g);
            assert_eq!(
                g.stats(source),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
            passes(&mut g);
            g.finish_cleanup();
            assert_eq!(
                g.stats(source),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
        } else {
            assert_eq!(
                g.stats(source),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
        }
    }
}
