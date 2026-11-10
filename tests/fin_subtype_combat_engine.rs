mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SUBTYPES: &[&str] = &["Kraken", "Leviathan", "Merfolk", "Octopus", "Serpent"];

const RETURN_OTHER_CREATURES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ReturnCreaturesWithoutSubtypes(SUBTYPES)],
    ),
    ..CardRules::EMPTY
};

const DELAYED_ATTACK_DRAWS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ScheduleAttackTriggerThisTurn { ability: 0 }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedCreatureAttacksSubtypeAny(SUBTYPES),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn return_group_excludes_the_union_of_live_creature_subtypes() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &RETURN_OTHER_CREATURES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let octopus = put(
        &mut game,
        player,
        "Ultros, Obnoxious Octopus",
        Zone::Battlefield,
    );
    let changeling = put(
        &mut game,
        player.other(),
        "Three Tree Mascot",
        Zone::Battlefield,
    );
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let giant = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player, "Island", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    for id in [octopus, changeling, land] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
    }
    for id in [bear, giant] {
        assert_eq!(game.object_view(id).zone, Zone::Hand);
    }
}

#[test]
fn delayed_subtype_draws_follow_each_actual_attacker_of_either_player() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DELAYED_ATTACK_DRAWS)]);
    let beneficiary = Player::P0;
    for active in [beneficiary, beneficiary.other()] {
        let mut game = scenario(active);
        let attackers = [
            put(
                &mut game,
                active,
                "Ultros, Obnoxious Octopus",
                Zone::Battlefield,
            ),
            put(&mut game, active, "Three Tree Mascot", Zone::Battlefield),
            put(&mut game, active, "Bear Cub", Zone::Battlefield),
        ];
        let spell = put(&mut game, beneficiary, "Opt", Zone::Hand);
        if active != beneficiary {
            act(&mut game, Action::Pass);
        }
        cast(&mut game, spell, None);
        settle(&mut game);
        let initial_hand = game.hand_count(beneficiary);
        assert_eq!(game.object_view(spell).zone, Zone::Graveyard);

        for _ in 0..12 {
            if game.state.pending == Pending::Attackers {
                break;
            }
            act(&mut game, Action::Pass);
        }
        assert_eq!(game.state.pending, Pending::Attackers);
        for card in attackers {
            act(&mut game, Action::Attack { card });
        }
        act(&mut game, Action::FinishAttackers);
        for _ in 0..20 {
            if game.state.pending == Pending::Priority
                && game.state.stack.is_empty()
                && game.state.pending_triggers.is_empty()
            {
                break;
            }
            choose(&mut game, |action| {
                matches!(action, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert_eq!(game.hand_count(beneficiary), initial_hand + 2);
        assert_eq!(game.hand_count(beneficiary.other()), 0);

        let turn = game.state.turn;
        for _ in 0..36 {
            if game.state.turn != turn {
                break;
            }
            choose(&mut game, |action| {
                matches!(action, Action::Pass | Action::FinishBlockers)
            });
        }
        assert_eq!(game.state.turn, turn + 1);
        assert!(game.state.delayed_attack_this_turn.is_none());
    }
}
