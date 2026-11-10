//! Delayed step expiry and linked, owner-controlled returns.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const NEXT_COMBAT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ScheduleStepThisTurn {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            ability: 0,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const EXILE_LINKS: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 2,
            max: 2,
        }),
        program: &[Instruction::ExileTargetsLinkedToSource],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnLinkedExileToBattlefield],
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn delayed_combat_fires_once_and_expires_if_the_step_is_missed() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &NEXT_COMBAT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(
        game.observe(player).delayed_steps.as_ref().unwrap().len(),
        1
    );

    let mut missed = game.clone();
    let turn = missed.state.turn;
    missed.begin_step(Step::End);
    for _ in 0..16 {
        if missed.state.turn != turn {
            break;
        }
        act(&mut missed, Action::Pass);
    }
    assert_ne!(missed.state.turn, turn);
    assert!(
        missed
            .observe(player)
            .delayed_steps
            .is_none_or(|rows| rows.is_empty())
    );

    game.begin_step(Step::BeginCombat);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    game.begin_step(Step::BeginCombat);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn linked_return_uses_only_current_exiles_and_follows_owners_after_source_departure() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Hopeful Vigil").unwrap(), &EXILE_LINKS)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let a = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let b = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        game.state.objects[b as usize].controller = player;
        let source = put(&mut game, player, "Hopeful Vigil", Zone::Hand);
        let removal = put(&mut game, player, "Stroke of Midnight", Zone::Hand);
        cast(&mut game, source, None);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        for id in [a, b] {
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        settle(&mut game);
        assert_eq!(game.object_view(a).zone, Zone::Exile);
        assert_eq!(game.object_view(b).zone, Zone::Exile);
        let mut stale = game.clone();
        stale.move_object(b, Zone::Graveyard);
        stale.move_object(b, Zone::Exile);

        for branch in [&mut game, &mut stale] {
            act(
                branch,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(branch);
            cast(branch, removal, Some(Target::Card(source)));
            settle(branch);
            assert_eq!(branch.object_view(source).zone, Zone::Graveyard);
            assert_eq!(branch.object_view(a).zone, Zone::Battlefield);
            assert_eq!(branch.object_view(a).controller, player);
        }
        assert_eq!(game.object_view(b).zone, Zone::Battlefield);
        assert_eq!(game.object_view(b).controller, player.other());
        assert_eq!(stale.object_view(b).zone, Zone::Exile);
    }
}
