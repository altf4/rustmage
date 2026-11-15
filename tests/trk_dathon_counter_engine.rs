//! Choosing a live existing counter kind on an exact enchantment target.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CHOOSE_EXISTING: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 32,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })]),
        program: &[Instruction::ChooseTargetExistingCounterKindToPut],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: true,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn target_choice_offers_only_live_kinds_and_owned_branch_restores() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &CHOOSE_EXISTING)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let target = put(&mut game, Player::P0, "Rest in Peace", Zone::Battlefield);
    let artifact = put(&mut game, Player::P0, "Short Bow", Zone::Battlefield);
    let opposing = put(&mut game, Player::P1, "Rest in Peace", Zone::Battlefield);
    game.state.objects[target as usize].counters.lore = 1;
    game.state.objects[target as usize].counters.stun = 2;
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    for _ in 0..8 {
        if matches!(
            game.state.pending,
            Pending::TargetExistingCounterKindToPut { .. }
        ) {
            break;
        }
        choose(&mut game, |action| matches!(action, Action::Pass));
    }
    assert!(matches!(
        game.state.pending,
        Pending::TargetExistingCounterKindToPut { .. }
    ));
    let choices = game.legal_actions();
    assert_eq!(
        choices
            .iter()
            .filter(|action| matches!(action, Action::ChooseTargetExistingCounterKindToPut { .. }))
            .count(),
        2
    );
    assert!(
        choices.contains(&Action::ChooseTargetExistingCounterKindToPut {
            counter: CounterKind::Lore
        })
    );
    assert!(
        choices.contains(&Action::ChooseTargetExistingCounterKindToPut {
            counter: CounterKind::Stun
        })
    );
    reject(
        &mut game,
        Action::ChooseTargetExistingCounterKindToPut {
            counter: CounterKind::Charge,
        },
    );

    let saved = game.snapshot();
    let revision = game.revision();
    let selected = Action::ChooseTargetExistingCounterKindToPut {
        counter: CounterKind::Stun,
    };
    let index = choices
        .iter()
        .position(|action| *action == selected)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, selected);
    indexed.apply_index(Player::P0, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.counter_count(target, CounterKind::Lore), 1);
    assert_eq!(game.counter_count(target, CounterKind::Stun), 3);
    game.restore(&saved).unwrap();
    assert_eq!(game.counter_count(target, CounterKind::Stun), 2);
    assert!(matches!(
        game.state.pending,
        Pending::TargetExistingCounterKindToPut { .. }
    ));
}

#[test]
fn no_counter_or_departed_target_never_opens_a_choice() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &CHOOSE_EXISTING)]);
    for depart in [false, true] {
        let mut game = scenario(Player::P0);
        let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let target = put(&mut game, Player::P0, "Rest in Peace", Zone::Battlefield);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        if depart {
            game.move_object(target, Zone::Graveyard);
        }
        settle(&mut game);
        assert_eq!(game.counter_count(target, CounterKind::Lore), 0);
        assert_eq!(game.counter_count(target, CounterKind::Stun), 0);
        assert_eq!(game.state.pending, Pending::Priority);
    }
}
