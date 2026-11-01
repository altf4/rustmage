//! Focused printed outcomes; shared engine invariants live in core tests.
//! Loyalty use belongs to a permanent, not the currently copied definition.
use super::big_helpers::{checked, settle, setup};
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};

#[test]
fn loyalty_use_survives_copy_effect_and_resets_for_new_turn() {
    {
        let p = Player::P0;
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
fn healer_grant_follows_live_controllers_and_new_planeswalkers() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn granted_loyalty_activation_notifies_existing_loyalty_observers() {
    {
        let p = Player::P0;
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
