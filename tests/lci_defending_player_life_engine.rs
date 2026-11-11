//! Defender identity and both life comparisons are sampled at attack declaration.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PREACHER: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacksPlayerWithMostLife,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::EventPlayer, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::Expression {
                    value: StatValue::ControllerHasMostLife,
                    minimum: Some(1),
                    maximum: None,
                },
                event: &TriggerCondition::SelfAttacks,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::LoseLife(PlayerSet::Controller, 1),
            ],
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn attack_life_qualifications_distinguish_players_from_planeswalkers_and_battles() {
    let definition = Card::from_name("Preacher of the Schism").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &PREACHER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let source = spawn(&mut game, player, definition, Zone::Battlefield);
        let walker = put(
            &mut game,
            opponent,
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        game.add_counter(walker, CounterKind::Loyalty, 4);
        let siege = Card::from_name("@fixture/siege-front").unwrap();
        let battle = spawn(&mut game, player, siege, Zone::Battlefield);
        game.add_counter(battle, CounterKind::Defense, 9);
        game.set_battle_protector(game.object_ref(battle), opponent);
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        let attack_player = Action::Attack { card: source };
        let attack_walker = Action::AttackPlaneswalker {
            card: source,
            defender: game.object_ref(walker),
        };
        let attack_battle = Action::AttackBattle {
            card: source,
            defender: game.object_ref(battle),
        };
        for (attack, at_declaration, expected, after_declaration) in [
            (attack_player.clone(), [20, 20], [true, true], [30, 10]),
            (attack_player.clone(), [15, 20], [true, false], [30, 10]),
            (attack_player, [25, 20], [false, true], [10, 30]),
            (attack_walker, [20, 20], [false, true], [10, 30]),
            (attack_battle, [20, 20], [false, true], [10, 30]),
        ] {
            let mut branch = game.clone();
            branch.state.players[player.index()].life = at_declaration[0];
            branch.state.players[opponent.index()].life = at_declaration[1];
            act(&mut branch, attack);
            act(&mut branch, Action::FinishAttackers);
            if matches!(branch.state.pending, Pending::OrderTriggers { .. }) {
                choose(&mut branch, |action| {
                    matches!(action, Action::PutTrigger { .. })
                });
            }
            let mut seen = [false; 2];
            for packet in &branch.state.stack {
                let StackKind::TriggeredAbility { ability } = packet.kind else {
                    panic!("unexpected attack packet");
                };
                seen[usize::from(ability)] = true;
                if ability == 0 {
                    assert_eq!(packet.event_player, Some(opponent));
                }
            }
            assert_eq!(seen, expected);
            branch.state.players[player.index()].life = after_declaration[0];
            branch.state.players[opponent.index()].life = after_declaration[1];
            settle(&mut branch);
            assert_eq!(branch.hand_count(player), usize::from(expected[1]));
            assert_eq!(
                branch.state.players[player.index()].life,
                after_declaration[0] - i128::from(expected[1])
            );
            assert_eq!(
                branch.state.players[opponent.index()].life,
                after_declaration[1] + 2 * i128::from(expected[0])
            );
        }
    }
}
