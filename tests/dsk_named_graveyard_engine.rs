//! An exact named graveyard cost is committed before a qualified zone search.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SOURCE: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileSourceAndNamed {
                    name: "Divination",
                    amount: 2,
                },
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SearchGraveyardHandAndOrLibrary(
            SearchFilter::Name("Grizzly Bears"),
        )],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        exhaust: false,
        cycling: false,
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const EXILE_PROHIBITOR: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::GraveyardCardsCannotBeExiled],
    }],
    ..CardRules::EMPTY
};

fn fixture() -> Card {
    Card::from_name("Divination").unwrap()
}

fn announce(card: CardId) -> Action {
    Action::AnnounceActivate { card, ability: 0 }
}

fn choose_cost(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::ChooseAdditionalCost { object });
}

fn zones(graveyard: bool, hand: bool, library: bool) -> Action {
    Action::ChooseGraveyardHandLibrarySearchZones {
        graveyard,
        hand,
        library,
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn submit_cost(game: &mut Game) {
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn start_search(game: &mut Game, player: Player) {
    let [source, first, second] = [(); 3].map(|_| put(game, player, "Divination", Zone::Graveyard));
    act(game, announce(source));
    choose_cost(game, first);
    choose_cost(game, second);
    submit_cost(game);
    pass_pair(game);
    assert!(
        game.observe(player)
            .hand_library_search
            .unwrap()
            .choosing_zones
    );
}

#[test]
fn the_cost_uses_three_exact_physical_cards_and_cannot_be_refunded_in_response() {
    let prohibitor = Card::from_name("Bear Cub").unwrap();
    let _rules = fixture_rules::activate(&[(fixture(), &SOURCE), (prohibitor, &EXILE_PROHIBITOR)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Divination", Zone::Graveyard);
    let first = put(&mut game, player, "Divination", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Divination", Zone::Graveyard);
    let unrelated = put(&mut game, player, "Island", Zone::Graveyard);
    let copy = put(&mut game, player, "Divination", Zone::Graveyard);
    game.state.objects[copy as usize].copy_kind = CopyKind::Card;
    reject(&mut game, announce(source));

    let second = put(&mut game, player, "Divination", Zone::Graveyard);
    let mut prohibited = game.clone();
    spawn(&mut prohibited, opponent, prohibitor, Zone::Battlefield);
    reject(&mut prohibited, announce(source));
    game.state.step = Step::BeginCombat;
    reject(&mut game, announce(source));
    game.state.step = Step::Main1;
    let found = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let binder = put(&mut game, opponent, "Tishana's Tidebinder", Zone::Hand);

    act(&mut game, announce(source));
    assert!(game.observe(player).stack.is_empty());
    let candidates = game.observe(player).additional_cost.unwrap().candidates;
    assert_eq!(
        candidates,
        [game.object_ref(first), game.object_ref(second)]
    );
    for invalid in [source, foreign, unrelated, copy] {
        let object = game.object_ref(invalid);
        reject(&mut game, Action::ChooseAdditionalCost { object });
    }
    choose_cost(&mut game, first);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert!(game.observe(player).stack.is_empty());
    reject(&mut game, Action::Pass);
    let object = game.object_ref(first);
    reject(&mut game, Action::ChooseAdditionalCost { object });

    let saved = game.snapshot();
    let stale = game.object_ref(second);
    game.move_object(second, Zone::Exile);
    game.move_object(second, Zone::Graveyard);
    reject(&mut game, Action::ChooseAdditionalCost { object: stale });
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    game.restore(&saved).unwrap();
    choose_cost(&mut game, second);
    submit_cost(&mut game);
    let observation = game.observe(player);
    let item = observation.stack.last().unwrap();
    assert!(matches!(
        item.kind,
        StackKind::ActivatedAbility { ability: 0 }
    ));
    assert!(item.targets.is_empty());
    for paid in [source, first, second] {
        assert_eq!(game.object_view(paid).zone, Zone::Exile);
    }
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);

    act(&mut game, Action::Pass);
    cast(&mut game, binder, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Stack(item.id),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.object_view(found).zone, Zone::Graveyard);
    assert!(game.observe(player).hand_library_search.is_none());
    for paid in [source, first, second] {
        assert_eq!(game.object_view(paid).zone, Zone::Exile);
    }
}

#[test]
fn selected_zones_have_private_cards_and_only_a_public_match_requires_a_find() {
    let _rules = fixture_rules::activate(&[(fixture(), &SOURCE)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let observer = put(
        &mut game,
        opponent,
        "Wan Shi Tong, Librarian",
        Zone::Battlefield,
    );
    let grave = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let hand = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let library = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let foreign = put(&mut game, opponent, "Grizzly Bears", Zone::Graveyard);
    let unrelated = put(&mut game, player, "Bear Cub", Zone::Hand);
    let original_library = game.state.players[player.index()].library.clone();
    start_search(&mut game, player);

    let initial = game.snapshot();
    assert_eq!(
        game.legal_actions()
            .iter()
            .filter(|action| matches!(action, Action::ChooseGraveyardHandLibrarySearchZones { .. }))
            .count(),
        7
    );
    reject(&mut game, Action::FinishSearch);
    reject(&mut game, zones(false, false, false));
    for (choice, found) in [
        (zones(true, false, false), grave),
        (zones(false, true, false), hand),
        (zones(false, false, true), library),
    ] {
        game.restore(&initial).unwrap();
        act(&mut game, choice);
        let public = game.observe(opponent).hand_library_search.unwrap();
        assert!(public.cards.is_none());
        assert!(public.selected.is_none());
        assert_eq!(public.minimum, Some(u16::from(found == grave)));
        reject(&mut game, Action::ChooseSearch { card: foreign });
        reject(&mut game, Action::ChooseSearch { card: unrelated });
        for other in [grave, hand, library].into_iter().filter(|&id| id != found) {
            reject(&mut game, Action::ChooseSearch { card: other });
        }
        if found == grave {
            reject(&mut game, Action::FinishSearch);
        }
        act(&mut game, Action::ChooseSearch { card: found });
        assert!(
            game.observe(opponent)
                .hand_library_search
                .unwrap()
                .selected
                .is_none()
        );
        act(&mut game, Action::FinishSearch);
        settle(&mut game);
        assert_eq!(game.object_view(found).zone, Zone::Battlefield);
        assert_eq!(game.object_view(found).controller, player);
        assert!(!game.object_view(found).tapped);
        assert_eq!(
            game.counter_count(observer, CounterKind::PlusOne),
            u64::from(found == library)
        );
        if found != library {
            assert_eq!(game.state.players[player.index()].library, original_library);
        }
    }

    for searched_library in [false, true] {
        game.restore(&initial).unwrap();
        act(&mut game, zones(false, !searched_library, searched_library));
        act(&mut game, Action::FinishSearch);
        settle(&mut game);
        assert_eq!(game.object_view(grave).zone, Zone::Graveyard);
        assert_eq!(game.object_view(hand).zone, Zone::Hand);
        assert_eq!(game.object_view(library).zone, Zone::Library);
        assert_eq!(
            game.counter_count(observer, CounterKind::PlusOne),
            u64::from(searched_library)
        );
        if !searched_library {
            assert_eq!(game.state.players[player.index()].library, original_library);
        } else {
            assert_ne!(game.state.players[player.index()].library, original_library);
        }
    }
}

#[test]
fn adding_a_zone_and_finding_one_card_owns_an_exact_branch() {
    let _rules = fixture_rules::activate(&[(fixture(), &SOURCE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let grave = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let hand = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let library = put(&mut game, player, "Grizzly Bears", Zone::Library);
    start_search(&mut game, player);
    act(&mut game, zones(false, false, true));
    let view = game.observe(player).hand_library_search.unwrap();
    assert!(
        view.cards
            .unwrap()
            .iter()
            .any(|object| object.id == library)
    );
    let initial = game.snapshot();

    let action = zones(true, true, true);
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, action);
    assert_eq!(indexed.snapshot(), game.snapshot());
    reject(&mut game, zones(false, true, false));
    reject(&mut game, Action::FinishSearch);
    act(&mut game, Action::ChooseSearch { card: hand });
    let exact = game.object_ref(hand);
    let selected = game.snapshot();
    game.move_object(hand, Zone::Exile);
    game.move_object(hand, Zone::Hand);
    assert_ne!(game.object_ref(hand), exact);
    reject(&mut game, Action::FinishSearch);

    game.restore(&selected).unwrap();
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(hand).zone, Zone::Battlefield);
    assert_eq!(game.object_view(grave).zone, Zone::Graveyard);
    assert_eq!(game.object_view(library).zone, Zone::Library);
    game.restore(&initial).unwrap();
    assert!(
        game.observe(player)
            .hand_library_search
            .unwrap()
            .selected
            .is_none()
    );
    assert_eq!(game.object_view(hand).zone, Zone::Hand);
}
