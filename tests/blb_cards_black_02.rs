//! Bloomburrow exact hand, graveyard, and attack choices.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn select_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn gifted_cast(game: &mut Game, spell: CardId, target: Target, gift: bool) {
    let beneficiary = gift.then(|| game.actor().unwrap().other());
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseGift { beneficiary });
    select_target(game, target);
    pay(game);
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

#[test]
fn coiling_rebirth_copies_the_returned_nonlegendary_creature_only_if_gifted() {
    let player = Player::P1;
    for (gift, legendary) in [(false, false), (true, false), (true, true)] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Coiling Rebirth", Zone::Hand);
        let target = put(
            &mut game,
            player,
            if legendary {
                "Maha, Its Feathers Night"
            } else {
                "Serra Angel"
            },
            Zone::Graveyard,
        );
        gifted_cast(&mut game, spell, Target::Card(target), gift);
        let mut stale = game.clone();
        stale.move_object(target, Zone::Exile);
        settle(&mut stale);
        assert_eq!(
            stale.state.players[player.other().index()].cards_drawn_this_turn,
            0
        );
        settle(&mut game);
        assert_eq!(game.object_view(target).zone, Zone::Battlefield);
        assert_eq!(
            game.state.players[player.other().index()].cards_drawn_this_turn,
            u64::from(gift)
        );
        let copy = game
            .state
            .objects
            .iter()
            .position(|object| object.is_copy_token())
            .map(|id| id as CardId);
        assert_eq!(copy.is_some(), gift && !legendary);
        if let Some(copy) = copy {
            assert_eq!(game.object_view(copy).card, game.object_view(target).card);
            assert_eq!(stats(&game, copy), (1, 1));
            assert!(game.object_view(copy).keywords.contains(Keywords::FLYING));
        }
    }
}

#[test]
fn cruelclaws_heist_can_take_the_gift_draw_and_cast_it_turns_later() {
    let player = Player::P0;
    let opponent = player.other();
    for gift in [false, true] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Cruelclaw's Heist", Zone::Hand);
        let land = put(&mut game, opponent, "Forest", Zone::Hand);
        let creature = put(
            &mut game,
            opponent,
            "Grizzly Bears",
            if gift { Zone::Library } else { Zone::Hand },
        );
        gifted_cast(&mut game, spell, Target::Player(opponent), gift);
        pass_pair(&mut game);
        assert!(
            game.observe(player)
                .objects
                .iter()
                .any(|object| object.id == land)
        );
        let bad = Action::ChooseHandCard {
            object: Some(game.object_ref(land)),
        };
        reject(&mut game, bad);
        let exact = game.object_ref(creature);
        act(
            &mut game,
            Action::ChooseHandCard {
                object: Some(exact),
            },
        );
        assert_eq!(game.object_view(creature).zone, Zone::Exile);
        game.state.turn += 4;
        game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 0];
        assert_eq!(
            game.permitted_cast_available(
                player,
                creature,
                CastMethod::Normal,
                CastFace::Normal,
                true,
                false
            ),
            gift
        );
        game.move_object(creature, Zone::Graveyard);
        game.move_object(creature, Zone::Exile);
        assert!(!game.permitted_cast_available(
            player,
            creature,
            CastMethod::Normal,
            CastFace::Normal,
            true,
            false
        ));
    }
}

#[test]
fn early_winter_chooses_the_creature_mode_or_gives_an_opponent_an_untargeted_choice() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let winter = put(&mut game, player, "Early Winter", Zone::Hand);
    let creature = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let enchantment = put(&mut game, opponent, "Banishing Light", Zone::Battlefield);
    game.state.objects[enchantment as usize].gained_keywords = Keywords::HEXPROOF;
    for (mode, target, exiled) in [
        (0, Target::Card(creature), creature),
        (1, Target::Player(opponent), enchantment),
    ] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: winter });
        act(&mut branch, Action::ChooseMode { mode });
        select_target(&mut branch, target);
        pay(&mut branch);
        pass_pair(&mut branch);
        if mode == 1 && branch.state.pending != Pending::Priority {
            assert_eq!(branch.actor(), Some(opponent));
            let object = branch.object_ref(enchantment);
            act(&mut branch, Action::SelectResolutionExile { object });
        }
        settle(&mut branch);
        assert_eq!(branch.object_view(exiled).zone, Zone::Exile);
    }
}

#[test]
fn osteomancer_adept_supports_repeated_casts_with_independent_forage_and_finality() {
    let player = Player::P0;
    let mut game = scenario(player);
    let adept = put(&mut game, player, "Osteomancer Adept", Zone::Battlefield);
    assert!(
        game.object_view(adept)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
    choose(
        &mut game,
        |action| matches!(action, Action::Activate { card, .. } if *card == adept),
    );
    settle(&mut game);
    assert!(game.object_view(adept).tapped);
    let permission = game.observe(player).paid_graveyard_cast_permissions[0].id;
    for _ in 0..2 {
        let bear = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
        let food = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
        let object = game.object_ref(bear);
        act(
            &mut game,
            Action::AnnounceGraveyardPaid {
                object,
                permission,
                face: CastFace::Normal,
                method: CastMethod::Normal,
                flash_surcharge: false,
            },
        );
        pay(&mut game);
        let index = game
            .observe(player)
            .cast_costs
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::OriginForage && !row.paid)
            .unwrap() as u16;
        act(&mut game, Action::ChooseCastCost { clause: index });
        act(
            &mut game,
            Action::ChooseForageRoute {
                route: ForageRoute::Food,
            },
        );
        let object = game.object_ref(food);
        act(&mut game, Action::ChooseForage { object });
        settle(&mut game);
        assert_eq!(game.object_view(food).zone, Zone::Graveyard);
        assert_eq!(game.counter_count(bear, CounterKind::Finality), 1);
        game.move_object(bear, Zone::Graveyard);
        assert_eq!(game.object_view(bear).zone, Zone::Exile);
    }
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert!(
        game.observe(player)
            .paid_graveyard_cast_permissions
            .is_empty()
    );
}

#[test]
fn persistent_marshstalker_requires_threshold_twice_and_returns_tapped_attacking() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rat = put(&mut game, player, "Daggerfang Duo", Zone::Battlefield);
    let second_rat = put(&mut game, player, "Burglar Rat", Zone::Battlefield);
    let marshstalker = put(
        &mut game,
        player,
        "Persistent Marshstalker",
        Zone::Graveyard,
    );
    for _ in 0..5 {
        put(&mut game, player, "Forest", Zone::Graveyard);
    }
    let mut short = game.clone();
    attack(&mut short, &[rat, second_rat]);
    assert!(short.state.stack.is_empty());
    let seventh = put(&mut game, player, "Forest", Zone::Graveyard);
    attack(&mut game, &[rat, second_rat]);
    assert_eq!(game.state.stack.len(), 1);
    let mut response = game.clone();
    response.move_object(seventh, Zone::Exile);
    settle(&mut response);
    assert_eq!(response.object_view(marshstalker).zone, Zone::Graveyard);
    let payment = [0, 0, 1, 0, 2, 0];
    game.state.players[player.index()].mana = payment;
    pass_pair(&mut game);
    act(&mut game, Action::PayResolutionMana { payment });
    if matches!(game.state.pending, Pending::EntryAttackingDefender { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::ChooseEntryAttackingDefender { .. })
        });
    }
    settle(&mut game);
    assert_eq!(game.object_view(marshstalker).zone, Zone::Battlefield);
    assert!(game.object_view(marshstalker).tapped);
    assert_eq!(stats(&game, marshstalker), (5, 1));
    assert!(game.observe(player).attackers.contains(&marshstalker));
}

#[test]
fn ruthless_negotiation_exiles_the_opponents_choice_and_draws_only_from_the_graveyard() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Ruthless Negotiation", Zone::Hand);
    let land = put(&mut game, opponent, "Forest", Zone::Hand);
    let creature = put(&mut game, opponent, "Grizzly Bears", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(opponent)));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    assert!(
        !game
            .observe(player)
            .objects
            .iter()
            .any(|object| object.id == creature)
    );
    let object = game.object_ref(land);
    act(
        &mut game,
        Action::ChooseHandCard {
            object: Some(object),
        },
    );
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    act(&mut game, Action::AnnounceFlashback { card: spell });
    select_target(&mut game, Target::Player(opponent));
    pay(&mut game);
    pass_pair(&mut game);
    if game.state.pending != Pending::Priority {
        let object = game.object_ref(creature);
        act(
            &mut game,
            Action::ChooseHandCard {
                object: Some(object),
            },
        );
    }
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn thought_stalker_warlock_switches_the_chooser_and_eligible_cards_after_life_loss() {
    let player = Player::P0;
    let opponent = player.other();
    for lost_life in [false, true] {
        let mut game = scenario(player);
        let warlock = put(&mut game, player, "Thought-Stalker Warlock", Zone::Hand);
        let land = put(&mut game, opponent, "Forest", Zone::Hand);
        let nonland = put(&mut game, opponent, "Grizzly Bears", Zone::Hand);
        if lost_life {
            let duo = put(&mut game, player, "Glidedive Duo", Zone::Hand);
            cast(&mut game, duo, None);
            settle(&mut game);
        }
        cast(&mut game, warlock, None);
        pass_pair(&mut game);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(opponent),
            },
        );
        if game.legal_actions().contains(&Action::FinishStackTargets) {
            act(&mut game, Action::FinishStackTargets);
        }
        pass_pair(&mut game);
        if lost_life {
            assert_eq!(game.actor(), Some(player));
            let land_object = game.object_ref(land);
            reject(
                &mut game,
                Action::ChooseRevealedDiscard {
                    object: land_object,
                },
            );
            let object = game.object_ref(nonland);
            act(&mut game, Action::ChooseRevealedDiscard { object });
            assert_eq!(game.object_view(nonland).zone, Zone::Graveyard);
        } else {
            assert_eq!(game.actor(), Some(opponent));
            act(&mut game, Action::Discard { card: land });
            assert_eq!(game.object_view(land).zone, Zone::Graveyard);
            assert_eq!(game.object_view(nonland).zone, Zone::Hand);
        }
        assert!(
            game.object_view(warlock)
                .keywords
                .contains(Keywords::MENACE)
        );
    }
}

#[test]
fn valley_rotcaller_counts_each_other_qualifying_permanent_at_resolution() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let rotcaller = put(&mut game, player, "Valley Rotcaller", Zone::Battlefield);
    let rat = put(&mut game, player, "Daggerfang Duo", Zone::Battlefield);
    let changeling = game.state.objects.len() as CardId;
    let token = Card::from_name("@token/shapeshifter_1_1_colorless_changeling").unwrap();
    game.create_tokens(player, token, 1, false);
    assert!(game.has_subtype(changeling, "Bat"));
    put(&mut game, opponent, "Daggerfang Duo", Zone::Battlefield);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    attack(&mut game, &[rotcaller]);
    for (remove_rat, count) in [(false, 2), (true, 1)] {
        let mut branch = game.clone();
        if remove_rat {
            branch.move_object(rat, Zone::Exile);
        }
        settle(&mut branch);
        assert_eq!(branch.state.players[player.index()].life, 20 + count);
        assert_eq!(branch.state.players[opponent.index()].life, 20 - count);
    }
}
