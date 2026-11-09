mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn choose_targets(game: &mut Game, targets: &[Target]) {
    for &target in targets {
        act(game, Action::ChooseTarget { target });
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn cast_mode(game: &mut Game, card: CardId, mode: u16, targets: &[Target]) {
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseMode { mode });
    choose_targets(game, targets);
    pay(game);
}

fn activate_mode(game: &mut Game, card: CardId, mode: u16, target: Target) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(game, Action::ChooseMode { mode });
    choose_targets(game, &[target]);
    pay(game);
}

#[test]
fn nibelheim_uses_the_creatures_lifelink_and_graveyard_cast_is_the_only_wheel() {
    let player = Player::P0;
    for flashback in [false, true] {
        let mut game = scenario(player);
        let caster = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let extra = put(&mut game, player, "Island", Zone::Hand);
        let other_extra = put(&mut game, player, "Short Bow", Zone::Hand);
        let spell = put(
            &mut game,
            player,
            "Nibelheim Aflame",
            if flashback {
                Zone::Graveyard
            } else {
                Zone::Hand
            },
        );
        let target = Target::Card(caster);
        if flashback {
            act(&mut game, Action::AnnounceFlashback { card: spell });
            choose_targets(&mut game, &[target]);
            pay(&mut game);
        } else {
            cast(&mut game, spell, Some(target));
        }
        settle(&mut game);
        assert_eq!(game.object_view(caster).damage, 0);
        for id in [own, foe] {
            assert_eq!(game.object_view(id).damage, 1);
        }
        assert_eq!(game.state.players[player.index()].life, 22);
        assert_eq!(game.hand_count(player), if flashback { 4 } else { 2 });
        for id in [extra, other_extra] {
            assert_eq!(
                game.object_view(id).zone,
                if flashback {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                }
            );
        }
        assert_eq!(
            game.object_view(spell).zone,
            if flashback {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn noctis_makes_an_exact_additional_life_route_and_finality_applies_to_noncreatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let noctis = put(
        &mut game,
        player,
        "Noctis, Prince of Lucis",
        Zone::Battlefield,
    );
    let artifact = put(&mut game, player, "Short Bow", Zone::Graveyard);
    let rival = put(&mut game, player.other(), "Short Bow", Zone::Graveyard);
    let route = |game: &Game, card| Action::AnnounceGraveyardWithLife {
        object: game.object_ref(card),
        face: CastFace::Normal,
        method: CastMethod::Normal,
        flash_surcharge: false,
        life: 3,
        entry_counter: CounterKind::Finality,
    };
    assert!(game.keywords(noctis).contains(Keywords::LIFELINK));
    let forbidden = route(&game, rival);
    reject(&mut game, forbidden);
    let action = route(&game, artifact);
    act(&mut game, action);
    pay(&mut game);
    let clause = game
        .observe(player)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Life(3) && !row.paid)
        .unwrap() as u16;
    act(&mut game, Action::ChooseCastCost { clause });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 17);
    assert_eq!(game.counter_count(artifact, CounterKind::Finality), 1);
    game.move_object(artifact, Zone::Graveyard);
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
}

#[test]
fn opera_grants_normal_land_play_and_the_other_card_expires_at_its_controllers_end_step() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Island", Zone::Library);
    let spell = put(&mut game, player, "Opt", Zone::Library);
    let song = put(&mut game, player, "Opera Love Song", Zone::Hand);
    cast_mode(&mut game, song, 0, &[]);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    let announce = Action::AnnounceExile { card: spell };
    let mut cast_branch = game.clone();
    act(&mut cast_branch, announce.clone());
    pay(&mut cast_branch);
    assert_eq!(cast_branch.object_view(spell).zone, Zone::Stack);
    game.begin_step(Step::End);
    game.give_priority();
    reject(&mut game, announce);
}

#[test]
fn phantom_train_sacrifices_another_permanent_and_its_counter_outlasts_animation() {
    let player = Player::P0;
    let mut game = scenario(player);
    let train = put(&mut game, player, "Phantom Train", Zone::Battlefield);
    let sacrifice = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let stale = game.object_ref(train);
    let other = game.object_ref(sacrifice);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: train,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(&mut game, Action::ChooseAdditionalCost { object: stale });
    act(&mut game, Action::ChooseAdditionalCost { object: other });
    if game.state.announcing.is_some() {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(game.object_view(sacrifice).zone, Zone::Graveyard);
    assert!(game.has_subtype(train, "Spirit"));
    assert_eq!(stats(&game, train), (5, 5));
    assert!(game.keywords(train).contains(Keywords::TRAMPLE));
    game.finish_cleanup();
    assert!(!game.is_creature(train));
    assert!(!game.has_subtype(train, "Spirit"));
    assert_eq!(game.counter_count(train, CounterKind::PlusOne), 1);
}

#[test]
fn phoenix_down_has_one_paid_modal_activation_with_distinct_targets() {
    let player = Player::P0;
    for mode in [0, 1] {
        let mut game = scenario(player);
        let relic = put(&mut game, player, "Phoenix Down", Zone::Battlefield);
        let own = put(&mut game, player, "Hill Giant", Zone::Graveyard);
        let zombie = put(
            &mut game,
            player.other(),
            "Diregraf Ghoul",
            Zone::Battlefield,
        );
        let target = if mode == 0 { own } else { zombie };
        activate_mode(&mut game, relic, mode, Target::Card(target));
        assert_eq!(game.object_view(relic).zone, Zone::Exile);
        settle(&mut game);
        if mode == 0 {
            assert_eq!(game.object_view(own).zone, Zone::Battlefield);
            assert!(game.object_view(own).tapped);
            assert_eq!(game.object_view(zombie).zone, Zone::Battlefield);
        } else {
            assert_eq!(game.object_view(zombie).zone, Zone::Exile);
            assert_eq!(game.object_view(own).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn poison_shrinks_each_side_or_reveals_a_hand_for_a_filtered_discard() {
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    let poison = put(&mut game, player, "Poison the Waters", Zone::Hand);
    cast_mode(&mut game, poison, 0, &[]);
    settle(&mut game);
    for id in [own, enemy] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    let opponent = player.other();
    let land = put(&mut game, opponent, "Island", Zone::Hand);
    let artifact = put(&mut game, opponent, "Short Bow", Zone::Hand);
    let poison = put(&mut game, player, "Poison the Waters", Zone::Hand);
    cast_mode(&mut game, poison, 1, &[Target::Player(opponent)]);
    assert!(game.observe(player).known_hand.is_empty());
    passes(&mut game);
    let land = game.object_ref(land);
    let object = game.object_ref(artifact);
    assert!(game.observe(player).known_hand.contains(&land));
    assert!(game.observe(player).known_hand.contains(&object));
    reject(&mut game, Action::ChooseHandCard { object: Some(land) });
    act(
        &mut game,
        Action::ChooseHandCard {
            object: Some(object),
        },
    );
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land.card).zone, Zone::Hand);
}

#[test]
fn prishe_targets_only_after_a_search_and_can_trigger_after_failing_to_find() {
    let player = Player::P0;
    for found in [false, true] {
        let mut game = scenario(player);
        let recipient = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let town = put(&mut game, player, "Rabanastre, Royal City", Zone::Library);
        let spell = put(&mut game, player, "Prishe's Wanderings", Zone::Hand);
        cast(&mut game, spell, None);
        assert!(game.state.stack[0].targets.is_empty());
        passes(&mut game);
        if found {
            act(&mut game, Action::ChooseSearch { card: town });
        }
        if game.legal_actions().contains(&Action::FinishSearch) {
            act(&mut game, Action::FinishSearch);
        }
        if found {
            assert_eq!(game.object_view(town).zone, Zone::Battlefield);
            assert!(game.object_view(town).tapped);
        } else {
            assert_eq!(game.object_view(town).zone, Zone::Library);
        }
        choose_targets(&mut game, &[Target::Card(recipient)]);
        assert!(matches!(
            game.state.stack[0].kind,
            StackKind::TriggeredAbility { .. }
        ));
        settle(&mut game);
        assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 1);
    }
}

fn reach_action(game: &mut Game, action: &Action) {
    for _ in 0..12 {
        if game.legal_actions().contains(action) {
            return;
        }
        choose(game, |candidate| {
            matches!(candidate, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("action {action:?} was not reached");
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(game, Action::Pass);
    }
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn battlefield_tokens(game: &Game, card: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.is_token() && object.card == card)
                .then_some(id as CardId)
        })
        .collect()
}

fn target_trigger(game: &mut Game, target: Target) {
    reach_action(game, &Action::ChooseTarget { target });
    choose_targets(game, &[target]);
}

#[test]
fn schemes_returns_the_exact_public_owned_flashback_card_and_makes_red_mana() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Sorceress's Schemes", Zone::Graveyard);
    let exiled = put(&mut game, player, "Resentful Revelation", Zone::Exile);
    let hidden = put(&mut game, player, "Resentful Revelation", Zone::Exile);
    game.remember_face_down_exile(game.object_ref(hidden), 1 << player.index(), None);
    let foreign = put(
        &mut game,
        player.other(),
        "Resentful Revelation",
        Zone::Exile,
    );
    let plain = put(&mut game, player, "Shock", Zone::Exile);
    act(&mut game, Action::AnnounceFlashback { card: spell });
    for card in [hidden, foreign, plain] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    choose_targets(&mut game, &[Target::Card(exiled)]);
    pay(&mut game);
    let red = game.state.players[player.index()].mana[3];
    let mut stale = game.clone();
    stale.move_object(exiled, Zone::Graveyard);
    stale.move_object(exiled, Zone::Exile);
    settle(&mut stale);
    assert_eq!(stale.object_view(exiled).zone, Zone::Exile);
    assert_eq!(stale.state.players[player.index()].mana[3], red);
    settle(&mut game);
    assert_eq!(game.object_view(exiled).zone, Zone::Hand);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].mana[3], red + 1);
}

#[test]
fn squall_returns_two_small_permanents_via_the_two_damage_steps_when_attacking_alone() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let squall = put(
        &mut game,
        player,
        "Squall, SeeD Mercenary",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Rabanastre, Royal City", Zone::Graveyard);
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    let oversized = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let instant = put(&mut game, player, "Shock", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Forest", Zone::Graveyard);
    attack(&mut game, &[squall]);
    settle(&mut game);
    assert!(game.keywords(squall).contains(Keywords::DOUBLE_STRIKE));
    passes(&mut game);
    act(&mut game, Action::FinishBlockers);
    passes(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 17);
    for card in [oversized, instant, foreign] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    target_trigger(&mut game, Target::Card(land));
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
    passes(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 14);
    target_trigger(&mut game, Target::Card(artifact));
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
}

#[test]
fn stiltzkin_draws_only_after_the_targeted_opponent_actually_gets_the_other_permanent() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let moogle = put(
        &mut game,
        player,
        "Stiltzkin, Moogle Merchant",
        Zone::Battlefield,
    );
    let gift = put(&mut game, player, "Forest", Zone::Battlefield);
    assert!(game.keywords(moogle).contains(Keywords::LIFELINK));
    act(
        &mut game,
        Action::AnnounceActivate {
            card: moogle,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(moogle),
        },
    );
    choose_targets(&mut game, &[Target::Card(gift)]);
    pay(&mut game);
    assert!(game.object_view(moogle).tapped);
    let mut departed = game.clone();
    departed.move_object(gift, Zone::Exile);
    settle(&mut departed);
    assert_eq!(departed.hand_count(player), 0);
    settle(&mut game);
    assert_eq!(game.object_view(gift).controller, opponent);
    assert_eq!(game.object_view(gift).owner, player);
    assert_eq!(game.hand_count(player), 1);
    game.finish_cleanup();
    assert_eq!(game.object_view(gift).controller, opponent);
}

#[test]
fn stolen_uniform_steals_even_without_the_recipient_and_unattaches_in_cleanup_on_control_loss() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let equipment = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
    let spell = put(&mut game, player, "Stolen Uniform", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    for id in [creature, equipment] {
        let action = Action::ChooseTarget {
            target: Target::Card(id),
        };
        if !game.legal_actions().contains(&action) {
            act(&mut game, Action::FinishTargets);
        }
        act(&mut game, action);
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    let mut gone = game.clone();
    gone.move_object(creature, Zone::Exile);
    settle(&mut gone);
    assert_eq!(gone.object_view(equipment).controller, player);
    assert_eq!(gone.object_view(equipment).attached_to, None);
    settle(&mut game);
    let attached = Some(game.object_ref(creature));
    assert_eq!(game.object_view(equipment).controller, player);
    assert_eq!(game.object_view(equipment).attached_to, attached);
    game.state.step = Step::End;
    passes(&mut game);
    assert_eq!(game.state.step, Step::Cleanup);
    assert_eq!(game.object_view(equipment).controller, opponent);
    assert_eq!(game.object_view(equipment).attached_to, attached);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.object_view(equipment).attached_to, None);
}

#[test]
fn sanctum_has_flash_and_stops_even_the_enchanted_artifacts_mana_ability() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let artifact = put(&mut game, opponent, "Hedron Archive", Zone::Battlefield);
    let aura = put(&mut game, player, "Stuck in Summoner's Sanctum", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, aura, Some(Target::Card(artifact)));
    settle(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(artifact))
    );
    assert!(game.object_view(artifact).tapped);
    game.untap_active_player();
    assert!(game.object_view(artifact).tapped);
    game.state.objects[artifact as usize].tapped = false;
    game.give_priority();
    let mana = Action::ActivateMana {
        card: artifact,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [0, 0, 0, 0, 0, 2],
    };
    reject(&mut game, mana.clone());
    let draw = Action::AnnounceActivate {
        card: artifact,
        ability: 1,
        counter_payment: Counters::default(),
    };
    reject(&mut game, draw.clone());
    game.move_object(aura, Zone::Graveyard);
    assert!(game.legal_actions().contains(&draw));
    act(&mut game, mana);
    assert!(game.object_view(artifact).tapped);
}

fn place_next_lore(game: &mut Game) {
    game.state.step = Step::Draw;
    passes(game);
    assert_eq!(game.state.step, Step::Main1);
}

#[test]
fn bahamut_destroys_up_to_one_nonland_then_draws_and_deals_its_other_mana_value_total() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let victim = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
    let land = put(&mut game, opponent, "Island", Zone::Battlefield);
    put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    put(&mut game, player, "Forest", Zone::Battlefield);
    let bahamut = put(&mut game, player, "Summon: Bahamut", Zone::Hand);
    cast(&mut game, bahamut, None);
    target_trigger(&mut game, Target::Card(victim));
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(bahamut, CounterKind::Lore), 1);
    assert!(game.keywords(bahamut).contains(Keywords::FLYING));

    place_next_lore(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    act(&mut game, Action::FinishTargets);
    settle(&mut game);
    place_next_lore(&mut game);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 2);
    let reference = game.object_ref(bahamut);
    game.grant_keyword_effects(&[bahamut], reference, Keywords::LIFELINK);
    place_next_lore(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    game.move_object(bahamut, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 14);
    assert_eq!(game.state.players[player.index()].life, 26);
}

#[test]
fn brynhildr_allows_the_lore_turn_play_and_grants_haste_only_to_the_next_creature() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let land = put(&mut game, player, "Island", Zone::Library);
    let saga = put(&mut game, player, "Summon: Brynhildr", Zone::Hand);
    cast(&mut game, saga, None);
    settle(&mut game);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 1);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);

    place_next_lore(&mut game);
    settle(&mut game);
    assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);
    game.state.players[player.index()].mana = [8; 6];
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);
    place_next_lore(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    assert_eq!(game.observe(player).delayed_spell_copies.len(), 2);
    game.state.players[player.index()].mana = [8; 6];
    let first = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, first, None);
    assert_eq!(game.state.ordering_triggers.len(), 2);
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    settle(&mut game);
    assert!(game.keywords(first).contains(Keywords::HASTE));
    let second = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, second, None);
    settle(&mut game);
    assert!(!game.keywords(second).contains(Keywords::HASTE));
    game.finish_cleanup();
    assert!(!game.keywords(first).contains(Keywords::HASTE));
}

#[test]
fn card_collection_discards_two_then_transforms_only_if_eight_cards_remain() {
    let player = Player::P0;
    let mut game = scenario(player);
    for _ in 0..6 {
        put(&mut game, player, "Forest", Zone::Graveyard);
    }
    let crew = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, player, "Island", Zone::Hand);
    let second = put(&mut game, player, "Mountain", Zone::Hand);
    let quest = put(
        &mut game,
        player,
        "Sidequest: Card Collection // Magicked Card",
        Zone::Hand,
    );
    cast(&mut game, quest, None);
    let discard = Action::Discard { card: first };
    reach_action(&mut game, &discard);
    assert_eq!(game.hand_count(player), 5);
    act(&mut game, discard);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: second });
    assert_eq!(game.hand_count(player), 3);
    for id in [first, second] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    game.state.step = Step::Main2;
    passes(&mut game);
    assert_eq!(game.state.step, Step::End);
    assert_eq!(game.state.stack.len(), 1);
    let mut fewer = game.clone();
    fewer.move_object(first, Zone::Exile);
    settle(&mut fewer);
    assert_eq!(
        fewer.object_view(quest).name(),
        "Sidequest: Card Collection"
    );
    settle(&mut game);
    assert_eq!(game.object_view(quest).name(), "Magicked Card");
    assert!(!game.is_creature(quest));
    act(
        &mut game,
        Action::AnnounceActivate {
            card: quest,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = game.object_ref(crew);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    assert!(game.object_view(crew).tapped);
    assert_eq!(stats(&game, quest), (4, 4));
    assert!(game.keywords(quest).contains(Keywords::FLYING));
}

#[test]
fn hunt_the_mark_counts_the_treasure_it_creates_after_an_opponent_controlled_death() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let treasure = Card(5302);
    let cost = spawn(&mut game, player, treasure, Zone::Battlefield);
    spawn(&mut game, player, treasure, Zone::Battlefield);
    let prey = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.record_control(game.object_ref(prey), opponent, None);
    let quest = put(
        &mut game,
        player,
        "Sidequest: Hunt the Mark // Yiazmat, Ultimate Mark",
        Zone::Hand,
    );
    cast(&mut game, quest, None);
    target_trigger(&mut game, Target::Card(prey));
    settle(&mut game);
    assert_eq!(game.object_view(prey).zone, Zone::Graveyard);
    assert_eq!(game.object_view(prey).owner, player);
    game.state.step = Step::Main2;
    passes(&mut game);
    assert_eq!(game.state.step, Step::End);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(battlefield_tokens(&game, treasure).len(), 3);
    assert_eq!(game.object_view(quest).name(), "Yiazmat, Ultimate Mark");

    game.state.players[player.index()].mana = [8; 6];
    let activation = Action::AnnounceActivate {
        card: quest,
        ability: 0,
        counter_payment: Counters::default(),
    };
    let mut tapped = game.clone();
    tapped.state.objects[quest as usize].tapped = true;
    assert!(tapped.legal_actions().contains(&activation));
    act(&mut game, activation);
    let source = game.object_ref(quest);
    reject(&mut game, Action::ChooseAdditionalCost { object: source });
    let object = game.object_ref(cost);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    assert!(!game.object_view(quest).tapped);
    settle(&mut game);
    assert!(game.object_view(quest).tapped);
    assert!(game.keywords(quest).contains(Keywords::INDESTRUCTIBLE));
    game.finish_cleanup();
    assert!(!game.keywords(quest).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn play_blitzball_counts_actual_combat_damage_then_attaches_without_targeting() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let front = "Sidequest: Play Blitzball // World Champion, Celestial Weapon";
    let quest = put(&mut game, player, front, Zone::Battlefield);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let recipient = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let opponent_creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let reference = game.object_ref(quest);
    let attached = game.object_ref(recipient);
    game.grant_keyword_effects(&[recipient], attached, Keywords::HEXPROOF);
    passes(&mut game);
    target_trigger(&mut game, Target::Card(attacker));
    settle(&mut game);
    assert_eq!(stats(&game, attacker), (4, 2));

    let mut short = game.clone();
    for (branch, attackers, damage) in [
        (&mut short, vec![attacker], 4),
        (&mut game, vec![attacker, second], 6),
    ] {
        attack(branch, &attackers);
        passes(branch);
        act(branch, Action::FinishBlockers);
        passes(branch);
        assert_eq!(branch.state.players[opponent.index()].life, 20 - damage);
        passes(branch);
        assert_eq!(branch.state.step, Step::EndCombat);
    }
    assert!(short.state.stack.is_empty());
    assert_eq!(short.object_view(quest).name(), "Sidequest: Play Blitzball");
    assert!(game.state.stack[0].targets.is_empty());
    passes(&mut game);
    assert_eq!(game.object_ref(quest), reference);
    assert_eq!(
        game.object_view(quest).name(),
        "World Champion, Celestial Weapon"
    );
    let illegal = game.object_ref(opponent_creature);
    reject(
        &mut game,
        Action::ChooseEquipmentRecipient { recipient: illegal },
    );
    act(
        &mut game,
        Action::ChooseEquipmentRecipient {
            recipient: attached,
        },
    );
    assert_eq!(game.object_view(quest).attached_to, Some(attached));
    assert_eq!(stats(&game, recipient), (5, 3));
    assert!(game.keywords(recipient).contains(Keywords::DOUBLE_STRIKE));
}

#[test]
fn raise_a_chocobo_transforms_with_four_birds_and_the_fetched_land_buffs_the_group() {
    let player = Player::P0;
    let mut game = scenario(player);
    let front = "Sidequest: Raise a Chocobo // Black Chocobo";
    let quest = put(&mut game, player, front, Zone::Hand);
    cast(&mut game, quest, None);
    settle(&mut game);
    let birds = battlefield_tokens(&game, Card(5269));
    assert_eq!(birds.len(), 1);
    let token = birds[0];
    assert_eq!(stats(&game, token), (2, 2));
    let mut other_bird = token;
    for _ in 0..3 {
        other_bird = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    }
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Rabanastre, Royal City", Zone::Library);
    game.begin_step(Step::Main1);
    game.give_priority();
    let mut fewer = game.clone();
    fewer.move_object(other_bird, Zone::Exile);
    settle(&mut fewer);
    assert_eq!(
        fewer.object_view(quest).name(),
        "Sidequest: Raise a Chocobo"
    );

    let search = Action::ChooseSearch { card: land };
    reach_action(&mut game, &search);
    assert_eq!(game.object_view(quest).name(), "Black Chocobo");
    act(&mut game, search);
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    assert!(game.object_view(land).tapped);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert_eq!(stats(&game, quest), (3, 2));
    assert_eq!(stats(&game, token), (4, 2));
    assert_eq!(stats(&game, other_bird), (2, 1));
    assert_eq!(stats(&game, bear), (2, 2));
    assert_eq!(stats(&game, enemy), (1, 1));
}

#[test]
fn sin_repeats_for_each_land_in_its_own_graveyard_and_replays_the_random_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let lands = [
        put(&mut game, player, "Forest", Zone::Graveyard),
        put(&mut game, player, "Island", Zone::Graveyard),
    ];
    let spell = put(&mut game, player, "Opt", Zone::Graveyard);
    let enemy = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    let sin = put(&mut game, player, "Sin, Spira's Punishment", Zone::Hand);
    cast(&mut game, sin, None);
    passes(&mut game);
    assert!(game.keywords(sin).contains(Keywords::FLYING));
    act(&mut game, Action::Pass);
    let revision = game.revision();
    let saved = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let mut replay = game.clone();
    let actor = game.actor().unwrap();
    act(&mut game, Action::Pass);
    replay.apply_index(actor, revision, index).unwrap();
    assert_eq!(game.snapshot(), replay.snapshot());
    replay.restore(&saved).unwrap();
    act(&mut replay, Action::Pass);
    assert_eq!(game.snapshot(), replay.snapshot());
    for land in lands {
        let card = game.object_view(land).card;
        assert_eq!(game.object_view(land).zone, Zone::Exile);
        let copies = battlefield_tokens(&game, card);
        assert_eq!(copies.len(), 1);
        assert!(game.object_view(copies[0]).tapped);
    }
    for id in [spell, enemy] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
}

#[test]
fn sleep_magic_freezes_the_untap_step_and_is_sacrificed_only_after_actual_damage() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let other = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let aura = put(&mut game, player, "Sleep Magic", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(creature))
    );
    assert!(game.object_view(creature).tapped);
    let mut next = game.clone();
    next.state.active = opponent;
    next.untap_active_player();
    assert!(next.object_view(creature).tapped);

    game.add_counter(creature, CounterKind::Shield, 1);
    for target in [other, creature, creature] {
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(target)));
        settle(&mut game);
        let damage = game.object_view(creature).damage;
        assert_eq!(
            game.object_view(aura).zone,
            if damage == 0 {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
    }
    assert_eq!(game.object_view(creature).damage, 2);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
}

#[test]
fn sandworm_gives_the_destroyed_lands_controller_the_optional_basic_search() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let land = put(
        &mut game,
        opponent,
        "Rabanastre, Royal City",
        Zone::Battlefield,
    );
    let basic = put(&mut game, opponent, "Island", Zone::Library);
    let nonbasic = put(&mut game, opponent, "Rabanastre, Royal City", Zone::Library);
    let worm = put(&mut game, player, "Sandworm", Zone::Hand);
    cast(&mut game, worm, None);
    target_trigger(&mut game, Target::Card(land));
    passes(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert!(game.keywords(worm).contains(Keywords::HASTE));
    assert_eq!(game.actor(), Some(opponent));
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(basic).zone, Zone::Library);
    act(&mut game, Action::ChooseOptional { accept: true });
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    act(&mut game, Action::ChooseSearch { card: basic });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert_eq!(game.object_view(basic).controller, opponent);
    assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
    assert!(game.object_view(basic).tapped);
}

#[test]
fn sazh_reveals_a_bird_then_doubles_the_targets_counters_after_adding_one() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bird = put(&mut game, player, "Sazh's Chocobo", Zone::Library);
    let nonbasic = put(&mut game, player, "Rabanastre, Royal City", Zone::Library);
    let source = put(&mut game, player, "Sazh Katzroy", Zone::Hand);
    cast(&mut game, source, None);
    let accept = Action::ChooseOptional { accept: true };
    reach_action(&mut game, &accept);
    act(&mut game, accept);
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    act(&mut game, Action::ChooseSearch { card: bird });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert_eq!(game.object_view(bird).zone, Zone::Hand);
    assert!(
        game.observe(player.other())
            .known_hand
            .contains(&game.object_ref(bird))
    );
    game.state.objects[source as usize].summoning_sick = false;
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.set_counters(opposing, Counters::of(CounterKind::PlusOne, 2));
    attack(&mut game, &[source]);
    target_trigger(&mut game, Target::Card(opposing));
    settle(&mut game);
    assert_eq!(game.counter_count(opposing, CounterKind::PlusOne), 6);
}

#[test]
fn seifer_grants_double_strike_and_offers_a_small_graveyard_spell_after_combat_damage() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let seifer = put(&mut game, player, "Seifer Almasy", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Graveyard);
    let too_large = put(&mut game, player, "Nibelheim Aflame", Zone::Graveyard);
    let mut other_attacks = game.clone();
    attack(&mut other_attacks, &[bear]);
    settle(&mut other_attacks);
    assert!(
        other_attacks
            .keywords(bear)
            .contains(Keywords::DOUBLE_STRIKE)
    );
    assert!(
        !other_attacks
            .keywords(seifer)
            .contains(Keywords::DOUBLE_STRIKE)
    );
    attack(&mut game, &[seifer]);
    settle(&mut game);
    assert!(game.keywords(seifer).contains(Keywords::DOUBLE_STRIKE));
    passes(&mut game);
    act(&mut game, Action::FinishBlockers);
    let target = Action::ChooseTarget {
        target: Target::Card(shock),
    };
    reach_action(&mut game, &target);
    assert_eq!(game.state.players[opponent.index()].life, 17);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(too_large),
        },
    );
    choose_targets(&mut game, &[Target::Card(shock)]);
    passes(&mut game);
    let object = game.object_ref(shock);
    let mut declined = game.clone();
    act(&mut declined, Action::FinishResolutionCasts);
    assert_eq!(declined.object_view(shock).zone, Zone::Graveyard);
    game.state.players[player.index()].mana = [0; 6];
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    choose_targets(&mut game, &[Target::Player(opponent)]);
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
    assert_eq!(game.object_view(shock).zone, Zone::Exile);
    assert_eq!(game.state.players[opponent.index()].life, 15);
}

#[test]
fn self_destruct_deals_its_creatures_damage_and_survives_loss_of_the_other_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let other = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Self-Destruct", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    choose_targets(&mut game, &[Target::Card(other)]);
    pay(&mut game);
    let mut stale_other = game.clone();
    stale_other.move_object(other, Zone::Exile);
    settle(&mut stale_other);
    assert_eq!(stale_other.object_view(source).zone, Zone::Graveyard);
    assert_eq!(stale_other.state.players[player.index()].life, 21);
    let mut stale_source = game.clone();
    stale_source.record_control(game.object_ref(source), player.other(), None);
    settle(&mut stale_source);
    assert_eq!(stale_source.object_view(source).zone, Zone::Battlefield);
    assert_eq!(stale_source.object_view(other).damage, 0);
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.object_view(other).damage, 1);
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn sephiroth_creates_the_emblem_on_the_fourth_drain_and_draws_for_the_back_faces_sacrifices() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let front = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";
    let back = "Sephiroth, One-Winged Angel";
    let initial = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let source = put(&mut game, player, front, Zone::Hand);
    cast(&mut game, source, None);
    let sacrifice = Action::Sacrifice {
        object: game.object_ref(initial),
    };
    reach_action(&mut game, &sacrifice);
    act(&mut game, sacrifice);
    target_trigger(&mut game, Target::Player(opponent));
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
    let reference = game.object_ref(source);
    let emblem_count = |game: &Game| {
        game.observe(player)
            .objects
            .into_iter()
            .filter(|object| object.zone == Zone::Command && object.card == Card(5535))
            .count()
    };
    for count in 2..=4 {
        let bear = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(bear)));
        target_trigger(&mut game, Target::Player(opponent));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 20 + count);
        assert_eq!(game.state.players[opponent.index()].life, 20 - count);
        assert_eq!(emblem_count(&game), usize::from(count == 4));
    }
    assert_eq!(game.object_ref(source), reference);
    assert_eq!(game.object_view(source).name(), back);
    assert!(game.keywords(source).contains(Keywords::FLYING));
    game.state.objects[source as usize].summoning_sick = false;
    let fodder = [
        put(&mut game, player, "Bear Cub", Zone::Battlefield),
        put(&mut game, player, "Bear Cub", Zone::Battlefield),
    ];
    attack(&mut game, &[source]);
    let finish = Action::FinishEffectChoice;
    reach_action(&mut game, &finish);
    let mut zero = game.clone();
    act(&mut zero, finish.clone());
    assert_eq!(zero.hand_count(player), 1);
    reject(&mut game, Action::Sacrifice { object: reference });
    for id in fodder {
        let object = game.object_ref(id);
        act(&mut game, Action::Sacrifice { object });
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
    }
    act(&mut game, finish);
    assert_eq!(game.hand_count(player), 3);
    for id in fodder {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    for _ in 0..2 {
        target_trigger(&mut game, Target::Player(opponent));
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 26);
    assert_eq!(game.state.players[opponent.index()].life, 14);
}

#[test]
fn serah_checks_other_legends_on_resolution_and_preserves_the_first_cast_history_across_faces() {
    let player = Player::P0;
    let mut game = scenario(player);
    let front = "Serah Farron // Crystallized Serah";
    let source = put(&mut game, player, front, Zone::Battlefield);
    let sazh = put(&mut game, player, "Sazh Katzroy", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, player, "Yargle, Glutton of Urborg", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: first });
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 2);
    pay(&mut game);
    settle(&mut game);
    passes(&mut game);
    assert_eq!(game.state.step, Step::BeginCombat);
    let mut no_longer_qualified = game.clone();
    no_longer_qualified.move_object(sazh, Zone::Exile);
    settle(&mut no_longer_qualified);
    assert!(no_longer_qualified.is_creature(source));
    let accept = Action::ChooseOptional { accept: true };
    reach_action(&mut game, &accept);
    act(&mut game, accept);
    assert_eq!(game.object_view(source).name(), "Crystallized Serah");
    assert!(!game.is_creature(source));
    assert_eq!(stats(&game, first), (11, 5));
    assert_eq!(stats(&game, bear), (2, 2));
    game.begin_step(Step::Main2);
    game.give_priority();
    game.state.players[player.index()].mana = [8; 6];
    let second = put(&mut game, player, "Seifer Almasy", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: second });
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 3);
}

#[test]
fn seymour_draws_and_grows_only_after_its_controllers_upkeep_life_payment() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Seymour Flux", Zone::Battlefield);
    game.begin_step(Step::Upkeep);
    game.give_priority();
    let accept = Action::ChooseOptional { accept: true };
    reach_action(&mut game, &accept);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.hand_count(player), 0);
    assert_eq!(declined.state.players[player.index()].life, 20);
    assert_eq!(declined.counter_count(source, CounterKind::PlusOne), 0);
    act(&mut game, accept);
    assert_eq!(game.hand_count(player), 1);
    assert_eq!(game.state.players[player.index()].life, 19);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
}

#[test]
fn shambling_cieth_only_returns_from_the_exact_graveyard_after_a_paid_noncreature_cast_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Shambling Cie'th", Zone::Graveyard);
    let bear = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, bear, None);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    assert_eq!(game.state.stack.len(), 2);
    let mut stale = game.clone();
    stale.move_object(source, Zone::Exile);
    stale.move_object(source, Zone::Graveyard);
    passes(&mut game);
    passes(&mut stale);
    let payment = Action::PayResolutionMana {
        payment: [0, 0, 1, 0, 0, 0],
    };
    act(&mut stale, payment.clone());
    act(&mut game, payment);
    assert_eq!(stale.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.object_view(source).zone, Zone::Hand);
    settle(&mut game);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.object_view(source).tapped);
}

#[test]
fn omega_counts_its_nonbasic_lands_even_when_no_permanent_is_targeted() {
    let player = Player::P0;
    let opponent = player.other();
    for target in [false, true] {
        let mut game = scenario(player);
        let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let land = put(
            &mut game,
            opponent,
            "Rabanastre, Royal City",
            Zone::Battlefield,
        );
        for name in ["Rabanastre, Royal City", "Rabanastre, Royal City", "Forest"] {
            put(&mut game, player, name, Zone::Battlefield);
        }
        let omega = put(&mut game, player, "Omega, Heartless Evolution", Zone::Hand);
        cast(&mut game, omega, None);
        let finish = Action::FinishTargets;
        reach_action(&mut game, &finish);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        if target {
            choose_targets(&mut game, &[Target::Card(enemy)]);
        } else {
            act(&mut game, finish);
        }
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 22);
        assert_eq!(game.object_view(enemy).tapped, target);
        assert_eq!(
            game.counter_count(enemy, CounterKind::Stun),
            if target { 2 } else { 0 }
        );
    }
}

#[test]
fn relentless_needs_three_blockers_and_returns_tapped_with_finality() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spider = put(&mut game, player, "Relentless X-ATM092", Zone::Graveyard);
    choose(
        &mut game,
        |action| matches!(action, Action::Activate { card, ability: 0, .. } if *card == spider),
    );
    settle(&mut game);
    assert_eq!(game.object_view(spider).zone, Zone::Battlefield);
    assert!(game.object_view(spider).tapped);
    assert_eq!(game.counter_count(spider, CounterKind::Finality), 1);
    let mut dies = game.clone();
    dies.move_object(spider, Zone::Graveyard);
    assert_eq!(dies.object_view(spider).zone, Zone::Exile);

    game.state.objects[spider as usize].tapped = false;
    game.state.objects[spider as usize].summoning_sick = false;
    let blockers = [
        put(&mut game, player.other(), "Bear Cub", Zone::Battlefield),
        put(&mut game, player.other(), "Bear Cub", Zone::Battlefield),
        put(&mut game, player.other(), "Bear Cub", Zone::Battlefield),
    ];
    let mut two = game.clone();
    two.move_object(blockers[2], Zone::Graveyard);
    for branch in [&mut two, &mut game] {
        attack(branch, &[spider]);
        passes(branch);
        assert_eq!(branch.state.pending, Pending::Blockers);
    }
    reject(
        &mut two,
        Action::Block {
            blocker: blockers[0],
            attacker: spider,
        },
    );
    for (index, blocker) in blockers.into_iter().enumerate() {
        act(
            &mut game,
            Action::Block {
                blocker,
                attacker: spider,
            },
        );
        assert_eq!(
            game.legal_actions().contains(&Action::FinishBlockers),
            index == 2
        );
    }
}

#[test]
fn relm_copies_copiable_values_beneath_animation_and_counters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ride = put(&mut game, player, "Ride the Shoopuf", Zone::Battlefield);
    choose(
        &mut game,
        |action| matches!(action, Action::Activate { card, ability: 0, .. } if *card == ride),
    );
    settle(&mut game);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    let target = Action::ChooseTarget {
        target: Target::Card(ride),
    };
    reach_action(&mut game, &target);
    choose_targets(&mut game, &[Target::Card(ride)]);
    settle(&mut game);
    assert_eq!(stats(&game, ride), (8, 8));
    let relm = put(&mut game, player, "Relm's Sketching", Zone::Hand);
    cast(&mut game, relm, Some(Target::Card(ride)));
    settle(&mut game);
    let copies = battlefield_tokens(&game, Card::from_name("Ride the Shoopuf").unwrap());
    assert_eq!(copies.len(), 1);
    assert!(!game.is_creature(copies[0]));
    assert_eq!(game.counter_count(copies[0], CounterKind::PlusOne), 0);
    assert!(game.has_subtype(ride, "Beast"));
    game.finish_cleanup();
    assert_eq!(stats(&game, ride), (8, 8));
    game.move_object(ride, Zone::Exile);
    game.move_object(ride, Zone::Battlefield);
    assert!(!game.is_creature(ride));
}

#[test]
fn reno_exiles_before_payment_and_uses_any_mana_for_the_paid_card() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let reno = put(&mut game, player, "Reno and Rude", Zone::Battlefield);
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let spell = put(&mut game, opponent, "Shock", Zone::Library);
    assert!(game.keywords(reno).contains(Keywords::MENACE));
    attack(&mut game, &[reno]);
    passes(&mut game);
    act(&mut game, Action::FinishBlockers);
    let pay_artifact = Action::Sacrifice {
        object: game.object_ref(artifact),
    };
    reach_action(&mut game, &pay_artifact);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(declined.object_view(artifact).zone, Zone::Battlefield);
    reject(&mut declined, Action::AnnounceExile { card: spell });
    let source = game.object_ref(reno);
    reject(&mut game, Action::Sacrifice { object: source });
    act(&mut game, pay_artifact);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut game, Action::AnnounceExile { card: spell });
    choose_targets(&mut game, &[Target::Player(opponent)]);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 16);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn retrieve_the_esper_only_puts_counters_on_tokens_when_cast_from_a_graveyard() {
    let player = Player::P0;
    for flashback in [false, true] {
        let mut game = scenario(player);
        let spell = put(
            &mut game,
            player,
            "Retrieve the Esper",
            if flashback {
                Zone::Graveyard
            } else {
                Zone::Hand
            },
        );
        if flashback {
            act(&mut game, Action::AnnounceFlashback { card: spell });
            pay(&mut game);
        } else {
            cast(&mut game, spell, None);
        }
        settle(&mut game);
        let robots = battlefield_tokens(&game, Card(5532));
        assert_eq!(robots.len(), 1);
        let robot = robots[0];
        assert!(game.has_subtype(robot, "Robot"));
        assert!(game.has_subtype(robot, "Warrior"));
        assert_eq!(game.object_view(robot).kinds & 17, 17);
        assert_eq!(
            game.counter_count(robot, CounterKind::PlusOne),
            if flashback { 2 } else { 0 }
        );
        assert_eq!(stats(&game, robot), if flashback { (5, 5) } else { (3, 3) });
        assert_eq!(
            game.object_view(spell).zone,
            if flashback {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn rinoa_creates_angelo_and_counts_the_creatures_left_when_her_trigger_resolves() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rinoa = put(&mut game, player, "Rinoa Heartilly", Zone::Hand);
    cast(&mut game, rinoa, None);
    settle(&mut game);
    let dogs = battlefield_tokens(&game, Card(5533));
    assert_eq!(dogs.len(), 1);
    let angelo = dogs[0];
    assert!(game.has_subtype(angelo, "Dog"));
    assert_eq!(stats(&game, angelo), (1, 1));
    game.state.objects[rinoa as usize].summoning_sick = false;
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    attack(&mut game, &[rinoa]);
    let target = Action::ChooseTarget {
        target: Target::Card(angelo),
    };
    reach_action(&mut game, &target);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(rinoa),
        },
    );
    choose_targets(&mut game, &[Target::Card(angelo)]);
    let mut intact = game.clone();
    game.move_object(other, Zone::Graveyard);
    settle(&mut game);
    settle(&mut intact);
    assert_eq!(stats(&game, angelo), (3, 3));
    assert_eq!(stats(&intact, angelo), (4, 4));
    game.finish_cleanup();
    assert_eq!(stats(&game, angelo), (1, 1));
}

#[test]
fn rook_turret_only_offers_draw_then_discard_when_drawing_is_legal() {
    let player = Player::P0;
    let mut game = scenario(player);
    let turret = put(&mut game, player, "Rook Turret", Zone::Battlefield);
    assert!(game.keywords(turret).contains(Keywords::FLYING));
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Hand);
    let original = put(&mut game, player, "Island", Zone::Hand);
    cast(&mut game, artifact, None);
    let mut prohibited = game.clone();
    put(
        &mut prohibited,
        player.other(),
        "Mornsong Aria",
        Zone::Battlefield,
    );
    settle(&mut prohibited);
    assert_eq!(prohibited.hand_count(player), 1);
    assert_eq!(prohibited.object_view(original).zone, Zone::Hand);
    let accept = Action::ChooseOptional { accept: true };
    reach_action(&mut game, &accept);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.hand_count(player), 1);
    act(&mut game, accept);
    assert_eq!(game.hand_count(player), 2);
    act(&mut game, Action::Discard { card: original });
    assert_eq!(game.object_view(original).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn rufus_checks_the_exact_controlled_darkstar_name_twice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rufus = put(&mut game, player, "Rufus Shinra", Zone::Battlefield);
    let darkstar = Card(5534);
    let opposing = spawn(&mut game, player.other(), darkstar, Zone::Battlefield);
    let mut already_controlled = game.clone();
    already_controlled.record_control(game.object_ref(opposing), player, None);
    attack(&mut already_controlled, &[rufus]);
    assert!(already_controlled.state.stack.is_empty());
    attack(&mut game, &[rufus]);
    let mut gained = game.clone();
    gained.record_control(game.object_ref(opposing), player, None);
    settle(&mut gained);
    assert_eq!(battlefield_tokens(&gained, darkstar), [opposing]);
    settle(&mut game);
    let dogs = battlefield_tokens(&game, darkstar);
    assert_eq!(dogs.len(), 2);
    let created = dogs.into_iter().find(|&dog| dog != opposing).unwrap();
    assert_eq!(stats(&game, created), (2, 2));
    assert_eq!(game.object_view(created).controller, player);
}

#[test]
fn rydia_discards_to_draw_and_returns_only_a_saga_matching_her_paid_x() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rydia = put(
        &mut game,
        player,
        "Rydia, Summoner of Mist",
        Zone::Battlefield,
    );
    let choco = put(&mut game, player, "Summon: Choco/Mog", Zone::Graveyard);
    let fat_choco = put(&mut game, player, "Summon: Fat Chocobo", Zone::Graveyard);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let spare = put(&mut game, player, "Island", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    let decline = Action::DeclineEffectPayment;
    reach_action(&mut game, &decline);
    let mut unchanged = game.clone();
    act(&mut unchanged, decline);
    assert_eq!(unchanged.object_view(spare).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: spare });
    assert_eq!(game.object_view(spare).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(player), 1);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: rydia,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(&mut game, Action::ChooseX { value: 4 });
    act(&mut game, Action::ChooseX { value: 5 });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(choco),
        },
    );
    choose_targets(&mut game, &[Target::Card(fat_choco)]);
    pay(&mut game);
    assert!(game.object_view(rydia).tapped);
    settle(&mut game);
    assert_eq!(game.object_view(fat_choco).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(fat_choco, CounterKind::Finality), 1);
    assert!(game.keywords(fat_choco).contains(Keywords::HASTE));
    game.finish_cleanup();
    assert!(!game.keywords(fat_choco).contains(Keywords::HASTE));
}

#[test]
fn qiqirn_pays_only_for_its_own_towns_on_the_sacrifice_activation() {
    let player = Player::P0;
    let mut game = scenario(player);
    let merchant = put(&mut game, player, "Qiqirn Merchant", Zone::Battlefield);
    for controller in [player, player, player.other()] {
        put(
            &mut game,
            controller,
            "Rabanastre, Royal City",
            Zone::Battlefield,
        );
    }
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 5];
    choose(
        &mut game,
        |action| matches!(action, Action::Activate { card, ability: 1, .. } if *card == merchant),
    );
    assert_eq!(game.object_view(merchant).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 3);
}

#[test]
fn queen_creates_the_printed_wizard_then_both_watch_the_next_noncreature_cast() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let queen = put(&mut game, player, "Queen Brahne", Zone::Battlefield);
    let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    attack(&mut game, &[queen]);
    settle(&mut game);
    let wizard = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| object.zone == Zone::Battlefield && object.card == Card(5531))
        .unwrap();
    assert_eq!((wizard.power, wizard.toughness, wizard.colors), (0, 1, 4));
    assert!(game.has_subtype(wizard.id, "Wizard"));
    assert!(!game.state.attackers.iter().any(|a| a.card == wizard.id));
    act(&mut game, Action::TapMana { card: mountain });
    cast(&mut game, shock, Some(Target::Player(opponent)));
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    passes(&mut game);
    passes(&mut game);
    assert_eq!(stats(&game, queen), (3, 2));
    assert_eq!(game.state.players[opponent.index()].life, 19);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 17);
}

#[test]
fn quistis_casts_an_exact_foreign_graveyard_spell_with_colorless_payment() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let shock = put(&mut game, opponent, "Shock", Zone::Graveyard);
    let quistis = put(&mut game, player, "Quistis Trepe", Zone::Hand);
    cast(&mut game, quistis, None);
    let target = Action::ChooseTarget {
        target: Target::Card(shock),
    };
    reach_action(&mut game, &target);
    act(&mut game, target);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    passes(&mut game);
    let object = game.object_ref(shock);
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    choose_targets(&mut game, &[Target::Player(opponent)]);
    pay(&mut game);
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Exile);
    assert_eq!(game.object_view(shock).owner, opponent);
    assert_eq!(game.state.players[opponent.index()].life, 18);
}

#[test]
fn qutrubs_second_mode_binds_the_graveyard_of_the_first_chosen_card() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Forest", Zone::Graveyard);
    let second = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let rival = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    let qutrub = put(&mut game, player, "Qutrub Forayer", Zone::Hand);
    cast(&mut game, qutrub, None);
    let mode = Action::ChooseMode { mode: 1 };
    reach_action(&mut game, &mode);
    act(&mut game, mode);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(rival),
        },
    );
    choose_targets(&mut game, &[Target::Card(second)]);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Exile);
    assert_eq!(game.object_view(second).zone, Zone::Exile);
    assert_eq!(game.object_view(rival).zone, Zone::Graveyard);
}

#[test]
fn ragnarok_destroys_a_permanent_and_recovers_a_nonlegendary_permanent_card() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ragnarok = put(
        &mut game,
        player,
        "Ragnarok, Divine Deliverance",
        Zone::Battlefield,
    );
    let destroy = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let recover = put(&mut game, player, "Short Bow", Zone::Graveyard);
    assert_eq!(stats(&game, ragnarok), (7, 6));
    game.move_object(ragnarok, Zone::Graveyard);
    game.prepare_triggers();
    choose_targets(&mut game, &[Target::Card(destroy), Target::Card(recover)]);
    settle(&mut game);
    assert_eq!(game.object_view(destroy).zone, Zone::Graveyard);
    assert_eq!(game.object_view(recover).zone, Zone::Battlefield);
}

#[test]
fn random_encounter_returns_only_its_actual_hasty_entrants_at_the_next_end_step() {
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].library.clear();
    let first = put(&mut game, player, "Bear Cub", Zone::Library);
    let second = put(&mut game, player, "Hill Giant", Zone::Library);
    let land = put(&mut game, player, "Mountain", Zone::Library);
    let old = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let encounter = put(&mut game, player, "Random Encounter", Zone::Hand);
    cast(&mut game, encounter, None);
    settle(&mut game);
    for id in [first, second] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
        assert!(game.keywords(id).contains(Keywords::HASTE));
    }
    for id in [land, old] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    game.move_object(second, Zone::Exile);
    game.move_object(second, Zone::Battlefield);
    game.begin_step(Step::End);
    game.give_priority();
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
}

#[test]
fn raubahn_uses_the_generic_ward_payment_and_can_equip_another_attacker() {
    let player = Player::P0;
    let mut ward = scenario(player.other());
    let source = put(
        &mut ward,
        player,
        "Raubahn, Bull of Ala Mhigo",
        Zone::Battlefield,
    );
    let shock = put(&mut ward, player.other(), "Shock", Zone::Hand);
    cast(&mut ward, shock, Some(Target::Card(source)));
    ward.add_counter(source, CounterKind::PlusOne, 2);
    passes(&mut ward);
    assert_eq!(
        ward.observe(player.other())
            .resolution_payment
            .unwrap()
            .life,
        Some(4)
    );
    act(&mut ward, Action::PayResolutionLife);
    settle(&mut ward);
    assert_eq!(ward.state.players[player.other().index()].life, 16);
    assert_eq!(ward.object_view(source).damage, 2);

    let mut combat = scenario(player);
    let source = put(
        &mut combat,
        player,
        "Raubahn, Bull of Ala Mhigo",
        Zone::Battlefield,
    );
    let ally = put(&mut combat, player, "Bear Cub", Zone::Battlefield);
    let equipment = put(&mut combat, player, "Short Bow", Zone::Battlefield);
    attack(&mut combat, &[source, ally]);
    choose_targets(&mut combat, &[Target::Card(equipment), Target::Card(ally)]);
    settle(&mut combat);
    assert_eq!(
        combat.object_view(equipment).attached_to,
        Some(combat.object_ref(ally))
    );
}

#[test]
fn reach_the_horizon_selects_mixed_towns_and_basics_with_distinct_names() {
    let player = Player::P0;
    let mut game = scenario(player);
    let a = put(&mut game, player, "Forest", Zone::Library);
    let duplicate = put(&mut game, player, "Forest", Zone::Library);
    let town = put(&mut game, player, "Rabanastre, Royal City", Zone::Library);
    let spell = put(&mut game, player, "Reach the Horizon", Zone::Hand);
    cast(&mut game, spell, None);
    passes(&mut game);
    act(&mut game, Action::ChooseSearch { card: a });
    reject(&mut game, Action::ChooseSearch { card: duplicate });
    act(&mut game, Action::ChooseSearch { card: town });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    for id in [a, town] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
        assert!(game.object_view(id).tapped);
    }
}
