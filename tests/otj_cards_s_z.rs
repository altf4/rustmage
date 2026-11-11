//! Rules scenarios for the S–Z Outlaws cards, including inherited definitions.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn activate(game: &mut Game, card: CardId, chosen: Option<CardId>) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if let Some(chosen) = chosen {
        act(game, target(chosen));
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn mode(game: &mut Game, card: CardId, mode: u16, targets: &[Target]) {
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseMode { mode });
    for &target in targets {
        act(game, Action::ChooseTarget { target });
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
}

fn start_attack(game: &mut Game, card: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

fn hit_player(game: &mut Game, sources: &[CardId], recipient: Player) {
    let events: Vec<_> = sources
        .iter()
        .map(|&card| DamageEvent {
            preventable: true,
            source: game.damage_source(card),
            target: Target::Player(recipient),
            amount: game.object_view(card).power.max(1),
            combat: true,
        })
        .collect();
    game.apply_damage_batch(&events);
    game.prepare_triggers();
}

fn token_objects(game: &Game, player: Player, subtype: &str) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.has_subtype(subtype)
        })
        .collect()
}

fn advance_turn(game: &mut Game) {
    let previous = game.state.turn;
    game.begin_step(Step::End);
    for _ in 0..8 {
        if game.state.turn > previous {
            game.begin_step(Step::Main1);
            return;
        }
        act(game, Action::Pass);
    }
    panic!("turn did not advance");
}

fn plot(game: &mut Game, card: CardId, payment: Mana) {
    act(game, Action::AnnouncePlot { card });
    act(game, Action::PayPlot { payment });
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    reject(game, Action::AnnouncePlotted { card });
}

#[test]
fn satoru_ignores_paid_and_token_entries_and_draws_once_for_a_free_group() {
    let player = Player::P0;
    let mut game = scenario(player);
    let satoru = put(&mut game, player, "Satoru, the Infiltrator", Zone::Hand);
    cast(&mut game, satoru, None);
    settle(&mut game);
    assert!(game.keywords(satoru).contains(Keywords::MENACE));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    let wanted = put(&mut game, player, "Wanted Griffin", Zone::Hand);
    cast(&mut game, wanted, None);
    settle(&mut game);
    assert!(game.keywords(wanted).contains(Keywords::FLYING));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    let kill = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, kill, Some(Target::Card(wanted)));
    settle(&mut game);
    assert_eq!(token_objects(&game, player, "Mercenary").len(), 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    let surprise = put(&mut game, player, "Smuggler's Surprise", Zone::Hand);
    let first = put(&mut game, player, "Llanowar Elves", Zone::Hand);
    let second = put(&mut game, player, "Savannah Lions", Zone::Hand);
    game.state.players[player.index()].mana = [8; 6];
    mode(&mut game, surprise, 1, &[]);
    pass_pair(&mut game);
    for card in [first, second] {
        let object = game.object_ref(card);
        act(&mut game, Action::PutHandCreature { object });
    }
    if game.legal_actions().contains(&Action::FinishEffectChoice) {
        act(&mut game, Action::FinishEffectChoice);
    }
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn scalestorm_checks_ferocious_when_the_attack_ability_resolves() {
    let player = Player::P1;
    let mut game = scenario(player);
    let summoner = put(&mut game, player, "Scalestorm Summoner", Zone::Battlefield);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    start_attack(&mut game, summoner);
    assert_eq!(game.state.stack.len(), 1);
    let mut unpumped = game.clone();
    settle(&mut unpumped);
    assert!(token_objects(&unpumped, player, "Dinosaur").is_empty());

    game.state.players[player.index()].mana = [8; 6];
    cast(&mut game, growth, Some(Target::Card(summoner)));
    settle(&mut game);
    let dinosaurs = token_objects(&game, player, "Dinosaur");
    assert_eq!(dinosaurs.len(), 1);
    assert_eq!(
        (
            dinosaurs[0].power,
            dinosaurs[0].toughness,
            dinosaurs[0].colors
        ),
        (3, 1, 8)
    );
}

#[test]
fn actual_crime_enables_seize_vault_buster_and_once_per_turn_vadmir() {
    let player = Player::P0;
    let mut game = scenario(player);
    let buster = put(
        &mut game,
        player,
        "Slickshot Vault-Buster",
        Zone::Battlefield,
    );
    let vadmir = put(&mut game, player, "Vadmir, New Blood", Zone::Battlefield);
    let seize = put(&mut game, player, "Seize the Secrets", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    game.add_counter(vadmir, CounterKind::PlusOne, 3);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    reject(&mut game, Action::AnnounceCast { card: seize });
    assert_eq!(stats(&game, buster), (1, 4));
    assert!(game.keywords(buster).contains(Keywords::VIGILANCE));

    game.state.players[player.index()].mana[3] = 1;
    cast(&mut game, shock, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(stats(&game, buster), (3, 4));
    assert_eq!(game.counter_count(vadmir, CounterKind::PlusOne), 4);
    assert!(
        game.keywords(vadmir)
            .contains(Keywords::MENACE.union(Keywords::LIFELINK))
    );
    cast(&mut game, seize, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);

    let second = put(&mut game, player, "Shock", Zone::Hand);
    game.state.players[player.index()].mana[3] = 1;
    cast(&mut game, second, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(game.counter_count(vadmir, CounterKind::PlusOne), 4);
    advance_turn(&mut game);
    assert_eq!(stats(&game, buster), (1, 4));
    game.set_counters(vadmir, Counters::of(CounterKind::PlusOne, 3));
    assert!(
        !game
            .keywords(vadmir)
            .intersects(Keywords::MENACE.union(Keywords::LIFELINK))
    );
}

#[test]
fn selvala_counts_distinct_signed_powers_and_creature_casts_make_a_real_mercenary() {
    let player = Player::P0;
    let mut game = scenario(player);
    let selvala = put(
        &mut game,
        player,
        "Selvala, Eager Trailblazer",
        Zone::Battlefield,
    );
    let negative = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    game.state.objects[negative as usize].growth.power = -2;
    let zero = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    game.state.objects[zero as usize].growth.power = -1;
    let duplicate = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.state.objects[duplicate as usize].growth.power = 1;
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    assert!(game.keywords(selvala).contains(Keywords::VIGILANCE));
    let before = game.state.players[player.index()].mana[4];
    choose(&mut game, |action| {
        matches!(action, Action::ActivateMana {
        card, production, ..
    } if *card == selvala && *production == [0, 0, 0, 0, 3, 0])
    });
    assert_eq!(game.state.players[player.index()].mana[4], before + 3);
    assert!(game.state.stack.is_empty());
    cast(&mut game, spell, Some(Target::Player(player.other())));
    settle(&mut game);
    assert!(token_objects(&game, player, "Mercenary").is_empty());
    cast(&mut game, creature, None);
    settle(&mut game);
    let tokens = token_objects(&game, player, "Mercenary");
    assert_eq!(tokens.len(), 1);
    let mercenary = &tokens[0];
    assert_eq!(
        (mercenary.power, mercenary.toughness, mercenary.colors),
        (1, 1, 8)
    );
    assert!(mercenary.summoning_sick);
    game.state.objects[mercenary.id as usize].summoning_sick = false;
    activate(&mut game, mercenary.id, Some(selvala));
    settle(&mut game);
    assert!(game.object_view(mercenary.id).tapped);
    assert_eq!(stats(&game, selvala), (5, 5));
}

#[test]
fn servant_requires_a_crime_and_the_actual_optional_self_sacrifice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let servant = put(
        &mut game,
        player,
        "Servant of the Stinger",
        Zone::Battlefield,
    );
    let other = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let find = put(&mut game, player, "Island", Zone::Library);
    assert!(game.keywords(servant).contains(Keywords::DEATHTOUCH));
    hit_player(&mut game, &[servant], player.other());
    assert!(game.state.stack.is_empty());
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    settle(&mut game);
    hit_player(&mut game, &[servant], player.other());
    pass_pair(&mut game);
    let outsider = game.object_ref(other);
    reject(&mut game, Action::Sacrifice { object: outsider });
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert_eq!(decline.object_view(servant).zone, Zone::Battlefield);
    assert_eq!(decline.object_view(find).zone, Zone::Library);
    let object = game.object_ref(servant);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(servant).zone, Zone::Graveyard);
    act(&mut game, Action::ChooseSearch { card: find });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert_eq!(game.object_view(find).zone, Zone::Hand);
    assert!(
        !game
            .observe(player.other())
            .known_hand
            .iter()
            .any(|row| row.card == find)
    );
}

#[test]
fn shackle_slinger_observes_only_the_second_cast_and_samples_the_target_live() {
    let player = Player::P0;
    let mut game = scenario(player);
    let slinger = put(&mut game, player, "Shackle Slinger", Zone::Hand);
    let opposing_elf = put(
        &mut game,
        player.other(),
        "Llanowar Elves",
        Zone::Battlefield,
    );
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, slinger, None);
    settle(&mut game);
    cast(&mut game, growth, Some(Target::Card(slinger)));
    reject(&mut game, target(slinger));
    act(&mut game, target(opposing_elf));
    let mut untapped = game.clone();
    settle(&mut untapped);
    assert!(untapped.object_view(opposing_elf).tapped);
    assert_eq!(untapped.counter_count(opposing_elf, CounterKind::Stun), 0);

    act(&mut game, Action::Pass);
    act(&mut game, Action::TapMana { card: opposing_elf });
    settle(&mut game);
    assert_eq!(game.counter_count(opposing_elf, CounterKind::Stun), 1);
    let third = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, third, Some(Target::Card(slinger)));
    assert_eq!(game.state.stack.len(), 1);
}

#[test]
fn shepherd_returns_a_small_permanent_to_hand_or_battlefield_with_a_mount() {
    let player = Player::P1;
    for mount in [false, true] {
        let mut game = scenario(player);
        let shepherd = put(&mut game, player, "Shepherd of the Clouds", Zone::Hand);
        let land = put(&mut game, player, "Forest", Zone::Graveyard);
        let instant = put(&mut game, player, "Shock", Zone::Graveyard);
        let expensive = put(&mut game, player, "Hill Giant", Zone::Graveyard);
        let foe = put(&mut game, player.other(), "Forest", Zone::Graveyard);
        if mount {
            put(&mut game, player, "Trained Arynx", Zone::Battlefield);
        }
        cast(&mut game, shepherd, None);
        pass_pair(&mut game);
        for illegal in [instant, expensive, foe] {
            reject(&mut game, target(illegal));
        }
        act(&mut game, target(land));
        settle(&mut game);
        assert!(
            game.keywords(shepherd)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        assert_eq!(
            game.object_view(land).zone,
            if mount { Zone::Battlefield } else { Zone::Hand }
        );
    }
}

#[test]
fn plotted_sheriff_enters_with_one_counter_plus_each_other_controlled_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sheriff = put(&mut game, player, "Sheriff of Safe Passage", Zone::Hand);
    let mut alone = game.clone();
    cast(&mut alone, sheriff, None);
    settle(&mut alone);
    assert_eq!(stats(&alone, sheriff), (1, 1));

    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    plot(&mut game, sheriff, [1, 0, 0, 0, 0, 1]);
    advance_turn(&mut game);
    advance_turn(&mut game);
    act(&mut game, Action::AnnouncePlotted { card: sheriff });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(game.counter_count(sheriff, CounterKind::PlusOne), 3);
    assert_eq!(stats(&game, sheriff), (3, 3));
}

#[test]
fn shifting_grift_resolves_each_paid_exchange_as_a_separate_pair() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Shifting Grift", Zone::Hand);
    let ours = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let theirs = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let our_artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let their_artifact = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
    let our_enchantment = put(&mut game, player, "Omniscience", Zone::Battlefield);
    let their_enchantment = put(&mut game, opponent, "Omniscience", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 4];
    let chosen = [
        ours,
        theirs,
        our_artifact,
        their_artifact,
        our_enchantment,
        their_enchantment,
    ];
    mode(&mut game, spell, 6, &chosen.map(Target::Card));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    game.move_object(theirs, Zone::Hand);
    settle(&mut game);
    assert_eq!(game.object_view(ours).controller, player);
    for card in [our_artifact, our_enchantment] {
        assert_eq!(game.object_view(card).controller, opponent);
    }
    for card in [their_artifact, their_enchantment] {
        assert_eq!(game.object_view(card).controller, player);
    }
}

#[test]
fn silver_deputy_may_reveal_a_desert_and_pumps_only_a_controlled_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let deputy = put(&mut game, player, "Silver Deputy", Zone::Hand);
    let desert = put(&mut game, player, "Sandstorm Verge", Zone::Library);
    let ordinary = put(&mut game, player, "Spirebluff Canal", Zone::Library);
    let foe = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, deputy, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    assert_eq!(
        decline.state.players[player.index()].library.last(),
        Some(&ordinary)
    );
    act(&mut game, Action::ChooseOptional { accept: true });
    reject(&mut game, Action::ChooseSearch { card: ordinary });
    act(&mut game, Action::ChooseSearch { card: desert });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&desert)
    );
    let activate_deputy = Action::AnnounceActivate {
        card: deputy,
        ability: 0,
        counter_payment: Counters::default(),
    };
    reject(&mut game, activate_deputy.clone());
    game.state.objects[deputy as usize].summoning_sick = false;
    act(&mut game, activate_deputy);
    reject(&mut game, target(foe));
    act(&mut game, target(own));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(stats(&game, own), (3, 2));
}

#[test]
fn skulduggery_needs_both_targets_but_keeps_a_surviving_group() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Skulduggery", Zone::Hand);
    let ours = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    reject(&mut game, Action::AnnounceCast { card: spell });
    let theirs = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    announce(
        &mut game,
        spell,
        &[Target::Card(ours), Target::Card(theirs)],
    );
    let mut both = game.clone();
    settle(&mut both);
    assert_eq!(stats(&both, ours), (3, 3));
    assert_eq!(stats(&both, theirs), (1, 1));
    game.move_object(theirs, Zone::Hand);
    game.move_object(theirs, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(stats(&game, ours), (3, 3));
    assert_eq!(stats(&game, theirs), (2, 2));
}

#[test]
fn slick_sequence_and_thunder_salvo_count_other_completed_casts_at_resolution() {
    let player = Player::P1;
    let mut game = scenario(player);
    let sequence = put(&mut game, player, "Slick Sequence", Zone::Hand);
    let salvo = put(&mut game, player, "Thunder Salvo", Zone::Hand);
    let target_card = put(
        &mut game,
        player.other(),
        "Spinewoods Armadillo",
        Zone::Battlefield,
    );
    cast(&mut game, sequence, Some(Target::Player(player.other())));
    let mut only = game.clone();
    settle(&mut only);
    assert_eq!(only.state.players[player.index()].cards_drawn_this_turn, 0);
    assert_eq!(only.state.players[player.other().index()].life, 18);

    // Cast onto the still-unresolved Sequence, but target our own creature to avoid ward.
    game.record_control(game.object_ref(target_card), player, None);
    cast(&mut game, salvo, Some(Target::Card(target_card)));
    settle(&mut game);
    assert_eq!(game.object_view(target_card).damage, 3);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn lockpicker_grants_printed_cost_flashback_to_one_owned_instant_or_sorcery() {
    let player = Player::P0;
    let mut game = scenario(player);
    let picker = put(&mut game, player, "Slickshot Lockpicker", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Graveyard);
    let wrong = put(&mut game, player.other(), "Shock", Zone::Graveyard);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let mut plotted = game.clone();
    plot(&mut plotted, picker, [0, 1, 0, 0, 0, 2]);
    cast(&mut game, picker, None);
    pass_pair(&mut game);
    reject(&mut game, target(wrong));
    reject(&mut game, target(creature));
    act(&mut game, target(shock));
    settle(&mut game);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    act(
        &mut game,
        Action::AnnounceGrantedFlashback {
            card: shock,
            flash_surcharge: false,
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Exile);
    assert_eq!(game.state.players[player.other().index()].life, 18);
}

#[test]
fn smuggler_combines_mill_recovery_private_deployment_and_current_power_protection() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Smuggler's Surprise", Zone::Hand);
    let outside = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let low = put(&mut game, player, "Savannah Lions", Zone::Hand);
    let recovered_land = put(&mut game, player, "Forest", Zone::Library);
    let recovered_creature = put(&mut game, player, "Daring Thunder-Thief", Zone::Library);
    let unselected = put(&mut game, player, "Island", Zone::Library);
    let nonpermanent = put(&mut game, player, "Shock", Zone::Library);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 2, 7];
    mode(&mut game, spell, 6, &[]);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    pass_pair(&mut game);
    for illegal in [outside, nonpermanent] {
        let object = game.object_ref(illegal);
        reject(&mut game, Action::SelectMilled { object });
    }
    for card in [recovered_land, recovered_creature] {
        let object = game.object_ref(card);
        act(&mut game, Action::SelectMilled { object });
    }
    if game
        .legal_actions()
        .contains(&Action::FinishMilledSelection)
    {
        act(&mut game, Action::FinishMilledSelection);
    }
    assert!(game.observe(player).effect_choice.is_some());
    assert!(game.observe(player.other()).effect_choice.is_none());
    let object = game.object_ref(recovered_land);
    reject(&mut game, Action::PutHandCreature { object });
    for card in [recovered_creature, low] {
        let object = game.object_ref(card);
        act(&mut game, Action::PutHandCreature { object });
    }
    if game.legal_actions().contains(&Action::FinishEffectChoice) {
        act(&mut game, Action::FinishEffectChoice);
    }
    settle(&mut game);
    assert_eq!(game.object_view(unselected).zone, Zone::Graveyard);
    assert_eq!(game.object_view(recovered_land).zone, Zone::Hand);
    assert_eq!(game.object_view(low).zone, Zone::Battlefield);
    let protection = Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE);
    assert!(game.keywords(recovered_creature).contains(protection));
    assert!(!game.keywords(low).intersects(protection));
    advance_turn(&mut game);
    assert!(!game.keywords(recovered_creature).intersects(protection));
}

#[test]
fn spring_splasher_targets_a_creature_of_the_actual_defender() {
    let player = Player::P0;
    let mut game = scenario(player);
    let frog = put(&mut game, player, "Spring Splasher", Zone::Battlefield);
    let own = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let rival = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let walker = put(
        &mut game,
        player.other(),
        "Oko, the Ringleader",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 6);
    let defender = game.object_ref(walker);
    game.begin_step(Step::BeginCombat);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::AttackPlaneswalker {
            card: frog,
            defender,
        },
    );
    act(&mut game, Action::FinishAttackers);
    reject(&mut game, target(own));
    act(&mut game, target(rival));
    settle(&mut game);
    assert_eq!(stats(&game, rival), (0, 3));
    assert_eq!(stats(&game, own), (3, 3));
}

#[test]
fn steer_clear_uses_the_mount_at_cast_and_rejects_noncombat_creatures() {
    let player = Player::P0;
    for mounted in [false, true] {
        let opponent = player.other();
        let mut game = scenario(opponent);
        let attacker = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let noncombatant = put(&mut game, opponent, "Goblin Piker", Zone::Battlefield);
        let spell = put(&mut game, player, "Steer Clear", Zone::Hand);
        let mount = mounted.then(|| put(&mut game, player, "Trained Arynx", Zone::Battlefield));
        start_attack(&mut game, attacker);
        act(&mut game, Action::Pass);
        game.state.players[player.index()].mana[0] = 1;
        act(&mut game, Action::AnnounceCast { card: spell });
        reject(&mut game, target(noncombatant));
        act(&mut game, target(attacker));
        pay(&mut game);
        if let Some(mount) = mount {
            game.move_object(mount, Zone::Graveyard);
        }
        settle(&mut game);
        if mounted {
            assert_eq!(game.object_view(attacker).zone, Zone::Graveyard);
        } else {
            assert_eq!(game.object_view(attacker).damage, 2);
        }
    }
}

#[test]
fn step_between_worlds_respects_both_players_choices_and_exiles_itself() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Step Between Worlds", Zone::Hand);
    let our_hand = put(&mut game, player, "Island", Zone::Hand);
    let our_grave = put(&mut game, player, "Swamp", Zone::Graveyard);
    let their_hand = put(&mut game, opponent, "Island", Zone::Hand);
    let their_grave = put(&mut game, opponent, "Swamp", Zone::Graveyard);
    let mut plotted = game.clone();
    plot(&mut plotted, spell, [0, 2, 0, 0, 0, 4]);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.actor(), Some(opponent));
    assert_eq!(game.object_view(our_hand).zone, Zone::Hand);
    assert_eq!(game.object_view(our_grave).zone, Zone::Graveyard);
    let saved = game.snapshot();
    let choice = Action::ChooseOptional { accept: false };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choice)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(opponent, indexed.revision(), index)
        .unwrap();
    act(&mut game, choice.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, choice);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(game.object_view(their_hand).zone, Zone::Hand);
    assert_eq!(game.object_view(their_grave).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 7);
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        0
    );
}

#[test]
fn taii_uses_paid_x_and_draws_only_for_exact_actual_noncombat_damage() {
    let player = Player::P1;
    let mut game = scenario(player);
    let taii = put(
        &mut game,
        player,
        "Taii Wakeen, Perfect Shot",
        Zone::Battlefield,
    );
    let first = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(first)));
    settle(&mut game);
    assert_eq!(game.object_view(first).damage, 2);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: taii,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(&mut game, Action::ChooseX { value: 1 });
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(taii).tapped);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(second)));
    settle(&mut game);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 17);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn take_for_a_ride_needs_a_prior_crime_off_turn_and_returns_its_untapped_hasty_target() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let ride = put(&mut game, player, "Take for a Ride", Zone::Hand);
    let crime = put(&mut game, player, "Shock", Zone::Hand);
    let victim = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[victim as usize].tapped = true;
    act(&mut game, Action::Pass);
    reject(&mut game, Action::AnnounceCast { card: ride });
    cast(&mut game, crime, Some(Target::Player(opponent)));
    cast(&mut game, ride, Some(Target::Card(victim)));
    assert!(game.keywords(ride).contains(Keywords::FLASH));
    settle(&mut game);
    assert_eq!(game.object_view(victim).controller, player);
    assert!(!game.object_view(victim).tapped);
    assert!(game.keywords(victim).contains(Keywords::HASTE));
    advance_turn(&mut game);
    assert_eq!(game.object_view(victim).controller, opponent);
    assert!(!game.keywords(victim).contains(Keywords::HASTE));
}

#[test]
fn take_the_fall_counts_outlaws_at_resolution_and_draws_after_either_branch() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Take the Fall", Zone::Hand);
    let outlaw = put(&mut game, player, "Vault Plunderer", Zone::Battlefield);
    let rival = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, spell, Some(Target::Card(rival)));
    let mut without = game.clone();
    without.move_object(outlaw, Zone::Exile);
    settle(&mut without);
    assert_eq!(stats(&without, rival), (2, 3));
    settle(&mut game);
    assert_eq!(stats(&game, rival), (-1, 3));
    for state in [&game, &without] {
        assert_eq!(state.state.players[player.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn terror_taxes_opposing_spell_targets_and_uses_live_entrant_power() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let terror = put(&mut game, player, "Terror of the Peaks", Zone::Battlefield);
    let entrant = put(&mut game, player, "Savannah Lions", Zone::Hand);
    let pump = put(&mut game, player, "Giant Growth", Zone::Hand);
    assert!(game.keywords(terror).contains(Keywords::FLYING));
    cast(&mut game, entrant, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    cast(&mut game, pump, Some(Target::Card(entrant)));
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 15);

    let removal = put(&mut game, opponent, "Murder", Zone::Hand);
    act(&mut game, Action::Pass);
    let mut broke = game.clone();
    broke.state.players[opponent.index()].life = 2;
    act(&mut broke, Action::AnnounceCast { card: removal });
    reject(&mut broke, target(terror));
    act(&mut game, Action::AnnounceCast { card: removal });
    act(&mut game, target(terror));
    for _ in 0..8 {
        match game.state.pending {
            Pending::Priority => break,
            Pending::PayCosts { .. } => pay(&mut game),
            Pending::CastCosts { .. } => choose(&mut game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            ref pending => panic!("unexpected Terror payment choice: {pending:?}"),
        }
    }
    assert_eq!(game.state.players[opponent.index()].life, 12);
    settle(&mut game);
    assert_eq!(game.object_view(terror).zone, Zone::Graveyard);
}

fn saddle(game: &mut Game, mount: CardId, rider: CardId) {
    activate(game, mount, None);
    let object = game.object_ref(rider);
    act(game, Action::ChooseSaddler { object });
    act(game, Action::FinishSaddle);
    settle(game);
}

#[test]
fn gitrog_sacrifices_only_its_exact_rider_then_draws_and_puts_tapped_lands() {
    let player = Player::P1;
    let mut game = scenario(player);
    let gitrog = put(
        &mut game,
        player,
        "The Gitrog, Ravenous Ride",
        Zone::Battlefield,
    );
    let rider = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let outside = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let land = put(&mut game, player, "Swamp", Zone::Library);
    assert!(
        game.keywords(gitrog)
            .contains(Keywords::TRAMPLE.union(Keywords::HASTE))
    );
    saddle(&mut game, gitrog, rider);
    hit_player(&mut game, &[gitrog], player.other());
    pass_pair(&mut game);
    let object = game.object_ref(outside);
    reject(&mut game, Action::ChooseEffectObject { object });
    let object = game.object_ref(rider);
    act(&mut game, Action::ChooseEffectObject { object });
    assert_eq!(game.object_view(rider).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    let object = game.object_ref(land);
    act(&mut game, Action::SelectLand { object });
    act(&mut game, Action::FinishLandSelection);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
    assert!(!game.state.land_played);
}

#[test]
fn key_equips_and_looks_at_damage_count_then_casts_only_the_selected_nonland() {
    let player = Player::P1;
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "The Key to the Vault", Zone::Battlefield);
    let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let below = put(&mut game, player, "Savannah Lions", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let spell = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let top = put(&mut game, player, "Shock", Zone::Library);
    let before = game.state.players[player.index()].mana.iter().sum::<u32>();
    activate(&mut game, equipment, Some(attacker));
    settle(&mut game);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(attacker))
    );
    assert_eq!(
        game.state.players[player.index()].mana.iter().sum::<u32>(),
        before - 3
    );
    hit_player(&mut game, &[attacker], player.other());
    pass_pair(&mut game);
    assert!(game.observe(player.other()).library_selection.is_none());
    let look = game.observe(player).library_selection.unwrap();
    assert_eq!(
        look.remaining.iter().map(|row| row.id).collect::<Vec<_>>(),
        [top, spell, land]
    );
    reject(&mut game, Action::SelectLibrary { card: land });
    act(&mut game, Action::SelectLibrary { card: spell });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.top_library_card(player), Some(below));
    let object = game.object_ref(spell);
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(
        game.state.players[player.index()].mana.iter().sum::<u32>(),
        before - 3
    );
}

#[test]
fn this_town_is_banned_in_standard_and_discounted_only_for_a_chosen_own_permanent() {
    let player = Player::P0;
    let definition = Card::from_name("This Town Ain't Big Enough").unwrap();
    let mut deck = vec![Card::Forest; 59];
    deck.push(definition);
    let config = |deck_rules| Config {
        decks: [deck.clone(), vec![Card::Forest; 60]],
        seed: 20261111,
        starting_player: player,
        deck_rules,
    };
    assert!(Game::new(config(DeckRules::Standard)).is_err());
    assert!(Game::new(config(DeckRules::Freeform)).is_ok());

    let mut game = scenario(player);
    let town = put(&mut game, player, "This Town Ain't Big Enough", Zone::Hand);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player, "Island", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 4];
    let mut zero = game.clone();
    act(&mut zero, Action::AnnounceCast { card: town });
    act(&mut zero, Action::FinishTargets);
    pay(&mut zero);
    settle(&mut zero);
    assert_eq!(zero.state.players[player.index()].mana, [0; 6]);
    assert_eq!(zero.object_view(own).zone, Zone::Battlefield);

    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    act(&mut game, Action::AnnounceCast { card: town });
    reject(&mut game, Action::FinishTargets);
    reject(&mut game, target(land));
    act(&mut game, target(enemy));
    reject(&mut game, Action::FinishTargets);
    act(&mut game, target(own));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_view(own).zone, Zone::Hand);
    assert_eq!(game.object_view(enemy).zone, Zone::Hand);
    assert!(game.state.players[player.index()].committed_crime_this_turn);
}

#[test]
fn three_steps_ahead_counters_then_copies_then_loots_in_the_same_paid_cast() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let countered = put(&mut game, opponent, "Shock", Zone::Hand);
    let three = put(&mut game, player, "Three Steps Ahead", Zone::Hand);
    let copy = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let discard = put(&mut game, player, "Island", Zone::Hand);
    game.add_counter(copy, CounterKind::PlusOne, 1);
    cast(&mut game, countered, Some(Target::Player(player)));
    act(&mut game, Action::Pass);
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 6];
    mode(
        &mut game,
        three,
        6,
        &[Target::Spell(countered), Target::Card(copy)],
    );
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    pass_pair(&mut game);
    assert_eq!(game.object_view(countered).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 20);
    let tokens = token_objects(&game, player, "Bear");
    assert_eq!(tokens.len(), 1);
    assert_eq!((tokens[0].power, tokens[0].toughness), (2, 2));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    act(&mut game, Action::Discard { card: discard });
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
}

#[test]
fn throw_from_the_saddle_boosts_the_live_mount_then_uses_that_creature_as_dealer() {
    let player = Player::P0;
    for mount in [false, true] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Throw from the Saddle", Zone::Hand);
        let own = put(
            &mut game,
            player,
            if mount {
                "Trained Arynx"
            } else {
                "Grizzly Bears"
            },
            Zone::Battlefield,
        );
        let foe = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        game.state.objects[own as usize].gained_keywords = Keywords::LIFELINK;
        announce(&mut game, spell, &[Target::Card(own), Target::Card(foe)]);
        let mut partial = game.clone();
        partial.move_object(foe, Zone::Hand);
        settle(&mut partial);
        assert_eq!(stats(&partial, own).0, stats(&game, own).0 + 1);
        assert_eq!(partial.state.players[player.index()].life, 20);

        let expected_damage = stats(&game, own).0 + 1;
        settle(&mut game);
        assert_eq!(
            game.counter_count(own, CounterKind::PlusOne),
            u64::from(mount)
        );
        assert_eq!(game.object_view(foe).zone, Zone::Graveyard);
        assert_eq!(
            game.state.players[player.index()].life,
            20 + expected_damage
        );
        advance_turn(&mut game);
        assert_eq!(stats(&game, own).0, expected_damage - i128::from(!mount));
    }
}

#[test]
fn tinybones_joins_up_targets_any_number_and_waits_for_all_private_discards() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let join = put(&mut game, player, "Tinybones Joins Up", Zone::Hand);
    let ours = put(&mut game, player, "Forest", Zone::Hand);
    let theirs = put(&mut game, opponent, "Island", Zone::Hand);
    let legend = put(&mut game, player, "Wylie Duke, Atiin Hero", Zone::Hand);
    cast(&mut game, join, None);
    pass_pair(&mut game);
    let mut zero = game.clone();
    act(&mut zero, Action::FinishTargets);
    settle(&mut zero);
    assert_eq!(zero.object_view(ours).zone, Zone::Hand);
    assert_eq!(zero.object_view(theirs).zone, Zone::Hand);
    for target in [player, opponent] {
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(target),
            },
        );
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    assert!(game.observe(opponent).effect_choice.is_none());
    act(&mut game, Action::Discard { card: ours });
    assert_eq!(game.actor(), Some(opponent));
    assert_eq!(game.object_view(ours).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: theirs });
    assert_eq!(game.object_view(ours).zone, Zone::Graveyard);
    assert_eq!(game.object_view(theirs).zone, Zone::Graveyard);

    let top = game.top_library_card(opponent).unwrap();
    cast(&mut game, legend, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    act(&mut game, Action::FinishTargets);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    assert_eq!(game.state.players[player.index()].life, 20);
}

#[test]
fn pickpocket_casts_only_a_damaged_players_nonland_permanent_and_can_pay_off_color() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let tinybones = put(
        &mut game,
        player,
        "Tinybones, the Pickpocket",
        Zone::Battlefield,
    );
    let stolen = put(&mut game, opponent, "Llanowar Elves", Zone::Graveyard);
    let land = put(&mut game, opponent, "Forest", Zone::Graveyard);
    let instant = put(&mut game, opponent, "Shock", Zone::Graveyard);
    let own = put(&mut game, player, "Llanowar Elves", Zone::Graveyard);
    assert!(game.keywords(tinybones).contains(Keywords::DEATHTOUCH));
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    hit_player(&mut game, &[tinybones], opponent);
    for illegal in [land, instant, own] {
        reject(&mut game, target(illegal));
    }
    act(&mut game, target(stolen));
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::FinishResolutionCasts);
    assert_eq!(decline.object_view(stolen).zone, Zone::Graveyard);
    let object = game.object_ref(stolen);
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    pay(&mut game);
    if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    let stolen = game.object_view(stolen);
    assert_eq!(stolen.zone, Zone::Battlefield);
    assert_eq!(stolen.owner, opponent);
    assert_eq!(stolen.controller, player);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn trash_the_town_can_apply_all_three_occurrences_to_one_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Trash the Town", Zone::Hand);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 4];
    mode(&mut game, spell, 6, &[Target::Card(creature); 3]);
    settle(&mut game);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);
    assert!(game.keywords(creature).contains(Keywords::TRAMPLE));
    hit_player(&mut game, &[creature], player.other());
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    advance_turn(&mut game);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);
    assert!(!game.keywords(creature).contains(Keywords::TRAMPLE));
    hit_player(&mut game, &[creature], player.other());
    assert!(game.state.stack.is_empty());
}

#[test]
fn trick_shot_keeps_independent_damage_amounts_for_distinct_targets() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Trick Shot", Zone::Hand);
    let ordinary = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.create_tokens(player.other(), Card::from_catalogue_id(5570), 2, false);
    let tokens = token_objects(&game, player.other(), "Dinosaur");
    let (first, second) = (tokens[0].id, tokens[1].id);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, target(first));
    reject(&mut game, target(first));
    reject(&mut game, target(ordinary));
    act(&mut game, target(second));
    pay(&mut game);
    game.move_object(first, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(second).zone, Zone::Outside);
    assert_eq!(game.object_view(ordinary).zone, Zone::Battlefield);

    let shot = put(&mut game, player, "Trick Shot", Zone::Hand);
    let large = put(&mut game, player, "Spinewoods Armadillo", Zone::Battlefield);
    game.state.players[player.index()].mana = [8; 6];
    announce(&mut game, shot, &[Target::Card(large)]);
    settle(&mut game);
    assert_eq!(game.object_view(large).damage, 6);
}

#[test]
fn tumbleweed_rising_freezes_the_highest_current_power_as_copiable_token_size() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Tumbleweed Rising", Zone::Hand);
    let own = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Spinewoods Armadillo",
        Zone::Battlefield,
    );
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    let mut plotted = game.clone();
    plot(&mut plotted, spell, [0, 0, 0, 0, 1, 2]);
    cast(&mut game, spell, None);
    cast(&mut game, growth, Some(Target::Card(own)));
    settle(&mut game);
    let tokens = token_objects(&game, player, "Elemental");
    assert_eq!(tokens.len(), 1);
    assert_eq!(
        (tokens[0].power, tokens[0].toughness, tokens[0].colors),
        (6, 6, 16)
    );
    let token = tokens[0].id;
    game.move_object(own, Zone::Graveyard);
    assert_eq!(stats(&game, token), (6, 6));
    let copy = put(&mut game, player, "Three Steps Ahead", Zone::Hand);
    mode(&mut game, copy, 1, &[Target::Card(token)]);
    settle(&mut game);
    let copies = token_objects(&game, player, "Elemental");
    assert_eq!(copies.len(), 2);
    assert!(
        copies
            .iter()
            .all(|token| (token.power, token.toughness) == (6, 6))
    );
}

#[test]
fn unfortunate_accident_can_create_a_mercenary_without_a_target_but_all_targets_can_fizzle() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Unfortunate Accident", Zone::Hand);
    let foe = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let mut only_token = game.clone();
    mode(&mut only_token, spell, 1, &[]);
    settle(&mut only_token);
    assert_eq!(token_objects(&only_token, player, "Mercenary").len(), 1);
    assert_eq!(only_token.object_view(foe).zone, Zone::Battlefield);
    mode(&mut game, spell, 2, &[Target::Card(foe)]);
    let mut illegal = game.clone();
    illegal.move_object(foe, Zone::Hand);
    settle(&mut illegal);
    assert!(token_objects(&illegal, player, "Mercenary").is_empty());
    settle(&mut game);
    assert_eq!(game.object_view(foe).zone, Zone::Graveyard);
    assert_eq!(token_objects(&game, player, "Mercenary").len(), 1);
}

#[test]
fn contractor_only_targets_a_player_after_the_optional_sacrifice() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let contractor = put(&mut game, player, "Unscrupulous Contractor", Zone::Hand);
    let enemy_creature = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let mut plotted = game.clone();
    plot(&mut plotted, contractor, [0, 0, 1, 0, 0, 2]);
    cast(&mut game, contractor, None);
    pass_pair(&mut game);
    assert!(game.state.stack.last().unwrap().targets.is_empty());
    assert!(!game.state.players[player.index()].committed_crime_this_turn);
    pass_pair(&mut game);
    let object = game.object_ref(enemy_creature);
    reject(&mut game, Action::Sacrifice { object });
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert_eq!(decline.object_view(contractor).zone, Zone::Battlefield);
    let object = game.object_ref(contractor);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(contractor).zone, Zone::Graveyard);
    assert!(!game.state.players[player.index()].committed_crime_this_turn);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    assert!(game.state.players[player.index()].committed_crime_this_turn);
    settle(&mut game);
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        2
    );
    assert_eq!(game.state.players[opponent.index()].life, 18);
}

#[test]
fn vengeful_townsfolk_counts_one_simultaneous_other_creature_death_event() {
    let player = Player::P0;
    let mut game = scenario(player);
    let townsfolk = put(&mut game, player, "Vengeful Townsfolk", Zone::Battlefield);
    let first = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    let stolen = put(
        &mut game,
        player.other(),
        "Savannah Lions",
        Zone::Battlefield,
    );
    let enemy = put(
        &mut game,
        player.other(),
        "Savannah Lions",
        Zone::Battlefield,
    );
    game.record_control(game.object_ref(stolen), player, None);
    let events = [first, stolen, enemy].map(|card| DamageEvent {
        preventable: true,
        source: game.damage_source(townsfolk),
        target: Target::Card(card),
        amount: 1,
        combat: false,
    });
    game.apply_damage_batch(&events);
    game.state_based_actions();
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.counter_count(townsfolk, CounterKind::PlusOne), 1);
}

#[test]
fn visage_bandit_can_decline_or_copy_only_a_controlled_creature_with_added_types() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bandit = put(&mut game, player, "Visage Bandit", Zone::Hand);
    let unmodified_bandit = put(&mut game, player, "Visage Bandit", Zone::Battlefield);
    let donor = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(donor, CounterKind::PlusOne, 1);
    let mut plotted = game.clone();
    plot(&mut plotted, bandit, [0, 1, 0, 0, 0, 2]);
    cast(&mut game, bandit, None);
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseEntryCopy { object: None });
    assert_eq!(stats(&decline, bandit), (2, 2));
    let foe = game.object_ref(enemy);
    reject(&mut game, Action::ChooseEntryCopy { object: Some(foe) });
    let same = game.object_ref(unmodified_bandit);
    for _ in 0..3 {
        act(&mut game, Action::ChooseEntryCopy { object: Some(same) });
        assert!(matches!(game.state.pending, Pending::EntryCopy { .. }));
        assert_eq!(game.object_view(bandit).zone, Zone::Stack);
    }
    let our_donor = game.object_ref(donor);
    act(
        &mut game,
        Action::ChooseEntryCopy {
            object: Some(our_donor),
        },
    );
    let copied = game.object_view(bandit);
    assert_eq!((copied.power, copied.toughness), (1, 1));
    assert!(
        copied.has_subtype("Elf")
            && copied.has_subtype("Shapeshifter")
            && copied.has_subtype("Rogue")
    );
    assert_eq!(game.counter_count(bandit, CounterKind::PlusOne), 0);
}

#[test]
fn vraska_joins_up_counters_only_own_creatures_and_draws_once_per_legendary_dealer() {
    let player = Player::P0;
    let mut game = scenario(player);
    let enchantment = put(&mut game, player, "Vraska Joins Up", Zone::Hand);
    let first = put(&mut game, player, "Vadmir, New Blood", Zone::Battlefield);
    let second = put(
        &mut game,
        player,
        "Tinybones, the Pickpocket",
        Zone::Battlefield,
    );
    let common = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    cast(&mut game, enchantment, None);
    settle(&mut game);
    for creature in [first, second, common] {
        assert_eq!(game.counter_count(creature, CounterKind::Deathtouch), 1);
    }
    assert_eq!(game.counter_count(enemy, CounterKind::Deathtouch), 0);
    hit_player(&mut game, &[first, second, common], player.other());
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn vraska_returns_an_opposing_nontoken_as_tapped_treasure_with_its_own_abilities() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let vraska = put(&mut game, player, "Vraska, the Silencer", Zone::Battlefield);
    let elf = put(&mut game, opponent, "Llanowar Elves", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    assert!(game.keywords(vraska).contains(Keywords::DEATHTOUCH));
    cast(&mut game, shock, Some(Target::Card(elf)));
    pass_pair(&mut game);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    let treasure = game.object_view(elf);
    assert_eq!(treasure.zone, Zone::Battlefield);
    assert!(treasure.has_subtype("Treasure") && !treasure.has_subtype("Elf"));
    assert_eq!(treasure.kinds, 16);
    assert_eq!(treasure.owner, opponent);
    assert_eq!(treasure.controller, player);
    assert!(treasure.tapped);
    game.state.objects[elf as usize].tapped = false;
    let before = game.state.players[player.index()].mana[4];
    choose(&mut game, |action| match action {
        Action::TapMana { card } => *card == elf,
        Action::ActivateMana {
            card, production, ..
        } => *card == elf && *production == [0, 0, 0, 0, 1, 0],
        _ => false,
    });
    assert_eq!(game.state.players[player.index()].mana[4], before + 1);
    assert_eq!(game.object_view(elf).zone, Zone::Battlefield);
    game.state.objects[elf as usize].tapped = false;
    let production = [0, 1, 0, 0, 0, 0];
    act(
        &mut game,
        Action::TapGrantedMana {
            card: elf,
            grantor: treasure.reference,
            production,
        },
    );
    if matches!(game.state.pending, Pending::EffectMana { .. }) {
        act(&mut game, Action::ChooseMana { production });
    }
    assert_eq!(game.object_view(elf).zone, Zone::Graveyard);
    assert!(game.object_view(elf).has_subtype("Elf"));
}

#[test]
fn wrangler_checks_own_hand_casts_again_and_creates_a_white_flying_spirit() {
    let player = Player::P0;
    let mut flash = scenario(player.other());
    let wrangler = put(&mut flash, player, "Wrangler of the Damned", Zone::Hand);
    act(&mut flash, Action::Pass);
    cast(&mut flash, wrangler, None);
    settle(&mut flash);
    assert!(flash.keywords(wrangler).contains(Keywords::FLASH));
    flash.begin_step(Step::End);
    flash.prepare_triggers();
    assert!(flash.state.stack.is_empty());

    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Wrangler of the Damned",
        Zone::Battlefield,
    );
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    game.begin_step(Step::End);
    game.prepare_triggers();
    let mut response = game.clone();
    cast(&mut response, shock, Some(Target::Player(player.other())));
    settle(&mut response);
    assert!(token_objects(&response, player, "Spirit").is_empty());
    settle(&mut game);
    let spirits = token_objects(&game, player, "Spirit");
    assert_eq!(spirits.len(), 1);
    let spirit = &spirits[0];
    assert_eq!((spirit.power, spirit.toughness, spirit.colors), (2, 2, 1));
    assert!(spirit.keywords.contains(Keywords::FLYING));
}

#[test]
fn sandstorm_verge_has_colorless_mana_and_a_sorcery_speed_block_restriction() {
    let player = Player::P0;
    let mut game = scenario(player);
    let verge = put(&mut game, player, "Sandstorm Verge", Zone::Hand);
    let foe = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    act(&mut game, Action::PlayLand { card: verge });
    assert!(game.object_view(verge).has_subtype("Desert"));
    assert!(!game.object_view(verge).tapped);
    let mut mana = game.clone();
    act(
        &mut mana,
        Action::ActivateMana {
            card: verge,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(mana.state.players[player.index()].mana[5], 9);
    let activate = Action::AnnounceActivate {
        card: verge,
        ability: 1,
        counter_payment: Counters::default(),
    };
    let mut off_timing = game.clone();
    off_timing.begin_step(Step::BeginCombat);
    reject(&mut off_timing, activate.clone());
    act(&mut game, activate);
    act(&mut game, target(foe));
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(verge).tapped);
    assert!(game.keywords(foe).contains(Keywords::CANT_BLOCK));
    advance_turn(&mut game);
    assert!(!game.keywords(foe).contains(Keywords::CANT_BLOCK));
}

#[test]
fn canal_threshold_and_soured_springs_entry_use_actual_lands_and_colors() {
    let player = Player::P1;
    for other_lands in [2, 3] {
        let mut game = scenario(player);
        for _ in 0..other_lands {
            put(&mut game, player, "Forest", Zone::Battlefield);
        }
        put(&mut game, player.other(), "Forest", Zone::Battlefield);
        let canal = put(&mut game, player, "Spirebluff Canal", Zone::Hand);
        act(&mut game, Action::PlayLand { card: canal });
        assert_eq!(game.object_view(canal).tapped, other_lands == 3);
        if other_lands == 2 {
            choose(&mut game, |action| {
                matches!(action, Action::ActivateMana { card, production, .. }
                if *card == canal && *production == [0, 0, 0, 1, 0, 0])
            });
            assert_eq!(game.state.players[player.index()].mana[3], 9);
        }
    }

    let mut game = scenario(player);
    let springs = put(&mut game, player, "Soured Springs", Zone::Hand);
    let swamp = put(&mut game, player, "Swamp", Zone::Battlefield);
    act(&mut game, Action::TapMana { card: swamp });
    assert_eq!(game.state.players[player.index()].mana[2], 9);
    act(&mut game, Action::PlayLand { card: springs });
    assert!(game.object_view(springs).tapped);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    assert!(game.state.players[player.index()].committed_crime_this_turn);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 19);
    game.state.objects[springs as usize].tapped = false;
    choose(&mut game, |action| {
        matches!(action, Action::ActivateMana { card, production, .. }
        if *card == springs && *production == [0, 1, 0, 0, 0, 0])
    });
    assert_eq!(game.state.players[player.index()].mana[1], 9);
}

#[test]
fn scorching_shot_and_shoot_the_sheriff_enforce_creature_and_outlaw_targeting() {
    let player = Player::P0;
    let mut game = scenario(player);
    let shot = put(&mut game, player, "Scorching Shot", Zone::Hand);
    let sheriff = put(&mut game, player, "Shoot the Sheriff", Zone::Hand);
    let large = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let outlaw = put(
        &mut game,
        player.other(),
        "Silver Deputy",
        Zone::Battlefield,
    );
    let non_outlaw = put(&mut game, player.other(), "Goblin Piker", Zone::Battlefield);
    game.add_counter(large, CounterKind::PlusOne, 3);
    act(&mut game, Action::AnnounceCast { card: shot });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    act(&mut game, target(large));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(large).damage, 5);
    act(&mut game, Action::AnnounceCast { card: sheriff });
    reject(&mut game, target(outlaw));
    act(&mut game, target(non_outlaw));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(non_outlaw).zone, Zone::Graveyard);
}

#[test]
fn wylie_vigilance_and_tap_trigger_interact_with_steeds_saddle_four() {
    let player = Player::P1;
    let mut game = scenario(player);
    let steed = put(&mut game, player, "Seraphic Steed", Zone::Battlefield);
    let wylie = put(
        &mut game,
        player,
        "Wylie Duke, Atiin Hero",
        Zone::Battlefield,
    );
    let small = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    assert!(
        game.keywords(steed)
            .contains(Keywords::FIRST_STRIKE.union(Keywords::LIFELINK))
    );
    let mut attack = game.clone();
    start_attack(&mut attack, wylie);
    assert!(!attack.object_view(wylie).tapped);
    assert_eq!(
        attack.state.players[player.index()].cards_drawn_this_turn,
        0
    );
    let mut unsaddled = game.clone();
    start_attack(&mut unsaddled, steed);
    assert!(token_objects(&unsaddled, player, "Angel").is_empty());
    activate(&mut game, steed, None);
    let little = game.object_ref(small);
    act(&mut game, Action::ChooseSaddler { object: little });
    reject(&mut game, Action::FinishSaddle);
    let duke = game.object_ref(wylie);
    act(&mut game, Action::ChooseSaddler { object: duke });
    act(&mut game, Action::FinishSaddle);
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    assert!(game.object_view(steed).saddled);
    assert!(game.object_view(wylie).tapped);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    start_attack(&mut game, steed);
    settle(&mut game);
    let angels = token_objects(&game, player, "Angel");
    assert_eq!(angels.len(), 1);
    assert_eq!(
        (angels[0].power, angels[0].toughness, angels[0].colors),
        (3, 3, 1)
    );
    assert!(angels[0].keywords.contains(Keywords::FLYING));
}

#[test]
fn trained_arynx_saddled_attack_grants_first_strike_and_scries() {
    let player = Player::P0;
    let mut game = scenario(player);
    let arynx = put(&mut game, player, "Trained Arynx", Zone::Battlefield);
    let rider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let top = put(&mut game, player, "Island", Zone::Library);
    let mut unsaddled = game.clone();
    start_attack(&mut unsaddled, arynx);
    assert!(unsaddled.state.stack.is_empty());
    assert!(!unsaddled.keywords(arynx).contains(Keywords::FIRST_STRIKE));
    saddle(&mut game, arynx, rider);
    start_attack(&mut game, arynx);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Bottom,
        },
    );
    assert!(game.keywords(arynx).contains(Keywords::FIRST_STRIKE));
    assert_eq!(
        game.state.players[player.index()].library.first(),
        Some(&top)
    );
    advance_turn(&mut game);
    assert!(!game.keywords(arynx).contains(Keywords::FIRST_STRIKE));
}

#[test]
fn stubborn_burrowfiend_mills_only_on_the_first_saddle_and_freezes_its_bonus() {
    let player = Player::P0;
    let mut game = scenario(player);
    let fiend = put(&mut game, player, "Stubborn Burrowfiend", Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    let prior = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let milled = put(&mut game, player, "Llanowar Elves", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    saddle(&mut game, fiend, first);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(stats(&game, fiend), (4, 4));
    game.move_object(prior, Zone::Exile);
    let deck_size = game.state.players[player.index()].library.len();
    saddle(&mut game, fiend, second);
    assert_eq!(game.state.players[player.index()].library.len(), deck_size);
    assert_eq!(stats(&game, fiend), (4, 4));
    advance_turn(&mut game);
    assert_eq!(stats(&game, fiend), (2, 2));
}

#[test]
fn slickshot_show_off_pumps_only_for_noncreature_casts_and_expires() {
    let player = Player::P1;
    let mut game = scenario(player);
    let show = put(&mut game, player, "Slickshot Show-Off", Zone::Hand);
    let mut plotted = game.clone();
    plot(&mut plotted, show, [0, 0, 0, 1, 0, 1]);
    cast(&mut game, show, None);
    settle(&mut game);
    assert!(
        game.keywords(show)
            .contains(Keywords::FLYING.union(Keywords::HASTE))
    );
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, creature, None);
    settle(&mut game);
    assert_eq!(stats(&game, show), (1, 2));
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(stats(&game, show), (3, 2));
    start_attack(&mut game, show);
    advance_turn(&mut game);
    assert_eq!(stats(&game, show), (1, 2));
}

#[test]
fn plotted_woodland_creatures_and_stagecoach_security_keep_distinct_entry_effects() {
    let player = Player::P0;
    let mut game = scenario(player);
    let paladin = put(&mut game, player, "Spinewoods Paladin", Zone::Hand);
    let security = put(&mut game, player, "Stagecoach Security", Zone::Hand);
    let terror = put(&mut game, player, "Stingerback Terror", Zone::Hand);
    let rival = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let mut paladin_plot = game.clone();
    plot(&mut paladin_plot, paladin, [0, 0, 0, 0, 1, 3]);
    let mut security_plot = game.clone();
    plot(&mut security_plot, security, [1, 0, 0, 0, 0, 3]);
    let mut terror_plot = game.clone();
    plot(&mut terror_plot, terror, [0, 0, 0, 1, 0, 2]);
    cast(&mut game, paladin, None);
    settle(&mut game);
    assert!(game.keywords(paladin).contains(Keywords::TRAMPLE));
    assert_eq!(game.state.players[player.index()].life, 23);
    cast(&mut game, security, None);
    settle(&mut game);
    assert_eq!(stats(&game, paladin), (6, 5));
    assert_eq!(stats(&game, security), (5, 6));
    assert!(game.keywords(paladin).contains(Keywords::VIGILANCE));
    assert_eq!(stats(&game, rival), (2, 2));
    cast(&mut game, terror, None);
    settle(&mut game);
    assert!(
        game.keywords(terror)
            .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
    );
    assert_eq!(stats(&game, terror), (7, 7));
    put(&mut game, player, "Forest", Zone::Hand);
    assert_eq!(stats(&game, terror), (6, 6));
    assert!(!game.keywords(terror).contains(Keywords::VIGILANCE));
}

#[test]
fn armadillo_discards_to_find_a_desert_and_reach_and_ward_are_live() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let card = put(&mut game, player, "Spinewoods Armadillo", Zone::Hand);
    let desert = put(&mut game, player, "Sandstorm Verge", Zone::Library);
    let invalid = put(&mut game, player, "Spirebluff Canal", Zone::Library);
    activate(&mut game, card, None);
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: invalid });
    act(&mut game, Action::ChooseSearch { card: desert });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert_eq!(game.object_view(desert).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].life, 23);

    let armadillo = put(
        &mut game,
        opponent,
        "Spinewoods Armadillo",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Murder", Zone::Hand);
    assert!(game.keywords(armadillo).contains(Keywords::REACH));
    cast(&mut game, spell, Some(Target::Card(armadillo)));
    pass_pair(&mut game);
    act(&mut game, Action::DeclineResolutionMana);
    settle(&mut game);
    assert_eq!(game.object_view(armadillo).zone, Zone::Battlefield);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn sterling_hound_surveils_two_and_supplier_targets_only_another_own_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hound = put(&mut game, player, "Sterling Hound", Zone::Hand);
    let supplier = put(&mut game, player, "Sterling Supplier", Zone::Hand);
    let ours = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foe = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let keep = put(&mut game, player, "Island", Zone::Library);
    let bin = put(&mut game, player, "Shock", Zone::Library);
    cast(&mut game, hound, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: bin,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: keep,
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.object_view(bin).zone, Zone::Graveyard);
    assert_eq!(game.top_library_card(player), Some(keep));
    cast(&mut game, supplier, None);
    pass_pair(&mut game);
    reject(&mut game, target(supplier));
    reject(&mut game, target(foe));
    act(&mut game, target(ours));
    settle(&mut game);
    assert!(game.keywords(supplier).contains(Keywords::FLYING));
    assert_eq!(game.counter_count(ours, CounterKind::PlusOne), 1);
}

#[test]
fn sterling_keykeeper_can_act_off_turn_but_cannot_choose_a_mount() {
    let player = Player::P1;
    let mut game = scenario(player.other());
    let keeper = put(&mut game, player, "Sterling Keykeeper", Zone::Battlefield);
    let mount = put(
        &mut game,
        player.other(),
        "Trained Arynx",
        Zone::Battlefield,
    );
    let ordinary = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    act(&mut game, Action::Pass);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: keeper,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(&mut game, target(mount));
    act(&mut game, target(ordinary));
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(keeper).tapped && game.object_view(ordinary).tapped);
    assert!(!game.object_view(mount).tapped);
}

#[test]
fn stoic_sphinx_qualifies_hexproof_by_its_controller_casts_and_stop_cold_blanks_it() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let sphinx = put(&mut game, player, "Stoic Sphinx", Zone::Battlefield);
    let cold = put(&mut game, opponent, "Stop Cold", Zone::Hand);
    assert!(
        game.keywords(sphinx)
            .contains(Keywords::FLYING.union(Keywords::HEXPROOF))
    );
    let mut guarded = game.clone();
    act(&mut guarded, Action::Pass);
    reject(&mut guarded, Action::AnnounceCast { card: cold });
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    settle(&mut game);
    assert!(!game.keywords(sphinx).contains(Keywords::HEXPROOF));
    act(&mut game, Action::Pass);
    cast(&mut game, cold, Some(Target::Card(sphinx)));
    settle(&mut game);
    assert_eq!(
        game.object_view(cold).attached_to,
        Some(game.object_ref(sphinx))
    );
    assert!(
        !game
            .keywords(sphinx)
            .intersects(Keywords::FLYING.union(Keywords::FLASH))
    );
    assert!(game.object_view(sphinx).tapped);
    advance_turn(&mut game);
    advance_turn(&mut game);
    assert!(game.object_view(sphinx).tapped);
    game.move_object(cold, Zone::Graveyard);
    assert!(
        game.keywords(sphinx)
            .contains(Keywords::FLYING.union(Keywords::HEXPROOF))
    );
}

#[test]
fn thunder_lasso_attaches_then_uses_the_equipped_attack_to_tap_an_opposing_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let lasso = put(&mut game, player, "Thunder Lasso", Zone::Hand);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let next = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    let rival = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, lasso, None);
    pass_pair(&mut game);
    reject(&mut game, target(rival));
    act(&mut game, target(attacker));
    settle(&mut game);
    assert_eq!(stats(&game, attacker), (3, 3));
    activate(&mut game, lasso, Some(next));
    settle(&mut game);
    assert_eq!(stats(&game, attacker), (2, 2));
    assert_eq!(stats(&game, next), (3, 2));
    start_attack(&mut game, next);
    reject(&mut game, target(attacker));
    act(&mut game, target(rival));
    settle(&mut game);
    assert!(game.object_view(rival).tapped);
}

#[test]
fn tomb_trawler_uses_its_controllers_graveyard_and_needs_no_tap() {
    let player = Player::P0;
    let mut game = scenario(player);
    let trawler = put(&mut game, player, "Tomb Trawler", Zone::Battlefield);
    let own = put(&mut game, player, "Forest", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Island", Zone::Graveyard);
    game.state.objects[trawler as usize].tapped = true;
    act(
        &mut game,
        Action::AnnounceActivate {
            card: trawler,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(&mut game, target(foreign));
    act(&mut game, target(own));
    pay(&mut game);
    let mut stale = game.clone();
    stale.move_object(own, Zone::Exile);
    stale.move_object(own, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(own).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.first(),
        Some(&own)
    );
}

#[test]
fn dredger_pays_life_to_make_treasure_and_varmint_can_sacrifice_for_either_type() {
    let player = Player::P0;
    let mut game = scenario(player);
    let dredger = put(&mut game, player, "Treasure Dredger", Zone::Battlefield);
    let varmint = put(&mut game, player, "Voracious Varmint", Zone::Battlefield);
    let artifact = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let enchantment = put(&mut game, player.other(), "Omniscience", Zone::Battlefield);
    activate(&mut game, dredger, None);
    settle(&mut game);
    assert!(game.object_view(dredger).tapped);
    assert_eq!(game.state.players[player.index()].life, 19);
    assert_eq!(token_objects(&game, player, "Treasure").len(), 1);
    assert!(game.keywords(varmint).contains(Keywords::VIGILANCE));
    for chosen in [artifact, enchantment] {
        let mut branch = game.clone();
        activate(&mut branch, varmint, Some(chosen));
        assert_eq!(branch.object_view(varmint).zone, Zone::Graveyard);
        settle(&mut branch);
        assert_eq!(branch.object_view(chosen).zone, Zone::Graveyard);
    }
}

#[test]
fn vial_smasher_only_hits_the_opponent_and_vault_plunderer_can_choose_either_player() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Vial Smasher, Gleeful Grenadier",
        Zone::Battlefield,
    );
    game.create_tokens(player, Card::from_catalogue_id(5612), 1, false);
    game.prepare_triggers();
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
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    assert!(game.state.players[player.index()].committed_crime_this_turn);

    let mut draw_game = scenario(player);
    let plunderer = put(&mut draw_game, player, "Vault Plunderer", Zone::Hand);
    cast(&mut draw_game, plunderer, None);
    pass_pair(&mut draw_game);
    for recipient in [player, opponent] {
        let mut branch = draw_game.clone();
        act(
            &mut branch,
            Action::ChooseTarget {
                target: Target::Player(recipient),
            },
        );
        settle(&mut branch);
        assert_eq!(
            branch.state.players[recipient.index()].cards_drawn_this_turn,
            1
        );
        assert_eq!(branch.state.players[recipient.index()].life, 19);
        assert_eq!(
            branch.state.players[player.index()].committed_crime_this_turn,
            recipient == opponent
        );
    }
}

#[test]
fn take_up_the_shield_leaves_the_counter_and_temporary_combat_abilities() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let shield = put(&mut game, player, "Take Up the Shield", Zone::Hand);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let removal = put(&mut game, opponent, "Murder", Zone::Hand);
    cast(&mut game, shield, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
    let abilities = Keywords::LIFELINK.union(Keywords::INDESTRUCTIBLE);
    assert!(game.keywords(creature).contains(abilities));
    hit_player(&mut game, &[creature], opponent);
    assert_eq!(game.state.players[player.index()].life, 23);
    act(&mut game, Action::Pass);
    cast(&mut game, removal, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    advance_turn(&mut game);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
    assert!(!game.keywords(creature).intersects(abilities));
}
