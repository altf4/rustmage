//! Live own-turn permission and printed costs for exactly linked exile objects.
mod support;
use rustmage::RoomDoor;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::PlayLinkedExileOnOwnTurnForLife],
    }],
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::ExileUncontrolledOpponentGraveyardCardsLinked),
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

const ROOM: Card = Card::from_catalogue_id(176); // Anthem of Champions
const LEFT: Card = Card::from_catalogue_id(1024); // Deeproot Pilgrimage, {1}{U}
const RIGHT: Card = Card::from_catalogue_id(3151); // Phyrexian Arena, {1}{B}{B}
const ROOM_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [LEFT, RIGHT],
            triggers: &[],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const HALF_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room_parent: Some(ROOM),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn life_cast(card: CardId, face: CastFace) -> Action {
    Action::AnnounceExileLife {
        card,
        face,
        flash_surcharge: false,
    }
}

fn exile_for_source(game: &mut Game, owner: Player, card: Card) -> CardId {
    let id = spawn(game, owner, card, Zone::Library);
    assert_eq!(
        game.move_object(id, Zone::Graveyard).destination,
        Zone::Exile
    );
    id
}

fn exile_named(game: &mut Game, owner: Player, name: &str) -> CardId {
    exile_for_source(game, owner, Card::from_name(name).unwrap())
}

fn choose_clause(game: &mut Game, clause: CastCostClause) {
    let position = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == clause && !row.paid)
        .unwrap();
    act(
        game,
        Action::ChooseCastCost {
            clause: position as u16,
        },
    );
}

fn finish_costs(game: &mut Game) {
    for _ in 0..12 {
        match game.state.pending {
            Pending::Priority => return,
            Pending::CastCosts { .. } => choose(game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::AdditionalCost { .. } => choose(game, |action| {
                matches!(action, Action::ChooseAdditionalCost { .. })
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
fn exact_links_follow_current_control_and_land_plays_use_the_ordinary_allowance() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let provider = game.object_ref(source);
    let land = exile_named(&mut game, opponent, "Forest");
    let second_land = exile_named(&mut game, opponent, "Island");
    let zero = exile_named(&mut game, opponent, "Mox Jasper");
    let instant = exile_named(&mut game, opponent, "Opt");
    let unrelated = put(&mut game, opponent, "Mox Jasper", Zone::Exile);
    let announce = life_cast(zero, CastFace::Normal);
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, life_cast(unrelated, CastFace::Normal));
    reject(&mut game, Action::AnnounceExile { card: zero });

    let mut copied = game.clone();
    copied.apply_temporary_copy(
        provider,
        CopyRecipe::new(Card::GrizzlyBears),
        CopyDeadline::EndOfTurn,
    );
    assert_eq!(copied.object_ref(source), provider);
    assert!(copied.copy_ability_origin(provider).is_some());
    reject(&mut copied, announce.clone());

    let mut returned = game.clone();
    returned.move_object(source, Zone::Graveyard);
    returned.move_object(source, Zone::Battlefield);
    assert_ne!(returned.object_ref(source), provider);
    reject(&mut returned, announce.clone());

    let mut reexiled = game.clone();
    let before = reexiled.object_ref(zero);
    reexiled.move_object(zero, Zone::Hand);
    reexiled.move_object(zero, Zone::Exile);
    assert_ne!(reexiled.object_ref(zero), before);
    reject(&mut reexiled, announce.clone());

    let mut transferred = game.clone();
    transferred.change_controller(provider, opponent);
    reject(&mut transferred, Action::PlayLand { card: land });
    reject(&mut transferred, announce.clone());
    act(&mut transferred, Action::Pass);
    assert_eq!(transferred.actor(), Some(opponent));
    reject(&mut transferred, life_cast(instant, CastFace::Normal));
    reach_next_main(&mut transferred);
    assert_eq!(transferred.state.active, opponent);
    assert!(transferred.legal_actions().contains(&announce));
    act(&mut transferred, Action::PlayLand { card: land });
    assert_eq!(transferred.object_view(land).controller, opponent);
    assert_eq!(transferred.object_ref(source), provider);

    act(&mut game, Action::PlayLand { card: land });
    let entered = game.object_view(land);
    assert_eq!(entered.zone, Zone::Battlefield);
    assert_eq!(entered.owner, opponent);
    assert_eq!(entered.controller, player);
    reject(&mut game, Action::PlayLand { card: second_land });
    act(&mut game, announce);
    finish_costs(&mut game);
    let item = game.state.stack.last().unwrap();
    assert_eq!(item.mana_value(), 0);
    assert_eq!(item.cast.unwrap().method, CastMethod::ExileLife);
    assert_eq!(item.cast.unwrap().payment, [0; 6]);
    settle(&mut game);
    assert_eq!(game.object_view(zero).owner, opponent);
    assert_eq!(game.object_view(zero).controller, player);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(game.state.players[player.index()].life_lost_this_turn, 0);
}

#[test]
fn life_cast_keeps_tax_and_sacrifice_after_the_only_granting_source_leaves() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SOURCE),
        (Card::from_name("Short Bow").unwrap(), &TAX),
    ]);
    let player = Player::P1;
    let opponent = player.other();
    let mut parent = scenario(player);
    let source = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let provider = parent.object_ref(source);
    put(&mut parent, opponent, "Short Bow", Zone::Battlefield);
    let spell = exile_named(&mut parent, opponent, "Corrupted Conviction");
    let announce = life_cast(spell, CastFace::Normal);
    parent.state.players[player.index()].mana = [0; 6];
    reject(&mut parent, announce.clone());
    let tax = [0, 0, 0, 0, 0, 2];
    parent.state.players[player.index()].mana = tax;
    reject(&mut parent, Action::AnnounceExile { card: spell });
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == announce)
        .unwrap();
    let revision = parent.revision();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    act(&mut raw, announce);
    indexed.apply_index(player, revision, index).unwrap();
    for game in [&mut raw, &mut indexed] {
        if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(game);
        }
        choose_clause(game, CastCostClause::Additional);
        reject(
            game,
            Action::ChooseAdditionalCost {
                object: ObjectRef {
                    card: source,
                    generation: provider.generation + 1,
                },
            },
        );
        act(game, Action::ChooseAdditionalCost { object: provider });
        assert_eq!(game.object_view(source).zone, Zone::Graveyard);
        assert_eq!(game.state.players[player.index()].life, 20);
        choose_clause(game, CastCostClause::AlternativeLife(1));
        finish_costs(game);
        let item = game.state.stack.last().unwrap();
        assert_eq!(item.cast.unwrap().method, CastMethod::ExileLife);
        assert_eq!(item.cast.unwrap().origin, Zone::Exile);
        assert_eq!(item.cast.unwrap().payment, tax);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        assert_eq!(game.state.players[player.index()].life, 19);
        settle(game);
        assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(game.hand_count(player), 2);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());

    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    indexed
        .grant_exile_play_permission_with_expiry(
            player,
            indexed.object_ref(spell),
            None,
            ExilePlayExpiry::WhileExiled,
        )
        .unwrap();
    let ordinary = [0, 0, 1, 0, 0, 2];
    indexed.state.players[player.index()].mana = ordinary;
    assert!(
        indexed
            .legal_actions()
            .contains(&life_cast(spell, CastFace::Normal))
    );
    act(&mut indexed, Action::AnnounceExile { card: spell });
    finish_costs(&mut indexed);
    let cast = indexed.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.method, CastMethod::Normal);
    assert_eq!(cast.payment, ordinary);
    assert_eq!(indexed.state.players[player.index()].life, 20);
    assert_eq!(raw.state.players[player.index()].life, 19);
}

#[test]
fn each_room_half_uses_its_own_mana_value_and_only_opens_its_selected_door() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SOURCE),
        (ROOM, &ROOM_RULES),
        (LEFT, &HALF_RULES),
        (RIGHT, &HALF_RULES),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let room = exile_for_source(&mut game, opponent, ROOM);
    reject(
        &mut game,
        Action::AnnounceRoom {
            card: room,
            door: RoomDoor::Left,
            flash_surcharge: false,
        },
    );
    reject(&mut game, life_cast(room, CastFace::Normal));
    game.state.players[player.index()].mana = [0; 6];
    game.state.players[player.index()].life = 2;
    assert!(
        game.legal_actions()
            .contains(&life_cast(room, CastFace::RoomLeft))
    );
    reject(&mut game, life_cast(room, CastFace::RoomRight));
    game.state.players[player.index()].life = 20;
    let saved = game.snapshot();
    for (face, cost, doors) in [
        (
            CastFace::RoomLeft,
            2,
            RoomDoors {
                left: true,
                right: false,
            },
        ),
        (
            CastFace::RoomRight,
            3,
            RoomDoors {
                left: false,
                right: true,
            },
        ),
    ] {
        game.restore(&saved).unwrap();
        act(&mut game, life_cast(room, face));
        finish_costs(&mut game);
        let item = game.state.stack.last().unwrap();
        assert_eq!(item.mana_value(), cost);
        assert_eq!(item.cast.unwrap().face, face);
        assert_eq!(item.cast.unwrap().method, CastMethod::ExileLife);
        assert_eq!(
            game.state.players[player.index()].life,
            20 - i128::from(cost)
        );
        settle(&mut game);
        let view = game.object_view(room);
        assert_eq!(view.zone, Zone::Battlefield);
        assert_eq!(view.owner, opponent);
        assert_eq!(view.controller, player);
        assert_eq!(view.room_doors, Some(doors));
    }
}

#[test]
fn life_replaces_only_the_base_mana_cost_and_x_stays_zero() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let lightning = exile_named(&mut game, opponent, "Burst Lightning");
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
    act(&mut game, life_cast(lightning, CastFace::Normal));
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
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 19);
    assert_eq!(game.state.players[opponent.index()].life, 16);

    let x_spell = exile_named(&mut game, opponent, "Wildwood Scourge");
    act(&mut game, life_cast(x_spell, CastFace::Normal));
    reject(&mut game, Action::ChooseX { value: 1 });
    finish_costs(&mut game);
    assert_eq!(game.state.stack.last().unwrap().x, Some(0));
    assert_eq!(game.state.players[player.index()].life, 18);
}
