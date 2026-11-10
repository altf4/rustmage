//! Actual sacrifice receipts and private, face-qualified offers during resolution.
mod support;

use rustmage as _;
use rustmage_core::PermanentType;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ANOTHER_CREATURE: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const RECEIPT: AmountValue = AmountValue::ResolutionSacrificedManaValue;
const ATTACKER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MaySacrificeForManaValue {
                filter: ANOTHER_CREATURE,
                skip: 2,
            },
            Instruction::DrawScaled(PlayerSet::Controller, RECEIPT),
            Instruction::PutHandPermanent {
                maximum: RECEIPT,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const OFFER: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CastHandFreeUpToOfType {
            maximum: AmountValue::SourceCounterQuantity(CounterKind::Ingenuity),
            permanent_type: PermanentType::Artifact,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const ADDITIONAL: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: SacrificeFilter {
                subtype: Some("Elf"),
                ..ANOTHER_CREATURE
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack_and_resolve(game: &mut Game, attacker: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
    assert!(game.state.stack.last().unwrap().targets.is_empty());
    resolve_top(game);
}

fn activate_offer(game: &mut Game, source: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if game.state.announcing.is_some() {
        pay(game);
    }
    assert!(game.object_view(source).tapped);
    assert!(matches!(
        game.state.stack.last().unwrap().kind,
        StackKind::ActivatedAbility { ability: 0 }
    ));
}

fn free(object: ObjectRef, face: CastFace) -> Action {
    Action::CastFreeDuringResolution { object, face }
}

fn finish_offer(game: &mut Game) {
    if game
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(game, Action::FinishResolutionCasts);
    }
}

#[test]
fn attack_sacrifice_draws_frozen_battlefield_value_then_offers_one_permanent() {
    let _rules = fixture_rules::activate(&[(card("Hill Giant"), &ATTACKER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let front = card("Bruce Banner // The Incredible Hulk");
        let donor = spawn(&mut game, player.other(), front, Zone::Battlefield);
        let donor_ref = game.object_ref(donor);
        game.transform_permanent(donor_ref);
        assert_eq!(game.object_view(donor).card, front.back_face().unwrap());
        assert_eq!(game.object_mana_value(donor), 6);
        game.change_controller(donor_ref, player);
        put(
            &mut game,
            player.other(),
            "Rest in Peace",
            Zone::Battlefield,
        );
        let foe = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let hidden_foe = put(&mut game, player.other(), "Forest", Zone::Hand);
        let too_expensive = put(&mut game, player, "The Endstone", Zone::Hand);
        let nonpermanent = put(&mut game, player, "Opt", Zone::Hand);
        let newly_drawn = put(&mut game, player, "Shivan Dragon", Zone::Library);
        let library_size = game.state.players[player.index()].library.len();

        attack_and_resolve(&mut game, attacker);
        for invalid in [attacker, foe] {
            let object = game.object_ref(invalid);
            reject(&mut game, Action::Sacrifice { object });
        }
        act(&mut game, Action::Sacrifice { object: donor_ref });
        assert_eq!(game.object_view(donor).zone, Zone::Exile);
        assert_eq!(game.object_view(donor).card, front);
        assert_eq!(
            game.state.players[player.index()].library.len(),
            library_size - 6
        );
        assert_eq!(game.object_view(newly_drawn).zone, Zone::Hand);
        assert!(game.observe(player.other()).effect_choice.is_none());
        let hand_choice = game.observe(player).effect_choice.unwrap();
        assert_eq!(
            hand_choice.kind,
            EffectChoiceKind::HandPermanent {
                maximum: 6,
                tapped: false,
            }
        );
        for invalid in [hidden_foe, too_expensive, nonpermanent] {
            let object = game.object_ref(invalid);
            reject(&mut game, Action::PutHandPermanent { object });
        }

        let chosen = game.object_ref(newly_drawn);
        let action = Action::PutHandPermanent { object: chosen };
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let saved = game.snapshot();
        let mut indexed = game.clone();
        act(&mut game, action.clone());
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.object_view(newly_drawn).zone, Zone::Battlefield);
        assert_ne!(game.object_ref(newly_drawn), chosen);
        reject(&mut game, action);

        indexed.restore(&saved).unwrap();
        act(&mut indexed, Action::DeclineHandEntry);
        assert_eq!(indexed.object_view(newly_drawn).zone, Zone::Hand);
        assert_eq!(indexed.object_view(donor).zone, Zone::Exile);
    }
}

struct ClearPrevention;

impl Drop for ClearPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|object| object.set(None));
    }
}

#[test]
fn decline_skips_the_program_but_a_replaced_payment_keeps_its_value() {
    const PROHIBITED: CardRules = CardRules {
        effect: Effect::Creature(2, 1),
        keywords: Keywords::CANT_BE_SACRIFICED,
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (card("Hill Giant"), &ATTACKER),
        (card("Savannah Lions"), &PROHIBITED),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let nonzero = put(&mut game, player, "Bear Cub", Zone::Hand);
    let replaceable = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let prohibited = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    let donor = game.create_token_batch_capture(player, Card::DrakeToken, 1, false, None, true)[0];
    let sacrifice = Action::Sacrifice { object: donor };
    let initial_draws = game.state.players[player.index()].cards_drawn_this_turn;
    game.state.land_played = true;

    attack_and_resolve(&mut game, attacker);
    let object = game.object_ref(prohibited);
    reject(&mut game, Action::Sacrifice { object });
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert!(declined.state.resolving.is_none());
    assert_eq!(declined.object_ref(donor.card), donor);
    {
        let _clear = ClearPrevention;
        let object = game.object_ref(replaceable);
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|target| target.set(Some(object)));
        let mut prevented = game.clone();
        act(&mut prevented, Action::Sacrifice { object });
        assert_eq!(prevented.object_ref(replaceable), object);
        assert_eq!(prevented.object_view(replaceable).zone, Zone::Battlefield);
        assert_eq!(
            prevented.state.players[player.index()].permanents_sacrificed_this_turn,
            0
        );
        assert_eq!(
            prevented.state.players[player.index()].cards_drawn_this_turn,
            initial_draws + 2
        );
        assert_eq!(
            prevented.observe(player).effect_choice.unwrap().kind,
            EffectChoiceKind::HandPermanent {
                maximum: 2,
                tapped: false,
            }
        );
        act(&mut prevented, Action::DeclineHandEntry);
    }
    act(&mut game, sacrifice);
    assert_eq!(
        game.state.players[player.index()].cards_drawn_this_turn,
        initial_draws
    );
    let invalid = game.object_ref(nonzero);
    reject(&mut game, Action::PutHandPermanent { object: invalid });
    let valid = game.object_ref(land);
    act(&mut game, Action::PutHandPermanent { object: valid });
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.state.land_played);
}

#[test]
fn private_artifact_offer_checks_the_modal_spell_face_against_departure_counter_lki() {
    let _rules = fixture_rules::activate(&[(card("Bear Cub"), &OFFER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let tony = put(
        &mut game,
        player,
        "Tony Stark // The Invincible Iron Man",
        Zone::Hand,
    );
    let fallback = put(&mut game, player, "Swiftfoot Boots", Zone::Hand);
    let oversized = put(&mut game, player, "The Endstone", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let foreign = put(&mut game, player.other(), "Swiftfoot Boots", Zone::Hand);
    let tony_ref = game.object_ref(tony);
    let back = free(tony_ref, CastFace::Back);

    game.add_counter(source, CounterKind::Ingenuity, 5);
    activate_offer(&mut game, source);
    let mut below = game.clone();
    resolve_top(&mut below);
    reject(&mut below, back.clone());
    assert!(
        below
            .legal_actions()
            .contains(&free(below.object_ref(fallback), CastFace::Normal))
    );

    game.add_counter(source, CounterKind::Ingenuity, 1);
    let original_source = game.object_ref(source);
    game.move_object(source, Zone::Hand);
    game.move_object(source, Zone::Battlefield);
    assert_ne!(game.object_ref(source), original_source);
    assert_eq!(game.counter_count(source, CounterKind::Ingenuity), 0);

    let mut hidden_hand = game.clone();
    hidden_hand.state.objects[fallback as usize].card = card("Grizzly Bears");
    for branch in [&mut game, &mut hidden_hand] {
        resolve_top(branch);
        assert!(
            branch
                .observe(player.other())
                .resolution_free_casts
                .is_none()
        );
    }
    assert_eq!(
        serde_json::to_value(game.observe(player.other())).unwrap(),
        serde_json::to_value(hidden_hand.observe(player.other())).unwrap()
    );
    let offer = game.observe(player).resolution_free_casts.unwrap();
    assert_eq!(offer.remaining_mana_value, Some(6));
    assert_eq!(offer.permanent_type, Some(PermanentType::Artifact));
    reject(&mut game, free(tony_ref, CastFace::Normal));
    for invalid in [oversized, land, foreign] {
        let object = game.object_ref(invalid);
        reject(&mut game, free(object, CastFace::Normal));
    }
    reject(&mut game, Action::PlayLand { card: land });
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == back)
        .unwrap();
    let mut indexed = game.clone();
    let saved = game.snapshot();
    act(&mut game, back.clone());
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    pay(&mut game);
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.method, CastMethod::Free);
    assert_eq!(cast.face, CastFace::Back);
    assert_eq!(cast.payment, [0; 6]);
    finish_offer(&mut game);
    settle(&mut game);
    assert_eq!(
        game.object_view(tony).card.name(),
        "The Invincible Iron Man"
    );
    assert_eq!(game.object_view(fallback).zone, Zone::Hand);

    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::FinishResolutionCasts);
    assert_eq!(indexed.object_view(tony).zone, Zone::Hand);
    reject(&mut indexed, back);
}

#[test]
fn free_artifact_cast_locks_x_to_zero_and_requires_printed_additional_cost() {
    let _rules = fixture_rules::activate(&[
        (card("Bear Cub"), &OFFER),
        (card("Marketback Walker"), &ADDITIONAL),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, player, "Marketback Walker", Zone::Hand);
    let donor = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    game.state.players[player.index()].mana = [0; 6];
    activate_offer(&mut game, source);
    let action = free(game.object_ref(artifact), CastFace::Normal);

    let mut unfunded = game.clone();
    unfunded.move_object(donor, Zone::Graveyard);
    resolve_top(&mut unfunded);
    reject(&mut unfunded, action.clone());
    assert!(
        unfunded
            .legal_actions()
            .contains(&Action::FinishResolutionCasts)
    );

    resolve_top(&mut game);
    act(&mut game, action);
    let pay_donor = Action::ChooseAdditionalCost {
        object: game.object_ref(donor),
    };
    act(&mut game, pay_donor);
    pay(&mut game);
    let spell = game.state.stack.last().unwrap();
    assert_eq!(spell.mana_value(), 0);
    assert_eq!(spell.cast.unwrap().method, CastMethod::Free);
    assert_eq!(game.object_view(donor).zone, Zone::Graveyard);
    finish_offer(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
}
