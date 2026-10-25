//! Complete paid Baxter/Raft scenarios. Any constructed layer is labeled bounded.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::msh_galactus_tests::{reject, until};
use super::*;
const BAXTER: &str = "Baxter Building";
const RAFT: &str = "Raft Security Officer";
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    g.state.players[p.index()].mana = [30; 6];
    super::msh_galactus_tests::paid(g, p, name, targets)
}
fn land(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, BAXTER, Zone::Hand);
    checked(g, Action::PlayLand { card: id });
    id
}
fn announce(g: &mut Game, id: CardId, ability: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseActivationCost { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .expect("unfinished cost");
        checked(g, action);
    }
    panic!("unfinished activation");
}
fn raft(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, RAFT, &[]);
    next_turn(g);
    next_turn(g);
    until(g, |g| {
        g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
    id
}
fn treasure() -> Card {
    Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
        .unwrap()
}
fn animation(g: &mut Game, id: CardId, artifact: bool, base: (i128, i128)) {
    // Explicit bounded earlier-layer fixture, never a new playable card admission.
    let object = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_default()
        .animations
        .push(characteristic_layers::AnimationView {
            object,
            timestamp,
            artifact,
            preserve_card_types: false,
            colors: None,
            creature_types: None,
            all_creature_types: false,
            base: Some(base),
            keywords: Keywords::NONE,
            expires_turn: None,
        });
    g.state.objects[id as usize].summoning_sick = false;
}
fn forge(g: &mut Game, p: Player, target: CardId) -> CardId {
    g.state.players[p.index()].mana = [30; 6];
    let id = named(g, p, "Territory Forge", Zone::Hand);
    cast(g, id, None, &[]);
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
        {
            return id;
        }
        let choices = g.legal_actions();
        let choose = Action::ChooseTarget {
            target: Target::Card(target),
        };
        let a = if choices.contains(&choose) {
            choose
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            choices
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::FinishTargets | Action::PayCosts { .. } | Action::PutTrigger { .. }
                    )
                })
                .unwrap()
        };
        checked(g, a);
    }
    panic!("forge stalled");
}
#[test]
fn msh_baxter_raft_combination_all_seventy_exact_unique_and_early_exit() {
    let production = ManaProduction::Combination {
        mask: 31,
        amount: 4,
    };
    let mut values = std::collections::BTreeSet::new();
    assert!(!production.visit(|m| {
        assert_eq!(m.into_iter().sum::<u32>(), 4);
        assert_eq!(m[5], 0);
        assert!(values.insert(m));
        false
    }));
    assert_eq!(values.len(), 70);
    assert!(values.contains(&[1, 1, 1, 1, 0, 0]));
    assert!(values.contains(&[0, 2, 0, 2, 0, 0]));
    let mut calls = 0;
    assert!(production.visit(|_| {
        calls += 1;
        true
    }));
    assert_eq!(calls, 1);
    let mut old = 0;
    ManaProduction::any_color(4).visit(|_| {
        old += 1;
        false
    });
    assert_eq!(old, 5);
    let mut zero = 0;
    ManaProduction::Combination {
        mask: 31,
        amount: 0,
    }
    .visit(|m| {
        assert_eq!(m, [0; 6]);
        zero += 1;
        false
    });
    assert_eq!(zero, 1);
}
#[test]
fn msh_baxter_raft_baxter_normal_land_play_colorless_immediate_and_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        assert!(!g.object_view(b).tapped);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::ActivateMana {
                card: b,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        assert!(g.state.stack.is_empty());
        assert!(g.object_view(b).tapped);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: b,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn msh_baxter_raft_all_split_productions_are_real_paid_actions() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        let snapshot = g.snapshot();
        let mut count = 0;
        ManaProduction::Combination {
            mask: 31,
            amount: 4,
        }
        .visit(|production| {
            let mut child = g.clone();
            checked(
                &mut child,
                Action::ActivateMana {
                    card: b,
                    ability: 1,
                    payment: [0, 0, 0, 0, 0, 4],
                    counter_payment: Counters::default(),
                    production,
                },
            );
            assert_eq!(child.state.players[p.index()].mana, production);
            assert!(child.state.stack.is_empty());
            count += 1;
            false
        });
        assert_eq!(count, 70);
        assert_eq!(g.snapshot(), snapshot);
        for production in [[0, 0, 0, 0, 0, 4], [1, 1, 0, 0, 0, 0], [4, 1, 0, 0, 0, 0]] {
            reject(
                &mut g,
                Action::ActivateMana {
                    card: b,
                    ability: 1,
                    payment: [0, 0, 0, 0, 0, 4],
                    counter_payment: Counters::default(),
                    production,
                },
            );
        }
    }
}
#[test]
fn msh_baxter_raft_actual_land_seed_mixed_colors_funds_paid_iron() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        for _ in 0..4 {
            let l = named(&mut g, p, "Plains", Zone::Battlefield);
            checked(&mut g, Action::TapMana { card: l });
        }
        checked(
            &mut g,
            Action::ActivateMana {
                card: b,
                ability: 1,
                payment: [4, 0, 0, 0, 0, 0],
                counter_payment: Counters::default(),
                production: [1, 1, 1, 1, 0, 0],
            },
        );
        let iron = named(&mut g, p, "Iron Man, Master of Machines", Zone::Hand);
        cast(&mut g, iron, None, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(iron));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_baxter_raft_converter_cannot_self_fund_or_double_tap() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        let a = Action::AnnounceMana {
            card: b,
            ability: 1,
            counter_payment: Counters::default(),
            production: [1, 1, 1, 1, 0, 0],
        };
        reject(&mut g, a.clone());
        checked(
            &mut g,
            Action::ActivateMana {
                card: b,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        reject(&mut g, a);
    }
}
#[test]
fn msh_baxter_raft_converter_in_mana_window_funds_spell_and_preserves_parent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        let iron = named(&mut g, p, "Iron Man, Master of Machines", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        checked(&mut g, Action::AnnounceCast { card: iron });
        checked(
            &mut g,
            Action::ActivateMana {
                card: b,
                ability: 1,
                payment: [0, 0, 0, 0, 0, 4],
                counter_payment: Counters::default(),
                production: [0, 2, 0, 2, 0, 0],
            },
        );
        assert_eq!(g.state.announcing.as_ref().unwrap().item.card, iron);
        finish(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(iron));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_baxter_raft_baxter_toughness_live_guard_draw_and_no_resolution_recheck() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        let a = Action::AnnounceActivate {
            card: b,
            ability: 2,
            counter_payment: Counters::default(),
        };
        reject(&mut g, a.clone());
        let witness = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(witness, CounterKind::PlusOne, 1);
        reject(&mut g, a.clone());
        g.add_counter(witness, CounterKind::PlusOne, 1);
        g.state.objects[witness as usize].tapped = true;
        announce(&mut g, b, 2);
        finish(&mut g);
        let before = g.hand_count(p);
        g.change_controller(g.object_ref(witness), p.other());
        g.move_object(witness, Zone::Graveyard);
        g.change_controller(g.object_ref(b), p.other());
        g.move_object(b, Zone::Hand);
        let after_return = g.hand_count(p);
        assert_eq!(after_return, before + 1); // Returning the physical land adds one card.
        settle(&mut g);
        assert_eq!(g.hand_count(p), after_return + 1);
    }
}
#[test]
fn msh_baxter_raft_baxter_lord_current_control_type_and_damage_guard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        let hero = paid(&mut g, p, "Human Torch, Johnny Storm", &[]);
        let av = paid(&mut g, p, "Avengers Assemble!", &[]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        assert!(g.activation_guard_available(
            ActivationGuard::ControlCreatureToughnessAtLeast(4),
            b,
            p
        ));
        g.state.objects[hero as usize].damage = 1;
        assert!(g.activation_guard_available(
            ActivationGuard::ControlCreatureToughnessAtLeast(4),
            b,
            p
        ));
        g.move_object(av, Zone::Graveyard);
        assert!(!g.activation_guard_available(
            ActivationGuard::ControlCreatureToughnessAtLeast(4),
            b,
            p
        ));
        g.change_controller(g.object_ref(hero), p.other());
        assert!(!g.activation_guard_available(
            ActivationGuard::ControlCreatureToughnessAtLeast(4),
            b,
            p
        ));
    }
}
#[test]
fn msh_baxter_raft_baxter_guard_not_rechecked_when_mana_witness_sacrificed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        g.create_tokens(p, treasure(), 1, false);
        let t = (g.state.objects.len() - 1) as CardId;
        animation(&mut g, t, false, (4, 4));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        announce(&mut g, b, 2);
        checked(
            &mut g,
            Action::ActivateMana {
                card: t,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        finish(&mut g);
        assert!(!g.on_battlefield(t));
        let before = g.hand_count(p);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
#[test]
fn msh_baxter_raft_animated_land_sickness_and_suppression() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        animation(&mut g, b, false, (4, 4));
        g.state.objects[b as usize].summoning_sick = true;
        let a = Action::ActivateMana {
            card: b,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 1],
        };
        reject(&mut g, a.clone());
        paid(&mut g, p, "Impolite Entrance", &[Target::Card(b)]);
        checked(&mut g, a);
        g.state.objects[b as usize].tapped = false;
        paid(&mut g, p, "Frozen in Ice", &[Target::Card(b)]);
        g.state.priority = p;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: b,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn msh_baxter_raft_raft_paid_body_sickness_haste_and_self_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        let r = named(&mut g, p, RAFT, Zone::Hand);
        cast(&mut g, r, None, &[]);
        settle(&mut g);
        assert_eq!((g.stats(r).power, g.stats(r).toughness), (1, 3));
        let a = Action::AnnounceActivate {
            card: r,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, a);
        paid(&mut g, p, "Impolite Entrance", &[Target::Card(r)]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        announce(&mut g, r, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(r),
            },
        );
        assert_eq!(
            g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
            1
        );
        finish(&mut g);
        settle(&mut g);
        assert!(g.object_view(r).tapped);
    }
}
#[test]
fn msh_baxter_raft_raft_power_threshold_negative_zero_three_four_paid() {
    for p in [Player::P0, Player::P1] {
        for power in [-1, 0, 3, 4] {
            let mut g = setup(p);
            let r = raft(&mut g, p);
            let t = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            animation(&mut g, t, false, (power, 5));
            let cost = if power <= 3 { 1 } else { 2 };
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, cost];
            announce(&mut g, r, 0);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(t),
                },
            );
            assert_eq!(
                g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
                cost
            );
            finish(&mut g);
            settle(&mut g);
            assert!(g.object_view(t).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn msh_baxter_raft_selected_target_not_any_cheap_target_controls_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = raft(&mut g, p);
        let big = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.add_counter(big, CounterKind::PlusOne, 1);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        announce(&mut g, r, 0);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(big),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(r),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert!(!g.object_view(big).tapped);
    }
}
#[test]
fn msh_baxter_raft_no_legal_cheap_target_no_false_affordability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = raft(&mut g, p);
        g.add_counter(r, CounterKind::PlusOne, 3);
        let t = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[t as usize].gained_keywords = Keywords::HEXPROOF;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: r,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut g, r, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(r),
            },
        );
        finish(&mut g);
        settle(&mut g);
    }
}
#[test]
fn msh_baxter_raft_direct_actions_have_target_specific_exact_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = raft(&mut g, p);
        let big = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.add_counter(big, CounterKind::PlusOne, 1);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        for (target, cost) in [(r, 1), (big, 2)] {
            let mut child = g.clone();
            let a = Action::Activate {
                card: r,
                ability: 0,
                target: Some(Target::Card(target)),
                payment: [0, 0, 0, 0, 0, cost],
                counter_payment: Counters::default(),
            };
            checked(&mut child, a);
            settle(&mut child);
            assert!(child.object_view(target).tapped);
        }
        reject(
            &mut g,
            Action::Activate {
                card: r,
                ability: 0,
                target: Some(Target::Card(big)),
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::Activate {
                card: r,
                ability: 0,
                target: Some(Target::Card(r)),
                payment: [0, 0, 0, 0, 0, 2],
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn msh_baxter_raft_target_cost_locks_before_actual_mana_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = raft(&mut g, p);
        let iron = paid(&mut g, p, "Iron Man, Master of Machines", &[]);
        g.create_tokens(p, treasure(), 3, false);
        assert_eq!(g.stats(iron).power, 4);
        let t = (g.state.objects.len() - 1) as CardId;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        announce(&mut g, r, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(iron),
            },
        );
        assert_eq!(
            g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
            2
        );
        checked(
            &mut g,
            Action::ActivateMana {
                card: t,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        finish(&mut g);
        assert_eq!(g.stats(iron).power, 3);
        settle(&mut g);
        assert!(g.object_view(iron).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_baxter_raft_later_target_power_change_never_reprices_or_restricts_effect() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = raft(&mut g, p);
        let t = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        announce(&mut g, r, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(t),
            },
        );
        // Bounded post-lock counter mutation: no claim of an intervening priority window.
        g.add_counter(t, CounterKind::PlusOne, 3);
        assert_eq!(
            g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
            1
        );
        finish(&mut g);
        settle(&mut g);
        assert!(g.object_view(t).tapped);
    }
}
#[test]
fn msh_baxter_raft_source_leave_control_change_and_target_blink() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let r = raft(&mut g, p);
            let t = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            announce(&mut g, r, 0);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(t),
                },
            );
            finish(&mut g);
            g.change_controller(g.object_ref(r), p.other());
            g.move_object(r, Zone::Hand);
            if blink {
                g.move_object(t, Zone::Hand);
                g.move_object(t, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(g.object_view(t).tapped, !blink);
        }
    }
}
#[test]
fn msh_baxter_raft_already_tapped_target_and_noncreature_rejection() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = raft(&mut g, p);
        let t = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[t as usize].tapped = true;
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        announce(&mut g, r, 0);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(t),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert!(g.object_view(t).tapped);
    }
}
#[test]
fn msh_baxter_raft_actual_forge_inherits_baxter_modes_and_guard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        let f = forge(&mut g, p, b);
        assert_eq!(g.object_view(b).zone, Zone::Exile);
        let exiled = g.object_ref(b);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        checked(
            &mut g,
            Action::AnnounceBorrowedMana {
                card: f,
                exiled,
                ability: 1,
                counter_payment: Counters::default(),
                production: [1, 1, 1, 1, 0, 0],
            },
        );
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [1, 1, 1, 1, 0, 0]);
        assert!(g.object_view(f).tapped);
        g.state.objects[f as usize].tapped = false;
        let witness = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.add_counter(witness, CounterKind::PlusOne, 1);
        let before = g.hand_count(p);
        checked(
            &mut g,
            Action::AnnounceBorrowedAbility {
                card: f,
                exiled,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        finish(&mut g);
        g.move_object(witness, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
#[test]
fn msh_baxter_raft_bounded_artifact_raft_actual_forge_discount_and_host_self_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = paid(&mut g, p, RAFT, &[]);
        animation(&mut g, r, true, (1, 3));
        let f = forge(&mut g, p, r);
        animation(&mut g, f, false, (3, 4));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let exiled = g.object_ref(r);
        checked(
            &mut g,
            Action::AnnounceBorrowedAbility {
                card: f,
                exiled,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(f),
            },
        );
        assert_eq!(
            g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
            1
        );
        finish(&mut g);
        settle(&mut g);
        assert!(g.object_view(f).tapped);
    }
}
#[test]
fn msh_baxter_raft_full_nonactor_observation_draw_privacy_and_atomic_short_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let witness = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(witness, CounterKind::PlusOne, 2);
        let mut other = g.clone();
        let top = *other.state.players[p.index()].library.last().unwrap();
        other.state.objects[top as usize].card = Card::from_name("Shock").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(other.observe(p.other())).unwrap()
        );
        for game in [&mut g, &mut other] {
            game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            announce(game, b, 2);
            reject(
                game,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 0, 3],
                },
            );
            finish(game);
            settle(game);
        }
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(other.observe(p.other())).unwrap()
        );
        assert_ne!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(other.observe(p)).unwrap()
        );
    }
}
#[test]
fn msh_baxter_raft_orphan_raw_index_restore_owns_locked_target_cost() {
    for p in [Player::P0, Player::P1] {
        let (mut raw, mut indexed, snapshot, revision, index, action) = {
            let mut parent = setup(p);
            let r = raft(&mut parent, p);
            parent.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            announce(&mut parent, r, 0);
            checked(
                &mut parent,
                Action::ChooseTarget {
                    target: Target::Card(r),
                },
            );
            let action = Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 1],
            };
            let index = parent
                .legal_actions()
                .iter()
                .position(|a| *a == action)
                .unwrap();
            (
                parent.clone(),
                parent.clone(),
                parent.snapshot(),
                parent.revision(),
                index,
                action,
            )
        };
        raw.apply(p, action).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        settle(&mut raw);
        settle(&mut indexed);
        for viewer in [p, p.other()] {
            assert_eq!(
                serde_json::to_value(raw.observe(viewer)).unwrap(),
                serde_json::to_value(indexed.observe(viewer)).unwrap()
            );
        }
        indexed.restore(&snapshot).unwrap();
        assert_eq!(
            indexed
                .funding_goal(indexed.state.announcing.as_ref().unwrap())
                .generic,
            1
        );
        checked(
            &mut indexed,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        settle(&mut indexed);
    }
}
#[test]
fn msh_baxter_raft_artifact_spell_only_lots_cannot_pay_either_activation() {
    for p in [Player::P0, Player::P1] {
        for filter in [true, false] {
            let mut g = setup(p);
            let b = land(&mut g, p);
            let r = if filter { b } else { raft(&mut g, p) };
            let castle = named(&mut g, p, "Castle Doom", Zone::Battlefield);
            g.state.players[p.index()].mana = [0; 6];
            checked(
                &mut g,
                Action::ActivateMana {
                    card: castle,
                    ability: 1,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [0, 0, 0, 0, 1, 0],
                },
            );
            if filter {
                g.state.players[p.index()].mana[5] = 3;
                reject(
                    &mut g,
                    Action::AnnounceMana {
                        card: b,
                        ability: 1,
                        counter_payment: Counters::default(),
                        production: [1, 1, 1, 1, 0, 0],
                    },
                );
            } else {
                g.state.objects[b as usize].tapped = true; // Exclude Baxter's unrelated C ability.
                reject(
                    &mut g,
                    Action::AnnounceActivate {
                        card: r,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
            }
        }
    }
}
#[test]
fn msh_baxter_raft_orphan_production_announcement_retains_exact_split() {
    for p in [Player::P0, Player::P1] {
        let (mut raw, mut indexed, snapshot, revision, index, action, b) = {
            let mut parent = setup(p);
            let b = land(&mut parent, p);
            parent.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            checked(
                &mut parent,
                Action::AnnounceMana {
                    card: b,
                    ability: 1,
                    counter_payment: Counters::default(),
                    production: [1, 1, 1, 1, 0, 0],
                },
            );
            let action = Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 4],
            };
            let index = parent
                .legal_actions()
                .iter()
                .position(|a| *a == action)
                .unwrap();
            (
                parent.clone(),
                parent.clone(),
                parent.snapshot(),
                parent.revision(),
                index,
                action,
                b,
            )
        };
        raw.apply(p, action).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[p.index()].mana, [1, 1, 1, 1, 0, 0]);
        assert!(raw.object_view(b).tapped);
        indexed.restore(&snapshot).unwrap();
        assert_eq!(
            indexed.state.announcing.as_ref().unwrap().production,
            Some([1, 1, 1, 1, 0, 0])
        );
        checked(
            &mut indexed,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 4],
            },
        );
        assert_eq!(indexed.state.players[p.index()].mana, [1, 1, 1, 1, 0, 0]);
    }
}
#[test]
fn msh_baxter_raft_ward_is_separate_from_discount_and_keeps_activation_payer() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let r = raft(&mut g, p);
            let target = named(
                &mut g,
                p.other(),
                "Captain America, Wings of Freedom",
                Zone::Battlefield,
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            announce(&mut g, r, 0);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            assert_eq!(
                g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
                1
            );
            finish(&mut g);
            g.move_object(r, Zone::Hand);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            assert_eq!(g.actor(), Some(p));
            assert_eq!(g.observe(p).resolution_payment.unwrap().payer, p);
            checked(
                &mut g,
                if accept {
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 1],
                    }
                } else {
                    Action::DeclineResolutionMana
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(target).tapped, accept);
        }
    }
}
#[test]
fn msh_baxter_raft_baxter_opponent_turn_and_controlled_effective_witness_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = land(&mut g, p);
        let witness = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.add_counter(witness, CounterKind::PlusOne, 1);
        g.state.active = p.other();
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        let a = Action::AnnounceActivate {
            card: b,
            ability: 2,
            counter_payment: Counters::default(),
        };
        reject(&mut g, a);
        g.change_controller(g.object_ref(witness), p);
        let before = g.hand_count(p);
        announce(&mut g, b, 2);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
