//! A live provider's alternative mana base for independently authorized spells.
mod support;

use rustmage::PaidCastOrigin;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use std::num::NonZeroU32;
use support::*;

const COLORS: Mana = [1, 1, 1, 1, 1, 0];
const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const BASE: ManaCost = ManaCost {
    colored: COLORS,
    generic: 0,
};
const PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SpellAlternativeManaCost(BASE)],
    }],
    ..CardRules::EMPTY
};
const SACRIFICIAL_PROVIDER: CardRules = CardRules {
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(WHITE))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..PROVIDER
};
const ZERO: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ExileCastForZeroOncePerTurn],
    }],
    ..CardRules::EMPTY
};
const TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OpponentGraveyardOrExileSpellTax(2)],
    }],
    ..CardRules::EMPTY
};
const PAID_OFFER: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::ExileTopMayCastPaid]),
    ..CardRules::EMPTY
};
const GRAVEYARD: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CastFromGraveyardWithLife {
            filter: CastFilter::Creature,
            life: 2,
            entry_counter: CounterKind::Finality,
        }],
    }],
    ..CardRules::EMPTY
};
const TOP: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::LookAtLibraryTop,
            StaticAbilityFragment::CastInstantSorceryFromLibraryTopOnceEachTurn,
        ],
    }],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn provider_card() -> Card {
    card("Anthem of Champions")
}

fn provider(game: &mut Game, player: Player) -> ObjectRef {
    let id = spawn(game, player, provider_card(), Zone::Battlefield);
    game.object_ref(id)
}

fn alternative(game: &Game, spell: CardId, provider: ObjectRef, origin: PaidCastOrigin) -> Action {
    Action::AnnounceCastWithManaAlternative {
        object: game.object_ref(spell),
        provider,
        face: CastFace::Normal,
        cost: BASE,
        flash_surcharge: false,
        origin,
    }
}

fn finish_costs(game: &mut Game) {
    for _ in 0..16 {
        match game.state.pending {
            Pending::Priority | Pending::ResolutionCasts { .. } => return,
            Pending::CastCosts { .. } => {
                choose(game, |action| {
                    matches!(action, Action::ChooseCastCost { .. })
                });
            }
            Pending::AdditionalCost { .. } => {
                choose(game, |action| {
                    matches!(action, Action::ChooseAdditionalCost { .. })
                });
            }
            Pending::PayCosts { .. } => pay(game),
            Pending::Targets { .. } => act(game, Action::FinishTargets),
            ref pending => panic!("unexpected cast choice {pending:?}"),
        }
    }
    panic!("cast did not finish");
}

#[test]
fn the_source_cost_has_five_colored_symbols_and_leaves_the_printed_x_route_available() {
    let _rules = fixture_rules::activate(&[(provider_card(), &PROVIDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = provider(&mut game, player);
    let spell = put(&mut game, player, "Wildwood Scourge", Zone::Hand);
    let action = alternative(&game, spell, source, PaidCastOrigin::Ordinary);
    game.state.players[player.index()].mana = [0, 1, 1, 1, 1, 1];
    reject(&mut game, action.clone());
    game.state.players[player.index()].mana = COLORS;

    let mut printed = game.clone();
    act(&mut printed, Action::AnnounceCast { card: spell });
    act(&mut printed, Action::ChooseX { value: 2 });
    finish_costs(&mut printed);
    assert_eq!(printed.state.stack.last().unwrap().x, Some(2));

    act(&mut game, action);
    reject(&mut game, Action::ChooseX { value: 1 });
    let domain = game.mana_payment_domain().unwrap();
    assert_eq!(domain.required, COLORS);
    assert_eq!(domain.generic, 0);
    finish_costs(&mut game);
    let item = game.state.stack.last().unwrap();
    assert_eq!(item.x, Some(0));
    assert_eq!(item.mana_value(), 1);
    assert_eq!(item.cast.unwrap().payment, COLORS);
}

#[test]
fn the_selected_adventure_face_keeps_its_own_timing_and_characteristics() {
    let _rules = fixture_rules::activate(&[(provider_card(), &PROVIDER)]);
    let player = Player::P1;
    let mut game = scenario(player.other());
    let source = provider(&mut game, player);
    let spell = put(&mut game, player, "Questing Druid", Zone::Hand);
    game.state.players[player.index()].mana = COLORS;
    act(&mut game, Action::Pass);
    let normal = alternative(&game, spell, source, PaidCastOrigin::Ordinary);
    reject(&mut game, normal);
    let action = Action::AnnounceCastWithManaAlternative {
        object: game.object_ref(spell),
        provider: source,
        face: CastFace::Adventure,
        cost: BASE,
        flash_surcharge: false,
        origin: PaidCastOrigin::Ordinary,
    };
    act(&mut game, action);
    finish_costs(&mut game);
    let item = game.state.stack.last().unwrap();
    let cast = item.cast.unwrap();
    assert_eq!(cast.face, CastFace::Adventure);
    assert_eq!(cast.physical_card, Some(card("Questing Druid")));
    assert_eq!(cast.payment, COLORS);
    assert_eq!(item.mana_value(), 2);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
}

#[test]
fn a_taxed_exile_cast_preserves_mandatory_costs_and_uses_one_mana_base() {
    let _rules = fixture_rules::activate(&[
        (provider_card(), &PROVIDER),
        (card("Deeproot Pilgrimage"), &ZERO),
        (card("Short Bow"), &TAX),
    ]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = provider(&mut game, player);
    let zero_id = put(&mut game, player, "Deeproot Pilgrimage", Zone::Battlefield);
    put(&mut game, opponent, "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, opponent, "Corrupted Conviction", Zone::Exile);
    let object = game.object_ref(spell);
    let surcharge = NonZeroU32::new(1).unwrap();
    game.grant_taxed_exile_play(player, object, surcharge);
    let origin = PaidCastOrigin::TaxedExile { surcharge };
    let action = alternative(&game, spell, source, origin);
    let payment = [1, 1, 1, 1, 1, 3];
    game.state.players[player.index()].mana = payment;
    reject(&mut game, action.clone());

    let sacrifice = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceExileForZero {
                object,
                provider: game.object_ref(zero_id),
                face: CastFace::Normal,
                flash_surcharge: false,
                origin: ExileZeroOrigin::Taxed { surcharge },
            })
    );
    act(&mut game, action);
    finish_costs(&mut game);
    assert_eq!(game.object_view(sacrifice).zone, Zone::Graveyard);
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.origin, Zone::Exile);
    assert_eq!(cast.exile_play_surcharge, 1);
    assert_eq!(cast.payment, payment);
    assert!(game.observe(player).exile_zero_uses.is_empty());
    settle(&mut game);
    assert_eq!(game.hand_count(player), 2);
}

#[test]
fn an_independent_exile_permission_controls_access_and_as_though_mana_spending() {
    let _rules = fixture_rules::activate(&[(provider_card(), &PROVIDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = provider(&mut game, player);
    let spell = put(&mut game, player.other(), "Quick Study", Zone::Exile);
    let object = game.object_ref(spell);
    let action = alternative(&game, spell, source, PaidCastOrigin::Ordinary);
    game.state.players[player.index()].mana = COLORS;
    reject(&mut game, action.clone());

    let mut life_only = game.clone();
    life_only
        .grant_exile_play_permission_with_expiry(player, object, None, ExilePlayExpiry::WhileExiled)
        .unwrap()
        .life_instead_of_mana = true;
    reject(&mut life_only, action.clone());
    let mut free_only = game.clone();
    free_only
        .grant_exile_play_permission_with_expiry(player, object, None, ExilePlayExpiry::WhileExiled)
        .unwrap()
        .without_paying_mana_cost = true;
    reject(&mut free_only, action.clone());

    game.grant_exile_play_permission_with_expiry(
        player,
        object,
        None,
        ExilePlayExpiry::WhileExiled,
    )
    .unwrap()
    .spend_any_mana = true;
    let payment = [0, 0, 0, 0, 0, 5];
    game.state.players[player.index()].mana = payment;
    act(&mut game, action);
    finish_costs(&mut game);
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert!(cast.spend_any_mana);
    assert_eq!(cast.payment, payment);
    assert_eq!(cast.caster, player);
    assert_eq!(game.object_view(spell).owner, player.other());
}

#[test]
fn a_sacrificed_mana_provider_funds_the_frozen_cost_across_owned_branches() {
    let _rules = fixture_rules::activate(&[(provider_card(), &SACRIFICIAL_PROVIDER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = provider(&mut game, player);
    let spell = put(&mut game, player, "Quick Study", Zone::Hand);
    let next = put(&mut game, player, "Quick Study", Zone::Hand);
    game.state.players[player.index()].mana = [0, 1, 1, 1, 1, 0];
    let action = alternative(&game, spell, source, PaidCastOrigin::Ordinary);
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let revision = game.revision();
    let saved = game.snapshot();
    let mut raw = game.clone();
    act(&mut raw, action.clone());
    game.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), raw.snapshot());

    for branch in [&mut raw, &mut game] {
        act(
            branch,
            Action::ActivateMana {
                card: source.card,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: WHITE,
            },
        );
        assert_eq!(branch.object_view(source.card).zone, Zone::Graveyard);
        finish_costs(branch);
        assert_eq!(
            branch.state.stack.last().unwrap().cast.unwrap().payment,
            COLORS
        );
        let stale = alternative(branch, next, source, PaidCastOrigin::Ordinary);
        reject(branch, stale);
    }
    assert_eq!(game.snapshot(), raw.snapshot());

    game.restore(&saved).unwrap();
    assert_eq!(
        game.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    assert!(game.legal_actions().contains(&action));
}

#[test]
fn a_paid_resolution_offer_accepts_the_alternative_and_keeps_optional_kicker() {
    let _rules =
        fixture_rules::activate(&[(provider_card(), &PROVIDER), (card("Opt"), &PAID_OFFER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = provider(&mut game, player);
    let spell = put(&mut game, player, "Burst Lightning", Zone::Library);
    let offer = put(&mut game, player, "Opt", Zone::Hand);
    game.state.players[player.index()].mana = [1, 2, 1, 1, 1, 4];
    cast(&mut game, offer, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.pending, Pending::ResolutionCasts { player });
    let ordinary = alternative(&game, spell, source, PaidCastOrigin::Ordinary);
    reject(&mut game, ordinary);
    let action = alternative(&game, spell, source, PaidCastOrigin::PaidResolution);
    act(&mut game, action);
    act(&mut game, Action::ChooseKicker { pay: true });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    finish_costs(&mut game);
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.kicker, Some(true));
    assert_eq!(cast.payment, [1, 1, 1, 1, 1, 4]);
    if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 16);
}

#[test]
fn library_and_graveyard_permissions_keep_their_use_costs_and_entry_consequences() {
    let _rules = fixture_rules::activate(&[
        (provider_card(), &PROVIDER),
        (card("Short Bow"), &TOP),
        (card("Deeproot Pilgrimage"), &GRAVEYARD),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = provider(&mut game, player);
    let spell = put(&mut game, player, "Quick Study", Zone::Library);
    let creature = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let graveyard_origin = PaidCastOrigin::GraveyardLife {
        life: 2,
        entry_counter: Some(CounterKind::Finality),
    };
    let from_graveyard = alternative(&game, creature, source, graveyard_origin);
    reject(&mut game, from_graveyard.clone());
    assert!(!game.legal_actions().iter().any(|action| {
        matches!(action, Action::AnnounceCastWithManaAlternative { object, .. } if object.card == spell)
    }));
    put(&mut game, player, "Deeproot Pilgrimage", Zone::Battlefield);
    let top_provider = put(&mut game, player, "Short Bow", Zone::Battlefield);
    assert!(
        game.observe(player.other())
            .objects
            .iter()
            .all(|object| object.id != spell)
    );

    let top_permission = game
        .legal_actions()
        .iter()
        .find_map(|action| match action {
            Action::AnnounceCastWithManaAlternative {
                object,
                origin: PaidCastOrigin::LibraryTop { permission },
                ..
            } if object.card == spell => Some(*permission),
            _ => None,
        })
        .unwrap();
    assert_eq!(top_permission.source, game.object_ref(top_provider));
    let from_library = alternative(
        &game,
        spell,
        source,
        PaidCastOrigin::LibraryTop {
            permission: top_permission,
        },
    );
    act(&mut game, from_library);
    finish_costs(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().origin,
        Zone::Library
    );
    settle(&mut game);
    let next = put(&mut game, player, "Quick Study", Zone::Library);
    let reuse = alternative(
        &game,
        next,
        source,
        PaidCastOrigin::LibraryTop {
            permission: top_permission,
        },
    );
    reject(&mut game, reuse);

    act(&mut game, from_graveyard);
    finish_costs(&mut game);
    assert_eq!(game.state.players[player.index()].life, 18);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(creature, CounterKind::Finality), 1);
}
