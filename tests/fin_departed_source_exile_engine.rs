mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TREASURE: Card = Card::from_catalogue_id(5302);
const DIES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ExileDepartedSource,
            Instruction::CreateTokens {
                token: TREASURE,
                amount: 10,
                tapped: true,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const REINCARNATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::ReturnToHand,
            Instruction::Discard(PlayerSet::Opponent, 1),
        ],
    ),
    ..CardRules::EMPTY
};

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn exile_moves_only_the_captured_death_incarnation_and_tokens_are_unconditional() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &DIES),
        (Card::from_name("Opt").unwrap(), &REINCARNATE),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    for reincarnated in [false, true] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let murder = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, murder, Some(Target::Card(source)));
        pass_priority(&mut game);
        assert_eq!(game.object_view(source).zone, Zone::Graveyard);
        let departed = game.object_ref(source);
        assert_eq!(
            game.state
                .stack
                .last()
                .unwrap()
                .source_move
                .as_ref()
                .unwrap()
                .after,
            departed
        );

        if reincarnated {
            act(&mut game, Action::Pass);
            let response = put(&mut game, opponent, "Opt", Zone::Hand);
            cast(&mut game, response, Some(Target::Card(source)));
            pass_priority(&mut game);
            act(&mut game, Action::Discard { card: source });
            assert_eq!(game.object_view(source).zone, Zone::Graveyard);
            assert_ne!(game.object_ref(source), departed);
        }

        settle(&mut game);
        assert_eq!(
            game.object_view(source).zone,
            if reincarnated {
                Zone::Graveyard
            } else {
                Zone::Exile
            }
        );
        let treasures: Vec<_> = game
            .state
            .objects
            .iter()
            .filter(|object| object.card == TREASURE && object.zone == Zone::Battlefield)
            .collect();
        assert_eq!(treasures.len(), 10);
        assert!(
            treasures
                .iter()
                .all(|object| object.controller == player && object.tapped)
        );
    }
}
