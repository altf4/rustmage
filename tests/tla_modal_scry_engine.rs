//! Rule fixtures for captured triggered modes and the targeted player's private scry.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: TargetSpec = TargetSpec::one(TargetRule::Player {
    opponent_only: false,
});
const LAND: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 2,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const EARTHBEND: &[Instruction] = &[
    Instruction::AnimateEarthbendTargets,
    Instruction::PutCounters {
        subject: Subject::Targets,
        kind: CounterKind::PlusOne,
        amount: 3,
    },
    Instruction::RegisterEarthbendReturn,
];
const MODE_CAP: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalUpToStat {
            maximum: StatValue::Count(CountFilter {
                zone: CountZone::Graveyard,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Lesson"),
                basic: false,
                other: false,
                cards_only: true,
            }),
            modes: &[
                SpellMode {
                    text: "Choose no modes",
                    targets: TargetPlan::NONE,
                    program: &[],
                },
                SpellMode {
                    text: "Scry",
                    targets: TargetPlan::Single(PLAYER),
                    program: &[Instruction::ScryTargetPlayer(3)],
                },
                SpellMode {
                    text: "Earthbend",
                    targets: TargetPlan::Single(LAND),
                    program: EARTHBEND,
                },
                SpellMode {
                    text: "Scry and earthbend",
                    targets: TargetPlan::Groups(&[PLAYER, LAND]),
                    program: &[
                        Instruction::ScryTargetPlayer(3),
                        Instruction::SelectGroup(1),
                        EARTHBEND[0],
                        EARTHBEND[1],
                        EARTHBEND[2],
                    ],
                },
            ],
            counts: &[0, 1, 1, 2],
        }],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn mode(index: u16) -> Action {
    Action::ChooseMode { mode: index }
}

fn finish_scry(game: &mut Game, expected_chooser: Player) {
    assert_eq!(game.actor(), Some(expected_chooser));
    assert!(
        game.observe(expected_chooser.other())
            .library_choice
            .is_none()
    );
    let candidates = game
        .observe(expected_chooser)
        .library_choice
        .unwrap()
        .remaining;
    assert_eq!(candidates.len(), 3);
    for card in candidates {
        act(
            game,
            Action::ArrangeLibrary {
                card: card.id,
                destination: LibraryDestination::Bottom,
            },
        );
    }
}

#[test]
fn a_trigger_captures_its_own_lesson_count_and_only_offers_bounded_modes() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Bear Cub").unwrap(), &MODE_CAP)]);
    let p = Player::P1;
    let mut game = scenario(p);
    put(&mut game, p, "Forest", Zone::Battlefield);
    put(&mut game, p.other(), "Firebending Lesson", Zone::Graveyard);
    let creature = put(&mut game, p, "Bear Cub", Zone::Hand);

    let mut no_lessons = game.clone();
    cast(&mut no_lessons, creature, None);
    pass_pair(&mut no_lessons);
    let offered_modes = no_lessons
        .legal_actions()
        .into_iter()
        .filter(|action| matches!(action, Action::ChooseMode { .. }))
        .collect::<Vec<_>>();
    assert_eq!(offered_modes, [mode(0)]);
    act(&mut no_lessons, mode(0));
    assert!(no_lessons.state.stack.is_empty());

    let own = put(&mut game, p, "Firebending Lesson", Zone::Graveyard);
    cast(&mut game, creature, None);
    pass_pair(&mut game);
    reject(&mut game, mode(3));
    assert!(game.legal_actions().contains(&mode(1)));
    game.move_object(own, Zone::Exile);
    assert!(game.legal_actions().contains(&mode(1)));
    act(&mut game, mode(0));
}

#[test]
fn choosing_the_opponent_scry_keeps_its_library_private_and_resumes_other_groups() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Bear Cub").unwrap(), &MODE_CAP)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    put(&mut game, p, "Firebending Lesson", Zone::Graveyard);
    put(&mut game, p, "Firebending Lesson", Zone::Graveyard);
    let creature = put(&mut game, p, "Bear Cub", Zone::Hand);
    cast(&mut game, creature, None);
    pass_pair(&mut game);
    let choice = mode(3);
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == choice)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(p, game.revision(), index).unwrap();
    act(&mut game, choice);
    assert_eq!(game.snapshot(), indexed.snapshot());

    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    let checkpoint = game.snapshot();
    let mut departed = game.clone();
    departed.move_object(land, Zone::Graveyard);
    pass_pair(&mut departed);
    finish_scry(&mut departed, p.other());
    assert!(departed.earthbend_views().is_empty());

    pass_pair(&mut game);
    finish_scry(&mut game, p.other());
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 3);
    assert_eq!(game.earthbend_views()[0].subject, game.object_ref(land));

    indexed.restore(&checkpoint).unwrap();
    pass_pair(&mut indexed);
    finish_scry(&mut indexed, p.other());
    assert_eq!(game.snapshot(), indexed.snapshot());
}
