//! Public combat requirements and exact quantities across relevant zones.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn names_count_controlled_creature_tokens_and_hidden_exile_cannot_contribute() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let source = game.object_ref(source);
    let spirit = Card::from_catalogue_id(5342);
    let goblin = Card::from_catalogue_id(5343);
    for (controller, token, amount) in [
        (player, spirit, 2),
        (player, goblin, 1),
        (player.other(), goblin, 1),
    ] {
        game.create_tokens(controller, token, amount, false);
        assert!(!game.advance_entries());
    }
    assert_eq!(
        game.stat_value(
            StatValue::DistinctControlledCreatureTokenNames,
            source,
            player
        ),
        2
    );

    let _named = put(&mut game, player, "Shock", Zone::Graveyard);
    let _subtype = put(&mut game, player, "Bear Cub", Zone::Exile);
    let hidden = put(&mut game, player, "Grizzly Bears", Zone::Exile);
    put(&mut game, player.other(), "Shock", Zone::Graveyard);
    game.remember_face_down_exile(game.object_ref(hidden), 0b11, None);
    let count = StatValue::OwnedGraveyardOrFaceUpExileNameOrSubtype {
        name: "Shock",
        subtype: "Bear",
    };
    assert_eq!(game.stat_value(count, source, player), 2);
}

const EVASION: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CannotBeBlockedByNamelessCreatures],
}];
const WATCHER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: EVASION,
    extras: &CardExtras {
        static_ability_groups: EVASION,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerAttacksWithAtLeast(2),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn nameless_blockers_are_ineligible_and_an_opposing_declaration_triggers_the_observer() {
    let watcher = Card::from_name("Grizzly Bears").unwrap();
    let _rules = fixture_rules::activate(&[(watcher, &WATCHER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, watcher, Zone::Battlefield);
    let named = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let hidden = put(&mut game, opponent, "Forest", Zone::Library);
    game.cloak_library_top(opponent, opponent);
    assert!(!game.advance_entries());
    assert!(game.can_block_pair(named, source));
    assert!(!game.can_block_pair(hidden, source));

    game.state.active = opponent;
    game.state.priority = opponent;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    let second = put(&mut game, opponent, "Llanowar Elves", Zone::Battlefield);
    act(&mut game, Action::Attack { card: named });
    act(&mut game, Action::Attack { card: second });
    act(&mut game, Action::FinishAttackers);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

const TOUGHNESS_DISCOUNT: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    extras: &CardExtras {
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::TotalControlledCreatureToughness,
            turn: CostTurn::Any,
            guard: StepGuard::Always,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn total_toughness_discount_counts_every_controlled_creature_at_announcement() {
    let spell = Card::from_name("Day of Judgment").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &TOUGHNESS_DISCOUNT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let card = spawn(&mut game, player, spell, Zone::Hand);
    let announce = Action::AnnounceCast { card };
    let payment = [2, 0, 0, 0, 0, 0];
    game.state.players[player.index()].mana = payment;
    put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    reject(&mut game, announce.clone());
    put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    act(&mut game, announce);
    act(&mut game, Action::PayCosts { payment });
    settle(&mut game);
}
