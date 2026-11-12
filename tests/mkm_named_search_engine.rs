//! Optional private zone searches with independent exact name slots.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const NAMES: [&str; 2] = ["Short Bow", "Swiftfoot Boots"];
const FIND: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchGraveyardHandLibraryForNames(NAMES)],
    ),
    ..CardRules::EMPTY
};
const WATCH: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersBatchYou {
            predicate: EntryPredicate::NontokenEquipment,
            other: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn begin(game: &mut Game, player: Player, source: Card) {
    let spell = spawn(game, player, source, Zone::Hand);
    cast(game, spell, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert!(
        game.observe(player)
            .hand_library_search
            .unwrap()
            .choosing_zones
    );
}

fn zones(graveyard: bool, hand: bool, library: bool) -> Action {
    Action::ChooseGraveyardHandLibrarySearchZones {
        graveyard,
        hand,
        library,
    }
}

fn finish_timestamps(game: &mut Game) {
    while let Some(action) = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseEntryTimestamp { .. }))
    {
        act(game, action);
    }
}

#[test]
fn one_card_per_name_enters_in_one_batch_across_the_selected_zones() {
    let spell = Card::from_name("Opt").unwrap();
    let observer = Card::from_name("Authority of the Consuls").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &FIND), (observer, &WATCH)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    spawn(&mut game, player, observer, Zone::Battlefield);
    let bow = put(&mut game, player, NAMES[0], Zone::Hand);
    let duplicate = put(&mut game, player, NAMES[0], Zone::Graveyard);
    let boots = put(&mut game, player, NAMES[1], Zone::Graveyard);
    let library_boots = put(&mut game, player, NAMES[1], Zone::Library);
    let opposing_bow = put(&mut game, opponent, NAMES[0], Zone::Graveyard);
    let rng = game.state.rng;
    begin(&mut game, player, spell);
    act(&mut game, zones(true, true, false));
    for card in [library_boots, opposing_bow] {
        reject(&mut game, Action::ChooseSearch { card });
    }
    act(&mut game, Action::ChooseSearch { card: bow });
    reject(&mut game, Action::ChooseSearch { card: duplicate });
    act(&mut game, Action::ChooseSearch { card: boots });

    let view = game.observe(player).hand_library_search.unwrap();
    assert_eq!(view.names, Some(NAMES));
    assert_eq!(
        view.named_selections,
        Some([bow, boots].map(|card| Some(game.object_ref(card))))
    );
    let public = game.observe(opponent).hand_library_search.unwrap();
    assert!(public.cards.is_none() && public.named_selections.is_none());
    assert_eq!(game.object_view(bow).zone, Zone::Hand);
    assert_eq!(game.object_view(boots).zone, Zone::Graveyard);

    let root = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == Action::FinishSearch)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&root).unwrap();
    assert_eq!(indexed.snapshot(), root);
    finish_timestamps(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(bow).zone, Zone::Battlefield);
    assert_eq!(game.object_view(boots).zone, Zone::Battlefield);
    assert_eq!(game.object_view(duplicate).zone, Zone::Graveyard);
    assert_eq!(game.object_view(opposing_bow).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.state.rng, rng);
}

#[test]
fn choosing_to_search_the_library_shuffles_even_if_nothing_is_found() {
    let spell = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &FIND)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let rng = game.state.rng;
    begin(&mut game, player, spell);
    let mut declined = game.clone();
    act(&mut declined, Action::FinishSearch);
    assert_eq!(declined.state.rng, rng);

    act(&mut game, zones(false, false, true));
    act(&mut game, Action::FinishSearch);
    assert_ne!(game.state.rng, rng);
}
