//! Death batches retain signed last-known power for a copiable token base.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SPIRIT: Card = Card::from_catalogue_id(5326);
const RAT: Card = Card::from_catalogue_id(5323);
const SIZED_DEATH_TOKEN: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::CreatureDiesBatch {
        filter: CreatureFilter {
            relation: 0,
            other: false,
            subtype: None,
            keyword: Keywords::NONE,
            attacking: false,
            tokens: Some(false),
            artifact: false,
        },
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::CreateSizedTokenFromDeathBatchPower { token: SPIRIT }],
};
const PERMANENT_OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[SIZED_DEATH_TOKEN],
    ..CardRules::EMPTY
};
const CREATURE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    ..PERMANENT_OBSERVER
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn spirits(game: &Game, player: Player) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.card == SPIRIT && object.controller == player && object.zone == Zone::Battlefield
        })
        .map(|(index, _)| index as CardId)
        .collect()
}

fn assert_base(game: &Game, token: CardId, size: i128) {
    let base = game
        .copiable_recipe(game.object_ref(token))
        .copiable_base()
        .unwrap();
    assert_eq!((base.power, base.toughness), (size, size));
}

fn death_power(game: &Game) -> i128 {
    assert_eq!(game.state.stack.len(), 1);
    let EventObjectFacts::DeathBatchPower(power) =
        game.state.stack[0].event_object.as_ref().unwrap().facts
    else {
        panic!("missing signed death batch power");
    };
    power
}

#[test]
fn simultaneous_deaths_sum_controlled_nontokens_using_signed_last_known_power() {
    let _rules = fixture_rules::activate(&[(named("Gingerbrute"), &CREATURE_OBSERVER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let observer = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
        let positive = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.state.objects[positive as usize].growth.power = 2;
        let negative = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let old = game.object_ref(negative);
        game.record_control(old, player, None);
        game.state.objects[negative as usize].growth.power = -4;
        let excluded = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
        let exiled = put(&mut game, player, "Magnigoth Sentry", Zone::Battlefield);
        game.add_counter(exiled, CounterKind::Finality, 1);
        game.create_tokens(player, RAT, 1, false);
        let sweeper = put(&mut game, player, "Day of Judgment", Zone::Hand);
        cast(&mut game, sweeper, None);
        pass_priority(&mut game);

        for id in [observer, positive, negative, excluded] {
            assert_eq!(game.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(game.object_view(exiled).zone, Zone::Exile);
        assert_eq!(game.object_view(negative).owner, opponent);
        assert_eq!(death_power(&game), 4);
        game.move_object(negative, Zone::Battlefield);
        assert_ne!(game.object_ref(negative), old);
        game.state.objects[negative as usize].growth.power = 20;
        settle(&mut game);
        let created = spirits(&game, player);
        assert_eq!(created.len(), 1);
        assert_base(&game, created[0], 4);
    }
}

#[test]
fn negative_base_survives_with_anthems_and_token_replacement_preserves_it() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &PERMANENT_OBSERVER)]);
    for player in [Player::P0, Player::P1] {
        let mut root = scenario(player);
        put(&mut root, player, "Short Bow", Zone::Battlefield);
        for _ in 0..2 {
            put(&mut root, player, "Anthem of Champions", Zone::Battlefield);
            put(&mut root, player, "Doubling Season", Zone::Battlefield);
        }
        let victim = put(&mut root, player, "Bear Cub", Zone::Battlefield);
        root.state.objects[victim as usize].growth.power = -5;
        assert_eq!(root.object_view(victim).power, -1);
        let murder = put(&mut root, player, "Murder", Zone::Hand);
        cast(&mut root, murder, Some(Target::Card(victim)));
        pass_priority(&mut root);
        assert_eq!(death_power(&root), -1);
        pass_priority(&mut root);

        assert_eq!(root.actor(), Some(player));
        assert!(spirits(&root, player).is_empty());
        let actions = root.legal_actions();
        let index = actions
            .iter()
            .position(|action| matches!(action, Action::ChooseReplacement { .. }))
            .unwrap();
        let action = actions[index].clone();
        let revision = root.revision();
        let saved = root.snapshot();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        drop(root);

        act(&mut raw, action.clone());
        indexed.apply_index(player, revision, index).unwrap();
        settle(&mut raw);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        let created = spirits(&raw, player);
        assert_eq!(created.len(), 4);
        for token in created {
            assert_base(&raw, token, -1);
            assert_eq!(stats(&raw, token), (1, 1));
        }
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}

#[test]
fn a_nonpositive_token_enters_then_dies_without_a_toughness_boost() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &PERMANENT_OBSERVER)]);
    let player = Player::P0;
    for power in [-1i128, 0] {
        let mut game = scenario(player);
        put(&mut game, player, "Short Bow", Zone::Battlefield);
        put(&mut game, player, "Impact Tremors", Zone::Battlefield);
        let victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.state.objects[victim as usize].growth.power = power - 2;
        let murder = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, murder, Some(Target::Card(victim)));
        pass_priority(&mut game);
        assert_eq!(death_power(&game), power);
        settle(&mut game);
        assert!(spirits(&game, player).is_empty());
        assert_eq!(
            game.state.players[player.other().index()].life,
            STARTING_LIFE - 1
        );
    }
}
