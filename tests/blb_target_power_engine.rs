//! A resolution branch can compare a legal target's effective power.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CONTROL_IF_SMALL_OR_LIZARD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::IfAmountAtLeast {
                value: AmountValue::TargetCreaturePower,
                minimum: 3,
                skip: 1,
            },
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: Some("Lizard"),
                        basic: false,
                        other: true,
                        cards_only: false,
                    }),
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::GainControlUntilEndTurn,
        ],
    ),
    ..CardRules::EMPTY
};

fn cast_control(game: &mut Game, player: Player, target: CardId) {
    let spell = put(game, player, "Opt", Zone::Hand);
    cast(game, spell, Some(Target::Card(target)));
}

#[test]
fn a_conditional_target_power_read_uses_the_target_at_resolution() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &CONTROL_IF_SMALL_OR_LIZARD)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let small = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let large = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);

        let mut high = game.clone();
        cast_control(&mut high, player, large);
        settle(&mut high);
        assert_eq!(high.object_view(large).controller, opponent);

        let mut lizard = game.clone();
        put(&mut lizard, player, "Flamecache Gecko", Zone::Battlefield);
        cast_control(&mut lizard, player, large);
        settle(&mut lizard);
        assert_eq!(lizard.object_view(large).controller, player);

        cast_control(&mut game, player, small);
        let mut boosted = game.clone();
        if boosted.actor() != Some(opponent) {
            act(&mut boosted, Action::Pass);
        }
        let growth = put(&mut boosted, opponent, "Giant Growth", Zone::Hand);
        cast(&mut boosted, growth, Some(Target::Card(small)));
        settle(&mut boosted);
        assert_eq!(boosted.object_view(small).power, 5);
        assert_eq!(boosted.object_view(small).controller, opponent);

        settle(&mut game);
        assert_eq!(game.object_view(small).controller, player);
    }
}
