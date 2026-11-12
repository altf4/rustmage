//! Independent, private library picks followed by a public entry decision.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const LOOK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 5,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(1),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(5),
            destination: LibraryPickDestination::MayEnterBattlefieldOnOwnTurn,
            public: false,
            maximum_mana_value: Some(AmountValue::Fixed(3)),
        })],
    ),
    ..CardRules::EMPTY
};

fn opt() -> Card {
    Card::from_name("Opt").unwrap()
}

fn start(game: &mut Game, player: Player) -> [CardId; 6] {
    let cards = [
        "Bear Cub",
        "Grizzly Bears",
        "Llanowar Elves",
        "Hill Giant",
        "Adaptive Automaton",
        "Plains",
    ]
    .map(|name| put(game, player, name, Zone::Library));
    if player != game.state.active {
        act(game, Action::Pass);
    }
    let spell = spawn(game, player, opt(), Zone::Hand);
    cast(game, spell, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    cards
}

fn select(game: &mut Game, card: CardId) {
    act(game, Action::SelectLibrary { card });
    act(game, Action::FinishLibrarySelection);
}

fn assert_random_bottom(game: &Game, player: Player, mut expected: Vec<CardId>, outside: CardId) {
    let library = &game.state.players[player.index()].library;
    let mut bottom = library[..expected.len()].to_vec();
    bottom.sort_unstable();
    expected.sort_unstable();
    assert_eq!(bottom, expected);
    assert_eq!(library.last(), Some(&outside));
}

#[test]
fn reveal_and_entry_are_independent_owned_choices_with_public_selected_card_only() {
    let _rules = fixture_rules::activate(&[(opt(), &LOOK)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let [outside, bears, elves, giant, automaton, land] = start(&mut game, player);
    let look = game.observe(player).library_selection.unwrap();
    assert_eq!(
        look.remaining
            .iter()
            .map(|card| card.id)
            .collect::<Vec<_>>(),
        [land, automaton, giant, elves, bears]
    );
    assert_eq!(
        look.candidates
            .iter()
            .map(|card| card.card)
            .collect::<Vec<_>>(),
        [automaton, elves, bears]
    );
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: outside });
    reject(&mut game, Action::SelectLibrary { card: giant });
    reject(&mut game, Action::SelectLibrary { card: land });
    reject(&mut game, Action::ChooseOptional { accept: true });

    let mut no_pick = game.clone();
    act(&mut no_pick, Action::FinishLibrarySelection);
    assert_eq!(no_pick.state.pending, Pending::Priority);
    assert!(no_pick.observe(player.other()).zone_reveals.is_empty());
    assert_random_bottom(
        &no_pick,
        player,
        vec![land, automaton, giant, elves, bears],
        outside,
    );

    select(&mut game, automaton);
    assert_eq!(game.object_view(automaton).zone, Zone::Library);
    let public = game.observe(player.other());
    let reveal = public.library_selection.unwrap();
    assert_eq!(reveal.selected[0].id, automaton);
    assert!(
        reveal.remaining.is_empty() && reveal.candidates.is_empty() && reveal.bottom.is_empty()
    );
    assert_eq!(
        public.zone_reveals,
        [ZoneRevealView {
            card: Card::from_name("Adaptive Automaton").unwrap(),
            owner: player,
            from: Zone::Library,
        }]
    );
    assert!(
        public
            .objects
            .iter()
            .all(|object| ![land, giant, elves, bears].contains(&object.id))
    );
    reject(&mut game, Action::FinishLibrarySelection);

    let snapshot = game.snapshot();
    let mut hand = game.clone();
    act(&mut hand, Action::ChooseOptional { accept: false });
    assert_eq!(hand.object_view(automaton).zone, Zone::Hand);
    assert!(
        hand.observe(player.other())
            .objects
            .iter()
            .any(|card| card.id == automaton)
    );
    assert_random_bottom(&hand, player, vec![land, giant, elves, bears], outside);

    let accept = Action::ChooseOptional { accept: true };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == accept)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, accept);
    assert_eq!(game.snapshot(), indexed.snapshot());
    for branch in [&mut game, &mut indexed] {
        let reveal = branch.observe(player.other()).library_selection.unwrap();
        assert_eq!(reveal.selected[0].id, automaton);
        assert!(reveal.remaining.is_empty());
        choose(branch, |action| {
            matches!(action, Action::ChooseEntryValue { .. })
        });
        assert_eq!(branch.object_view(automaton).zone, Zone::Battlefield);
        assert!(!branch.object_view(automaton).tapped);
        assert_random_bottom(branch, player, vec![land, giant, elves, bears], outside);
        assert_eq!(branch.state.rng, hand.state.rng);
    }
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&snapshot).unwrap();
    assert_eq!(indexed.snapshot(), snapshot);
}

#[test]
fn off_turn_reveal_goes_directly_to_known_hand() {
    let _rules = fixture_rules::activate(&[(opt(), &LOOK)]);
    let player = Player::P1;
    let mut game = scenario(player.other());
    let [outside, bears, elves, giant, automaton, land] = start(&mut game, player);
    select(&mut game, bears);
    assert_eq!(game.state.pending, Pending::Priority);
    assert_eq!(game.object_view(bears).zone, Zone::Hand);
    let public = game.observe(player.other());
    assert!(public.library_selection.is_none());
    assert!(public.objects.iter().any(|card| card.id == bears));
    assert_eq!(
        public.zone_reveals,
        [ZoneRevealView {
            card: Card::from_name("Grizzly Bears").unwrap(),
            owner: player,
            from: Zone::Library,
        }]
    );
    assert_random_bottom(&game, player, vec![land, automaton, giant, elves], outside);
}
