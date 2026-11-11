//! Self-entry caster checks retain the actual cast, independently of control.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ENTRY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCastByController,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const COPY_TRIGGER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::TriggeredAbility {
            relation: 1,
        })),
        &[Instruction::CopyTargetAbility { group: 0 }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn an_uncast_entrant_or_an_entrant_under_the_other_controller_does_not_trigger() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ENTRY)]);
    let caster = Player::P0;
    let mut game = scenario(caster);

    let uncast = put(&mut game, caster, "Grizzly Bears", Zone::Graveyard);
    let movement = game.entry_move(uncast, Zone::Battlefield, false);
    game.stage_entries(vec![movement], None);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());

    let casted = put(&mut game, caster, "Grizzly Bears", Zone::Hand);
    cast(&mut game, casted, None);
    let spell = game.state.stack.pop().unwrap();
    let receipt = spell.actual_cast_event().unwrap();
    for controller in [caster, caster.other()] {
        let mut branch = game.clone();
        let mut movement = branch.entry_move(casted, Zone::Battlefield, false);
        movement.controller = controller;
        movement.cast_origin = Some(Zone::Hand);
        movement.cast_event = Some(Box::new(receipt));
        movement.cast_payment = Some(receipt.payment);
        branch.stage_entries(vec![movement], None);
        branch.prepare_triggers();
        assert_eq!(branch.object_view(casted).controller, controller);
        assert_eq!(branch.state.stack.len(), usize::from(caster == controller));
        settle(&mut branch);
        assert_eq!(
            branch.state.players[controller.index()].cards_drawn_this_turn,
            u64::from(caster == controller),
        );
    }
}

#[test]
fn another_players_copy_rechecks_who_actually_cast_the_permanent() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ENTRY), (opt, &COPY_TRIGGER)]);
    let caster = Player::P0;
    let other = caster.other();
    let mut game = scenario(caster);
    let entrant = put(&mut game, caster, "Grizzly Bears", Zone::Hand);
    let copier = put(&mut game, other, "Opt", Zone::Hand);
    cast(&mut game, entrant, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let trigger = game.state.stack.last().expect("cast-entry trigger").id;
    act(&mut game, Action::Pass);
    cast(&mut game, copier, Some(Target::Stack(trigger)));
    settle(&mut game);
    assert_eq!(game.state.players[caster.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.state.players[other.index()].cards_drawn_this_turn, 0);
}
