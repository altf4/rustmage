//! Exact paid copy offers and private mana-value-limited casts during resolution.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const COPY_OFFER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Colors(1 << 2),
        })),
        &[
            Instruction::ExileTargetCopyCastPaid,
            Instruction::IfOfferedCastDeclined { skip: 1 },
            Instruction::LoseLife(PlayerSet::Controller, 2),
        ],
    ),
    ..CardRules::EMPTY
};

const HAND_OFFER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayFromOutsideOwnHand(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CastHandPermanentFreeUpTo(AmountValue::CastSpellManaValue),
            Instruction::IfOfferedCastCommitted { skip: 1 },
            Instruction::PutHandLand { tapped: false },
        ],
    }],
    ..CardRules::EMPTY
};

fn reach_offer(game: &mut Game) {
    for _ in 0..12 {
        if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("cast scenario did not reach its resolution offer");
}

#[test]
fn paid_copy_is_a_distinct_cast_and_life_is_lost_only_on_commit() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &COPY_OFFER)]);
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let original = put(&mut game, other, "Exsanguinate", Zone::Graveyard);
    let bystander = put(&mut game, player, "Exsanguinate", Zone::Graveyard);
    let provider = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, provider, Some(Target::Card(original)));
    reach_offer(&mut game);

    assert_eq!(game.object_view(original).zone, Zone::Exile);
    assert_eq!(game.object_view(bystander).zone, Zone::Graveyard);
    let copy = game.state.resolution_free_casts.as_ref().unwrap().remaining[0];
    assert_ne!(copy.card, original);
    assert_eq!(game.object_view(copy.card).owner, player);
    let original_ref = game.object_ref(original);
    reject(
        &mut game,
        Action::CastExiledDuringResolution {
            object: original_ref,
            face: CastFace::Normal,
        },
    );
    let mut declined = game.clone();
    act(&mut declined, Action::FinishResolutionCasts);
    settle(&mut declined);
    assert_eq!(declined.state.players[player.index()].life, 20);
    assert_eq!(declined.object_view(original).zone, Zone::Exile);
    assert_eq!(declined.object_view(copy.card).zone, Zone::Outside);

    game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 3];
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object: copy,
            face: CastFace::Normal,
        },
    );
    act(&mut game, Action::ChooseX { value: 3 });
    pay(&mut game);
    let child = game.state.stack.last().unwrap();
    assert_eq!(child.mana_value(), 5);
    assert_eq!(child.cast.unwrap().origin, Zone::Exile);
    assert_eq!(child.cast.unwrap().method, CastMethod::Normal);
    assert_eq!(game.state.players[player.index()].life, 20);
    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(game.state.players[player.index()].life, 18);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.state.players[other.index()].life, 17);
    assert_eq!(game.object_view(original).zone, Zone::Exile);
}

#[test]
fn a_paid_copy_of_a_permanent_spell_resolves_as_a_token() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &COPY_OFFER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let original = put(
        &mut game,
        player.other(),
        "Marauding Blight-Priest",
        Zone::Graveyard,
    );
    let provider = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, provider, Some(Target::Card(original)));
    reach_offer(&mut game);
    let copy = game.state.resolution_free_casts.as_ref().unwrap().remaining[0];
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object: copy,
            face: CastFace::Normal,
        },
    );
    pay(&mut game);
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
    let token = game.object_view(copy.card);
    assert!(token.is_token());
    assert_eq!(token.zone, Zone::Battlefield);
    assert_eq!(token.card, game.object_view(original).card);
    assert_eq!(token.controller, player);
    assert_eq!(game.object_view(original).zone, Zone::Exile);
}

#[test]
fn private_hand_offer_uses_actual_x_and_falls_back_to_putting_a_land() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &HAND_OFFER)]);
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, player, "Exsanguinate", Zone::Exile);
    let permanent = put(&mut game, player, "Hill Giant", Zone::Hand);
    let expensive = put(&mut game, player, "Shivan Dragon", Zone::Hand);
    let instant = put(&mut game, player, "Shock", Zone::Hand);
    let land = put(&mut game, player, "Mountain", Zone::Hand);
    let rival_permanent = put(&mut game, other, "Hill Giant", Zone::Hand);
    game.state.land_played = true;
    game.grant_exile_play_permission(player, game.object_ref(spell), false);
    act(&mut game, Action::AnnounceExile { card: spell });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    reach_offer(&mut game);

    let opponent = game.observe(other);
    assert!(opponent.resolution_free_casts.is_none());
    assert!(!opponent.objects.iter().any(|object| object.id == permanent));
    let free = |game: &Game, card| Action::CastFreeDuringResolution {
        object: game.object_ref(card),
        face: CastFace::Normal,
    };
    for forbidden in [expensive, instant, land, rival_permanent] {
        let action = free(&game, forbidden);
        reject(&mut game, action);
    }
    let mut declined = game.clone();
    act(&mut declined, Action::FinishResolutionCasts);
    let object = declined.object_ref(land);
    act(&mut declined, Action::PutHandLand { object });
    settle(&mut declined);
    assert!(declined.state.land_played);
    assert_eq!(declined.object_view(land).zone, Zone::Battlefield);
    assert!(!declined.object_view(land).tapped);

    let action = free(&game, permanent);
    let snapshot = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, action);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    pay(&mut game);
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
    assert_eq!(game.object_view(permanent).zone, Zone::Battlefield);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    indexed.restore(&snapshot).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
}
