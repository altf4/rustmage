//! The active player chooses whether to draw at combat and then cannot attack.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PEACE: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayDrawForPlayer {
                player: PlayerSet::Active,
                skip: 3,
            },
            Instruction::Draw(PlayerSet::Active, 1),
            Instruction::ProhibitAttacksThisTurn(PlayerSet::Active),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Peace,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const NO_DRAW: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CantDrawCards],
    }],
    ..CardRules::EMPTY
};

fn to_choice(game: &mut Game) {
    for _ in 0..24 {
        if matches!(game.state.pending, Pending::EffectOptional { .. }) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("combat draw choice was not reached");
}

#[test]
fn active_player_draws_and_loses_all_attack_choices_with_owned_restore() {
    let peace = Card::from_name("Banishing Light").unwrap();
    let _rules = fixture_rules::activate(&[(peace, &PEACE)]);
    for active in [Player::P0, Player::P1] {
        let mut game = scenario(active);
        let source = spawn(&mut game, active.other(), peace, Zone::Battlefield);
        let attacker = put(&mut game, active, "Hill Giant", Zone::Battlefield);
        game.state.objects[attacker as usize].summoning_sick = false;
        to_choice(&mut game);
        assert_eq!(game.actor(), Some(active));
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let accept = Action::ChooseOptional { accept: true };
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == accept)
            .unwrap();
        let mut accepted = game.clone();
        let mut indexed = game.clone();
        act(&mut accepted, accept);
        indexed.apply_index(active, revision, index).unwrap();
        assert_eq!(accepted.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(active, revision, index),
            Err(Error::StaleRevision)
        );
        settle(&mut accepted);
        assert_eq!(accepted.counter_count(source, CounterKind::Peace), 1);
        assert_eq!(
            accepted.state.players[active.index()].cards_drawn_this_turn,
            1
        );
        for viewer in [active, active.other()] {
            assert!(accepted.observe(viewer).players[active.index()].cannot_attack_this_turn);
        }
        accepted.state.step = Step::Attackers;
        accepted.state.pending = Pending::Attackers;
        assert!(
            !accepted
                .legal_actions()
                .contains(&Action::Attack { card: attacker })
        );
        reject(&mut accepted, Action::Attack { card: attacker });

        let mut declined = game;
        act(&mut declined, Action::ChooseOptional { accept: false });
        settle(&mut declined);
        assert_eq!(declined.counter_count(source, CounterKind::Peace), 0);
        assert_eq!(
            declined.state.players[active.index()].cards_drawn_this_turn,
            0
        );
        declined.state.step = Step::Attackers;
        declined.state.pending = Pending::Attackers;
        assert!(
            declined
                .legal_actions()
                .contains(&Action::Attack { card: attacker })
        );
    }
}

#[test]
fn draw_prohibition_prevents_the_peace_choice_and_attack_lock() {
    let peace = Card::from_name("Banishing Light").unwrap();
    let no_draw = Card::from_name("Candy Trail").unwrap();
    let _rules = fixture_rules::activate(&[(peace, &PEACE), (no_draw, &NO_DRAW)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P1, "Banishing Light", Zone::Battlefield);
    let attacker = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    spawn(&mut game, Player::P1, no_draw, Zone::Battlefield);
    for _ in 0..24 {
        if game.state.step == Step::BeginCombat
            && game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
        {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.counter_count(source, CounterKind::Peace), 0);
    assert!(!game.observe(Player::P0).players[0].cannot_attack_this_turn);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    assert!(
        game.legal_actions()
            .contains(&Action::Attack { card: attacker })
    );
}
