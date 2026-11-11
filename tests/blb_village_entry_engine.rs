//! Historical subtype guards and live counters on exact current entrants.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const START: ActivatedAbility = ActivatedAbility {
    guard: ActivationGuard::Always,
    cost: ActivationCost::ZERO,
    program: &[],
    targets: TargetPlan::NONE,
    origin: ActivationOrigin::Battlefield,
    sorcery: false,
    cycling: false,
    exhaust: false,
    once_per_turn: false,
};

const ENTRY_GUARD: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::SubtypeEnteredUnderYourControl(&["Bird", "Frog", "Otter", "Rat"]),
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        ..START
    }],
    ..CardRules::EMPTY
};

const COUNTER_GROUP: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        program: &[Instruction::ChangeNewlyEnteredPermanentGroupCounters {
            filters: &[
                ControlledPermanentFilter {
                    subtype: Some("Frog"),
                    ..ControlledPermanentFilter::ANY
                },
                ControlledPermanentFilter {
                    subtype: Some("Squirrel"),
                    ..ControlledPermanentFilter::ANY
                },
            ],
            effect: PermanentGroupCounters::Add {
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        }],
        ..START
    }],
    ..CardRules::EMPTY
};

fn available(game: &Game, id: CardId) -> Option<Action> {
    game.legal_actions().into_iter().find(|action| {
        matches!(action, Action::Activate { card, ability: 0, .. } if *card == id)
            || matches!(action, Action::AnnounceActivate { card, ability: 0, .. } if *card == id)
    })
}

#[test]
fn historical_subtype_and_controller_stay_fixed_after_the_creature_leaves() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &ENTRY_GUARD)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let village = put(&mut game, player, "Short Bow", Zone::Battlefield);
        assert!(available(&game, village).is_none());
        let enemy_bird = put(&mut game, player.other(), "Healer's Hawk", Zone::Hand);
        game.move_object(enemy_bird, Zone::Battlefield);
        game.record_control(game.object_ref(enemy_bird), player, None);
        assert!(available(&game, village).is_none());

        let changeling = put(&mut game, player, "Three Tree Mascot", Zone::Hand);
        let before = game.snapshot();
        game.move_object(changeling, Zone::Battlefield);
        game.move_object(changeling, Zone::Exile);
        let action = available(&game, village).unwrap();
        let index = game
            .legal_actions()
            .iter()
            .position(|row| *row == action)
            .unwrap();
        let mut indexed = game.clone();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&before).unwrap();
        assert!(available(&indexed, village).is_none());

        game.begin_turn();
        assert!(!game.subtype_entered_under_control(player, &["Bird", "Frog"]));
    }
}

#[test]
fn counter_union_uses_current_control_and_one_counter_per_exact_new_permanent() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &COUNTER_GROUP)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let village = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let old = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    game.begin_turn();
    game.state.pending = Pending::Priority;
    game.state.step = Step::Main1;
    game.state.priority = player;
    let own = put(&mut game, player, "Three Tree Mascot", Zone::Hand);
    let other = put(&mut game, player.other(), "Three Tree Mascot", Zone::Hand);
    let stolen = put(&mut game, player.other(), "Three Tree Mascot", Zone::Hand);
    let wrong_type = put(&mut game, player, "Bear Cub", Zone::Hand);
    for card in [own, other, stolen, wrong_type] {
        game.move_object(card, Zone::Battlefield);
    }
    game.record_control(game.object_ref(stolen), player, None);
    let action = available(&game, village).unwrap();
    act(&mut game, action);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    for card in [own, stolen] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
    }
    for card in [old, other, wrong_type] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 0);
    }
}
