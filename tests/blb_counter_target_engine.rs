//! Dynamic target legality uses an exact source's live or last-known counters.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WELL: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutSourceCounterReflexive {
            kind: CounterKind::Coin,
            amount: 1,
            ability: 0,
        }],
        origin: ActivationOrigin::Battlefield,
        sorcery: true,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::ManaValueExactly {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 4 | 8,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            amount: AmountValue::SourceCounterQuantity(CounterKind::Coin),
            value: None,
        })),
        program: &[Instruction::CastTargetedGraveyardFree],
    }],
    ..CardRules::EMPTY
};

#[test]
fn target_uses_live_coin_count_and_retains_its_exact_source_last_known_count() {
    let definition = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &WELL)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let well = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let one = put(&mut game, player, "Opt", Zone::Graveyard);
    let three = put(&mut game, player, "Divination", Zone::Graveyard);
    let other = put(&mut game, player.other(), "Opt", Zone::Graveyard);
    let creature = put(&mut game, player, "Healer's Hawk", Zone::Graveyard);
    let target = |card| Action::ChooseTarget {
        target: Target::Card(card),
    };
    choose(
        &mut game,
        |action| matches!(action, Action::Activate { card, ability: 0, .. } | Action::AnnounceActivate { card, ability: 0, .. } if *card == well),
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    for _ in 0..12 {
        if game.legal_actions().contains(&target(one)) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.counter_count(well, CounterKind::Coin), 1);
    for excluded in [three, other, creature] {
        reject(&mut game, target(excluded));
    }
    act(&mut game, target(one));

    let mut changed = game.clone();
    changed.add_counter(well, CounterKind::Coin, 1);
    settle(&mut changed);
    assert_eq!(changed.object_view(one).zone, Zone::Graveyard);

    let before = game.object_ref(well);
    game.move_object(well, Zone::Exile);
    game.move_object(well, Zone::Battlefield);
    assert_ne!(game.object_ref(well), before);
    assert_eq!(game.counter_count(well, CounterKind::Coin), 0);
    for _ in 0..12 {
        if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
            break;
        }
        act(&mut game, Action::Pass);
    }
    let cast = Action::CastExiledDuringResolution {
        object: game.object_ref(one),
        face: CastFace::Normal,
    };
    assert!(game.legal_actions().contains(&cast));
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
}
