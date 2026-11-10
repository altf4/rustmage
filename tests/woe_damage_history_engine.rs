//! Actual damage accumulates across healed marks for one permanent incarnation.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const RAT: Card = Card(5323);
const HEAL: ReplacementSpec = ReplacementSpec::HealPreviousDamageToSelf;
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, HEAL)],
}];
const RECIPIENT: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfCreatureDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensScaled {
            token: RAT,
            amount: AmountValue::SourceDamageReceivedThisTurn,
            tapped: false,
        }],
    }],
    extras: &CardExtras {
        replacement_effects: &[HEAL],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const BLINK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::BlinkImmediately {
            counter: None,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};

fn rats(game: &Game) -> usize {
    game.state
        .objects
        .iter()
        .filter(|object| object.zone == Zone::Battlefield && object.card == RAT)
        .count()
}

#[test]
fn death_reads_total_actual_damage_to_the_dead_incarnation() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Bear Cub").unwrap(), &RECIPIENT),
        (Card::from_name("Giant Growth").unwrap(), &BLINK),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let exact = game.object_ref(creature);
        for _ in 0..2 {
            let shock = put(&mut game, player, "Shock", Zone::Hand);
            cast(&mut game, shock, Some(Target::Card(creature)));
            settle(&mut game);
        }
        assert_eq!(
            game.object_view(creature).damage,
            2,
            "the second hit healed prior marks"
        );
        for seat in [Player::P0, Player::P1] {
            assert_eq!(
                game.observe(seat)
                    .permanent_damage_history
                    .iter()
                    .filter(|row| row.recipient == exact)
                    .map(|row| row.amount)
                    .sum::<u128>(),
                4
            );
        }

        let kill = put(&mut game, player, "Murder", Zone::Hand);
        let blink = put(&mut game, player, "Giant Growth", Zone::Hand);
        let mut fresh = game.clone();
        cast(&mut fresh, blink, Some(Target::Card(creature)));
        settle(&mut fresh);
        assert_ne!(fresh.object_ref(creature), exact);
        cast(&mut fresh, kill, Some(Target::Card(creature)));
        settle(&mut fresh);
        assert_eq!(rats(&fresh), 0);

        cast(&mut game, kill, Some(Target::Card(creature)));
        settle(&mut game);
        assert_eq!(rats(&game), 4);
    }
}
