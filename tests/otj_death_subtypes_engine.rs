//! Pre-event subtype union and legendary creature death receipts.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const OUTLAWS: &[&str] = &["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"];
const DRAIN: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledPermanentWithSubtypeDiesAny(OUTLAWS),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLife(PlayerSet::Opponent, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
const OUTLAW_TYPE_CHANGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::AddCreatureTypes {
            subject: Subject::Targets,
            types: &["Assassin", "Rogue"],
        }],
    ),
    ..CardRules::EMPTY
};
const LEGENDARY_DEATH: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledLegendaryCreatureDies,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[Instruction::DamageScaled(AmountValue::DeathEventPower)],
    }],
    ..CardRules::EMPTY
};

fn resolve(g: &mut Game, victim: Player) {
    for _ in 0..40 {
        match g.state.pending {
            Pending::OrderTriggers { .. } => {
                choose(g, |action| matches!(action, Action::PutTrigger { .. }))
            }
            Pending::Targets { .. } => act(
                g,
                Action::ChooseTarget {
                    target: Target::Player(victim),
                },
            ),
            Pending::Priority if g.state.stack.is_empty() && g.state.resolving.is_none() => return,
            Pending::Priority => act(g, Action::Pass),
            ref other => panic!("unexpected death trigger state {other:?}"),
        }
    }
    panic!("death trigger resolution exceeded 40 actions");
}

#[test]
fn a_dual_outlaw_and_the_observer_each_drain_once_on_simultaneous_death() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &DRAIN),
        (Card::GiantGrowth, &OUTLAW_TYPE_CHANGE),
    ]);
    let player = Player::P1;
    let mut g = scenario(player);
    let observer = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
    let outlaw = put(&mut g, player, "Goblin Piker", Zone::Battlefield);
    put(&mut g, player, "Bear Cub", Zone::Battlefield);
    let opposing_outlaw = put(&mut g, player.other(), "Goblin Piker", Zone::Battlefield);
    let sweep = put(&mut g, player, "Day of Judgment", Zone::Hand);
    for creature in [observer, outlaw, opposing_outlaw] {
        let change = put(&mut g, player, "Giant Growth", Zone::Hand);
        cast(&mut g, change, Some(Target::Card(creature)));
        settle(&mut g);
        assert!(g.has_subtype(creature, "Assassin"));
        assert!(g.has_subtype(creature, "Rogue"));
    }

    cast(&mut g, sweep, None);
    resolve(&mut g, player.other());
    assert_eq!(g.state.objects[observer as usize].zone, Zone::Graveyard);
    assert_eq!(g.life(player), 22);
    assert_eq!(g.life(player.other()), 18);
}

fn make_legendary(g: &mut Game, id: CardId) {
    let mut copy = CopyRecipe::new(g.state.objects[id as usize].card);
    copy.overrides = Some(CopyOverrides {
        legendary: Some(true),
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(g.object_ref(id), copy, CopyDeadline::EndOfTurn);
    assert!(g.object_view(id).is_legendary());
}

#[test]
fn legendary_death_uses_departure_power_and_chooses_a_legal_opponent() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &LEGENDARY_DEATH)]);
    let player = Player::P0;
    let mut root = scenario(player);
    put(&mut root, player, "Ghost Vacuum", Zone::Battlefield);
    let high = put(&mut root, player, "Goblin Piker", Zone::Battlefield);
    let low = put(&mut root, player, "Bear Cub", Zone::Battlefield);
    let ordinary = put(&mut root, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(&mut root, player.other(), "Goblin Piker", Zone::Battlefield);
    let sweep = put(&mut root, player, "Day of Judgment", Zone::Hand);
    for creature in [high, low, foreign] {
        make_legendary(&mut root, creature);
    }
    root.state.objects[high as usize].growth.power = 3;
    root.state.objects[low as usize].growth.power = -5;
    assert_eq!(root.stats(high).power, 5);
    assert!(root.stats(low).power < 0);
    assert!(!root.object_view(ordinary).is_legendary());

    let mut hexproof = root.clone();
    hexproof.state.players[player.other().index()].hexproof_until_end_turn = true;
    cast(&mut hexproof, sweep, None);
    resolve(&mut hexproof, player.other());
    assert_eq!(hexproof.life(player.other()), 20);

    let original = root.object_ref(high);
    cast(&mut root, sweep, None);
    act(&mut root, Action::Pass);
    act(&mut root, Action::Pass);
    assert_eq!(root.state.objects[high as usize].zone, Zone::Graveyard);
    root.move_object(high, Zone::Battlefield);
    assert_ne!(root.object_ref(high), original);
    assert_eq!(root.stats(high).power, 2);
    resolve(&mut root, player.other());
    assert_eq!(root.life(player.other()), 15);
}
