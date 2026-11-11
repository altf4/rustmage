//! Ordered nonmana X, projected artifact funding and owned exact selections.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const TWO_GENERIC: Mana = [0, 0, 0, 0, 0, 2];
const TREASURE: Card = Card::from_catalogue_id(5302);

const TAP_MANA: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Tagged {
        production: &ManaProduction::Fixed(WHITE),
        quality: &ManaQuality {
            restriction: ManaRestriction::ArtifactSpellOrArtifactActivation,
            spell_bonus: ManaSpellBonus::None,
            retention: ManaRetention::Step,
        },
    })],
    sorcery: false,
    once_per_turn: false,
};

const RETURN: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        colored: WHITE,
        generic: 2,
        mana_component: true,
        tap: true,
        additional: Some(AdditionalCost {
            kind: EffectChoiceKind::ExilePermanentsForTargetManaValue { minimum: None },
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 16,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 16,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    })),
    program: &[Instruction::ReturnToBattlefield],
    sorcery: true,
    ..TAP_MANA
};

const FOUNDRY: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[TAP_MANA, RETURN],
    ..CardRules::EMPTY
};
const FUNDER: CardRules = CardRules {
    activated: &[TAP_MANA],
    ..FOUNDRY
};

fn definition(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn activate(source: CardId) -> Action {
    Action::AnnounceActivate {
        card: source,
        ability: 1,
        counter_payment: Counters::default(),
    }
}

fn mana(source: CardId) -> Action {
    Action::ActivateMana {
        card: source,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: WHITE,
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn material(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn open_additional_cost(game: &mut Game) {
    let clause = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional && !row.paid)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause });
}

#[test]
fn only_fundable_graveyard_targets_offer_ordered_live_mana_value_payments() {
    let boots = definition("Swiftfoot Boots");
    let _rules = fixture_rules::activate(&[(definition("Short Bow"), &FOUNDRY), (boots, &FUNDER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let cheap = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Candy Trail", Zone::Battlefield);
    let foe_target = put(&mut game, player.other(), "Braided Net", Zone::Graveyard);
    let good = put(&mut game, player, "Braided Net", Zone::Graveyard);
    let large = put(&mut game, player, "Solemn Simulacrum", Zone::Graveyard);
    let wrong_type = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    game.state.players[player.index()].mana = TWO_GENERIC;
    reject(&mut game, activate(source));

    let copied = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    game.apply_temporary_copy(
        game.object_ref(copied),
        CopyRecipe::new(boots),
        CopyDeadline::EndOfTurn,
    );
    assert_eq!(game.object_mana_value(copied), 2);
    game.state.step = Step::Upkeep;
    reject(&mut game, activate(source));
    game.state.step = Step::Main1;
    act(&mut game, activate(source));
    assert!(game.x_choice().is_none());
    for invalid in [large, foe_target, wrong_type] {
        reject(&mut game, target(invalid));
    }
    act(&mut game, target(good));
    reject(&mut game, mana(source));
    act(&mut game, mana(copied));
    assert!(game.object_view(copied).tapped);

    let mut exile_first = game.clone();
    act(&mut exile_first, Action::BeginOrderedCosts);
    open_additional_cost(&mut exile_first);
    for card in [copied, cheap] {
        let action = material(&exile_first, card);
        act(&mut exile_first, action);
    }
    act(&mut exile_first, Action::FinishAdditionalCostGroup);
    assert!(!exile_first.object_view(source).tapped);
    assert_eq!(exile_first.object_view(copied).zone, Zone::Exile);
    choose(&mut exile_first, |a| {
        matches!(a, Action::ChooseCastCost { .. })
    });
    pay(&mut exile_first);
    assert!(exile_first.object_view(source).tapped);
    assert_eq!(exile_first.state.stack.last().unwrap().x, Some(3));
    settle(&mut exile_first);
    assert_eq!(exile_first.object_view(good).zone, Zone::Battlefield);

    pay(&mut game);
    assert!(game.object_view(source).tapped);
    open_additional_cost(&mut game);
    assert!(game.observe(player.other()).additional_cost.is_none());
    for invalid in [source, creature, foreign, good] {
        let action = material(&game, invalid);
        reject(&mut game, action);
    }
    let first = material(&game, cheap);
    act(&mut game, first.clone());
    reject(&mut game, first);
    reject(&mut game, Action::FinishAdditionalCostGroup);
    assert_eq!(game.object_view(cheap).zone, Zone::Battlefield);

    let saved = game.snapshot();
    let second = material(&game, copied);
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == second)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, second.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, second);
    assert_eq!(game.snapshot(), indexed.snapshot());
    act(&mut game, Action::FinishAdditionalCostGroup);
    act(&mut indexed, Action::FinishAdditionalCostGroup);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(cheap).zone, Zone::Exile);
    assert_eq!(game.object_view(copied).zone, Zone::Exile);
    assert_eq!(game.object_mana_value(copied), 1);
    let packet = game.state.stack.last().unwrap();
    assert_eq!(packet.x, Some(3));
    let paid = packet.paid_additional_cost.as_ref().unwrap();
    assert_eq!(
        (paid.selected, paid.count, paid.total_mana_value),
        (2, 2, Some(3))
    );

    let old_target = indexed.object_ref(good);
    indexed.move_object(good, Zone::Exile);
    indexed.move_object(good, Zone::Graveyard);
    assert_ne!(old_target, indexed.object_ref(good));
    settle(&mut indexed);
    assert_eq!(indexed.object_view(good).zone, Zone::Graveyard);
    assert_eq!(indexed.object_view(copied).zone, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(good).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn zero_mana_value_requires_a_material_that_survives_the_mana_window() {
    let _rules = fixture_rules::activate(&[(definition("Short Bow"), &FOUNDRY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let zero = put(&mut game, player, "Mox Jasper", Zone::Graveyard);
    reject(&mut game, activate(source));
    let token = game.create_token_batch_capture(player, TREASURE, 1, false, None, true)[0];
    game.state.players[player.index()].mana = TWO_GENERIC;
    reject(&mut game, activate(source));

    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 2];
    act(&mut game, activate(source));
    act(&mut game, target(zero));
    pay(&mut game);
    open_additional_cost(&mut game);
    reject(&mut game, Action::FinishAdditionalCostGroup);
    let action = material(&game, token.card);
    act(&mut game, action);
    act(&mut game, Action::FinishAdditionalCostGroup);
    let packet = game.state.stack.last().unwrap();
    assert_eq!(packet.x, Some(0));
    assert_eq!(packet.paid_additional_cost.as_ref().unwrap().selected, 1);
    assert!(!game.on_battlefield(token.card));
    settle(&mut game);
    assert_eq!(game.object_view(zero).zone, Zone::Battlefield);
}
