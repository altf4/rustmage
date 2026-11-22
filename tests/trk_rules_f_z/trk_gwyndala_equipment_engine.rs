//! Equipment token attachment and simultaneous temporary copies for Gwyndala.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::{cards::*, engine::*};

const CONTROLLED_EQUIPMENT: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 16,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Subtype("Equipment"),
};

const GWYNDALA: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateEquipmentForSource {
            token: Card::MshSturdyShieldToken,
            amount: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(CONTROLLED_EQUIPMENT)),
        program: &[Instruction::CopyControlledEquipmentTokensFromTargetUntilEndOfTurn],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn rules() -> fixture_rules::Guard {
    fixture_rules::activate(&[(Card::GrizzlyBears, &GWYNDALA)])
}

fn equipment_tokens(game: &Game, controller: Player) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == controller
                && object.is_token()
                && game.is_equipment(id as CardId))
            .then_some(id as CardId)
        })
        .collect()
}

fn announce_copy(game: &mut Game, source: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

#[test]
fn created_equipment_tokens_attach_to_exact_source_and_survive_its_departure() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Doubling Season", Zone::Battlefield);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        cast(&mut game, source, None);
        settle(&mut game);
        let tokens = equipment_tokens(&game, player);
        assert_eq!(tokens.len(), 2);
        assert!(
            tokens
                .iter()
                .all(|&id| { game.object_view(id).attached_to == Some(game.object_ref(source)) })
        );
        assert_eq!(stats(&game, source), (3, 5));

        let mut departed = scenario(player);
        let source = put(&mut departed, player, "Grizzly Bears", Zone::Hand);
        cast(&mut departed, source, None);
        act(&mut departed, Action::Pass);
        act(&mut departed, Action::Pass);
        assert_eq!(departed.object_view(source).zone, Zone::Battlefield);
        assert_eq!(departed.state.stack.len(), 1);
        departed.move_object(source, Zone::Graveyard);
        settle(&mut departed);
        let tokens = equipment_tokens(&departed, player);
        assert_eq!(tokens.len(), 1);
        assert_eq!(departed.object_view(tokens[0]).attached_to, None);
    }
}

#[test]
fn controlled_equipment_tokens_copy_one_frozen_target_and_restore_at_cleanup() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        cast(&mut game, source, None);
        settle(&mut game);
        let attached = equipment_tokens(&game, player)[0];
        let additional = game.create_token_batch_capture(
            player,
            Card::MshSturdyShieldToken,
            1,
            false,
            None,
            true,
        )[0]
        .card;
        let opponent_token = game.create_token_batch_capture(
            player.other(),
            Card::MshSturdyShieldToken,
            1,
            false,
            None,
            true,
        )[0]
        .card;
        let target = put(&mut game, player, "Short Bow", Zone::Battlefield);
        assert_eq!(stats(&game, source), (2, 3));
        announce_copy(&mut game, source);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        let choose_target = Action::ChooseTarget {
            target: Target::Card(target),
        };
        let checkpoint = game.snapshot();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == choose_target)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, game.revision(), index).unwrap();
        act(&mut game, choose_target);
        assert_eq!(game.snapshot(), indexed.snapshot());
        pay(&mut game);
        settle(&mut game);
        for id in [attached, additional] {
            let view = game.object_view(id);
            assert_eq!(view.name, "Short Bow");
            assert_eq!(view.card, Card::from_name("Short Bow").unwrap());
            assert!(view.is_token());
        }
        assert_eq!(
            game.object_view(attached).attached_to,
            Some(game.object_ref(source))
        );
        assert_eq!(stats(&game, source), (2, 2));
        assert!(game.keywords(source).contains(Keywords::REACH));
        assert!(game.keywords(source).contains(Keywords::VIGILANCE));
        assert_eq!(game.object_view(opponent_token).name, "Sturdy Shield");
        assert_eq!(game.object_view(target).name, "Short Bow");

        game.expire_temporary_copies(true);
        assert_eq!(game.object_view(attached).name, "Sturdy Shield");
        assert_eq!(game.object_view(additional).name, "Sturdy Shield");
        assert_eq!(
            game.object_view(attached).attached_to,
            Some(game.object_ref(source))
        );
        assert_eq!(stats(&game, source), (2, 3));
        assert!(!game.keywords(source).contains(Keywords::REACH));

        indexed.restore(&checkpoint).unwrap();
        act(
            &mut indexed,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut indexed);
        settle(&mut indexed);
        indexed.expire_temporary_copies(true);
        assert_eq!(indexed.snapshot(), game.snapshot());
    }
}

#[test]
fn stale_equipment_target_prevents_the_copy_without_touching_tokens() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    let token = equipment_tokens(&game, player)[0];
    let target = put(&mut game, player, "Short Bow", Zone::Battlefield);
    announce_copy(&mut game, source);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(&mut game);
    game.move_object(target, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(token).name, "Sturdy Shield");
    assert_eq!(
        game.object_view(token).attached_to,
        Some(game.object_ref(source))
    );
}
