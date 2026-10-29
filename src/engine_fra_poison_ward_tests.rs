use super::*;

#[test]
fn serpent_ward_uses_targeting_player_cost_and_keeps_choice_transactional() {
    for owner in [Player::P0, Player::P1] {
        let mut game = scenario(owner);
        let society = put(&mut game, owner, "The Serpent Society", Zone::Battlefield);
        let attacker = owner.other();
        let spell = put(&mut game, attacker, "Unsummon", Zone::Hand);
        game.state.priority = attacker;
        game.state.players[attacker.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut game, Action::Cast {
            card: spell, target: Some(Target::Card(society)), payment: [0, 1, 0, 0, 0, 0],
        });
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(matches!(game.state.pending, Pending::EffectOptional { player, .. } if player == attacker));
        let saved = game.snapshot();
        for pay_cost in [false, true] {
            game.restore(&saved).unwrap();
            let action = Action::ChooseOptional { accept: pay_cost };
            assert!(game.apply(owner, action.clone()).is_err());
            assert_eq!(game.snapshot(), saved);
            act(&mut game, action);
            settle(&mut game);
            assert_eq!(game.state.players[attacker.index()].poison_counters, if pay_cost { 5 } else { 0 });
            assert_eq!(game.state.players[owner.index()].poison_counters, 0);
            assert_eq!(game.object_view(society).zone, if pay_cost { Zone::Hand } else { Zone::Battlefield });
        }
    }
}

#[test]
fn serpent_ward_still_offers_payment_after_related_spell_leaves() {
    for owner in [Player::P0, Player::P1] {
        let mut game = scenario(owner);
        let society = put(&mut game, owner, "The Serpent Society", Zone::Battlefield);
        let attacker = owner.other();
        let spell = put(&mut game, attacker, "Unsummon", Zone::Hand);
        game.state.priority = attacker;
        game.state.players[attacker.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut game, Action::Cast {
            card: spell, target: Some(Target::Card(society)), payment: [0, 1, 0, 0, 0, 0],
        });
        game.prepare_triggers();
        game.move_object(spell, Zone::Graveyard);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(matches!(game.state.pending, Pending::EffectOptional { player, .. } if player == attacker));
        let saved = game.snapshot();
        for pay_cost in [false, true] {
            game.restore(&saved).unwrap();
            act(&mut game, Action::ChooseOptional { accept: pay_cost });
            settle(&mut game);
            assert_eq!(game.state.players[attacker.index()].poison_counters, if pay_cost { 5 } else { 0 });
            assert_eq!(game.object_view(society).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn poison_prohibition_tracks_live_provider_controller_before_ward_resolves() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for owner in [Player::P0, Player::P1] {
        for transfer in [false, true] {
            let mut game = scenario(owner);
            let society = put(&mut game, owner, "The Serpent Society", Zone::Battlefield);
            let provider = put(&mut game, owner, "Bear Cub", Zone::Battlefield);
            let attacker = owner.other();
            let spell = put(&mut game, attacker, "Unsummon", Zone::Hand);
            game.state.priority = attacker;
            game.state.players[attacker.index()].mana = [0, 1, 0, 0, 0, 0];
            act(&mut game, Action::Cast { card: spell, target: Some(Target::Card(society)), payment: [0, 1, 0, 0, 0, 0] });
            if transfer { game.change_controller(game.object_ref(provider), attacker); }
            act(&mut game, Action::Pass);
            act(&mut game, Action::Pass);
            if transfer {
                assert!(!matches!(game.state.pending, Pending::EffectOptional { .. }));
                settle(&mut game);
                assert_eq!(game.object_view(society).zone, Zone::Battlefield);
                assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
                assert_eq!(game.state.players[attacker.index()].poison_counters, 0);
            } else {
                assert!(matches!(game.state.pending, Pending::EffectOptional { player, .. } if player == attacker));
                act(&mut game, Action::ChooseOptional { accept: true });
                settle(&mut game);
                assert_eq!(game.state.players[attacker.index()].poison_counters, 5);
                assert_eq!(game.object_view(society).zone, Zone::Hand);
            }
        }
    }
}

#[test]
fn poison_cost_replacements_offer_affected_player_order_and_restore() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for owner in [Player::P0, Player::P1] {
        let mut game = scenario(owner);
        let society = put(&mut game, owner, "The Serpent Society", Zone::Battlefield);
        let payer = owner.other();
        let double = put(&mut game, payer, "Hill Giant", Zone::Battlefield);
        let plus = put(&mut game, payer, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut game, payer, "Unsummon", Zone::Hand);
        game.state.priority = payer;
        game.state.players[payer.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut game, Action::Cast { card: spell, target: Some(Target::Card(society)), payment: [0, 1, 0, 0, 0, 0] });
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        act(&mut game, Action::ChooseOptional { accept: true });
        assert!(matches!(game.state.pending, Pending::Replacement { player } if player == payer));
        let replacement = game.observe(owner).replacement_choice.unwrap().player_counters.unwrap();
        assert_eq!(replacement.player, payer);
        assert_eq!(replacement.placed_by, payer);
        assert!(!replacement.effect);
        let saved = game.snapshot();
        for (first, expected) in [(double, 11), (plus, 12)] {
            game.restore(&saved).unwrap();
            let action = Action::ChooseReplacement { source: game.object_ref(first), ability: 0 };
            assert!(game.apply(owner, action.clone()).is_err());
            assert_eq!(game.snapshot(), saved);
            act(&mut game, action);
            assert_eq!(game.state.players[payer.index()].poison_counters, expected);
        }
    }
}

#[test]
fn effects_only_counter_provider_excludes_cost_and_tracks_controller() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for payer in [Player::P0, Player::P1] {
        let mut game = scenario(payer);
        let provider = put(&mut game, payer, "Aegis Turtle", Zone::Battlefield);
        game.pay_poison_counter_cost(payer, 1);
        assert_eq!(game.state.players[payer.index()].poison_counters, 1);
        let mut poison = [0; 2];
        poison[payer.index()] = 1;
        game.effect_counters_with_poison(Vec::new(), poison, payer, replacements::ResumeMode::Pending);
        game.advance_replacements(false);
        assert_eq!(game.state.players[payer.index()].poison_counters, 3);
        game.change_controller(game.object_ref(provider), payer.other());
        game.effect_counters_with_poison(Vec::new(), poison, payer, replacements::ResumeMode::Pending);
        game.advance_replacements(false);
        assert_eq!(game.state.players[payer.index()].poison_counters, 4);
    }
}

#[test]
fn player_counter_prohibition_stops_effects_but_not_opponent_counters() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for controller in [Player::P0, Player::P1] {
        let mut game = scenario(controller);
        let provider = put(&mut game, controller, "Bear Cub", Zone::Battlefield);
        game.effect_counters_with_poison(Vec::new(), [1, 1], controller, replacements::ResumeMode::Pending);
        game.advance_replacements(false);
        assert_eq!(game.state.players[controller.index()].poison_counters, 0);
        assert_eq!(game.state.players[controller.other().index()].poison_counters, 1);
        game.change_controller(game.object_ref(provider), controller.other());
        game.effect_counters_with_poison(Vec::new(), [1, 1], controller, replacements::ResumeMode::Pending);
        game.advance_replacements(false);
        assert_eq!(game.state.players[controller.index()].poison_counters, 1);
        assert_eq!(game.state.players[controller.other().index()].poison_counters, 1);
    }
}

#[test]
fn proliferate_replacement_creates_separate_owned_choices() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let tam = put(&mut game, player, "Tam, the Possibility", Zone::Battlefield);
        put(&mut game, player, "Ajani, Caller of the Pride", Zone::Battlefield);
        put(&mut game, player, "Serra Angel", Zone::Battlefield);
        let subject = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        game.add_counter(subject, CounterKind::PlusOne, 1);
        game.state.players[player.index()].mana = [1, 1, 1, 1, 1, 0];
        act(&mut game, Action::AnnounceActivate {
            card: tam, ability: 0, counter_payment: Counters::default(),
        });
        pay(&mut game);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(matches!(game.state.pending, Pending::Proliferate { player: p } if p == player));
        let saved = game.snapshot();
        let reference = game.object_ref(subject);
        for replay in 0..2 {
            if replay > 0 { game.restore(&saved).unwrap(); }
            act(&mut game, Action::ProliferatePermanent { object: reference });
            act(&mut game, Action::FinishProliferate);
            assert_eq!(game.counter_count(subject, CounterKind::PlusOne), 2);
            assert!(matches!(game.state.pending, Pending::Proliferate { player: p } if p == player));
            // Declining the second event is independent from the first choice.
            act(&mut game, Action::FinishProliferate);
            assert_eq!(game.counter_count(subject, CounterKind::PlusOne), 2);
            assert!(!matches!(game.state.pending, Pending::Proliferate { .. }));
        }
    }
}

#[test]
fn player_counter_prohibition_stops_when_provider_loses_abilities() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for controller in [Player::P0, Player::P1] {
        let mut game = scenario(controller);
        let provider = put(&mut game, controller, "Bear Cub", Zone::Battlefield);
        let source = put(&mut game, controller.other(), "Forest", Zone::Battlefield);
        let reference = game.object_ref(provider);
        let source = game.object_ref(source);
        assert!(game.player_counters_prohibited(controller, crate::cards::PlayerCounterKind::Poison));
        let before = game.snapshot();
        let timestamp = game.next_role_timestamp();
        game.state.roles.get_or_insert_default().source_linked_effects.push(SourceLinkedEffectView {
            source, recipient: reference, controller: controller.other(),
            kind: SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn, timestamp,
        });
        assert!(!game.player_counters_prohibited(controller, crate::cards::PlayerCounterKind::Poison));
        game.effect_counters_with_poison(Vec::new(), [1, 1], controller, replacements::ResumeMode::Pending);
        game.advance_replacements(false);
        assert_eq!(game.state.players[controller.index()].poison_counters, 1);
        game.restore(&before).unwrap();
        assert!(game.player_counters_prohibited(controller, crate::cards::PlayerCounterKind::Poison));
    }
}
