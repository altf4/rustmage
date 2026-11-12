//! One nontargeting destruction batch, followed by exact recipients' searches.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DESTROY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::DestroyNonbasicLandsForEachPlayer,
            players: PlayerSet::Controller,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 2,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
            amount: 0,
            optional_skip: None,
        }],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn start(game: &mut Game, player: Player) {
    let spell = put(game, player, "Opt", Zone::Hand);
    cast(game, spell, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.pending, Pending::EffectChoice { player });
}

fn land(game: &Game, card: CardId) -> Action {
    Action::ChooseEffectObject {
        object: game.object_ref(card),
    }
}

#[test]
fn the_chooser_can_destroy_one_nontargeted_land_per_controller() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &DESTROY)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let stolen = put(&mut game, opponent, "Evolving Wilds", Zone::Battlefield);
    let other_own = put(&mut game, player, "Evolving Wilds", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Evolving Wilds", Zone::Battlefield);
    let basic = put(&mut game, player, "Forest", Zone::Battlefield);
    game.record_control(game.object_ref(stolen), player, None);
    game.state.objects[enemy as usize].gained_keywords = Keywords::HEXPROOF;
    put(&mut game, opponent, "Rest in Peace", Zone::Battlefield);
    start(&mut game, player);
    assert_eq!(game.observe(player).effect_choice.unwrap().amount, 2);
    let basic_choice = land(&game, basic);
    reject(&mut game, basic_choice);
    let select_stolen = land(&game, stolen);
    act(&mut game, select_stolen);
    let duplicate_controller = land(&game, other_own);
    reject(&mut game, duplicate_controller);
    let select_enemy = land(&game, enemy);
    act(&mut game, select_enemy);
    assert!(game.on_battlefield(stolen) && game.on_battlefield(enemy));

    let checkpoint = game.snapshot();
    let finish = Action::FinishEffectChoice;
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == finish)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, finish);
    assert_eq!(game.snapshot(), indexed.snapshot());
    for id in [stolen, enemy] {
        assert_eq!(game.object_view(id).zone, Zone::Exile);
    }
    assert_eq!(game.actor(), Some(player));
    assert_eq!(game.state.pending, Pending::OptionalSearch(player));
    let declined_library = game.state.players[player.index()].library.clone();
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.state.players[player.index()].library, declined_library);
    assert_eq!(game.state.pending, Pending::OptionalSearch(opponent));

    let nonbasic = put(&mut game, opponent, "Evolving Wilds", Zone::Library);
    let basic = put(&mut game, opponent, "Island", Zone::Library);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert!(game.observe(player).search_choice.is_none());
    assert!(game.observe(opponent).search_choice.is_some());
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    act(&mut game, Action::ChooseSearch { card: basic });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
    assert!(game.object_view(basic).tapped);
    assert!(game.on_battlefield(other_own));

    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.on_battlefield(stolen) && indexed.on_battlefield(enemy));
}

#[test]
fn declined_or_prevented_destruction_offers_no_search() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &DESTROY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let land_id = put(
        &mut game,
        player.other(),
        "Evolving Wilds",
        Zone::Battlefield,
    );
    game.state.objects[land_id as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    start(&mut game, player);
    let mut declined = game.clone();
    act(&mut declined, Action::FinishEffectChoice);
    assert_eq!(declined.state.pending, Pending::Priority);
    assert!(declined.on_battlefield(land_id));

    let choice = land(&game, land_id);
    act(&mut game, choice);
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.state.pending, Pending::Priority);
    assert!(game.on_battlefield(land_id));
    assert!(game.observe(player).optional_search_sequence.is_none());
}
