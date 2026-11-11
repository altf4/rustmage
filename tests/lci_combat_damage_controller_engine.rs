//! The damaged controller observes one actual combat batch and captures its attacker.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TREASURE: Card = Card(5302);
const BALL: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CombatDamageToController,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GainControl {
                subject: Subject::Source,
                controller: PlayerSet::EventPlayer,
            },
            Instruction::UntapSource,
        ],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Point,
                amount: 1,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::SourceCounterQuantity(CounterKind::Point),
                minimum: 5,
                skip: 2,
            },
            Instruction::SacrificeSource,
            Instruction::CreateTokens {
                token: TREASURE,
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn one_defensive_combat_packet_transfers_and_untaps_only_its_exact_source() {
    let definition = Card::from_name("Contested Game Ball").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &BALL)]);
    let attacker = Player::P0;
    let defender = attacker.other();
    let mut game = scenario(attacker);
    let ball = spawn(&mut game, defender, definition, Zone::Battlefield);
    let friendly = spawn(&mut game, attacker, definition, Zone::Battlefield);
    game.state.objects[ball as usize].tapped = true;
    game.state.objects[friendly as usize].tapped = true;
    let first = put(&mut game, attacker, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, attacker, "Grizzly Bears", Zone::Battlefield);
    game.apply_damage_batch(&[DamageEvent {
        preventable: true,
        source: game.damage_source(first),
        target: Target::Player(defender),
        amount: 1,
        combat: false,
    }]);
    assert!(game.state.pending_triggers.is_empty());

    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for card in [first, second] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    let packet = &game.state.stack[0];
    assert_eq!(packet.controller, defender);
    assert_eq!(packet.event_player, Some(attacker));
    assert_eq!(packet.source.unwrap().object, game.object_ref(ball));
    assert!(packet.targets.is_empty());
    assert_eq!(packet.damage_event.unwrap().amount, 4);
    assert!(packet.damage_event.unwrap().combat);

    let mut stale = game.clone();
    stale.move_object(ball, Zone::Graveyard);
    stale.move_object(ball, Zone::Battlefield);
    stale.state.objects[ball as usize].tapped = true;
    settle(&mut stale);
    assert_eq!(stale.object_view(ball).controller, defender);
    assert!(stale.object_view(ball).tapped);

    settle(&mut game);
    assert_eq!(game.object_view(ball).owner, defender);
    assert_eq!(game.object_view(ball).controller, attacker);
    assert!(!game.object_view(ball).tapped);
    assert!(game.object_view(friendly).tapped);
    game.add_counter(ball, CounterKind::Point, 4);
    game.state.players[attacker.index()].mana = [0, 0, 0, 0, 0, 2];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: ball,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(ball).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(attacker), 1);
    assert_eq!(
        game.state
            .objects
            .iter()
            .filter(|object| {
                object.card == TREASURE
                    && object.zone == Zone::Battlefield
                    && object.controller == attacker
            })
            .count(),
        1
    );
}
