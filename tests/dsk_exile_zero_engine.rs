//! An exact provider's optional zero mana alternative for a separately legal exile cast.
mod support;
use rustmage::RoomDoor;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use std::num::NonZeroU32;
use support::*;

const ROOM: Card = Card::from_catalogue_id(176); // Anthem of Champions
const LEFT: Card = Card::from_catalogue_id(1024); // Deeproot Pilgrimage
const RIGHT: Card = Card::from_catalogue_id(3151); // Phyrexian Arena
const WARPED_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ExileCastForZeroOncePerTurn],
    }],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [LEFT, RIGHT],
            triggers: &[],
            static_groups: &[(1, RoomDoor::Right)],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ROOM_HALF: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room_parent: Some(ROOM),
        ..CardExtras::EMPTY
    },
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
const LINKED_PAID: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LinkedExilePaidCast {
            filter: CastFilter::Any,
            entry_counter: Some(CounterKind::PlusOne),
        }],
    }],
    ..CardRules::EMPTY
};
const LINKED_COUNTERS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LinkedExileCreatureCounterCasting(2)],
    }],
    ..CardRules::EMPTY
};
const PAID_OFFER: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::ExileTopMayCastPaid]),
    ..CardRules::EMPTY
};

fn rules() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (ROOM, &WARPED_RULES),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
        (Card::from_name("Short Bow").unwrap(), &TAX),
        (Card::from_name("Bear Cub").unwrap(), &LINKED_PAID),
        (Card::from_name("Hill Giant").unwrap(), &LINKED_COUNTERS),
        (Card::from_name("Opt").unwrap(), &PAID_OFFER),
    ])
}

fn provider(game: &mut Game, player: Player, door: RoomDoor) -> ObjectRef {
    let id = spawn(game, player, ROOM, Zone::Battlefield);
    let object = game.object_ref(id);
    game.open_room_on_entry(object, door);
    object
}

fn play_permission(game: &mut Game, card: CardId, player: Player) -> &mut ExilePlayPermission {
    game.grant_exile_play_permission_with_expiry(
        player,
        game.object_ref(card),
        None,
        ExilePlayExpiry::WhileExiled,
    )
    .unwrap()
}

fn zero(game: &Game, card: CardId, provider: ObjectRef, origin: ExileZeroOrigin) -> Action {
    Action::AnnounceExileForZero {
        object: game.object_ref(card),
        provider,
        face: CastFace::Normal,
        flash_surcharge: false,
        origin,
    }
}

fn finish_costs(game: &mut Game) {
    for _ in 0..16 {
        match game.state.pending {
            Pending::Priority | Pending::ResolutionCasts { .. } => return,
            Pending::CastCosts { .. } => choose(game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::AdditionalCost { .. } => choose(game, |action| {
                matches!(
                    action,
                    Action::ChooseAdditionalCost { .. } | Action::ChooseCreatureCounterCost { .. }
                )
            }),
            Pending::PayCosts { .. } => pay(game),
            Pending::Targets { .. } => act(game, Action::FinishTargets),
            ref pending => panic!("unexpected cast choice {pending:?}"),
        }
    }
    panic!("cast did not finish");
}

fn reach_next_main(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..48 {
        if game.state.turn > turn && game.state.step == Step::Main1 {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("expected the next main phase");
}

#[test]
fn a_normal_cast_leaves_the_offer_and_each_source_spends_only_its_completed_use() {
    let _rules = rules();
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = provider(&mut game, player, RoomDoor::Left);
    let second = provider(&mut game, player, RoomDoor::Right);
    let paid = put(&mut game, player, "Quick Study", Zone::Exile);
    let x_spell = put(&mut game, player, "Wildwood Scourge", Zone::Exile);
    let extra = put(&mut game, player, "Quick Study", Zone::Exile);
    let later = put(&mut game, opponent, "Quick Study", Zone::Exile);
    for card in [paid, x_spell, extra] {
        play_permission(&mut game, card, player);
    }
    play_permission(&mut game, later, opponent);
    let ordinary = ExileZeroOrigin::Ordinary;
    let first_offer = zero(&game, x_spell, first, ordinary);
    let second_offer = zero(&game, x_spell, second, ordinary);
    reject(&mut game, first_offer.clone());
    assert!(game.legal_actions().contains(&second_offer));
    game.open_room_on_entry(first, RoomDoor::Right);

    act(&mut game, Action::AnnounceExile { card: paid });
    finish_costs(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::Normal
    );
    settle(&mut game);
    assert!(game.observe(player).exile_zero_uses.is_empty());

    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, first_offer);
    assert!(game.observe(player).exile_zero_uses.is_empty());
    reject(&mut game, Action::ChooseX { value: 1 });
    finish_costs(&mut game);
    let item = game.state.stack.last().unwrap();
    assert_eq!(item.x, Some(0));
    assert_eq!(item.cast.unwrap().method, CastMethod::Free);
    assert_eq!(item.cast.unwrap().origin, Zone::Exile);
    settle(&mut game);

    let mut returned = game.clone();
    returned.move_object(first.card, Zone::Hand);
    returned.move_object(first.card, Zone::Battlefield);
    let new_source = returned.object_ref(first.card);
    returned.open_room_on_entry(new_source, RoomDoor::Right);
    reject(&mut returned, zero(&game, extra, first, ordinary));
    assert!(
        returned
            .legal_actions()
            .contains(&zero(&returned, extra, new_source, ordinary))
    );

    let mut opponent_turn = game.clone();
    opponent_turn.change_controller(second, opponent);
    act(&mut opponent_turn, Action::Pass);
    let off_turn_cast = zero(&opponent_turn, later, second, ordinary);
    act(&mut opponent_turn, off_turn_cast);
    finish_costs(&mut opponent_turn);
    assert_eq!(
        opponent_turn.observe(opponent).exile_zero_uses[1].source,
        second
    );

    let already_used = zero(&game, extra, first, ordinary);
    reject(&mut game, already_used);
    let use_second = zero(&game, extra, second, ordinary);
    act(&mut game, use_second);
    finish_costs(&mut game);
    settle(&mut game);
    let used = game.observe(player).exile_zero_uses;
    assert_eq!(
        used.iter().map(|row| row.source).collect::<Vec<_>>(),
        [first, second]
    );
    assert_eq!(used, game.observe(opponent).exile_zero_uses);

    game.change_controller(first, opponent);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(opponent));
    let off_turn = zero(&game, later, first, ordinary);
    reject(&mut game, off_turn.clone());
    reach_next_main(&mut game);
    assert!(game.observe(opponent).exile_zero_uses.is_empty());
    assert!(game.legal_actions().contains(&off_turn));
    act(&mut game, off_turn);
    finish_costs(&mut game);
    assert_eq!(game.observe(opponent).exile_zero_uses[0].player, opponent);
}

#[test]
fn an_exile_alternative_requires_its_own_ordinary_permission_and_never_grants_land_plays() {
    let _rules = rules();
    let player = Player::P1;
    let mut game = scenario(player);
    let source = provider(&mut game, player, RoomDoor::Right);
    let card = put(&mut game, player.other(), "Quick Study", Zone::Exile);
    let land = put(&mut game, player.other(), "Forest", Zone::Exile);
    let action = zero(&game, card, source, ExileZeroOrigin::Ordinary);
    reject(&mut game, action.clone());
    reject(&mut game, Action::PlayLand { card: land });
    play_permission(&mut game, card, player).life_instead_of_mana = true;
    let life = Action::AnnounceExileLife {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    assert!(game.legal_actions().contains(&life));
    reject(&mut game, action.clone());
    play_permission(&mut game, card, player);
    play_permission(&mut game, land, player);
    assert!(game.legal_actions().contains(&life));
    assert!(game.legal_actions().contains(&action));
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceExile { card })
    );
    let cast_land = zero(&game, land, source, ExileZeroOrigin::Ordinary);
    reject(&mut game, cast_land);
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).owner, player.other());
    assert_eq!(game.object_view(land).controller, player);
    assert!(game.observe(player).exile_zero_uses.is_empty());
}

#[test]
fn exact_surcharges_taxes_and_mandatory_costs_survive_transactional_casting() {
    let _rules = rules();
    let player = Player::P0;
    let opponent = player.other();
    let mut parent = scenario(player);
    let source = provider(&mut parent, player, RoomDoor::Right);
    let payment = put(&mut parent, player, "Bear Cub", Zone::Battlefield);
    put(&mut parent, opponent, "Short Bow", Zone::Battlefield);
    let spell = put(&mut parent, opponent, "Corrupted Conviction", Zone::Exile);
    let surcharge = NonZeroU32::new(1).unwrap();
    let object = parent.object_ref(spell);
    parent.grant_taxed_exile_play(player, object, surcharge);
    let unrelated = put(&mut parent, opponent, "Savannah Lions", Zone::Exile);
    parent.state.exile_links.push(ExileLink {
        ability_origin: None,
        source,
        exiled: parent.object_ref(unrelated),
        return_zone: Zone::Battlefield,
    });
    let action = zero(&parent, spell, source, ExileZeroOrigin::Taxed { surcharge });
    parent.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut parent, action.clone());
    let total = [0, 0, 0, 0, 0, 3];
    parent.state.players[player.index()].mana = total;
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let revision = parent.revision();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);

    act(&mut raw, action.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let mut cancelled = raw.clone();
    assert!(cancelled.state.cast_rollback.is_some());
    cancelled.finish_cast_rollback(false);
    assert_eq!(cancelled.snapshot(), saved);
    assert!(cancelled.legal_actions().contains(&action));

    for game in [&mut raw, &mut indexed] {
        assert!(game.observe(player).exile_zero_uses.is_empty());
        finish_costs(game);
        let cast = game.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.exile_play_surcharge, 1);
        assert_eq!(cast.payment, total);
        assert_eq!(game.object_view(payment).zone, Zone::Graveyard);
        assert_eq!(game.observe(player).exile_zero_uses[0].source, source);
        settle(game);
        assert_eq!(game.hand_count(player), 2);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    assert!(indexed.legal_actions().contains(&action));
    assert!(indexed.observe(player).exile_zero_uses.is_empty());
}

#[test]
fn linked_exile_permissions_keep_their_entry_counter_or_counter_cost() {
    let _rules = rules();
    let player = Player::P0;
    for with_counter_cost in [false, true] {
        let mut game = scenario(player);
        let source = provider(&mut game, player, RoomDoor::Right);
        let grant = put(
            &mut game,
            player,
            if with_counter_cost {
                "Hill Giant"
            } else {
                "Bear Cub"
            },
            Zone::Battlefield,
        );
        let grant = game.object_ref(grant);
        let spell = put(&mut game, player, "Savannah Lions", Zone::Exile);
        let object = game.object_ref(spell);
        game.link_exile_to_ability(grant, None, object);
        game.state.players[player.index()].mana = [0; 6];
        let origin = if with_counter_cost {
            ExileZeroOrigin::LinkedCounters { source: grant }
        } else {
            ExileZeroOrigin::LinkedPaid { source: grant }
        };
        let action = zero(&game, spell, source, origin);
        if with_counter_cost {
            reject(&mut game, action.clone());
            game.add_counter(grant.card, CounterKind::PlusOne, 2);
        }
        act(&mut game, action);
        finish_costs(&mut game);
        assert_eq!(
            game.state.stack.last().unwrap().cast.unwrap().payment,
            [0; 6]
        );
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
        assert_eq!(game.counter_count(grant.card, CounterKind::PlusOne), 0);
        assert_eq!(
            game.counter_count(spell, CounterKind::PlusOne),
            u64::from(!with_counter_cost)
        );
    }
}

#[test]
fn a_paid_resolution_offer_can_use_zero_and_still_pay_kicker() {
    let _rules = rules();
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = provider(&mut game, player, RoomDoor::Right);
    let spell = put(&mut game, player, "Burst Lightning", Zone::Library);
    let offer = put(&mut game, player, "Opt", Zone::Hand);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 4];
    cast(&mut game, offer, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.pending, Pending::ResolutionCasts { player });
    let ordinary = zero(&game, spell, source, ExileZeroOrigin::Ordinary);
    reject(&mut game, ordinary);
    let use_offer = zero(&game, spell, source, ExileZeroOrigin::PaidResolution);
    act(&mut game, use_offer);
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
    assert_eq!(cast.payment, [0, 0, 0, 0, 0, 4]);
    assert_eq!(game.observe(player).exile_zero_uses[0].source, source);
    if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 16);
}
