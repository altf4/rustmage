use super::tests::{act, cast, scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, Zone::Hand)
}
fn replay(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let old_revision = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, action.clone());
    act(g, action.clone());
    let after = g.snapshot();
    assert_eq!(after, fork.snapshot());
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, old_revision, 0),
        Err(Error::StaleRevision)
    );
    act(g, action);
    assert_eq!(after, g.snapshot());
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
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
fn target_spell(g: &mut Game, name: &str) -> CardId {
    g.state.players[0].mana = [30; 6];
    let id = named(g, Player::P0, name);
    cast(
        g,
        id,
        if name == "Shock" {
            Some(Target::Player(Player::P1))
        } else {
            None
        },
    );
    replay(g, Action::Pass);
    id
}
fn counter_action(g: &Game, counter: CardId, target: CardId) -> Action {
    let (mut payment, generic) = g.state.objects[counter as usize].card.cost();
    payment[5] += generic;
    Action::Cast {
        card: counter,
        target: Some(Target::Spell(target)),
        payment,
    }
}

#[test]
fn restricted_counter_color_unions_use_spell_colors_not_identity_or_mana_paid() {
    for (counter_name, yes, no) in [
        (
            "Flashfreeze",
            vec!["Shock", "Bear Cub", "Anthem of Champions"],
            vec!["Opt", "Hinterland Sanctifier", "Three Tree Mascot"],
        ),
        (
            "Precise Redaction",
            vec![
                "Hinterland Sanctifier",
                "Phyrexian Arena",
                "Anthem of Champions",
                "Cloudblazer",
            ],
            vec!["Opt", "Shock", "Three Tree Mascot"],
        ),
    ] {
        for (names, expected) in [(yes, true), (no, false)] {
            for name in names {
                let mut g = scenario();
                let target = target_spell(&mut g, name);
                let counter = named(&mut g, Player::P1, counter_name);
                g.state.players[1].mana = [20; 6];
                let action = counter_action(&g, counter, target);
                assert_eq!(
                    g.legal_actions().contains(&action),
                    expected,
                    "{counter_name}: {name}"
                );
                if expected {
                    replay(&mut g, action);
                    passes(&mut g);
                    assert_eq!(g.object_view(target).zone, Zone::Graveyard);
                    assert_eq!(g.object_view(counter).zone, Zone::Graveyard);
                } else {
                    reject(&mut g, action);
                }
            }
        }
    }
}

#[test]
fn restricted_counter_mana_value_equality_and_threshold_have_exact_boundaries() {
    for (name, mv) in [
        ("Hinterland Sanctifier", 1),
        ("Bear Cub", 2),
        ("Goldvein Pick", 2),
        ("Phyrexian Arena", 3),
        ("Serra Angel", 5),
        ("Shivan Dragon", 6),
        ("Ordinary Bear", 4),
    ] {
        for counter_name in ["Spell Snare", "Disdainful Stroke"] {
            let mut g = scenario();
            let target = target_spell(&mut g, name);
            assert_eq!(g.state.objects[target as usize].card.fixed_mana_value(), mv);
            // Payment costs neither define nor alter a spell's mana value.
            g.state.players[0].mana = [0; 6];
            let counter = named(&mut g, Player::P1, counter_name);
            g.state.players[1].mana = [20; 6];
            let action = counter_action(&g, counter, target);
            let expected = if counter_name == "Spell Snare" {
                mv == 2
            } else {
                mv >= 4
            };
            assert_eq!(
                g.legal_actions().contains(&action),
                expected,
                "{counter_name}: {name}"
            );
            if expected {
                replay(&mut g, action);
                passes(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            } else {
                reject(&mut g, action);
            }
        }
    }
}

#[test]
fn annul_and_spectral_union_arms_include_multitype_spells_and_exclude_unrelated_types() {
    for (name, annul, spectral) in [
        ("Goldvein Pick", true, true),
        ("Anthem of Champions", true, false),
        ("Bear Cub", false, true),
        ("Three Tree Mascot", true, true),
        ("Opt", false, false),
    ] {
        for (counter_name, expected) in [("Annul", annul), ("Spectral Interference", spectral)] {
            let mut g = scenario();
            let target = target_spell(&mut g, name);
            let counter = named(&mut g, Player::P1, counter_name);
            g.state.players[1].mana = [20; 6];
            let action = counter_action(&g, counter, target);
            assert_eq!(
                g.legal_actions().contains(&action),
                expected,
                "{counter_name}: {name}"
            );
            if expected {
                replay(&mut g, action);
                passes(&mut g);
                if spectral && counter_name == "Spectral Interference" {
                    replay(&mut g, Action::DeclineResolutionMana);
                }
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            } else {
                reject(&mut g, action);
            }
        }
    }
}

#[test]
fn spectral_payment_checks_four_mana_and_restores_both_outcomes() {
    for paid in [false, true] {
        let mut g = scenario();
        let target = target_spell(&mut g, "Bear Cub");
        g.state.players[0].mana = [0; 6];
        let counter = named(&mut g, Player::P1, "Spectral Interference");
        g.state.players[1].mana = [20; 6];
        let action = counter_action(&g, counter, target);
        replay(&mut g, action);
        passes(&mut g);
        assert_eq!(g.actor(), Some(Player::P0));
        assert_eq!(
            g.observe(Player::P0)
                .resolution_payment
                .unwrap()
                .cost
                .generic,
            4
        );
        assert!(g.observe(Player::P1).actions.is_empty());
        reject(&mut g, Action::Pass);
        reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
        if paid {
            g.state.players[0].mana[5] = 4;
            replay(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 4],
                },
            );
            assert_eq!(g.object_view(target).zone, Zone::Stack);
            passes(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        } else {
            replay(&mut g, Action::DeclineResolutionMana);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        }
        assert_eq!(g.object_view(counter).zone, Zone::Graveyard);
    }
}

#[test]
fn restricted_counter_revalidates_stack_incarnation_and_rejects_self_target() {
    let mut g = scenario();
    let target = target_spell(&mut g, "Shock");
    let counter = named(&mut g, Player::P1, "Flashfreeze");
    g.state.players[1].mana = [20; 6];
    let action = counter_action(&g, counter, target);
    replay(&mut g, action);
    let item = g.state.stack.last().unwrap().clone();
    assert!(!g.target_matches(
        TargetRule::Spell,
        Player::P1,
        item.object,
        Target::Spell(counter)
    ));
    let old = item.targets[0].object.unwrap();
    g.move_object(target, Zone::Hand);
    g.state.priority = Player::P0;
    cast(&mut g, target, Some(Target::Player(Player::P1)));
    assert_ne!(g.object_ref(target), old);
    g.state.stack.swap(0, 1);
    passes(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Stack);
    assert_eq!(g.state.stack.len(), 1);
    passes(&mut g);
    assert_eq!(g.state.players[1].life, 18);
}

#[test]
fn restricted_counter_rechecks_live_stack_characteristics_before_resolution() {
    for (counter_name, original, replacement) in [
        ("Flashfreeze", "Bear Cub", "Three Tree Mascot"),
        ("Spell Snare", "Bear Cub", "Shivan Dragon"),
        ("Annul", "Goldvein Pick", "Bear Cub"),
    ] {
        let mut g = scenario();
        let target = target_spell(&mut g, original);
        let counter = named(&mut g, Player::P1, counter_name);
        g.state.players[1].mana = [20; 6];
        let action = counter_action(&g, counter, target);
        replay(&mut g, action);
        // A synthetic metadata change isolates resolution revalidation. No card
        // that changes stack characteristics is admitted by this stage.
        g.state.stack[0].source_card = Card::from_name(replacement).unwrap();
        passes(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Stack);
        assert_eq!(g.object_view(counter).zone, Zone::Graveyard);
    }
}

#[test]
fn greatshark_flash_and_entry_counter_trigger_use_both_union_arms_and_survive_source_departure() {
    for name in ["Bear Cub", "Goldvein Pick", "Three Tree Mascot"] {
        let mut g = scenario();
        let target = target_spell(&mut g, name);
        let shark = named(&mut g, Player::P1, "Voracious Greatshark");
        g.state.players[1].mana = [20; 6];
        cast(&mut g, shark, None);
        passes(&mut g);
        assert_eq!(g.object_view(shark).zone, Zone::Battlefield);
        g.prepare_triggers();
        while !matches!(g.state.pending, Pending::Priority) {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(a, Action::PutTrigger { .. } | Action::FinishTargets)
                        || *a
                            == Action::ChooseTarget {
                                target: Target::Spell(target),
                            }
                })
                .unwrap();
            replay(&mut g, action);
        }
        assert_eq!(g.state.stack.len(), 2);
        g.move_object(shark, Zone::Hand);
        passes(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.object_view(shark).zone, Zone::Hand);
    }
}

#[test]
fn greatshark_enters_without_countering_when_no_legal_spell_is_present() {
    let mut g = scenario();
    g.state.players[0].mana = [20; 6];
    let shark = named(&mut g, Player::P0, "Voracious Greatshark");
    cast(&mut g, shark, None);
    passes(&mut g);
    g.prepare_triggers();
    while let Some(action) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        replay(&mut g, action);
    }
    assert!(g.state.stack.is_empty());
    assert_eq!(g.object_view(shark).zone, Zone::Battlefield);
    assert_eq!((g.stats(shark).power, g.stats(shark).toughness), (5, 4));
}

#[test]
fn hybrid_spell_keeps_both_colors_and_printed_mana_value_after_single_color_payment() {
    for (target_name, pool, counter_name) in [
        ("Pond Prophet", [0, 2, 0, 0, 0, 0], "Flashfreeze"),
        ("Pond Prophet", [0, 2, 0, 0, 0, 0], "Spell Snare"),
        ("Gallant Citizen", [0, 0, 0, 0, 2, 0], "Precise Redaction"),
    ] {
        let mut g = scenario();
        let target = named(&mut g, Player::P0, target_name);
        g.state.players[0].mana = pool;
        cast(&mut g, target, None);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        replay(&mut g, Action::Pass);
        let counter = named(&mut g, Player::P1, counter_name);
        g.state.players[1].mana = [20; 6];
        let action = counter_action(&g, counter, target);
        assert!(g.legal_actions().contains(&action));
        replay(&mut g, action);
        passes(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert!(g.state.stack.is_empty()); // Its ETB draw did not trigger.
    }
}
