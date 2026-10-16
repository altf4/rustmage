use super::tests::{act, cast, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(expected, fork.snapshot());
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(saved, g.snapshot());
    assert_eq!(rev, g.revision());
}
fn declaration(g: &mut Game, cards: &[CardId]) {
    g.state.attackers.clear();
    g.state.blocks.clear();
    g.state.blocking.clear();
    g.state.selected_attackers.clear();
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &id in cards {
        g.state.objects[id as usize].tapped = false;
        replay(g, Action::Attack { card: id });
    }
    replay(g, Action::FinishAttackers);
}
fn main2(g: &mut Game) {
    g.state.attackers.clear();
    g.state.step = Step::Main2;
    g.give_priority();
}
fn triggers(g: &mut Game, target: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..20 {
        if g.state.pending == Pending::Priority {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a, Action::PutTrigger { .. } | Action::FinishTargets)
                    || target.is_some_and(|t| *a == Action::ChooseTarget { target: t })
            })
            .unwrap();
        replay(g, a);
    }
    panic!("trigger announcement stalled");
}
#[test]
fn history_records_only_committed_attackers_and_survives_departure_combat_and_restore() {
    let mut g = scenario();
    let left = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let right = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(&mut g, Action::Attack { card: left });
    assert_eq!(g.observe(Player::P0).creatures_attacked_this_turn, 0);
    replay(&mut g, Action::Attack { card: right });
    assert_eq!(g.observe(Player::P1).creatures_attacked_this_turn, 0);
    replay(&mut g, Action::FinishAttackers);
    assert_eq!(g.observe(Player::P0).creatures_attacked_this_turn, 2);
    assert_eq!(g.observe(Player::P1).creatures_attacked_this_turn, 2);
    g.move_object(left, Zone::Graveyard);
    g.state.step = Step::EndCombat;
    g.advance_step();
    assert!(g.state.attackers.is_empty());
    assert_eq!(g.state.attacked_this_turn.len(), 2);
    let saved = g.snapshot();
    let mut fork = g.clone();
    fork.state.active = Player::P1;
    fork.begin_turn();
    assert_eq!(fork.observe(Player::P0).creatures_attacked_this_turn, 0);
    assert_eq!(g.state.attacked_this_turn.len(), 2);
    g.state.active = Player::P1;
    g.begin_turn();
    assert_eq!(g.state.attacked_this_turn.len(), 0);
    g.restore(&saved).unwrap();
    assert_eq!(g.state.attacked_this_turn.len(), 2);
}
#[test]
fn same_object_deduplicates_but_reentered_generation_counts_again() {
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Stormcatch Mentor", Zone::Battlefield);
    declaration(&mut g, &[id]);
    let reference = g.object_ref(id);
    declaration(&mut g, &[id]);
    assert_eq!(g.state.attacked_this_turn.len(), 1);
    g.move_object(id, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    assert_ne!(g.object_ref(id), reference);
    declaration(&mut g, &[id]);
    assert_eq!(g.state.attacked_this_turn.len(), 2);
}
#[test]
fn reused_token_slot_retains_old_attack_and_counts_new_generation() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Thrumming Hivepool", Zone::Battlefield);
    let card = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|i| i.name == "Sliver Token"))
        .unwrap();
    g.create_tokens(Player::P0, card, 1, false);
    let id = g
        .state
        .objects
        .iter()
        .position(|o| o.card == card && o.zone == Zone::Battlefield)
        .unwrap() as CardId;
    declaration(&mut g, &[id]);
    let reference = g.object_ref(id);
    g.move_object(id, Zone::Graveyard);
    g.state_based_actions();
    assert_eq!(g.object_view(id).zone, Zone::Outside);
    g.create_tokens(Player::P0, card, 1, false);
    assert_ne!(g.object_ref(id), reference);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    declaration(&mut g, &[id]);
    assert_eq!(g.observe(Player::P1).creatures_attacked_this_turn, 2);
}
#[test]
fn empty_or_illegal_declarations_never_inflate_history() {
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].summoning_sick = true;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    reject(&mut g, Action::Attack { card: id });
    assert_eq!(g.state.attacked_this_turn.len(), 0);
    replay(&mut g, Action::FinishAttackers);
    assert_eq!(g.state.attacked_this_turn.len(), 0);
    assert!(!g.state.players[0].attacked_this_turn);
}
#[test]
fn global_attack_discounts_include_departed_attackers_and_either_spell_controller() {
    for actor in [Player::P0, Player::P1] {
        for name in ["Rowdy Research", "Witchstalker Frenzy"] {
            let mut g = scenario();
            let ids: Vec<_> = (0..3)
                .map(|_| named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield))
                .collect();
            let spell = named(&mut g, actor, name, Zone::Hand);
            g.state.players[actor.index()].mana = if name == "Rowdy Research" {
                [0, 1, 0, 0, 0, 3]
            } else {
                [0, 0, 0, 1, 0, 0]
            };
            if actor == Player::P0 {
                reject(&mut g, Action::AnnounceCast { card: spell });
            }
            declaration(&mut g, &ids);
            g.move_object(ids[0], Zone::Graveyard);
            g.move_object(ids[1], Zone::Graveyard);
            main2(&mut g);
            if actor == Player::P1 {
                replay(&mut g, Action::Pass);
            }
            let target = if name == "Witchstalker Frenzy" {
                Some(Target::Card(ids[2]))
            } else {
                None
            };
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(a,Action::Cast{card,..} if *card==spell)
                        && match a {
                            Action::Cast { target: t, .. } => *t == target,
                            _ => false,
                        }
                })
                .unwrap();
            replay(&mut g, a);
            passes(&mut g);
            if name == "Rowdy Research" {
                assert_eq!(g.state.players[actor.index()].library.len(), 17);
            } else {
                assert_eq!(g.object_view(ids[2]).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn staged_attack_discount_validates_exact_payment_and_retains_colored_cost() {
    let mut g = scenario();
    let ids: Vec<_> = (0..3)
        .map(|_| named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield))
        .collect();
    declaration(&mut g, &ids);
    main2(&mut g);
    let spell = named(&mut g, Player::P0, "Rowdy Research", Zone::Hand);
    g.state.players[0].mana[5] = 4;
    reject(&mut g, Action::AnnounceCast { card: spell });
    g.state.players[0].mana = [0, 1, 0, 0, 0, 3];
    replay(&mut g, Action::AnnounceCast { card: spell });
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 3);
    reject(
        &mut g,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 2],
        },
    );
    replay(
        &mut g,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 3],
        },
    );
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), 17);
}
#[test]
fn raid_draw_creatures_check_their_controllers_committed_attack_flag() {
    for name in ["Skyship Buccaneer", "Storm Fleet Spy"] {
        for raid in [false, true] {
            let mut g = scenario();
            let attacker = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            if raid {
                declaration(&mut g, &[attacker]);
                main2(&mut g);
            } else {
                g.state.players[1].attacked_this_turn = true;
            }
            let spell = named(&mut g, Player::P0, name, Zone::Hand);
            g.state.players[0].mana = [0, 2, 0, 0, 0, 5];
            cast(&mut g, spell, None);
            passes(&mut g);
            triggers(&mut g, None);
            if raid {
                passes(&mut g);
            } else {
                assert!(g.state.stack.is_empty());
            }
            assert_eq!(g.state.players[0].library.len(), if raid { 19 } else { 20 });
            assert_eq!(
                g.keywords(spell).contains(Keywords::FLYING),
                name == "Skyship Buccaneer"
            );
        }
    }
}
#[test]
fn raid_clue_and_damage_bodies_execute_only_after_an_owned_attack() {
    for name in ["Fire Nation Raider", "Gorehorn Raider"] {
        for raid in [false, true] {
            let mut g = scenario();
            let attacker = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            if raid {
                declaration(&mut g, &[attacker]);
                main2(&mut g);
            }
            let spell = named(&mut g, Player::P0, name, Zone::Hand);
            g.state.players[0].mana = [0, 0, 0, 1, 0, 4];
            cast(&mut g, spell, None);
            passes(&mut g);
            triggers(&mut g, Some(Target::Player(Player::P1)));
            if raid {
                passes(&mut g);
            } else {
                assert!(g.state.stack.is_empty());
            }
            assert_eq!(
                g.state.players[1].life,
                if raid && name == "Gorehorn Raider" {
                    18
                } else {
                    20
                }
            );
            let clues = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.card.token_info().is_some_and(|i| i.name == "Clue Token")
                })
                .count();
            assert_eq!(clues, usize::from(raid && name == "Fire Nation Raider"));
            assert_eq!(
                (g.stats(spell).power, g.stats(spell).toughness),
                if name == "Fire Nation Raider" {
                    (4, 2)
                } else {
                    (4, 4)
                }
            );
        }
    }
}
#[test]
fn chart_a_course_draws_once_then_requires_private_discard_only_without_raid() {
    for raid in [false, true] {
        let mut g = scenario();
        if raid {
            let attacker = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            declaration(&mut g, &[attacker]);
            main2(&mut g);
        }
        let spell = named(&mut g, Player::P0, "Chart a Course", Zone::Hand);
        g.state.players[0].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, spell, None);
        passes(&mut g);
        assert_eq!(g.state.players[0].library.len(), 18);
        if raid {
            assert_eq!(g.state.pending, Pending::Priority);
            assert_eq!(g.hand_count(Player::P0), 2);
        } else {
            assert_eq!(g.observe(Player::P0).decision, "effect_discard");
            assert!(g.observe(Player::P1).actions.is_empty());
            assert!(g.observe(Player::P1).effect_discards.is_empty());
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Discard { .. }))
                .unwrap();
            replay(&mut g, a);
            assert_eq!(g.state.players[0].library.len(), 18);
            assert_eq!(g.hand_count(Player::P0), 1);
        }
    }
}

#[test]
fn every_attack_event_sees_the_complete_tapped_declared_batch_in_either_order() {
    for reverse in [false, true] {
        for vigilant in [false, true] {
            let mut g = scenario();
            let first = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            let second = named(
                &mut g,
                Player::P0,
                if vigilant { "Serra Angel" } else { "Bear Cub" },
                Zone::Battlefield,
            );
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            let cards = if reverse {
                [second, first]
            } else {
                [first, second]
            };
            for card in cards {
                replay(&mut g, Action::Attack { card });
            }
            let condition = TriggerCondition::InterveningIf {
                guard: StepGuard::Threshold {
                    value: StepValue::TappedCreatures,
                    minimum: Some(2),
                    maximum: None,
                },
                event: &TriggerCondition::SelfAttacks,
            };
            let mut events = Vec::new();
            g.commit_attackers_with(Player::P0, |game, id| {
                let source = game.object_view(id);
                assert!(source.attacking);
                assert_eq!(game.state.attackers.len(), 2);
                assert_eq!(game.state.attacked_this_turn.len(), 2);
                assert_eq!(
                    game.trigger_guards_hold(
                        condition,
                        Player::P0,
                        source.reference,
                        source.tapped
                    ),
                    !vigilant
                );
                assert!(game.object_view(first).tapped);
                assert_eq!(game.object_view(second).tapped, !vigilant);
                events.push(id);
            });
            assert_eq!(events, cards);
        }
    }
}
