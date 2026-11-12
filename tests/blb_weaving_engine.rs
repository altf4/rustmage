//! Repeated untargeted copy choices and a simultaneous nonland nontoken return.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SELECT: Instruction = Instruction::ChooseObjects {
    kind: EffectChoiceKind::RememberPermanents {
        require_battlefield_source: false,
    },
    players: PlayerSet::Controller,
    filter: SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 1 | 16,
        nonland: false,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
    amount: 1,
    optional_skip: None,
};
const COPY_TWICE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            SELECT,
            Instruction::CreateCopyOfRememberedPermanent,
            SELECT,
            Instruction::CreateCopyOfRememberedPermanent,
        ],
    ),
    ..CardRules::EMPTY
};
const RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ReturnNonlandNontokenPermanents],
    ),
    ..CardRules::EMPTY
};

fn remember(game: &Game, card: CardId) -> Action {
    Action::ChooseEffectObject {
        object: game.object_ref(card),
    }
}

#[test]
fn the_second_choice_sees_the_first_copy_and_retains_its_own_exact_subject() {
    let definition = Card::from_name("Divination").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &COPY_TWICE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let blueprint = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    game.add_counter(blueprint, CounterKind::Charge, 1);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, spell, None);
    for _ in 0..8 {
        if game.legal_actions().contains(&remember(&game, blueprint)) {
            break;
        }
        act(&mut game, Action::Pass);
    }
    let enemy_choice = remember(&game, enemy);
    let land_choice = remember(&game, land);
    reject(&mut game, enemy_choice);
    reject(&mut game, land_choice);
    let snapshot = game.snapshot();
    let chosen = remember(&game, blueprint);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == chosen)
        .unwrap();
    let mut twin = game.clone();
    twin.apply_index(player, twin.revision(), index).unwrap();
    act(&mut game, chosen);
    assert_eq!(game.snapshot(), twin.snapshot());
    let first = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| object.is_token())
        .unwrap();
    let choose_first = remember(&game, first.id);
    act(&mut game, choose_first);
    settle(&mut game);
    let tokens = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|object| object.is_token())
        .collect::<Vec<_>>();
    assert_eq!(tokens.len(), 2);
    assert!(tokens.iter().all(|object| {
        object.card == first.card && game.counter_count(object.id, CounterKind::Charge) == 0
    }));
    twin.restore(&snapshot).unwrap();
    assert_eq!(twin.snapshot(), snapshot);
}

#[test]
fn the_return_keeps_lands_and_tokens_and_returns_stolen_cards_to_their_owner() {
    let definition = Card::from_name("Divination").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &RETURN)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let stolen = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.record_control(game.object_ref(stolen), player, None);
    let other = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let token = game.state.objects.len() as CardId;
    game.create_tokens(player, Card::DrakeToken, 1, false);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.object_view(stolen).zone, Zone::Hand);
    assert_eq!(game.object_view(stolen).owner, player.other());
    assert_eq!(game.object_view(other).zone, Zone::Hand);
    assert!(game.on_battlefield(land));
    assert!(game.on_battlefield(token));
}
