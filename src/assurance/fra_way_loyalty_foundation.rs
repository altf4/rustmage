//! Loyalty use belongs to a permanent, not the currently copied definition.
use super::big_helpers::{checked, settle, setup};
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};

#[test]
fn loyalty_use_survives_copy_effect_and_resets_for_new_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.add_counter(walker, CounterKind::Loyalty, 4);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: walker,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        settle(&mut g);
        let reference = g.object_ref(walker);
        g.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::JacePlaneswalkerToken),
            CopyDeadline::EndOfTurn,
        );
        let again = Action::AnnounceActivate {
            card: walker,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(!g.legal_actions().contains(&again));
        reject(&mut g, again);
        g.state.turn += 1;
        g.begin_turn();
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        let later = Action::AnnounceActivate {
            card: walker,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(g.legal_actions().contains(&later));
    }
}

#[test]
fn healer_grant_tracks_controllers_and_survives_provider_departure_on_stack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let way = put(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[way as usize].card = Card::from_name("Way of the Healer").unwrap();
        super::big_helpers::cast(&mut g, way, None, &[]);
        settle(&mut g);
        let walker = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.has_subtype(i as CardId, "Jace"))
                .then_some(i as CardId)
            })
            .unwrap();
        let source = g.object_ref(walker);
        let grantor = g.object_ref(way);
        let granted = Action::AnnouncePlaneswalkerGrant {
            source,
            grantor,
            ability: 0,
        };
        assert!(g.legal_actions().contains(&granted));
        checked(&mut g, granted.clone());
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 3);
        assert!(g.object_view(walker).loyalty_activated_this_turn);
        assert_eq!(g.object_view(walker).used_abilities, 0);
        g.move_object(way, Zone::Graveyard);
        settle(&mut g);
        assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
            && o.controller == p
            && o.card == Card::test_index(5298)));
        assert!(!g.legal_actions().contains(&granted));
        let printed = Action::AnnounceActivate {
            card: walker,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, printed);
    }
}

#[test]
fn healer_grant_follows_live_controllers_and_new_planeswalkers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let way = put(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[way as usize].card = Card::from_name("Way of the Healer").unwrap();
        super::big_helpers::cast(&mut g, way, None, &[]);
        settle(&mut g);
        let own = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        let foreign = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.add_counter(own, CounterKind::Loyalty, 4);
        g.add_counter(foreign, CounterKind::Loyalty, 4);
        let grantor = g.object_ref(way);
        let own_action = Action::AnnouncePlaneswalkerGrant {
            source: g.object_ref(own),
            grantor,
            ability: 0,
        };
        let foreign_action = Action::AnnouncePlaneswalkerGrant {
            source: g.object_ref(foreign),
            grantor,
            ability: 0,
        };
        assert!(g.legal_actions().contains(&own_action));
        assert!(!g.legal_actions().contains(&foreign_action));
        g.change_controller(grantor, p.other());
        assert!(!g.legal_actions().contains(&own_action));
        g.state.active = p.other();
        g.state.priority = p.other();
        assert!(g.legal_actions().contains(&foreign_action));
    }
}

#[test]
fn healer_grant_respects_later_ability_removal_and_returns_after_it_leaves() {
    for p in [Player::P0, Player::P1] {
        for blank_provider in [false, true] {
            let mut g = setup(p);
            let way = put(&mut g, p, "Forest", Zone::Hand);
            g.state.objects[way as usize].card = Card::from_name("Way of the Healer").unwrap();
            super::big_helpers::cast(&mut g, way, None, &[]);
            settle(&mut g);
            let walker = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && g.has_subtype(i as CardId, "Jace"))
                    .then_some(i as CardId)
                })
                .unwrap();
            let source = g.object_ref(walker);
            let grantor = g.object_ref(way);
            let granted = Action::AnnouncePlaneswalkerGrant {
                source,
                grantor,
                ability: 0,
            };
            assert!(g.legal_actions().contains(&granted));
            let blanked = if blank_provider { grantor } else { source };
            g.change_object_form(
                blanked,
                AnimationSpec {
                    artifact: false,
                    creature_types: AnimationCreatureTypes::Unchanged,
                    base: Some((2, 2)),
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
                characteristic_layers::AnimationTypeMode::AddCreature,
            );
            let aura = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
            g.state.objects[aura as usize].attached_to = Some(blanked);
            assert!(!g.legal_actions().contains(&granted));
            g.move_object(aura, Zone::Graveyard);
            assert!(g.legal_actions().contains(&granted));
        }
    }
}

#[test]
fn printed_loyalty_locks_grant_through_theft_until_new_turn_or_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let way = put(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[way as usize].card = Card::from_name("Way of the Healer").unwrap();
        super::big_helpers::cast(&mut g, way, None, &[]);
        settle(&mut g);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.add_counter(walker, CounterKind::Loyalty, 4);
        let old = g.object_ref(walker);
        let grantor = g.object_ref(way);
        let grant = Action::AnnouncePlaneswalkerGrant {
            source: old,
            grantor,
            ability: 0,
        };
        assert!(g.legal_actions().contains(&grant));
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: walker,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        settle(&mut g);
        for viewer in [p, p.other()] {
            assert!(
                g.observe(viewer)
                    .objects
                    .iter()
                    .any(|o| o.reference == old && o.loyalty_activated_this_turn)
            );
        }
        reject(&mut g, grant.clone());
        g.change_controller(old, p.other());
        g.change_controller(grantor, p.other());
        g.state.active = p.other();
        g.state.priority = p.other();
        reject(&mut g, grant.clone());
        g.state.turn += 1;
        g.begin_turn();
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Main1;
        assert!(g.legal_actions().contains(&grant));
        g.move_object(walker, Zone::Exile);
        g.move_object(walker, Zone::Battlefield);
        let new = g.object_ref(walker);
        assert_ne!(old, new);
        assert!(!g.object_view(walker).loyalty_activated_this_turn);
        reject(&mut g, grant);
    }
}

#[test]
fn healer_grant_rejects_low_loyalty_and_wrong_timing_and_counter_keeps_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let way = put(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[way as usize].card = Card::from_name("Way of the Healer").unwrap();
        super::big_helpers::cast(&mut g, way, None, &[]);
        settle(&mut g);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.add_counter(walker, CounterKind::Loyalty, 1);
        let source = g.object_ref(walker);
        let grant = Action::AnnouncePlaneswalkerGrant {
            source,
            grantor: g.object_ref(way),
            ability: 0,
        };
        reject(&mut g, grant.clone());
        g.add_counter(walker, CounterKind::Loyalty, 2);
        g.state.step = Step::BeginCombat;
        reject(&mut g, grant.clone());
        g.state.step = Step::Main1;
        let before = g.snapshot();
        checked(&mut g, grant.clone());
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 1);
        let id = g.state.stack.last().unwrap().id;
        assert!(g.counter_stack_item(id, false));
        settle(&mut g);
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Battlefield && o.card == Card::test_index(5298))
        );
        assert!(g.object_view(walker).loyalty_activated_this_turn);
        reject(&mut g, grant.clone());
        g.restore(&before).unwrap();
        assert!(g.legal_actions().contains(&grant));
    }
}

#[test]
fn two_grant_providers_share_one_loyalty_activation_limit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let way = put(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[way as usize].card = Card::from_name("Way of the Healer").unwrap();
        super::big_helpers::cast(&mut g, way, None, &[]);
        settle(&mut g);
        let duplicate = put(&mut g, p, "Forest", Zone::Battlefield);
        g.state.objects[duplicate as usize].card = Card::from_name("Way of the Healer").unwrap();
        let duplicate_ref = g.object_ref(duplicate);
        let mut recipe = CopyRecipe::new(Card::from_name("Way of the Healer").unwrap());
        recipe.overrides = Some(CopyOverrides {
            legendary: Some(false),
            ..CopyOverrides::NONE
        });
        g.set_copy_recipe(duplicate_ref, recipe);
        g.register_battlefield_providers(duplicate);
        let walker = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.has_subtype(i as CardId, "Jace"))
                .then_some(i as CardId)
            })
            .unwrap();
        let source = g.object_ref(walker);
        let first = Action::AnnouncePlaneswalkerGrant {
            source,
            grantor: g.object_ref(way),
            ability: 0,
        };
        let second = Action::AnnouncePlaneswalkerGrant {
            source,
            grantor: duplicate_ref,
            ability: 0,
        };
        assert!(g.legal_actions().contains(&first));
        assert!(g.legal_actions().contains(&second));
        checked(&mut g, first);
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        settle(&mut g);
        reject(&mut g, second);
    }
}

#[test]
fn granted_loyalty_activation_notifies_existing_loyalty_observers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let paradox = put(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[paradox as usize].card = Card::from_name("Way of the Paradox").unwrap();
        super::big_helpers::cast(&mut g, paradox, None, &[]);
        settle(&mut g);
        let way = put(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[way as usize].card = Card::from_name("Way of the Healer").unwrap();
        super::big_helpers::cast(&mut g, way, None, &[]);
        settle(&mut g);
        let walker = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.has_subtype(i as CardId, "Jace"))
                .then_some(i as CardId)
            })
            .unwrap();
        let life = g.state.players[p.index()].life;
        let lands = g.state.players[p.index()].extra_land_plays_remaining;
        let source = g.object_ref(walker);
        let grantor = g.object_ref(way);
        checked(
            &mut g,
            Action::AnnouncePlaneswalkerGrant {
                source,
                grantor,
                ability: 0,
            },
        );
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
        assert_eq!(
            g.state.players[p.index()].extra_land_plays_remaining,
            lands + 1
        );
    }
}

#[test]
fn pyromancer_loyalty_production_uses_stack_and_cannot_fund_a_spell_midcast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let way = put(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[way as usize].card = Card::from_name("Way of the Pyromancer").unwrap();
        super::big_helpers::cast(&mut g, way, None, &[]);
        settle(&mut g);
        let walker = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.has_subtype(i as CardId, "Jace"))
                .then_some(i as CardId)
            })
            .unwrap();
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: shock })
        );
        let grant = Action::AnnouncePlaneswalkerGrant {
            source: g.object_ref(walker),
            grantor: g.object_ref(way),
            ability: 0,
        };
        let before = g.snapshot();
        checked(&mut g, grant.clone());
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 3);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let id = g.state.stack.last().unwrap().id;
        assert!(g.counter_stack_item(id, false));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.restore(&before).unwrap();
        checked(&mut g, grant);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: shock })
        );
    }
}

#[test]
fn loyalty_use_resets_when_token_slot_is_recycled() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.create_tokens(p, Card::JacePlaneswalkerToken, 1, false);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.card == Card::JacePlaneswalkerToken)
                    .then_some(id as CardId)
            })
            .unwrap();
        g.state.objects[token as usize].loyalty_activated_this_turn = true;
        g.move_object(token, Zone::Graveyard);
        if g.object_view(token).zone != Zone::Outside {
            g.cease_token(token);
        }
        assert!(!g.object_view(token).loyalty_activated_this_turn);
        g.create_tokens(p, Card::JacePlaneswalkerToken, 1, false);
        assert_eq!(g.object_view(token).zone, Zone::Battlefield);
        assert!(!g.object_view(token).loyalty_activated_this_turn);
    }
}
