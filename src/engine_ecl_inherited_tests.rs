//! Evidence audit of previously admitted physical ECL cards.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;
#[test]
fn ecl_inherited_basic_lands_play_once_generate_exact_color_and_pay_real_spells() {
    for p in [Player::P0, Player::P1] {
        for (color, land, spell) in [
            (0, "Plains", "Savannah Lions"),
            (1, "Island", "Unsummon"),
            (2, "Swamp", "Duress"),
            (3, "Mountain", "Shock"),
            (4, "Forest", "Llanowar Elves"),
        ] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            let first = named(&mut g, p, land, Zone::Hand);
            let second = named(&mut g, p, land, Zone::Hand);
            let spell = named(&mut g, p, spell, Zone::Hand);
            let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let visible = serde_json::to_value(g.observe(p.other())).unwrap();
            assert!(
                !g.observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.reference.card == first),
                "hidden hand land"
            );
            let before = g.snapshot();
            assert!(
                g.apply(p.other(), Action::PlayLand { card: first })
                    .is_err()
            );
            assert_eq!(g.snapshot(), before);
            checked(&mut g, Action::PlayLand { card: first });
            assert_eq!(g.object_view(first).zone, Zone::Battlefield);
            assert!(!g.object_view(first).tapped);
            assert!(
                !g.legal_actions()
                    .contains(&Action::PlayLand { card: second })
            );
            checked(&mut g, Action::TapMana { card: first });
            let mut mana = [0; 6];
            mana[color] = 1;
            assert_eq!(g.state.players[p.index()].mana, mana);
            assert!(g.state.stack.is_empty());
            let before = g.snapshot();
            assert!(g.apply(p, Action::TapMana { card: first }).is_err());
            assert_eq!(g.snapshot(), before);
            checked(&mut g, Action::AnnounceCast { card: spell });
            let actions = g.legal_actions();
            if actions.contains(&Action::ChooseTarget {
                target: Target::Card(target),
            }) {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
            } else if actions.contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            }) {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
            }
            for _ in 0..5 {
                if g.state.announcing.is_none() {
                    break;
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                    .unwrap();
                checked(&mut g, action);
            }
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_ne!(visible, serde_json::to_value(g.observe(p.other())).unwrap());
        }
    }
}
fn finish_entry_trigger(g: &mut Game, targets: &[Target]) {
    for _ in 0..60 {
        if matches!(g.state.pending, Pending::Targets { .. })
            && g.state
                .announcing
                .as_ref()
                .is_some_and(|a| matches!(a.item.kind, StackKind::TriggeredAbility { .. }))
        {
            break;
        }
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
                .unwrap()
        };
        checked(g, action);
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
}
#[test]
fn ecl_inherited_changeling_hybrid_pays_both_colors_and_tramples_a_real_blocker() {
    for p in [Player::P0, Player::P1] {
        for (cost, color) in [(0, 4), (1, 3)] {
            let mut g = setup(p);
            let creature = named(&mut g, p, "Gangly Stompling", Zone::Hand);
            assert!(g.has_subtype(creature, "Elf") && g.has_subtype(creature, "Merfolk"));
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[color] = 1;
            g.state.players[p.index()].mana[5] = 2;
            checked(&mut g, Action::AnnounceCast { card: creature });
            checked(&mut g, Action::ChooseSpellCost { cost });
            let payment = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, payment);
            settle(&mut g);
            assert_eq!(
                g.stats(creature),
                Stats {
                    power: 4,
                    toughness: 2
                }
            );
            assert!(g.keywords(creature).contains(Keywords::TRAMPLE));
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[creature as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: creature });
            checked(&mut g, Action::FinishAttackers);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            checked(
                &mut g,
                Action::Block {
                    blocker: bear,
                    attacker: creature,
                },
            );
            checked(&mut g, Action::FinishBlockers);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            if g.state.pending == Pending::Damage {
                checked(
                    &mut g,
                    Action::AssignDamage {
                        target: Target::Card(bear),
                        amount: 2,
                    },
                );
            }
            assert_eq!(g.state.players[p.other().index()].life, 18);
            assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn ecl_inherited_nomad_grave_exile_pays_before_token_and_obeys_sorcery_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let nomad = named(&mut g, p, "Goldmeadow Nomad", Zone::Hand);
        super::ability_timeline_tests::cast(&mut g, nomad, None, &[]);
        settle(&mut g);
        assert_eq!(
            g.stats(nomad),
            Stats {
                power: 1,
                toughness: 2
            }
        );
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        super::ability_timeline_tests::cast(&mut g, murder, None, &[Target::Card(nomad)]);
        settle(&mut g);
        let action = Action::AnnounceActivate {
            card: nomad,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.step = Step::BeginCombat;
        assert!(!g.legal_actions().contains(&action));
        g.state.step = Step::Main1;
        checked(&mut g, action.clone());
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(g.object_view(nomad).zone, Zone::Exile);
        assert!(!g.legal_actions().contains(&action));
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card.has_subtype("Kithkin")
            })
            .map(|(i, _)| i as CardId)
            .unwrap();
        assert_eq!(
            g.stats(token),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(g.object_view(token).colors, 17);
    }
}
#[test]
fn ecl_inherited_merrow_all_hybrid_routes_create_exact_white_blue_token() {
    for p in [Player::P0, Player::P1] {
        for cost in 0..3 {
            let mut g = setup(p);
            let merrow = named(&mut g, p, "Merrow Skyswimmer", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: merrow });
            checked(&mut g, Action::ChooseSpellCost { cost });
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, pay);
            settle(&mut g);
            assert!(
                g.keywords(merrow)
                    .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
            );
            let token = g
                .state
                .objects
                .iter()
                .enumerate()
                .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
                .map(|(i, _)| i as CardId)
                .unwrap();
            assert_eq!(g.object_view(token).colors, 3);
            assert_eq!(
                g.stats(token),
                Stats {
                    power: 1,
                    toughness: 1
                }
            );
            assert!(g.has_subtype(token, "Merfolk"));
        }
    }
}
#[test]
fn ecl_inherited_flock_flash_other_owned_bounce_and_rooftop_partial_grave_exile() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[target as usize].controller = p;
        let flock = named(&mut g, p, "Flock Impostor", Zone::Hand);
        g.state.active = p.other();
        g.state.step = Step::BeginCombat;
        super::ability_timeline_tests::cast(&mut g, flock, None, &[]);
        finish_entry_trigger(&mut g, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Hand);
        assert_eq!(g.object_view(target).owner, p.other());
        assert!(g.has_subtype(flock, "Goblin"));
        assert!(g.keywords(flock).contains(Keywords::FLYING));
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        let first = named(&mut g, p, "Forest", Zone::Graveyard);
        let second = named(&mut g, p.other(), "Mountain", Zone::Graveyard);
        let rooftop = named(&mut g, p, "Rooftop Percher", Zone::Hand);
        super::ability_timeline_tests::cast(&mut g, rooftop, None, &[]);
        finish_entry_trigger(&mut g, &[Target::Card(first), Target::Card(second)]);
        g.move_object(first, Zone::Exile);
        let life = g.state.players[p.index()].life;
        settle(&mut g);
        assert_eq!(g.object_view(second).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].life, life + 3);
        assert!(g.has_subtype(rooftop, "Elf"));
    }
}
#[test]
fn ecl_inherited_trigger_and_activation_bodies_have_paid_entry_evidence_both_players() {
    for p in [Player::P0, Player::P1] {
        for (name, power, toughness, keywords, counters) in [
            ("Boggart Prankster", 1, 3, Keywords::NONE, 0),
            ("Enraged Flamecaster", 3, 2, Keywords::REACH, 0),
            ("Kulrath Mystic", 2, 4, Keywords::NONE, 0),
            ("Gnarlbark Elm", 1, 2, Keywords::NONE, 2),
            ("Iron-Shield Elf", 3, 1, Keywords::NONE, 0),
        ] {
            let mut g = setup(p);
            let source = named(&mut g, p, name, Zone::Hand);
            super::ability_timeline_tests::cast(&mut g, source, None, &[]);
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(g.stats(source), Stats { power, toughness });
            assert_eq!(g.object_view(source).keywords, keywords);
            assert_eq!(g.object_view(source).counters.minus_one, counters);
        }
    }
}
