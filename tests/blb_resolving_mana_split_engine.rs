//! A resolving split choice uses the current quantity and the specified colors.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SPLITTER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddManaSplitScaled {
            first: ChoiceColor::Red,
            second: ChoiceColor::Green,
            amount: AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Raccoon"),
                basic: false,
                other: false,
                cards_only: false,
            })),
        }],
    }],
    ..CardRules::EMPTY
};

fn choose_rg(red: u32, green: u32) -> Action {
    Action::ChooseMana {
        production: [0, 0, 0, red, green, 0],
    }
}

#[test]
fn resolving_two_color_mana_counts_only_current_controlled_matching_permanents() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SPLITTER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let first = put(&mut game, player, "Scrapshooter", Zone::Battlefield);
    put(&mut game, player, "Scrapshooter", Zone::Battlefield);
    put(&mut game, player.other(), "Scrapshooter", Zone::Battlefield);
    game.state.players[player.index()].mana = [0; 6];
    game.begin_step(Step::Main1);
    game.give_priority();
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    let mut changed = game.clone();
    changed.move_object(first, Zone::Exile);
    for branch in [&mut game, &mut changed] {
        act(branch, Action::Pass);
        act(branch, Action::Pass);
    }
    assert_eq!(game.mana_split_choice().unwrap().total, 2);
    assert_eq!(changed.mana_split_choice().unwrap().total, 1);

    reject(&mut game, choose_rg(2, 1));
    reject(&mut changed, choose_rg(1, 1));
    for (red, green) in [(2, 0), (1, 1), (0, 2)] {
        let mut branch = game.clone();
        act(&mut branch, choose_rg(red, green));
        assert_eq!(
            branch.state.players[player.index()].mana,
            [0, 0, 0, red, green, 0]
        );
    }
    act(&mut changed, choose_rg(1, 0));
    assert_eq!(
        changed.state.players[player.index()].mana,
        [0, 0, 0, 1, 0, 0]
    );
}
