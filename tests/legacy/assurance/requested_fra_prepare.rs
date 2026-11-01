//! Prepared inset spells and their associated creatures.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

fn prepared(g: &mut Game, p: Player, name: &str) -> (CardId, ObjectRef) {
    let creature = put(g, p, name, Zone::Hand);
    cast(g, creature, None, &[]);
    settle(g);
    let copy = g
        .state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.permanent == g.object_ref(creature))
        .unwrap()
        .spell_copy;
    (creature, copy)
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("missing prepared choice {:?}", g.state.pending);
}
fn cast_prepared(g: &mut Game, copy: ObjectRef) {
    checked(g, Action::AnnounceExile { card: copy.card });
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, pay);
}
#[test]
fn cultivator_enters_prepared_and_enroot_searches_to_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (creature, copy) = prepared(&mut g, p, "Carnivorous Cultivator // Enroot");
        assert!(g.keywords(creature).contains(Keywords::DEATHTOUCH));
        let land = put(&mut g, p, "Island", Zone::Library);
        let nonland = put(&mut g, p, "Shock", Zone::Library);
        cast_prepared(&mut g, copy);
        until(&mut g, |g| g.observe(p).search_choice.is_some());
        assert!(g.observe(p.other()).search_choice.is_none());
        reject(&mut g, Action::ChooseSearch { card: nonland });
        checked(&mut g, Action::ChooseSearch { card: land });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        assert!(!g.is_prepared(g.object_ref(creature)));
        assert!(!g.on_battlefield(copy.card));
    }
}
#[test]
fn battlemage_prepared_spell_discards_at_most_one_and_draws_only_if_discarded() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = setup(p);
            let (creature, copy) = prepared(&mut g, p, "Pompous Battlemage // Improvised Act");
            let hand = put(&mut g, p, "Island", Zone::Hand);
            let drawn = put(&mut g, p, "Mountain", Zone::Library);
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            cast_prepared(&mut g, copy);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            assert!(g.observe(p.other()).effect_choice.is_none());
            if discard {
                checked(&mut g, Action::Discard { card: hand });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn - before,
                u64::from(discard)
            );
            assert_eq!(
                g.object_view(hand).zone,
                if discard { Zone::Graveyard } else { Zone::Hand }
            );
            assert_eq!(
                g.object_view(drawn).zone,
                if discard { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(
                g.stats(creature).power,
                2,
                "casting prepared noncreature triggers prowess"
            );
            assert!(!g.is_prepared(g.object_ref(creature)));
        }
    }
}
#[test]
fn leaving_battlefield_ceases_prepared_copy() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Carnivorous Cultivator // Enroot",
            "Pompous Battlemage // Improvised Act",
        ] {
            let mut g = setup(p);
            let (creature, copy) = prepared(&mut g, p, name);
            g.move_object(creature, Zone::Graveyard);
            assert!(
                !g.legal_actions()
                    .contains(&Action::AnnounceExile { card: copy.card })
            );
        }
    }
}

#[test]
fn unassociated_prepared_copy_remains_until_state_based_actions() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [false, true] {
            let mut g = setup(p);
            let (creature, copy) = prepared(&mut g, p, "Carnivorous Cultivator // Enroot");
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                let permanent = g.object_ref(creature);
                // Exercise the two mutations used inside resolving effects, before
                // their enclosing action gets to run state-based actions.
                if source_leaves {
                    g.move_object(creature, Zone::Graveyard);
                } else {
                    g.forget_prepared(permanent);
                }
                assert!(!g.is_prepared(permanent));
                assert!(!g.can_cast_prepared(p, copy.card));
                assert_eq!(g.object_ref(copy.card), copy);
                assert_eq!(g.object_view(copy.card).zone, Zone::Exile);
                let new_copy = if source_leaves {
                    None
                } else {
                    g.prepare_permanent(permanent);
                    Some(g.state.preparations.as_ref().unwrap()[0].spell_copy)
                };
                g.state_based_actions();
                assert_eq!(g.object_view(copy.card).zone, Zone::Outside);
                assert_ne!(g.object_ref(copy.card), copy);
                if let Some(new_copy) = new_copy {
                    assert_ne!(new_copy, copy);
                    assert_eq!(g.object_ref(new_copy.card), new_copy);
                    assert_eq!(g.object_view(new_copy.card).zone, Zone::Exile);
                    assert!(g.can_cast_prepared(p, new_copy.card));
                }
            }
        }
    }
}

#[test]
fn cultivator_combat_damage_returns_own_land_and_trigger_survives_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (creature, _) = prepared(&mut g, p, "Carnivorous Cultivator // Enroot");
        let land = put(&mut g, p, "Island", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Forest", Zone::Graveyard);
        let source = g.damage_source(creature);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Player(p.other()),
            amount: 2,
            combat: true,
            preventable: true,
        }]);
        g.prepare_triggers();
        g.move_object(creature, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Hand);
    }
}

#[test]
fn improvised_act_draws_when_discard_is_replaced_by_exile() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (_, copy) = prepared(&mut g, p, "Pompous Battlemage // Improvised Act");
        put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        let discarded = put(&mut g, p, "Island", Zone::Hand);
        let drawn = put(&mut g, p, "Mountain", Zone::Library);
        cast_prepared(&mut g, copy);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        checked(&mut g, Action::Discard { card: discarded });
        settle(&mut g);
        assert_eq!(g.object_view(discarded).zone, Zone::Exile);
        assert_eq!(g.object_view(drawn).zone, Zone::Hand);
    }
}

#[test]
fn prepared_creature_names_and_mana_values_use_physical_fronts() {
    for (identity, name, mana, value) in [
        (
            "Carnivorous Cultivator // Enroot",
            "Carnivorous Cultivator",
            "{1}{G}",
            2,
        ),
        (
            "Pompous Battlemage // Improvised Act",
            "Pompous Battlemage",
            "{R}",
            1,
        ),
    ] {
        let card = Card::from_name(identity).unwrap();
        assert_eq!(Card::from_name(name), Some(card));
        assert_eq!(card.name(), name);
        assert_eq!(card.info().mana_cost, mana);
        assert_eq!(card.fixed_mana_value(), value);
    }
}

#[test]
fn improvised_act_with_empty_hand_cannot_discard_or_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (_, copy) = prepared(&mut g, p, "Pompous Battlemage // Improvised Act");
        let drawn = put(&mut g, p, "Mountain", Zone::Library);
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        cast_prepared(&mut g, copy);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::Discard { .. }))
        );
        checked(&mut g, Action::DeclineEffectPayment);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before);
        assert_eq!(g.object_view(drawn).zone, Zone::Library);
    }
}

#[test]
fn cultivator_trigger_does_not_return_land_that_left_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (creature, _) = prepared(&mut g, p, "Carnivorous Cultivator // Enroot");
        let land = put(&mut g, p, "Island", Zone::Graveyard);
        let source = g.damage_source(creature);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Player(p.other()),
            amount: 2,
            combat: true,
            preventable: true,
        }]);
        g.prepare_triggers();
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        g.move_object(land, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Exile);
    }
}
