use crate::support::*;
use rustmage::PaidCastOrigin;
use rustmage_core::{cards::*, engine::*};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn evidence(game: &mut Game, cards: &[CardId]) {
    for &card in cards {
        let object = game.object_ref(card);
        act(game, Action::ChooseEvidence { object });
    }
    act(game, Action::FinishEvidence);
}

fn choose_cost(game: &mut Game, kind: CastCostClause) {
    let index = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == kind)
        .unwrap();
    act(
        game,
        Action::ChooseCastCost {
            clause: index.try_into().unwrap(),
        },
    );
}

#[test]
fn surprise_witness_returns_the_exact_small_creature_then_adds_flying_and_spirit() {
    let player = Player::P0;
    let mut game = scenario(player);
    let witness = put(&mut game, player, "Call a Surprise Witness", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let opponent = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
    let giant = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let old = game.object_ref(bear);
    act(&mut game, Action::AnnounceCast { card: witness });
    reject(&mut game, target(opponent));
    reject(&mut game, target(giant));
    act(&mut game, target(bear));
    pay(&mut game);
    settle(&mut game);
    let returned = game.object_view(bear);
    assert_ne!(returned.reference, old);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert!(returned.has_subtype("Spirit") && returned.has_subtype("Bear"));
    assert!(returned.keywords.contains(Keywords::FLYING));
    assert_eq!(game.counter_count(bear, CounterKind::Flying), 1);
    game.move_object(bear, Zone::Exile);
    game.move_object(bear, Zone::Battlefield);
    assert!(!game.object_view(bear).has_subtype("Spirit"));
    assert!(!game.object_view(bear).keywords.contains(Keywords::FLYING));
}

#[test]
fn no_witnesses_investigates_for_every_player_tied_for_most_even_at_zero() {
    let player = Player::P0;
    for (counts, clues) in [([0, 0], [1, 1]), ([1, 1], [1, 1]), ([1, 2], [0, 1])] {
        let mut game = scenario(player);
        let mut creatures = Vec::new();
        for seat in [player, player.other()] {
            for _ in 0..counts[seat.index()] {
                creatures.push(put(&mut game, seat, "Grizzly Bears", Zone::Battlefield));
            }
        }
        let wipe = put(&mut game, player, "No Witnesses", Zone::Hand);
        cast(&mut game, wipe, None);
        settle(&mut game);
        assert!(
            creatures
                .iter()
                .all(|&card| game.object_view(card).zone == Zone::Graveyard)
        );
        for seat in [player, player.other()] {
            assert_eq!(
                game.controlled_token_ids(seat)
                    .filter(|&id| game.object_view(id).card == Card::from_catalogue_id(5296))
                    .count(),
                clues[seat.index()]
            );
        }
    }
}

#[test]
fn tenth_district_hero_can_pay_either_cost_and_keeps_the_name_after_reapplying_first_form() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hero = put(&mut game, player, "Tenth District Hero", Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let rival = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    for (ability, fuel, expected) in [
        (1, "Hill Giant", (2, 3)),
        (0, "Grizzly Bears", (4, 4)),
        (1, "Hill Giant", (5, 5)),
        (0, "Grizzly Bears", (4, 4)),
    ] {
        game.state.players[player.index()].mana = [8; 6];
        let fuel = put(&mut game, player, fuel, Zone::Graveyard);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: hero,
                ability,
                counter_payment: Counters::default(),
            },
        );
        evidence(&mut game, &[fuel]);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(fuel).zone, Zone::Exile);
        assert_eq!(stats(&game, hero), expected);
    }
    let named = game.object_view(hero);
    assert_eq!(named.name, "Mileva, the Stalwart");
    assert!(named.is_legendary() && named.has_subtype("Human") && named.has_subtype("Detective"));
    assert!(named.keywords.contains(Keywords::VIGILANCE));
    assert!(!named.keywords.contains(Keywords::INDESTRUCTIBLE));
    assert!(
        game.object_view(ally)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );
    assert!(
        !game
            .object_view(rival)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );
    game.move_object(hero, Zone::Exile);
    game.move_object(hero, Zone::Battlefield);
    assert_eq!(game.object_view(hero).name, "Tenth District Hero");
    assert!(
        !game
            .object_view(ally)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );
}

#[test]
fn agency_outfitter_searches_only_selected_zones_and_finds_at_most_one_per_name() {
    let player = Player::P1;
    let mut game = scenario(player);
    let outfitter = put(&mut game, player, "Agency Outfitter", Zone::Hand);
    let glass = Card::from_name("Magnifying Glass").unwrap();
    let cap = Card::from_name("Thinking Cap").unwrap();
    let from_hand = spawn(&mut game, player, glass, Zone::Hand);
    let duplicate = spawn(&mut game, player, glass, Zone::Graveyard);
    let from_graveyard = spawn(&mut game, player, cap, Zone::Graveyard);
    let from_library = spawn(&mut game, player, cap, Zone::Library);
    let rng = game.state.rng;
    cast(&mut game, outfitter, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let mut library = game.clone();
    act(
        &mut library,
        Action::ChooseGraveyardHandLibrarySearchZones {
            graveyard: false,
            hand: false,
            library: true,
        },
    );
    act(&mut library, Action::FinishSearch);
    assert_ne!(library.state.rng, rng);

    act(
        &mut game,
        Action::ChooseGraveyardHandLibrarySearchZones {
            graveyard: true,
            hand: true,
            library: false,
        },
    );
    reject(&mut game, Action::ChooseSearch { card: from_library });
    act(&mut game, Action::ChooseSearch { card: from_hand });
    reject(&mut game, Action::ChooseSearch { card: duplicate });
    act(
        &mut game,
        Action::ChooseSearch {
            card: from_graveyard,
        },
    );
    let hidden = game.observe(player.other()).hand_library_search.unwrap();
    assert!(hidden.cards.is_none() && hidden.named_selections.is_none());
    act(&mut game, Action::FinishSearch);
    while let Some(action) = game
        .legal_actions()
        .into_iter()
        .find(|action| matches!(action, Action::ChooseEntryTimestamp { .. }))
    {
        act(&mut game, action);
    }
    settle(&mut game);
    assert_eq!(game.state.rng, rng);
    assert_eq!(game.object_view(from_hand).zone, Zone::Battlefield);
    assert_eq!(game.object_view(from_graveyard).zone, Zone::Battlefield);
    assert_eq!(game.object_view(duplicate).zone, Zone::Graveyard);
    assert!(
        game.object_view(outfitter)
            .keywords
            .contains(Keywords::FLYING)
    );
}

#[test]
fn behind_the_mask_uses_only_its_linked_evidence_when_unraveler_pays_for_the_spell() {
    let player = Player::P0;
    let mut game = scenario(player);
    let unraveler = put(&mut game, player, "Conspiracy Unraveler", Zone::Battlefield);
    assert!(
        game.object_view(unraveler)
            .keywords
            .contains(Keywords::FLYING)
    );
    let mask = put(&mut game, player, "Behind the Mask", Zone::Hand);
    let artifact = put(&mut game, player, "Candlestick", Zone::Battlefield);
    let origin_fuel = spawn(
        &mut game,
        player,
        Card::from_name("Omniscience").unwrap(),
        Zone::Graveyard,
    );
    let first = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let second = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, Action::AnnounceCast { card: mask });
    let alternative = Action::AnnounceCastWithEvidenceAlternative {
        object: game.object_ref(mask),
        provider: game.object_ref(unraveler),
        face: CastFace::Normal,
        threshold: 10,
        flash_surcharge: false,
        origin: PaidCastOrigin::Ordinary,
    };
    act(&mut game, alternative);
    let mut alternative_only = game.clone();
    for (branch, collect) in [(&mut alternative_only, false), (&mut game, true)] {
        act(branch, Action::ChooseCollectEvidence { collect });
        act(branch, target(artifact));
        act(branch, Action::BeginOrderedCosts);
        choose_cost(branch, CastCostClause::OriginEvidence(10));
        evidence(branch, &[origin_fuel]);
        if collect {
            choose_cost(branch, CastCostClause::Additional);
            let paid = branch.object_ref(origin_fuel);
            reject(branch, Action::ChooseEvidence { object: paid });
            evidence(branch, &[first, second]);
            assert_eq!(
                branch
                    .observe(player.other())
                    .last_evidence
                    .unwrap()
                    .threshold,
                6
            );
        }
        choose_cost(branch, CastCostClause::Mana);
        pay(branch);
        settle(branch);
        let animated = branch.object_view(artifact);
        assert_eq!(animated.kinds & (1 | 16), 1 | 16);
        assert!(animated.has_subtype("Equipment"));
        assert_eq!(
            stats(branch, artifact),
            if collect { (1, 1) } else { (4, 3) }
        );
    }
}

#[test]
fn intrude_counts_actual_graveyard_arrivals_and_exposes_both_piles_to_the_opponent() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let intrude = put(&mut game, player, "Intrude on the Mind", Zone::Hand);
    let [forest, cub, shock, colossus, hawk] = [
        "Forest",
        "Bear Cub",
        "Shock",
        "Darksteel Colossus",
        "Healer's Hawk",
    ]
    .map(|name| put(&mut game, player, name, Zone::Library));
    cast(&mut game, intrude, None);
    pass_pair(&mut game);
    let public = game.observe(opponent).library_piles.unwrap();
    assert_eq!(public.remaining.len(), 5);
    for (card, pile) in [
        (forest, LibraryPile::First),
        (hawk, LibraryPile::First),
        (cub, LibraryPile::Second),
        (shock, LibraryPile::Second),
        (colossus, LibraryPile::Second),
    ] {
        let object = game.object_ref(card);
        act(&mut game, Action::AssignLibraryPile { object, pile });
    }
    assert_eq!(game.actor(), Some(opponent));
    assert!(
        game.observe(player)
            .library_piles
            .unwrap()
            .revealed_piles
            .is_some()
    );
    let choose_first = Action::ChooseLibraryPile {
        pile: LibraryPile::First,
    };
    let checkpoint = game.snapshot();
    let mut indexed = game.clone();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choose_first)
        .unwrap();
    indexed
        .apply_index(opponent, game.revision(), index)
        .unwrap();
    act(&mut game, choose_first);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(colossus).zone, Zone::Library);
    assert_eq!(game.object_view(cub).zone, Zone::Graveyard);
    assert!(
        game.observe(opponent)
            .known_hand
            .contains(&game.object_ref(hawk))
    );
    let token = game
        .controlled_token_ids(player)
        .find(|&id| game.object_view(id).card == Card::from_catalogue_id(5680))
        .unwrap();
    let thopter = game.object_view(token);
    assert_eq!(thopter.colors, 0);
    assert_eq!(thopter.kinds & (1 | 16), 1 | 16);
    assert!(thopter.has_subtype("Thopter") && thopter.keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, token), (2, 2));
    assert_eq!(game.counter_count(token, CounterKind::PlusOne), 2);

    indexed.restore(&checkpoint).unwrap();
    act(
        &mut indexed,
        Action::ChooseLibraryPile {
            pile: LibraryPile::Second,
        },
    );
    assert_eq!(indexed.object_view(colossus).zone, Zone::Hand);
    assert_eq!(indexed.object_view(hawk).zone, Zone::Graveyard);
}

#[test]
fn jaded_analyst_triggers_only_on_the_second_completed_draw() {
    let player = Player::P0;
    let mut game = scenario(player);
    let analyst = put(&mut game, player, "Jaded Analyst", Zone::Battlefield);
    assert!(
        game.object_view(analyst)
            .keywords
            .contains(Keywords::DEFENDER)
    );
    for ordinal in 1..=3 {
        let draw = put(&mut game, player, "Deduce", Zone::Hand);
        cast(&mut game, draw, None);
        pass_pair(&mut game);
        assert_eq!(game.state.stack.len(), usize::from(ordinal == 2));
        settle(&mut game);
        let keywords = game.object_view(analyst).keywords;
        assert_eq!(keywords.contains(Keywords::DEFENDER), ordinal < 2);
        assert_eq!(keywords.contains(Keywords::VIGILANCE), ordinal >= 2);
    }
}

#[test]
fn living_conundrum_skips_each_empty_draw_and_changes_stats_with_library_contents() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Living Conundrum", Zone::Hand);
    cast(&mut game, creature, None);
    settle(&mut game);
    assert_eq!(stats(&game, creature), (2, 5));
    let library = game.state.players[player.index()].library.clone();
    for card in library.into_iter().skip(1) {
        game.move_object(card, Zone::Exile);
    }
    let draw = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, draw, None);
    settle(&mut game);
    assert!(game.outcome().is_none());
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(stats(&game, creature), (10, 10));
    let keywords = game.object_view(creature).keywords;
    assert!(keywords.contains(Keywords::FLYING) && keywords.contains(Keywords::VIGILANCE));
    assert!(keywords.contains(Keywords::HEXPROOF));
    put(&mut game, player, "Forest", Zone::Library);
    assert_eq!(stats(&game, creature), (2, 5));
    assert!(
        !game
            .object_view(creature)
            .keywords
            .contains(Keywords::FLYING)
    );
}

#[test]
fn lost_in_the_maze_casts_with_flash_and_stuns_only_opposing_targets() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let maze = put(&mut game, player, "Lost in the Maze", Zone::Hand);
    let ours = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let theirs = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    game.state.objects[theirs as usize].tapped = true;
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: maze });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    pass_pair(&mut game);
    reject(&mut game, Action::FinishTargets);
    act(&mut game, target(ours));
    act(&mut game, target(theirs));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert!(game.object_view(ours).tapped && game.object_view(theirs).tapped);
    assert_eq!(game.counter_count(ours, CounterKind::Stun), 0);
    assert_eq!(game.counter_count(theirs, CounterKind::Stun), 1);
    assert!(game.object_view(ours).keywords.contains(Keywords::HEXPROOF));
    assert!(
        !game
            .object_view(theirs)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
}

#[test]
fn proft_stays_banned_in_standard_and_uses_actual_draws_in_freeform() {
    let player = Player::P0;
    let card = Card::from_name("Proft's Eidetic Memory").unwrap();
    assert!(card.is_supported());
    assert!(!card.is_standard_legal());
    let mut standard = vec![Card::Island; 60];
    standard[0] = card;
    assert!(matches!(
        Game::new(Config {
            decks: [standard, vec![Card::Island; 60]],
            seed: 83,
            starting_player: player,
            deck_rules: DeckRules::Standard,
        }),
        Err(Error::InvalidDeck(_))
    ));

    let mut game = scenario(player);
    let first_draw = put(&mut game, player, "Deduce", Zone::Hand);
    cast(&mut game, first_draw, None);
    settle(&mut game);
    let proft = put(&mut game, player, "Proft's Eidetic Memory", Zone::Hand);
    cast(&mut game, proft, None);
    settle(&mut game);
    let p = player.index();
    assert_eq!(game.state.players[p].cards_drawn_this_turn, 2);
    assert!(game.observe(player).players[p].no_maximum_hand_size);
    let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    act(&mut game, target(recipient));
    let response = put(&mut game, player, "Deduce", Zone::Hand);
    cast(&mut game, response, None);
    settle(&mut game);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 2);
    game.move_object(proft, Zone::Graveyard);
    assert!(!game.observe(player).players[p].no_maximum_hand_size);
}

#[test]
fn reenact_only_targets_this_turns_nonlands_and_casts_a_copy_owned_by_the_caster() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let reenact = put(&mut game, player, "Reenact the Crime", Zone::Hand);
    let old = put(&mut game, opponent, "Grizzly Bears", Zone::Graveyard);
    let recent = put(&mut game, opponent, "Bear Cub", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    game.move_object(recent, Zone::Graveyard);
    game.move_object(land, Zone::Graveyard);
    act(&mut game, Action::AnnounceCast { card: reenact });
    reject(&mut game, target(old));
    reject(&mut game, target(land));
    act(&mut game, target(recent));
    pay(&mut game);
    pass_pair(&mut game);
    let offer = game.observe(player).resolution_free_casts.unwrap();
    assert_eq!(offer.origin, Zone::Exile);
    assert_eq!(game.object_view(recent).zone, Zone::Exile);
    let copy = offer.remaining[0];
    let physical = game.object_ref(recent);
    reject(
        &mut game,
        Action::CastExiledDuringResolution {
            object: physical,
            face: CastFace::Normal,
        },
    );
    let mut declined = game.clone();
    act(&mut declined, Action::FinishResolutionCasts);
    assert_eq!(declined.object_view(copy.card).zone, Zone::Outside);
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object: copy,
            face: CastFace::Normal,
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    if game
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    let token = game.object_view(copy.card);
    assert_eq!(token.zone, Zone::Battlefield);
    assert_eq!(token.controller, player);
    assert_eq!(token.owner, player);
    assert_eq!(game.object_view(recent).zone, Zone::Exile);
}

#[test]
fn steamcore_scholar_draws_first_and_keeps_the_qualified_discard_choice_private() {
    let player = Player::P0;
    let mut game = scenario(player);
    let scholar = put(&mut game, player, "Steamcore Scholar", Zone::Hand);
    let ground = put(&mut game, player, "Bear Cub", Zone::Hand);
    let instant = put(&mut game, player, "Shock", Zone::Hand);
    let land = put(&mut game, player, "Island", Zone::Library);
    let flyer = put(&mut game, player, "Healer's Hawk", Zone::Library);
    cast(&mut game, scholar, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(
        game.observe(player).effect_choice.unwrap().candidates,
        [game.object_ref(instant), game.object_ref(flyer)]
    );
    assert!(game.observe(player.other()).effect_choice.is_none());
    reject(&mut game, Action::Discard { card: land });
    reject(&mut game, Action::Discard { card: ground });
    let mut two = game.clone();
    act(&mut two, Action::DeclineEffectPayment);
    act(&mut two, Action::Discard { card: land });
    assert_eq!(two.object_view(land).zone, Zone::Hand);
    act(&mut two, Action::Discard { card: instant });
    assert_eq!(two.object_view(land).zone, Zone::Graveyard);
    assert_eq!(two.object_view(instant).zone, Zone::Graveyard);
    act(&mut game, Action::Discard { card: flyer });
    assert_eq!(game.object_view(flyer).zone, Zone::Graveyard);
    assert_eq!(game.object_view(instant).zone, Zone::Hand);
    let keywords = game.object_view(scholar).keywords;
    assert!(keywords.contains(Keywords::FLYING) && keywords.contains(Keywords::VIGILANCE));
}

#[test]
fn sudden_setback_moves_an_uncounterable_spell_to_its_owners_chosen_position() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let cold = put(&mut game, opponent, "Out Cold", Zone::Hand);
    let setback = put(&mut game, player, "Sudden Setback", Zone::Hand);
    cast(&mut game, cold, None);
    act(&mut game, Action::Pass);
    cast(&mut game, setback, Some(Target::Spell(cold)));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let object = game.object_ref(cold);
    act(
        &mut game,
        Action::ChooseLibraryDestination {
            object,
            destination: LibraryDestination::Bottom,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(cold).zone, Zone::Library);
    assert_eq!(game.state.players[opponent.index()].library[0], cold);
    assert!(game.state.stack.is_empty());
    assert!(game.controlled_token_ids(opponent).next().is_none());
}
