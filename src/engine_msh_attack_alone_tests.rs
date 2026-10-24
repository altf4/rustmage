//! Paid rules evidence for the controlled-creature attacks-alone event.
use super::ability_world::CharacteristicWorld;
use super::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn attackers(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
}
fn declare(g: &mut Game, p: Player, ids: &[CardId]) {
    attackers(g, p);
    for &card in ids {
        checked(g, Action::Attack { card });
    }
    checked(g, Action::FinishAttackers);
}
fn ready(g: &mut Game, ids: &[CardId]) {
    for &id in ids {
        g.state.objects[id as usize].summoning_sick = false;
    }
}
fn clues(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card.identity() == "@token/a5a792437637731d892720f8")
                .then_some(id as CardId)
        })
        .collect()
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn visible(g: &Game, p: Player, id: CardId) -> bool {
    g.observe(p).objects.iter().any(|o| o.reference.card == id)
}

#[test]
fn msh_attack_alone_paid_observers_buff_exact_other_attacker_and_expire() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let agent = paid(&mut g, p, "Agent 13, Sharon Carter");
        let shield = paid(&mut g, p, "Agents of S.H.I.E.L.D.");
        let widow = paid(&mut g, p, "Black Widow, Double Agent");
        let bear = paid(&mut g, p, "Bear Cub");
        // Age the paid creatures through real turn transitions.
        next_turn(&mut g);
        next_turn(&mut g);
        declare(&mut g, p, &[bear]);
        settle(&mut g);
        assert_eq!(stats(&g, bear), (3, 3));
        assert!(
            g.keywords(bear)
                .contains(Keywords::FIRST_STRIKE.union(Keywords::MENACE))
        );
        assert!(!g.keywords(bear).contains(Keywords::DEATHTOUCH));
        assert_eq!(clues(&g, p).len(), 1);
        for observer in [agent, shield, widow] {
            assert!(!g.object_view(observer).tapped);
            assert!(!g.state.attackers.iter().any(|a| a.card == observer));
        }
        assert_eq!(stats(&g, agent), (3, 2));
        assert_eq!(stats(&g, shield), (2, 4));
        assert_eq!(stats(&g, widow), (3, 2));
        next_turn(&mut g);
        assert_eq!(stats(&g, bear), (2, 2));
        assert!(!g.keywords(bear).contains(Keywords::FIRST_STRIKE));
        assert!(!g.keywords(bear).contains(Keywords::MENACE));
    }
}

#[test]
fn msh_attack_alone_self_observers_and_actual_clue_sacrifice_draw_privacy() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Agent 13, Sharon Carter",
            "Agents of S.H.I.E.L.D.",
            "Black Widow, Double Agent",
        ] {
            let mut g = setup(p);
            let source = paid(&mut g, p, name);
            ready(&mut g, &[source]);
            declare(&mut g, p, &[source]);
            settle(&mut g);
            match name {
                "Agent 13, Sharon Carter" => {
                    let clue = clues(&g, p)[0];
                    let hand = g.hand_count(p);
                    let drawn = *g.state.players[p.index()].library.last().unwrap();
                    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
                    checked(
                        &mut g,
                        Action::AnnounceActivate {
                            card: clue,
                            ability: 0,
                            counter_payment: Counters::default(),
                        },
                    );
                    let payment = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayCosts { .. }))
                        .unwrap();
                    checked(&mut g, payment);
                    settle(&mut g);
                    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                    assert_eq!(g.object_view(clue).zone, Zone::Outside);
                    assert_eq!(g.hand_count(p), hand + 1);
                    assert!(visible(&g, p, drawn));
                    assert!(!visible(&g, p.other(), drawn));
                }
                "Agents of S.H.I.E.L.D." => assert_eq!(stats(&g, source), (3, 5)),
                _ => assert!(
                    g.keywords(source).contains(
                        Keywords::DEATHTOUCH
                            .union(Keywords::FIRST_STRIKE)
                            .union(Keywords::MENACE)
                    )
                ),
            }
        }
    }
}

#[test]
fn msh_attack_alone_only_one_declared_creature_and_our_controller_qualify() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = setup(p);
            paid(&mut g, p, "Agent 13, Sharon Carter");
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            declare(&mut g, p, &[a, b][..count]);
            if count == 2 {
                g.move_object(b, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(clues(&g, p).len(), usize::from(count == 1));
        }
        let mut g = setup(p);
        paid(&mut g, p, "Agent 13, Sharon Carter");
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        declare(&mut g, p.other(), &[enemy]);
        settle(&mut g);
        assert!(clues(&g, p).is_empty());
        assert!(clues(&g, p.other()).is_empty());
    }
}

#[test]
fn msh_attack_alone_already_attacking_tokens_do_not_change_declaration_event() {
    for p in [Player::P0, Player::P1] {
        for before in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, "Agent 13, Sharon Carter");
            paid(&mut g, p, "Agents of S.H.I.E.L.D.");
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let token = Card::catalogue().nth(5388).unwrap();
            // A token put onto the battlefield attacking was never declared.
            if before {
                g.create_token_batch_capture(
                    p,
                    token,
                    1,
                    true,
                    Some(&[CombatDefender::Player(p.other())]),
                    true,
                );
            }
            declare(&mut g, p, &[bear]);
            if !before {
                g.create_token_batch_capture(
                    p,
                    token,
                    1,
                    true,
                    Some(&[CombatDefender::Player(p.other())]),
                    true,
                );
            }
            settle(&mut g);
            assert_eq!(g.state.attackers.len(), 2);
            assert_eq!(stats(&g, bear), (3, 3));
            assert_eq!(clues(&g, p).len(), 1);
        }
    }
}

#[test]
fn msh_attack_alone_frozen_attacker_and_captured_controller_survive_departures() {
    for p in [Player::P0, Player::P1] {
        for variant in 0..4 {
            let mut g = setup(p);
            let agent = paid(&mut g, p, "Agent 13, Sharon Carter");
            let shield = paid(&mut g, p, "Agents of S.H.I.E.L.D.");
            let widow = paid(&mut g, p, "Black Widow, Double Agent");
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            declare(&mut g, p, &[bear]);
            match variant {
                0 => {
                    for id in [agent, shield, widow] {
                        g.move_object(id, Zone::Exile);
                    }
                }
                1 => {
                    for id in [agent, shield, widow] {
                        g.state.objects[id as usize].controller = p.other();
                    }
                }
                2 => {
                    let old = g.object_ref(bear);
                    g.move_object(bear, Zone::Exile);
                    g.move_object(bear, Zone::Battlefield);
                    assert_ne!(g.object_ref(bear), old);
                }
                _ => g.state.objects[bear as usize].controller = p.other(),
            }
            settle(&mut g);
            assert_eq!(clues(&g, p).len(), 1);
            assert!(clues(&g, p.other()).is_empty());
            assert_eq!(stats(&g, bear), if variant == 2 { (2, 2) } else { (3, 3) });
            assert_eq!(
                g.keywords(bear)
                    .contains(Keywords::FIRST_STRIKE.union(Keywords::MENACE)),
                variant != 2
            );
        }
    }
}

#[test]
fn msh_attack_alone_printed_ability_suppression_checked_at_event_only() {
    for p in [Player::P0, Player::P1] {
        for before in [false, true] {
            let mut g = setup(p);
            let agent = paid(&mut g, p, "Agent 13, Sharon Carter");
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
            if before {
                cast(&mut g, moon, None, &[Target::Card(agent)]);
                settle(&mut g);
            }
            declare(&mut g, p, &[bear]);
            if !before {
                // The source is blanked after its trigger has been created.
                let aura = named(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(agent));
            }
            settle(&mut g);
            assert_eq!(clues(&g, p).len(), usize::from(!before));
        }
    }
}

#[test]
fn msh_attack_alone_hydra_paid_etb_opponent_discards_private_hand_and_minimum() {
    for p in [Player::P0, Player::P1] {
        for count in 0usize..=3 {
            let mut g = setup(p);
            let cards: Vec<_> = (0..count)
                .map(|_| named(&mut g, p.other(), "Bear Cub", Zone::Hand))
                .collect();
            let source = named(&mut g, p, "HYDRA Infiltration", Zone::Hand);
            cast(&mut g, source, None, &[]);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Player(p.other()),
                })
            });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            g.move_object(source, Zone::Exile);
            for &card in cards.iter().take(2) {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectDiscard { .. })
                });
                assert_eq!(g.actor(), Some(p.other()));
                for &hidden in &cards {
                    if g.object_view(hidden).zone == Zone::Hand {
                        assert!(!visible(&g, p, hidden));
                        assert!(visible(&g, p.other(), hidden));
                    }
                }
                reject(&mut g, Action::Discard { card: source });
                checked(&mut g, Action::Discard { card });
            }
            settle(&mut g);
            for &card in cards.iter().take(2) {
                assert_eq!(g.object_view(card).zone, Zone::Graveyard);
                assert!(visible(&g, p, card));
                assert!(visible(&g, p.other(), card));
            }
            assert_eq!(g.hand_count(p.other()), count.saturating_sub(2));
            assert_eq!(g.object_view(source).zone, Zone::Exile);
        }
    }
}

#[test]
fn msh_attack_alone_hydra_drain_target_fizzle_and_captured_controller() {
    for p in [Player::P0, Player::P1] {
        for variant in 0..5 {
            let mut g = setup(p);
            let source = paid(&mut g, p, "HYDRA Infiltration");
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let life = [g.life(p), g.life(p.other())];
            if variant == 4 {
                named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
            }
            declare(&mut g, p, &[bear]);
            if variant != 4 {
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    })
                });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    checked(&mut g, Action::FinishTargets);
                }
                match variant {
                    0 => {
                        g.move_object(source, Zone::Exile);
                    }
                    1 => g.state.objects[source as usize].controller = p.other(),
                    2 => {
                        named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
                    }
                    _ => (),
                }
            }
            settle(&mut g);
            let resolves = variant < 2 || variant == 3;
            assert_eq!(g.life(p), life[0] + i128::from(resolves));
            assert_eq!(g.life(p.other()), life[1] - i128::from(resolves));
        }
    }
}

#[test]
fn msh_attack_alone_planeswalker_defender_and_investigate_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Agent 13, Sharon Carter");
        paid(&mut g, p, "Doubling Season");
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let walker = named(
            &mut g,
            p.other(),
            "Liliana, Dreadhorde General",
            Zone::Battlefield,
        );
        g.effect_counter(
            walker,
            CounterKind::Loyalty,
            6,
            replacements::ResumeMode::Pending,
        );
        attackers(&mut g, p);
        let defender = g.object_ref(walker);
        checked(
            &mut g,
            Action::AttackPlaneswalker {
                card: bear,
                defender,
            },
        );
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(
            g.state.attackers[0].defender,
            Some(CombatDefender::Planeswalker(defender))
        );
        assert_eq!(clues(&g, p).len(), 2);
        assert!(clues(&g, p.other()).is_empty());
    }
}

#[test]
fn msh_attack_alone_widow_first_strike_deathtouch_and_menace_real_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let widow = paid(&mut g, p, "Black Widow, Double Agent");
        let first = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let second = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        ready(&mut g, &[widow]);
        declare(&mut g, p, &[widow]);
        settle(&mut g);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: first,
                attacker: widow,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                blocker: second,
                attacker: widow,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.pending == Pending::Damage);
        checked(
            &mut g,
            Action::AssignDamage {
                target: Target::Card(first),
                amount: 1,
            },
        );
        checked(
            &mut g,
            Action::AssignDamage {
                target: Target::Card(second),
                amount: 2,
            },
        );
        until(&mut g, |g| {
            g.object_view(first).zone == Zone::Graveyard
                && g.object_view(second).zone == Zone::Graveyard
        });
        assert_eq!(g.object_view(widow).zone, Zone::Battlefield);
        assert_eq!(g.object_view(widow).damage, 0);
    }
}
