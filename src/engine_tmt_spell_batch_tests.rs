//! Complete paid TMT spell/ETB consumers, private choices and exact-object costs.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, settle_on, setup};
use super::*;
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn finish(g: &mut Game) {
    let original = g.state.announcing.as_ref().map(|a| a.item.id);
    for _ in 0..12 {
        if g.state.announcing.as_ref().map(|a| a.item.id) != original
            || g.state.announcing.is_none()
        {
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
                        | Action::ChooseSpellCost { .. }
                )
            })
            .expect("payment");
        checked(g, action);
    }
    panic!("payment");
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    finish(g);
    id
}
fn food(g: &mut Game, p: Player) -> CardId {
    named(g, p, "@token/283fd83a9aec83a51b736926", Zone::Battlefield)
}
fn subtype_count(g: &Game, p: Player, kind: &str) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype(kind))
        .count()
}
fn resolve_to_choice(g: &mut Game) {
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("no choice");
}
fn activate(g: &mut Game, id: CardId, ability: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
#[test]
fn tmt_spells_baxter_robot_combat_target_filter_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let baxter = paid(&mut g, p, "Baxter Stockman");
        settle(&mut g);
        assert_eq!(subtype_count(&g, p, "Robot"), 1);
        let robot = g
            .state
            .objects
            .iter()
            .position(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Robot"))
            .unwrap() as CardId;
        let artifact = food(&mut g, p);
        let enemy = named(
            &mut g,
            p.other(),
            "@token/667dad32c9f20b70d5e9bda3",
            Zone::Battlefield,
        );
        for _ in 0..8 {
            if g.state.announcing.is_some() {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        assert!(g.state.announcing.is_some());
        for id in [baxter, artifact, enemy] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(robot),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.stats(robot).power, 4);
        assert!(
            g.keywords(robot)
                .contains(Keywords::FIRST_STRIKE.union(Keywords::VIGILANCE))
        );
        next_turn(&mut g);
        assert_eq!(g.stats(robot).power, 1);
        assert!(!g.keywords(robot).contains(Keywords::FIRST_STRIKE));
    }
}
#[test]
fn tmt_spells_nobody_optional_bounce_owner_and_scry_privacy() {
    for p in [Player::P0, Player::P1] {
        for choose in [false, true] {
            let mut g = setup(p);
            let artifact = food(&mut g, p);
            let enemy = food(&mut g, p.other());
            let nobody = paid(&mut g, p, "Nobody");
            for _ in 0..8 {
                if g.state.announcing.is_some() {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(nobody),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            if choose {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(artifact),
                    },
                );
            }
            finish(&mut g);
            for _ in 0..8 {
                if g.observe(p).library_choice.is_some() {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            assert!(g.observe(p).library_choice.is_some());
            assert!(g.observe(p.other()).library_choice.is_none());
            settle(&mut g);
            assert_eq!(
                g.state.objects[artifact as usize].zone,
                if choose {
                    Zone::Outside
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.hand_count(p), 0);
            assert!(g.state.resolving.is_none());
        }
    }
}
#[test]
fn tmt_spells_nobody_all_targets_illegal_prevents_scry() {
    let p = Player::P0;
    let mut g = setup(p);
    let artifact = food(&mut g, p);
    paid(&mut g, p, "Nobody");
    for _ in 0..8 {
        if g.state.announcing.is_some() {
            break;
        }
        checked(&mut g, Action::Pass);
    }
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    finish(&mut g);
    g.move_object(artifact, Zone::Exile);
    let library = g.state.players[p.index()].library.clone();
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].library, library);
    assert!(g.state.resolving.is_none());
}
#[test]
fn tmt_spells_pizza_any_target_self_damage_and_food_payment() {
    for p in [Player::P0, Player::P1] {
        for self_target in [false, true] {
            let mut g = setup(p);
            let pizza = paid(&mut g, p, "Spicy Oatmeal Pizza");
            settle_on(
                &mut g,
                Some(Target::Player(if self_target { p } else { p.other() })),
            );
            assert_eq!(
                g.state.players[p.index()].life,
                if self_target { 13 } else { 17 }
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                if self_target { 20 } else { 16 }
            );
            activate(&mut g, pizza, 0);
            finish(&mut g);
            assert_eq!(g.state.objects[pizza as usize].zone, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if self_target { 16 } else { 20 }
            );
        }
    }
}
#[test]
fn tmt_spells_pizza_invalid_target_fizzles_controller_damage() {
    let p = Player::P0;
    let mut g = setup(p);
    let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    paid(&mut g, p, "Spicy Oatmeal Pizza");
    for _ in 0..8 {
        if g.state.announcing.is_some() {
            break;
        }
        checked(&mut g, Action::Pass);
    }
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    finish(&mut g);
    g.move_object(bear, Zone::Hand);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, 20);
}
#[test]
fn tmt_spells_kitty_creature_or_token_cost_rejects_source_nontoken_artifact_and_opponent() {
    for p in [Player::P0, Player::P1] {
        for use_token in [false, true] {
            let mut g = setup(p);
            let kitty = paid(&mut g, p, "Ice Cream Kitty");
            settle(&mut g);
            let token = food(&mut g, p);
            let creature = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let artifact = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
            let enemy = food(&mut g, p.other());
            activate(&mut g, kitty, 0);
            for id in [kitty, artifact, enemy] {
                let object = g.object_ref(id);
                reject(&mut g, Action::ChooseAdditionalCost { object });
            }
            let selected = if use_token { token } else { creature };
            let object = g.object_ref(selected);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            finish(&mut g);
            assert_eq!(
                g.state.objects[selected as usize].zone,
                if use_token {
                    Zone::Outside
                } else {
                    Zone::Graveyard
                }
            );
            let hand = g.hand_count(p);
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 1);
            assert!(g.on_battlefield(kitty));
        }
    }
}
#[test]
fn tmt_spells_kitty_food_has_tap_summoning_restriction_and_stack_independent_draw() {
    let p = Player::P0;
    let mut g = setup(p);
    let kitty = paid(&mut g, p, "Ice Cream Kitty");
    settle(&mut g);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: kitty,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    let token = food(&mut g, p);
    activate(&mut g, kitty, 0);
    let object = g.object_ref(token);
    checked(&mut g, Action::ChooseAdditionalCost { object });
    finish(&mut g);
    g.move_object(kitty, Zone::Hand);
    let hand = g.hand_count(p);
    settle(&mut g);
    assert_eq!(g.hand_count(p), hand + 1);
}
#[test]
fn tmt_spells_metalhead_bounce_and_sacrifice_counter_keywords() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let metal = paid(&mut g, p, "Metalhead");
        settle_on(&mut g, Some(Target::Card(enemy)));
        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Hand);
        let token = food(&mut g, p);
        activate(&mut g, metal, 0);
        let object = g.object_ref(metal);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(token);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(metal, CounterKind::PlusOne), 1);
        assert!(
            g.keywords(metal)
                .contains(Keywords::MENACE.union(Keywords::HASTE))
        );
        next_turn(&mut g);
        assert!(!g.keywords(metal).contains(Keywords::HASTE));
        assert_eq!(g.stats(metal).power, 5);
    }
}
#[test]
fn tmt_spells_metalhead_does_not_put_counter_on_new_incarnation() {
    let p = Player::P0;
    let mut g = setup(p);
    let metal = paid(&mut g, p, "Metalhead");
    settle(&mut g);
    let token = food(&mut g, p);
    activate(&mut g, metal, 0);
    let object = g.object_ref(token);
    checked(&mut g, Action::ChooseAdditionalCost { object });
    finish(&mut g);
    g.move_object(metal, Zone::Exile);
    g.move_object(metal, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.counter_count(metal, CounterKind::PlusOne), 0);
    assert!(!g.keywords(metal).contains(Keywords::HASTE));
}
#[test]
fn tmt_spells_tainted_mana_value_boundary_artifact_and_indestructible() {
    for p in [Player::P0, Player::P1] {
        for (name, makes_food, dies) in [
            ("Grizzly Bears", true, true),
            ("Hedron Archive", true, true),
            ("Gigantosaurus", false, true),
            ("Grizzly Bears", true, false),
        ] {
            let mut g = setup(p);
            let target = named(&mut g, p.other(), name, Zone::Battlefield);
            if !dies {
                let source = g.object_ref(target);
                g.grant_keyword_effects(&[target], source, Keywords::INDESTRUCTIBLE);
            }
            let spell = named(&mut g, p, "Tainted Treats", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(target)]);
            settle(&mut g);
            assert_eq!(subtype_count(&g, p, "Food"), usize::from(makes_food));
            assert_eq!(g.on_battlefield(target), !dies);
        }
    }
}
#[test]
fn tmt_spells_tainted_token_mana_value_zero_and_illegal_target_no_food() {
    let p = Player::P0;
    for illegal in [false, true] {
        let mut g = setup(p);
        let target = food(&mut g, p.other());
        let spell = named(&mut g, p, "Tainted Treats", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(target)]);
        if illegal {
            g.move_object(target, Zone::Exile);
        }
        settle(&mut g);
        assert_eq!(subtype_count(&g, p, "Food"), usize::from(!illegal));
    }
}
#[test]
fn tmt_spells_manhole_private_optional_bottom_draw_and_decline() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let target = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let bottom = named(&mut g, p, "Island", Zone::Hand);
            let foreign = named(&mut g, p.other(), "Mountain", Zone::Hand);
            let spell = named(&mut g, p, "Manhole Missile", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(target)]);
            resolve_to_choice(&mut g);
            assert!(g.observe(p.other()).effect_choice.is_none());
            assert!(
                !serde_json::to_string(&g.observe(p.other()))
                    .unwrap()
                    .contains("Island")
            );
            let view = g.observe(p).effect_choice.unwrap();
            assert_eq!(view.kind, EffectChoiceKind::BottomHand);
            assert_eq!(view.candidates, vec![g.object_ref(bottom)]);
            let object = g.object_ref(foreign);
            reject(&mut g, Action::BottomHandCard { object });
            let mut object = g.object_ref(bottom);
            object.generation += 1;
            reject(&mut g, Action::BottomHandCard { object });
            let before = g.hand_count(p);
            if accept {
                let object = g.object_ref(bottom);
                checked(&mut g, Action::BottomHandCard { object });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
            assert_eq!(g.hand_count(p), before);
            assert!(g.observe(p.other()).last_target_library_move.is_none());
            assert!(
                !serde_json::to_string(&g.observe(p.other()))
                    .unwrap()
                    .contains("Island")
            );
            assert_eq!(
                g.state.objects[bottom as usize].zone,
                if accept { Zone::Library } else { Zone::Hand }
            );
            if accept {
                assert_eq!(g.state.players[p.index()].library.first(), Some(&bottom));
            }
        }
    }
}
#[test]
fn tmt_spells_manhole_empty_hand_same_private_phase_and_invalid_target_fizzles() {
    let p = Player::P0;
    for illegal in [false, true] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let spell = named(&mut g, p, "Manhole Missile", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(target)]);
        if illegal {
            g.move_object(target, Zone::Hand);
            settle(&mut g);
        } else {
            resolve_to_choice(&mut g);
            assert!(g.observe(p.other()).effect_choice.is_none());
            assert!(
                !serde_json::to_string(&g.observe(p.other()))
                    .unwrap()
                    .contains("Island")
            );
            assert!(g.observe(p).effect_choice.unwrap().candidates.is_empty());
            checked(&mut g, Action::DeclineEffectPayment);
            settle(&mut g);
        }
        assert_eq!(g.hand_count(p), 0);
    }
}
#[test]
fn tmt_spells_go_ninja_all_modes_and_recomputed_power_after_blink() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            g.set_counters(own, Counters::of(CounterKind::PlusOne, 4));
            let enemy = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            let original = g.object_ref(own);
            let spell = named(&mut g, p, "Go Ninja Go", Zone::Hand);
            let targets = match mode {
                0 => vec![Target::Card(own)],
                1 => vec![Target::Card(enemy)],
                _ => vec![Target::Card(own), Target::Card(enemy)],
            };
            cast(&mut g, spell, Some(mode), &targets);
            settle(&mut g);
            assert_eq!(
                g.state.objects[enemy as usize].zone,
                if mode == 1 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            if mode != 1 {
                assert_ne!(g.object_ref(own), original);
                assert_eq!(g.counter_count(own, CounterKind::PlusOne), 0);
            }
            assert_eq!(
                g.state.objects[enemy as usize].damage,
                if mode == 2 { 2 } else { 0 }
            );
        }
    }
}
#[test]
fn tmt_spells_go_ninja_both_partial_illegal_blink_does_not_select_enemy() {
    let p = Player::P0;
    let mut g = setup(p);
    let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let big = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
    let enemy = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
    let enemy_ref = g.object_ref(enemy);
    let spell = named(&mut g, p, "Go Ninja Go", Zone::Hand);
    cast(
        &mut g,
        spell,
        Some(2),
        &[Target::Card(own), Target::Card(enemy)],
    );
    g.move_object(own, Zone::Hand);
    settle(&mut g);
    assert_eq!(g.object_ref(enemy).generation, enemy_ref.generation + 1);
    assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
    assert!(g.on_battlefield(big));
}
#[test]
fn tmt_spells_nobody_returns_borrowed_artifact_to_owner() {
    let p = Player::P0;
    let mut g = setup(p);
    let artifact = named(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
    let object = g.object_ref(artifact);
    g.change_controller(object, p);
    paid(&mut g, p, "Nobody");
    settle_on(&mut g, Some(Target::Card(artifact)));
    assert_eq!(g.state.objects[artifact as usize].zone, Zone::Hand);
    assert_eq!(g.state.objects[artifact as usize].owner, p.other());
    assert_eq!(g.hand_count(p.other()), 1);
}
#[test]
fn tmt_spells_go_ninja_returns_borrowed_creature_under_owner_and_token_ceases() {
    let p = Player::P0;
    for token in [false, true] {
        let mut g = setup(p);
        let target = named(
            &mut g,
            p.other(),
            if token {
                "@token/667dad32c9f20b70d5e9bda3"
            } else {
                "Grizzly Bears"
            },
            Zone::Battlefield,
        );
        let object = g.object_ref(target);
        g.change_controller(object, p);
        let spell = named(&mut g, p, "Go Ninja Go", Zone::Hand);
        cast(&mut g, spell, Some(0), &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(
            g.state.objects[target as usize].zone,
            if token {
                Zone::Outside
            } else {
                Zone::Battlefield
            }
        );
        if !token {
            assert_eq!(g.state.objects[target as usize].controller, p.other());
        }
    }
}
#[test]
fn tmt_spells_kitty_sorcery_timing_and_mature_food_activation() {
    let p = Player::P0;
    let mut g = setup(p);
    let kitty = paid(&mut g, p, "Ice Cream Kitty");
    settle(&mut g);
    food(&mut g, p);
    checked(&mut g, Action::Pass);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: kitty,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::BeginCombat);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: kitty,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    next_turn(&mut g);
    next_turn(&mut g);
    while g.state.step != Step::Main1 {
        checked(&mut g, Action::Pass);
    }
    for state in &mut g.state.players {
        state.mana = [30; 6];
    }
    activate(&mut g, kitty, 1);
    finish(&mut g);
    settle(&mut g);
    assert_eq!(g.state.objects[kitty as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].life, 23);
}
