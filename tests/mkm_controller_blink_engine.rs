//! A controller's exact return and the opponent's alternative token reward.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DETECTIVE: Card = Card::test_index(5329);
const GATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: 0,
            other: true,
            nonland: true,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::IfSubjectControlledBy {
                subject: Subject::Targets,
                controller: PlayerSet::Controller,
                skip: 2,
            },
            Instruction::BlinkImmediatelyUnderController { tapped: true },
            Instruction::Skip(2),
            Instruction::Exile,
            Instruction::CreateTokensFor {
                players: PlayerSet::TargetController,
                token: DETECTIVE,
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};

fn opt() -> Card {
    Card::from_name("Opt").unwrap()
}

fn cast_gate(game: &mut Game, player: Player, target: CardId) {
    let spell = spawn(game, player, opt(), Zone::Hand);
    cast(game, spell, Some(Target::Card(target)));
}

fn detectives(game: &Game, viewer: Player) -> Vec<ObjectView> {
    game.observe(viewer)
        .objects
        .into_iter()
        .filter(|object| object.card == DETECTIVE && object.zone == Zone::Battlefield)
        .collect()
}

#[test]
fn a_borrowed_permanent_returns_tapped_under_the_ability_controllers_control() {
    let _rules = fixture_rules::activate(&[(opt(), &GATE)]);
    let player = Player::P0;
    let owner = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, owner, "Grizzly Bears", Zone::Battlefield);
    let aura = put(&mut game, player, "Tractor Beam", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(creature)));
    settle(&mut game);
    let old = game.object_ref(creature);
    assert_eq!(game.object_view(creature).controller, player);
    cast_gate(&mut game, player, creature);
    act(&mut game, Action::Pass);

    let root = game.snapshot();
    let pass = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(owner, game.revision(), pass).unwrap();
    act(&mut game, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&root).unwrap();
    assert_eq!(indexed.snapshot(), root);

    let returned = game.object_view(creature);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!(returned.owner, owner);
    assert_eq!(returned.controller, player);
    assert_eq!(returned.reference.generation, old.generation + 2);
    assert!(returned.tapped);
    assert!(returned.attached_to.is_none());
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
    assert!(detectives(&game, player).is_empty());
}

#[test]
fn the_resolution_controller_receives_the_reward_and_a_stale_target_does_not() {
    let _rules = fixture_rules::activate(&[(opt(), &GATE)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let target = put(&mut game, player, "Short Bow", Zone::Battlefield);
    cast_gate(&mut game, player, target);
    let mut stale = game.clone();
    let exact = game.object_ref(target);
    game.record_control(exact, opponent, None);
    assert_eq!(game.object_view(target).controller, opponent);
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Exile);
    let reward = detectives(&game, player);
    assert_eq!(reward.len(), 1);
    assert_eq!(reward[0].controller, opponent);
    assert_eq!(reward[0].owner, opponent);

    stale.move_object(target, Zone::Exile);
    stale.move_object(target, Zone::Battlefield);
    let new = stale.object_ref(target);
    assert_ne!(exact, new);
    settle(&mut stale);
    assert_eq!(stale.object_ref(target), new);
    assert_eq!(stale.object_view(target).zone, Zone::Battlefield);
    assert!(detectives(&stale, player).is_empty());
}
