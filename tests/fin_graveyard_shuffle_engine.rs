//! Filtered graveyard shuffle counts only cards that actually reach the library.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ELIXIR: CardRules = CardRules {
    effect: Effect::Permanent,
    entry: EntryTapped::Always,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 5,
            mana_component: true,
            tap: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ShuffleGraveyardIntoLibraryCount { excluded_kinds: 2 },
            Instruction::GainLifeMoved,
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

struct ResetPrevention;

impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
    }
}

fn elixir() -> Card {
    Card::from_name("Elixir").unwrap()
}

fn activate(source: CardId) -> Action {
    Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

#[test]
fn the_shuffle_uses_current_own_nonlands_and_gains_life_for_actual_arrivals() {
    let _rules = fixture_rules::activate(&[(elixir(), &ELIXIR)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, elixir(), Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.object_view(source).tapped);
    reject(&mut game, activate(source));
    game.state.objects[source as usize].tapped = false;
    let before = game.object_ref(source);
    let spell = put(&mut game, player, "Shock", Zone::Graveyard);
    let stopped = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    let removed = put(&mut game, player, "Llanowar Elves", Zone::Graveyard);
    let land = put(&mut game, player, "Island", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Grizzly Bears", Zone::Graveyard);
    let late = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 5];
    act(&mut game, activate(source));
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    assert_eq!(game.state.stack[0].source().unwrap().object, before);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    game.move_object(removed, Zone::Exile);
    game.move_object(late, Zone::Graveyard);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(opponent));

    let _reset = ResetPrevention;
    let blocked = game.object_ref(stopped);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(blocked)));
    let old_spell = game.object_ref(spell);
    let mut expected = game.state.players[player.index()].library.to_vec();
    expected.extend([spell, late]);
    let mut expected_rng = game.state.rng;
    expected_rng.shuffle(&mut expected);

    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let revision = game.revision();
    let saved = game.snapshot();
    let mut raw = game.clone();
    act(&mut raw, Action::Pass);
    game.apply_index(opponent, revision, index).unwrap();
    for branch in [&raw, &game] {
        assert_eq!(branch.object_view(spell).zone, Zone::Library);
        assert_eq!(branch.object_view(late).zone, Zone::Library);
        assert_ne!(branch.object_ref(spell), old_spell);
        assert_eq!(branch.object_ref(stopped), blocked);
        for id in [stopped, land, foreign] {
            assert_eq!(branch.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(branch.object_view(removed).zone, Zone::Exile);
        assert_eq!(branch.state.players[player.index()].life, 22);
        assert_eq!(&branch.state.players[player.index()].library[..], &expected);
        assert_eq!(branch.state.rng, expected_rng);
    }
    assert_eq!(raw.snapshot(), game.snapshot());

    raw.restore(&saved).unwrap();
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
    act(&mut raw, Action::Pass);
    assert_eq!(raw.object_view(stopped).zone, Zone::Library);
    assert_eq!(raw.state.players[player.index()].life, 23);
    assert_eq!(game.object_view(stopped).zone, Zone::Graveyard);
}

#[test]
fn zero_eligible_cards_still_shuffle_without_gaining_life() {
    let _rules = fixture_rules::activate(&[(elixir(), &ELIXIR)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, elixir(), Zone::Battlefield);
    let land = put(&mut game, player, "Island", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Shock", Zone::Graveyard);
    let top = *game.state.players[player.index()].library.last().unwrap();
    game.remember_library_top(player, game.object_ref(top));
    assert!(game.observe(opponent).known_library_top[player.index()].is_some());
    let mut expected_rng = game.state.rng;
    let mut expected = game.state.players[player.index()].library.to_vec();
    expected_rng.shuffle(&mut expected);

    act(&mut game, activate(source));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(&game.state.players[player.index()].library[..], &expected);
    assert_eq!(game.state.rng, expected_rng);
    assert!(game.observe(opponent).known_library_top[player.index()].is_none());
}
