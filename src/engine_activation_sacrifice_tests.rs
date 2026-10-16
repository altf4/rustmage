use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 19] = [
    "Acolyte of Aclazotz",
    "Ahriman",
    "Cauldron of Essence",
    "Fountainport",
    "Hardened Tactician",
    "Hungry Ghoul",
    "Kingpin's Enforcers",
    "Legion Extruder",
    "Marwyn, the Clearcutter",
    "Ripchain Razorkin",
    "Seismic Monstrosaur",
    "Slagdrill Scrapper",
    "Sweettooth Witch",
    "Umbral Collar Zealot",
    "Unburied Earthcarver",
    "Unrooted Ancestor",
    "Vampiric Rites",
    "Vanguard of the Rose",
    "Vito's Inquisitor",
];
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn token(subtype: &str) -> Card {
    Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == subtype))
        .unwrap()
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(expected, fork.snapshot());
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
fn announce(g: &mut Game, id: CardId, ability: u16) {
    replay(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn choose(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    replay(g, Action::ChooseAdditionalCost { object });
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
fn settle(g: &mut Game) {
    for _ in 0..100 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.ordering_triggers.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let a = if let Some(choice) = g.observe(g.actor().unwrap()).library_choice {
            let card = choice.remaining[0].id;
            Action::ArrangeLibrary {
                card,
                destination: LibraryDestination::Top,
            }
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::Pass | Action::FinishTargets
                    )
                })
                .unwrap()
        };
        replay(g, a);
    }
    panic!("unsettled resolution");
}
fn external_index(g: &Game, id: CardId) -> u16 {
    g.state.objects[id as usize]
        .card
        .activations()
        .iter()
        .position(|a| {
            a.cost
                .additional
                .is_some_and(|s| s.kind == EffectChoiceKind::Sacrifice)
        })
        .unwrap() as u16
}

#[test]
fn all_nineteen_sacrifice_bodies_choose_and_pay_before_effects_with_owned_replay() {
    for p in [Player::P0, Player::P1] {
        for name in NAMES {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let source = named(&mut g, p, name, Zone::Battlefield);
            let index = external_index(&g, source);
            let resource = match name {
                "Fountainport" | "Hardened Tactician" => {
                    spawn(&mut g, p, token("Treasure"), Zone::Battlefield)
                }
                "Legion Extruder" | "Sweettooth Witch" => {
                    spawn(&mut g, p, token("Food"), Zone::Battlefield)
                }
                "Marwyn, the Clearcutter"
                | "Slagdrill Scrapper"
                | "Ripchain Razorkin"
                | "Seismic Monstrosaur" => named(&mut g, p, "Forest", Zone::Battlefield),
                _ => named(&mut g, p, "Grizzly Bears", Zone::Battlefield),
            };
            let recovered = named(&mut g, p, "Grizzly Bears", Zone::Graveyard);
            g.state.players[p.index()].mana = [50; 6];
            announce(&mut g, source, index);
            if name == "Cauldron of Essence" {
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(recovered),
                    },
                );
            }
            if name == "Sweettooth Witch" {
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
            }
            assert!(
                matches!(g.state.pending, Pending::AdditionalCost { .. }),
                "{name}"
            );
            assert!(g.observe(p.other()).additional_cost.is_none());
            assert_eq!(g.object_view(resource).zone, Zone::Battlefield);
            choose(&mut g, resource);
            assert_eq!(g.object_view(resource).zone, Zone::Battlefield);
            assert!(!g.object_view(source).tapped);
            let library = g.state.players[p.index()].library.len();
            pay(&mut g);
            assert_ne!(g.object_view(resource).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].library.len(), library);
            assert_eq!(g.object_view(source).counters.plus_one, 0);
            assert_eq!(
                g.object_view(source).tapped,
                g.state.objects[source as usize].card.activations()[index as usize]
                    .cost
                    .tap
            );
            settle(&mut g);
            match name {
                "Acolyte of Aclazotz" | "Cauldron of Essence" => {
                    assert_eq!(g.state.players[p.index()].life, 21);
                    assert_eq!(g.state.players[p.other().index()].life, 19);
                    if name == "Cauldron of Essence" {
                        assert_eq!(g.object_view(recovered).zone, Zone::Battlefield);
                    }
                }
                "Hungry Ghoul" | "Unburied Earthcarver" | "Vito's Inquisitor" => {
                    assert_eq!(g.object_view(source).counters.plus_one, 1);
                    if name == "Vito's Inquisitor" {
                        assert!(g.keywords(source).contains(Keywords::MENACE));
                    }
                }
                "Vanguard of the Rose" | "Unrooted Ancestor" => {
                    assert!(g.object_view(source).tapped);
                    assert!(g.keywords(source).contains(Keywords::INDESTRUCTIBLE));
                }
                "Legion Extruder" => {
                    let t = g
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .find(|(_, o)| {
                            o.zone == Zone::Battlefield
                                && o.card.token_info().is_some_and(|t| t.subtypes == "Golem")
                        })
                        .unwrap();
                    let o = g.object_view(t.0 as CardId);
                    assert_eq!((o.power, o.toughness), (3, 3));
                    assert!(o.card.kind_mask() & 16 != 0);
                    assert_eq!(o.card.colors(), 0);
                }
                "Sweettooth Witch" => assert_eq!(g.state.players[p.other().index()].life, 18),
                "Umbral Collar Zealot" => {
                    assert_eq!(g.state.players[p.index()].library.len(), library)
                }
                _ => {
                    assert_eq!(
                        g.state.players[p.index()].library.len(),
                        library - 1,
                        "{name}"
                    );
                    if name == "Vampiric Rites" {
                        assert_eq!(g.state.players[p.index()].life, 21);
                    }
                }
            }
        }
    }
}

#[test]
fn sacrifice_predicates_reject_foreign_stale_wrong_kind_and_another_source() {
    for name in [
        "Hungry Ghoul",
        "Slagdrill Scrapper",
        "Sweettooth Witch",
        "Hardened Tactician",
    ] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let i = external_index(&g, source);
        let resource = match name {
            "Slagdrill Scrapper" => named(&mut g, Player::P0, "Forest", Zone::Battlefield),
            "Sweettooth Witch" => spawn(&mut g, Player::P0, token("Food"), Zone::Battlefield),
            "Hardened Tactician" => spawn(&mut g, Player::P0, token("Treasure"), Zone::Battlefield),
            _ => named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield),
        };
        let resource_card = g.state.objects[resource as usize].card;
        let foreign = spawn(&mut g, Player::P1, resource_card, Zone::Battlefield);
        let wrong = named(&mut g, Player::P0, "Forest", Zone::Hand);
        let wrong_kind = match name {
            "Hungry Ghoul" => named(&mut g, Player::P0, "Forest", Zone::Battlefield),
            "Slagdrill Scrapper" => named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield),
            "Sweettooth Witch" => spawn(&mut g, Player::P0, token("Treasure"), Zone::Battlefield),
            _ => named(&mut g, Player::P0, "Candy Trail", Zone::Battlefield),
        };
        g.state.players[0].mana = [50; 6];
        announce(&mut g, source, i);
        if name == "Sweettooth Witch" {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(Player::P1),
                },
            );
        }
        for object in [
            g.object_ref(source),
            g.object_ref(foreign),
            g.object_ref(wrong),
            g.object_ref(wrong_kind),
            ObjectRef {
                card: resource,
                generation: g.object_ref(resource).generation + 1,
            },
            ObjectRef {
                card: u32::MAX,
                generation: 0,
            },
        ] {
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        reject(&mut g, Action::DeclineEffectPayment);
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        choose(&mut g, resource);
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        pay(&mut g);
    }
}

#[test]
fn source_can_be_sacrificed_when_printed_filter_allows_and_direct_paths_cannot_bypass_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        g.state.active = p;
        g.state.priority = p;
        let id = named(&mut g, p, "Kingpin's Enforcers", Zone::Battlefield);
        g.state.players[p.index()].mana = [50; 6];
        let direct = Action::Activate {
            card: id,
            ability: 0,
            target: None,
            payment: [0, 0, 1, 0, 0, 2],
            counter_payment: Counters::default(),
        };
        reject(&mut g, direct);
        announce(&mut g, id, 0);
        assert!(
            g.observe(p)
                .additional_cost
                .unwrap()
                .candidates
                .contains(&g.object_ref(id))
        );
        choose(&mut g, id);
        let library = g.state.players[p.index()].library.len();
        pay(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library.len(), library);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
    }
}

#[test]
fn reserved_land_can_fund_the_activation_then_be_sacrificed() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Ripchain Razorkin", Zone::Battlefield);
    let chosen = named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
    let a = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let b = named(&mut g, Player::P0, "Island", Zone::Battlefield);
    announce(&mut g, source, 0);
    choose(&mut g, chosen);
    for card in [chosen, a, b] {
        replay(&mut g, Action::TapMana { card });
    }
    assert!(g.object_view(chosen).tapped);
    assert_eq!(g.object_view(chosen).zone, Zone::Battlefield);
    pay(&mut g);
    assert_eq!(g.object_view(chosen).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    settle(&mut g);
}

#[test]
fn reserved_token_cannot_be_consumed_for_mana_and_other_tokens_remain_usable() {
    for extras in [0, 1] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, "Hardened Tactician", Zone::Battlefield);
        let chosen = spawn(&mut g, Player::P0, token("Treasure"), Zone::Battlefield);
        let s = g.snapshot();
        assert!(
            g.apply(
                Player::P0,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default()
                }
            )
            .is_err()
        );
        assert_eq!(g.snapshot(), s);
        let other = if extras == 0 {
            named(&mut g, Player::P0, "Forest", Zone::Battlefield)
        } else {
            spawn(&mut g, Player::P0, token("Treasure"), Zone::Battlefield)
        };
        announce(&mut g, source, 0);
        choose(&mut g, chosen);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::ActivateMana{card,..}if *card==chosen))
        );
        if extras == 0 {
            replay(&mut g, Action::TapMana { card: other });
        } else {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==other))
                .unwrap();
            replay(&mut g, a);
        }
        pay(&mut g);
        assert_eq!(g.object_view(chosen).zone, Zone::Outside);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        settle(&mut g);
    }
}

#[test]
fn printed_entry_effects_and_all_fountainport_abilities_are_complete() {
    for name in ["Sweettooth Witch", "Legion Extruder"] {
        let mut g = scenario();
        let id = named(&mut g, Player::P0, name, Zone::Hand);
        g.state.players[0].mana = [50; 6];
        super::tests::cast(&mut g, id, None);
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        if name == "Legion Extruder" {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(Player::P1),
                },
            );
        }
        settle(&mut g);
        if name == "Legion Extruder" {
            assert_eq!(g.state.players[1].life, 18);
        } else {
            assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                && o.card.token_info().is_some_and(|t| t.subtypes == "Food")));
        }
    }
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Fountainport", Zone::Hand);
    replay(&mut g, Action::PlayLand { card: source });
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,ability:0,production,..}if *card==source&&*production==[0,0,0,0,0,1])).unwrap();
    replay(&mut g, a);
    assert!(g.object_view(source).tapped);
    assert_eq!(g.state.players[0].mana, [0, 0, 0, 0, 0, 1]);
    for ability in [2, 3] {
        g.state.active = Player::P0;
        g.begin_turn();
        g.state.step = Step::Main1;
        g.give_priority();
        g.state.players[0].mana = [50; 6];
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Activate{card,ability:n,..}if *card==source&&*n==ability))
            .unwrap();
        replay(&mut g, a);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.state.players[0].life, 19);
        settle(&mut g);
    }
    let fish = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| {
            o.zone == Zone::Battlefield && o.card.token_info().is_some_and(|t| t.subtypes == "Fish")
        })
        .unwrap();
    let fish = g.object_view(fish.0 as CardId);
    assert_eq!((fish.power, fish.toughness), (1, 1));
    assert_eq!(fish.card.colors(), 2);
    assert_eq!(fish.keywords, Keywords::NONE);
    assert!(g.state.objects.iter().any(|o| {
        o.zone == Zone::Battlefield
            && o.card
                .token_info()
                .is_some_and(|t| t.subtypes == "Treasure")
    }));
    g.state.active = Player::P0;
    g.begin_turn();
    g.state.step = Step::Main1;
    g.give_priority();
    g.state.players[0].mana = [50; 6];
    let library = g.state.players[0].library.len();
    announce(&mut g, source, 1);
    choose(&mut g, fish.id);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[0].library.len(), library - 1);
    assert_eq!(g.object_view(fish.id).zone, Zone::Outside);
}

#[test]
fn death_capacity_failure_rolls_back_source_tap_payment_and_selected_object() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Acolyte of Aclazotz", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    announce(&mut g, source, 0);
    choose(&mut g, bear);
    g.state.creatures_died_this_turn = u64::MAX;
    let saved = g.snapshot();
    let rev = g.revision();
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(Player::P0, a.clone()),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
        assert!(!g.object_view(source).tapped);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    }
    let mut fork = g.clone();
    assert!(matches!(
        fork.apply(Player::P0, a),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(fork.snapshot(), saved);
}

#[test]
fn sacrifice_return_entry_suspension_never_repeats_payment_or_source_costs() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Ahriman", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let aura = named(&mut g, Player::P0, "Frantic Strength", Zone::Exile);
    // Synthetic link isolates the already supported linked-return entry transaction.
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(bear),
        exiled: g.object_ref(aura),
        return_zone: Zone::Battlefield,
    });
    g.state.players[0].mana = [0, 0, 0, 0, 0, 3];
    announce(&mut g, source, 0);
    choose(&mut g, bear);
    pay(&mut g);
    assert_eq!(g.observe(Player::P0).decision, "aura_entry");
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    let a = g.state.announcing.as_ref().unwrap();
    assert!(a.additional_paid);
    assert!(a.payment.is_none());
    assert!(g.state.stack.is_empty());
    let object = g.object_ref(source);
    replay(&mut g, Action::ChooseAuraAttachment { object });
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
    assert!(g.state.announcing.is_none());
    let n = g.state.players[0].library.len();
    settle(&mut g);
    assert_eq!(g.state.players[0].library.len(), n - 1);
}

#[test]
fn surveil_and_mountaincycling_keep_private_owned_continuations_after_cost_commit() {
    let mut g = scenario();
    let source = named(
        &mut g,
        Player::P0,
        "Umbral Collar Zealot",
        Zone::Battlefield,
    );
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    announce(&mut g, source, 0);
    choose(&mut g, bear);
    pay(&mut g);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert!(g.observe(Player::P1).library_choice.is_none());
    let choice = g.observe(Player::P0).library_choice.unwrap();
    assert!(choice.surveil);
    assert_eq!(choice.remaining.len(), 1);
    let card = choice.remaining[0].id;
    replay(
        &mut g,
        Action::ArrangeLibrary {
            card,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(g.object_view(card).zone, Zone::Graveyard);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Seismic Monstrosaur", Zone::Hand);
    let mountain = named(&mut g, Player::P0, "Mountain", Zone::Library);
    let bad = named(&mut g, Player::P0, "Island", Zone::Library);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..}if *card==source))
        .unwrap();
    replay(&mut g, a);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(
        g.observe(Player::P0).search_choice.unwrap().filter,
        crate::SearchFilter::Subtype("Mountain")
    );
    assert!(g.observe(Player::P1).search_choice.is_none());
    reject(&mut g, Action::ChooseSearch { card: bad });
    replay(&mut g, Action::ChooseSearch { card: mountain });
    replay(&mut g, Action::FinishSearch);
    assert_eq!(g.object_view(mountain).zone, Zone::Hand);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
}

#[test]
fn cauldron_targets_precede_sacrifice_and_cost_deaths_trigger_before_recovery() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Cauldron of Essence", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let target = named(&mut g, Player::P0, "Piranha Fly", Zone::Graveyard);
    g.state.players[0].mana = [50; 6];
    announce(&mut g, source, 0);
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    let object = g.object_ref(bear);
    reject(&mut g, Action::ChooseAdditionalCost { object });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    choose(&mut g, bear);
    pay(&mut g);
    assert_eq!(g.state.stack.len(), 2);
    assert!(matches!(
        g.state.stack[0].kind,
        StackKind::ActivatedAbility { .. }
    ));
    assert!(matches!(
        g.state.stack[1].kind,
        StackKind::TriggeredAbility { .. }
    ));
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    assert!(g.object_view(source).tapped);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.state.players[0].life, 21);
    assert_eq!(g.state.players[1].life, 19);
    assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    // Structural removal models a countered activation without admitting a new counterspell.
    g.state.stack.pop().unwrap();
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    assert!(g.object_view(source).tapped);
}

#[test]
fn fountainport_cannot_fund_its_own_tap_cost_or_spend_reserved_token_for_mana() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Fountainport", Zone::Battlefield);
    let chosen = spawn(&mut g, Player::P0, token("Treasure"), Zone::Battlefield);
    let forest = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    let island = named(&mut g, Player::P0, "Island", Zone::Battlefield);
    announce(&mut g, source, 1);
    choose(&mut g, chosen);
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::ActivateMana{card,..}if *card==source||*card==chosen))
    );
    replay(&mut g, Action::TapMana { card: forest });
    replay(&mut g, Action::TapMana { card: island });
    pay(&mut g);
    assert!(g.object_view(source).tapped);
    assert_eq!(g.object_view(chosen).zone, Zone::Outside);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    settle(&mut g);
}
