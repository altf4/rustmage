//! TMNT's casting and attacking-entry contracts on the split engine.
mod support;

use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn reach(g: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if condition(g) {
            return;
        }
        choose(g, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("condition not reached: {:?}", g.state.pending);
}

fn attack_unblocked(g: &mut Game, attacker: CardId) {
    reach(g, |g| g.state.pending == Pending::Attackers);
    act(g, Action::Attack { card: attacker });
    act(g, Action::FinishAttackers);
    reach(g, |g| g.state.pending == Pending::Blockers);
    act(g, Action::FinishBlockers);
}

fn checked_branch(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let snapshot = g.snapshot();
    let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
    let mut child = {
        let parent = g.clone();
        parent.clone()
    };
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), snapshot);
    child.apply_index(actor, revision, index).unwrap();
    act(g, action.clone());
    assert_eq!(g.snapshot(), child.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(child.observe(viewer)).unwrap(),
        );
    }
    g.restore(&snapshot).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(g, action);
    assert_eq!(g.snapshot(), child.snapshot());
}

fn finish_simple_sneak(g: &mut Game, attacker: ObjectRef) {
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        choose(g, |action| matches!(action, Action::ChooseSpellCost { .. }));
    }
    act(g, Action::ChooseAdditionalCost { object: attacker });
    pay(g);
}

#[test]
fn tmt_foot_ninjas_sneak_hybrid_routes_and_exact_entry() {
    for player in [Player::P0, Player::P1] {
        for color in [0, 2] {
            let mut g = scenario(player);
            let attacker = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
            let ninjas = put(&mut g, player, "Foot Ninjas", Zone::Hand);
            let announce = Action::AnnounceSneak {
                card: ninjas,
                permission: None,
            };
            reject(&mut g, announce.clone());
            attack_unblocked(&mut g, attacker);
            let mut mana = [0; 6];
            mana[color] = 1;
            mana[5] = 3;
            g.state.players[player.index()].mana = mana;
            let attacker = g.object_ref(attacker);
            checked_branch(&mut g, announce);
            assert_eq!(
                g.legal_actions()
                    .iter()
                    .filter(|a| matches!(a, Action::ChooseSpellCost { .. }))
                    .count(),
                1
            );
            finish_simple_sneak(&mut g, attacker);
            assert_eq!(g.state.objects[attacker.card as usize].zone, Zone::Hand);
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].life, 23);
            assert!(g.state.objects[ninjas as usize].tapped);
            assert!(g.state.attackers.iter().any(|row| {
                row.card == ninjas && row.defender == Some(CombatDefender::Player(player.other()))
            }));
            let entry = g.object_ref(ninjas);
            assert!(g.source_sneaked_this_turn(entry));
            g.move_object(ninjas, Zone::Hand);
            g.move_object(ninjas, Zone::Battlefield);
            assert_ne!(g.object_ref(ninjas), entry);
            assert!(!g.source_sneaked_this_turn(g.object_ref(ninjas)));
        }
    }
}

#[test]
fn tmt_ninja_teen_pays_distinct_departure_costs_in_either_order() {
    for player in [Player::P0, Player::P1] {
        for return_first in [false, true] {
            let mut g = scenario(player);
            let class = put(&mut g, player, "Ninja Teen", Zone::Battlefield);
            g.state.objects[class as usize].class_level = 3;
            let attacker = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
            let arbiter = put(&mut g, player, "Arbiter of Woe", Zone::Graveyard);
            let announce = Action::AnnounceSneak {
                card: arbiter,
                permission: None,
            };
            let mut unfundable = g.clone();
            attack_unblocked(&mut unfundable, attacker);
            unfundable.state.players[player.index()].mana = [8; 6];
            reject(&mut unfundable, announce.clone());
            let sacrifice = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
            attack_unblocked(&mut g, attacker);
            g.state.players[player.index()].mana = [8; 6];
            act(&mut g, announce);
            act(&mut g, Action::BeginOrderedCosts);
            let choose_clause = |g: &mut Game, predicate: fn(CastCostClause) -> bool| {
                let costs = g
                    .state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .cast_costs
                    .as_ref()
                    .unwrap();
                let clause = costs
                    .clauses
                    .iter()
                    .position(|row| !row.paid && predicate(row.clause))
                    .unwrap()
                    .try_into()
                    .unwrap();
                act(g, Action::ChooseCastCost { clause });
            };
            let attacker_ref = g.object_ref(attacker);
            let sacrifice_ref = g.object_ref(sacrifice);
            for pay_return in [return_first, !return_first] {
                if pay_return {
                    choose_clause(&mut g, |clause| {
                        matches!(clause, CastCostClause::SneakReturn(_))
                    });
                    reject(
                        &mut g,
                        Action::PaySneakReturn {
                            object: sacrifice_ref,
                        },
                    );
                    checked_branch(
                        &mut g,
                        Action::PaySneakReturn {
                            object: attacker_ref,
                        },
                    );
                } else {
                    choose_clause(&mut g, |clause| clause == CastCostClause::Additional);
                    reject(
                        &mut g,
                        Action::ChooseAdditionalCost {
                            object: attacker_ref,
                        },
                    );
                    checked_branch(
                        &mut g,
                        Action::ChooseAdditionalCost {
                            object: sacrifice_ref,
                        },
                    );
                }
            }
            choose_clause(&mut g, |clause| clause == CastCostClause::Mana);
            pay(&mut g);
            assert!(g.state.announcing.is_none());
            assert_eq!(g.state.objects[attacker as usize].zone, Zone::Hand);
            assert_eq!(g.state.objects[sacrifice as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.objects[arbiter as usize].zone, Zone::Stack);
        }
    }
}

#[test]
fn tmt_samurai_finality_belongs_to_the_chosen_permission() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let samurai = put(&mut g, player, "Leonardo, Sewer Samurai", Zone::Battlefield);
        let class = put(&mut g, player, "Ninja Teen", Zone::Battlefield);
        g.state.objects[class as usize].class_level = 3;
        let attacker = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let small = put(&mut g, player, "Llanowar Elves", Zone::Graveyard);
        let large = put(&mut g, player, "Grizzly Bears", Zone::Graveyard);
        reject(
            &mut g,
            Action::AnnounceGraveyardFinality {
                card: large,
                sneak: false,
            },
        );
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceGraveyardFinality {
                    card: small,
                    sneak: false,
                })
        );
        attack_unblocked(&mut g, attacker);
        g.state.players[player.index()].mana = [8; 6];
        let attacker = g.object_ref(attacker);
        for finality in [false, true] {
            let mut branch = g.clone();
            let action = if finality {
                Action::AnnounceGraveyardFinality {
                    card: small,
                    sneak: true,
                }
            } else {
                Action::AnnounceSneak {
                    card: small,
                    permission: None,
                }
            };
            act(&mut branch, action);
            finish_simple_sneak(&mut branch, attacker);
            branch.move_object(samurai, Zone::Exile);
            settle(&mut branch);
            assert_eq!(
                branch.counter_amounts(small).get(CounterKind::Finality),
                u64::from(finality)
            );
            let result = branch.move_object(small, Zone::Graveyard);
            assert_eq!(
                result.destination,
                if finality {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}

#[test]
fn tmt_last_ronins_technique_enters_attacking_without_delayed_sacrifice() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let attacker = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, player, "The Last Ronin's Technique", Zone::Hand);
        attack_unblocked(&mut g, attacker);
        g.state.players[player.index()].mana = [8; 6];
        let attacker = g.object_ref(attacker);
        act(
            &mut g,
            Action::AnnounceSneak {
                card: spell,
                permission: None,
            },
        );
        finish_simple_sneak(&mut g, attacker);
        settle(&mut g);
        assert_eq!(g.state.attackers.len(), 3);
        for row in &g.state.attackers {
            assert_eq!(row.defender, Some(CombatDefender::Player(player.other())));
            let token = &g.state.objects[row.card as usize];
            assert!(token.is_token() && token.tapped);
            assert!(token.card.has_subtype("Ninja") && token.card.has_subtype("Turtle"));
        }
        assert!(g.state.delayed_sacrifices.is_none());
    }
}

fn resolve_one_target(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}

#[test]
fn tmt_neutrinos_blink_can_choose_a_different_exact_defender() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let neutrinos = put(&mut g, player, "The Neutrinos", Zone::Battlefield);
        let subject = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let walker = put(
            &mut g,
            player.other(),
            "Elspeth, Storm Slayer",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        reach(&mut g, |g| g.state.pending == Pending::Attackers);
        act(&mut g, Action::Attack { card: neutrinos });
        act(&mut g, Action::FinishAttackers);
        resolve_one_target(&mut g, subject);
        let before = g.object_ref(subject);
        reach(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryAttackingDefender { .. })
        });
        let defender = CombatDefender::Planeswalker(g.object_ref(walker));
        checked_branch(&mut g, Action::ChooseEntryAttackingDefender { defender });
        assert_ne!(g.object_ref(subject), before);
        assert!(g.state.objects[subject as usize].tapped);
        assert!(
            g.state
                .attackers
                .iter()
                .any(|attack| { attack.card == subject && attack.defender == Some(defender) })
        );
        assert!(g.state.attackers.iter().any(|attack| {
            attack.card == neutrinos
                && attack.defender == Some(CombatDefender::Player(player.other()))
        }));
    }
}

#[test]
fn tmt_shark_shredder_returns_the_damaged_players_creature_attacking_that_player() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let shark = put(
            &mut g,
            player,
            "Shark Shredder, Killer Clone",
            Zone::Battlefield,
        );
        let yours = put(&mut g, player, "Grizzly Bears", Zone::Graveyard);
        let theirs = put(&mut g, player.other(), "Grizzly Bears", Zone::Graveyard);
        attack_unblocked(&mut g, shark);
        reach(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(yours),
            },
        );
        resolve_one_target(&mut g, theirs);
        settle(&mut g);
        let reanimated = &g.state.objects[theirs as usize];
        assert_eq!(reanimated.zone, Zone::Battlefield);
        assert_eq!(reanimated.owner, player.other());
        assert_eq!(reanimated.controller, player);
        assert!(reanimated.tapped);
        assert!(g.state.attackers.iter().any(|row| {
            row.card == theirs && row.defender == Some(CombatDefender::Player(player.other()))
        }));
    }
}

#[test]
fn tmt_sibling_rivals_only_schedule_an_additional_combat_from_the_first() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let rivals = put(
            &mut g,
            player,
            "Raph & Leo, Sibling Rivals",
            Zone::Battlefield,
        );
        reach(&mut g, |g| g.state.pending == Pending::Attackers);
        act(&mut g, Action::Attack { card: rivals });
        act(&mut g, Action::FinishAttackers);
        resolve_one_target(&mut g, rivals);
        settle(&mut g);
        assert!(!g.state.objects[rivals as usize].tapped);
        assert_eq!(g.observe(player).additional_combats, 1);
        reach(&mut g, |g| g.state.pending == Pending::Attackers);
        act(&mut g, Action::Attack { card: rivals });
        act(&mut g, Action::FinishAttackers);
        assert_eq!(g.observe(player).additional_combats, 0);
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.state.stack.is_empty());
    }
}
