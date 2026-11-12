//! Repeated target groups use their own legal target and exact controller history.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const NONLAND: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: true,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const RESTORE: TargetSpec = TargetSpec::one(TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 0,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: 3,
    },
    predicate: &TargetPredicate::Permanent,
});
const DRAW: Instruction = Instruction::Draw(PlayerSet::TargetController, 1);

const MODES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Exile two nonland permanents, each controller draws",
            targets: TargetPlan::Groups(&[NONLAND, NONLAND]),
            program: &[
                Instruction::Exile,
                DRAW,
                Instruction::SelectGroup(1),
                Instruction::Exile,
                DRAW,
            ],
        },
        SpellMode {
            text: "Return a permanent card with mana value 3 or less",
            targets: TargetPlan::Single(RESTORE),
            program: &[Instruction::ReturnToBattlefieldWithCounters(&[(
                CounterKind::Indestructible,
                1,
            )])],
        },
    ]),
    mode_counts: Some(&[2, 1]),
    ..CardRules::EMPTY
};

fn opt() -> Card {
    Card::from_name("Opt").unwrap()
}

fn choose_mode(game: &mut Game, player: Player, mode: u16) {
    let card = spawn(game, player, opt(), Zone::Hand);
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseMode { mode });
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn select_twice(game: &mut Game, player: Player, first: CardId, second: CardId) {
    choose_mode(game, player, 0);
    act(game, target(first));
    act(game, target(second));
    pay(game);
}

fn drawn(game: &Game, player: Player) -> u64 {
    game.state.players[player.index()].cards_drawn_this_turn
}

#[test]
fn repeated_exile_uses_each_actual_departure_controller_after_control_changes() {
    let _rules = fixture_rules::activate(&[(opt(), &MODES)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let bears = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let beam = put(&mut game, player, "Tractor Beam", Zone::Hand);
    cast(&mut game, beam, Some(Target::Card(bears)));
    settle(&mut game);
    assert_eq!(game.object_view(bears).controller, player);
    select_twice(&mut game, player, beam, bears);
    act(&mut game, Action::Pass);

    let snapshot = game.snapshot();
    let mut indexed = game.clone();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    indexed
        .apply_index(opponent, game.revision(), index)
        .unwrap();
    act(&mut game, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    for branch in [&game, &indexed] {
        assert_eq!(branch.object_view(beam).zone, Zone::Exile);
        assert_eq!(branch.object_view(bears).zone, Zone::Exile);
        assert_eq!(drawn(branch, player), 1);
        assert_eq!(drawn(branch, opponent), 1);
    }
    indexed.restore(&snapshot).unwrap();
    assert_eq!(indexed.snapshot(), snapshot);
}

struct ResetPrevention;

impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
    }
}

#[test]
fn illegal_group_does_not_draw_but_a_prevented_legal_exile_does() {
    let _rules = fixture_rules::activate(&[(opt(), &MODES)]);
    let _reset = ResetPrevention;
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let legal = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let illegal = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    select_twice(&mut game, player, legal, illegal);
    game.move_object(illegal, Zone::Exile);
    let old = game.object_ref(legal);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(old)));
    game.state.zone_rules.get_or_insert_default();
    settle(&mut game);
    assert_eq!(game.object_ref(legal), old);
    assert_eq!(game.object_view(legal).zone, Zone::Battlefield);
    assert_eq!(drawn(&game, player), 0);
    assert_eq!(drawn(&game, opponent), 1);
}

#[test]
fn a_repeated_exact_target_keeps_its_departure_controller_for_both_groups() {
    let _rules = fixture_rules::activate(&[(opt(), &MODES)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let bears = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    select_twice(&mut game, player, bears, bears);
    game.record_control(game.object_ref(bears), opponent, None);
    assert_eq!(game.object_view(bears).controller, opponent);
    settle(&mut game);
    assert_eq!(game.object_view(bears).zone, Zone::Exile);
    assert_eq!(drawn(&game, player), 0);
    assert_eq!(drawn(&game, opponent), 2);
}

#[test]
fn reanimation_uses_own_permanent_cards_including_lands_and_adds_entry_counter() {
    let _rules = fixture_rules::activate(&[(opt(), &MODES)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let other_permanent = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
    let expensive = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let nonpermanent = put(&mut game, player, "Shock", Zone::Graveyard);
    choose_mode(&mut game, player, 1);
    for illegal in [other_permanent, expensive, nonpermanent] {
        reject(&mut game, target(illegal));
    }
    act(&mut game, target(land));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(land, CounterKind::Indestructible), 1);
}
