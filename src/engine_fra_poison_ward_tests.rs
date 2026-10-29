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

#[test]
fn proliferate_replacement_sibling_survives_provider_departure_and_restore() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let tam = put(&mut game, player, "Tam, the Possibility", Zone::Battlefield);
        put(&mut game, player, "Ajani, Caller of the Pride", Zone::Battlefield);
        let provider = put(&mut game, player, "Serra Angel", Zone::Battlefield);
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
        let subject_ref = game.object_ref(subject);
        for _ in 0..2 {
            act(&mut game, Action::ProliferatePermanent { object: subject_ref });
            // The already-created sibling is an independent event, even if the
            // replacement provider no longer exists when it is processed.
            game.move_object(provider, Zone::Graveyard);
            act(&mut game, Action::FinishProliferate);
            assert_eq!(game.counter_count(subject, CounterKind::PlusOne), 2);
            assert!(matches!(game.state.pending, Pending::Proliferate { player: p } if p == player));
            act(&mut game, Action::ProliferatePermanent { object: subject_ref });
            act(&mut game, Action::FinishProliferate);
            assert_eq!(game.counter_count(subject, CounterKind::PlusOne), 3);
            assert!(!matches!(game.state.pending, Pending::Proliferate { .. }));
            game.restore(&saved).unwrap();
        }
    }
}

#[test]
fn forbidden_poison_ward_cost_cannot_use_ordinary_replacement_to_escape() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for owner in [Player::P0, Player::P1] {
        let mut game = scenario(owner);
        let payer = owner.other();
        let society = put(&mut game, owner, "The Serpent Society", Zone::Battlefield);
        put(&mut game, payer, "Bear Cub", Zone::Battlefield);
        put(&mut game, payer, "Hill Giant", Zone::Battlefield);
        put(&mut game, payer, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut game, payer, "Unsummon", Zone::Hand);
        game.state.priority = payer;
        game.state.players[payer.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut game, Action::Cast {
            card: spell, target: Some(Target::Card(society)), payment: [0, 1, 0, 0, 0, 0],
        });
        let saved = game.snapshot();
        for _ in 0..2 {
            act(&mut game, Action::Pass);
            act(&mut game, Action::Pass);
            assert!(!matches!(game.state.pending, Pending::EffectOptional { .. } | Pending::Replacement { .. }));
            settle(&mut game);
            assert_eq!(game.state.players[payer.index()].poison_counters, 0);
            assert_eq!(game.object_view(society).zone, Zone::Battlefield);
            assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
            game.restore(&saved).unwrap();
        }
    }
}

#[test]
fn tam_proliferate_replaces_only_eligible_player_counter_with_snapshot_replay() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let tam = put(&mut game, player, "Tam, the Possibility", Zone::Battlefield);
        let walker = put(&mut game, player, "Ajani, Caller of the Pride", Zone::Battlefield);
        game.state.objects[walker as usize].counters.loyalty = 3;
        // The test provider doubles counters placed by its controller on themself.
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.state.players[player.index()].poison_counters = 1;
        game.state.players[player.other().index()].poison_counters = 1;
        game.state.players[player.index()].mana = [1, 1, 1, 1, 1, 0];
        act(&mut game, Action::AnnounceActivate {
            card: tam, ability: 0, counter_payment: Counters::default(),
        });
        pay(&mut game);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(matches!(game.state.pending, Pending::Proliferate { player: actor } if actor == player));
        let before = game.snapshot();
        let opponent_choice = Action::ProliferatePlayer { player: player.other() };
        assert!(game.apply(player.other(), opponent_choice.clone()).is_err());
        assert_eq!(game.snapshot(), before);
        for _ in 0..2 {
            act(&mut game, Action::ProliferatePlayer { player });
            act(&mut game, opponent_choice.clone());
            act(&mut game, Action::FinishProliferate);
            assert_eq!(game.state.players[player.index()].poison_counters, 3);
            assert_eq!(game.state.players[player.other().index()].poison_counters, 2);
            assert!(!matches!(game.state.pending, Pending::Proliferate { .. }));
            game.restore(&before).unwrap();
        }
    }
}

#[test]
fn tam_simultaneous_player_and_permanent_replacements_wait_for_public_choice() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let tam = put(&mut game, player, "Tam, the Possibility", Zone::Battlefield);
        let walker = put(&mut game, player, "Jace, the Perfected Mind", Zone::Battlefield);
        game.state.objects[walker as usize].counters.loyalty = 3;
        put(&mut game, player, "Doubling Season", Zone::Battlefield);
        put(&mut game, player, "Doc Samson, Super Psychiatrist", Zone::Battlefield);
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.state.players[player.index()].poison_counters = 1;
        game.state.players[player.index()].mana = [1, 1, 1, 1, 1, 0];
        act(&mut game, Action::AnnounceActivate {
            card: tam, ability: 0, counter_payment: Counters::default(),
        });
        pay(&mut game);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(matches!(game.state.pending, Pending::Proliferate { player: actor } if actor == player));
        let walker_ref = game.object_ref(walker);
        act(&mut game, Action::ProliferatePermanent { object: walker_ref });
        act(&mut game, Action::ProliferatePlayer { player });
        act(&mut game, Action::FinishProliferate);
        assert!(matches!(game.state.pending, Pending::Replacement { player: actor } if actor == player));
        assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 3);
        assert_eq!(game.state.players[player.index()].poison_counters, 1);
        let saved = game.snapshot();
        let choices: Vec<_> = game.legal_actions().into_iter()
            .filter(|action| matches!(action, Action::ChooseReplacement { .. })).collect();
        assert_eq!(choices.len(), 2);
        for choice in choices {
            game.restore(&saved).unwrap();
            assert!(game.apply(player.other(), choice.clone()).is_err());
            assert_eq!(game.snapshot(), saved);
            act(&mut game, choice);
            assert!(matches!(game.counter_count(walker, CounterKind::Loyalty), 6 | 7));
            assert_eq!(game.state.players[player.index()].poison_counters, 3);
        }
    }
}

#[test]
fn mixed_counter_batch_preserves_atomicity_across_prohibition_and_opponent_choice() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for active in [Player::P0, Player::P1] {
        let mut game = scenario(active);
        let recipient = active.other();
        // Active player cannot receive poison. The other player chooses the
        // order of two applicable replacements for their own poison event.
        put(&mut game, active, "Bear Cub", Zone::Battlefield);
        let double = put(&mut game, recipient, "Hill Giant", Zone::Battlefield);
        let plus = put(&mut game, recipient, "Grizzly Bears", Zone::Battlefield);
        let permanent = put(&mut game, active, "Healer's Hawk", Zone::Battlefield);
        let object = game.object_ref(permanent);
        game.effect_counters_with_poison(
            vec![(object, Counters::of(CounterKind::PlusOne, 1))],
            [1, 1], recipient, replacements::ResumeMode::Pending,
        );
        assert!(game.advance_replacements(false));
        assert!(matches!(game.state.pending, Pending::Replacement { player } if player == recipient));
        assert_eq!(game.counter_count(permanent, CounterKind::PlusOne), 0);
        assert_eq!(game.state.players[active.index()].poison_counters, 0);
        assert_eq!(game.state.players[recipient.index()].poison_counters, 0);
        let saved = game.snapshot();
        for (first, expected) in [(double, 3), (plus, 4)] {
            game.restore(&saved).unwrap();
            let action = Action::ChooseReplacement { source: game.object_ref(first), ability: 0 };
            assert!(game.apply(active, action.clone()).is_err());
            assert_eq!(game.snapshot(), saved);
            act(&mut game, action);
            assert_eq!(game.counter_count(permanent, CounterKind::PlusOne), 1);
            assert_eq!(game.state.players[active.index()].poison_counters, 0);
            assert_eq!(game.state.players[recipient.index()].poison_counters, expected);
        }
    }
}

#[test]
fn tam_proliferates_all_player_counter_types_with_atomic_replacements() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let tam = put(&mut game, player, "Tam, the Possibility", Zone::Battlefield);
        let walker = put(&mut game, player, "Ajani, Caller of the Pride", Zone::Battlefield);
        game.state.objects[walker as usize].counters.loyalty = 3;
        let provider = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
        let subject = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        game.add_counter(subject, CounterKind::PlusOne, 1);
        game.state.players[player.index()].add_player_counter(crate::cards::PlayerCounterKind::Energy, 2);
        game.state.players[player.index()].add_player_counter(crate::cards::PlayerCounterKind::Experience, 1);
        game.state.players[player.other().index()].add_player_counter(crate::cards::PlayerCounterKind::Rad, 1);
        game.state.players[player.index()].mana = [1, 1, 1, 1, 1, 0];
        act(&mut game, Action::AnnounceActivate { card: tam, ability: 0, counter_payment: Counters::default() });
        pay(&mut game);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(game.legal_actions().contains(&Action::ProliferatePlayer { player }));
        assert!(game.legal_actions().contains(&Action::ProliferatePlayer { player: player.other() }));
        let object = game.object_ref(subject);
        act(&mut game, Action::ProliferatePermanent { object });
        act(&mut game, Action::ProliferatePlayer { player });
        act(&mut game, Action::ProliferatePlayer { player: player.other() });
        act(&mut game, Action::FinishProliferate);
        assert!(matches!(game.state.pending, Pending::Replacement { player: actor } if actor == player));
        assert_eq!(game.counter_count(subject, CounterKind::PlusOne), 1);
        assert_eq!(game.state.players[player.index()].player_counter(crate::cards::PlayerCounterKind::Energy), 2);
        assert_eq!(game.state.players[player.other().index()].player_counter(crate::cards::PlayerCounterKind::Rad), 1);
        let saved = game.snapshot();
        for (ability, expected) in [(0, 5), (1, 6)] {
            game.restore(&saved).unwrap();
            let choice = Action::ChooseReplacement { source: game.object_ref(provider), ability };
            assert!(game.apply(player.other(), choice.clone()).is_err());
            assert_eq!(game.snapshot(), saved);
            act(&mut game, choice);
            assert_eq!(game.state.players[player.index()].player_counter(crate::cards::PlayerCounterKind::Energy), expected);
            assert_eq!(game.state.players[player.index()].player_counter(crate::cards::PlayerCounterKind::Experience), 2);
            assert_eq!(game.state.players[player.other().index()].player_counter(crate::cards::PlayerCounterKind::Rad), 2);
            assert_eq!(game.counter_count(subject, CounterKind::PlusOne), 2);
            for viewer in [player, player.other()] {
                let view = game.observe(viewer);
                assert!(view.players[player.index()].other_counters.iter().any(|c| c.kind == crate::cards::PlayerCounterKind::Energy && c.count == expected));
            }
        }
    }
}

#[test]
fn radiation_mill_counts_only_nonland_cards_actually_milled() {
    for player in [Player::P0, Player::P1] {
        for replacement in [false, true] {
            let mut game = scenario(player);
            if replacement { put(&mut game, player, "Rest in Peace", Zone::Battlefield); }
            let land = put(&mut game, player, "Forest", Zone::Library);
            let creature = put(&mut game, player, "Bear Cub", Zone::Library);
            let spell = put(&mut game, player, "Shock", Zone::Library);
            let opt = put(&mut game, player, "Opt", Zone::Hand);
            cast(&mut game, opt, None);
            game.resolve_top();
            assert!(game.state.resolving.is_some());
            let before = game.snapshot();
            for _ in 0..2 {
                // A request larger than the library still mills only existing cards.
                let actual = game.mill_radiation(player, 7);
                assert_eq!(actual, if replacement { 0 } else { 2 });
                let destination = if replacement { Zone::Exile } else { Zone::Graveyard };
                for card in [land, creature, spell] { assert_eq!(game.object_view(card).zone, destination); }
                game.restore(&before).unwrap();
            }
        }
    }
}

#[test]
fn sourceless_stack_identity_survives_copy_and_snapshot() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let mut packet = game.activation_item(player, source, 0);
        assert_eq!(packet.source_identity(), Some((game.object_ref(source), Card::from_name("Bear Cub").unwrap())));
        packet.source_less_rule = Some(SourceLessRule::Radiation);
        packet.source_card = Card::SOURCELESS;
        packet.card = CardId::MAX;
        packet.object = ObjectRef { card: CardId::MAX, generation: 0 };
        packet.source_facts.card = Card::SOURCELESS;
        let serialized = serde_json::to_value(&packet).unwrap();
        for field in ["card", "object", "source_card", "source_facts"] {
            assert!(serialized[field].is_null(), "{field} should have no source");
        }
        assert_eq!(packet.source_identity(), None);
        assert_eq!(packet.source(), None);
        assert_eq!(packet.mana_value(), 0);
        let copy = packet.clone();
        assert_eq!(copy.source_identity(), None);
        assert_eq!(copy.source_less_rule, Some(SourceLessRule::Radiation));
        game.state.pending_triggers.push(packet);
        let snapshot = game.snapshot();
        game.state.pending_triggers.clear();
        game.restore(&snapshot).unwrap();
        assert_eq!(game.state.pending_triggers.last().unwrap().source_identity(), None);
        assert_eq!(game.state.pending_triggers.last().unwrap().source_less_rule, Some(SourceLessRule::Radiation));
    }
}

#[test]
fn radiation_trigger_is_sourceless_and_uses_current_counters_on_resolution() {
    for player in [Player::P0, Player::P1] {
        for replacement in [false, true] {
            let mut game = scenario(player);
            if replacement { put(&mut game, player, "Rest in Peace", Zone::Battlefield); }
            let land = put(&mut game, player, "Forest", Zone::Library);
            let creature = put(&mut game, player, "Bear Cub", Zone::Library);
            let spell = put(&mut game, player, "Shock", Zone::Library);
            game.state.players[player.index()].add_player_counter(crate::cards::PlayerCounterKind::Rad, 2);
            game.begin_step(Step::Main1);
            game.prepare_triggers();
            let packet = game.state.stack.last().unwrap();
            assert_eq!(packet.source_identity(), None);
            assert_eq!(packet.kind, StackKind::InherentRule { rule: SourceLessRule::Radiation, affected_player: player });
            for viewer in [player, player.other()] {
                let observed = serde_json::to_value(game.observe(viewer)).unwrap();
                let stack = observed["stack"].as_array().unwrap();
                let source = stack.last().unwrap();
                for field in ["card", "object", "source_card", "source_facts"] { assert!(source[field].is_null()); }
            }
            // Additional rad acquired after the trigger changes its resolution count.
            game.state.players[player.index()].add_player_counter(crate::cards::PlayerCounterKind::Rad, 1);
            let before = game.snapshot();
            for _ in 0..2 {
                game.resolve_top();
                assert_eq!(game.state.players[player.index()].life, if replacement { 20 } else { 18 });
                assert_eq!(game.state.players[player.index()].player_counter(crate::cards::PlayerCounterKind::Rad), if replacement { 3 } else { 1 });
                let destination = if replacement { Zone::Exile } else { Zone::Graveyard };
                for card in [land, creature, spell] { assert_eq!(game.object_view(card).zone, destination); }
                game.restore(&before).unwrap();
            }
            game.state.players[player.index()].other_player_counters.as_mut().unwrap()[crate::cards::PlayerCounterKind::Rad.index() - 1] = 0;
            game.resolve_top();
            assert_eq!(game.object_view(spell).zone, Zone::Library);
            assert_eq!(game.state.players[player.index()].life, 20);
        }
    }
}

#[test]
fn radiation_copy_keeps_affected_player_when_controller_changes() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let first = put(&mut game, player, "Shock", Zone::Library);
        let second = put(&mut game, player, "Bear Cub", Zone::Library);
        let other_card = put(&mut game, player.other(), "Hill Giant", Zone::Library);
        let copier_source = put(&mut game, player.other(), "Gilded Lotus", Zone::Battlefield);
        game.state.players[player.index()].add_player_counter(crate::cards::PlayerCounterKind::Rad, 1);
        game.begin_step(Step::Main1);
        game.prepare_triggers();
        let original = game.state.stack.last().unwrap().clone();
        let mut copier = game.activation_item(player.other(), copier_source, 0);
        copier.targets = vec![BoundTarget { group: 0, target: Target::Stack(original.id), object: None }];
        game.copy_target_ability(&copier, 0);
        if game.legal_actions().contains(&Action::FinishStackTargets) { act(&mut game, Action::FinishStackTargets); }
        let copy = game.state.stack.last().unwrap();
        assert!(copy.is_copy);
        assert_eq!(copy.controller, player.other());
        assert_eq!(copy.source_identity(), None);
        assert_eq!(copy.kind, StackKind::InherentRule { rule: SourceLessRule::Radiation, affected_player: player });
        let snapshot = game.snapshot();
        for _ in 0..2 {
            game.resolve_top();
            // The copy consumes the rad counter; the original's intervening-if fails.
            assert_eq!(game.state.players[player.index()].life, 19);
            assert_eq!(game.state.players[player.index()].player_counter(crate::cards::PlayerCounterKind::Rad), 0);
            game.resolve_top();
            assert_eq!(game.state.players[player.other().index()].life, 20);
            assert_eq!(game.object_view(second).zone, Zone::Graveyard);
            assert_eq!(game.object_view(first).zone, Zone::Library);
            assert_eq!(game.object_view(other_card).zone, Zone::Library);
            game.restore(&snapshot).unwrap();
        }
    }
}

#[test]
fn countered_radiation_trigger_neither_mills_nor_changes_counters() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let top = put(&mut game, player, "Shock", Zone::Library);
        game.state.players[player.index()].add_player_counter(crate::cards::PlayerCounterKind::Rad, 1);
        game.begin_step(Step::Main1);
        game.prepare_triggers();
        let id = game.state.stack.last().unwrap().id;
        let saved = game.snapshot();
        for _ in 0..2 {
            assert!(game.counter_stack_item(id, false));
            assert_eq!(game.object_view(top).zone, Zone::Library);
            assert_eq!(game.state.players[player.index()].life, 20);
            assert_eq!(game.state.players[player.index()].player_counter(crate::cards::PlayerCounterKind::Rad), 1);
            game.restore(&saved).unwrap();
        }
    }
}

#[test]
fn own_counter_replacements_precede_general_replacements_and_prohibition() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for player in [Player::P0, Player::P1] {
        for prohibited in [false, true] {
            let mut game = scenario(player);
            let top = put(&mut game, player, "Forest", Zone::Library);
            let opt = put(&mut game, player, "Opt", Zone::Hand);
            put(&mut game, player, "Hill Giant", Zone::Battlefield); // doubles poison normally
            if prohibited { put(&mut game, player, "Bear Cub", Zone::Battlefield); }
            cast(&mut game, opt, None);
            game.resolve_top();
            assert!(matches!(game.state.pending, Pending::EffectLibrary { .. }));
            let origin_stack_id = game.state.resolving.as_ref().unwrap().item.id;
            let make = |effect, factor, add| replacements::SelfPlayerCounterReplacement {
                origin_stack_id, effect, player, kind: crate::cards::PlayerCounterKind::Poison, factor, add, redirect_player: None, redirect_kind: None, split: &[],
            };
            game.effect_player_counters_with_self(player, crate::cards::PlayerCounterKind::Poison,
                1, origin_stack_id, vec![make(0, 2, 0), make(1, 1, 1)], replacements::ResumeMode::Program);
            assert!(game.advance_replacements(false));
            let before = game.snapshot();
            assert_eq!(game.observe(player).replacement_choice.unwrap().self_counter_effects.len(), 2);
            for (effect, expected) in [(0, 6), (1, 8)] {
                game.restore(&before).unwrap();
                let action = Action::ChooseSelfCounterReplacement { effect };
                assert!(game.apply(player.other(), action.clone()).is_err());
                assert_eq!(game.snapshot(), before);
                act(&mut game, action);
                assert_eq!(game.state.players[player.index()].poison_counters, if prohibited { 0 } else { expected });
                assert_eq!(game.object_view(top).zone, Zone::Library);
            }
        }
    }
}

#[test]
fn redirected_self_event_stays_distinct_from_simultaneous_matching_event() {
    let _fixture = crate::cards::test_counter_providers::activate();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let recipient = player.other();
        let source = put(&mut game, recipient, "Goblin Piker", Zone::Battlefield);
        put(&mut game, player, "Forest", Zone::Library);
        let opt = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, opt, None);
        game.resolve_top();
        let origin = game.state.resolving.as_ref().unwrap().item.id;
        let mut counters = [[0; 5]; 2];
        counters[player.index()][crate::cards::PlayerCounterKind::Poison.index()] = 1;
        counters[recipient.index()][crate::cards::PlayerCounterKind::Energy.index()] = 1;
        let own = replacements::SelfPlayerCounterReplacement { origin_stack_id: origin, effect: 0,
            player, kind: crate::cards::PlayerCounterKind::Poison, factor: 1, add: 0,
            redirect_player: Some(recipient), redirect_kind: Some(crate::cards::PlayerCounterKind::Energy), split: &[] };
        game.effect_mixed_player_counters_with_self(counters, recipient, origin, vec![own], replacements::ResumeMode::Program);
        assert!(game.advance_replacements(false));
        let before = game.snapshot();
        for _ in 0..2 {
            for _ in 0..2 {
                assert!(matches!(game.state.pending, Pending::Replacement { player: actor } if actor == recipient));
                assert_eq!(game.state.players[recipient.index()].player_counter(crate::cards::PlayerCounterKind::Energy), 0);
                act(&mut game, Action::ChooseReplacement { source: game.object_ref(source), ability: 0 });
            }
            // Each separate 1-energy event is doubled and then incremented.
            assert_eq!(game.state.players[recipient.index()].player_counter(crate::cards::PlayerCounterKind::Energy), 6);
            assert_eq!(game.state.players[player.index()].poison_counters, 0);
            game.restore(&before).unwrap();
        }
    }
}

#[test]
fn authored_self_replacement_instruction_uses_real_resolving_packet() {
    use crate::cards::SelfPlayerCounterTemplate;
    static REPLACEMENTS: &[SelfPlayerCounterTemplate] = &[
        SelfPlayerCounterTemplate { factor: 1, add: 0, redirect_to_opponent: true, redirect_kind: Some(crate::cards::PlayerCounterKind::Energy), split: &[] },
    ];
    static PROGRAM: &[Instruction] = &[Instruction::PutPlayerCountersWithSelf {
        players: PlayerSet::Controller, kind: crate::cards::PlayerCounterKind::Poison,
        amount: 1, effects: REPLACEMENTS,
    }];
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Forest", Zone::Library);
        let opt = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, opt, None);
        game.resolve_top();
        assert!(matches!(game.state.pending, Pending::EffectLibrary { .. }));
        let r = game.state.resolving.as_mut().unwrap();
        r.library = None;
        r.program = PROGRAM;
        r.pc = 0;
        game.state.pending = Pending::Priority;
        let before = game.snapshot();
        for _ in 0..2 {
            game.run_program();
            assert_eq!(game.state.players[player.index()].poison_counters, 0);
            assert_eq!(game.state.players[player.other().index()].player_counter(crate::cards::PlayerCounterKind::Energy), 1);
            game.restore(&before).unwrap();
        }
    }
}

#[test]
fn split_self_events_choose_affected_players_in_apnap_order() {
    use crate::cards::PlayerCounterSplit;
    static SPLIT: &[PlayerCounterSplit] = &[
        PlayerCounterSplit { to_opponent: true, kind: Some(crate::cards::PlayerCounterKind::Energy), factor: 1, add: 0 },
        PlayerCounterSplit { to_opponent: false, kind: Some(crate::cards::PlayerCounterKind::Energy), factor: 1, add: 0 },
    ];
    for active in [Player::P0, Player::P1] {
        let mut game = scenario(active);
        put(&mut game, active, "Forest", Zone::Library);
        let opt = put(&mut game, active, "Opt", Zone::Hand);
        cast(&mut game, opt, None);
        game.resolve_top();
        let origin = game.state.resolving.as_ref().unwrap().item.id;
        let make = |effect, player, kind, factor, add, split: &'static [PlayerCounterSplit]| replacements::SelfPlayerCounterReplacement {
            origin_stack_id: origin, effect, player, kind, factor, add,
            redirect_player: None, redirect_kind: None, split,
        };
        let mut counters = [[0; 5]; 2];
        counters[active.index()][crate::cards::PlayerCounterKind::Poison.index()] = 1;
        let effects = vec![
            make(0, active, crate::cards::PlayerCounterKind::Poison, 1, 0, SPLIT),
            make(1, active, crate::cards::PlayerCounterKind::Energy, 2, 0, &[]),
            make(2, active, crate::cards::PlayerCounterKind::Energy, 1, 1, &[]),
            make(3, active.other(), crate::cards::PlayerCounterKind::Energy, 2, 0, &[]),
            make(4, active.other(), crate::cards::PlayerCounterKind::Energy, 1, 1, &[]),
        ];
        game.effect_mixed_player_counters_with_self(counters, active, origin, effects, replacements::ResumeMode::Program);
        assert!(game.advance_replacements(false));
        assert!(matches!(game.state.pending, Pending::Replacement { player } if player == active));
        let before = game.snapshot();
        for _ in 0..2 {
            let first = Action::ChooseSelfCounterReplacement { effect: 1 };
            assert!(game.apply(active.other(), first.clone()).is_err());
            assert_eq!(game.snapshot(), before);
            act(&mut game, first);
            assert!(matches!(game.state.pending, Pending::Replacement { player } if player == active.other()));
            assert_eq!(game.state.players[active.index()].player_counter(crate::cards::PlayerCounterKind::Energy), 0);
            act(&mut game, Action::ChooseSelfCounterReplacement { effect: 3 });
            for player in [active, active.other()] {
                assert_eq!(game.state.players[player.index()].player_counter(crate::cards::PlayerCounterKind::Energy), 3);
            }
            game.restore(&before).unwrap();
        }
    }
}

#[test]
fn mixed_permanent_and_split_player_replacements_follow_apnap() {
    use crate::cards::PlayerCounterSplit;
    static SPLIT: &[PlayerCounterSplit] = &[
        PlayerCounterSplit { to_opponent: true, kind: Some(crate::cards::PlayerCounterKind::Energy), factor: 1, add: 0 },
        PlayerCounterSplit { to_opponent: false, kind: Some(crate::cards::PlayerCounterKind::Energy), factor: 1, add: 0 },
    ];
    for active in [Player::P0, Player::P1] {
        let mut game = scenario(active);
        let walker = put(&mut game, active, "Jace, the Perfected Mind", Zone::Battlefield);
        game.state.objects[walker as usize].counters.loyalty = 3;
        let double = put(&mut game, active, "Doubling Season", Zone::Battlefield);
        put(&mut game, active, "Doc Samson, Super Psychiatrist", Zone::Battlefield);
        put(&mut game, active, "Forest", Zone::Library);
        let opt = put(&mut game, active, "Opt", Zone::Hand);
        cast(&mut game, opt, None);
        game.resolve_top();
        let origin = game.state.resolving.as_ref().unwrap().item.id;
        let make = |effect, player, kind, factor, add, split: &'static [PlayerCounterSplit]| replacements::SelfPlayerCounterReplacement {
            origin_stack_id: origin, effect, player, kind, factor, add,
            redirect_player: None, redirect_kind: None, split,
        };
        let mut counters = [[0; 5]; 2];
        counters[active.index()][crate::cards::PlayerCounterKind::Poison.index()] = 1;
        let effects = vec![
            make(0, active, crate::cards::PlayerCounterKind::Poison, 1, 0, SPLIT),
            make(1, active, crate::cards::PlayerCounterKind::Energy, 2, 0, &[]),
            make(2, active, crate::cards::PlayerCounterKind::Energy, 1, 1, &[]),
            make(3, active.other(), crate::cards::PlayerCounterKind::Energy, 2, 0, &[]),
            make(4, active.other(), crate::cards::PlayerCounterKind::Energy, 1, 1, &[]),
        ];
        let walker_ref = game.object_ref(walker);
        game.effect_mixed_counters_with_self(vec![(walker_ref, Counters::of(CounterKind::Loyalty, 1))],
            counters, active, origin, effects, replacements::ResumeMode::Program);
        assert!(game.advance_replacements(false));
        assert!(matches!(game.state.pending, Pending::Replacement { player } if player == active));
        let before = game.snapshot();
        for _ in 0..2 {
            // The active player's permanent event precedes that player's split event.
            act(&mut game, Action::ChooseReplacement { source: game.object_ref(double), ability: 0 });
            assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 3);
            assert!(matches!(game.state.pending, Pending::Replacement { player } if player == active));
            act(&mut game, Action::ChooseSelfCounterReplacement { effect: 1 });
            assert!(matches!(game.state.pending, Pending::Replacement { player } if player == active.other()));
            assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 3);
            act(&mut game, Action::ChooseSelfCounterReplacement { effect: 3 });
            assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 6);
            for player in [active, active.other()] {
                assert_eq!(game.state.players[player.index()].player_counter(crate::cards::PlayerCounterKind::Energy), 3);
            }
            game.restore(&before).unwrap();
        }
    }
}
