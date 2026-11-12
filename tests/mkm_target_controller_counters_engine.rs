//! An exact-X entry trigger samples each surviving target's live controller.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const ENTRANT: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: usize::MAX,
        }),
        program: &[
            Instruction::TargetsExactlyX,
            Instruction::Tap,
            Instruction::PutCountersOnTargetsByController {
                relation: CastRelation::Opponent,
                kind: CounterKind::Stun,
                amount: 1,
            },
        ],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn begin(game: &mut Game, player: Player, card: Card, x: u32) {
    let spell = spawn(game, player, card, Zone::Hand);
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseX { value: x });
    pay(game);
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

#[test]
fn all_targets_tap_and_only_current_opposing_targets_receive_stun_counters() {
    let source = Card::from_name("Authority of the Consuls").unwrap();
    let _rules = fixture_rules::activate(&[(source, &ENTRANT)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let ours = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let theirs = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let bystander = put(&mut game, opponent, "Healer's Hawk", Zone::Battlefield);
    game.state.objects[theirs as usize].tapped = true;
    begin(&mut game, player, source, 2);
    reject(&mut game, Action::FinishTargets);
    act(&mut game, target(ours));
    reject(&mut game, target(ours));
    act(&mut game, target(theirs));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }

    let mut changed = game.clone();
    let ours_ref = changed.object_ref(ours);
    let theirs_ref = changed.object_ref(theirs);
    changed.record_control(ours_ref, opponent, None);
    changed.record_control(theirs_ref, player, None);
    settle(&mut changed);
    assert!(changed.object_view(ours).tapped && changed.object_view(theirs).tapped);
    assert_eq!(changed.counter_count(ours, CounterKind::Stun), 1);
    assert_eq!(changed.counter_count(theirs, CounterKind::Stun), 0);
    assert!(!changed.object_view(bystander).tapped);

    settle(&mut game);
    assert!(game.object_view(ours).tapped && game.object_view(theirs).tapped);
    assert_eq!(game.counter_count(ours, CounterKind::Stun), 0);
    assert_eq!(game.counter_count(theirs, CounterKind::Stun), 1);
}

#[test]
fn insufficient_creatures_prevents_the_trigger_from_acquiring_targets() {
    let source = Card::from_name("Authority of the Consuls").unwrap();
    let _rules = fixture_rules::activate(&[(source, &ENTRANT)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let only = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    begin(&mut game, player, source, 2);
    settle(&mut game);
    assert!(!game.object_view(only).tapped);
    assert_eq!(game.counter_count(only, CounterKind::Stun), 0);
}
