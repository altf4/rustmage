use super::tests::{act, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn announce(g: &mut Game, id: CardId, ability: u16) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::AnnounceActivate{card,ability:n,..}if *card==id&&*n==ability))
        .expect("available announcement");
    replay(g, a);
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .expect("available payment");
    replay(g, a);
}
fn equivalent(g: &mut Game, mana: Mana, generic: u32) {
    let o = g.observe(g.actor().unwrap());
    let n = o
        .cost_choices
        .iter()
        .find(|c| {
            c.cost
                == ManaCost {
                    colored: mana,
                    generic,
                }
        })
        .expect("offered equivalent")
        .index;
    replay(g, Action::ChooseActivationCost { cost: n });
}

#[test]
fn direct_hybrid_payments_include_either_color_and_deduplicate_overlapping_allocations() {
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Cindering Cutthroat", Zone::Battlefield);
    assert_eq!(g.object_view(id).card.activations().len(), 1);
    for pool in [[0, 0, 2, 0, 0, 0], [0, 0, 0, 2, 0, 0], [0, 0, 1, 1, 0, 0]] {
        g.state.players[0].mana = pool;
        let actions: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter(|a| matches!(a,Action::Activate{card,..}if *card==id))
            .collect();
        assert_eq!(actions.len(), 1);
        let a = actions[0].clone();
        if let Action::Activate { payment, .. } = a {
            assert_eq!(payment, pool);
        }
        let saved = g.snapshot();
        replay(&mut g, a);
        passes(&mut g);
        assert!(g.keywords(id).contains(Keywords::MENACE));
        g.restore(&saved).unwrap();
    }
    reject(
        &mut g,
        Action::Activate {
            card: id,
            ability: 0,
            target: None,
            payment: [2, 0, 0, 0, 0, 0],
            counter_payment: Counters::default(),
        },
    );
}

#[test]
fn activation_cost_equivalent_is_committed_before_payment_and_choices_are_controller_owned() {
    for color in [2, 3] {
        let mut g = scenario();
        let id = named(&mut g, Player::P0, "Cindering Cutthroat", Zone::Battlefield);
        named(&mut g, Player::P0, "Swamp", Zone::Battlefield);
        named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
        g.state.players[0].mana = [0, 0, 2, 2, 0, 0];
        announce(&mut g, id, 0);
        let saved = g.snapshot();
        let revision = g.revision();
        assert_eq!(
            g.apply(Player::P1, Action::ChooseActivationCost { cost: 0 }),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        let o = g.observe(Player::P0);
        assert_eq!(o.schema, 51);
        assert_eq!(o.decision, "activation_cost");
        assert_eq!(o.cost_choices.len(), 2);
        assert!(o.payment_request.is_none());
        assert!(g.observe(Player::P1).cost_choices.is_empty());
        reject(&mut g, Action::ChooseSpellCost { cost: 0 });
        reject(&mut g, Action::ChooseActivationCost { cost: 999 });
        let mut required = [0; 6];
        required[color] = 1;
        equivalent(&mut g, required, 1);
        reject(&mut g, Action::ChooseActivationCost { cost: 0 });
        let mut wrong_payment = [0; 6];
        wrong_payment[if color == 2 { 3 } else { 2 }] = 2;
        reject(
            &mut g,
            Action::PayCosts {
                payment: wrong_payment,
            },
        );
        let o = g.observe(Player::P0);
        assert_eq!(o.decision, "pay_costs");
        assert_eq!(o.payment_request.unwrap().required, required);
        // Existing mana sources supply the selected exact colored requirement.
        for _ in 0..3 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayCosts { .. }))
            {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::TapMana { .. }))
                .unwrap();
            replay(&mut g, a);
        }
        pay(&mut g);
        passes(&mut g);
        assert!(g.keywords(id).contains(Keywords::MENACE));
    }
}

#[test]
fn only_reachable_activation_equivalents_are_offered_without_mutating_funding_state() {
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Cindering Cutthroat", Zone::Battlefield);
    named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
    named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
    let saved = g.snapshot();
    for _ in 0..3 {
        let _ = g.legal_actions();
        let _ = g.observe(Player::P0);
    }
    assert_eq!(g.snapshot(), saved);
    announce(&mut g, id, 0);
    let o = g.observe(Player::P0);
    assert_eq!(o.cost_choices.len(), 1);
    assert_eq!(o.cost_choices[0].cost.colored, [0, 0, 0, 1, 0, 0]);
    let n = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cost_options()
        .enumerate()
        .find(|(_, c)| c.colored[2] == 1)
        .unwrap()
        .0;
    reject(&mut g, Action::ChooseActivationCost { cost: n as u16 });
}

#[test]
fn hurler_selects_one_ability_equivalent_before_target_and_pays_reserved_counter_at_commit() {
    for required in [[2, 0, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0], [0, 0, 0, 2, 0, 0]] {
        let mut g = scenario();
        let id = named(&mut g, Player::P0, "Hovel Hurler", Zone::Hand);
        g.move_object(id, Zone::Battlefield);
        let target = named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
        let opponent = named(&mut g, Player::P1, "Ordinary Bear", Zone::Battlefield);
        g.state.players[0].mana = [2, 0, 0, 2, 0, 0];
        assert_eq!(
            (g.object_view(id).power, g.object_view(id).toughness),
            (4, 5)
        );
        announce(&mut g, id, 0);
        assert_eq!(g.object_view(id).counters.minus_one, 2);
        assert_eq!(g.observe(Player::P0).cost_choices.len(), 3);
        assert!(g.observe(Player::P0).payment_request.is_none());
        equivalent(&mut g, required, 0);
        assert_eq!(g.observe(Player::P0).decision, "targets");
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(opponent),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        assert_eq!(g.object_view(id).counters.minus_one, 2);
        pay(&mut g);
        assert_eq!(g.object_view(id).counters.minus_one, 1);
        passes(&mut g);
        assert_eq!(
            (g.object_view(target).power, g.object_view(target).toughness),
            (5, 5)
        );
        assert!(g.keywords(target).contains(Keywords::FLYING));
    }
}

#[test]
fn hurler_sorcery_and_counter_restrictions_apply_to_every_cost_route() {
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Hovel Hurler", Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
    g.state.players[0].mana = [2, 0, 0, 2, 0, 0];
    let offered = |g: &Game| {
        g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,..}|Action::AnnounceActivate{card,..}|Action::BeginActivate{card,..}if *card==id))
    };
    assert!(offered(&g));
    g.state.step = Step::BeginCombat;
    assert!(!offered(&g));
    g.state.step = Step::Main1;
    g.state.objects[id as usize].counters = Counters::default();
    assert!(!offered(&g));
}

#[test]
fn all_three_trostani_abilities_keep_their_indices_and_temporary_keyword_bodies() {
    for (index, color, generic, keyword) in [
        (0, 4, 1, Keywords::DEATHTOUCH),
        (1, 0, 0, Keywords::VIGILANCE),
        (1, 4, 0, Keywords::VIGILANCE),
        (2, 0, 2, Keywords::DOUBLE_STRIKE),
    ] {
        let mut g = scenario();
        let id = named(
            &mut g,
            Player::P0,
            "Trostani, Three Whispers",
            Zone::Battlefield,
        );
        let target = named(&mut g, Player::P1, "Ordinary Bear", Zone::Battlefield);
        let mut mana = [0; 6];
        mana[color] = 1;
        g.state.players[0].mana = mana;
        g.state.players[0].mana[5] = generic;
        announce(&mut g, id, index);
        if index == 1 {
            equivalent(&mut g, mana, 0);
        }
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        passes(&mut g);
        assert!(g.keywords(target).contains(keyword));
        g.finish_cleanup();
        assert!(!g.keywords(target).contains(keyword));
    }
}

#[test]
fn cost_selection_and_target_continuation_survive_departed_mana_only_source() {
    let mut g = scenario();
    let id = named(
        &mut g,
        Player::P0,
        "Trostani, Three Whispers",
        Zone::Battlefield,
    );
    let target = named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
    g.state.players[0].mana[4] = 1;
    announce(&mut g, id, 1);
    equivalent(&mut g, [0, 0, 0, 0, 1, 0], 0);
    g.move_object(id, Zone::Graveyard);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(&mut g);
    passes(&mut g);
    assert!(g.keywords(target).contains(Keywords::VIGILANCE));
}

#[test]
fn immutable_alternatives_share_once_per_turn_and_nonmana_source_restrictions() {
    let mut g = scenario();
    let id = named(&mut g, Player::P0, "Cindering Cutthroat", Zone::Battlefield);
    let mut ability = g.object_view(id).card.activations()[0];
    ability.once_per_turn = true;
    ability.cost.tap = true;
    ability.cost.alternatives = &[
        ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
        },
        ManaCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 2,
        },
    ];
    let object = &mut g.state.objects[id as usize];
    object.summoning_sick = true;
    assert!(!Game::object_activation_available(
        object,
        Keywords::NONE,
        20,
        &ability,
        0,
        Player::P0,
        true
    ));
    object.summoning_sick = false;
    assert!(Game::object_activation_available(
        object,
        Keywords::NONE,
        20,
        &ability,
        0,
        Player::P0,
        true
    ));
    object.used_abilities = 1;
    assert!(!Game::object_activation_available(
        object,
        Keywords::NONE,
        20,
        &ability,
        0,
        Player::P0,
        true
    ));
    assert_eq!(ability.cost.mana_options().count(), 3);
}

#[test]
fn selected_two_brid_future_ir_locks_generic_total_through_owned_payment_and_replay() {
    // Explicit selected-cost IR verifies generic2 versus colored1 plumbing.
    // Actual ChooseActivationCost still accepts only printed catalogue options.
    for (cost, pool) in [
        (
            ManaCost {
                colored: [0; 6],
                generic: 2,
            },
            [0, 0, 0, 0, 0, 2],
        ),
        (
            ManaCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 0,
            },
            [1, 0, 0, 0, 0, 0],
        ),
    ] {
        let mut g = scenario();
        let id = named(&mut g, Player::P0, "Cindering Cutthroat", Zone::Battlefield);
        g.state.players[0].mana = [0, 0, 1, 1, 0, 0];
        announce(&mut g, id, 0);
        g.state.announcing.as_mut().unwrap().cost = Some(cost);
        g.state.players[0].mana = pool;
        g.continue_announcement_choices(false);
        let request = g.observe(Player::P0).payment_request.unwrap();
        assert_eq!(request.required, cost.colored);
        assert_eq!(request.generic, cost.generic);
        pay(&mut g);
        passes(&mut g);
        assert!(g.keywords(id).contains(Keywords::MENACE));
        assert_eq!(g.state.players[0].mana, [0; 6]);
    }
}
