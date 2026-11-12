//! A targeted control batch remembers only actual changes of controller.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CONTROL_BATCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            }),
            TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: usize::MAX,
            },
        ]),
        &[
            Instruction::GainControlRememberCount {
                subject: Subject::Targets,
                controller: PlayerSet::Targeted,
            },
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::MovedCount),
        ],
    ),
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OpponentTakesPermanent,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn give_away_two_permanents_together_and_draw_only_for_live_exact_gains() {
    let spell = Card::from_name("Opt").unwrap();
    let observer = Card::from_name("Interface Ace").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &CONTROL_BATCH), (observer, &OBSERVER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let observer = spawn(&mut game, player, observer, Zone::Battlefield);
    let equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let stale = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let spell = spawn(&mut game, player, spell, Zone::Hand);
    announce(
        &mut game,
        spell,
        &[
            Target::Player(opponent),
            Target::Card(observer),
            Target::Card(equipment),
            Target::Card(stale),
        ],
    );

    let mut invalid_player = game.clone();
    invalid_player.state.players[opponent.index()].hexproof_until_end_turn = true;
    settle(&mut invalid_player);
    assert_eq!(invalid_player.object_view(observer).controller, player);
    assert_eq!(invalid_player.object_view(equipment).controller, player);
    assert_eq!(invalid_player.hand_count(player), 0);

    game.move_object(stale, Zone::Exile);
    game.move_object(stale, Zone::Battlefield);
    let new = game.object_ref(stale);
    act(&mut game, Action::Pass);
    let root = game.snapshot();
    let pass = game
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(opponent, game.revision(), pass)
        .unwrap();
    act(&mut game, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&root).unwrap();
    assert_eq!(indexed.snapshot(), root);

    assert_eq!(game.object_view(observer).controller, opponent);
    assert_eq!(game.object_view(equipment).controller, opponent);
    assert_eq!(game.object_ref(stale), new);
    assert_eq!(game.object_view(stale).controller, player);
    assert_eq!(game.hand_count(player), 2);
    assert_eq!(game.state.pending, Pending::OrderTriggers { player });
    assert_eq!(game.state.ordering_triggers.len(), 2);
    assert!(
        game.state
            .ordering_triggers
            .iter()
            .all(|trigger| trigger.controller == player)
    );
}
