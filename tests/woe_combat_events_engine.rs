//! Blocking pairs and newly blocked attackers are separate historical events.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PAIR_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBlocksOrBecomesBlockedByCreature,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamageToEventObject(1)],
    }],
    ..CardRules::EMPTY
};

const RAT_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledSubtypeBecomesBlocked("Rat"),
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Entering,
            power: 2,
            toughness: 0,
        }],
    }],
    ..CardRules::EMPTY
};

const BLINK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::BlinkImmediately {
            counter: None,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};

fn declare_blocks(game: &mut Game, attacker: CardId, blockers: &[CardId]) {
    game.begin_step(Step::BeginCombat);
    act(game, Action::Pass);
    act(game, Action::Pass);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.pending, Pending::Blockers);
    for &blocker in blockers {
        act(game, Action::Block { blocker, attacker });
    }
    act(game, Action::FinishBlockers);
}

fn announce_triggers(game: &mut Game) {
    for _ in 0..4 {
        if !matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            return;
        }
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    panic!("more block triggers than expected");
}

#[test]
fn each_blocking_pair_deals_damage_only_to_its_original_incarnation() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &PAIR_OBSERVER),
        (Card::from_name("Giant Growth").unwrap(), &BLINK),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let slinger = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let stay = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let blink = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let old_blink = game.object_ref(blink);
        let spell = put(&mut game, player, "Giant Growth", Zone::Hand);
        let forest = put(&mut game, player, "Forest", Zone::Battlefield);

        declare_blocks(&mut game, slinger, &[stay, blink]);
        announce_triggers(&mut game);
        assert_eq!(game.state.stack.len(), 2);
        act(&mut game, Action::TapMana { card: forest });
        cast(&mut game, spell, Some(Target::Card(blink)));
        settle(&mut game);
        assert_ne!(game.object_ref(blink), old_blink);
        assert_eq!(game.object_view(stay).damage, 1);
        assert_eq!(game.object_view(blink).damage, 0);

        let mut defending = scenario(player);
        let attacker = put(&mut defending, player, "Bear Cub", Zone::Battlefield);
        let slinger = put(
            &mut defending,
            player.other(),
            "Hill Giant",
            Zone::Battlefield,
        );
        declare_blocks(&mut defending, attacker, &[slinger]);
        announce_triggers(&mut defending);
        settle(&mut defending);
        assert_eq!(defending.object_view(attacker).damage, 1);
    }
}

#[test]
fn a_rat_blocked_by_two_creatures_becomes_blocked_once() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &RAT_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let rat = put(&mut game, player, "Voracious Vermin", Zone::Battlefield);
    let first = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let base = game.object_view(rat).power;

    declare_blocks(&mut game, rat, &[first, second]);
    announce_triggers(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.object_view(rat).power, base + 2);
}
