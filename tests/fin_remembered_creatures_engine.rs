//! A temporary grant follows the exact counter recipients, even if prevented.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WINGS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCreatureCountersAndRemember {
                filter: CreatureFilter {
                    relation: 0,
                    other: true,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::GrantRememberedCreatures(Keywords::FLYING),
        ],
    }],
    ..CardRules::EMPTY
};

const NO_COUNTERS: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(1)],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn remembered_recipients_gain_flying_for_this_turn_even_without_a_counter() {
    let boots = Card::from_name("Swiftfoot Boots").unwrap();
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &WINGS), (boots, &NO_COUNTERS)]);
    let player = Player::P0;
    for prevented in [false, true] {
        let mut game = scenario(player);
        let bearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let renewed = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        let foreign = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        if prevented {
            spawn(&mut game, player.other(), boots, Zone::Battlefield);
        }
        let source = spawn(&mut game, player, Card::HillGiant, Zone::Hand);
        cast(&mut game, source, None);
        pass_pair(&mut game);
        act(&mut game, Action::Pass);
        let mut indexed = game.clone();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == Action::Pass)
            .unwrap();
        let revision = game.revision();
        act(&mut game, Action::Pass);
        indexed
            .apply_index(player.other(), revision, index)
            .unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());

        for creature in [bearer, renewed] {
            assert!(game.keywords(creature).contains(Keywords::FLYING));
            assert_eq!(
                game.object_view(creature).counters.plus_one,
                u64::from(!prevented)
            );
        }
        for creature in [foreign, source] {
            assert!(!game.keywords(creature).contains(Keywords::FLYING));
            assert_eq!(game.object_view(creature).counters.plus_one, 0);
        }

        game.move_object(renewed, Zone::Exile);
        game.move_object(renewed, Zone::Battlefield);
        assert!(!game.keywords(renewed).contains(Keywords::FLYING));
        game.state.step = Step::End;
        game.state.priority = player;
        pass_pair(&mut game);
        assert_eq!(game.state.active, player.other());
        assert!(!game.keywords(bearer).contains(Keywords::FLYING));
    }
}
