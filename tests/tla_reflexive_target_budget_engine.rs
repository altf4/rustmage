//! A paid X binds a distinct reflexive trigger's combined target budget.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const REANIMATOR: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PayXReflexive { ability: 1 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::DamagedPlayerObject {
                        zone: TargetZone::Graveyard,
                        kinds: 1,
                    },
                    constraint: TargetGroupConstraint::TotalManaValueAtMostX,
                },
                min: 0,
                max: usize::MAX,
            }),
            program: &[Instruction::ReturnToBattlefieldUnderControl],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddCombatRed(2)],
        },
    ],
    ..CardRules::EMPTY
};
const ZERO_BODY: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn choose_budget(game: &mut Game, value: u32) {
    act(game, Action::ChooseX { value });
    act(
        game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, value, 0, 0],
        },
    );
}

#[test]
fn paid_x_limits_the_exact_damaged_players_graveyard_and_allows_zero_value_cards() {
    let zero_body = Card::from_name("Marketback Walker").unwrap();
    let _rules =
        fixture_rules::activate(&[(Card::GoblinPiker, &REANIMATOR), (zero_body, &ZERO_BODY)]);
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        let source = put(&mut game, actor, "Goblin Piker", Zone::Battlefield);
        let first = put(&mut game, actor.other(), "Bear Cub", Zone::Graveyard);
        let second = put(&mut game, actor.other(), "Bear Cub", Zone::Graveyard);
        let zero = spawn(&mut game, actor.other(), zero_body, Zone::Graveyard);
        let noncreature = put(&mut game, actor.other(), "Mox Jasper", Zone::Graveyard);
        let own = spawn(&mut game, actor, zero_body, Zone::Graveyard);
        game.state.players[actor.index()].mana = [0; 6];
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: source });
        act(&mut game, Action::FinishAttackers);
        settle(&mut game);
        assert_eq!(game.state.players[actor.index()].combat_red_mana, 2);
        pass_pair(&mut game);
        act(&mut game, Action::FinishBlockers);
        pass_pair(&mut game);
        pass_pair(&mut game);
        assert_eq!(game.state.pending, Pending::ResolutionX { player: actor });
        reject(&mut game, Action::ChooseX { value: 3 });

        let mut zero_only = game.clone();
        choose_budget(&mut zero_only, 0);
        for illegal in [first, own, noncreature] {
            reject(&mut zero_only, target(illegal));
        }
        act(&mut zero_only, target(zero));
        act(&mut zero_only, Action::FinishTargets);
        settle(&mut zero_only);
        assert_eq!(zero_only.object_view(zero).zone, Zone::Battlefield);
        assert_eq!(zero_only.object_view(zero).controller, actor);

        choose_budget(&mut game, 2);
        act(&mut game, target(first));
        reject(&mut game, target(second));
        reject(&mut game, target(own));
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let action = target(zero);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        indexed
            .apply_index(actor, indexed.revision(), index)
            .unwrap();
        act(&mut game, action);
        assert_eq!(indexed.snapshot(), game.snapshot());
        act(&mut game, Action::FinishTargets);
        game.move_object(first, Zone::Exile);
        settle(&mut game);
        assert_eq!(game.object_view(first).zone, Zone::Exile);
        assert_eq!(game.object_view(zero).zone, Zone::Battlefield);
        assert_eq!(game.object_view(zero).controller, actor);

        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, Action::FinishTargets);
        settle(&mut indexed);
        assert_eq!(indexed.object_view(first).zone, Zone::Battlefield);
        assert_eq!(indexed.object_view(first).controller, actor);
        assert_eq!(indexed.object_view(zero).zone, Zone::Graveyard);
    }
}
