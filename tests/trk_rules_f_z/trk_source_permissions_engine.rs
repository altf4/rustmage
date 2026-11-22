//! Exact source permissions for Star Trek graveyard, outside and exiled-card plays.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};

const OVERFLOW_FRAGMENT: StaticAbilityFragment =
    StaticAbilityFragment::CastArtifactCreatureFromGraveyardOnceEachTurnWithFinality;
const HOLO_FRAGMENT: StaticAbilityFragment =
    StaticAbilityFragment::PlayPermanentFromGraveyardOrOutsideOnceEachTurn;
const OVERFLOW_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[OVERFLOW_FRAGMENT],
}];
const HOLO_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[HOLO_FRAGMENT],
}];
const OVERFLOW: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: OVERFLOW_GROUPS,
    extras: &CardExtras {
        static_ability_groups: OVERFLOW_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const HOLODECK: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: HOLO_GROUPS,
    extras: &CardExtras {
        static_ability_groups: HOLO_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const WEB_HART: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        web_slinging_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn graveyard_action(
    game: &Game,
    card: CardId,
    source: CardId,
    play_kind: SourcePlayKind,
) -> Action {
    Action::AnnounceSourceGraveyard {
        card,
        source: game.object_ref(source),
        play_kind,
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}

#[test]
fn overflow_allows_paid_alternative_and_retains_finality_after_returning_provider() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &OVERFLOW),
        (Card::from_name("Burnished Hart").unwrap(), &WEB_HART),
    ]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[source as usize].tapped = true;
    let card = put(&mut game, Player::P0, "Burnished Hart", Zone::Graveyard);
    let source_ref = game.object_ref(source);
    let action = Action::AnnounceWebSlinging {
        card,
        face: CastFace::Normal,
        cost: ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 0,
        },
        flash_surcharge: false,
        origin: PaidCastOrigin::SourcePlay {
            source: source_ref,
            kind: SourcePlayKind::GraveyardArtifactCreatureFinality,
            outside: false,
        },
    };
    assert!(game.legal_actions().contains(&action));
    act(&mut game, action);
    act(&mut game, Action::BeginOrderedCosts);
    let clause = game
        .observe(Player::P0)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| matches!(row.clause, CastCostClause::WebSlingingReturn(_)))
        .unwrap() as u16;
    act(&mut game, Action::ChooseCastCost { clause });
    act(
        &mut game,
        Action::PayWebSlingingReturn { object: source_ref },
    );
    let mana_clause = game
        .observe(Player::P0)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && row.clause == CastCostClause::Mana)
        .unwrap() as u16;
    act(
        &mut game,
        Action::ChooseCastCost {
            clause: mana_clause,
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(card, CounterKind::Finality), 1);
}

#[test]
fn overflow_uses_exact_source_once_and_the_cast_permanent_enters_with_finality() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OVERFLOW)]);
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        let source = put(&mut game, actor, "Grizzly Bears", Zone::Battlefield);
        let first = put(&mut game, actor, "Burnished Hart", Zone::Graveyard);
        let second = put(&mut game, actor, "Burnished Hart", Zone::Graveyard);
        let foreign = put(&mut game, actor.other(), "Burnished Hart", Zone::Graveyard);
        let nonartifact = put(&mut game, actor, "Hill Giant", Zone::Graveyard);
        let kind = SourcePlayKind::GraveyardArtifactCreatureFinality;
        let action = graveyard_action(&game, first, source, kind);
        assert!(game.legal_actions().contains(&action));
        let forbidden_foreign = graveyard_action(&game, foreign, source, kind);
        let forbidden_nonartifact = graveyard_action(&game, nonartifact, source, kind);
        reject(&mut game, forbidden_foreign);
        reject(&mut game, forbidden_nonartifact);
        let before = game.snapshot();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| a == &action)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(actor, game.revision(), index).unwrap();
        act(&mut game, action.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.observe(actor).source_play_uses.len(), 1);
        let spent = graveyard_action(&game, second, source, kind);
        reject(&mut game, spent);
        game.move_object(source, Zone::Graveyard);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(first).zone, Zone::Battlefield);
        assert_eq!(game.counter_count(first, CounterKind::Finality), 1);
        game.move_object(first, Zone::Graveyard);
        assert_eq!(game.object_view(first).zone, Zone::Exile);
        game.restore(&before).unwrap();
        assert!(game.observe(actor).source_play_uses.is_empty());
        assert!(
            game.legal_actions()
                .contains(&graveyard_action(&game, second, source, kind))
        );
    }
}

fn with_outside(actor: Player) -> Game {
    let mut pools = [Vec::new(), Vec::new()];
    pools[actor.index()] = vec![Card::from_name("Serra Angel").unwrap(), Card::Forest];
    let configured = Game::new_with_outside(
        Config {
            decks: [vec![Card::Forest; 7], vec![Card::Forest; 7]],
            seed: 20261023,
            starting_player: actor,
            deck_rules: DeckRules::Freeform,
        },
        OutsideGameConfig {
            mode: OutsideGameMode::CasualCollection,
            pools,
        },
    )
    .unwrap();
    let mut game = scenario(actor);
    game.state.outside_game = configured.state.outside_game;
    game
}

#[test]
fn holodeck_outside_selection_is_private_cancelable_and_shares_the_graveyard_use() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &HOLODECK)]);
    for actor in [Player::P0, Player::P1] {
        let mut game = with_outside(actor);
        let source = put(&mut game, actor, "Hill Giant", Zone::Battlefield);
        let graveyard = put(&mut game, actor, "Grizzly Bears", Zone::Graveyard);
        let select = Action::SelectOutsideSourcePlay {
            source: game.object_ref(source),
            selection: outside_choices::Selection::Pool(0),
        };
        assert!(game.legal_actions().contains(&select));
        let before = game.snapshot();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| a == &select)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(actor, game.revision(), index).unwrap();
        act(&mut game, select.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        let hidden = game.observe(actor.other());
        assert!(hidden.actions.is_empty());
        assert!(hidden.owned_outside.is_none());
        let angel = Card::from_name("Serra Angel").unwrap();
        assert!(hidden.objects.iter().all(|o| o.card != angel));
        act(&mut game, Action::CancelOutsideSourcePlay);
        assert_eq!(game.observe(actor).source_play_uses.len(), 0);
        assert!(game.legal_actions().contains(&select));
        game.restore(&before).unwrap();
        act(&mut game, select);
        act(
            &mut game,
            Action::AnnounceSelectedOutsidePermanent {
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        assert_eq!(game.observe(actor).source_play_uses.len(), 1);
        let spent = graveyard_action(
            &game,
            graveyard,
            source,
            SourcePlayKind::GraveyardOrOutsidePermanent,
        );
        reject(&mut game, spent);
        pay(&mut game);
        settle(&mut game);
        assert!(
            game.observe(actor)
                .objects
                .iter()
                .any(|o| o.card == angel && o.zone == Zone::Battlefield)
        );
        assert_eq!(
            game.observe(actor).owned_outside.unwrap()[actor.index()],
            Some(vec![Card::Forest])
        );
    }
}

#[test]
fn holodeck_plays_a_graveyard_land_once_and_restore_recovers_the_permission() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &HOLODECK)]);
    let mut game = scenario(Player::P1);
    let source = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let first = put(&mut game, Player::P1, "Forest", Zone::Graveyard);
    let second = put(&mut game, Player::P1, "Mountain", Zone::Graveyard);
    let action = Action::PlaySourceGraveyardLand {
        card: first,
        source: game.object_ref(source),
        play_kind: SourcePlayKind::GraveyardOrOutsidePermanent,
    };
    let before = game.snapshot();
    assert!(game.legal_actions().contains(&action));
    act(&mut game, action);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.observe(Player::P1).source_play_uses.len(), 1);
    let source_ref = game.object_ref(source);
    reject(
        &mut game,
        Action::PlaySourceGraveyardLand {
            card: second,
            source: source_ref,
            play_kind: SourcePlayKind::GraveyardOrOutsidePermanent,
        },
    );
    let used_turn = game.state.turn;
    game.state.step = Step::Cleanup;
    game.state.pending = Pending::Priority;
    game.state.priority = Player::P1;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.turn, used_turn + 1);
    assert!(game.observe(Player::P1).source_play_uses.is_empty());
    game.restore(&before).unwrap();
    assert!(game.observe(Player::P1).source_play_uses.is_empty());
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
}

#[test]
fn holodeck_outside_spell_can_use_paid_alternative_and_commit_the_pool_card() {
    let angel = Card::from_name("Serra Angel").unwrap();
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &HOLODECK), (angel, &WEB_HART)]);
    let mut game = with_outside(Player::P0);
    let source = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    game.state.objects[source as usize].tapped = true;
    let source_ref = game.object_ref(source);
    act(
        &mut game,
        Action::SelectOutsideSourcePlay {
            source: source_ref,
            selection: outside_choices::Selection::Pool(0),
        },
    );
    let action = game
        .legal_actions()
        .into_iter()
        .find(|action| {
            matches!(
                action,
                Action::AnnounceWebSlinging {
                    origin: PaidCastOrigin::SourcePlay {
                        source,
                        outside: true,
                        ..
                    },
                    ..
                } if *source == source_ref
            )
        })
        .expect("outside paid alternative");
    act(&mut game, action);
    act(&mut game, Action::BeginOrderedCosts);
    let return_clause = game
        .observe(Player::P0)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| matches!(row.clause, CastCostClause::WebSlingingReturn(_)))
        .unwrap() as u16;
    act(
        &mut game,
        Action::ChooseCastCost {
            clause: return_clause,
        },
    );
    act(
        &mut game,
        Action::PayWebSlingingReturn { object: source_ref },
    );
    let mana_clause = game
        .observe(Player::P0)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && row.clause == CastCostClause::Mana)
        .unwrap() as u16;
    act(
        &mut game,
        Action::ChooseCastCost {
            clause: mana_clause,
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert!(
        game.observe(Player::P0)
            .objects
            .iter()
            .any(|o| o.card == angel && o.zone == Zone::Battlefield)
    );
    assert_eq!(
        game.observe(Player::P0).owned_outside.unwrap()[0],
        Some(vec![Card::Forest])
    );
}

#[test]
fn copied_static_provider_switches_between_overflow_and_holodeck_permissions() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &OVERFLOW),
        (Card::HillGiant, &HOLODECK),
    ]);
    let mut game = with_outside(Player::P0);
    let source = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let card = put(&mut game, Player::P0, "Burnished Hart", Zone::Graveyard);
    let source_ref = game.object_ref(source);
    let overflow = graveyard_action(
        &game,
        card,
        source,
        SourcePlayKind::GraveyardArtifactCreatureFinality,
    );
    let outside = Action::SelectOutsideSourcePlay {
        source: source_ref,
        selection: outside_choices::Selection::Pool(0),
    };
    reject(&mut game, overflow.clone());
    reject(&mut game, outside.clone());
    game.set_copy_recipe(source_ref, CopyRecipe::new(Card::GrizzlyBears));
    assert!(game.legal_actions().contains(&overflow));
    assert!(!game.legal_actions().contains(&outside));
    let checkpoint = game.snapshot();
    game.set_copy_recipe(source_ref, CopyRecipe::new(Card::HillGiant));
    assert!(!game.legal_actions().contains(&overflow));
    assert!(game.legal_actions().contains(&outside));
    game.clear_copy_recipe(source_ref);
    reject(&mut game, overflow.clone());
    reject(&mut game, outside);
    game.restore(&checkpoint).unwrap();
    act(&mut game, overflow);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(card, CounterKind::Finality), 1);
}

const ANY_CARD: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const Q_ACTIVATION: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        additional: Some(AdditionalCost {
            kind: EffectChoiceKind::ExileFromHand,
            filter: ANY_CARD,
        }),
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::CopyExiledSourceOfferPaidCast],
    sorcery: false,
    once_per_turn: false,
};
const Q_PROGRAM: &[Instruction] = &[
    Instruction::ShuffleYourLibrary,
    Instruction::ExileUntilNonlandGrantCopyCast { ability: 0 },
];
const Q_TEMPLATE: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, Q_PROGRAM),
    activated: &[Q_ACTIVATION],
    extras: &CardExtras {
        self_granted_activation_templates: &[0],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn q_grant_exiles_a_hand_card_and_offers_an_exact_paid_copy_cast() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &Q_TEMPLATE)]);
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        game.state.objects.clear();
        for player in &mut game.state.players {
            player.library.clear();
        }
        put(&mut game, actor, "Forest", Zone::Library);
        put(&mut game, actor, "Grizzly Bears", Zone::Library);
        let spell = put(&mut game, actor, "Opt", Zone::Hand);
        cast(&mut game, spell, None);
        settle(&mut game);
        let grant = game.observe(actor).exiled_copy_grants[0];
        assert_eq!(game.object_view(grant.recipient.card).zone, Zone::Exile);
        assert_eq!(
            game.object_view(grant.recipient.card).card,
            Card::GrizzlyBears
        );
        let action = Action::AnnounceExiledCopyGrant {
            source: grant.recipient,
        };
        assert!(!game.observe(actor.other()).actions.contains(&action));
        assert!(!game.legal_actions().contains(&action));
        let payment = put(&mut game, actor, "Forest", Zone::Hand);
        assert!(game.legal_actions().contains(&action));
        act(&mut game, action.clone());
        let payment_ref = game.object_ref(payment);
        act(
            &mut game,
            Action::ChooseAdditionalCost {
                object: payment_ref,
            },
        );
        pay(&mut game);
        assert_eq!(game.object_view(payment).zone, Zone::Exile);
        game.move_object(grant.recipient.card, Zone::Hand);
        for _ in 0..2 {
            act(&mut game, Action::Pass);
        }
        assert!(
            game.legal_actions()
                .contains(&Action::FinishResolutionCasts)
        );
        let observed = game.observe(actor);
        let copies: Vec<_> = observed
            .objects
            .iter()
            .filter(|o| o.card == Card::GrizzlyBears && o.zone == Zone::Exile)
            .collect();
        assert_eq!(copies.len(), 1);
        act(&mut game, Action::FinishResolutionCasts);
        settle(&mut game);
        assert!(!game.legal_actions().contains(&action));
    }
}
