//! The combat scheduler and the decision controller follow actual phase boundaries.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: TargetPlan = one(TargetRule::Player {
    opponent_only: false,
});
const COMBAT_CONTROL: CardRules = CardRules {
    effect: Effect::Spell(PLAYER, &[Instruction::ControlTargetPlayerNextCombat]),
    ..CardRules::EMPTY
};
const TURN_CONTROL: CardRules = CardRules {
    effect: Effect::Spell(PLAYER, &[Instruction::ControlTargetPlayerNextTurn]),
    ..CardRules::EMPTY
};
const LAND_COMBAT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::UntapControlledLands,
            Instruction::AdditionalLandCreatureCombat,
        ],
    ),
    ..CardRules::EMPTY
};
const ORDINARY_COMBAT: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::AdditionalCombat]),
    ..CardRules::EMPTY
};
const ANIMATE: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[
            Instruction::AnimateEarthbendTargets,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn reach(game: &mut Game, stop: impl Fn(&Game) -> bool) {
    for _ in 0..112 {
        if stop(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("combat phase checkpoint not reached");
}

fn priority(game: &Game, player: Player, step: Step) -> bool {
    game.state.active == player
        && game.state.step == step
        && game.state.pending == Pending::Priority
}

fn can_see(game: &Game, viewer: Player, card: CardId) -> bool {
    game.observe(viewer)
        .objects
        .iter()
        .any(|object| object.id == card)
}

fn resolve(game: &mut Game, caster: Player, spell: CardId, target: Option<Target>) {
    reach(game, |g| {
        g.state.pending == Pending::Priority && g.semantic_actor() == Some(caster)
    });
    game.state.players[caster.index()].mana = [20; 6];
    cast(game, spell, target);
    settle(game);
}

#[test]
fn next_actual_combat_controls_decisions_and_information_for_only_that_phase() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &COMBAT_CONTROL)]);
    for controller in [Player::P0, Player::P1] {
        let target = controller.other();
        let mut game = scenario(controller);
        let control = put(&mut game, controller, "Opt", Zone::Hand);
        let secret = put(&mut game, target, "Grizzly Bears", Zone::Hand);
        let attacker = put(&mut game, target, "Goblin Piker", Zone::Battlefield);
        resolve(&mut game, controller, control, Some(Target::Player(target)));
        assert_eq!(
            game.observe(controller)
                .player_control
                .unwrap()
                .pending_combat[target.index()],
            Some(controller)
        );
        reach(&mut game, |g| priority(g, target, Step::Main1));
        assert_eq!(game.actor(), Some(target));
        assert!(!can_see(&game, controller, secret));
        reach(&mut game, |g| priority(g, target, Step::BeginCombat));
        assert!(can_see(&game, controller, secret));
        assert!(can_see(&game, target, secret));
        assert_eq!(game.actor(), Some(controller));
        assert_eq!(game.semantic_actor(), Some(target));
        reach(&mut game, |g| g.state.pending == Pending::Attackers);
        let attack = Action::Attack { card: attacker };
        let view = game.observe(controller);
        let choice = view
            .actions
            .iter()
            .position(|action| *action == attack)
            .unwrap();
        let checkpoint = game.snapshot();
        assert!(game.apply(target, attack.clone()).is_err());
        assert_eq!(game.snapshot(), checkpoint);
        let mut raw = game.clone();
        let mut indexed = game.clone();
        let mut restored = game;
        act(&mut raw, attack.clone());
        indexed
            .apply_index(controller, view.revision, choice)
            .unwrap();
        restored.restore(&checkpoint).unwrap();
        act(&mut restored, attack);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), restored.snapshot());
        for child in [&mut raw, &mut indexed, &mut restored] {
            act(child, Action::FinishAttackers);
            reach(child, |g| priority(g, target, Step::Main2));
            assert_eq!(child.actor(), Some(target));
            assert_eq!(child.state.active, target);
            assert!(!can_see(child, controller, secret));
        }
    }
}

#[test]
fn ending_the_turn_before_combat_preserves_control_for_the_next_actual_combat() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &COMBAT_CONTROL)]);
    let controller = Player::P0;
    let target = controller.other();
    let mut game = scenario(controller);
    let control = put(&mut game, controller, "Opt", Zone::Hand);
    let end_turn = put(&mut game, target, "Time Stop", Zone::Hand);
    resolve(&mut game, controller, control, Some(Target::Player(target)));
    reach(&mut game, |g| priority(g, target, Step::Main1));
    let skipped = game.state.turn;
    resolve(&mut game, target, end_turn, None);
    assert_eq!(
        game.observe(controller)
            .player_control
            .unwrap()
            .pending_combat[target.index()],
        Some(controller)
    );
    reach(&mut game, |g| priority(g, target, Step::BeginCombat));
    assert!(game.state.turn > skipped);
    assert_eq!(game.actor(), Some(controller));
}

#[test]
fn later_player_control_wins_only_during_the_overlapping_duration() {
    let _rules = fixture_rules::activate(&[
        (named("Opt"), &COMBAT_CONTROL),
        (named("Negate"), &TURN_CONTROL),
    ]);
    let opponent = Player::P0;
    let target = opponent.other();
    for combat_is_later in [false, true] {
        let mut game = scenario(opponent);
        let combat = put(&mut game, opponent, "Opt", Zone::Hand);
        let own_turn = put(&mut game, target, "Negate", Zone::Hand);
        let target_action = Some(Target::Player(target));
        if combat_is_later {
            resolve(&mut game, target, own_turn, target_action);
            resolve(&mut game, opponent, combat, target_action);
        } else {
            resolve(&mut game, opponent, combat, target_action);
            resolve(&mut game, target, own_turn, target_action);
        }
        reach(&mut game, |g| priority(g, target, Step::Main1));
        assert_eq!(game.actor(), Some(target));
        reach(&mut game, |g| priority(g, target, Step::BeginCombat));
        assert_eq!(
            game.actor(),
            Some(if combat_is_later { opponent } else { target })
        );
        reach(&mut game, |g| priority(g, target, Step::Main2));
        assert_eq!(game.actor(), Some(target));
    }
}

#[test]
fn land_only_combat_keeps_its_own_place_in_the_inserted_phase_schedule() {
    let _rules = fixture_rules::activate(&[
        (named("Opt"), &LAND_COMBAT),
        (named("Giant Growth"), &ORDINARY_COMBAT),
        (named("Negate"), &ANIMATE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let animate = put(&mut game, player, "Negate", Zone::Hand);
    let restricted = put(&mut game, player, "Opt", Zone::Hand);
    let ordinary = put(&mut game, player, "Giant Growth", Zone::Hand);
    resolve(&mut game, player, animate, Some(Target::Card(land)));
    reach(&mut game, |g| g.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: land });
    act(&mut game, Action::FinishAttackers);
    reach(&mut game, |g| priority(g, player, Step::EndCombat));
    assert!(game.object_view(land).tapped);
    resolve(&mut game, player, restricted, None);
    assert!(!game.object_view(land).tapped);
    resolve(&mut game, player, ordinary, None);
    let phases = game.observe(player).after_combat_phases.unwrap();
    assert_eq!(phases.len(), 2);
    assert!(!phases[0].land_creature_attackers_only);
    assert!(phases[1].land_creature_attackers_only);
    reach(&mut game, |g| g.state.pending == Pending::Attackers);
    assert!(
        game.legal_actions()
            .contains(&Action::Attack { card: bear })
    );
    act(&mut game, Action::FinishAttackers);
    reach(&mut game, |g| g.state.pending == Pending::Attackers);
    assert!(game.observe(player).land_creature_attackers_only);
    reject(&mut game, Action::Attack { card: bear });
    assert!(
        game.legal_actions()
            .contains(&Action::Attack { card: land })
    );
    act(&mut game, Action::FinishAttackers);
    reach(&mut game, |g| priority(g, player, Step::Main2));
    assert!(!game.observe(player).land_creature_attackers_only);
}
