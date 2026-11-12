//! Authored library exile, linked play, opening-hand, copy, and attachment rules.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn choose_target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

#[test]
fn doomsday_excruciator_requires_a_cast_and_keeps_each_librarys_exact_bottom_six() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let initial = [player, opponent].map(|p| game.state.players[p.index()].library.clone());
    for &card in &initial[opponent.index()][5..] {
        game.move_object(card, Zone::Outside);
    }
    let own: Vec<_> = game.state.players[player.index()]
        .library
        .iter()
        .copied()
        .collect();
    let opposing: Vec<_> = game.state.players[opponent.index()]
        .library
        .iter()
        .copied()
        .collect();
    let source = put(&mut game, player, "Doomsday Excruciator", Zone::Hand);

    let mut put_into_play = game.clone();
    put_into_play.move_object(source, Zone::Battlefield);
    put_into_play.prepare_triggers();
    assert!(put_into_play.state.stack.is_empty());
    assert_eq!(
        &put_into_play.state.players[player.index()].library[..],
        own.as_slice()
    );

    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (6, 6));
    assert!(game.object_view(source).keywords.contains(Keywords::FLYING));
    assert_eq!(&game.state.players[player.index()].library[..], &own[..6]);
    assert_eq!(
        &game.state.players[opponent.index()].library[..],
        opposing.as_slice()
    );
    for &card in &own[6..] {
        let view = game.object_view(card);
        assert_eq!(view.zone, Zone::Exile);
        assert_eq!(view.kinds, 0);
        assert!(game.exiled_face_down(view.reference));
        reject(&mut game, Action::AnnounceExile { card });
    }
    for observer in [player, opponent] {
        let view = game.observe(observer);
        assert!(view.last_library_reveal.is_none());
        assert!(view.known_library_top.iter().all(Option::is_none));
        assert!(view.exile_permissions.is_empty());
    }
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(own[5]).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn killers_mask_attaches_to_its_manifested_creature_and_equip_moves_menace() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let alternate = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let other = put(&mut game, player, "Mountain", Zone::Library);
    let face_down = put(&mut game, player, "Hill Giant", Zone::Library);
    let mask = put(&mut game, player, "Killer's Mask", Zone::Hand);
    cast(&mut game, mask, None);
    pass_pair(&mut game);
    let mut left = game.clone();
    left.move_object(mask, Zone::Graveyard);
    pass_pair(&mut left);
    pass_pair(&mut game);
    assert!(game.observe(opponent).manifest_choice.is_none());
    let action = Action::ChooseManifest {
        object: game.object_ref(face_down),
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|choice| *choice == action)
        .unwrap();
    let pending = game.snapshot();
    let mut indexed = game.clone();
    act(&mut game, action.clone());
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&pending).unwrap();
    assert_eq!(indexed.snapshot(), pending);
    act(&mut left, action);
    assert_eq!(left.object_view(face_down).zone, Zone::Battlefield);
    assert_eq!(left.object_view(mask).attached_to, None);
    assert!(
        !left
            .object_view(face_down)
            .keywords
            .contains(Keywords::MENACE)
    );
    assert_eq!(game.object_view(other).zone, Zone::Graveyard);
    assert_eq!(
        game.object_view(mask).attached_to,
        Some(game.object_ref(face_down))
    );
    assert!(
        game.object_view(face_down)
            .keywords
            .contains(Keywords::MENACE)
    );
    assert_eq!(
        game.observe(opponent).face_down_permanents[0].underlying,
        None
    );

    act(
        &mut game,
        Action::AnnounceActivate {
            card: mask,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    choose_target(&mut game, alternate);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.object_view(mask).attached_to,
        Some(game.object_ref(alternate))
    );
    assert!(
        game.object_view(alternate)
            .keywords
            .contains(Keywords::MENACE)
    );
    assert!(
        !game
            .object_view(face_down)
            .keywords
            .contains(Keywords::MENACE)
    );
}

#[test]
fn either_leyline_can_begin_on_the_battlefield_after_keeping_hands() {
    let player = Player::P1;
    let resonance = Card::from_name("Leyline of Resonance").unwrap();
    let void = Card::from_name("Leyline of the Void").unwrap();
    let mut deck = vec![Card::Forest; 5];
    deck.extend([resonance, void]);
    let mut game = Game::new(Config {
        decks: [vec![Card::Forest; 7], deck],
        seed: 20261023,
        starting_player: player,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    act(&mut game, Action::Keep);
    act(&mut game, Action::Keep);
    let choices: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter_map(|action| {
            if let Action::BeginOnBattlefield { object } = action {
                Some(object)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(choices.len(), 2);
    for entered in choices.iter().copied() {
        let mut branch = game.clone();
        act(&mut branch, Action::BeginOnBattlefield { object: entered });
        act(&mut branch, Action::FinishOpeningHand);
        act(&mut branch, Action::FinishOpeningHand);
        assert_eq!(branch.object_view(entered.card).zone, Zone::Battlefield);
        let kept = choices.iter().find(|&&other| other != entered).unwrap();
        assert_eq!(branch.object_view(kept.card).zone, Zone::Hand);
        assert!(branch.state.stack.is_empty());
    }
}

#[test]
fn leyline_of_resonance_counts_one_distinct_creature_across_two_target_groups() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let leyline = put(&mut game, player, "Leyline of Resonance", Zone::Battlefield);
    let own = put(&mut game, player, "Jade-Cast Sentinel", Zone::Battlefield);
    let opposing_creature = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let opposing_artifact = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    let spell = put(&mut game, player, "Explosive Derailment", Zone::Hand);
    let prepare = |branch: &mut Game, creature, artifact| {
        act(branch, Action::AnnounceCast { card: spell });
        act(branch, Action::ChooseMode { mode: 2 });
        choose_target(branch, creature);
        choose_target(branch, artifact);
        pay(branch);
    };
    let mut separate = game.clone();
    prepare(&mut separate, own, opposing_artifact);
    assert_eq!(separate.state.stack.len(), 1);

    prepare(&mut game, own, own);
    assert_eq!(game.state.stack.len(), 2);
    assert_eq!(game.state.stack.last().unwrap().source.unwrap().id, leyline);
    pass_pair(&mut game);
    let copy = game.retarget_view().unwrap().item;
    assert!(copy.is_copy);
    assert_eq!(copy.mode, Some(2));
    assert_eq!(copy.targets[0].object, copy.targets[1].object);
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(opposing_creature),
        },
    );
    act(
        &mut game,
        Action::SetStackTarget {
            index: 1,
            target: Target::Card(opposing_artifact),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    assert_eq!(game.state.stack.len(), 2);
    settle(&mut game);
    for card in [own, opposing_creature, opposing_artifact] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
}

#[test]
fn leyline_of_the_void_checks_ownership_from_each_origin_and_ignores_tokens() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let leyline = put(&mut game, player, "Leyline of the Void", Zone::Hand);
    cast(&mut game, leyline, None);
    settle(&mut game);
    for (name, zone) in [
        ("Opt", Zone::Library),
        ("Forest", Zone::Hand),
        ("Bear Cub", Zone::Battlefield),
    ] {
        let owned = put(&mut game, player, name, zone);
        let opposing = put(&mut game, opponent, name, zone);
        if zone == Zone::Battlefield {
            game.change_controller(game.object_ref(owned), opponent);
            game.change_controller(game.object_ref(opposing), player);
        }
        game.move_object(owned, Zone::Graveyard);
        game.move_object(opposing, Zone::Graveyard);
        assert_eq!(game.object_view(owned).zone, Zone::Graveyard);
        assert_eq!(game.object_view(opposing).zone, Zone::Exile);
    }
    game.create_tokens(opponent, Card::from_catalogue_id(5258), 1, false);
    let token = game.controlled_token_ids(opponent).next().unwrap();
    game.move_object(token, Zone::Graveyard);
    assert_ne!(game.object_view(token).zone, Zone::Exile);
    game.move_object(leyline, Zone::Graveyard);
    let next = put(&mut game, opponent, "Opt", Zone::Hand);
    game.move_object(next, Zone::Graveyard);
    assert_eq!(game.object_view(next).zone, Zone::Graveyard);
}

#[test]
fn valgavoth_links_only_cards_it_exiles_and_plays_the_land_and_life_routes() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Valgavoth, Terror Eater",
        Zone::Battlefield,
    );
    let borrowed = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    game.change_controller(game.object_ref(borrowed), player);
    game.move_object(borrowed, Zone::Graveyard);
    assert_eq!(game.object_view(borrowed).zone, Zone::Graveyard);
    let land = put(&mut game, opponent, "Island", Zone::Hand);
    let spell = put(&mut game, opponent, "Quick Study", Zone::Hand);
    let unrelated = put(&mut game, opponent, "Quick Study", Zone::Exile);
    for card in [land, spell] {
        game.move_object(card, Zone::Graveyard);
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    let life_route = |card| Action::AnnounceExileLife {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    reject(&mut game, life_route(unrelated));
    reject(&mut game, Action::AnnounceExile { card: spell });
    let mut stale = game.clone();
    stale.move_object(source, Zone::Exile);
    stale.move_object(source, Zone::Battlefield);
    reject(&mut stale, life_route(spell));
    reject(&mut stale, Action::PlayLand { card: land });

    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).owner, opponent);
    assert_eq!(game.object_view(land).controller, player);
    act(&mut game, life_route(spell));
    assert_eq!(game.state.players[player.index()].life, 20);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let life_cost = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::AlternativeLife(3) && !row.paid)
        .expect("three-life alternative cost after closing the mana window");
    act(
        &mut game,
        Action::ChooseCastCost {
            clause: life_cost.try_into().unwrap(),
        },
    );
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::ExileLife
    );
    assert_eq!(game.state.players[player.index()].life, 17);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn valgavoth_ward_cost_is_exactly_three_nonland_permanents_chosen_by_the_caster() {
    let caster = Player::P0;
    let owner = caster.other();
    let mut game = scenario(caster);
    let source = put(
        &mut game,
        owner,
        "Valgavoth, Terror Eater",
        Zone::Battlefield,
    );
    let own = ["Bear Cub", "Candy Trail", "Pacifism"].map(|name| {
        let zone = if name == "Pacifism" {
            Zone::Hand
        } else {
            Zone::Battlefield
        };
        put(&mut game, caster, name, zone)
    });
    let other = put(&mut game, owner, "Hill Giant", Zone::Battlefield);
    cast(&mut game, own[2], Some(Target::Card(other)));
    settle(&mut game);
    let land = put(&mut game, caster, "Forest", Zone::Battlefield);
    let spell = put(&mut game, caster, "Murder", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(source)));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(caster));
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    assert_eq!(declined.object_view(source).zone, Zone::Battlefield);
    assert_eq!(declined.object_view(spell).zone, Zone::Exile);
    let choose = |g: &Game, card| Action::ChooseAdditionalCost {
        object: g.object_ref(card),
    };
    for card in [land, other] {
        let action = choose(&game, card);
        reject(&mut game, action);
    }
    for card in own {
        let action = choose(&game, card);
        act(&mut game, action);
    }
    for card in own {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    for card in own {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
}
