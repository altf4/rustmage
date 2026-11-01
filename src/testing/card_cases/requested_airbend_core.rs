//! Focused printed outcomes; shared engine invariants live in core tests.
//! Independent airbend permissions, costs, and exact object lifetimes.
use super::big_helpers::checked;
use super::*;

fn airbend_action(card: CardId) -> Action {
    Action::AnnounceAirbend {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}

#[test]
fn avatars_wrath_without_target_exiles_all_creatures_but_a_countered_wrath_does_nothing() {
    {
        let player = Player::P0;
        let mut game = super::big_helpers::setup(player);
        let wrath = put(&mut game, player, "Avatar's Wrath", Zone::Hand);
        let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        super::big_helpers::cast(&mut game, wrath, None, &[]);
        super::big_helpers::settle(&mut game);
        assert!(game.has_airbend_permission(player.other(), creature));
        assert_eq!(game.object_view(wrath).zone, Zone::Exile);

        let mut game = super::big_helpers::setup(player);
        let wrath = put(&mut game, player, "Avatar's Wrath", Zone::Hand);
        let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        super::big_helpers::cast(&mut game, wrath, None, &[Target::Card(creature)]);
        game.move_object(creature, Zone::Hand);
        super::big_helpers::settle(&mut game);
        assert_eq!(game.object_view(wrath).zone, Zone::Graveyard);
        assert!(!game.nonhand_cast_prohibited(player.other()));
    }
}

#[test]
fn airbend_owner_pays_two_and_permission_expires_on_zone_change() {
    for owner in [Player::P0, Player::P1] {
        let mut g = scenario(owner);
        let card = put(&mut g, owner, "Llanowar Elves", Zone::Battlefield);
        // Another player controls the permanent, but its owner receives the right.
        g.state.objects[card as usize].controller = owner.other();
        let old = g.object_ref(card);
        g.airbend_objects(&[card]);
        assert_eq!(g.object_view(card).zone, Zone::Exile);
        assert_ne!(g.object_ref(card), old);
        assert!(g.has_airbend_permission(owner, card));
        assert!(!g.has_airbend_permission(owner.other(), card));
        assert!(!g.can_play_exiled(owner, card));
        assert_eq!(
            g.observe(owner).exile_permissions,
            g.observe(owner.other()).exile_permissions
        );
        g.state.players[owner.index()].mana = [0; 6];
        reject(&mut g, airbend_action(card));
        g.state.players[owner.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, airbend_action(card));
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .cast
                .unwrap()
                .method,
            CastMethod::Airbend
        );
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, action);
        assert_eq!(g.state.players[owner.index()].mana, [0; 6]);
        assert!(!g.has_airbend_permission(owner, card));
    }
}

fn await_target(g: &mut Game, target: Target) {
    for _ in 0..40 {
        if g.legal_actions().contains(&Action::ChooseTarget { target }) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. } | Action::Pass))
            .expect("waiting for trigger targets");
        checked(g, action);
    }
    panic!("target never offered");
}
fn finish_targets(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
}

#[test]
fn airbender_ascension_entry_counters_and_intervening_end_step() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let own = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let ascension = put(&mut g, p, "Airbender Ascension", Zone::Hand);
        super::big_helpers::cast(&mut g, ascension, None, &[]);
        await_target(&mut g, Target::Card(enemy));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        finish_targets(&mut g);
        super::big_helpers::settle(&mut g);
        assert!(g.has_airbend_permission(p.other(), enemy));
        assert!(!g.has_airbend_permission(p, enemy));
        for _ in 0..4 {
            let entering = put(&mut g, p, "Bear Cub", Zone::Hand);
            g.move_object(entering, Zone::Battlefield);
            super::big_helpers::settle(&mut g);
        }
        assert_eq!(g.counter_count(ascension, CounterKind::Quest), 4);
        g.begin_step(Step::End);
        await_target(&mut g, Target::Card(own));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        finish_targets(&mut g);
        let before = g.object_ref(own);
        super::big_helpers::settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        assert_ne!(g.object_ref(own), before);
        assert_eq!(g.counter_count(ascension, CounterKind::Quest), 5);
    }
}

#[test]
fn appa_multiple_targets_and_cast_from_exile_trigger() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let one = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let two = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let appa = put(&mut g, p, "Appa, Steadfast Guardian", Zone::Hand);
        super::big_helpers::cast(&mut g, appa, None, &[]);
        await_target(&mut g, Target::Card(one));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(appa),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(one),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(two),
            },
        );
        finish_targets(&mut g);
        super::big_helpers::settle(&mut g);
        assert!(g.has_airbend_permission(p, one));
        assert!(g.has_airbend_permission(p, two));
        assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
        checked(&mut g, airbend_action(one));
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        super::big_helpers::settle(&mut g);
        let allies = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(i, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.has_subtype(*i as CardId, "Ally")
            })
            .count();
        assert_eq!(allies, 1);
    }
}

#[test]
fn airbend_fixed_alternative_sets_printed_x_to_zero() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let card = put(&mut g, p, "Mind Spring", Zone::Exile);
        g.grant_airbend_permission(p, g.object_ref(card));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, airbend_action(card));
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(0));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseX { .. }))
        );
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn airbend_does_not_waive_mandatory_additional_costs() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Bitter Triumph", Zone::Exile);
        g.grant_airbend_permission(p, g.object_ref(spell));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let life = g.state.players[p.index()].life;
        checked(&mut g, airbend_action(spell));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        checked(&mut g, Action::BeginOrderedCosts);
        let additional = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::Additional)
            .unwrap() as u16;
        checked(&mut g, Action::ChooseCastCost { clause: additional });
        checked(&mut g, Action::ChooseAdditionalLife);
        let mana = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::Mana)
            .unwrap() as u16;
        checked(&mut g, Action::ChooseCastCost { clause: mana });
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        super::big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 3);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
}

fn ally_count(game: &Game, owner: Player) -> usize {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(index, object)| {
            object.zone == Zone::Battlefield
                && object.controller == owner
                && object.is_token()
                && game.has_subtype(*index as CardId, "Ally")
        })
        .count()
}

#[test]
fn appa_cast_trigger_tracks_cast_origin_controller_and_survives_countering() {
    {
        let player = Player::P0;
        for (caster, from_exile, countered) in [
            (player, true, true),
            (player, false, false),
            (player.other(), true, false),
        ] {
            let mut game = scenario(caster);
            put(
                &mut game,
                player,
                "Appa, Steadfast Guardian",
                Zone::Battlefield,
            );
            let shock = put(
                &mut game,
                caster,
                "Shock",
                if from_exile { Zone::Exile } else { Zone::Hand },
            );
            let counter =
                countered.then(|| put(&mut game, caster.other(), "Counterspell", Zone::Hand));
            if from_exile {
                game.grant_exile_play_permission(caster, game.object_ref(shock), false);
                checked(&mut game, Action::AnnounceExile { card: shock });
            } else {
                checked(&mut game, Action::AnnounceCast { card: shock });
            }
            checked(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Player(caster.other()),
                },
            );
            let payment = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut game, payment);
            if let Some(counter) = counter {
                game.prepare_triggers();
                while game.actor() != Some(caster.other()) {
                    checked(&mut game, Action::Pass);
                }
                super::big_helpers::cast(&mut game, counter, None, &[Target::Spell(shock)]);
            }
            super::big_helpers::settle(&mut game);
            assert_eq!(
                ally_count(&game, player),
                usize::from(caster == player && from_exile)
            );
            if countered {
                assert_eq!(game.object_view(shock).zone, Zone::Graveyard);
            }
            if caster == player && from_exile {
                let ally = game
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .find_map(|(id, object)| {
                        (object.zone == Zone::Battlefield
                            && object.controller == player
                            && object.is_token()
                            && game.has_subtype(id as CardId, "Ally"))
                        .then_some(id as CardId)
                    })
                    .unwrap();
                assert_eq!(stats(&game, ally), (1, 1));
                assert_eq!(game.state.objects[ally as usize].card.colors(), 1);
            }
        }
    }
}
