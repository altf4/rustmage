//! A positive division can have zero targets when its target plan allows it.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const FAMILIAR: Card = Card::from_catalogue_id(1630);
const OPTIONAL_FIXED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::creature_you_control(),
            min: 0,
            max: 3,
        }),
        &[Instruction::DistributeCounters {
            total: 3,
            kind: CounterKind::PlusOne,
        }],
    ),
    extras: &CardExtras {
        adventure_parent: Some(FAMILIAR),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn red_payment(game: &mut Game) {
    act(
        game,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 0, 0],
        },
    );
}

#[test]
fn fixed_optional_division_keeps_empty_and_invalid_target_packets_distinct() {
    let inset = Card::from_name("Blow Off Steam").unwrap();
    let _rules = fixture_rules::activate(&[(inset, &OPTIONAL_FIXED)]);
    for player in [Player::P0, Player::P1] {
        let mut empty = scenario(player);
        let source = put(&mut empty, player, "Frolicking Familiar", Zone::Hand);
        act(&mut empty, Action::AnnounceAdventure { card: source });
        assert_eq!(empty.state.pending, Pending::PayCosts { player });
        red_payment(&mut empty);
        assert!(empty.state.stack.last().unwrap().targets.is_empty());
        settle(&mut empty);
        assert_eq!(empty.object_view(source).zone, Zone::Exile);
        assert!(
            empty
                .legal_actions()
                .contains(&Action::AnnounceExile { card: source })
        );

        let mut game = scenario(player);
        let source = put(&mut game, player, "Frolicking Familiar", Zone::Hand);
        let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let bounce_first = put(&mut game, player, "Unsummon", Zone::Hand);
        let bounce_second = put(&mut game, player, "Unsummon", Zone::Hand);
        act(&mut game, Action::AnnounceAdventure { card: source });
        assert_eq!(game.state.pending, Pending::Targets { player });
        let actions = game.legal_actions();
        assert!(actions.contains(&target(first)));
        let finish = actions
            .iter()
            .position(|action| *action == Action::FinishTargets)
            .unwrap();
        let checkpoint = game.snapshot();
        let mut declined = game.clone();
        let mut indexed = game.clone();
        act(&mut declined, Action::FinishTargets);
        indexed
            .apply_index(player, game.revision(), finish)
            .unwrap();
        assert_eq!(declined.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, Action::FinishTargets);
        assert_eq!(declined.snapshot(), indexed.snapshot());
        red_payment(&mut declined);
        assert!(declined.state.stack.last().unwrap().targets.is_empty());
        settle(&mut declined);
        assert_eq!(declined.object_view(source).zone, Zone::Exile);
        assert_eq!(declined.counter_count(first, CounterKind::PlusOne), 0);
        assert_eq!(declined.counter_count(second, CounterKind::PlusOne), 0);

        reject(&mut game, target(foreign));
        act(&mut game, target(first));
        act(&mut game, target(second));
        act(&mut game, Action::FinishTargets);
        reject(&mut game, Action::AllocateTargetAmount { amount: 0 });
        reject(&mut game, Action::AllocateTargetAmount { amount: 3 });
        act(&mut game, Action::AllocateTargetAmount { amount: 1 });
        act(&mut game, Action::AllocateTargetAmount { amount: 2 });
        red_payment(&mut game);
        cast(&mut game, bounce_first, Some(Target::Card(first)));

        let mut all_invalid = game.clone();
        cast(&mut all_invalid, bounce_second, Some(Target::Card(second)));
        settle(&mut all_invalid);
        assert_eq!(all_invalid.object_view(first).zone, Zone::Hand);
        assert_eq!(all_invalid.object_view(second).zone, Zone::Hand);
        assert_eq!(all_invalid.object_view(source).zone, Zone::Graveyard);
        reject(&mut all_invalid, Action::AnnounceExile { card: source });

        settle(&mut game);
        assert_eq!(game.object_view(first).zone, Zone::Hand);
        assert_eq!(game.object_view(source).zone, Zone::Exile);
        assert_eq!(game.counter_count(second, CounterKind::PlusOne), 2);
        assert_eq!(game.counter_count(foreign, CounterKind::PlusOne), 0);
    }
}

#[test]
fn groves_bounty_accepts_zero_targets_with_paid_positive_x() {
    let player = Player::P0;
    for has_recipient in [false, true] {
        let mut game = scenario(player);
        let recipient =
            has_recipient.then(|| put(&mut game, player, "Bear Cub", Zone::Battlefield));
        let otter = put(
            &mut game,
            player,
            "Elusive Otter // Grove's Bounty",
            Zone::Hand,
        );
        act(&mut game, Action::AnnounceAdventure { card: otter });
        act(&mut game, Action::ChooseX { value: 2 });
        if let Some(recipient) = recipient {
            assert_eq!(game.state.pending, Pending::Targets { player });
            assert!(game.legal_actions().contains(&target(recipient)));
            act(&mut game, Action::FinishTargets);
        } else {
            assert_eq!(game.state.pending, Pending::PayCosts { player });
        }
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 1, 2],
            },
        );
        let spell = game.state.stack.last().unwrap();
        assert_eq!(spell.x, Some(2));
        assert!(spell.targets.is_empty());
        settle(&mut game);
        assert_eq!(game.object_view(otter).zone, Zone::Exile);
        if let Some(recipient) = recipient {
            assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 0);
        }
    }
}

#[test]
fn a_required_division_still_requires_its_first_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bolt = put(&mut game, player, "Twin Bolt", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: bolt });
    reject(&mut game, Action::FinishTargets);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    act(&mut game, Action::FinishTargets);
    act(&mut game, Action::AllocateTargetAmount { amount: 2 });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 18);
}
