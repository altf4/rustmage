//! Paid complete-card programs, historical qualification and owned choice replay.
use super::ability_timeline_tests::{cast, checked, named, next_turn, setup};
use super::*;

fn catalogue_card(index: usize) -> Card {
    Card::catalogue().nth(index).unwrap()
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(before, g.snapshot());
}
fn drive(g: &mut Game, target: Option<CardId>, mode: Option<u16>, pay: bool, accept: bool) {
    g.prepare_triggers();
    for _ in 0..150 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        if let Pending::EffectChoice { player } = g.state.pending {
            assert!(g.observe(player.other()).actions.is_empty());
            for (id, object) in g.state.objects.iter().enumerate() {
                if object.zone == Zone::Hand && object.owner == player {
                    assert!(
                        !g.observe(player.other())
                            .objects
                            .iter()
                            .any(|o| o.reference.card == id as CardId)
                    );
                }
            }
        }
        let wanted = match g.state.pending {
            Pending::Priority => Some(Action::Pass),
            Pending::EffectOptional { .. } => Some(Action::ChooseOptional { accept }),
            Pending::ResolutionMana { .. } if !pay => Some(Action::DeclineResolutionMana),
            _ => None,
        };
        let action = wanted
            .or_else(|| {
                target.and_then(|card| {
                    actions
                        .iter()
                        .find(|a| {
                            **a == Action::ChooseTarget {
                                target: Target::Card(card),
                            }
                        })
                        .cloned()
                })
            })
            .or_else(|| {
                mode.and_then(|mode| {
                    actions
                        .iter()
                        .find(|a| **a == Action::ChooseMode { mode })
                        .cloned()
                })
            })
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| {
                        matches!(
                            a,
                            Action::PayResolutionMana { .. }
                                | Action::PutTrigger { .. }
                                | Action::FinishTargets
                                | Action::PayCosts { .. }
                                | Action::Discard { .. }
                                | Action::KeepLegend { .. }
                        )
                    })
                    .cloned()
            })
            .unwrap_or_else(|| panic!("unexpected {:?}: {actions:?}", g.state.pending));
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn attack(g: &mut Game, ids: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &card in ids {
        g.state.objects[card as usize].summoning_sick = false;
        checked(g, Action::Attack { card });
    }
    checked(g, Action::FinishAttackers);
}
fn count(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .filter(|s| s.card == id)
        .count()
}
fn tokens(g: &Game, c: Card, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.card == c && o.controller == p)
        .map(|(i, _)| i as CardId)
        .collect()
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    drive(g, None, None, false, true);
    id
}
#[test]
fn fdn_program_seven_printed_paid_costs_and_intrinsic_bodies() {
    for p in [Player::P0, Player::P1] {
        for (name, mana, power, toughness) in [
            ("Ashroot Animist", [0, 0, 0, 1, 1, 2], 4, 4),
            ("Jazal Goldmane", [2, 0, 0, 0, 0, 2], 4, 4),
            ("Kiora, the Rising Tide", [0, 1, 0, 0, 0, 2], 3, 2),
            ("Koma, World-Eater", [0, 2, 0, 0, 2, 3], 8, 12),
            ("Mentor of the Meek", [1, 0, 0, 0, 0, 2], 2, 2),
            ("Frenzied Goblin", [0, 0, 0, 1, 0, 0], 1, 1),
        ] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name, mana);
            assert_eq!(g.stats(id), Stats { power, toughness });
            let k = g.keywords(id);
            if name == "Ashroot Animist" || name == "Koma, World-Eater" {
                assert!(k.contains(Keywords::TRAMPLE));
            }
            if name == "Jazal Goldmane" {
                assert!(k.contains(Keywords::FIRST_STRIKE));
            }
        }
        let mut g = setup(p);
        let id = paid(&mut g, p, "Sylvan Scavenging", [0, 0, 0, 0, 2, 1]);
        assert!(g.object_view(id).kinds & 32 != 0);
    }
}
#[test]
fn fdn_program_seven_ashroot_live_lki_negative_and_target_revalidation() {
    for p in [Player::P0, Player::P1] {
        for (departure, negative, blink) in [
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Ashroot Animist", [0, 0, 0, 1, 1, 2]);
            let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            attack(&mut g, &[source]);
            g.prepare_triggers();
            if let Some(a) = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
            {
                checked(&mut g, a);
            }
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(source),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(ally),
                },
            );
            while g.state.announcing.is_some() {
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, a);
            }
            g.state.objects[source as usize].growth.power = if negative { -6 } else { 3 };
            if departure {
                g.move_object(source, Zone::Graveyard);
            }
            if blink {
                g.move_object(ally, Zone::Exile);
                g.move_object(ally, Zone::Battlefield);
            }
            drive(&mut g, None, None, false, true);
            assert_eq!(g.stats(ally).power, if negative || blink { 2 } else { 9 });
            assert_eq!(g.keywords(ally).contains(Keywords::TRAMPLE), !blink);
            if !blink {
                next_turn(&mut g);
                assert_eq!(g.stats(ally).power, 2);
                assert!(!g.keywords(ally).contains(Keywords::TRAMPLE));
            }
        }
    }
}
#[test]
fn fdn_program_seven_jazal_paid_activation_freezes_attackers_and_amount() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Jazal Goldmane", [2, 0, 0, 0, 0, 2]);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let rest = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, &[source, a, b]);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 3];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        while g.state.announcing.is_some() {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.move_object(b, Zone::Graveyard);
        drive(&mut g, None, None, false, true);
        assert_eq!(g.stats(source).power, 6);
        assert_eq!(g.stats(a).power, 4);
        assert_eq!(g.stats(rest).power, 2);
        g.state.attackers.retain(|attacker| attacker.card != a);
        assert_eq!(g.stats(a).power, 4);
        next_turn(&mut g);
        assert_eq!(g.stats(a).power, 2);
        let mut g = setup(p);
        let j = named(&mut g, p, "Jazal Goldmane", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.active = p.other();
        g.state.priority = p.other();
        attack(&mut g, &[enemy]);
        assert_eq!(
            g.stat_value(StatValue::AllAttackingCreatures, g.object_ref(j), p),
            1
        );
        assert_eq!(
            g.stat_value(StatValue::AttackingCreatures, g.object_ref(j), p),
            0
        );
    }
}
#[test]
fn fdn_program_seven_kiora_loot_private_choices_and_threshold_rechecked() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let before = g.hand_count(p);
        let k = paid(&mut g, p, "Kiora, the Rising Tide", [0, 1, 0, 0, 0, 2]);
        assert_eq!(g.hand_count(p), before);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 2);
        for (grave, remove, accept, expected) in [
            (6, false, true, 0),
            (7, true, true, 0),
            (7, false, false, 0),
            (7, false, true, 1),
        ] {
            let mut g = setup(p);
            let k = named(&mut g, p, "Kiora, the Rising Tide", Zone::Battlefield);
            let mut ids = Vec::new();
            for _ in 0..grave {
                ids.push(named(&mut g, p, "Forest", Zone::Graveyard));
            }
            attack(&mut g, &[k]);
            assert_eq!(count(&g, k), usize::from(grave >= 7));
            if remove {
                g.move_object(ids[0], Zone::Exile);
            }
            drive(&mut g, None, None, false, accept);
            let ts = tokens(&g, catalogue_card(5334), p);
            assert_eq!(ts.len(), expected);
            if let Some(&id) = ts.first() {
                let v = g.object_view(id);
                assert_eq!(
                    g.stats(id),
                    Stats {
                        power: 8,
                        toughness: 8
                    }
                );
                assert!(v.card.is_legendary());
                assert_eq!(v.card.token_info().unwrap().name, "Scion of the Deep");
                assert!(!v.attacking);
            }
        }
        assert_eq!(g.object_view(k).zone, Zone::Battlefield);
    }
}
#[test]
fn fdn_program_seven_mentor_entry_power_historical_optional_mana_and_other() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let m = paid(&mut g, p, "Mentor of the Meek", [1, 0, 0, 0, 0, 2]);
            assert_eq!(count(&g, m), 0);
            let creature = named(&mut g, p, "Bear Cub", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 1];
            cast(&mut g, creature, None, &[]);
            // Resolve the actual paid creature before changing its power: qualification is entry-time.
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(count(&g, m), 1);
            g.state.objects[creature as usize].growth.power = 7;
            let hand = g.hand_count(p);
            drive(&mut g, None, None, pay, true);
            assert_eq!(g.hand_count(p), hand + usize::from(pay));
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                u32::from(!pay)
            );
        }
        let mut g = setup(p);
        let m = named(&mut g, p, "Mentor of the Meek", Zone::Battlefield);
        let _anthem = named(&mut g, p, "Anthem of Champions", Zone::Battlefield);
        let c = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, c, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.stats(c).power, 3);
        assert_eq!(count(&g, m), 0);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        g.move_object(foe, Zone::Battlefield);
        assert_eq!(count(&g, m), 0);
    }
}
#[test]
fn fdn_program_seven_frenzied_target_before_payment_optional_red_and_fizzle() {
    for p in [Player::P0, Player::P1] {
        for (pay, blink) in [(false, false), (true, false), (true, true)] {
            let mut g = setup(p);
            let gob = paid(&mut g, p, "Frenzied Goblin", [0, 0, 0, 1, 0, 0]);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            attack(&mut g, &[gob]);
            g.prepare_triggers();
            if let Some(a) = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
            {
                checked(&mut g, a);
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            while g.state.announcing.is_some() {
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, a);
            }
            if blink {
                g.move_object(enemy, Zone::Exile);
                g.move_object(enemy, Zone::Battlefield);
            }
            drive(&mut g, None, None, pay, true);
            assert_eq!(
                g.keywords(enemy).contains(Keywords::CANT_BLOCK),
                pay && !blink
            );
            assert_eq!(g.state.players[p.index()].mana[3], u32::from(!pay || blink));
            if pay && !blink {
                let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
                g.state.players[p.index()].mana = [30; 6];
                cast(&mut g, stop, None, &[Target::Card(enemy)]);
                drive(&mut g, None, None, false, true);
                assert!(g.keywords(enemy).contains(Keywords::CANT_BLOCK));
            }
            next_turn(&mut g);
            assert!(!g.keywords(enemy).contains(Keywords::CANT_BLOCK));
        }
    }
}
#[test]
fn fdn_program_seven_sylvan_modes_targeting_and_resolution_only_ferocious() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let _s = paid(&mut g, p, "Sylvan Scavenging", [0, 0, 0, 0, 2, 1]);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.begin_step(Step::End);
        drive(&mut g, Some(bear), Some(0), false, true);
        assert_eq!(g.object_view(bear).counters.plus_one, 1);
        for (initial, remove, add, expected) in [
            (false, false, false, 0),
            (true, true, false, 0),
            (false, false, true, 1),
            (true, false, false, 1),
        ] {
            let mut g = setup(p);
            named(&mut g, p, "Sylvan Scavenging", Zone::Battlefield);
            let big = if initial {
                Some(named(&mut g, p, "Serra Angel", Zone::Battlefield))
            } else {
                None
            };
            g.begin_step(Step::End);
            g.prepare_triggers();
            if let Some(a) = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
            {
                checked(&mut g, a);
            }
            checked(&mut g, Action::ChooseMode { mode: 1 });
            while g.state.announcing.is_some() {
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, a);
            }
            if remove {
                g.move_object(big.unwrap(), Zone::Graveyard);
            }
            if add {
                named(&mut g, p, "Serra Angel", Zone::Battlefield);
            }
            drive(&mut g, None, None, false, true);
            let ts = tokens(&g, catalogue_card(5336), p);
            assert_eq!(ts.len(), expected);
            if let Some(&id) = ts.first() {
                assert_eq!(
                    g.stats(id),
                    Stats {
                        power: 3,
                        toughness: 3
                    }
                );
                assert_eq!(g.object_view(id).card.token_info().unwrap().colors, 16);
            }
        }
    }
}
#[test]
fn fdn_program_seven_koma_uncounterable_ward_four_and_combat_only_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = named(&mut g, p, "Koma, World-Eater", Zone::Hand);
        cast(&mut g, k, None, &[]);
        checked(&mut g, Action::Pass);
        let refute = named(&mut g, p.other(), "Refute", Zone::Hand);
        cast(&mut g, refute, None, &[Target::Spell(k)]);
        drive(&mut g, None, None, false, true);
        assert_eq!(g.object_view(refute).zone, Zone::Graveyard);
        assert_eq!(g.object_view(k).zone, Zone::Battlefield);
        for pay in [false, true] {
            let mut g = setup(p);
            let k = named(&mut g, p.other(), "Koma, World-Eater", Zone::Battlefield);
            let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
            cast(&mut g, growth, None, &[Target::Card(k)]);
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            drive(&mut g, None, None, pay, true);
            assert_eq!(g.stats(k).power, if pay { 11 } else { 8 });
            assert_eq!(
                mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                if pay { 4 } else { 0 }
            );
            assert_eq!(g.object_view(growth).zone, Zone::Graveyard);
        }
        let mut g = setup(p);
        let k = named(&mut g, p, "Koma, World-Eater", Zone::Battlefield);
        for combat in [false, true] {
            let event = DamageEvent {
                source: g.damage_source(k),
                target: Target::Player(p.other()),
                amount: 1,
                combat,
            };
            g.apply_damage_batch(&[event]);
            drive(&mut g, None, None, false, true);
            assert_eq!(
                tokens(&g, catalogue_card(5335), p).len(),
                if combat { 4 } else { 0 }
            );
        }
        for id in tokens(&g, catalogue_card(5335), p) {
            let c = g.object_view(id).card;
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
            assert_eq!(c.token_info().unwrap().name, "Koma's Coil");
            assert!(!c.is_legendary());
            assert_eq!(c.token_info().unwrap().colors, 2);
        }
        assert!(catalogue_card(5333).is_emblem());
        assert!(!catalogue_card(5333).is_token());
        for id in [5334, 5335, 5336] {
            assert!(catalogue_card(id).is_token());
            assert!(!catalogue_card(id).is_emblem());
        }
    }
}

#[test]
fn fdn_program_seven_appended_tokens_preserve_every_legacy_identity_and_legend_choices() {
    // Metadata append must preserve the factories before and after the emblem gap.
    let metadata: serde_json::Value =
        serde_json::from_str(include_str!("../data/catalogue-identities.json")).unwrap();
    let entries = metadata["tokens"].as_array().unwrap();
    let cards: Vec<_> = metadata["cards"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["category"] == "token")
        .collect();
    assert_eq!(entries.len(), cards.len());
    for (entry, card) in entries.iter().zip(cards) {
        let identity = catalogue_card(card["index"].as_u64().unwrap() as usize);
        let token = identity.token_info().unwrap();
        assert_eq!(
            token.factory_identity,
            entry["factory_identity"].as_str().unwrap()
        );
        assert_eq!(token.name, entry["name"].as_str().unwrap());
        assert_eq!(token.subtypes, entry["subtypes"].as_str().unwrap());
        assert_eq!(token.colors, entry["colors"].as_u64().unwrap() as u8);
    }
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.create_tokens(p, catalogue_card(5334), 2, false);
        g.state_based_actions();
        let ids = tokens(&g, catalogue_card(5334), p);
        assert_eq!(ids.len(), 2);
        let keep = g.object_ref(ids[0]);
        assert!(matches!(g.state.pending, Pending::Legend { .. }));
        checked(&mut g, Action::KeepLegend { object: keep });
        assert_eq!(tokens(&g, catalogue_card(5334), p), vec![ids[0]]);
        assert_eq!(g.object_view(ids[1]).zone, Zone::Outside);
    }
}
