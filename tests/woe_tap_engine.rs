//! The player instructed to tap owns the event, including paid choices.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OpponentCreatureTappedByYou { one_or_more: false },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(
                &TriggerCondition::OpponentCreatureTappedByYou { one_or_more: true },
            ),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};

const TAP_TWO: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 2,
            max: 2,
        }),
        &[Instruction::Tap],
    ),
    ..CardRules::EMPTY
};

const OPPONENT_TAPS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::TapPermanents(1),
            players: PlayerSet::Opponent,
            filter: SacrificeFilter {
                artifact_enchantment_or_token: false,
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
                nonland: false,
                other: false,
                tokens: None,
                subtype: None,
            },
            amount: 1,
            optional_skip: None,
        }],
    ),
    ..CardRules::EMPTY
};

fn finish_tap_triggers(game: &mut Game) {
    for _ in 0..16 {
        match game.state.pending {
            Pending::OrderTriggers { .. } => {
                choose(game, |a| matches!(a, Action::PutTrigger { .. }))
            }
            Pending::Priority if game.state.stack.is_empty() => return,
            Pending::Priority => act(game, Action::Pass),
            _ => panic!(
                "unexpected choice in tap trigger test: {:?}",
                game.state.pending
            ),
        }
    }
    panic!("tap trigger queue did not finish");
}

#[test]
fn actor_tap_group_observers_keep_owned_triggers_and_skip_already_tapped_objects() {
    let _rules =
        fixture_rules::activate(&[(Card::HillGiant, &OBSERVER), (Card::Divination, &TAP_TWO)]);
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        put(&mut parent, player, "Hill Giant", Zone::Battlefield);
        put(&mut parent, player.other(), "Hill Giant", Zone::Battlefield);
        let first = put(&mut parent, player.other(), "Bear Cub", Zone::Battlefield);
        let second = put(&mut parent, player.other(), "Bear Cub", Zone::Battlefield);
        let later = put(&mut parent, player.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut parent, player, "Divination", Zone::Hand);
        let repeat = put(&mut parent, player, "Divination", Zone::Hand);
        announce(
            &mut parent,
            spell,
            &[Target::Card(first), Target::Card(second)],
        );
        act(&mut parent, Action::Pass);

        let checkpoint = parent.snapshot();
        let revision = parent.revision();
        let pass = parent
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        drop(parent);

        assert!(indexed.apply_index(player, revision, pass).is_err());
        assert_eq!(indexed.snapshot(), checkpoint);
        indexed.apply_index(player.other(), revision, pass).unwrap();
        act(&mut raw, Action::Pass);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert!(raw.object_view(first).tapped && raw.object_view(second).tapped);
        assert!(!raw.object_view(later).tapped);
        finish_tap_triggers(&mut raw);
        finish_tap_triggers(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());

        let before_repeat = &raw.state.players[player.index()];
        assert_eq!(before_repeat.life, 22, "each actual opposing tap triggers");
        assert_eq!(
            before_repeat.cards_drawn_this_turn, 1,
            "the group triggers once"
        );
        assert_eq!(raw.state.players[player.other().index()].life, 20);

        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, Action::Pass);
        finish_tap_triggers(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());

        announce(
            &mut raw,
            repeat,
            &[Target::Card(first), Target::Card(later)],
        );
        finish_tap_triggers(&mut raw);
        assert!(raw.object_view(later).tapped);
        assert_eq!(
            raw.state.players[player.index()].life,
            23,
            "only the new tap counts"
        );
        assert_eq!(raw.state.players[player.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn instruction_for_opponent_to_tap_is_attributed_to_that_player() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &OBSERVER),
        (Card::Divination, &OPPONENT_TAPS),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut game, player, "Divination", Zone::Hand);
        cast(&mut game, spell, None);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.actor(), Some(player.other()));
        let object = game.object_ref(creature);
        act(&mut game, Action::Sacrifice { object });
        finish_tap_triggers(&mut game);

        assert!(game.object_view(creature).tapped);
        for seat in [Player::P0, Player::P1] {
            assert_eq!(game.state.players[seat.index()].life, 20);
            assert_eq!(game.state.players[seat.index()].cards_drawn_this_turn, 0);
        }
    }
}
