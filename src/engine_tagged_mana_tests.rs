use super::super::ability_timeline_tests::{cast, checked, settle, setup};
use super::super::tests::spawn;
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn activate(g: &mut Game, card: CardId, ability: u16, production: Mana) {
    checked(
        g,
        Action::ActivateMana {
            card,
            ability,
            production,
            payment: [0; 6],
            counter_payment: Counters::default(),
        },
    );
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(before, g.snapshot());
    assert_eq!(revision, g.revision());
}
#[test]
fn tagged_mana_orb_paid_cast_chooses_real_haste_and_owned_transaction() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert_eq!(std::mem::size_of::<ManaProduction>(), 32);
    assert_eq!(std::mem::size_of::<Pending>(), 40);
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 7, 0, 0];
        let orb = named(&mut g, p, "Carnelian Orb of Dragonkind", Zone::Battlefield);
        activate(&mut g, orb, 0, [0, 0, 0, 1, 0, 0]);
        let dragon = named(&mut g, p, "Shivan Dragon", Zone::Hand);
        checked(
            &mut g,
            Action::Cast {
                card: dragon,
                target: None,
                payment: [0, 0, 0, 6, 0, 0],
            },
        );
        assert_eq!(g.object_view(dragon).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].mana[3], 8);
        let range = g.mana_lot_spend_range().unwrap();
        assert_eq!((range.minimum, range.maximum), (0, 1));
        assert!(g.observe(p).mana_lot_spend.is_some());
        assert!(g.observe(p.other()).mana_lot_spend.is_none());
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == dragon));
        reject(&mut g, Action::ChooseManaLotSpend { amount: 2 });
        let choice = g.snapshot();
        let mut conceded = g.clone();
        conceded.apply(p.other(), Action::Concede).unwrap();
        assert_eq!(conceded.outcome(), Some(Outcome::Winner(p)));
        assert!(conceded.legal_action_space().mana_lot_spend.is_none());
        assert_eq!(g.snapshot(), choice);
        checked(&mut g, Action::ChooseManaLotSpend { amount: 0 });
        settle(&mut g);
        assert!(!g.keywords(dragon).contains(Keywords::HASTE));
        g.restore(&choice).unwrap();
        checked(&mut g, Action::ChooseManaLotSpend { amount: 1 });
        settle(&mut g);
        assert!(g.keywords(dragon).contains(Keywords::HASTE));
        assert_eq!(g.state.players[p.index()].mana[3], 2);
    }
}
#[test]
fn tagged_mana_giada_restricts_raw_announced_and_funding_spells() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let giada = named(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        activate(&mut g, giada, 0, [1, 0, 0, 0, 0, 0]);
        let chaplain = named(&mut g, p, "Ajani's Pridemate", Zone::Hand);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: chaplain })
        );
        reject(
            &mut g,
            Action::Cast {
                card: chaplain,
                target: None,
                payment: [1, 0, 0, 0, 0, 1],
            },
        );
        assert_eq!(g.available_mana(p, Purpose::Other), [0, 0, 0, 0, 0, 1]);
        let angel = named(&mut g, p, "Serra Angel", Zone::Hand);
        g.add_mana(p, [1, 0, 0, 0, 0, 2]);
        cast(&mut g, angel, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(angel).zone, Zone::Battlefield);
        assert_eq!(g.counter_amounts(angel).get(CounterKind::PlusOne), 1);
    }
}
#[test]
fn tagged_mana_courtyard_captures_type_after_producer_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 0];
        let courtyard = named(&mut g, p, "Secluded Courtyard", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: courtyard });
        let angel = CreatureType::from_name("Angel").unwrap();
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(angel),
            },
        );
        activate(&mut g, courtyard, 1, [1, 0, 0, 0, 0, 0]);
        g.sacrifice_batch(&[(p, courtyard)]);
        assert_eq!(g.object_view(courtyard).zone, Zone::Graveyard);
        assert_eq!(
            g.mana_lots(p)[0].quality.restriction,
            ManaRestrictionView::CreatureSpellOrActivation(angel)
        );
        let serra = Card::from_name("Serra Angel").unwrap();
        assert_eq!(
            g.available_mana(p, g.spell_mana_purpose(serra)),
            [1, 0, 0, 0, 0, 0]
        );
        assert_eq!(g.available_mana(p, Purpose::Other), [0; 6]);
        let creature = Source::printed(serra);
        assert_eq!(
            g.available_mana(p, Purpose::Activation(creature)),
            [1, 0, 0, 0, 0, 0]
        );
        g.add_mana(p, [1, 0, 0, 0, 0, 3]);
        let spell = named(&mut g, p, "Serra Angel", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}
#[test]
fn tagged_mana_giada_simultaneous_entries_count_pre_event_angels_and_double() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        named(&mut g, p, "Serra Angel", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let a = named(&mut g, p, "Inspiring Overseer", Zone::Graveyard);
        let b = named(&mut g, p, "Serra Angel", Zone::Graveyard);
        let moves = [a, b]
            .into_iter()
            .map(|id| g.entry_move(id, Zone::Battlefield, false))
            .collect();
        g.stage_entries(moves, None);
        g.advance_entries();
        while matches!(g.state.pending, Pending::Replacement { .. }) {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| *a != Action::Concede)
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(g.counter_amounts(a).get(CounterKind::PlusOne), 4);
        assert_eq!(g.counter_amounts(b).get(CounterKind::PlusOne), 4);
    }
}
#[test]
fn tagged_mana_ventmaw_attack_retains_both_colors_until_actual_turn_end() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let ventmaw = named(&mut g, p, "Savage Ventmaw", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: ventmaw });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 3, 3, 0]);
        let turn = g.state.turn;
        let mut saw_main2 = false;
        for _ in 0..60 {
            if g.state.turn != turn {
                break;
            }
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 3, 3, 0]);
            saw_main2 |= g.state.step == Step::Main2;
            let action = g
                .legal_actions()
                .into_iter()
                .find(|action| {
                    matches!(
                        action,
                        Action::Pass
                            | Action::FinishBlockers
                            | Action::FinishAttackers
                            | Action::Discard { .. }
                    )
                })
                .unwrap();
            checked(&mut g, action);
        }
        assert!(saw_main2);
        assert_ne!(g.state.turn, turn);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.state.mana_lots.is_none());
    }
}
#[test]
fn tagged_mana_guided_scarecrow_cannot_strand_enclosing_angel_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 3, 0, 0];
        for (kind, production) in [
            ("Scarecrow", [0, 0, 0, 1, 0, 0]),
            ("Angel", [1, 0, 0, 0, 0, 0]),
        ] {
            let court = named(&mut g, p, "Secluded Courtyard", Zone::Battlefield);
            g.set_entry_value(
                g.object_ref(court),
                EntryChoiceValue::CreatureType(CreatureType::from_name(kind).unwrap()),
            );
            activate(&mut g, court, 1, production);
        }
        let prismite = named(&mut g, p, "Scarecrow Guide", Zone::Battlefield);
        let angel = named(&mut g, p, "Serra Angel", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: angel });
        let before = g.snapshot();
        checked(
            &mut g,
            Action::ActivateMana {
                card: prismite,
                ability: 0,
                payment: [0, 0, 0, 1, 0, 0],
                counter_payment: Counters::default(),
                production: [1, 0, 0, 0, 0, 0],
            },
        );
        // Paying ordinary red strands this announced spell. The exact
        // guided allocation must instead spend one Scarecrow-only unit.
        assert_eq!(
            g.available_mana(
                p,
                g.spell_mana_purpose(Card::from_name("Serra Angel").unwrap())
            ),
            [2, 0, 0, 3, 0, 0]
        );
        assert!(g.mana_lot_spend_range().is_none());
        reject(&mut g, Action::ChooseManaLotSpend { amount: 0 });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [2, 0, 0, 3, 0, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(angel).zone, Zone::Battlefield);
        assert!(g.state.mana_lots.is_none());
        g.restore(&before).unwrap();
        assert!(g.legal_actions().iter().any(
            |action| matches!(action, Action::ActivateMana { card, .. } if *card == prismite)
        ));
    }
}
#[test]
fn tagged_mana_courtyard_pays_real_creature_card_activations_in_hand_and_graveyard() {
    for p in [Player::P0, Player::P1] {
        for (kind, name, zone, ordinary, payment) in [
            (
                "Dinosaur",
                "Soaring Sandwing",
                Zone::Hand,
                [0, 0, 0, 0, 0, 1],
                [0, 0, 0, 1, 0, 1],
            ),
            (
                "Zombie",
                "Suspicious Shambler",
                Zone::Graveyard,
                [0, 0, 1, 0, 0, 4],
                [0, 0, 2, 0, 0, 4],
            ),
        ] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = ordinary;
            let courtyard = named(&mut g, p, "Secluded Courtyard", Zone::Battlefield);
            g.set_entry_value(
                g.object_ref(courtyard),
                EntryChoiceValue::CreatureType(CreatureType::from_name(kind).unwrap()),
            );
            let production = if zone == Zone::Hand {
                [0, 0, 0, 1, 0, 0]
            } else {
                [0, 0, 1, 0, 0, 0]
            };
            activate(&mut g, courtyard, 1, production);
            let source = named(&mut g, p, name, zone);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            checked(&mut g, Action::PayCosts { payment });
            assert_ne!(g.object_view(source).zone, zone);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn tagged_mana_giada_counts_fresh_chosen_angel_and_respects_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let giada = named(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        let automaton = named(&mut g, p, "Adaptive Automaton", Zone::Hand);
        cast(&mut g, automaton, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(CreatureType::from_name("Angel").unwrap()),
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(automaton, CounterKind::PlusOne), 1);
        let stop = named(&mut g, p.other(), "Stop Cold", Zone::Battlefield);
        g.state.objects[stop as usize].attached_to = Some(g.object_ref(giada));
        assert!(!g.printed_abilities_available(giada));
        let angel = named(&mut g, p, "Serra Angel", Zone::Hand);
        cast(&mut g, angel, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(angel, CounterKind::PlusOne), 0);
    }
}
#[test]
fn tagged_mana_giada_unproduced_restricted_source_funds_only_matching_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 3];
        named(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        let angel = named(&mut g, p, "Serra Angel", Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: angel })
        );
        assert!(
            g.state.mana_lots.is_none(),
            "funding must not manufacture real mana"
        );
        checked(&mut g, Action::AnnounceCast { card: angel });
        let ability = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ActivateMana { .. }))
            .unwrap();
        checked(&mut g, ability);
        checked(
            &mut g,
            Action::PayCosts {
                payment: [2, 0, 0, 0, 0, 3],
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(angel, CounterKind::PlusOne), 1);
    }
}

#[test]
fn tagged_mana_guided_prefixes_never_mix_incompatible_complete_plans() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 0];
        for (kind, production) in [
            ("Scarecrow", [0, 0, 0, 1, 0, 0]),
            ("Construct", [0, 0, 0, 1, 0, 0]),
            ("Angel", [1, 0, 0, 0, 0, 0]),
        ] {
            let court = named(&mut g, p, "Secluded Courtyard", Zone::Battlefield);
            g.set_entry_value(
                g.object_ref(court),
                EntryChoiceValue::CreatureType(CreatureType::from_name(kind).unwrap()),
            );
            activate(&mut g, court, 1, production);
        }
        let first = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        let second = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        let angel = named(&mut g, p, "Serra Angel", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: angel });
        checked(
            &mut g,
            Action::ActivateMana {
                card: first,
                ability: 0,
                payment: [0, 0, 0, 1, 0, 0],
                counter_payment: Counters::default(),
                production: [1, 0, 0, 0, 0, 0],
            },
        );
        assert!(g.mana_lot_spend_range().is_some());
        let before = g.snapshot();
        for amount in [0, 1] {
            g.restore(&before).unwrap();
            reject(&mut g, Action::ChooseManaLotSpend { amount: 2 });
            checked(&mut g, Action::ChooseManaLotSpend { amount });
            assert!(
                g.mana_lot_spend_range().is_none(),
                "the other restricted unit must be paid if the first is retained"
            );
            checked(
                &mut g,
                Action::ActivateMana {
                    card: second,
                    ability: 0,
                    payment: [0, 0, 0, 1, 0, 0],
                    counter_payment: Counters::default(),
                    production: [1, 0, 0, 0, 0, 0],
                },
            );
            checked(
                &mut g,
                Action::PayCosts {
                    payment: [3, 0, 0, 2, 0, 0],
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(angel).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn tagged_mana_guided_scalar_domain_preserves_holes_and_correlated_rows() {
    let p = Player::P0;
    let mut g = setup(p);
    let quality = Quality {
        restriction: ManaRestrictionView::Unrestricted,
        spell_bonus: ManaSpellBonusView::DragonCreatureHaste,
        copy_source: None,
        retention: ManaRetention::Step,
    };
    let request = SpendRequest {
        actor: p,
        purpose: Purpose::Other,
        payment: [0, 0, 0, 2, 0, 0],
        ordinary: [0; 6],
        lots: vec![
            Lot {
                quality,
                amounts: [0, 0, 0, 2, 0, 0]
            };
            2
        ]
        .into(),
        viable: Some(vec![
            vec![[0; 6], [0, 0, 0, 2, 0, 0]],
            vec![[0, 0, 0, 2, 0, 0], [0; 6]],
        ]),
    };
    let mut choice = SpendChoice {
        request,
        action: Action::Pass,
        pending: Pending::Priority,
        allocations: vec![[0; 6]; 2],
        remaining: [0, 0, 0, 2, 0, 0],
        cursor: 0,
    };
    let range = choice.range().unwrap();
    assert_eq!(range.values, Some(vec![0, 2]));
    assert!(!range.contains(1));
    g.state.mana_lots = Some(Box::new(ManaState {
        choice: Some(Box::new(choice)),
        ..ManaState::default()
    }));
    g.state.pending = Pending::ManaLotSpend { player: p };
    reject(&mut g, Action::ChooseManaLotSpend { amount: 1 });
}
#[test]
fn tagged_mana_adventure_purpose_uses_actual_sorcery_face_and_cannot_spend_creature_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 6, 0];
        let court = named(&mut g, p, "Secluded Courtyard", Zone::Battlefield);
        g.set_entry_value(
            g.object_ref(court),
            EntryChoiceValue::CreatureType(CreatureType::from_name("Elemental").unwrap()),
        );
        activate(&mut g, court, 1, [0, 0, 0, 0, 1, 0]);
        let familiar = named(&mut g, p, "Bramble Familiar", Zone::Hand);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceAdventure { card: familiar })
        );
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: familiar })
        );
        reject(&mut g, Action::AnnounceAdventure { card: familiar });
        g.add_mana(p, [0, 0, 0, 0, 1, 0]);
        checked(&mut g, Action::AnnounceAdventure { card: familiar });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 7, 0],
            },
        );
        assert!(!g.state.stack.last().unwrap().source_card.is_creature());
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        assert_eq!(g.mana_lots(p)[0].amounts, [0, 0, 0, 0, 1, 0]);
    }
}
#[test]
fn tagged_mana_orb_haste_is_not_copied_and_later_ability_removal_wins() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 5, 0, 0];
        let orb = named(&mut g, p, "Carnelian Orb of Dragonkind", Zone::Battlefield);
        activate(&mut g, orb, 0, [0, 0, 0, 1, 0, 0]);
        let dragon = named(&mut g, p, "Shivan Dragon", Zone::Hand);
        checked(
            &mut g,
            Action::Cast {
                card: dragon,
                target: None,
                payment: [0, 0, 0, 6, 0, 0],
            },
        );
        settle(&mut g);
        assert!(g.keywords(dragon).contains(Keywords::HASTE));
        g.add_mana(p, [0, 6, 0, 0, 0, 0]);
        let reflect = named(&mut g, p, "Self-Reflection", Zone::Hand);
        cast(&mut g, reflect, None, &[Target::Card(dragon)]);
        settle(&mut g);
        let copy = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield && o.is_copy_token() && o.card.name() == "Shivan Dragon"
            })
            .unwrap()
            .0 as CardId;
        assert!(!g.keywords(copy).contains(Keywords::HASTE));
        g.add_mana(p, [0, 1, 0, 0, 0, 3]);
        let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, stop, None, &[Target::Card(dragon)]);
        settle(&mut g);
        assert!(!g.keywords(dragon).contains(Keywords::HASTE));
        g.sacrifice_batch(&[(p, stop)]);
        g.state_based_actions();
        assert!(g.keywords(dragon).contains(Keywords::HASTE));
    }
}

#[test]
fn tagged_mana_orb_countered_spell_and_dragon_activation_do_not_keep_haste() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 5, 0, 0];
        let orb = named(&mut g, p, "Carnelian Orb of Dragonkind", Zone::Battlefield);
        activate(&mut g, orb, 0, [0, 0, 0, 1, 0, 0]);
        let dragon = named(&mut g, p, "Shivan Dragon", Zone::Hand);
        checked(
            &mut g,
            Action::Cast {
                card: dragon,
                target: None,
                payment: [0, 0, 0, 6, 0, 0],
            },
        );
        assert!(g.keywords(dragon).contains(Keywords::HASTE));
        checked(&mut g, Action::Pass);
        let counter = named(&mut g, p.other(), "Essence Scatter", Zone::Hand);
        cast(&mut g, counter, None, &[Target::Spell(dragon)]);
        settle(&mut g);
        assert_eq!(g.object_view(dragon).zone, Zone::Graveyard);
        assert!(!g.keywords(dragon).contains(Keywords::HASTE));
        assert!(
            !g.observe(p)
                .keyword_effects
                .iter()
                .any(|row| row.recipient.card == dragon)
        );

        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let orb = named(&mut g, p, "Carnelian Orb of Dragonkind", Zone::Battlefield);
        let dragon = named(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        activate(&mut g, orb, 0, [0, 0, 0, 1, 0, 0]);
        checked(
            &mut g,
            Action::Activate {
                card: dragon,
                ability: 0,
                target: None,
                payment: [0, 0, 0, 1, 0, 0],
                counter_payment: Counters::default(),
            },
        );
        settle(&mut g);
        assert!(!g.keywords(dragon).contains(Keywords::HASTE));
        assert_eq!(g.stats(dragon).power, 6);
    }
}

#[test]
fn tagged_mana_three_lot_suspended_prefix_survives_parent_disposal_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let ventmaw = named(&mut g, p, "Savage Ventmaw", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: ventmaw });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        for _ in 0..30 {
            if g.state.step == Step::Main2 {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::FinishBlockers))
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(g.state.step, Step::Main2);
        let orb = named(&mut g, p, "Carnelian Orb of Dragonkind", Zone::Battlefield);
        activate(&mut g, orb, 0, [0, 0, 0, 1, 0, 0]);
        let court = named(&mut g, p, "Secluded Courtyard", Zone::Battlefield);
        g.set_entry_value(
            g.object_ref(court),
            EntryChoiceValue::CreatureType(CreatureType::from_name("Dragon").unwrap()),
        );
        activate(&mut g, court, 1, [0, 0, 0, 1, 0, 0]);
        g.add_mana(p, [0, 0, 0, 6, 0, 0]);
        let dragon = named(&mut g, p, "Shivan Dragon", Zone::Hand);
        let bytes_before = g.state_bytes();
        checked(
            &mut g,
            Action::Cast {
                card: dragon,
                target: None,
                payment: [0, 0, 0, 6, 0, 0],
            },
        );
        checked(&mut g, Action::ChooseManaLotSpend { amount: 1 });
        checked(&mut g, Action::ChooseManaLotSpend { amount: 1 });
        assert!(
            g.mana_lot_spend_range().is_some(),
            "third lot remains after two actual scalar decisions"
        );
        let cold = g.state.mana_lots.as_ref().unwrap();
        let choice = cold.choice.as_ref().unwrap();
        assert!(
            g.state_bytes()
                >= bytes_before
                    + std::mem::size_of::<SpendChoice>()
                    + choice.allocations.len() * std::mem::size_of::<Mana>()
        );
        assert!(cold.pools[p.index()].ptr_eq(&choice.request.lots));
        let suspended = g.snapshot();
        let mut owned = {
            let parent = g.clone();
            parent.clone()
        };
        drop(g);
        assert_eq!(owned.snapshot(), suspended);
        reject(&mut owned, Action::ChooseManaLotSpend { amount: 2 });
        checked(&mut owned, Action::ChooseManaLotSpend { amount: 1 });
        settle(&mut owned);
        assert_eq!(owned.object_view(dragon).zone, Zone::Battlefield);
        assert!(owned.keywords(dragon).contains(Keywords::HASTE));
        owned.restore(&suspended).unwrap();
        assert!(owned.mana_lot_spend_range().is_some());
        checked(&mut owned, Action::ChooseManaLotSpend { amount: 0 });
        settle(&mut owned);
        assert_eq!(owned.object_view(dragon).zone, Zone::Battlefield);
    }
}

#[test]
fn tagged_mana_convoke_guidance_and_can_pay_exclude_non_angel_giada_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 0];
        let giada = named(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        activate(&mut g, giada, 0, [1, 0, 0, 0, 0, 0]);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let bears: Vec<_> = (0..3)
            .map(|_| named(&mut g, p, "Bear Cub", Zone::Battlefield))
            .collect();
        let celebrant = named(&mut g, p, "Sun-Dappled Celebrant", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: celebrant });
        reject(&mut g, Action::BeginConvoke);
        checked(&mut g, Action::TapMana { card: forest });
        checked(&mut g, Action::BeginConvoke);
        for bear in bears {
            let object = g.object_ref(bear);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution: ConvokeContribution::Generic,
                },
            );
        }
        assert!(g.convoke_choice().unwrap().can_pay);
        checked(
            &mut g,
            Action::PayCosts {
                payment: [2, 0, 0, 0, 1, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);

        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let giada = named(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        activate(&mut g, giada, 0, [1, 0, 0, 0, 0, 0]);
        let first = named(&mut g, p, "Healer's Hawk", Zone::Battlefield);
        let second = named(&mut g, p, "Healer's Hawk", Zone::Battlefield);
        let bears: Vec<_> = (0..4)
            .map(|_| named(&mut g, p, "Bear Cub", Zone::Battlefield))
            .collect();
        let celebrant = named(&mut g, p, "Sun-Dappled Celebrant", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: celebrant });
        checked(&mut g, Action::BeginConvoke);
        let object = g.object_ref(first);
        reject(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: ConvokeContribution::Generic,
            },
        );
        checked(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: ConvokeContribution::White,
            },
        );
        for bear in bears {
            let object = g.object_ref(bear);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution: ConvokeContribution::Generic,
                },
            );
        }
        assert!(!g.convoke_choice().unwrap().can_pay);
        reject(
            &mut g,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 0, 0],
            },
        );
        let object = g.object_ref(second);
        checked(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: ConvokeContribution::White,
            },
        );
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
    }
}

#[test]
fn tagged_mana_announced_producers_capture_quality_before_costs_and_departure() {
    for p in [Player::P0, Player::P1] {
        for (name, ability, production) in [
            ("Carnelian Orb of Dragonkind", 0, [0, 0, 0, 1, 0, 0]),
            ("Giada, Font of Hope", 0, [1, 0, 0, 0, 0, 0]),
            ("Secluded Courtyard", 1, [1, 0, 0, 0, 0, 0]),
        ] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            let source = named(
                &mut g,
                p,
                name,
                if name == "Secluded Courtyard" {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                },
            );
            if name == "Secluded Courtyard" {
                checked(&mut g, Action::PlayLand { card: source });
                checked(
                    &mut g,
                    Action::ChooseEntryValue {
                        value: EntryChoiceValue::CreatureType(
                            CreatureType::from_name("Angel").unwrap(),
                        ),
                    },
                );
            }
            checked(
                &mut g,
                Action::AnnounceMana {
                    card: source,
                    ability,
                    counter_payment: Counters::default(),
                    production,
                },
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(
                g.state
                    .mana_lots
                    .as_ref()
                    .unwrap()
                    .announcement_quality
                    .is_some()
            );
            reject(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 0, 1],
                },
            );
            checked(&mut g, Action::PayCosts { payment: [0; 6] });
            assert_eq!(g.state.players[p.index()].mana, production);
            assert!(
                g.state
                    .mana_lots
                    .as_ref()
                    .unwrap()
                    .announcement_quality
                    .is_none()
            );
            g.sacrifice_batch(&[(p, source)]);
            let (spell, payment) = if name == "Carnelian Orb of Dragonkind" {
                g.add_mana(p, [0, 0, 0, 5, 0, 0]);
                ("Shivan Dragon", [0, 0, 0, 6, 0, 0])
            } else {
                assert_eq!(g.available_mana(p, Purpose::Other), [0; 6]);
                g.add_mana(p, [1, 0, 0, 0, 0, 3]);
                ("Serra Angel", [2, 0, 0, 0, 0, 3])
            };
            let spell = named(&mut g, p, spell, Zone::Hand);
            checked(
                &mut g,
                Action::Cast {
                    card: spell,
                    target: None,
                    payment,
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
            assert_eq!(
                g.keywords(spell).contains(Keywords::HASTE),
                name == "Carnelian Orb of Dragonkind"
            );
        }
    }
}
