//! Cast one half of a physical split card and preserve exact zone identity.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SPLIT: Card = Card::from_catalogue_id(2879); // Murder.
const LEFT: Card = Card::from_catalogue_id(3027); // Opt, {U}.
const RIGHT: Card = Card::from_catalogue_id(5095); // Witch's Mark, {1}{R}.
const OFFER: Card = Card::from_catalogue_id(10); // Shock.

const SPLIT_RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    extras: &CardExtras {
        split_faces: Some([LEFT, RIGHT]),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const HALF_EXTRAS: &CardExtras = &CardExtras {
    split_parent: Some(SPLIT),
    ..CardExtras::EMPTY
};
const LEFT_RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        &[Instruction::Draw(PlayerSet::Targeted, 1)],
    ),
    extras: HALF_EXTRAS,
    ..CardRules::EMPTY
};
const RIGHT_RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 2)],
    ),
    extras: HALF_EXTRAS,
    ..CardRules::EMPTY
};
const FREE_OFFER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 12,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::CastTargetedGraveyardFree],
    ),
    ..CardRules::EMPTY
};
const RULES: &[(Card, &CardRules)] = &[
    (SPLIT, &SPLIT_RULES),
    (LEFT, &LEFT_RULES),
    (RIGHT, &RIGHT_RULES),
    (OFFER, &FREE_OFFER),
];

fn split_action(card: CardId, half: SplitHalf, method: CastMethod) -> Action {
    Action::AnnounceSplit {
        card,
        half,
        method,
        flash_surcharge: false,
    }
}

#[test]
fn selected_half_controls_timing_cost_targets_and_stack_characteristics() {
    let _rules = fixture_rules::activate(RULES);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let split = spawn(&mut game, player, SPLIT, Zone::Hand);
    assert_eq!(SPLIT.fixed_mana_value(), 3);
    assert_eq!(game.object_view(split).card, SPLIT);
    reject(&mut game, Action::AnnounceCast { card: split });

    game.state.active = opponent;
    game.state.priority = player;
    reject(
        &mut game,
        split_action(split, SplitHalf::Right, CastMethod::Normal),
    );
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    let action = split_action(split, SplitHalf::Left, CastMethod::Normal);
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let before = game.snapshot();
    let mut indexed = game.clone();
    act(&mut game, action);
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    pay(&mut game);

    let spell = &game.spell_characteristics()[0];
    assert_eq!(spell.physical_card, SPLIT);
    assert_eq!(spell.face, CastFace::SplitLeft);
    assert_eq!(spell.name, LEFT.name());
    assert_eq!(spell.mana_cost, "{U}");
    assert_eq!(spell.mana_value, 1);
    assert_eq!(spell.type_line, "Instant");
    assert_eq!(spell.colors, 2);
    assert_eq!(game.object_view(split).card, LEFT);
    settle(&mut game);
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        1
    );
    assert_eq!(game.object_view(split).card, SPLIT);
    assert_eq!(game.object_view(split).zone, Zone::Graveyard);

    indexed.restore(&before).unwrap();
    indexed.state.active = player;
    indexed.state.players[player.index()].mana = [0, 0, 0, 1, 0, 1];
    act(
        &mut indexed,
        split_action(split, SplitHalf::Right, CastMethod::Normal),
    );
    pay(&mut indexed);
    let spell = &indexed.spell_characteristics()[0];
    assert_eq!(spell.face, CastFace::SplitRight);
    assert_eq!(spell.type_line, "Sorcery");
    assert_eq!(spell.mana_value, 2);
    assert_eq!(spell.colors, 8);
    settle(&mut indexed);
    assert_eq!(
        indexed.state.players[player.index()].cards_drawn_this_turn,
        2
    );
    assert_eq!(indexed.object_view(split).card, SPLIT);
}

#[test]
fn split_paid_exile_and_granted_flashback_keep_the_physical_identity() {
    let _rules = fixture_rules::activate(RULES);
    let player = Player::P1;
    let mut game = scenario(player);
    let split = spawn(&mut game, player.other(), SPLIT, Zone::Exile);
    let exact = game.object_ref(split);
    let action = split_action(split, SplitHalf::Right, CastMethod::Normal);
    reject(&mut game, action.clone());
    game.grant_exile_play_permission(player, exact, false);
    act(&mut game, action);
    pay(&mut game);
    settle(&mut game);
    let grave = game.object_ref(split);
    assert_ne!(grave, exact);
    assert_eq!(game.object_view(split).owner, player.other());
    assert_eq!(game.object_view(split).card, SPLIT);
    assert_eq!(game.object_view(split).zone, Zone::Graveyard);

    let own = spawn(&mut game, player, SPLIT, Zone::Graveyard);
    game.grant_graveyard_permission(player, game.object_ref(own), GraveyardCastKind::Flashback);
    act(
        &mut game,
        split_action(own, SplitHalf::Right, CastMethod::GrantedFlashback),
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(own).card, SPLIT);
    assert_eq!(game.object_view(own).zone, Zone::Exile);
}

#[test]
fn free_cast_during_resolution_offers_both_halves_and_halves_cannot_be_deck_cards() {
    let _rules = fixture_rules::activate(RULES);
    let player = Player::P0;
    let mut game = scenario(player);
    let split = spawn(&mut game, player.other(), SPLIT, Zone::Graveyard);
    let offer = spawn(&mut game, player, OFFER, Zone::Hand);
    cast(&mut game, offer, Some(Target::Card(split)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let object = game.object_ref(split);
    let options = [CastFace::SplitLeft, CastFace::SplitRight]
        .map(|face| Action::CastExiledDuringResolution { object, face });
    assert!(
        options
            .iter()
            .all(|action| game.legal_actions().contains(action))
    );
    reject(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    act(&mut game, options[1].clone());
    if game
        .legal_actions()
        .iter()
        .any(|action| matches!(action, Action::PayCosts { .. }))
    {
        pay(&mut game);
    }
    if game
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.object_view(split).zone, Zone::Exile);
    assert_eq!(game.object_view(split).card, SPLIT);

    let invalid = Game::new(Config {
        decks: [vec![LEFT; 7], vec![Card::Forest; 7]],
        seed: 4,
        starting_player: player,
        deck_rules: DeckRules::Freeform,
    });
    assert!(matches!(invalid, Err(Error::InvalidDeck(message)) if message.contains("split half")));
}
