//! Qualified searches and public destinations after private library selections.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::Kinds(1 | 2),
            maximum: 1,
            destination: SearchDestination::BattlefieldIfLandElseHand,
            tapped: true,
            reveal: true,
        })],
    ),
    ..CardRules::EMPTY
};

const LOOK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 6,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(1),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(6),
            destination: LibraryPickDestination::MayEnterBattlefield {
                maximum_mana_value: 2,
                keywords: Keywords::HASTE,
            },
            public: false,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn start(game: &mut Game, player: Player) {
    if game.actor() != Some(player) {
        act(game, Action::Pass);
    }
    let spell = put(game, player, "Opt", Zone::Hand);
    cast(game, spell, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn pass_end_of_turn(game: &mut Game) {
    game.state.step = Step::End;
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn qualified_search_reveals_before_lands_enter_tapped_or_creatures_go_to_hand() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &SEARCH)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let land = put(&mut game, player, "Cavern of Souls", Zone::Library);
        let creature = put(&mut game, player, "Bear Cub", Zone::Library);
        let other = put(&mut game, player, "Shock", Zone::Library);
        let enemy_land = put(&mut game, opponent, "Island", Zone::Library);
        start(&mut game, player);
        assert!(game.observe(player).search_choice.is_some());
        assert!(game.observe(opponent).search_choice.is_none());
        reject(&mut game, Action::ChooseSearch { card: other });
        reject(&mut game, Action::ChooseSearch { card: enemy_land });

        let rng = game.state.rng;
        let mut missed = game.clone();
        act(&mut missed, Action::FinishSearch);
        assert!(missed.observe(opponent).zone_reveals.is_empty());
        assert_ne!(missed.state.rng, rng);

        let mut hand = game.clone();
        act(&mut hand, Action::ChooseSearch { card: creature });
        assert!(hand.observe(opponent).zone_reveals.is_empty());
        act(&mut hand, Action::FinishSearch);
        let public = hand.observe(opponent);
        assert_eq!(
            public.zone_reveals,
            [ZoneRevealView {
                card: named("Bear Cub"),
                owner: player,
                from: Zone::Library,
            }]
        );
        assert!(
            public
                .objects
                .iter()
                .any(|object| { object.id == creature && object.zone == Zone::Hand })
        );
        assert_ne!(hand.state.rng, rng);

        act(&mut game, Action::ChooseSearch { card: land });
        assert!(game.observe(opponent).zone_reveals.is_empty());
        let snapshot = game.snapshot();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == Action::FinishSearch)
            .unwrap();
        let mut indexed = game.clone();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, Action::FinishSearch);
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&snapshot).unwrap();
        act(&mut indexed, Action::FinishSearch);
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(
            game.observe(opponent).zone_reveals,
            [ZoneRevealView {
                card: named("Cavern of Souls"),
                owner: player,
                from: Zone::Library,
            }]
        );
        assert_eq!(game.object_view(land).zone, Zone::Library);
        choose(&mut game, |action| {
            matches!(action, Action::ChooseEntryValue { .. })
        });
        let entered = game.object_view(land);
        assert_eq!(entered.zone, Zone::Battlefield);
        assert_eq!(entered.controller, player);
        assert!(entered.tapped);
        assert_ne!(game.state.rng, rng);
    }
}

fn select(game: &mut Game, card: CardId) {
    act(game, Action::SelectLibrary { card });
    act(game, Action::FinishLibrarySelection);
}

fn assert_remainder(game: &Game, player: Player, mut cards: Vec<CardId>, next: CardId) {
    let library = &game.state.players[player.index()].library;
    let mut bottom = library[..cards.len()].to_vec();
    bottom.sort_unstable();
    cards.sort_unstable();
    assert_eq!(bottom, cards);
    assert_eq!(library.last(), Some(&next));
}

struct ClearEntryRedirect;

impl Drop for ClearEntryRedirect {
    fn drop(&mut self) {
        zone_replacements::TEST_REDIRECT_ENTRY_TO_EXILE.with(|slot| slot.set(None));
    }
}

#[test]
fn optional_entry_reveals_one_creature_and_grants_haste_to_the_exact_entrant() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &LOOK)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(opponent);
        let [
            outside,
            cheap,
            expensive,
            forest,
            other_creature,
            mountain,
            island,
        ] = [
            "Bear Cub",
            "Llanowar Elves",
            "Hill Giant",
            "Forest",
            "Grizzly Bears",
            "Mountain",
            "Island",
        ]
        .map(|name| put(&mut game, player, name, Zone::Library));
        let rest = vec![expensive, forest, other_creature, mountain, island];
        start(&mut game, player);
        assert!(game.observe(opponent).library_selection.is_none());
        let look = game.observe(player).library_selection.unwrap();
        assert_eq!(look.remaining.len(), 6);
        assert_eq!(
            look.candidates.iter().map(|r| r.card).collect::<Vec<_>>(),
            [other_creature, expensive, cheap]
        );
        reject(&mut game, Action::SelectLibrary { card: outside });
        reject(&mut game, Action::SelectLibrary { card: forest });

        let mut none = game.clone();
        act(&mut none, Action::FinishLibrarySelection);
        assert!(none.observe(opponent).zone_reveals.is_empty());
        let mut all = rest.clone();
        all.push(cheap);
        assert_remainder(&none, player, all, outside);

        let mut too_expensive = game.clone();
        select(&mut too_expensive, expensive);
        assert_eq!(too_expensive.state.pending, Pending::Priority);
        assert_eq!(too_expensive.object_view(expensive).zone, Zone::Hand);
        assert!(
            too_expensive
                .observe(opponent)
                .objects
                .iter()
                .any(|o| o.id == expensive)
        );

        select(&mut game, cheap);
        let observed = game.observe(opponent);
        let public = observed.library_selection.unwrap();
        assert_eq!(public.selected[0].id, cheap);
        assert!(public.remaining.is_empty() && public.candidates.is_empty());
        assert_eq!(
            observed.zone_reveals,
            [ZoneRevealView {
                card: named("Llanowar Elves"),
                owner: player,
                from: Zone::Library,
            }]
        );
        assert_eq!(game.object_view(cheap).zone, Zone::Library);

        let mut hand = game.clone();
        act(&mut hand, Action::ChooseOptional { accept: false });
        assert_eq!(hand.object_view(cheap).zone, Zone::Hand);
        assert_remainder(&hand, player, rest.clone(), outside);

        let accept = Action::ChooseOptional { accept: true };
        let mut redirected = game.clone();
        let original = redirected.object_ref(cheap);
        {
            let _clear = ClearEntryRedirect;
            zone_replacements::TEST_REDIRECT_ENTRY_TO_EXILE.with(|slot| slot.set(Some(original)));
            act(&mut redirected, accept.clone());
        }
        assert_eq!(redirected.object_view(cheap).zone, Zone::Hand);
        assert_eq!(
            redirected.object_ref(cheap).generation,
            original.generation + 2
        );
        assert!(
            !redirected
                .object_view(cheap)
                .keywords
                .contains(Keywords::HASTE)
        );
        assert_remainder(&redirected, player, rest.clone(), outside);

        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == accept)
            .unwrap();
        let saved = game.snapshot();
        let mut indexed = game.clone();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, accept.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, accept);
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_remainder(&game, player, rest, outside);
        assert_eq!(game.state.rng, hand.state.rng);
        let entered = game.object_view(cheap);
        assert_eq!(entered.zone, Zone::Battlefield);
        assert!(entered.summoning_sick);
        assert!(entered.keywords.contains(Keywords::HASTE));

        pass_end_of_turn(&mut game);
        assert!(!game.object_view(cheap).keywords.contains(Keywords::HASTE));
    }
}
