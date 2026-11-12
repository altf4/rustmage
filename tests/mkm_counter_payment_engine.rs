//! An optional counter placement cost on a target is distinct from its reward.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const LOCK: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[
            Instruction::Tap,
            Instruction::IfSubjectCanReceiveCounters {
                subject: Subject::Targets,
                skip: 3,
            },
            Instruction::May { skip: 2 },
            Instruction::PaySubjectCounterPlacement {
                subject: Subject::Targets,
                player: PlayerSet::Controller,
                kind: CounterKind::Stun,
                amount: 1,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Stun,
                amount: 1,
            },
        ],
        sorcery: false,
        once_per_turn: false,
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

fn activate(game: &mut Game, source: CardId, card: CardId) {
    let target = Target::Card(card);
    let action = Action::Activate {
        card: source,
        ability: 0,
        target: Some(target),
        payment: [0; 6],
        counter_payment: Counters::default(),
    };
    if game.legal_actions().contains(&action) {
        act(game, action);
    } else {
        choose(
            game,
            |action| matches!(action, Action::AnnounceActivate { card, ability: 0, .. } if *card == source),
        );
        act(game, Action::ChooseTarget { target });
        pay(game);
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn a_stun_counter_cost_and_its_source_effect_use_separate_events() {
    let source = Card::from_name("Grizzly Bears").unwrap();
    let prohibition = Card::from_name("Authority of the Consuls").unwrap();
    let _rules = fixture_rules::activate(&[(source, &LOCK), (prohibition, &NO_COUNTERS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let wall = spawn(&mut game, player, source, Zone::Battlefield);
    let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    activate(&mut game, wall, target);
    let mut gone = game.clone();
    pass_pair(&mut game);
    assert!(game.object_view(target).tapped);
    assert!(matches!(game.state.pending, Pending::EffectOptional { player: p, .. } if p == player));

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.counter_count(target, CounterKind::Stun), 0);
    assert_eq!(declined.counter_count(wall, CounterKind::Stun), 0);

    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.counter_count(target, CounterKind::Stun), 1);
    assert_eq!(game.counter_count(wall, CounterKind::Stun), 2);

    gone.move_object(wall, Zone::Exile);
    gone.move_object(wall, Zone::Battlefield);
    pass_pair(&mut gone);
    act(&mut gone, Action::ChooseOptional { accept: true });
    assert_eq!(gone.counter_count(target, CounterKind::Stun), 1);
    assert_eq!(gone.counter_count(wall, CounterKind::Stun), 0);

    let mut blocked = scenario(player);
    spawn(&mut blocked, player, prohibition, Zone::Battlefield);
    let wall = spawn(&mut blocked, player, source, Zone::Battlefield);
    let target = put(&mut blocked, player.other(), "Bear Cub", Zone::Battlefield);
    activate(&mut blocked, wall, target);
    pass_pair(&mut blocked);
    assert!(blocked.object_view(target).tapped);
    assert_eq!(blocked.state.pending, Pending::Priority);
    assert_eq!(blocked.counter_count(target, CounterKind::Stun), 0);
    assert_eq!(blocked.counter_count(wall, CounterKind::Stun), 0);
}
