//! Seven complete event-driven Foundations cards, through owned legal decisions.
use super::ability_timeline_tests::{cast, checked, named, next_turn, setup};
use super::*;

fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool, preferred: &[Action]) {
    g.prepare_triggers();
    for _ in 0..200 {
        if stop(g) {
            return;
        }
        let legal = g.legal_actions();
        let a = preferred
            .iter()
            .find(|a| legal.contains(a))
            .cloned()
            .unwrap_or_else(|| {
                if g.state.pending == Pending::Priority {
                    Action::Pass
                } else {
                    legal
                        .iter()
                        .find(|a| {
                            matches!(
                                a,
                                Action::FinishTargets
                                    | Action::PayCosts { .. }
                                    | Action::PutTrigger { .. }
                                    | Action::PayResolutionMana { .. }
                                    | Action::FinishSearch
                                    | Action::DeclineEffectPayment
                            )
                        })
                        .or_else(|| legal.iter().find(|a| !matches!(a, Action::Concede)))
                        .unwrap()
                        .clone()
                }
            });
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn settle(g: &mut Game, p: &[Action]) {
    until(g, idle, p)
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(before, g.snapshot());
    assert_eq!(rev, g.revision())
}
fn attack(g: &mut Game, id: CardId) {
    g.state.objects[id as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
}
fn tokens(g: &Game, p: Player, c: Card) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.card == c)
        .map(|(id, _)| id as CardId)
        .collect()
}

#[test]
fn fdn_events_all_seven_paid_printed_costs_and_legal_aura_target() {
    for p in [Player::P0, Player::P1] {
        for (name, cost) in [
            ("Felidar Retreat", 4),
            ("Kalastria Highborn", 2),
            ("Vampire Gourmand", 2),
            ("Immersturm Predator", 4),
            ("Faebloom Trick", 3),
            ("Alesha, Who Laughs at Fate", 3),
            ("Ordeal of Nylea", 2),
        ] {
            let mut g = setup(p);
            let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            let before = g.state.players[p.index()].mana.iter().sum::<u32>();
            let targets = if name == "Ordeal of Nylea" {
                vec![Target::Card(target)]
            } else {
                vec![]
            };
            cast(&mut g, id, None, &targets);
            settle(&mut g, &[]);
            assert_eq!(
                before - g.state.players[p.index()].mana.iter().sum::<u32>(),
                cost,
                "{name}"
            );
            if name == "Ordeal of Nylea" {
                assert_eq!(g.object_view(id).attached_to, Some(g.object_ref(target)));
            }
        }
    }
}
#[test]
fn fdn_events_felidar_own_land_modes_counters_frozen_vigilance_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let retreat = named(&mut g, p, "Felidar Retreat", Zone::Battlefield);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Forest", Zone::Hand);
        g.move_object(foreign, Zone::Battlefield);
        assert!(g.state.pending_triggers.is_empty());
        let land = named(&mut g, p, "Forest", Zone::Hand);
        g.move_object(land, Zone::Battlefield);
        settle(&mut g, &[Action::ChooseMode { mode: 0 }]);
        let cats = tokens(&g, p, Card::catalogue().nth(5337).unwrap());
        assert_eq!(cats.len(), 1);
        assert_eq!(
            g.stats(cats[0]),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert_eq!(g.object_view(cats[0]).card.colors(), 1);
        let land = named(&mut g, p, "Plains", Zone::Hand);
        g.move_object(land, Zone::Battlefield);
        until(
            &mut g,
            |g| g.legal_actions().contains(&Action::ChooseMode { mode: 1 }),
            &[],
        );
        checked(&mut g, Action::ChooseMode { mode: 1 });
        g.move_object(retreat, Zone::Graveyard);
        settle(&mut g, &[]);
        for id in [own, cats[0]] {
            assert_eq!(g.object_view(id).counters.plus_one, 1);
            assert!(g.keywords(id).contains(Keywords::VIGILANCE));
        }
        assert_eq!(g.object_view(other).counters.plus_one, 0);
        let late = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(!g.keywords(late).contains(Keywords::VIGILANCE));
        next_turn(&mut g);
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
        assert_eq!(g.object_view(own).counters.plus_one, 1);
    }
}
#[test]
fn fdn_events_highborn_simultaneous_vampire_deaths_optional_black_payment_and_self_target() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let high = named(&mut g, p, "Kalastria Highborn", Zone::Battlefield);
            let vamp = named(&mut g, p, "Vampire Gourmand", Zone::Battlefield);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foreign = named(&mut g, p.other(), "Vampire Gourmand", Zone::Battlefield);
            g.sacrifice_batch(&[(p, high), (p, vamp), (p, bear), (p.other(), foreign)]);
            assert_eq!(
                g.state
                    .pending_triggers
                    .iter()
                    .filter(|t| t.card == high)
                    .count(),
                2
            );
            let black = g.state.players[p.index()].mana[2];
            let pref = if pay {
                vec![
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                    Action::PayResolutionMana {
                        payment: [0, 0, 1, 0, 0, 0],
                    },
                ]
            } else {
                vec![
                    Action::ChooseTarget {
                        target: Target::Player(p),
                    },
                    Action::DeclineResolutionMana,
                ]
            };
            settle(&mut g, &pref);
            assert_eq!(g.state.players[p.index()].life, if pay { 24 } else { 20 });
            assert_eq!(
                g.state.players[p.other().index()].life,
                if pay { 16 } else { 20 }
            );
            assert_eq!(
                g.state.players[p.index()].mana[2],
                black - if pay { 2 } else { 0 }
            );
        }
    }
}
#[test]
fn fdn_events_gourmand_optional_other_sacrifice_draw_unblockable_and_expiry() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let gourmand = named(&mut g, p, "Vampire Gourmand", Zone::Battlefield);
            let food = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let card = named(&mut g, p, "Forest", Zone::Library);
            attack(&mut g, gourmand);
            until(
                &mut g,
                |g| g.legal_actions().contains(&Action::DeclineEffectPayment),
                &[],
            );
            let source = g.object_ref(gourmand);
            reject(&mut g, Action::Sacrifice { object: source });
            let action = if pay {
                Action::Sacrifice {
                    object: g.object_ref(food),
                }
            } else {
                Action::DeclineEffectPayment
            };
            checked(&mut g, action);
            settle(&mut g, &[]);
            assert_eq!(
                g.object_view(food).zone,
                if pay {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.object_view(card).zone,
                if pay { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(
                g.keywords(gourmand).contains(Keywords::CANT_BE_BLOCKED),
                pay
            );
            next_turn(&mut g);
            assert!(!g.keywords(gourmand).contains(Keywords::CANT_BE_BLOCKED));
        }
    }
}
#[test]
fn fdn_events_predator_paid_another_sacrifice_taps_once_optional_grave_target() {
    for p in [Player::P0, Player::P1] {
        for choose in [false, true] {
            let mut g = setup(p);
            let predator = named(&mut g, p, "Immersturm Predator", Zone::Battlefield);
            let fodder = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let grave = named(&mut g, p.other(), "Forest", Zone::Graveyard);
            let before = g.state.players[p.index()].mana;
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: predator,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let source = g.object_ref(predator);
            reject(&mut g, Action::ChooseAdditionalCost { object: source });
            let object = g.object_ref(fodder);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            until(&mut g, |g| g.state.announcing.is_none(), &[]);
            assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
            assert!(!g.object_view(predator).tapped);
            let prefs = if choose {
                vec![Action::ChooseTarget {
                    target: Target::Card(grave),
                }]
            } else {
                vec![]
            };
            settle(&mut g, &prefs);
            assert!(g.object_view(predator).tapped);
            assert!(g.keywords(predator).contains(Keywords::INDESTRUCTIBLE));
            assert_eq!(g.object_view(predator).counters.plus_one, 1);
            assert_eq!(
                g.object_view(grave).zone,
                if choose { Zone::Exile } else { Zone::Graveyard }
            );
            assert_eq!(g.state.players[p.index()].mana, before);
            g.tap_objects(&[predator]);
            assert!(g.state.pending_triggers.is_empty());
            next_turn(&mut g);
            assert!(!g.keywords(predator).contains(Keywords::INDESTRUCTIBLE));
        }
    }
}
#[test]
fn fdn_events_predator_departed_grave_target_fizzles_counter_and_source_reentry_is_new() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let predator = named(&mut g, p, "Immersturm Predator", Zone::Battlefield);
        let grave = named(&mut g, p, "Forest", Zone::Graveyard);
        g.tap_objects(&[predator]);
        until(
            &mut g,
            |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(grave),
                })
            },
            &[],
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(grave),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none(), &[]);
        g.move_object(grave, Zone::Hand);
        settle(&mut g, &[]);
        assert_eq!(g.object_view(predator).counters.plus_one, 0);
        g.state.objects[predator as usize].tapped = false;
        g.tap_objects(&[predator]);
        g.move_object(predator, Zone::Exile);
        g.move_object(predator, Zone::Battlefield);
        settle(&mut g, &[]);
        assert_eq!(g.object_view(predator).counters.plus_one, 0);
    }
}
#[test]
fn fdn_events_faebloom_no_initial_targets_reflexive_response_and_whole_ability_fizzle() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Faebloom Trick", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        assert!(g.state.stack.last().unwrap().targets.is_empty());
        assert!(tokens(&g, p, Card::catalogue().nth(5286).unwrap()).is_empty());
        until(
            &mut g,
            |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(victim),
                })
            },
            &[],
        );
        assert_eq!(tokens(&g, p, Card::catalogue().nth(5286).unwrap()).len(), 2);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none(), &[]);
        assert!(!g.object_view(victim).tapped);
        g.move_object(victim, Zone::Hand);
        g.move_object(victim, Zone::Battlefield);
        settle(&mut g, &[]);
        assert!(!g.object_view(victim).tapped);
        assert_eq!(tokens(&g, p, Card::catalogue().nth(5286).unwrap()).len(), 2);
        let spell = named(&mut g, p, "Faebloom Trick", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        settle(
            &mut g,
            &[Action::ChooseTarget {
                target: Target::Card(victim),
            }],
        );
        assert!(g.object_view(victim).tapped);
        assert_eq!(tokens(&g, p, Card::catalogue().nth(5286).unwrap()).len(), 4);
    }
}
#[test]
fn fdn_events_alesha_live_power_then_last_known_departure_bound_and_raid() {
    for p in [Player::P0, Player::P1] {
        for leaves in [false, true] {
            let mut g = setup(p);
            let alesha = named(&mut g, p, "Alesha, Who Laughs at Fate", Zone::Battlefield);
            let three = named(&mut g, p, "Marauding Blight-Priest", Zone::Graveyard);
            let two = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            attack(&mut g, alesha);
            settle(&mut g, &[]);
            assert_eq!(g.object_view(alesha).counters.plus_one, 1);
            assert!(g.keywords(alesha).contains(Keywords::FIRST_STRIKE));
            g.begin_step(Step::End);
            until(
                &mut g,
                |g| {
                    g.legal_actions().contains(&Action::ChooseTarget {
                        target: Target::Card(three),
                    })
                },
                &[],
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(three),
                },
            );
            until(&mut g, |g| g.state.announcing.is_none(), &[]);
            if leaves {
                g.move_object(alesha, Zone::Exile);
                g.move_object(alesha, Zone::Battlefield);
            } else {
                g.state.objects[alesha as usize].growth.power = -2;
            }
            settle(&mut g, &[]);
            assert_eq!(
                g.object_view(three).zone,
                if leaves {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.object_view(two).zone, Zone::Graveyard);
        }
    }
    let mut g = setup(Player::P0);
    let _a = named(
        &mut g,
        Player::P0,
        "Alesha, Who Laughs at Fate",
        Zone::Battlefield,
    );
    g.begin_step(Step::End);
    assert!(g.state.pending_triggers.is_empty());
}
#[test]
fn fdn_events_ordeal_third_counter_sacrifice_search_tapped_up_to_two_and_no_death_shortcut() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let aura = named(&mut g, p, "Ordeal of Nylea", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g, &[]);
        g.state.objects[bear as usize].counters.plus_one = 2;
        let l1 = named(&mut g, p, "Forest", Zone::Library);
        let l2 = named(&mut g, p, "Plains", Zone::Library);
        let notland = named(&mut g, p, "Bear Cub", Zone::Library);
        // The Aura's controller gets the trigger even when another player attacks.
        g.state.active = p.other();
        g.state.priority = p.other();
        attack(&mut g, bear);
        until(
            &mut g,
            |g| matches!(g.state.pending, Pending::EffectSearch { .. }),
            &[],
        );
        assert_eq!(g.object_view(bear).counters.plus_one, 3);
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        reject(&mut g, Action::ChooseSearch { card: notland });
        checked(&mut g, Action::ChooseSearch { card: l1 });
        checked(&mut g, Action::ChooseSearch { card: l2 });
        settle(&mut g, &[]);
        for land in [l1, l2] {
            assert_eq!(g.object_view(land).zone, Zone::Battlefield);
            assert!(g.object_view(land).tapped);
            assert_eq!(g.object_view(land).controller, p);
        }
        let aura = named(&mut g, p, "Ordeal of Nylea", Zone::Battlefield);
        g.move_object(aura, Zone::Graveyard);
        assert!(g.state.pending_triggers.iter().all(|t| t.card != aura));
    }
}
#[test]
fn fdn_events_ordeal_queued_recipient_survives_aura_departure_and_never_reincarnation() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let aura = named(&mut g, p, "Ordeal of Nylea", Zone::Hand);
            cast(&mut g, aura, None, &[Target::Card(bear)]);
            settle(&mut g, &[]);
            attack(&mut g, bear);
            until(
                &mut g,
                |g| g.state.stack.iter().any(|t| t.card == aura),
                &[],
            );
            g.move_object(aura, Zone::Graveyard);
            if blink {
                g.move_object(bear, Zone::Exile);
                g.move_object(bear, Zone::Battlefield);
            }
            settle(&mut g, &[]);
            assert_eq!(
                g.object_view(bear).counters.plus_one,
                if blink { 0 } else { 1 }
            );
            assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn fdn_events_faebloom_reflexive_capacity_failure_is_atomic_after_token_creation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Faebloom Trick", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        checked(&mut g, Action::Pass);
        g.state.next_stack_id = u64::MAX;
        let before = g.snapshot();
        let revision = g.revision();
        let actor = g.actor().unwrap();
        for _ in 0..2 {
            assert!(matches!(
                g.apply(actor, Action::Pass),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(before, g.snapshot());
            assert_eq!(revision, g.revision());
            assert!(tokens(&g, p, Card::catalogue().nth(5286).unwrap()).is_empty());
            assert!(!g.object_view(victim).tapped);
        }
    }
}
#[test]
fn fdn_events_ordeal_external_sacrifice_search_can_find_zero_and_never_leaks_library() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let aura = named(&mut g, p, "Ordeal of Nylea", Zone::Battlefield);
        let hidden = named(&mut g, p, "Forest", Zone::Library);
        g.sacrifice_batch(&[(p, aura)]);
        until(
            &mut g,
            |g| matches!(g.state.pending, Pending::EffectSearch { .. }),
            &[],
        );
        assert!(g.observe(p).search_choice.is_some());
        assert!(g.observe(p.other()).search_choice.is_none());
        checked(&mut g, Action::FinishSearch);
        settle(&mut g, &[]);
        assert_eq!(g.object_view(hidden).zone, Zone::Library);
    }
}
