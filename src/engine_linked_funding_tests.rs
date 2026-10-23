//! Reachable CR610.3 return during a mana ability, including a new copied ability epoch.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;
fn step(g: &mut Game) {
    let actions = g.legal_actions();
    let action = if actions.contains(&Action::Pass) {
        Action::Pass
    } else {
        actions
            .into_iter()
            .find(|a| matches!(a, Action::ChooseOptional { accept: true }))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
            })
            .unwrap()
    };
    checked(g, action);
}
fn mana(g: &Game, card: CardId, production: Mana) -> Action {
    g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card: c, production: p, .. } if *c==card && *p==production)).unwrap_or_else(|| panic!("missing mana {card} {production:?}; pending={:?} actions={:?}", g.state.pending, g.legal_actions()))
}
fn reachable(p: Player) -> (Game, CardId, CardId) {
    let mut g = setup(p.other());
    for name in [
        "Crawling Barrens",
        "Henchbots",
        "Goatnap",
        "Mirrorform",
        "Mazemind Tome",
    ] {
        assert!(Card::from_name(name).unwrap().is_standard_legal(), "{name}");
    }
    let barrens = named(&mut g, p, "Crawling Barrens", Zone::Battlefield);
    // An ordinary opponent-priority window in their first main phase.
    checked(&mut g, Action::Pass);
    checked(
        &mut g,
        Action::AnnounceActivate {
            card: barrens,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    for _ in 0..30 {
        if g.current_kinds(barrens) == 3 && g.state.stack.is_empty() && g.state.resolving.is_none()
        {
            break;
        }
        step(&mut g);
    }
    assert_eq!(g.current_kinds(barrens), 3);
    let source = named(&mut g, p.other(), "Henchbots", Zone::Hand);
    cast(&mut g, source, None, &[]);
    checked(&mut g, Action::Pass);
    let tap = mana(&g, barrens, [0, 0, 0, 0, 0, 1]);
    checked(&mut g, tap);
    for _ in 0..40 {
        if g.object_view(barrens).zone == Zone::Exile && g.state.stack.is_empty() {
            break;
        }
        let target = Action::ChooseTarget {
            target: Target::Card(barrens),
        };
        if g.legal_actions().contains(&target) {
            checked(&mut g, target);
        } else {
            step(&mut g);
        }
    }
    assert_eq!(g.object_view(barrens).zone, Zone::Exile);
    let reference = g.object_ref(source);
    assert_eq!(g.state.exile_links[0].source, reference);
    next_turn(&mut g);
    for _ in 0..40 {
        if g.state.step == Step::Main1 && g.state.pending == Pending::Priority {
            break;
        }
        step(&mut g);
    }
    assert_eq!(g.state.active, p);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    for land in [
        "Island", "Island", "Mountain", "Plains", "Plains", "Plains", "Plains", "Plains", "Plains",
    ] {
        let card = named(&mut g, p, land, Zone::Battlefield);
        checked(&mut g, Action::TapMana { card });
    }
    let goatnap = named(&mut g, p, "Goatnap", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: goatnap });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    checked(
        &mut g,
        Action::PayCosts {
            payment: [2, 0, 0, 1, 0, 0],
        },
    );
    settle(&mut g);
    assert_eq!(g.object_view(source).controller, p);
    let treasure = super::tests::spawn(
        &mut g,
        p.other(),
        Card::catalogue()
            .find(|card| {
                card.token_info()
                    .is_some_and(|info| info.name == "Treasure Token")
            })
            .unwrap(),
        Zone::Battlefield,
    );
    let mirror = named(&mut g, p, "Mirrorform", Zone::Hand);
    cast(&mut g, mirror, None, &[Target::Card(treasure)]);
    settle(&mut g);
    assert_eq!(g.object_ref(source), reference);
    assert_ne!(g.copy_ability_origin(reference), 0);
    assert!(g.has_subtype(source, "Treasure"));
    assert_eq!(g.state.exile_links[0].source, reference);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    (g, source, barrens)
}
#[test]
fn linked_funding_reachable_barrens_return_funds_announced_tome() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source, barrens) = reachable(p);
        let tome = named(&mut g, p, "Mazemind Tome", Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: tome })
        );
        checked(&mut g, Action::AnnounceCast { card: tome });
        let action = mana(&g, source, [1, 0, 0, 0, 0, 0]);
        checked(&mut g, action);
        assert_eq!(g.object_view(barrens).zone, Zone::Battlefield);
        assert_eq!(g.current_kinds(barrens), 2);
        assert!(!g.object_view(barrens).tapped);
        assert!(g.state.exile_links.is_empty());
        let action = mana(&g, barrens, [0, 0, 0, 0, 0, 1]);
        checked(&mut g, action);
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        settle(&mut g);
        assert_eq!(g.object_view(tome).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn linked_funding_audit_repeatable_mana_abilities() {
    println!(
        "linked funding layouts Game={} State={} Window={} Goal={}",
        std::mem::size_of::<Game>(),
        std::mem::size_of::<State>(),
        std::mem::size_of::<LinkedFundingWindow>(),
        std::mem::size_of::<LinkedFundingGoal>()
    );
    for card in Card::catalogue().filter(|c| c.is_supported()) {
        for (index, a) in card.activations().iter().enumerate() {
            if a.mana_production().is_some()
                && !a.cost.tap
                && !a.cost.sacrifice
                && !a.cost.exile_self
                && !a.cost.discard
                && !a.once_per_turn
                && !a.exhaust
                && a.cost.remove.is_none_or(|(_, amount)| amount == 0)
                && a.cost.life == 0
            {
                panic!(
                    "non-decreasing pure mana activation {card:?}/{index}: {:?}",
                    a.cost
                );
            }
        }
    }
}

// A prior opponent-controlled Henchbots can have exiled its tapped opposing
// Lavaleaper before Goatnap/Mirrorform. Its CR610.3 return affects both players.
#[test]
fn linked_funding_returned_opponent_lavaleaper_funds_basic_land_bonus() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let source = super::tests::spawn(
            &mut g,
            p,
            Card::catalogue()
                .find(|c| c.token_info().is_some_and(|i| i.name == "Treasure Token"))
                .unwrap(),
            Zone::Battlefield,
        );
        let lava = named(&mut g, p.other(), "Lavaleaper", Zone::Exile);
        g.state.exile_links.push(ExileLink {
            source: g.object_ref(source),
            exiled: g.object_ref(lava),
            return_zone: Zone::Battlefield,
        });
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let tome = named(&mut g, p, "Carnelian Orb of Dragonkind", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: tome });
        let a = mana(&g, source, [0, 0, 0, 1, 0, 0]);
        checked(&mut g, a);
        assert_eq!(g.object_view(lava).zone, Zone::Battlefield);
        checked(&mut g, Action::TapMana { card: forest });
        while g.mana_tap_bonus_pending() {
            step(&mut g);
        }
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.object_view(tome).zone, Zone::Battlefield);
    }
}

#[test]
fn linked_funding_omni_cycle_keeps_only_completable_entry_prefixes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        // Imperious Perfect keeps the earlier uncopied Changeling alive. Its
        // token activation is a stack ability and cannot fund this mana window.
        named(&mut g, p, "Imperious Perfect", Zone::Battlefield);
        let earlier = named(&mut g, p, "Omni-Changeling", Zone::Battlefield);
        let donor = named(&mut g, p, "Great Forest Druid", Zone::Battlefield);
        named(&mut g, p.other(), "Lavaleaper", Zone::Battlefield);
        let source = super::tests::spawn(
            &mut g,
            p,
            Card::catalogue()
                .find(|c| c.token_info().is_some_and(|i| i.name == "Treasure Token"))
                .unwrap(),
            Zone::Battlefield,
        );
        let incoming = named(&mut g, p, "Omni-Changeling", Zone::Exile);
        g.state.exile_links.push(ExileLink {
            source: g.object_ref(source),
            exiled: g.object_ref(incoming),
            return_zone: Zone::Battlefield,
        });
        let spell = named(&mut g, p, "Carnelian Orb of Dragonkind", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        let action = mana(&g, source, [1, 0, 0, 0, 0, 0]);
        checked(&mut g, action);
        assert!(matches!(g.state.pending, Pending::EntryCopy { .. }));
        assert!(g.state.exile_links.is_empty());
        assert!(g.observe(p).cost_completion.is_some());
        assert!(
            !g.legal_actions()
                .contains(&Action::ChooseEntryCopy { object: None })
        );
        let looping = Action::ChooseEntryCopy {
            object: Some(g.object_ref(earlier)),
        };
        checked(&mut g, looping.clone());
        checked(&mut g, looping);
        let donor_ref = g.object_ref(donor);
        checked(
            &mut g,
            Action::ChooseEntryCopy {
                object: Some(donor_ref),
            },
        );
        for (id, production) in [(donor, [0, 0, 0, 0, 1, 0]), (incoming, [0, 0, 0, 1, 0, 0])] {
            let action = mana(&g, id, production);
            checked(&mut g, action);
        }
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert!(g.observe(p).cost_completion.is_none());
        assert!(g.funding_projection.is_none());
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}

#[test]
fn linked_funding_returned_eirdu_dynamically_enables_convoke() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let source = super::tests::spawn(
            &mut g,
            p,
            Card::catalogue()
                .find(|c| c.token_info().is_some_and(|i| i.name == "Treasure Token"))
                .unwrap(),
            Zone::Battlefield,
        );
        let eirdu = named(&mut g, p, "Eirdu, Carrier of Dawn", Zone::Exile);
        g.state.exile_links.push(ExileLink {
            source: g.object_ref(source),
            exiled: g.object_ref(eirdu),
            return_zone: Zone::Battlefield,
        });
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Great Forest Druid", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        let a = mana(&g, source, [1, 0, 0, 0, 0, 0]);
        checked(&mut g, a);
        checked(&mut g, Action::BeginConvoke);
        for _ in 0..12 {
            if g.state.announcing.is_none() {
                break;
            }
            step(&mut g);
        }
        assert!(g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}

#[test]
fn linked_funding_completed_fatal_life_cost_remains_legal() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source, barrens) = reachable(p);
        let ring = named(&mut g, p, "Ring of the Lucii", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].life = 1;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: ring,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        let a = mana(&g, source, [1, 0, 0, 0, 0, 0]);
        checked(&mut g, a);
        let a = mana(&g, barrens, [0, 0, 0, 0, 0, 1]);
        checked(&mut g, a);
        assert!(
            g.state
                .mana_lots
                .as_ref()
                .is_none_or(|s| s.prepared.is_none() && s.replay.is_none())
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(g.state.players[p.index()].life, 0);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
    }
}

#[test]
fn linked_funding_paid_converter_retains_tagged_allocations_and_return() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source, barrens) = reachable(p);
        g.copy_permanents_exact(
            &[g.object_ref(source)],
            CopyRecipe::new(Card::from_name("Omni-Cheese Pizza").unwrap()),
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        g.add_quality_mana(
            p,
            [0, 0, 0, 1, 0, 0],
            Some(mana_lots::Quality {
                restriction: ManaRestrictionView::SpellSubtype("Dragon"),
                spell_bonus: ManaSpellBonusView::None,
                retention: crate::cards::ManaRetention::Step,
                copy_source: None,
            }),
        );
        g.add_quality_mana(
            p,
            [0, 0, 0, 0, 0, 1],
            Some(mana_lots::Quality {
                restriction: ManaRestrictionView::Unrestricted,
                spell_bonus: ManaSpellBonusView::None,
                retention: crate::cards::ManaRetention::Turn,
                copy_source: None,
            }),
        );
        let spell = named(&mut g, p, "Shivan Dragon", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,payment,..} if *card==source && *production==[0,0,0,1,0,0] && *payment==[0,0,0,0,0,1])).unwrap();
        checked(&mut g, action);
        assert!(g.mana_lot_spend_range().is_some());
        while let Some(range) = g.mana_lot_spend_range() {
            checked(
                &mut g,
                Action::ChooseManaLotSpend {
                    amount: range.minimum,
                },
            );
        }
        assert_eq!(g.object_view(barrens).zone, Zone::Battlefield);
        assert_eq!(
            g.object_view(source).card,
            Card::from_name("Henchbots").unwrap()
        );
        let a = mana(&g, barrens, [0, 0, 0, 0, 0, 1]);
        checked(&mut g, a);
        assert!(
            g.state
                .mana_lots
                .as_ref()
                .is_none_or(|s| s.prepared.is_none() && s.replay.is_none())
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        while let Some(range) = g.mana_lot_spend_range() {
            checked(
                &mut g,
                Action::ChooseManaLotSpend {
                    amount: range.minimum,
                },
            );
        }
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}

#[test]
fn linked_funding_billion_wide_lot_domain_streams_early_completion() {
    let p = Player::P0;
    let (mut g, _, _) = reachable(p);
    let half = u32::MAX / 2;
    g.state.players[p.index()].mana = [0, 0, half, 0, 0, 0];
    g.add_quality_mana(
        p,
        [0, 0, half, 0, 0, 0],
        Some(mana_lots::Quality {
            restriction: ManaRestrictionView::Unrestricted,
            spell_bonus: ManaSpellBonusView::None,
            retention: crate::cards::ManaRetention::Turn,
            copy_source: None,
        }),
    );
    let spell = named(&mut g, p, "Scavenger Regent // Exude Toxin", Zone::Hand);
    checked(&mut g, Action::AnnounceOmen { card: spell });
    g.apply(p, Action::ChooseX { value: half - 2 }).unwrap();
    checked(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, half, 0, 0, 0],
        },
    );
    let range = g.mana_lot_spend_range().unwrap();
    assert_eq!((range.minimum, range.maximum), (0, half));
    checked(&mut g, Action::ChooseManaLotSpend { amount: 0 });
    assert!(g.state.announcing.is_none());
    assert_eq!(g.state.players[p.index()].mana[2], half);
}

#[test]
fn linked_funding_returned_elf_can_witness_behold_after_mana_window() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let source = super::tests::spawn(
            &mut g,
            p,
            Card::catalogue()
                .find(|c| c.token_info().is_some_and(|i| i.name == "Treasure Token"))
                .unwrap(),
            Zone::Battlefield,
        );
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Exile);
        g.state.exile_links.push(ExileLink {
            source: g.object_ref(source),
            exiled: g.object_ref(elf),
            return_zone: Zone::Battlefield,
        });
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let spell = named(&mut g, p, "Lys Alana Dignitary", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        for _ in 0..8 {
            if matches!(g.state.pending, Pending::AdditionalCost { .. }) {
                break;
            }
            step(&mut g);
        }
        assert!(matches!(g.state.pending, Pending::AdditionalCost { .. }));
        let locked = g.state.announcing.as_ref().unwrap().total_cost;
        assert!(locked.is_some());
        let a = mana(&g, source, [1, 0, 0, 0, 0, 0]);
        checked(&mut g, a);
        assert_eq!(g.state.announcing.as_ref().unwrap().total_cost, locked);
        assert_eq!(g.object_view(elf).zone, Zone::Battlefield);
        let object = g.object_ref(elf);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(&mut g, Action::TapMana { card: forest });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 1, 0],
            },
        );
        assert!(g.state.stack.iter().any(|item| item.card == spell));
        assert_eq!(g.object_view(elf).zone, Zone::Battlefield);
    }
}

fn future_witness(p: Player, name: &str) -> (Game, CardId, CardId) {
    let mut g = setup(p);
    g.state.players[p.index()].mana = [0; 6];
    let source = super::tests::spawn(
        &mut g,
        p,
        Card::catalogue()
            .find(|c| c.token_info().is_some_and(|i| i.name == "Treasure Token"))
            .unwrap(),
        Zone::Battlefield,
    );
    let witness = named(&mut g, p, name, Zone::Exile);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(source),
        exiled: g.object_ref(witness),
        return_zone: Zone::Battlefield,
    });
    (g, source, witness)
}
#[test]
fn linked_funding_optional_blight_declares_intent_before_future_witness() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source, witness) = future_witness(p, "Great Forest Druid");
        let target = named(&mut g, p.other(), "Great Forest Druid", Zone::Battlefield);
        let spell = named(&mut g, p, "Cinder Strike", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        assert!(matches!(
            g.state.pending,
            Pending::OptionalAdditionalCost { .. }
        ));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ActivateMana { .. }))
        );
        checked(&mut g, Action::ChooseOptional { accept: true });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        assert!(matches!(g.state.pending, Pending::AdditionalCost { .. }));
        let a = mana(&g, source, [0, 0, 0, 1, 0, 0]);
        checked(&mut g, a);
        let object = g.object_ref(witness);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(witness, CounterKind::MinusOne), 1);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
}
#[test]
fn linked_funding_bargain_declares_intent_before_future_artifact() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source, witness) = future_witness(p, "Three Tree Mascot");
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Great Forest Druid", Zone::Battlefield);
        let spell = named(&mut g, p, "Candy Grapple", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        assert!(matches!(g.state.pending, Pending::Bargain { .. }));
        checked(&mut g, Action::ChooseOptional { accept: true });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        let a = mana(&g, source, [0, 0, 1, 0, 0, 0]);
        checked(&mut g, a);
        let object = g.object_ref(witness);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(&mut g, Action::TapMana { card: forest });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 1, 0],
            },
        );
        assert_eq!(g.object_view(witness).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
}
#[test]
fn linked_funding_behold_type_intent_preserves_new_second_witness() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source, witness) = future_witness(p, "Llanowar Elves");
        let first = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        let spell = named(&mut g, p, "Celestial Reunion", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseX { value: 0 });
        checked(
            &mut g,
            Action::ChooseBeholdType {
                creature_type: Some(crate::cards::CreatureType::from_name("Elf").unwrap()),
            },
        );
        let a = mana(&g, source, [0, 0, 0, 0, 1, 0]);
        checked(&mut g, a);
        for id in [first, witness] {
            let object = g.object_ref(id);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        let item = g
            .state
            .stack
            .iter()
            .find(|item| item.card == spell)
            .unwrap();
        assert_eq!(item.cast.unwrap().optional_additional, Some(true));
        assert_eq!(item.source_chosen_type.unwrap().name(), "Elf");
    }
}

#[test]
fn linked_funding_future_bargain_discount_is_part_of_initial_cost_route() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source, witness) = future_witness(p, "Three Tree Mascot");
        let forests = (0..4)
            .map(|_| named(&mut g, p, "Forest", Zone::Battlefield))
            .collect::<Vec<_>>();
        let spell = named(&mut g, p, "Hamlet Glutton", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseOptional { accept: true });
        let a = mana(&g, source, [0, 0, 0, 0, 1, 0]);
        checked(&mut g, a);
        let object = g.object_ref(witness);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        let locked = g.state.announcing.as_ref().unwrap().total_cost.unwrap();
        assert_eq!((locked.generic, locked.colored), (3, [0, 0, 0, 0, 2, 0]));
        for card in forests {
            checked(&mut g, Action::TapMana { card });
        }
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 5, 0],
            },
        );
        assert_eq!(g.object_view(witness).zone, Zone::Graveyard);
        assert_eq!(
            g.state
                .stack
                .iter()
                .find(|s| s.card == spell)
                .unwrap()
                .cast
                .unwrap()
                .bargain,
            Some(true)
        );
    }
}

#[test]
fn linked_funding_blight_x_zero_can_use_returned_witness_without_raising_bound() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source, witness) = future_witness(p, "Llanowar Elves");
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 3];
        let spell = named(&mut g, p, "Soul Immolation", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(
            g.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: 0
            }]
        );
        checked(&mut g, Action::ChooseX { value: 0 });
        let a = mana(&g, source, [0, 0, 0, 1, 0, 0]);
        checked(&mut g, a);
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(0));
        let object = g.object_ref(witness);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 2, 0, 3],
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(witness, CounterKind::MinusOne), 0);
    }
}
