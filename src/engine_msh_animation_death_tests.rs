//! Paid whole-card animation and exact death-card returns, both seats and owned replay.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const IRON: &str = "I Am Iron Man";
const ARES: &str = "Ares, God of War";
#[track_caller]
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..180 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?} at {:?}", g.state.pending, g.state.step));
        checked(g, a);
    }
    panic!("scenario stalled {:?}", g.state.pending);
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn iron(g: &mut Game, p: Player, target: CardId) -> CardId {
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    let id = paid(g, p, IRON, &[Target::Card(target)]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn ares(g: &mut Game, p: Player) -> CardId {
    g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 1];
    let id = paid(g, p, ARES, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    g.state.players[p.index()].mana = [30; 6];
    id
}
fn ready(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    until(g, |g| {
        g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
    assert_eq!(g.state.active, p);
    g.state.players[p.index()].mana = [30; 6];
}
fn attack(g: &mut Game, ids: &[CardId]) {
    until(g, |g| g.state.pending == Pending::Attackers);
    for &card in ids {
        checked(g, Action::Attack { card });
    }
    checked(g, Action::FinishAttackers);
}
fn kill(g: &mut Game, p: Player, target: CardId) {
    priority(g, p);
    g.state.players[p.index()].mana = [30; 6];
    let id = named(g, p, "Murder", Zone::Hand);
    cast(g, id, None, &[Target::Card(target)]);
    until(g, |g| g.object_view(target).zone != Zone::Battlefield);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let before = g.snapshot();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
#[test]
fn msh_animation_death_iron_paid_artifacts_creatures_either_controller_draws_for_caster() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            for name in ["Bear Cub", "S.H.I.E.L.D. Spy Kit"] {
                let mut g = setup(p);
                let target = named(&mut g, owner, name, Zone::Battlefield);
                let library = g.state.players[p.index()].library.len();
                let colors = g.object_view(target).colors;
                iron(&mut g, p, target);
                let view = g.object_view(target);
                assert!(view.is_creature());
                assert_eq!(view.kinds & 16, 16);
                assert_eq!((g.stats(target).power, g.stats(target).toughness), (4, 4));
                assert!(g.keywords(target).contains(Keywords::FLYING));
                assert_eq!(g.object_view(target).colors, colors);
                assert_eq!(view.controller, owner);
                assert_eq!(g.state.players[p.index()].library.len(), library - 1);
                if name == "Bear Cub" {
                    assert!(g.has_subtype(target, "Bear"));
                } else {
                    assert!(g.has_subtype(target, "Equipment"));
                }
            }
        }
    }
}
#[test]
fn msh_animation_death_iron_rejects_nonartifact_noncreature_and_extra_target_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let enchant = named(&mut g, p, "Spelunking", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, IRON, Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        for target in [
            Target::Card(forest),
            Target::Card(enchant),
            Target::Player(p),
        ] {
            reject(&mut g, Action::ChooseTarget { target });
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 4);
    }
}
#[test]
fn msh_animation_death_iron_counters_modifiers_and_characteristics_survive_then_expire() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, "Snakeskin Veil", &[Target::Card(bear)]);
        paid(&mut g, p, "Giant Growth", &[Target::Card(bear)]);
        assert_eq!(g.stats(bear).power, 6);
        iron(&mut g, p, bear);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (8, 8));
        assert!(
            g.keywords(bear)
                .contains(Keywords::HEXPROOF.union(Keywords::FLYING))
        );
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        next_turn(&mut g);
        assert_eq!(g.stats(bear).power, 3);
        assert_eq!(g.object_view(bear).kinds & 16, 0);
        assert!(!g.keywords(bear).contains(Keywords::FLYING));
        assert!(g.has_subtype(bear, "Bear"));
    }
}
#[test]
fn msh_animation_death_iron_real_ability_removal_timestamp_before_and_after() {
    for p in [Player::P0, Player::P1] {
        for removal_first in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            if removal_first {
                paid(&mut g, p, "Stop Cold", &[Target::Card(bear)]);
            }
            iron(&mut g, p, bear);
            if !removal_first {
                g.state.players[p.index()].mana = [30; 6];
                paid(&mut g, p, "Stop Cold", &[Target::Card(bear)]);
            }
            assert_eq!(g.stats(bear).power, 4);
            assert_eq!(g.object_view(bear).kinds & 16, 16);
            assert_eq!(g.keywords(bear).contains(Keywords::FLYING), removal_first);
        }
    }
}
#[test]
fn msh_animation_death_iron_opponent_hexproof_response_fizzles_whole_spell_no_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, IRON, Zone::Hand);
        let library = g.state.players[p.index()].library.len();
        cast(&mut g, id, None, &[Target::Card(bear)]);
        priority(&mut g, p.other());
        let veil = named(&mut g, p.other(), "Snakeskin Veil", Zone::Hand);
        cast(&mut g, veil, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 3);
        assert_eq!(g.object_view(bear).kinds & 16, 0);
        assert_eq!(g.state.players[p.index()].library.len(), library);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_animation_death_iron_exact_target_departure_reentry_cannot_bind_or_draw() {
    for p in [Player::P0, Player::P1] {
        for return_again in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let old = g.object_ref(bear);
            let id = named(&mut g, p, IRON, Zone::Hand);
            let library = g.state.players[p.index()].library.len();
            cast(&mut g, id, None, &[Target::Card(bear)]);
            g.move_object(bear, Zone::Exile);
            if return_again {
                g.move_object(bear, Zone::Battlefield);
            }
            settle(&mut g);
            assert_ne!(g.object_ref(bear), old);
            assert_eq!(g.state.players[p.index()].library.len(), library);
            assert_eq!(g.object_view(bear).kinds & 16, 0);
        }
    }
}
#[test]
fn msh_animation_death_iron_land_types_and_real_mana_ability_survive_animation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Crawling Barrens", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: land });
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        iron(&mut g, p, land);
        assert_eq!(g.object_view(land).kinds & 19, 19);
        assert_eq!(g.stats(land).power, 6);
        assert!(g.has_subtype(land, "Elemental"));
        assert!(g.keywords(land).contains(Keywords::FLYING));
        // The new creature retains summoning sickness from the actual land entry.
        reject(&mut g, Action::TapMana { card: land });
        next_turn(&mut g);
        assert_eq!(g.object_view(land).kinds & 19, 2);
        assert_eq!(g.counter_count(land, CounterKind::PlusOne), 2);
    }
}
#[test]
fn msh_animation_death_iron_can_cast_in_combat_and_flying_changes_actual_block_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attack(&mut g, &[bear]);
        iron(&mut g, p, bear);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: bear,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.players[p.other().index()].life == 16);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
#[test]
fn msh_animation_death_iron_target_control_change_keeps_spell_controller_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, IRON, Zone::Hand);
        let libraries = [
            g.state.players[0].library.len(),
            g.state.players[1].library.len(),
        ];
        cast(&mut g, id, None, &[Target::Card(bear)]);
        g.change_controller(g.object_ref(bear), p.other());
        settle(&mut g);
        assert_eq!(g.object_view(bear).controller, p.other());
        assert_eq!(g.stats(bear).power, 4);
        assert_eq!(
            g.state.players[p.index()].library.len(),
            libraries[p.index()] - 1
        );
        assert_eq!(
            g.state.players[p.other().index()].library.len(),
            libraries[p.other().index()]
        );
    }
}
#[test]
fn msh_animation_death_iron_owned_target_choice_and_private_draw_preserve_full_nonactor_view() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, IRON, Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: id });
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        let mut other = g.clone();
        let top = *other.state.players[p.index()].library.last().unwrap();
        other.state.objects[top as usize].card = Card::from_name("Shock").unwrap();
        settle(&mut g);
        settle(&mut other);
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(other.observe(p.other())).unwrap()
        );
        assert_eq!(g.stats(bear).power, 4);
        assert_eq!(other.stats(bear).power, 4);
    }
}
#[test]
fn msh_animation_death_iron_power_mana_agrees_in_actual_and_speculative_worlds() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Topiary Lecturer", Zone::Battlefield);
        iron(&mut g, p, source);
        let goal = |amount| FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, amount, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(g.stats(source).power, 5);
        assert!(g.can_fund(goal(5)));
        assert!(!g.can_fund(goal(6)));
        assert_eq!(g.snapshot(), before);
        checked(
            &mut g,
            Action::ActivateMana {
                card: source,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 5, 0],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 5, 0]);
    }
}
#[test]
fn msh_animation_death_ares_paid_body_sickness_and_each_combat_requirement_are_legal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ares(&mut g, p);
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (4, 3));
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        reject(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        ready(&mut g, p);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        reject(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        assert!(g.object_view(id).attacking);
    }
}
#[test]
fn msh_animation_death_ares_attacking_self_and_other_return_without_a_target() {
    for p in [Player::P0, Player::P1] {
        for self_dies in [false, true] {
            let mut g = setup(p);
            let id = ares(&mut g, p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            ready(&mut g, p);
            attack(&mut g, &[id, bear]);
            let target = if self_dies { id } else { bear };
            kill(&mut g, p, target);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Hand);
            assert_eq!(g.object_view(target).owner, p);
        }
    }
}
#[test]
fn msh_animation_death_ares_ignores_nonattacker_and_opponent_attacker() {
    for p in [Player::P0, Player::P1] {
        for opponent in [false, true] {
            let mut g = setup(p);
            ares(&mut g, p);
            let owner = if opponent { p.other() } else { p };
            let bear = named(&mut g, owner, "Bear Cub", Zone::Battlefield);
            if opponent {
                next_turn(&mut g);
                until(&mut g, |g| {
                    g.state.step == Step::Main1 && g.state.pending == Pending::Priority
                });
                attack(&mut g, &[bear]);
            }
            kill(&mut g, p, bear);
            settle(&mut g);
            assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_animation_death_ares_simultaneous_dying_attackers_look_back_independent_of_order() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            let id = ares(&mut g, p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            ready(&mut g, p);
            attack(&mut g, &[id, bear]);
            let ids = if reverse {
                vec![bear, id]
            } else {
                vec![id, bear]
            };
            let sources = g.death_trigger_sources();
            let events = g.move_objects_raw(&ids, Zone::Graveyard);
            g.finish_zone_events(&events, &sources);
            settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Hand);
            assert_eq!(g.object_view(bear).zone, Zone::Hand);
        }
    }
}
#[test]
fn msh_animation_death_ares_uses_owner_hand_and_captured_old_controller_after_queue() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ares(&mut g, p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, "Involuntary Employment", &[Target::Card(bear)]);
        ready(&mut g, p);
        // The temporary control expired; a real second spell establishes this attack's controller.
        paid(&mut g, p, "Involuntary Employment", &[Target::Card(bear)]);
        attack(&mut g, &[id, bear]);
        kill(&mut g, p, bear);
        g.change_controller(g.object_ref(id), p.other());
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
        assert_eq!(g.object_view(bear).owner, p.other());
        assert_eq!(g.object_view(bear).controller, p.other());
    }
}
#[test]
fn msh_animation_death_ares_exact_graveyard_link_never_returns_a_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        for destination in [Zone::Exile, Zone::Battlefield] {
            let mut g = setup(p);
            let id = ares(&mut g, p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            ready(&mut g, p);
            attack(&mut g, &[id, bear]);
            kill(&mut g, p, bear);
            let dead = g.object_ref(bear);
            g.move_object(bear, Zone::Exile);
            if destination == Zone::Battlefield {
                g.move_object(bear, Zone::Battlefield);
                g.move_object(bear, Zone::Graveyard);
            }
            let now = g.object_ref(bear);
            assert_ne!(dead, now);
            settle(&mut g);
            assert_eq!(g.object_ref(bear), now);
            assert_eq!(
                g.object_view(bear).zone,
                if destination == Zone::Exile {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn msh_animation_death_ares_suppression_before_death_prevents_but_after_queue_does_not() {
    for p in [Player::P0, Player::P1] {
        for suppress_before in [false, true] {
            let mut g = setup(p);
            let id = ares(&mut g, p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            ready(&mut g, p);
            if suppress_before {
                paid(&mut g, p, "Stop Cold", &[Target::Card(id)]);
                attack(&mut g, &[bear]);
            } else {
                attack(&mut g, &[id, bear]);
            }
            kill(&mut g, p, bear);
            if !suppress_before {
                g.move_object(id, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(bear).zone,
                if suppress_before {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                }
            );
        }
    }
}
#[test]
fn msh_animation_death_ares_finality_replacement_is_not_death_and_tokens_do_not_return() {
    for p in [Player::P0, Player::P1] {
        for token in [false, true] {
            let mut g = setup(p);
            let id = ares(&mut g, p);
            let target = if token {
                paid(&mut g, p, "Dragon Fodder", &[]);
                (0..g.state.objects.len() as CardId)
                    .find(|&c| g.object_view(c).is_token() && g.object_view(c).controller == p)
                    .unwrap()
            } else {
                let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                g.add_counter(c, CounterKind::Finality, 1);
                c
            };
            ready(&mut g, p);
            attack(&mut g, &[id, target]);
            kill(&mut g, p, target);
            settle(&mut g);
            assert_ne!(g.object_view(target).zone, Zone::Hand);
            if !token {
                assert_eq!(g.object_view(target).zone, Zone::Exile);
            }
        }
    }
}
#[test]
fn msh_animation_death_ares_combat_damage_returns_exact_dead_attacker() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ares(&mut g, p);
        let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        ready(&mut g, p);
        g.state.players[p.other().index()].mana = [30; 6];
        paid(&mut g, p.other(), "Giant Growth", &[Target::Card(blocker)]);
        attack(&mut g, &[id]);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: id,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.object_view(id).zone == Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Hand);
        assert_eq!(g.object_view(blocker).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_animation_death_ares_owned_pending_return_outlives_parent_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ares(&mut g, p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        ready(&mut g, p);
        attack(&mut g, &[id, bear]);
        kill(&mut g, p, bear);
        g.prepare_triggers();
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| !matches!(a, Action::Concede))
            .unwrap();
        g = orphan(g, action);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
        assert!(g.observe(p.other()).objects.iter().any(|o| o.id == bear));
    }
}

#[test]
fn msh_animation_death_iron_animating_attached_equipment_detaches_it_as_a_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let kit = paid(&mut g, p, "S.H.I.E.L.D. Spy Kit", &[]);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: kit,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 3);
        assert_eq!(
            g.state.objects[kit as usize].attached_to,
            Some(g.object_ref(bear))
        );
        iron(&mut g, p, kit);
        assert_eq!(g.state.objects[kit as usize].attached_to, None);
        assert_eq!(g.stats(bear).power, 2);
        assert_eq!(g.stats(kit).power, 4);
        assert!(g.has_subtype(kit, "Equipment"));
    }
}
