//! Dynamic private library counts and public typed selection with exact free casts.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PRIVATE_SELECTION: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::ChosenX,
                    filter: LibrarySelectionFilter::Any,
                    optional: false,
                    reveal: false,
                    remainder: LibraryRemainder::Graveyard,
                },
                look: LibraryLook::Amount(AmountValue::Multiple {
                    value: &AmountValue::ChosenX,
                    factor: 2,
                }),
                destination: LibraryPickDestination::Hand,
                public: false,
                maximum_mana_value: None,
            }),
            Instruction::LoseLifeScaled(PlayerSet::Controller, AmountValue::ChosenX),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const PUBLIC_SELECTION: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::SelectLibraryToExileRemember(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::DistinctCardTypes(
                        1 | 2 | 4 | 8 | 16 | 32 | 64 | 128 | 256,
                    ),
                    filter: LibrarySelectionFilter::Any,
                    optional: true,
                    reveal: true,
                    remainder: LibraryRemainder::Graveyard,
                },
                look: LibraryLook::ChosenX,
                destination: LibraryPickDestination::Exile,
                public: true,
                maximum_mana_value: None,
            }),
            Instruction::CastOneRememberedExileFreeThenRestToHand { minimum: 4 },
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn spell_card() -> Card {
    Card::from_name("Exsanguinate").unwrap()
}

fn replace_library(game: &mut Game, player: Player, names: &[&str]) -> Vec<CardId> {
    let previous = game.state.players[player.index()].library.clone();
    for card in previous {
        game.move_object(card, Zone::Exile);
    }
    let mut cards: Vec<_> = names
        .iter()
        .rev()
        .map(|name| put(game, player, name, Zone::Library))
        .collect();
    cards.reverse();
    cards
}

fn cast_x(game: &mut Game, player: Player, x: u32) {
    let card = spawn(game, player, spell_card(), Zone::Hand);
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseX { value: x });
    pay(game);
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn chosen_x_is_a_private_exact_count_and_life_loss_is_independent_of_availability() {
    let _rules = fixture_rules::activate(&[(spell_card(), &PRIVATE_SELECTION)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let cards = replace_library(
            &mut game,
            player,
            &["Island", "Swamp", "Grizzly Bears", "Hill Giant", "Plains"],
        );
        cast_x(&mut game, player, 2);
        let view = game.observe(player).library_selection.unwrap();
        assert_eq!((view.minimum, view.maximum), (2, 2));
        assert_eq!(
            view.remaining
                .iter()
                .map(|object| object.id)
                .collect::<Vec<_>>(),
            cards[..4]
        );
        assert!(game.observe(player.other()).library_selection.is_none());
        assert!(
            game.observe(player.other())
                .objects
                .iter()
                .all(|object| !cards.contains(&object.id))
        );
        reject(&mut game, Action::SelectLibrary { card: cards[4] });
        reject(&mut game, Action::FinishLibrarySelection);
        act(&mut game, Action::SelectLibrary { card: cards[0] });
        reject(&mut game, Action::FinishLibrarySelection);
        let saved = game.snapshot();
        let second = Action::SelectLibrary { card: cards[2] };
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == second)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, game.revision(), index).unwrap();
        act(&mut game, second);
        for branch in [&mut game, &mut indexed] {
            act(branch, Action::FinishLibrarySelection);
            assert_eq!(branch.state.players[player.index()].life, 18);
            for card in [cards[0], cards[2]] {
                assert_eq!(branch.object_view(card).zone, Zone::Hand);
                assert!(
                    !branch
                        .observe(player.other())
                        .objects
                        .iter()
                        .any(|object| object.id == card)
                );
            }
            for card in [cards[1], cards[3]] {
                assert_eq!(branch.object_view(card).zone, Zone::Graveyard);
            }
            assert_eq!(branch.object_view(cards[4]).zone, Zone::Library);
        }
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        assert_eq!(indexed.snapshot(), saved);

        for (x, names) in [
            (0, &["Island"][..]),
            (3, &["Island", "Swamp"][..]),
            (3, &[][..]),
        ] {
            let mut short = scenario(player);
            let cards = replace_library(&mut short, player, names);
            cast_x(&mut short, player, x);
            assert_eq!(short.state.pending, Pending::Priority);
            assert_eq!(short.state.players[player.index()].life, 20 - i128::from(x));
            for card in cards {
                assert_eq!(
                    short.object_view(card).zone,
                    if x == 0 { Zone::Library } else { Zone::Hand }
                );
            }
        }
    }
}

fn free_cast(game: &Game, card: CardId) -> Action {
    Action::CastExiledDuringResolution {
        object: game.object_ref(card),
        face: CastFace::Normal,
    }
}

#[test]
fn public_distinct_types_enable_one_optional_cast_from_actual_exiles() {
    let _rules = fixture_rules::activate(&[(spell_card(), &PUBLIC_SELECTION)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let cards = replace_library(
        &mut game,
        player,
        &[
            "Gingerbrute",
            "Grizzly Bears",
            "Hill Giant",
            "Forest",
            "Divination",
            "Opt",
        ],
    );
    cast_x(&mut game, player, 6);
    let public = game.observe(player.other()).library_selection.unwrap();
    assert!(public.public);
    assert_eq!(
        public
            .remaining
            .iter()
            .map(|object| object.id)
            .collect::<Vec<_>>(),
        cards
    );
    for index in [0, 1, 3] {
        act(&mut game, Action::SelectLibrary { card: cards[index] });
    }
    reject(&mut game, Action::SelectLibrary { card: cards[2] });
    let mut three = game.clone();
    put(
        &mut three,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    act(&mut three, Action::FinishLibrarySelection);
    assert_eq!(three.state.pending, Pending::Priority);
    assert_eq!(three.object_view(cards[5]).zone, Zone::Exile);
    for index in [0, 1, 3] {
        assert_eq!(three.object_view(cards[index]).zone, Zone::Hand);
    }

    act(&mut game, Action::SelectLibrary { card: cards[4] });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.state.pending, Pending::ResolutionCasts { player });
    assert_eq!(game.object_view(cards[5]).zone, Zone::Graveyard);
    let land = free_cast(&game, cards[3]);
    reject(&mut game, land);
    let mut stale = game.clone();
    let invalidated = free_cast(&stale, cards[0]);
    stale.move_object(cards[0], Zone::Graveyard);
    stale.move_object(cards[0], Zone::Exile);
    reject(&mut stale, invalidated);
    let replacement = free_cast(&stale, cards[0]);
    reject(&mut stale, replacement);
    act(&mut stale, Action::FinishResolutionCasts);
    assert_eq!(stale.object_view(cards[0]).zone, Zone::Exile);

    let mut declined = game.clone();
    act(&mut declined, Action::FinishResolutionCasts);
    assert_eq!(declined.object_view(cards[4]).zone, Zone::Hand);

    let cast = free_cast(&game, cards[4]);
    act(&mut game, cast);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let second_cast = free_cast(&game, cards[1]);
    reject(&mut game, second_cast);
    act(&mut game, Action::FinishResolutionCasts);
    let spell = game.state.stack.last().unwrap();
    assert_eq!(spell.source().unwrap().id, cards[4]);
    assert_eq!(spell.cast.unwrap().payment, [0; 6]);
    for index in [0, 1, 3] {
        assert_eq!(game.object_view(cards[index]).zone, Zone::Hand);
        assert!(
            game.observe(player.other())
                .objects
                .iter()
                .any(|object| object.id == cards[index])
        );
    }
}
