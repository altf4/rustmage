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
        assert_eq!(game.hand_count(player), if flashback { 4 } else { 1 });
        assert_eq!(
            game.object_view(extra).zone,
            if flashback {
                Zone::Graveyard
            } else {
                Zone::Hand
            }
        );
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
    assert!(game.legal_actions().iter().any(|action| {
        matches!(action, Action::Cast { card, .. } | Action::AnnounceCast { card } if *card == spell)
    }));
    game.begin_step(Step::End);
    game.give_priority();
    assert!(!game.legal_actions().iter().any(|action| {
        matches!(action, Action::Cast { card, .. } | Action::AnnounceCast { card } if *card == spell)
    }));
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
    let land = put(&mut game, player, "Island", Zone::Hand);
    let artifact = put(&mut game, player, "Short Bow", Zone::Hand);
    let poison = put(&mut game, player, "Poison the Waters", Zone::Hand);
    cast_mode(&mut game, poison, 1, &[Target::Player(player)]);
    passes(&mut game);
    let land = game.object_ref(land);
    reject(&mut game, Action::ChooseRevealedDiscard { object: land });
    let object = game.object_ref(artifact);
    act(&mut game, Action::ChooseRevealedDiscard { object });
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
