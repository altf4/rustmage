mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DRAW_IF_GREATEST: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::If {
                guard: StepGuard::ControlsGreatestCreaturePower,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};

const CONTROLLED_SAGA: TargetSpec = TargetSpec::one(TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 0,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Subtype("Saga"),
});

const REPEAT_CHAPTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[CONTROLLED_SAGA, CONTROLLED_SAGA]),
        &[
            Instruction::RemoveCounter {
                subject: Subject::Targets,
                kind: CounterKind::Lore,
            },
            Instruction::SelectGroup(1),
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Lore,
                amount: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};

const SAGA: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::NONE,
            program: &[],
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn greatest_creature_power_counts_ties_and_negative_values() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DRAW_IF_GREATEST)]);
    let player = Player::P0;
    for (ours, theirs, draws) in [
        (None, None, false),
        (None, Some(-2), false),
        (Some(-2), None, true),
        (Some(-2), Some(-2), true),
        (Some(-2), Some(-1), false),
        (Some(4), Some(3), true),
    ] {
        let mut game = scenario(player);
        for (controller, power) in [(player, ours), (player.other(), theirs)] {
            if let Some(power) = power {
                let creature = put(&mut game, controller, "Bear Cub", Zone::Battlefield);
                game.state.objects[creature as usize].growth.power = power - 2;
            }
        }
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, None);
        settle(&mut game);
        assert_eq!(
            game.hand_count(player),
            usize::from(draws),
            "{ours:?} versus {theirs:?}"
        );
    }
}

#[test]
fn independent_saga_targets_allow_removal_then_a_forward_chapter_crossing() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &REPEAT_CHAPTER),
        (Card::from_name("Summon: G.F. Ifrit").unwrap(), &SAGA),
    ]);
    let player = Player::P0;
    for (lore, expected_lore, gain) in [(0, 1, 0), (2, 2, 3)] {
        let mut game = scenario(player);
        let saga = put(&mut game, player, "Summon: G.F. Ifrit", Zone::Battlefield);
        game.state.objects[saga as usize].counters.lore = lore;
        game.state.objects[saga as usize].counters.stun = 1;
        let life = game.state.players[player.index()].life;
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        for _ in 0..2 {
            let choice = Action::ChooseTarget {
                target: Target::Card(saga),
            };
            if !game.legal_actions().contains(&choice) {
                act(&mut game, Action::FinishTargets);
            }
            act(&mut game, choice);
        }
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(saga).counters.lore, expected_lore);
        assert_eq!(game.object_view(saga).counters.stun, 1);
        assert_eq!(game.state.players[player.index()].life, life + gain);
    }
}
