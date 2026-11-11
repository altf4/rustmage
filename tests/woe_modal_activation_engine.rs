//! A paid modal activation consumes its mode for that ability incarnation.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const MODAL: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            tap: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalOnce(&[
            SpellMode {
                text: "Deal damage",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
                program: &[Instruction::Damage(1)],
            },
            SpellMode {
                text: "Tap",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
                program: &[Instruction::Tap],
            },
            SpellMode {
                text: "Gain life",
                targets: TargetPlan::NONE,
                program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
            },
        ])],
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

const BLINK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::BlinkImmediately {
            counter: None,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};

fn announce_ability(source: CardId) -> Action {
    Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn choose_and_pay(game: &mut Game, mode: u16, target: Option<CardId>) {
    act(game, Action::ChooseMode { mode });
    if let Some(target) = target {
        act(
            game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    pay(game);
}

#[test]
fn activated_modes_are_consumed_after_payment_and_reset_with_a_new_incarnation() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &MODAL),
        (Card::from_name("Giant Growth").unwrap(), &BLINK),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let blink = put(&mut game, player, "Giant Growth", Zone::Hand);

        act(&mut game, announce_ability(source));
        choose_and_pay(&mut game, 0, Some(creature));
        settle(&mut game);
        assert_eq!(game.object_view(creature).damage, 1);

        game.state.objects[source as usize].tapped = false;
        act(&mut game, announce_ability(source));
        let snapshot = game.snapshot();
        assert!(game.apply(player, Action::ChooseMode { mode: 0 }).is_err());
        assert_eq!(game.snapshot(), snapshot);
        choose_and_pay(&mut game, 1, Some(creature));
        let activation = game.state.stack.last().unwrap().id;
        game.counter_stack_item(activation, false);
        settle(&mut game);
        assert!(!game.object_view(creature).tapped);

        game.state.objects[source as usize].tapped = false;
        act(&mut game, announce_ability(source));
        assert!(
            !game
                .legal_actions()
                .contains(&Action::ChooseMode { mode: 1 })
        );
        choose_and_pay(&mut game, 2, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 23);

        game.state.objects[source as usize].tapped = false;
        assert!(!game.legal_actions().contains(&announce_ability(source)));
        cast(&mut game, blink, Some(Target::Card(source)));
        settle(&mut game);
        act(&mut game, announce_ability(source));
        assert!(
            game.legal_actions()
                .contains(&Action::ChooseMode { mode: 0 })
        );
    }
}
