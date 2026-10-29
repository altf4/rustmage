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
            announcement.item.source.as_mut().unwrap().id = rip;
            announcement.item.source.as_mut().unwrap().object = source;
            let mut definition = *announcement.item.source.unwrap().card.activation(0);
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

#[test]
fn projected_borrowed_slots_match_owned_inventory_without_mutating_parent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let forge = named(&mut g, p, "Territory Forge", Zone::Battlefield);
        let donor = named(&mut g, p.other(), "Hedron Archive", Zone::Exile);
        let source = g.object_ref(forge);
        let exiled = g.object_ref(donor);
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
        g.state.borrowed_slots = borrowed_activations::refreshed_slots(&g);
        let saved = g.snapshot();
        let owned = borrowed_activations::abilities(&g, forge);
        let mut projected = FundingState::from_game(&g, &goal(p));
        projected.refresh_control_projection();
        let projected_abilities = borrowed_activations::abilities(&projected, forge);
        assert_eq!(
            owned
                .iter()
                .map(|a| (a.ability, a.slot))
                .collect::<Vec<_>>(),
            projected_abilities
                .iter()
                .map(|a| (a.ability, a.slot))
                .collect::<Vec<_>>()
        );
        assert_eq!(g.snapshot(), saved);
    }
}

#[test]
fn funding_key_distinguishes_same_land_added_type_and_timestamp() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Meticulous Commons", Zone::Battlefield);
        let reference = g.object_ref(land);
        let required = goal(p);
        let mut base = FundingState::from_game(&g, &required);
        base.prepare_entry_projection();
        let mut island = base.clone();
        let mut forest = base.clone();
        let mut later_island = base.clone();
        for (world, kind, timestamp) in [
            (&mut island, crate::cards::BasicLandType::Island, 7),
            (&mut forest, crate::cards::BasicLandType::Forest, 7),
            (&mut later_island, crate::cards::BasicLandType::Island, 8),
        ] {
            world
                .entry_projection
                .as_mut()
                .unwrap()
                .roles
                .basic_land_types
                .push(subtypes::BasicLandTypesEffectView {
                    recipient: reference,
                    source: reference,
                    kind,
                    timestamp,
                });
        }
        assert_eq!(island.intrinsic_mana_color(land), Some(1));
        assert_eq!(forest.intrinsic_mana_color(land), Some(4));
        assert!(island.key(&required) != forest.key(&required));
        assert!(island.key(&required) != later_island.key(&required));
    }
}

#[test]
fn projected_manifest_keeps_physical_identity_and_key_until_it_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Library);
        let movement = g.manifest_entry(g.object_ref(elf), p).unwrap();
        let required = goal(p);
        let world = FundingState::from_game(&g, &required);
        let saved = g.snapshot();
        let mut observed = false;
        world.commit_projected_entries(vec![movement], &[0], &mut |next| {
            let object = next.reference(elf);
            assert_eq!(
                next.object(elf).card,
                Card::from_name("@face-down/manifest").unwrap()
            );
            assert_eq!(
                next.projected_face_down_physical(object).unwrap().name(),
                "Llanowar Elves"
            );
            let mut different = next.clone();
            different
                .entry_projection
                .as_mut()
                .unwrap()
                .manifested
                .clear();
            assert!(next.key(&required) != different.key(&required));
            observed = true;
            true
        });
        assert!(observed);
        assert_eq!(g.snapshot(), saved);
    }
}

#[test]
fn projected_type_transformation_funds_new_basic_type_without_live_fallback() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Island", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let source = g.object_ref(land);
        let required = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        let mut world = FundingState::from_game(&g, &required);
        world.prepare_entry_projection();
        let roles = &mut world.entry_projection.as_mut().unwrap().roles;
        for (timestamp, food_only, land_type) in [(100, true, false), (101, false, true)] {
            roles.animations.push(AnimationView {
                object: source,
                timestamp,
                artifact: false,
                land: land_type,
                preserve_card_types: false,
                food_only,
                artifact_noncreature: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: land_type.then_some((1, 1)),
                keywords: Keywords::NONE,
                expires_turn: None,
            });
        }
        roles
            .basic_land_types
            .push(subtypes::BasicLandTypesEffectView {
                recipient: source,
                source,
                kind: crate::cards::BasicLandType::Forest,
                timestamp: 102,
            });
        assert_eq!(world.intrinsic_mana_color(land), Some(4));
        assert!(
            world
                .moves(&required)
                .iter()
                .any(|step| world.resources[step.resource].id == land
                    && step.production == [0, 0, 0, 0, 1, 0])
        );
        assert_eq!(g.intrinsic_mana_color(land), Some(1));
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn prohibition_provider_is_tracked_in_funding_world_and_key() {
    use crate::engine::ability_world::TEST_SACRIFICE_PROHIBITOR;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_SACRIFICE_PROHIBITOR.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let provider = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let treasure = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[treasure as usize].card = Card::test_index(5437);
        g.state.objects[treasure as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        TEST_SACRIFICE_PROHIBITOR.with(|row| row.set(Some(g.object_ref(provider))));
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        assert!(world.resources.iter().any(|r| r.id == provider));
        assert!(
            !world
                .moves(&goal)
                .iter()
                .any(|m| world.resources[m.resource].id == treasure)
        );
        let mut departed = world.clone();
        let source = departed
            .resources
            .iter_mut()
            .find(|r| r.id == provider)
            .unwrap();
        source.object.zone = Zone::Graveyard;
        assert!(world.key(&goal) != departed.key(&goal));
        assert!(
            departed
                .moves(&goal)
                .iter()
                .any(|m| departed.resources[m.resource].id == treasure)
        );
        TEST_SACRIFICE_PROHIBITOR.with(|row| row.set(None));
    }
}

#[test]
fn prevented_sacrifice_mana_projection_matches_owned_payment() {
    use crate::engine::zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let id = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[id as usize].card = Card::test_index(5437);
        g.state.objects[id as usize].summoning_sick = false;
        g.state.players[player.index()].mana = [0; 6];
        let original = g.object_ref(id);
        g.state
            .roles
            .get_or_insert_default()
            .source_linked_effects
            .push(SourceLinkedEffectView {
                source: original,
                recipient: original,
                controller: player,
                kind: SourceLinkedEffectKind::PreventDamageBy,
                timestamp: 1,
            });
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(original)));
        let goal = FundingGoal {
            actor: player,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|step| {
                world.resources[step.resource].id == id && step.production == [0, 0, 0, 0, 1, 0]
            })
            .unwrap();
        let mut projected = None;
        assert!(world.visit_applied(&step, &mut |next| {
            projected = Some(next.clone());
            true
        }));
        let projected = projected.unwrap();
        assert_eq!(projected.reference(id), original);
        assert_eq!(projected.object(id).zone, Zone::Battlefield);
        assert!(projected.object(id).tapped);
        assert_eq!(projected.mana[4], 1);
        assert!(
            projected
                .roles()
                .unwrap()
                .source_linked_effects
                .iter()
                .any(|row| row.source == original)
        );
        assert!(world.key(&goal) != projected.key(&goal));
        let projected_values = (
            projected.reference(id),
            projected.object(id).zone,
            projected.object(id).tapped,
            projected.mana[4],
        );
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a,
            Action::ActivateMana { card, production: [0, 0, 0, 0, 1, 0], .. } if *card == id)
            })
            .unwrap();
        owned_step(&mut g, action);
        assert_eq!(g.object_ref(id), projected_values.0);
        assert_eq!(g.state.objects[id as usize].zone, projected_values.1);
        assert_eq!(g.state.objects[id as usize].tapped, projected_values.2);
        assert!(
            g.state
                .roles
                .as_ref()
                .unwrap()
                .source_linked_effects
                .iter()
                .any(|row| row.source == original)
        );
        assert_eq!(
            u128::from(g.state.players[player.index()].mana[4]),
            projected_values.3
        );
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn prevention_provider_uses_live_control_and_ability_layer_after_restore() {
    use crate::engine::zone_replacements::TEST_PREVENT_PROVIDER;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_PROVIDER.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let provider = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, player, "Forest", Zone::Battlefield);
        g.state.static_sources.push(provider);
        g.state.zone_rules.get_or_insert_default();
        let source = g.object_ref(provider);
        let target = g.object_ref(land);
        TEST_PREVENT_PROVIDER.with(|p| p.set(Some(source)));
        assert!(g.zone_move_prevented(target, Zone::Graveyard));
        let saved = g.snapshot();
        let mut blank = g.activation_item(player, land, 0);
        blank.targets = vec![BoundTarget {
            group: 0,
            target: Target::Card(provider),
            object: Some(source),
        }];
        g.remove_target_abilities_until_end_of_turn(&blank);
        assert!(!g.zone_move_prevented(target, Zone::Graveyard));
        g.restore(&saved).unwrap();
        assert!(g.zone_move_prevented(target, Zone::Graveyard));
        g.change_controller(source, player.other());
        assert!(!g.zone_move_prevented(target, Zone::Graveyard));
        TEST_PREVENT_PROVIDER.with(|p| p.set(None));
    }
}

#[test]
fn projected_competing_prevention_and_exile_are_distinct_funding_branches() {
    use crate::engine::zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        named(&mut g, player, "Rest in Peace", Zone::Battlefield);
        let id = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[id as usize].card = Card::test_index(5437);
        g.state.objects[id as usize].summoning_sick = false;
        g.state.players[player.index()].mana = [0; 6];
        let reference = g.object_ref(id);
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(reference)));
        let goal = FundingGoal {
            actor: player,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|step| {
                world.resources[step.resource].id == id && step.production == [0, 0, 0, 0, 1, 0]
            })
            .unwrap();
        let mut branches = Vec::new();
        world.visit_applied(&step, &mut |next| {
            branches.push(next.clone());
            false
        });
        let prevented = branches
            .iter()
            .find(|w| w.object(id).zone == Zone::Battlefield)
            .unwrap();
        let exiled = branches
            .iter()
            .find(|w| w.object(id).zone == Zone::Exile)
            .unwrap();
        assert_eq!(prevented.reference(id), reference);
        assert_ne!(exiled.reference(id), reference);
        assert_eq!(prevented.mana[4], 1);
        assert_eq!(exiled.mana[4], 1);
        assert!(prevented.key(&goal) != exiled.key(&goal));
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn prevented_nontapping_sacrifice_mana_can_fund_repeatedly() {
    use crate::engine::zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    let _fixture = crate::cards::sacrifice_mana_fixture::Guard::new();
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        named(&mut g, player, "Rest in Peace", Zone::Battlefield);
        let id = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[id as usize].card = Card::test_index(5437);
        let reference = g.object_ref(id);
        g.state.players[player.index()].mana = [0; 6];
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(reference)));
        let goal = FundingGoal {
            actor: player,
            required: [0, 0, 0, 0, 2, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(g.can_fund(goal));
        let saved = g.snapshot();
        for _ in 0..2 {
            for _ in 0..2 {
                let action = g.legal_actions().into_iter().find(|a| matches!(a,
                    Action::ActivateMana { card, production: [0, 0, 0, 0, 1, 0], .. } if *card == id)).unwrap();
                checked(&mut g, action);
                checked(
                    &mut g,
                    Action::ChooseZoneReplacement {
                        effect: ZoneReplacementKind::Prevent,
                    },
                );
            }
            assert_eq!(g.object_ref(id), reference);
            assert_eq!(g.state.players[player.index()].mana[4], 2);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
            g.restore(&saved).unwrap();
        }
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn prevented_repeatable_black_mana_cannot_fund_missing_blue() {
    use crate::engine::zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    let _fixture = crate::cards::sacrifice_mana_fixture::Guard::new();
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let id = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[id as usize].card = Card::test_index(5437);
        g.state.players[player.index()].mana = [0; 6];
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(id))));
        let goal = FundingGoal {
            actor: player,
            required: [0, 1, 0, 0, 0, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(!g.can_fund(goal));
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn simultaneous_sacrifices_freeze_animated_type_before_provider_leaves() {
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let artifact = named(&mut g, player, "Gilded Lotus", Zone::Battlefield);
        let aura = named(&mut g, player, "Puppet Crafting", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(artifact));
        assert!(g.is_creature(artifact));
        let refs = [g.object_ref(aura), g.object_ref(artifact)];
        let goal = FundingGoal {
            actor: player,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        assert!(world.resources.iter().any(|r| r.id == aura));
        assert!(world.resources.iter().any(|r| r.id == artifact));
        let mut projected = None;
        assert!(world.cost_departures(&refs, Zone::Graveyard, &mut |next| {
            projected = Some(next.clone());
            true
        }));
        let projected = projected.unwrap();
        assert_eq!(projected.creatures_died, world.creatures_died + 1);
        assert_eq!(
            projected.controlled_creatures_died[player.index()],
            world.controlled_creatures_died[player.index()] + 1
        );
        let projected_values = (
            projected.creatures_died,
            projected.controlled_creatures_died[player.index()],
        );
        let saved = g.snapshot();
        for _ in 0..2 {
            g.sacrifice_batch(&[(player, aura), (player, artifact)]);
            assert_eq!(g.state.creatures_died_this_turn, projected_values.0);
            assert_eq!(
                g.state.players[player.index()].controlled_creatures_died_this_turn,
                projected_values.1
            );
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn spell_only_mana_normalizes_unusable_search_keys_without_mutating_pools() {
    use crate::engine::zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    let _fixture = crate::cards::sacrifice_mana_fixture::Guard::artifact_only();
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let id = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[id as usize].card = Card::test_index(5437);
        g.state.players[player.index()].mana = [0; 6];
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(id))));
        let goal = FundingGoal {
            actor: player,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut world = FundingState::from_game(&g, &goal);
        world.set_purpose(mana_lots::Purpose::Other);
        for amount in 1..=4 {
            let old_key = world.key(&goal);
            let step = world
                .moves(&goal)
                .into_iter()
                .find(|step| world.resources[step.resource].id == id)
                .unwrap();
            let mut next = None;
            assert!(world.visit_applied(&step, &mut |branch| {
                next = Some(branch.clone());
                true
            }));
            world = next.unwrap();
            assert_eq!(world.mana[4], amount);
            assert_eq!(
                mana_lots::available(world.mana, &world.lots, mana_lots::Purpose::Other)[4],
                0
            );
            if amount > 1 {
                assert!(old_key == world.key(&goal));
            }
        }
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn spell_only_repeat_mana_cannot_pay_other_or_fuel_a_converter_but_can_pay_artifact_spell() {
    use crate::engine::zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    let _fixture = crate::cards::sacrifice_mana_fixture::Guard::artifact_only();
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let id = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[id as usize].card = Card::test_index(5437);
        g.state.players[player.index()].mana = [0; 6];
        named(
            &mut g,
            player,
            "Barrels of Blasting Jelly",
            Zone::Battlefield,
        );
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(id))));
        let goal = FundingGoal {
            actor: player,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(!g.can_fund_for(goal.clone(), mana_lots::Purpose::Other));
        let artifact = mana_lots::Source::in_world(&g, id);
        assert!(g.can_fund_for(goal, mana_lots::Purpose::Spell(artifact)));
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn repeatable_artifact_only_mana_is_inert_for_a_fixed_nonartifact_spell_goal() {
    use crate::engine::zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    let _fixture = crate::cards::sacrifice_mana_fixture::Guard::artifact_only();
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let source = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[source as usize].card = Card::test_index(5437);
        let converter = named(
            &mut g,
            player,
            "Barrels of Blasting Jelly",
            Zone::Battlefield,
        );
        let creature = named(&mut g, player, "Bear Cub", Zone::Hand);
        let artifact = named(&mut g, player, "Gilded Lotus", Zone::Hand);
        g.state.players[player.index()].mana = [0; 6];
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(source))));
        let goal = FundingGoal {
            actor: player,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let creature_purpose = mana_lots::Purpose::Spell(mana_lots::Source::in_world(&g, creature));
        let artifact_purpose = mana_lots::Purpose::Spell(mana_lots::Source::in_world(&g, artifact));
        let mut world = FundingState::from_game(&g, &goal);
        world.set_purpose(creature_purpose);
        for amount in 1..=3 {
            let old = world.key(&goal);
            let step = world
                .moves(&goal)
                .into_iter()
                .find(|step| world.resources[step.resource].id == source)
                .unwrap();
            let mut next = None;
            assert!(world.visit_applied(&step, &mut |branch| {
                next = Some(branch.clone());
                true
            }));
            world = next.unwrap();
            assert_eq!(world.mana[4], amount);
            assert_eq!(
                mana_lots::available(world.mana, &world.lots, creature_purpose)[4],
                0
            );
            assert_eq!(
                mana_lots::available(world.mana, &world.lots, artifact_purpose)[4],
                amount
            );
            assert!(
                world
                    .moves(&goal)
                    .iter()
                    .all(|step| world.resources[step.resource].id != converter)
            );
            if amount > 1 {
                assert!(old == world.key(&goal));
            }
        }
        assert!(!g.can_fund_for(goal.clone(), creature_purpose));
        assert!(g.can_fund_for(goal, artifact_purpose));
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn eligible_restricted_mana_does_not_enable_converter_or_missing_color() {
    use crate::engine::zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    let _fixture = crate::cards::sacrifice_mana_fixture::Guard::artifact_only();
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let source = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[source as usize].card = Card::test_index(5437);
        let converter = named(
            &mut g,
            player,
            "Barrels of Blasting Jelly",
            Zone::Battlefield,
        );
        let artifact = named(&mut g, player, "Gilded Lotus", Zone::Hand);
        g.state.players[player.index()].mana = [0; 6];
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(source))));
        let goal = FundingGoal {
            actor: player,
            required: [0, 1, 0, 0, 0, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let purpose = mana_lots::Purpose::Spell(mana_lots::Source::in_world(&g, artifact));
        let mut world = FundingState::from_game(&g, &goal);
        world.set_purpose(purpose);
        for amount in 1..=4 {
            let old = world.key(&goal);
            let step = world
                .moves(&goal)
                .into_iter()
                .find(|step| world.resources[step.resource].id == source)
                .unwrap();
            let mut next = None;
            assert!(world.visit_applied(&step, &mut |branch| {
                next = Some(branch.clone());
                true
            }));
            world = next.unwrap();
            assert_eq!(
                mana_lots::available(world.mana, &world.lots, purpose)[4],
                amount
            );
            assert_eq!(
                mana_lots::available(world.mana, &world.lots, mana_lots::Purpose::Other)[4],
                0
            );
            assert!(!payable(
                mana_lots::available(world.mana, &world.lots, purpose),
                goal.required,
                goal.generic
            ));
            assert!(
                world
                    .moves(&goal)
                    .iter()
                    .all(|step| world.resources[step.resource].id != converter)
            );
            assert!(world.optimistic(&goal));
            if amount > 1 {
                assert!(old == world.key(&goal));
            }
        }
        assert!(!g.can_fund_for(goal, purpose));
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn ordinary_mana_can_fuel_blue_converter_alongside_artifact_only_mana() {
    use crate::engine::zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    let _fixture = crate::cards::sacrifice_mana_fixture::Guard::artifact_only();
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let source = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[source as usize].card = Card::test_index(5437);
        let converter = named(
            &mut g,
            player,
            "Barrels of Blasting Jelly",
            Zone::Battlefield,
        );
        let forest = named(&mut g, player, "Forest", Zone::Battlefield);
        let artifact = named(&mut g, player, "Gilded Lotus", Zone::Hand);
        g.state.players[player.index()].mana = [0; 6];
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(source))));
        let goal = FundingGoal {
            actor: player,
            required: [0, 1, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let purpose = mana_lots::Purpose::Spell(mana_lots::Source::in_world(&g, artifact));
        let mut world = FundingState::from_game(&g, &goal);
        world.set_purpose(purpose);
        for (id, expected) in [
            (source, [0, 0, 0, 0, 1, 0]),
            (forest, [0, 0, 0, 0, 1, 0]),
            (converter, [0, 1, 0, 0, 0, 0]),
        ] {
            let step = world
                .moves(&goal)
                .into_iter()
                .find(|m| world.resources[m.resource].id == id && m.production == expected)
                .unwrap();
            let mut next = None;
            assert!(world.visit_applied(&step, &mut |branch| {
                next = Some(branch.clone());
                true
            }));
            world = next.unwrap();
        }
        assert!(payable(
            mana_lots::available(world.mana, &world.lots, purpose),
            goal.required,
            goal.generic
        ));
        assert_eq!(mana_lots::available(world.mana, &world.lots, purpose)[4], 1);
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn standalone_spell_only_key_preserves_qualities_substitution_and_exact_pool() {
    use crate::cards::ManaRetention;
    use mana_lots::{Lot, ManaRestrictionView, ManaSpellBonusView, Quality};
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let artifact = named(&mut g, player, "Gilded Lotus", Zone::Hand);
        let goal = FundingGoal {
            actor: player,
            required: [0, 1, 0, 0, 0, 0],
            generic: 1,
            reserve: None,
            source: None,
        };
        let mut facts = mana_lots::Source::in_world(&g, artifact);
        facts.spend_any_mana = true;
        let purpose = mana_lots::Purpose::Spell(facts);
        let mut world = FundingState::from_game(&g, &goal);
        world.set_purpose(purpose);
        world.mana = [0, 0, 0, 0, 8, 0];
        let plain = Quality {
            restriction: ManaRestrictionView::ArtifactSpell,
            spell_bonus: ManaSpellBonusView::None,
            retention: ManaRetention::Step,
            copy_source: None,
            treasure_source: false,
        };
        let bonus = Quality {
            spell_bonus: ManaSpellBonusView::CopyRedInstantSorcery,
            ..plain
        };
        world.lots = vec![
            Lot {
                quality: plain,
                amounts: [0, 0, 0, 0, 4, 0],
            },
            Lot {
                quality: bonus,
                amounts: [0, 0, 0, 0, 4, 0],
            },
        ];
        let key = world.key(&goal);
        assert_eq!(key.mana[4], 4);
        assert_eq!(key.lots.len(), 2);
        assert_eq!(key.lots[0].quality, plain);
        assert_eq!(key.lots[1].quality, bonus);
        assert_eq!(key.lots[0].amounts[4], 2);
        assert_eq!(key.lots[1].amounts[4], 2);
        assert_eq!(world.mana[4], 8);
        assert_eq!(world.lots[0].amounts[4], 4);
        assert!(world.reachable(&goal, &mut HashSet::new()));
        let adjusted = cast_origins::payment_goal(&world, &goal, purpose);
        assert_eq!(adjusted.required, [0; 6]);
        assert_eq!(adjusted.generic, 2);
        let mut witnessed = false;
        funding_payments(
            mana_lots::available(world.mana, &world.lots, purpose),
            adjusted.required,
            adjusted.generic,
            |payment| {
                if payment[4] == 2 {
                    witnessed = true;
                    assert_eq!(world.mana[4] - u128::from(payment[4]), 6);
                }
            },
        );
        assert!(witnessed);
        facts.spend_any_mana = false;
        world.set_purpose(mana_lots::Purpose::Spell(facts));
        assert!(!world.reachable(&goal, &mut HashSet::new()));
    }
}

#[test]
fn projected_control_return_can_unlock_linked_land_mana() {
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        g.state.players[player.index()].mana = [0; 6];
        let land = named(&mut g, player, "Forest", Zone::Battlefield);
        let controller = named(&mut g, player, "Bear Cub", Zone::Battlefield);
        let ring = named(&mut g, player, "Lotus Ring", Zone::Battlefield);
        let exile = named(&mut g, player, "Shock", Zone::Exile);
        let land_ref = g.object_ref(land);
        let controller_ref = g.object_ref(controller);
        let ring_ref = g.object_ref(ring);
        g.state.objects[controller as usize].summoning_sick = false;
        g.state.objects[ring as usize].attached_to = Some(controller_ref);
        g.ensure_control_base(land_ref);
        let control_time = g.next_role_timestamp();
        let grant_time = g.next_role_timestamp();
        let source = g.object_ref(exile);
        let rows = &mut g.state.roles.get_or_insert_default().source_linked_effects;
        rows.push(SourceLinkedEffectView {
            source: controller_ref,
            recipient: land_ref,
            controller: player.other(),
            kind: SourceLinkedEffectKind::Control,
            timestamp: control_time,
        });
        rows.push(SourceLinkedEffectView {
            source,
            recipient: land_ref,
            controller: player,
            kind: SourceLinkedEffectKind::GrantTwoColorlessUntilSourceCast,
            timestamp: grant_time,
        });
        g.refresh_controllers();
        assert_eq!(g.object_view(land).controller, player.other());
        let goal = FundingGoal {
            actor: player,
            required: [0; 6],
            generic: 5,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        assert!(
            world.total_mana_upper() < 5,
            "current controller inventory misses the land"
        );
        assert!(
            world.optimistic(&goal),
            "dynamic control must bypass that bound"
        );
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|step| {
                world.resources[step.resource].id == controller && step.grantor == Some(ring_ref)
            })
            .expect("Lotus Ring grants sacrifice mana");
        let mut projected = None;
        assert!(world.visit_applied(&step, &mut |next| {
            projected = Some(next.clone());
            true
        }));
        let projected = projected.unwrap();
        assert_eq!(projected.object(land).controller, player);
        assert!(
            projected.moves(&goal).iter().any(
                |step| projected.resources[step.resource].id == land && step.production[5] == 2
            )
        );
        assert!(projected.reachable(&goal, &mut HashSet::new()));
    }
}

#[test]
fn stable_control_records_keep_large_impossible_funding_search_bounded() {
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        g.state.players[player.index()].mana = [0; 6];
        let lands: Vec<_> = (0..18)
            .map(|_| named(&mut g, player, "Forest", Zone::Battlefield))
            .collect();
        let reference = g.object_ref(lands[0]);
        g.ensure_control_base(reference);
        g.record_control(reference, player, None);
        assert!(
            g.state
                .controls
                .as_ref()
                .is_some_and(|state| !state.effects.is_empty())
        );
        let goal = FundingGoal {
            actor: player,
            required: [0; 6],
            generic: 1000,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&g, &goal);
        assert!(world.total_mana_upper() < 1000);
        assert!(
            !world.optimistic(&goal),
            "stable control must retain the proven upper bound"
        );
        assert!(!world.reachable(&goal, &mut HashSet::new()));
    }
}

#[test]
fn lute_spell_mana_does_not_fund_emrakul_hand_activation() {
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let lute = named(&mut g, player, "Resonating Lute", Zone::Battlefield);
        let source = named(&mut g, player, "Bear Cub", Zone::Hand);
        let emrakul = Card::test_index(1324);
        g.state.objects[source as usize].card = emrakul;
        let names = [
            "Escape Tunnel",
            "Hobbit Hole",
            "Elven Passage",
            "Vibrant Cityscape",
            "Evolving Wilds",
            "Terramorphic Expanse",
            "Fabled Passage",
        ];
        let mut fetchlands = Vec::new();
        for name in names {
            for _ in 0..4 {
                fetchlands.push(named(&mut g, player, name, Zone::Battlefield));
            }
        }
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
        let goal = FundingGoal {
            actor: player,
            required: [0; 6],
            generic: 3,
            reserve: None,
            source: Some((g.object_ref(source), 0, Counters::default(), emrakul, None)),
        };
        let world = FundingState::from_game(&g, &goal);
        assert!(world.fixed_spell_only_activation_cannot_pay(&goal));
        let before = g.snapshot();
        assert!(!g.can_fund(goal.clone()));
        assert_eq!(g.snapshot(), before);
        let land = fetchlands[0];
        let grantor = g.object_ref(lute);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: land,
                grantor,
                production: [0, 2, 0, 0, 0, 0],
            },
        );
        let with_restricted = g.snapshot();
        assert!(!g.can_fund(goal.clone()));
        assert_eq!(g.snapshot(), with_restricted);
        g.restore(&before).unwrap();
        assert!(!g.can_fund(goal.clone()));
        let forest = named(&mut g, player, "Forest", Zone::Battlefield);
        let world = FundingState::from_game(&g, &goal);
        assert!(!world.fixed_spell_only_activation_cannot_pay(&goal));
        assert!(g.can_fund(goal));
        assert!(!g.object_view(forest).tapped);
    }
}
