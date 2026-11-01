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
fn mass_airbend_excludes_only_the_exact_creature_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut game = scenario(p);
        let kept = put(&mut game, p, "Bear Cub", Zone::Battlefield);
        let yours = put(&mut game, p, "Llanowar Elves", Zone::Battlefield);
        let theirs = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
        let land = put(&mut game, p, "Forest", Zone::Battlefield);
        let except = game.object_ref(kept);
        let before = game.snapshot();
        let mut branch = game.clone();
        game.airbend_other_creatures(Some(except));
        branch.airbend_other_creatures(Some(except));
        assert_eq!(game.snapshot(), branch.snapshot());
        assert_eq!(game.object_view(kept).zone, Zone::Battlefield);
        assert_eq!(game.object_view(land).zone, Zone::Battlefield);
        assert!(game.has_airbend_permission(p, yours));
        assert!(game.has_airbend_permission(p.other(), theirs));
        game.restore(&before).unwrap();
        game.move_object(kept, Zone::Hand);
        game.move_object(kept, Zone::Battlefield);
        game.airbend_other_creatures(Some(except));
        assert!(game.has_airbend_permission(p, kept));
    }
}

#[test]
fn avatars_wrath_keeps_target_airbends_others_and_restricts_opponent_until_next_turn() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = super::big_helpers::setup(player);
        let wrath = put(&mut game, player, "Avatar's Wrath", Zone::Hand);
        let kept = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let yours = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        let theirs = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let hand = put(&mut game, opponent, "Llanowar Elves", Zone::Hand);
        let land = put(&mut game, opponent, "Forest", Zone::Hand);
        super::big_helpers::cast(&mut game, wrath, None, &[Target::Card(kept)]);
        super::big_helpers::settle(&mut game);
        assert_eq!(game.object_view(wrath).zone, Zone::Exile);
        assert_eq!(game.object_view(kept).zone, Zone::Battlefield);
        assert!(game.has_airbend_permission(player, yours));
        assert!(game.has_airbend_permission(opponent, theirs));
        assert!(game.nonhand_cast_prohibited(opponent));
        assert!(!game.nonhand_cast_prohibited(player));
        game.state.active = opponent;
        game.state.priority = opponent;
        game.state.pending = Pending::Priority;
        assert!(!game.legal_actions().contains(&airbend_action(theirs)));
        assert!(
            game.legal_actions()
                .contains(&Action::AnnounceCast { card: hand })
        );
        assert!(
            game.legal_actions()
                .contains(&Action::PlayLand { card: land })
        );
        let snapshot = game.snapshot();
        let branch = game.clone();
        game.state.active = player;
        game.begin_turn();
        assert!(!game.nonhand_cast_prohibited(opponent));
        game.state.active = opponent;
        game.state.priority = opponent;
        game.state.step = Step::Main1;
        assert!(game.legal_actions().contains(&airbend_action(theirs)));
        game.restore(&snapshot).unwrap();
        assert_eq!(game.snapshot(), branch.snapshot());
        assert!(game.nonhand_cast_prohibited(opponent));
    }
}

#[test]
fn avatars_wrath_without_target_exiles_all_creatures_but_a_countered_wrath_does_nothing() {
    for player in [Player::P0, Player::P1] {
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

#[test]
fn airbend_lifetime_and_independent_ordinary_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.airbend_objects(&[card]);
        let exiled = g.object_ref(card);
        g.grant_exile_play_permission(p, exiled, true);
        assert!(g.legal_actions().contains(&airbend_action(card)));
        assert!(g.legal_actions().contains(&Action::AnnounceExile { card }));
        g.move_object(card, Zone::Hand);
        g.move_object(card, Zone::Exile);
        assert_ne!(g.object_ref(card), exiled);
        assert!(!g.has_airbend_permission(p, card));
        reject(&mut g, airbend_action(card));
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
    for p in [Player::P0, Player::P1] {
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
    for p in [Player::P0, Player::P1] {
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
fn appa_rechecks_target_incarnations_and_keeps_surviving_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let lost = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let survives = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let appa = put(&mut g, p, "Appa, Steadfast Guardian", Zone::Hand);
        super::big_helpers::cast(&mut g, appa, None, &[]);
        await_target(&mut g, Target::Card(lost));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(lost),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(survives),
            },
        );
        finish_targets(&mut g);
        let before = g.object_ref(lost);
        g.move_object(lost, Zone::Hand);
        g.move_object(lost, Zone::Battlefield);
        assert_ne!(g.object_ref(lost), before);
        super::big_helpers::settle(&mut g);
        assert_eq!(g.object_view(lost).zone, Zone::Battlefield);
        assert!(!g.has_airbend_permission(p, lost));
        assert_eq!(g.object_view(survives).zone, Zone::Exile);
        assert!(g.has_airbend_permission(p, survives));
    }
}

#[test]
fn ascension_rechecks_counter_threshold_on_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ascension = put(&mut g, p, "Airbender Ascension", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(ascension, CounterKind::Quest, 4);
        g.begin_step(Step::End);
        await_target(&mut g, Target::Card(creature));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        finish_targets(&mut g);
        let prior = g.object_ref(creature);
        g.pay_counters(ascension, &Counters::of(CounterKind::Quest, 1));
        super::big_helpers::settle(&mut g);
        assert_eq!(g.object_ref(creature), prior);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
    }
}

#[test]
fn airbend_exiles_a_spell_from_stack_and_its_owner_can_cast_it_later() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        super::big_helpers::cast(&mut g, spell, None, &[Target::Player(p.other())]);
        let victim = g
            .state
            .stack
            .iter()
            .find(|s| s.source.is_some_and(|source| source.id == spell))
            .unwrap()
            .clone();
        let reference = g.object_ref(spell);
        let mut effect = victim.clone();
        effect.controller = p.other();
        effect.targets = vec![BoundTarget {
            group: 0,
            target: Target::Spell(spell),
            object: Some(reference),
        }];
        g.airbend_targets(&effect);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert!(!g.state.stack.iter().any(|item| item.id == victim.id));
        assert!(g.has_airbend_permission(p, spell));
        assert!(!g.has_airbend_permission(p.other(), spell));
    }
}

#[test]
fn airbend_fixed_alternative_sets_printed_x_to_zero() {
    for p in [Player::P0, Player::P1] {
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
    for p in [Player::P0, Player::P1] {
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

#[test]
fn appa_allows_zero_targets_and_preserves_other_permanents() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let permanent = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let appa = put(&mut g, p, "Appa, Steadfast Guardian", Zone::Hand);
        super::big_helpers::cast(&mut g, appa, None, &[]);
        await_target(&mut g, Target::Card(permanent));
        checked(&mut g, Action::FinishTargets);
        super::big_helpers::settle(&mut g);
        assert_eq!(g.object_view(permanent).zone, Zone::Battlefield);
        assert_eq!(g.object_view(appa).zone, Zone::Battlefield);
    }
}

#[test]
fn airbend_spell_copy_does_not_affect_original_or_grant_a_cast_right() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Shock", Zone::Hand);
        super::big_helpers::cast(&mut game, spell, None, &[Target::Player(player.other())]);
        let original = game
            .state
            .stack
            .iter()
            .find(|item| item.source.is_some_and(|source| source.id == spell))
            .unwrap()
            .clone();
        let copied = game.allocate_spell_copy(&original, player.other());
        let copy_id = copied.id;
        let copy_object = copied.source.unwrap().object;
        game.state.stack.push(copied);
        let mut effect = original.clone();
        effect.targets = vec![BoundTarget {
            group: 0,
            target: Target::Stack(copy_id),
            object: None,
        }];
        game.airbend_targets(&effect);
        assert!(game.state.stack.iter().any(|item| item.id == original.id));
        assert!(!game.state.stack.iter().any(|item| item.id == copy_id));
        assert_eq!(game.object_view(spell).zone, Zone::Stack);
        assert!(
            !game
                .state
                .exile_permissions
                .iter()
                .any(|row| row.object.card == copy_object.card)
        );
        assert!(!game.has_airbend_permission(player, spell));
        assert!(!game.has_airbend_permission(player.other(), spell));
    }
}

#[test]
fn airbend_competing_prevention_and_redirect_replays_pending_choice() {
    use zone_replacements::{TEST_PREVENT_ZONE_MOVE, TEST_REDIRECT_EXILE_TO_BATTLEFIELD};
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
            TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|v| v.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let appa = put(&mut game, player, "Appa, Steadfast Guardian", Zone::Hand);
        super::big_helpers::cast(&mut game, appa, None, &[]);
        await_target(&mut game, Target::Card(target));
        checked(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish_targets(&mut game);
        let reference = game.object_ref(target);
        TEST_PREVENT_ZONE_MOVE.with(|v| v.set(Some(reference)));
        TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|v| v.set(Some(reference)));
        game.state.zone_rules.get_or_insert_default();
        for _ in 0..40 {
            if matches!(game.state.pending, Pending::ZoneReplacement { .. }) {
                break;
            }
            let action = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                .expect("advance to replacement choice");
            act(&mut game, action);
        }
        assert!(
            matches!(game.state.pending, Pending::ZoneReplacement { player: actor } if actor == player)
        );
        assert_eq!(game.object_ref(target), reference);
        let owner_view = game.observe(player);
        let choice = owner_view.zone_replacement_choice.unwrap();
        assert_eq!(
            choice.choices,
            vec![
                ZoneReplacementKind::Prevent,
                ZoneReplacementKind::Battlefield
            ]
        );
        assert!(
            game.observe(player.other())
                .zone_replacement_choice
                .is_none()
        );
        let pending = game.snapshot();
        // A synthetic battlefield-to-battlefield replacement currently hits the
        // engine's same-zone assertion. No supported real provider has that form;
        // exercise the replayed prevention branch here and a synthetic redirect below.
        let action = Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::Prevent,
        };
        let index = game
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let mut raw = game.clone();
        let mut indexed = game.clone();
        assert_eq!(
            raw.apply(player.other(), action.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(raw.snapshot(), pending);
        raw.apply(player, action.clone()).unwrap();
        indexed.apply_index(player, game.revision(), index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(target).zone, Zone::Battlefield);
        assert_eq!(raw.object_ref(target), reference);
        assert!(!raw.has_airbend_permission(player, target));
        let after = raw.snapshot();
        raw.restore(&pending).unwrap();
        act(&mut raw, action);
        assert_eq!(raw.snapshot(), after);
        TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|v| v.set(None));
    }
}

#[test]
fn airbend_redirected_stack_spell_moves_to_battlefield_without_cast_right() {
    use zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|v| v.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Bear Cub", Zone::Hand);
        super::big_helpers::cast(&mut game, spell, None, &[]);
        let reference = game.object_ref(spell);
        TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|v| v.set(Some(reference)));
        game.airbend_objects(&[spell]);
        assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
        assert_ne!(game.object_ref(spell), reference);
        assert!(!game.has_airbend_permission(player, spell));
        assert!(
            !game
                .state
                .exile_permissions
                .iter()
                .any(|row| row.object.card == spell)
        );
        TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|v| v.set(None));
    }
}

#[test]
fn ascension_end_step_guard_uses_counter_lki_after_paid_source_departure_and_reentry() {
    for player in [Player::P0, Player::P1] {
        for (below_threshold, reenter) in
            [(false, false), (false, true), (true, false), (true, true)]
        {
            let mut game = scenario(player);
            let ascension = put(&mut game, player, "Airbender Ascension", Zone::Battlefield);
            let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            let disenchant = put(&mut game, player.other(), "Disenchant", Zone::Hand);
            game.add_counter(ascension, CounterKind::Quest, 4);
            game.begin_step(Step::End);
            await_target(&mut game, Target::Card(target));
            checked(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            finish_targets(&mut game);
            let trigger = game
                .state
                .stack
                .iter()
                .find(|item| item.source.is_some_and(|source| source.id == ascension))
                .unwrap();
            assert_eq!(
                trigger
                    .source_counter_lki
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|fact| fact.kind == CounterKind::Quest)
                    .unwrap()
                    .count,
                4
            );
            if below_threshold {
                game.pay_counters(ascension, &Counters::of(CounterKind::Quest, 1));
            }
            // Pass priority normally; the opponent pays for a real Disenchant.
            while game.actor() != Some(player.other()) {
                checked(&mut game, Action::Pass);
            }
            super::big_helpers::cast(&mut game, disenchant, None, &[Target::Card(ascension)]);
            for _ in 0..8 {
                if game.object_view(ascension).zone == Zone::Graveyard {
                    break;
                }
                checked(&mut game, Action::Pass);
            }
            assert_eq!(game.object_view(ascension).zone, Zone::Graveyard);
            let old_ascension = game.object_ref(ascension);
            if reenter {
                game.move_object(ascension, Zone::Battlefield);
                assert_ne!(game.object_ref(ascension), old_ascension);
                // A new incarnation cannot supply counters to the old trigger.
                game.add_counter(
                    ascension,
                    CounterKind::Quest,
                    if below_threshold { 7 } else { 0 },
                );
            }
            let before_target = game.object_ref(target);
            let before = game.snapshot();
            let mut branch = game.clone();
            super::big_helpers::settle(&mut game);
            super::big_helpers::settle(&mut branch);
            assert_eq!(game.snapshot(), branch.snapshot());
            assert_eq!(game.object_view(target).zone, Zone::Battlefield);
            assert_eq!(game.object_ref(target) != before_target, !below_threshold);
            branch.restore(&before).unwrap();
            super::big_helpers::settle(&mut branch);
            assert_eq!(branch.snapshot(), game.snapshot());
        }
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
fn ascension_entry_filters_end_step_timing_and_owner_return() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let ascension = put(&mut game, player, "Airbender Ascension", Zone::Hand);
        super::big_helpers::cast(&mut game, ascension, None, &[]);
        super::big_helpers::settle(&mut game); // no creatures: optional ETB target is zero
        let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Hand);
        game.move_object(foreign, Zone::Battlefield);
        super::big_helpers::settle(&mut game);
        let land = put(&mut game, player, "Forest", Zone::Hand);
        game.move_object(land, Zone::Battlefield);
        super::big_helpers::settle(&mut game);
        assert_eq!(game.counter_count(ascension, CounterKind::Quest), 0);
        let own = put(&mut game, player, "Llanowar Elves", Zone::Hand);
        game.move_object(own, Zone::Battlefield);
        super::big_helpers::settle(&mut game);
        assert_eq!(game.counter_count(ascension, CounterKind::Quest), 1);
        game.add_counter(ascension, CounterKind::Quest, 2);
        game.begin_step(Step::End);
        super::big_helpers::settle(&mut game);
        assert_eq!(game.object_view(own).zone, Zone::Battlefield);
        assert_eq!(game.counter_count(ascension, CounterKind::Quest), 3);
        game.add_counter(ascension, CounterKind::Quest, 1);
        game.state.active = player.other();
        game.state.priority = player.other();
        game.begin_step(Step::End);
        super::big_helpers::settle(&mut game);
        assert_eq!(game.counter_count(ascension, CounterKind::Quest), 4);
        let reference = game.object_ref(foreign);
        game.change_controller(reference, player);
        game.state.active = player;
        game.state.priority = player;
        game.begin_step(Step::End);
        await_target(&mut game, Target::Card(foreign));
        let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        finish_targets(&mut game);
        let prior = game.object_ref(foreign);
        super::big_helpers::settle(&mut game);
        assert_ne!(game.object_ref(foreign), prior);
        assert_eq!(game.object_view(foreign).zone, Zone::Battlefield);
        assert_eq!(game.object_view(foreign).controller, player.other());
        assert_eq!(game.object_view(foreign).owner, player.other());
    }
}

#[test]
fn appa_flash_flying_owned_rights_token_ceasing_and_source_departure() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player.other());
        let artifact = put(
            &mut game,
            player.other(),
            "Soul-Guide Lantern",
            Zone::Battlefield,
        );
        let reference = game.object_ref(artifact);
        game.change_controller(reference, player);
        let land = put(&mut game, player, "Forest", Zone::Battlefield);
        let token_card = Card::from_name("@token/6318fca926ee864ca3aad7f8").unwrap();
        game.create_tokens(player, token_card, 1, false);
        let token = game
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, object)| {
                (object.zone == Zone::Battlefield
                    && object.controller == player
                    && object.is_token()
                    && object.card == token_card)
                    .then_some(id as CardId)
            })
            .unwrap();
        let appa = put(&mut game, player, "Appa, Steadfast Guardian", Zone::Hand);
        let murder = put(&mut game, player.other(), "Murder", Zone::Hand);
        checked(&mut game, Action::Pass); // opponent's turn, pass to Appa's controller
        super::big_helpers::cast(&mut game, appa, None, &[]);
        await_target(&mut game, Target::Card(artifact));
        assert_eq!(stats(&game, appa), (3, 4));
        assert!(game.keywords(appa).contains(Keywords::FLYING));
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        checked(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        checked(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(token),
            },
        );
        finish_targets(&mut game);
        while game.actor() != Some(player.other()) {
            checked(&mut game, Action::Pass);
        }
        super::big_helpers::cast(&mut game, murder, None, &[Target::Card(appa)]);
        for _ in 0..8 {
            if game.object_view(appa).zone == Zone::Graveyard {
                break;
            }
            checked(&mut game, Action::Pass);
        }
        assert_eq!(game.object_view(appa).zone, Zone::Graveyard);
        super::big_helpers::settle(&mut game);
        assert_eq!(game.object_view(artifact).zone, Zone::Exile);
        assert!(game.has_airbend_permission(player.other(), artifact));
        assert!(!game.has_airbend_permission(player, artifact));
        assert!(
            !game
                .state
                .exile_permissions
                .iter()
                .any(|row| row.object.card == token)
        );
        assert_ne!(game.object_view(token).zone, Zone::Battlefield);
    }
}

#[test]
fn appa_cast_trigger_tracks_cast_origin_controller_and_survives_countering() {
    for player in [Player::P0, Player::P1] {
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
