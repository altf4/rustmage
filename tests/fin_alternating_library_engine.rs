//! The revealed library cohort is owned and each mandatory pick commits in order.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const MEMORIES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::AlternateRevealedLibraryPicks(5)],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 7,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn memories() -> Card {
    Card::from_name("Memories Returning").unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn select(game: &mut Game, card: CardId) {
    act(game, Action::SelectLibrary { card });
}

#[test]
fn both_players_see_the_same_exact_public_cohort_and_alternate_real_moves() {
    let _rules = fixture_rules::activate(&[(memories(), &MEMORIES)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, memories(), Zone::Hand);
    let hidden = put(&mut game, player, "Llanowar Elves", Zone::Library);
    let revealed = [
        "Island",
        "Shock",
        "Grizzly Bears",
        "Forest",
        "Swiftfoot Boots",
    ]
    .map(|name| put(&mut game, player, name, Zone::Library));
    let refs = revealed.map(|id| game.object_ref(id));
    let [first, second, third, fourth, fifth] = revealed;
    cast(&mut game, spell, None);
    assert!(game.state.stack[0].targets.is_empty());
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    for viewer in [player, opponent] {
        let view = game.observe(viewer).library_selection.unwrap();
        assert_eq!(view.candidates, refs.into_iter().rev().collect::<Vec<_>>());
        assert_eq!(view.remaining.len(), 5);
        assert!(view.public && view.reveal);
        assert_eq!(view.destination, "hand");
    }
    reject(&mut game, Action::SelectLibrary { card: hidden });
    reject(&mut game, Action::FinishLibrarySelection);
    select(&mut game, third);
    assert_eq!(game.object_view(third).zone, Zone::Hand);
    assert_ne!(game.object_ref(third), refs[2]);
    let view = game.observe(opponent);
    assert!(view.known_hand.contains(&game.object_ref(third)));
    assert_eq!(view.library_selection.unwrap().destination, "bottom");
    assert_eq!(game.actor(), Some(opponent));
    reject(&mut game, Action::SelectLibrary { card: third });

    let action = Action::SelectLibrary { card: second };
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let revision = game.revision();
    let saved = game.snapshot();
    assert_eq!(game.apply(player, action.clone()), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), saved);
    let mut raw = game.clone();
    let mut indexed = game;
    act(&mut raw, action);
    indexed.apply_index(opponent, revision, index).unwrap();
    for branch in [&mut raw, &mut indexed] {
        assert_eq!(branch.state.players[player.index()].library[0], second);
        assert_eq!(branch.object_ref(second), refs[1]);
        assert_eq!(branch.actor(), Some(player));
        assert_eq!(
            branch.observe(player).library_selection.unwrap().bottom[0].reference,
            refs[1]
        );
        select(branch, fifth);
        assert_eq!(branch.actor(), Some(opponent));
        select(branch, first);
        assert_eq!(branch.state.pending, Pending::Priority);
        assert_eq!(branch.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(
            branch.state.players[player.index()].library[..2],
            [first, second]
        );
        assert_eq!(
            branch.state.players[player.index()].library.last(),
            Some(&hidden)
        );
        assert!(branch.observe(opponent).known_library_top[player.index()].is_none());
        for id in [third, fourth, fifth] {
            assert_eq!(branch.object_view(id).zone, Zone::Hand);
            assert!(
                branch
                    .observe(opponent)
                    .known_hand
                    .contains(&branch.object_ref(id))
            );
        }
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            0
        );
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());

    raw.restore(&saved).unwrap();
    select(&mut raw, first);
    select(&mut raw, second);
    select(&mut raw, fifth);
    assert_eq!(
        raw.state.players[player.index()].library[..2],
        [fifth, first]
    );
    assert_eq!(raw.object_view(second).zone, Zone::Hand);
    assert_eq!(indexed.object_view(second).zone, Zone::Library);

    raw.restore(&saved).unwrap();
    raw.move_object(second, Zone::Exile);
    raw.move_object(second, Zone::Library);
    reject(&mut raw, Action::SelectLibrary { card: second });
    let view = raw.observe(opponent).library_selection.unwrap();
    assert!(view.remaining.iter().all(|card| card.id != second));
    select(&mut raw, first);
    select(&mut raw, fifth);
    assert_eq!(raw.object_view(second).zone, Zone::Library);
}

#[test]
fn short_libraries_complete_the_available_steps_and_auto_assign_the_last_card() {
    let _rules = fixture_rules::activate(&[(memories(), &MEMORIES)]);
    let player = Player::P1;
    for count in 0usize..=4 {
        let mut game = scenario(player);
        let initial = game.state.players[player.index()].library.clone();
        for &id in initial.iter() {
            game.move_object(id, Zone::Exile);
        }
        let cards: Vec<_> = (0..count)
            .map(|_| put(&mut game, player, "Island", Zone::Library))
            .collect();
        let spell = spawn(&mut game, player, memories(), Zone::Hand);
        cast(&mut game, spell, None);
        pass_pair(&mut game);
        let mut choices = 0usize;
        while let Some(view) = game.observe(player).library_selection {
            let chooser = if choices.is_multiple_of(2) {
                player
            } else {
                player.other()
            };
            assert_eq!(game.actor(), Some(chooser));
            assert_eq!(view.candidates.len(), count - choices);
            assert!(
                !game
                    .legal_actions()
                    .contains(&Action::FinishLibrarySelection)
            );
            select(&mut game, view.candidates[0].card);
            choices += 1;
        }
        assert_eq!(choices, count.saturating_sub(1));
        assert_eq!(game.state.pending, Pending::Priority);
        assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
        for (index, id) in cards.into_iter().rev().enumerate() {
            let zone = if index.is_multiple_of(2) {
                Zone::Hand
            } else {
                Zone::Library
            };
            assert_eq!(game.object_view(id).zone, zone);
        }
        assert_eq!(game.state.players[player.index()].library.len(), count / 2);
        if let Some(&top) = game.state.players[player.index()].library.last() {
            let known = game.observe(player.other()).known_library_top[player.index()].unwrap();
            assert_eq!(known.object, game.object_ref(top));
        }
    }
}

#[test]
fn flashback_reuses_the_same_split_and_exiles_on_resolution_or_counter() {
    let _rules = fixture_rules::activate(&[(memories(), &MEMORIES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = spawn(&mut game, player, memories(), Zone::Graveyard);
    let cast_memories = Action::AnnounceFlashback { card: spell };
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 6];
    reject(&mut game, cast_memories.clone());
    game.state.players[player.index()].mana[5] = 7;
    act(&mut game, cast_memories);
    pay(&mut game);
    assert_eq!(
        game.state.stack[0].cast.unwrap().method,
        CastMethod::Flashback
    );
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);

    let mut countered = game.clone();
    let library = countered.state.players[player.index()].library.clone();
    act(&mut countered, Action::Pass);
    let negate = put(&mut countered, player.other(), "Negate", Zone::Hand);
    cast(&mut countered, negate, Some(Target::Spell(spell)));
    settle(&mut countered);
    assert_eq!(countered.object_view(spell).zone, Zone::Exile);
    assert_eq!(countered.state.players[player.index()].library, library);
    assert!(countered.observe(player).library_selection.is_none());

    pass_pair(&mut game);
    while let Some(view) = game.observe(player).library_selection {
        select(&mut game, view.candidates[0].card);
    }
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(game.observe(player.other()).known_hand.len(), 3);
}

#[test]
fn a_prevented_hand_move_leaves_its_card_revealed_and_keeps_the_five_step_limit() {
    struct ResetPrevention;
    impl Drop for ResetPrevention {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }

    let _rules = fixture_rules::activate(&[(memories(), &MEMORIES)]);
    let _reset = ResetPrevention;
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let [first, second, third, fourth, fifth] = [
        "Island",
        "Shock",
        "Grizzly Bears",
        "Forest",
        "Swiftfoot Boots",
    ]
    .map(|name| put(&mut game, player, name, Zone::Library));
    let blocked = game.object_ref(first);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(blocked)));
    let spell = spawn(&mut game, player, memories(), Zone::Hand);
    cast(&mut game, spell, None);
    pass_pair(&mut game);

    select(&mut game, first);
    assert_eq!(game.actor(), Some(opponent));
    assert_eq!(game.object_ref(first), blocked);
    assert_eq!(game.object_view(first).zone, Zone::Library);
    assert!(
        game.observe(opponent)
            .library_selection
            .unwrap()
            .candidates
            .contains(&blocked)
    );

    select(&mut game, second);
    select(&mut game, third);
    select(&mut game, fourth);
    assert_eq!(game.actor(), Some(player));
    let view = game.observe(player).library_selection.unwrap();
    assert_eq!(view.candidates, [game.object_ref(fifth), blocked]);
    select(&mut game, fifth);
    assert_eq!(game.state.pending, Pending::Priority);
    assert_eq!(game.object_view(first).zone, Zone::Library);
    assert_eq!(
        game.state.players[player.index()].library[..2],
        [fourth, second]
    );
    for id in [third, fifth] {
        assert_eq!(game.object_view(id).zone, Zone::Hand);
    }
}
