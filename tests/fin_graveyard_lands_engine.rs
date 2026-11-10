mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ReturnOwnedGraveyardLandsTapped],
    ),
    ..CardRules::EMPTY
};

#[test]
fn all_owned_graveyard_lands_enter_tapped_and_trigger_for_each_actual_entrant() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &RETURN)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let landfall = put(&mut game, player, "Mossborn Hydra", Zone::Battlefield);
    game.add_counter(landfall, CounterKind::PlusOne, 1);
    let basic = put(&mut game, player, "Forest", Zone::Graveyard);
    let nonbasic = put(&mut game, player, "Fabled Passage", Zone::Graveyard);
    let other_land = put(&mut game, player.other(), "Mountain", Zone::Graveyard);
    let nonland = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let hand = put(&mut game, player, "Island", Zone::Hand);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    let previous = [basic, nonbasic].map(|id| game.object_ref(id));
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    let saved = game.snapshot();
    let actor = game.actor().unwrap();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(actor, revision, index).unwrap();
    act(&mut game, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    for (id, before) in [basic, nonbasic].into_iter().zip(previous) {
        assert_ne!(game.object_ref(id), before);
        let object = game.object_view(id);
        assert_eq!(object.zone, Zone::Battlefield);
        assert_eq!(object.controller, player);
        assert!(object.tapped);
    }
    assert_eq!(game.object_view(other_land).zone, Zone::Graveyard);
    assert_eq!(game.object_view(nonland).zone, Zone::Graveyard);
    assert_eq!(game.object_view(hand).zone, Zone::Hand);
    assert_eq!(game.counter_count(landfall, CounterKind::PlusOne), 1);
    assert_eq!(game.state.ordering_triggers.len(), 2);
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    settle(&mut game);
    assert_eq!(game.counter_count(landfall, CounterKind::PlusOne), 4);
}

struct ResetPrevention;
impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
    }
}

#[test]
fn entry_replacements_apply_to_titan_land_moves_as_one_batch() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &RETURN)]);
    let _reset = ResetPrevention;
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Spelunking", Zone::Battlefield);
    let prevented = put(&mut game, player, "Forest", Zone::Graveyard);
    let enters = put(&mut game, player, "Fabled Passage", Zone::Graveyard);
    let previous = game.object_ref(prevented);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(Some(previous)));
    game.state.zone_rules.get_or_insert_default();
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.object_ref(prevented), previous);
    assert_eq!(game.object_view(prevented).zone, Zone::Graveyard);
    assert_eq!(game.object_view(enters).zone, Zone::Battlefield);
    assert!(!game.object_view(enters).tapped);
}
