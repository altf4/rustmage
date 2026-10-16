use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
const NAMES: [&str; 16] = [
    "Adorned Crocodile",
    "Agent of Kotis",
    "Beetle, Legacy Criminal",
    "Bonebind Orator",
    "Cryotheory Adept",
    "Eternal Student",
    "Faerie Dreamthief",
    "Goldmeadow Nomad",
    "Gravestone Strider",
    "Leering Onlooker",
    "Rubblebelt Maverick",
    "Stoic Grove-Guide",
    "Stone Docent",
    "Suspicious Shambler",
    "Theoretical Necromancer",
    "Venom, Evil Unleashed",
];
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn slot(g: &Game, id: CardId) -> u16 {
    g.object_view(id)
        .card
        .activations()
        .iter()
        .position(|a| a.origin == ActivationOrigin::Graveyard)
        .unwrap() as u16
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let p = g.actor().unwrap();
    let rev = g.revision();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(f.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), rev);
}
fn activation(g: &Game, id: CardId, target: Option<Target>) -> Action {
    let index = slot(g, id);
    g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,ability,target:t,..}if *card==id&&*ability==index&&*t==target)).expect("graveyard activation available")
}
fn finish_library(g: &mut Game, p: Player) {
    while let Some(c) = g.observe(p).library_choice {
        assert!(g.observe(p.other()).library_choice.is_none());
        assert!(g.observe(p.other()).actions.is_empty());
        replay(
            g,
            Action::ArrangeLibrary {
                card: c.remaining[0].id,
                destination: LibraryDestination::Top,
            },
        );
    }
}
#[test]
fn sixteen_graveyard_abilities_pay_exact_source_before_resolving_complete_bodies() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Graveyard);
            let old = g.object_ref(id);
            let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let grave = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
            let foreign = named(&mut g, p.other(), "Island", Zone::Graveyard);
            let chosen = match i {
                0 | 1 | 2 | 4 | 10 | 15 => Some(Target::Card(target)),
                3 | 14 => Some(Target::Card(grave)),
                8 => Some(Target::Card(foreign)),
                _ => None,
            };
            let a = activation(&g, id, chosen);
            let Action::Activate { payment, .. } = a else {
                unreachable!()
            };
            let life = g.state.players[p.index()].life;
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            let deaths = g.state.creatures_died_this_turn;
            replay(&mut g, a);
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            assert_ne!(g.object_ref(id), old);
            assert_eq!(
                g.state.players[p.index()].mana,
                std::array::from_fn(|n| 20 - payment[n])
            );
            assert_eq!(g.state.creatures_died_this_turn, deaths);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn);
            assert_eq!(g.object_view(target).counters.plus_one, 0);
            assert_eq!(g.state.stack.len(), 1);
            assert_eq!(g.state.stack[0].object, old);
            assert!(!g.state.stack[0].source_facts.battlefield);
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
            pass_pair(&mut g);
            finish_library(&mut g, p);
            assert!(g.state.stack.is_empty());
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            match i {
                0 | 2 | 10 => assert_eq!(g.object_view(target).counters.plus_one, 1),
                1 | 15 => assert_eq!(g.object_view(target).counters.plus_one, 2),
                3 | 14 => assert_eq!(g.object_view(grave).zone, Zone::Hand),
                4 => {
                    assert!(g.object_view(target).tapped);
                    assert_eq!(g.object_view(target).counters.stun, 1);
                }
                6 => {
                    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 1);
                    assert_eq!(g.state.players[p.index()].life, life - 1);
                }
                8 => assert_eq!(g.object_view(foreign).zone, Zone::Exile),
                12 => assert_eq!(g.state.players[p.index()].life, life + 2),
                5 | 7 | 9 | 11 | 13 => {
                    let (subtype, count, power, toughness, colors, tapped, flying) = match i {
                        5 => ("Inkling", 2, 1, 1, 5, false, true),
                        7 => ("Kithkin", 1, 1, 1, 17, false, false),
                        9 => ("Bat", 2, 1, 1, 4, true, true),
                        11 => ("Elf", 1, 2, 2, 20, false, false),
                        _ => ("Zombie", 2, 2, 2, 4, false, false),
                    };
                    let tokens: Vec<_> = g
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(_, o)| o.zone == Zone::Battlefield && o.card.is_token())
                        .map(|(id, _)| g.object_view(id as CardId))
                        .collect();
                    assert_eq!(tokens.len(), count);
                    for t in tokens {
                        assert!(t.card.has_subtype(subtype));
                        assert_eq!(t.card.colors(), colors);
                        assert_eq!((t.power, t.toughness), (power, toughness));
                        assert_eq!(t.tapped, tapped);
                        assert_eq!(t.keywords.contains(Keywords::FLYING), flying);
                        assert_eq!(t.owner, p);
                    }
                }
                _ => {}
            }
            if i == 2 {
                assert!(g.keywords(target).contains(Keywords::FLYING));
            }
            if i == 15 {
                assert!(g.keywords(target).contains(Keywords::DEATHTOUCH));
            }
            assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,..}|Action::AnnounceActivate{card,..}if *card==id)));
        }
    }
}
#[test]
fn graveyard_activation_legality_rejects_foreign_wrong_zone_timing_and_targets() {
    for p in [Player::P0, Player::P1] {
        for name in NAMES {
            for zone in [
                Zone::Hand,
                Zone::Battlefield,
                Zone::Exile,
                Zone::Library,
                Zone::Graveyard,
            ] {
                let mut g = setup(p);
                let id = named(
                    &mut g,
                    if zone == Zone::Graveyard {
                        p.other()
                    } else {
                        p
                    },
                    name,
                    zone,
                );
                named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
                let index = slot(&g, id);
                assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::AnnounceActivate{card,ability,..}|Action::Activate{card,ability,..}|Action::BeginActivate{card,ability,..}if *card==id&&*ability==index)),"{name} {zone:?}");
                reject(
                    &mut g,
                    Action::AnnounceActivate {
                        card: id,
                        ability: index,
                        counter_payment: Counters::default(),
                    },
                );
            }
        }
    }
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Graveyard);
            named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
            g.state.active = p.other();
            g.state.step = Step::BeginCombat;
            let allowed = matches!(i, 3 | 5 | 6 | 8 | 9 | 14);
            assert_eq!(g.legal_actions().iter().any(|a|matches!(a,Action::AnnounceActivate{card,ability,..}if *card==id&&*ability==slot(&g,id))),allowed,"{name}");
        }
    }
}
#[test]
fn staged_targets_and_funding_keep_source_in_graveyard_until_cost_commit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Agent of Kotis", Zone::Graveyard);
        let target = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let land = named(&mut g, p, "Island", Zone::Battlefield);
        for _ in 0..3 {
            named(&mut g, p, "Forest", Zone::Battlefield);
        }
        g.state.players[p.index()].mana = [0; 6];
        let index = slot(&g, id);
        let original = g.object_ref(id);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: index,
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(g.object_ref(id), original);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert!(g.observe(p.other()).actions.is_empty());
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        assert_eq!(g.observe(p).decision, "pay_costs");
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        replay(&mut g, Action::TapMana { card: land });
        while !g
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::PayCosts { .. }))
        {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::TapMana { .. }))
                .unwrap();
            replay(&mut g, a);
            assert_eq!(g.object_ref(id), original);
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        assert_eq!(g.state.stack[0].object, original);
        pass_pair(&mut g);
        assert_eq!(g.object_view(target).counters.plus_one, 2);
    }
}
#[test]
fn source_cost_obligation_requires_exact_graveyard_incarnation_during_owned_payment() {
    for returns in [false, true] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, "Agent of Kotis", Zone::Graveyard);
        let target = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let index = slot(&g, id);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: index,
                counter_payment: Counters::default(),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        let original = g.object_ref(id);
        g.move_object(id, Zone::Exile);
        if returns {
            g.move_object(id, Zone::Graveyard);
        }
        assert_ne!(g.object_ref(id), original);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayCosts { .. }))
        );
        reject(
            &mut g,
            Action::PayCosts {
                payment: [0, 4, 0, 0, 0, 0],
            },
        );
    }
}
#[test]
fn graveyard_exile_cost_capacity_failure_rolls_back_all_resources_and_revision() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Faerie Dreamthief", Zone::Graveyard);
        g.state.objects[id as usize].generation = u64::MAX;
        let action = activation(&g, id, None);
        let saved = g.snapshot();
        let rev = g.revision();
        for _ in 0..2 {
            assert!(matches!(
                g.apply(p, action.clone()),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), rev);
        }
        let mut fork = g.clone();
        assert!(matches!(
            fork.apply(p, action),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(fork.snapshot(), saved);
    }
}
#[test]
fn another_graveyard_creature_filter_excludes_self_and_other_owners_and_kinds() {
    for name in ["Bonebind Orator", "Theoretical Necromancer"] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, name, Zone::Graveyard);
        let index = slot(&g, id);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==id))
        );
        let valid = named(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
        let enemy = named(&mut g, Player::P1, "Grizzly Bears", Zone::Graveyard);
        let land = named(&mut g, Player::P0, "Forest", Zone::Graveyard);
        let live = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: index,
                counter_payment: Counters::default(),
            },
        );
        for wrong in [id, enemy, land, live] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(wrong),
                },
            );
        }
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(valid),
            },
        );
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, a);
        pass_pair(&mut g);
        assert_eq!(g.object_view(valid).zone, Zone::Hand);
    }
}
#[test]
fn gravestone_can_target_itself_but_source_exile_invalidates_that_target() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Gravestone Strider", Zone::Graveyard);
    let a = activation(&g, id, Some(Target::Card(id)));
    replay(&mut g, a);
    assert_eq!(g.object_view(id).zone, Zone::Exile);
    pass_pair(&mut g);
    assert!(g.state.stack.is_empty());
    assert_eq!(g.object_view(id).zone, Zone::Exile);
}
#[test]
fn renewed_target_ward_counters_independent_ability_without_refunding_exile_cost() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Agent of Kotis", Zone::Graveyard);
            let target = named(&mut g, p.other(), "Hulking Raptor", Zone::Battlefield);
            let a = activation(&g, id, Some(Target::Card(target)));
            replay(&mut g, a);
            assert_eq!(g.state.stack.len(), 2);
            let related = g.state.stack[1].related_stack.unwrap();
            assert_eq!(related.id, g.state.stack[0].id);
            assert!(matches!(
                g.state.stack[0].kind,
                StackKind::ActivatedAbility { .. }
            ));
            pass_pair(&mut g);
            let a = if paid {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                    .unwrap()
            } else {
                Action::DeclineResolutionMana
            };
            replay(&mut g, a);
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            assert_eq!(g.state.stack.len(), usize::from(paid));
            if paid {
                pass_pair(&mut g);
            }
            assert_eq!(
                g.object_view(target).counters.plus_one,
                if paid { 2 } else { 0 }
            );
        }
    }
}
#[test]
fn graveyard_sorcery_restrictions_check_phase_and_empty_stack_independently() {
    for mode in 0..3 {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(Player::P0);
            let id = named(&mut g, Player::P0, name, Zone::Graveyard);
            named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
            named(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
            match mode {
                0 => g.state.step = Step::BeginCombat,
                1 => g.state.active = Player::P1,
                _ => {
                    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
                    cast(&mut g, bear, None);
                }
            }
            let expected = matches!(i, 3 | 5 | 6 | 8 | 9 | 14);
            assert_eq!(g.legal_actions().iter().any(|a|matches!(a,Action::AnnounceActivate{card,ability,..}if *card==id&&*ability==slot(&g,id))),expected,"{name} mode {mode}");
        }
    }
}
#[test]
fn unfunded_graveyard_sources_cannot_announce_or_use_battlefield_mana_conversion() {
    for name in NAMES {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, name, Zone::Graveyard);
        named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        named(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
        g.state.players[0].mana = [0; 6];
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::AnnounceActivate{card,..}|Action::Activate{card,..}|Action::AnnounceMana{card,..}|Action::ActivateMana{card,..}if *card==id)));
        let index = slot(&g, id);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: index,
                counter_payment: Counters::default(),
            },
        );
    }
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Gravestone Strider", Zone::Battlefield);
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::ActivateMana{card,ability:0,..}if *card==id))
        .unwrap();
    replay(&mut g, a);
    assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana{card,ability:0,..}|Action::AnnounceMana{card,ability:0,..}if *card==id)));
    g.move_object(id, Zone::Graveyard);
    assert!(
        g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceActivate{card,ability:1,..}if *card==id))
    );
    assert!(!g.legal_actions().iter().any(
        |a| matches!(a,Action::ActivateMana{card,..}|Action::AnnounceMana{card,..}if *card==id)
    ));
}
#[test]
fn graveyard_hybrid_cost_is_committed_owned_and_paid_in_either_color() {
    for p in [Player::P0, Player::P1] {
        for color in [2, 4] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Stoic Grove-Guide", Zone::Graveyard);
            let old = g.object_ref(id);
            let index = slot(&g, id);
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: index,
                    counter_payment: Counters::default(),
                },
            );
            let o = g.observe(p);
            assert_eq!(o.decision, "activation_cost");
            assert!(g.observe(p.other()).cost_choices.is_empty());
            assert_eq!(g.object_ref(id), old);
            let c = o
                .cost_choices
                .iter()
                .position(|c| c.cost.colored[color] == 1)
                .unwrap();
            replay(&mut g, Action::ChooseActivationCost { cost: c as u16 });
            let mut wrong = [0; 6];
            wrong[if color == 2 { 4 } else { 2 }] = 2;
            reject(&mut g, Action::PayCosts { payment: wrong });
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            replay(&mut g, a);
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            pass_pair(&mut g);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield
                        && o.card.is_token()
                        && o.card.has_subtype("Elf"))
                    .count(),
                1
            );
        }
    }
}
#[test]
fn actual_creature_cast_entry_and_death_bodies_survive_graveyard_exile_extension() {
    for (name, n) in [("Faerie Dreamthief", 1), ("Rubblebelt Maverick", 2)] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, name, Zone::Hand);
        cast(&mut g, id, None);
        pass_pair(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        pass_pair(&mut g);
        let c = g.observe(Player::P0).library_choice.unwrap();
        assert_eq!(c.remaining.len(), n);
        assert!(g.observe(Player::P1).library_choice.is_none());
        finish_library(&mut g, Player::P0);
    }
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Adorned Crocodile", Zone::Hand);
    cast(&mut g, id, None);
    pass_pair(&mut g);
    assert_eq!(
        (g.object_view(id).power, g.object_view(id).toughness),
        (5, 3)
    );
    let murder = named(&mut g, Player::P0, "Murder", Zone::Hand);
    cast(&mut g, murder, Some(Target::Card(id)));
    pass_pair(&mut g);
    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    pass_pair(&mut g);
    let zombies: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.card.is_token())
        .map(|(id, _)| g.object_view(id as CardId))
        .collect();
    assert_eq!(zombies.len(), 1);
    let token = &zombies[0];
    assert!(token.card.has_subtype("Zombie") && token.card.has_subtype("Druid"));
    assert_eq!(
        (token.power, token.toughness, token.card.colors()),
        (2, 2, 4)
    );
    let a = activation(&g, id, Some(Target::Card(token.id)));
    replay(&mut g, a);
    pass_pair(&mut g);
    assert_eq!(g.object_view(token.id).counters.plus_one, 1);
    assert_eq!(g.state.creatures_died_this_turn, 1);
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Cryotheory Adept", Zone::Hand);
    cast(&mut g, id, None);
    pass_pair(&mut g);
    let shock = named(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Player(Player::P1)));
    pass_pair(&mut g);
    assert_eq!(
        (g.object_view(id).power, g.object_view(id).toughness),
        (3, 2)
    );
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 18);
}
#[test]
fn effect_capacity_rollback_keeps_paid_exile_cost_and_owned_private_surveil() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Stone Docent", Zone::Graveyard);
    let a = activation(&g, id, None);
    replay(&mut g, a);
    g.state.players[0].life = i128::MAX;
    replay(&mut g, Action::Pass);
    let s = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(p, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), s);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
    }
    g.state.players[0].life = 20;
    replay(&mut g, Action::Pass);
    assert_eq!(g.state.players[0].life, 22);
    assert!(g.observe(Player::P0).library_choice.is_some());
    finish_library(&mut g, Player::P0);
    assert_eq!(g.object_view(id).zone, Zone::Exile);
}
#[test]
fn graveyard_ability_keeps_bound_target_generation_and_remains_independent_of_source() {
    for name in [
        "Agent of Kotis",
        "Beetle, Legacy Criminal",
        "Cryotheory Adept",
        "Venom, Evil Unleashed",
    ] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, name, Zone::Graveyard);
        let target = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let old = g.object_ref(target);
        let a = activation(&g, id, Some(Target::Card(target)));
        replay(&mut g, a);
        g.move_object(target, Zone::Hand);
        g.move_object(target, Zone::Battlefield);
        assert_ne!(g.object_ref(target), old);
        pass_pair(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        assert_eq!(g.object_view(target).counters, Counters::default());
        assert!(!g.object_view(target).tapped);
        assert_eq!(g.keywords(target), Keywords::NONE);
    }
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Faerie Dreamthief", Zone::Graveyard);
    let a = activation(&g, id, None);
    replay(&mut g, a);
    let first = g.state.stack[0].clone();
    g.move_object(id, Zone::Graveyard);
    let second_source = g.object_ref(id);
    let a = activation(&g, id, None);
    replay(&mut g, a);
    assert_eq!(g.state.stack.len(), 2);
    assert_eq!(g.state.stack[0], first);
    assert_eq!(g.state.stack[1].object, second_source);
    assert_ne!(g.state.stack[0].id, g.state.stack[1].id);
    assert_ne!(g.state.stack[0].object, g.state.stack[1].object);
    pass_pair(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.object_view(id).zone, Zone::Exile);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(g.state.players[0].life, 18);
}
#[test]
fn temporary_granted_keywords_expire_while_renew_growth_counters_persist() {
    for (name, keyword, amount) in [
        ("Beetle, Legacy Criminal", Keywords::FLYING, 1),
        ("Venom, Evil Unleashed", Keywords::DEATHTOUCH, 2),
    ] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, name, Zone::Graveyard);
        let target = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let a = activation(&g, id, Some(Target::Card(target)));
        replay(&mut g, a);
        pass_pair(&mut g);
        assert!(g.keywords(target).contains(keyword));
        assert_eq!(g.object_view(target).counters.plus_one, amount);
        g.finish_cleanup();
        assert!(!g.keywords(target).contains(keyword));
        assert_eq!(g.object_view(target).counters.plus_one, amount);
        assert_eq!(g.object_view(target).power, 2 + i128::from(amount));
        assert_eq!(g.object_view(id).zone, Zone::Exile);
    }
}
