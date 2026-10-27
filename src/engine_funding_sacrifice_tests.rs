//! Selected mana costs must spend exact projected objects, including the last haste witness.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
use crate::engine::subtypes::CreatureTypesEffectView;

fn goal(p: Player) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0, 0, 1, 1, 0, 0],
        generic: 0,
        reserve: None,
        source: None,
    }
}
fn fixture(p: Player) -> (Game, CardId, CardId) {
    let mut g = setup(p);
    let bolg = named(&mut g, p, "Bolg's Company", Zone::Battlefield);
    let fodder = named(&mut g, p, "Goblin Piker", Zone::Battlefield);
    g.state.objects[bolg as usize].summoning_sick = true;
    g.state.players[p.index()].mana = [0; 6];
    assert!(g.keywords(bolg).contains(Keywords::HASTE));
    (g, bolg, fodder)
}
fn step(w: &FundingState<'_>, p: Player, bolg: CardId, fodder: CardId) -> FundingMove {
    w.moves(&goal(p))
        .into_iter()
        .find(|m| {
            w.resources[m.resource].id == bolg
                && m.external.is_some_and(|e| e.object.card == fodder)
        })
        .expect("Bolg must offer a selected-sacrifice mana move")
}
fn child<'g>(w: &FundingState<'g>, m: &FundingMove) -> FundingState<'g> {
    let mut result = None;
    w.visit_applied(m, &mut |n| {
        result = Some(n.clone());
        true
    });
    result.expect("realizable selected payment")
}
fn owned_step(g: &mut Game, action: Action) {
    let p = g.actor().unwrap();
    let before = g.snapshot();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap_or_else(|| panic!("missing {action:?}; pending {:?}", g.state.pending));
    let mut raw = g.clone();
    let mut indexed = {
        let parent = g.clone();
        parent.clone()
    };
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(g.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), before);
    g.apply(p, action.clone()).unwrap();
    let after = g.snapshot();
    g.restore(&before).unwrap();
    g.apply(p, action).unwrap();
    assert_eq!(g.snapshot(), after);
}
fn checked(g: &mut Game, action: Action) {
    assert!(g.legal_actions().contains(&action), "missing {action:?}");
    g.apply(g.actor().unwrap(), action).unwrap();
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .expect("payable costs");
    checked(g, action);
}
fn mana(g: &mut Game, bolg: CardId, fodder: CardId) {
    mana_contract(g, bolg, fodder, false);
}
fn mana_contract(g: &mut Game, bolg: CardId, fodder: CardId, branch: bool) {
    checked(
        g,
        Action::AnnounceMana {
            card: bolg,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 1, 1, 0, 0],
        },
    );
    let saved = g.snapshot();
    let invalid = Action::ChooseAdditionalCost {
        object: g.object_ref(bolg),
    };
    assert!(g.apply(g.actor().unwrap(), invalid).is_err());
    assert_eq!(g.snapshot(), saved);
    checked(
        g,
        Action::ChooseAdditionalCost {
            object: g.object_ref(fodder),
        },
    );
    if branch {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        owned_step(g, action);
    } else {
        pay(g);
    }
}
fn finish_targets(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
}
fn settle(g: &mut Game) {
    for _ in 0..16 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("settle budget");
}

#[test]
fn hob_sacrifice_funding_last_goblin_haste_exact_projection_and_purity() {
    for p in [Player::P0, Player::P1] {
        let (g, bolg, fodder) = fixture(p);
        let saved = g.snapshot();
        assert!(g.can_fund(goal(p)));
        let w = FundingState::from_game(&g, &goal(p));
        assert!(
            w.resources.iter().any(|r| r.id == fodder),
            "nonmana Goblin retained"
        );
        let m = step(&w, p, bolg, fodder);
        assert_eq!(m.external.unwrap().object, g.object_ref(fodder));
        let n = child(&w, &m);
        assert!(n.object(bolg).tapped);
        assert_eq!(n.object(fodder).zone, Zone::Graveyard);
        assert_ne!(n.reference(fodder), g.object_ref(fodder));
        assert!(!n.activation_keywords(bolg).contains(Keywords::HASTE));
        assert_eq!(n.mana, [0, 0, 1, 1, 0, 0]);
        assert!(n.moves(&goal(p)).is_empty(), "last Goblin cannot be reused");
        assert!(n.key(&goal(p)) != w.key(&goal(p)));
        let mut stale = m.clone();
        stale.external.as_mut().unwrap().object.generation += 1;
        assert!(!w.visit_applied(&stale, &mut |_| panic!("stale selection accepted")));
        let mut four = goal(p);
        four.generic = 2;
        assert!(!g.can_fund(four), "one mana activation cannot pay twice");
        assert_eq!(g.snapshot(), saved);
    }
}

#[test]
fn hob_sacrifice_funding_projected_subtype_control_readiness_and_owner() {
    for p in [Player::P0, Player::P1] {
        let (mut g, bolg, fodder) = fixture(p);
        // A stolen Goblin is sacrificeable; ownership is not control.
        g.state.objects[fodder as usize].owner = p.other();
        assert!(g.can_fund(goal(p)));
        let w = FundingState::from_game(&g, &goal(p));
        let n = child(&w, &step(&w, p, bolg, fodder));
        assert_eq!(n.object(fodder).controller, p.other());
        assert_eq!(n.object(fodder).owner, p.other());
        for mode in 0..3 {
            let mut n = w.clone();
            let i = n.resources.iter().position(|r| r.id == fodder).unwrap();
            match mode {
                0 => n.resources[i].object.controller = p.other(),
                1 => n.resources[i].object.zone = Zone::Graveyard,
                _ => n.resources[i].object.card = Card::from_name("Grizzly Bears").unwrap(),
            }
            assert!(!n.activation_keywords(bolg).contains(Keywords::HASTE));
            assert!(
                n.moves(&goal(p)).is_empty(),
                "projected loss of last Goblin: {mode}"
            );
        }
        let mut n = w.clone();
        let i = n.resources.iter().position(|r| r.id == bolg).unwrap();
        n.resources[i].object.tapped = true;
        assert!(n.moves(&goal(p)).is_empty());
        g.state.objects[fodder as usize].controller = p.other();
        assert!(!g.can_fund(goal(p)));
        g.state.objects[fodder as usize].controller = p;
        g.state.objects[fodder as usize].card = Card::from_name("Grizzly Bears").unwrap();
        assert!(!g.can_fund(goal(p)));
    }
}

#[test]
fn hob_sacrifice_funding_reserved_cost_cannot_spend_same_goblin_twice() {
    for p in [Player::P0, Player::P1] {
        let (g, bolg, fodder) = fixture(p);
        let spec = g.state.objects[bolg as usize].card.activations()[0]
            .cost
            .additional
            .unwrap();
        let mut required = goal(p);
        required.reserve = Some((g.object_ref(fodder), spec));
        assert!(!g.can_fund(required));
    }
}

#[test]
fn hob_sacrifice_funding_replacements_and_indestructible_do_not_block_payment() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let exile = mode != 0;
            let (mut g, bolg, fodder) = fixture(p);
            g.state.objects[fodder as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            if mode == 1 {
                named(&mut g, p, "Rest in Peace", Zone::Battlefield);
            } else if mode == 2 {
                g.add_counter(fodder, CounterKind::Finality, 1);
            }
            let saved = g.snapshot();
            let w = FundingState::from_game(&g, &goal(p));
            let n = child(&w, &step(&w, p, bolg, fodder));
            assert_eq!(
                n.object(fodder).zone,
                if exile { Zone::Exile } else { Zone::Graveyard }
            );
            assert_eq!(
                n.creatures_died - w.creatures_died,
                if exile { 0 } else { 1 }
            );
            assert!(g.can_fund(goal(p)));
            assert_eq!(g.snapshot(), saved);
            let expected = n.object(fodder).zone;
            drop(n);
            drop(w);
            mana(&mut g, bolg, fodder);
            assert_eq!(g.object_view(fodder).zone, expected);
            assert!(g.object_view(bolg).tapped);
            assert!(!g.keywords(bolg).contains(Keywords::HASTE));
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 1, 0, 0]);
        }
    }
}

#[test]
fn hob_sacrifice_funding_cast_and_equip_only_through_bolg_raw_index_restore() {
    for p in [Player::P0, Player::P1] {
        for spell in [false, true] {
            let (mut g, bolg, fodder) = fixture(p);
            let item = named(
                &mut g,
                p,
                if spell {
                    "Bolg's Company"
                } else {
                    "Goldvein Pick"
                },
                if spell { Zone::Hand } else { Zone::Battlefield },
            );
            if spell {
                checked(&mut g, Action::AnnounceCast { card: item });
            } else {
                checked(
                    &mut g,
                    Action::AnnounceActivate {
                        card: item,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(bolg),
                    },
                );
            }
            finish_targets(&mut g);
            mana_contract(&mut g, bolg, fodder, spell && p == Player::P0);
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
            if spell {
                assert_eq!(g.object_view(item).zone, Zone::Battlefield);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            } else {
                assert_eq!(
                    g.state.objects[item as usize].attached_to,
                    Some(g.object_ref(bolg))
                );
                assert_eq!(g.state.players[p.index()].mana.iter().sum::<u32>(), 1);
            }
        }
    }
}

#[test]
fn hob_sacrifice_funding_layer_four_goblin_shuffle_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bolg = named(&mut g, p, "Bolg's Company", Zone::Battlefield);
        let colossus = named(&mut g, p, "Darksteel Colossus", Zone::Battlefield);
        // Shared layer-four fixture: a permanent additional Goblin type, using
        // the same owned type row as amass. Printed identity remains Colossus.
        g.grant_amass_goblin_type(g.object_ref(bolg), g.object_ref(colossus));
        g.state.objects[bolg as usize].summoning_sick = true;
        g.state.players[p.index()].mana = [0; 6];
        let saved = g.snapshot();
        let w = FundingState::from_game(&g, &goal(p));
        let n = child(&w, &step(&w, p, bolg, colossus));
        assert_eq!(n.object(colossus).zone, Zone::Library);
        assert_eq!(n.creatures_died, w.creatures_died);
        assert_eq!(n.mana, [0, 0, 1, 1, 0, 0]);
        assert!(g.can_fund(goal(p)));
        assert_eq!(g.snapshot(), saved);
        drop(n);
        drop(w);
        mana(&mut g, bolg, colossus);
        assert_eq!(g.object_view(colossus).zone, Zone::Library);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 1, 0, 0]);
    }
}

#[test]
fn hob_sacrifice_funding_real_amass_adds_goblin_to_non_goblin_army() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bolg = named(&mut g, p, "Bolg's Company", Zone::Battlefield);
        let army = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        // Existing Army, before the real amass instruction adds Goblin.
        let reference = g.object_ref(army);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .creature_types
            .push(CreatureTypesEffectView {
                recipient: reference,
                source: reference,
                types: &["Army"],
                additional: false,
                timestamp,
            });
        let rage = named(&mut g, p, "Rage into the Valley", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: rage });
        finish_targets(&mut g);
        pay(&mut g);
        settle(&mut g);
        assert!(!g.state.objects[army as usize].card.has_subtype("Goblin"));
        assert!(g.has_subtype(army, "Goblin"));
        assert!(g.has_subtype(army, "Army"));
        g.state.objects[bolg as usize].summoning_sick = true;
        g.state.players[p.index()].mana = [0; 6];
        let saved = g.snapshot();
        assert!(g.can_fund(goal(p)));
        assert_eq!(g.snapshot(), saved);
        mana(&mut g, bolg, army);
        assert_eq!(g.object_view(army).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 1, 0, 0]);
    }
}

#[test]
fn hob_sacrifice_funding_borrowed_activation_needs_projected_haste_before_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let forge = named(&mut g, p, "Territory Forge", Zone::Battlefield);
        let bolg = named(&mut g, p, "Bolg's Company", Zone::Exile);
        let fodder = named(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let source = g.object_ref(forge);
        let exiled = g.object_ref(bolg);
        g.state
            .borrowed_links
            .get_or_insert_default()
            .push(borrowed_activations::Link {
                source,
                origin: None,
                exiled,
                first_slot: 0,
                width: 1,
            });
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .animations
            .push(AnimationView {
                object: source,
                timestamp,
                artifact: true,
                land: false,
                preserve_card_types: false,
                food_only: false,
                artifact_noncreature: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                expires_turn: None,
            });
        g.state.objects[forge as usize].summoning_sick = true;
        g.state.players[p.index()].mana = [0; 6];
        assert_ne!(
            g.current_kinds(forge) & 1,
            0,
            "fixture must actually animate Forge into a creature"
        );
        assert!(!g.keywords(forge).contains(Keywords::HASTE));
        let saved = g.snapshot();
        let w = FundingState::from_game(&g, &goal(p));
        assert!(
            w.moves(&goal(p)).is_empty(),
            "a borrowed mana ability does not also inherit Bolg's conditional haste"
        );
        assert!(!g.can_fund(goal(p)));
        let mut hasty = w.clone();
        let i = hasty.resources.iter().position(|r| r.id == forge).unwrap();
        hasty.resources[i].object.gained_keywords = Keywords::HASTE;
        let m = step(&hasty, p, forge, fodder);
        assert_eq!(
            m.borrowed.unwrap().definition,
            Card::from_name("Bolg's Company").unwrap()
        );
        let n = child(&hasty, &m);
        assert_eq!(n.mana, [0, 0, 1, 1, 0, 0]);
        assert_eq!(n.object(fodder).zone, Zone::Graveyard);
        assert_eq!(g.snapshot(), saved);
        drop(n);
        drop(hasty);
        drop(w);
        g.state.objects[forge as usize].gained_keywords = Keywords::HASTE;
        assert!(g.can_fund(goal(p)));
        checked(
            &mut g,
            Action::AnnounceBorrowedMana {
                card: forge,
                exiled,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 1, 1, 0, 0],
            },
        );
        let object = g.object_ref(fodder);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
        assert!(g.object_view(forge).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 1, 0, 0]);
    }
}
#[test]
fn hob_sacrifice_funding_memo_retains_frozen_nested_selection_identity() {
    for p in [Player::P0, Player::P1] {
        let (mut g, bolg, selected) = fixture(p);
        let interchangeable = named(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let spell = named(&mut g, p, "Bolg's Company", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        finish_targets(&mut g);
        checked(
            &mut g,
            Action::AnnounceMana {
                card: bolg,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 1, 1, 0, 0],
            },
        );
        let object = g.object_ref(selected);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        let saved = g.snapshot();
        let required = goal(p);
        let mut w = FundingState::from_game(&g, &required);
        w.set_funding_continuation(
            g.state.announcing.as_ref().unwrap(),
            g.state.mana_announcement_stack.as_deref().unwrap(),
            None,
        );
        assert!(w.continuation.is_some());
        let i = w.resources.iter().position(|r| r.id == selected).unwrap();
        let j = w
            .resources
            .iter()
            .position(|r| r.id == interchangeable)
            .unwrap();
        let mut a = w.clone();
        let mut b = w.clone();
        a.resources[i].object.zone = Zone::Graveyard;
        b.resources[j].object.zone = Zone::Graveyard;
        assert!(
            a.key(&required) != b.key(&required),
            "losing the frozen selection differs from losing an otherwise identical Goblin"
        );
        a.continuation = None;
        b.continuation = None;
        assert!(
            a.key(&required) == b.key(&required),
            "ordinary unlinked identical Goblins remain interchangeable"
        );
        assert_eq!(g.snapshot(), saved);
    }
}

#[test]
fn hob_sacrifice_funding_ordered_source_departure_matches_native_cost_executor() {
    for p in [Player::P0, Player::P1] {
        for exile_source in [false, true] {
            let (mut g, bolg, fodder) = fixture(p);
            let rip = named(&mut g, p, "Rest in Peace", Zone::Battlefield);
            checked(
                &mut g,
                Action::AnnounceMana {
                    card: bolg,
                    ability: 0,
                    counter_payment: Counters::default(),
                    production: [0, 0, 1, 1, 0, 0],
                },
            );
            let object = g.object_ref(fodder);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            let mut announcement = g.state.announcing.take().unwrap();
            let source = g.object_ref(rip);
            // Existing bounded typed executor fixture: retain the immutable Bolg
            // selected-cost definition, but make RIP the source paying a source
            // departure component before that additional cost.
            announcement.item.card = rip;
            announcement.item.object = source;
            let mut definition = *announcement.item.source_card.activation(0);
            definition.cost.sacrifice = !exile_source;
            definition.cost.exile_self = exile_source;
            let spec = definition.cost.additional.unwrap();
            let required = goal(p);
            let w = FundingState::from_game(&g, &required);
            let external = ExternalManaCost {
                resource: w.resources.iter().position(|r| r.id == fodder).unwrap(),
                object,
                kind: spec.kind,
            };
            let mut projected = None;
            w.visit_mana_cost_departures(
                source,
                Some(if exile_source {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }),
                Some((external, spec)),
                &mut |n| {
                    projected = Some(n.clone());
                    true
                },
            );
            let projected = projected.unwrap();
            let expected = (
                projected.object(rip).zone,
                projected.object(fodder).zone,
                projected.reference(rip),
                projected.reference(fodder),
                projected.creatures_died,
            );
            drop(projected);
            drop(w);
            g.pay_activation_definition(
                &mut announcement.item,
                [0; 6],
                Counters::default(),
                0,
                &definition,
            );
            g.pay_additional_cost(&mut announcement);
            assert_eq!(g.object_view(rip).zone, Zone::Exile);
            assert_eq!(
                g.object_view(fodder).zone,
                Zone::Graveyard,
                "RIP has already departed before the additional sacrifice"
            );
            assert_eq!(
                (
                    g.object_view(rip).zone,
                    g.object_view(fodder).zone,
                    g.object_ref(rip),
                    g.object_ref(fodder),
                    g.state.creatures_died_this_turn
                ),
                expected
            );
        }
    }
}

#[path = "assurance/hob_head_hunt_funding.rs"]
mod head_hunt_tests;
