//! Actual C–D Outlaws card programs and inherited interactions.
#[path = "support/otj_e_l.rs"]
mod scenarios;
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use scenarios::{announce_activation, resolve, saddle, spree, start_step, tokens};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, card: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

#[test]
fn cactusfolk_checks_current_power_and_excludes_itself_and_opposing_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Cactusfolk Sureshot", Zone::Battlefield);
    let large = put(&mut game, player, "Daring Thunder-Thief", Zone::Battlefield);
    let small = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Daring Thunder-Thief",
        Zone::Battlefield,
    );
    start_step(&mut game, Step::BeginCombat);
    game.add_counter(small, CounterKind::PlusOne, 1);
    resolve(&mut game);
    let granted = Keywords::HASTE.union(Keywords::TRAMPLE);
    for recipient in [large, small] {
        assert!(game.keywords(recipient).contains(granted));
    }
    for other in [source, enemy] {
        assert!(!game.keywords(other).contains(granted));
    }
    assert!(game.keywords(source).contains(Keywords::REACH));
}

#[test]
fn calamity_can_copy_the_same_paid_rider_twice_and_sacrifices_both_copies() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Calamity, Galloping Inferno",
        Zone::Battlefield,
    );
    let rider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let unpaid = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    saddle(&mut game, source, &[rider]);
    attack(&mut game, source);
    pass_pair(&mut game);
    let unpaid = game.object_ref(unpaid);
    reject(&mut game, Action::ChooseEffectObject { object: unpaid });
    let rider_ref = game.object_ref(rider);
    for _ in 0..2 {
        act(&mut game, Action::ChooseEffectObject { object: rider_ref });
    }
    let giant = Card::from_name("Hill Giant").unwrap();
    let copies: Vec<_> = tokens(&game, player, giant)
        .into_iter()
        .filter(|&id| game.object_view(id).is_token())
        .collect();
    assert_eq!(copies.len(), 2);
    for &copy in &copies {
        assert!(game.object_view(copy).tapped);
        assert!(
            game.state
                .attackers
                .iter()
                .any(|attack| attack.card == copy)
        );
    }
    start_step(&mut game, Step::End);
    resolve(&mut game);
    assert!(copies.iter().all(|&id| !game.on_battlefield(id)));
    assert!(game.on_battlefield(rider));
}

#[test]
fn canyon_crab_checks_hand_casts_at_both_end_step_boundaries() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Canyon Crab", Zone::Battlefield);
    announce_activation(&mut game, source, 0);
    pay(&mut game);
    resolve(&mut game);
    assert_eq!(stats(&game, source), (2, 3));
    let land = put(&mut game, player, "Island", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let top = game.top_library_card(player).unwrap();
    act(&mut game, Action::PlayLand { card: land });
    start_step(&mut game, Step::End);
    let mut response = game.clone();
    response.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    cast(&mut response, shock, Some(Target::Player(player.other())));
    resolve(&mut response);
    assert_eq!(response.object_view(top).zone, Zone::Library);
    pass_pair(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: top });
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}

#[test]
fn caught_in_the_crossfire_resolves_each_paid_outlaw_cohort() {
    let player = Player::P1;
    for mode in 0..3 {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Caught in the Crossfire", Zone::Hand);
        let outlaw = put(
            &mut game,
            player.other(),
            "Spyglass Siren",
            Zone::Battlefield,
        );
        let other = put(&mut game, player, "Ankle Biter", Zone::Battlefield);
        spree(&mut game, spell, mode, &[]);
        let cast = game.state.stack.last().unwrap();
        assert_eq!(cast.mana_value(), 2);
        assert_eq!(
            cast.cast.unwrap().payment.into_iter().sum::<u32>(),
            if mode == 2 { 4 } else { 3 }
        );
        resolve(&mut game);
        assert_eq!(game.on_battlefield(outlaw), mode == 1);
        assert_eq!(game.on_battlefield(other), mode == 0);
    }
}

#[test]
fn caustic_bronco_reveals_and_takes_the_card_without_counting_a_draw() {
    let player = Player::P0;
    for saddled in [false, true] {
        let mut game = scenario(player);
        let bronco = put(&mut game, player, "Caustic Bronco", Zone::Battlefield);
        let rider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let top = put(&mut game, player, "Hill Giant", Zone::Library);
        if saddled {
            saddle(&mut game, bronco, &[rider]);
        }
        attack(&mut game, bronco);
        resolve(&mut game);
        assert_eq!(game.object_view(top).zone, Zone::Hand);
        assert!(
            game.observe(player.other())
                .known_hand
                .contains(&game.object_ref(top))
        );
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
        let payer = if saddled { player.other() } else { player };
        assert_eq!(game.state.players[payer.index()].life, 16);
        assert_eq!(game.state.players[payer.other().index()].life, 20);
    }
}

#[test]
fn claim_jumper_rechecks_after_each_optional_search_and_shuffles_once() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Claim Jumper", Zone::Hand);
    let first = put(&mut game, player, "Plains", Zone::Library);
    let second = put(&mut game, player, "Plains", Zone::Library);
    for _ in 0..2 {
        put(&mut game, player.other(), "Forest", Zone::Battlefield);
    }
    let mut expected_rng = game.state.rng;
    expected_rng.shuffle(&mut [0; 8]);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    let mut caught_up = game.clone();
    for _ in 0..2 {
        put(&mut caught_up, player, "Forest", Zone::Battlefield);
    }
    resolve(&mut caught_up);
    assert_eq!(caught_up.state.players[player.index()].library.len(), 10);
    pass_pair(&mut game);
    let mut decline = game.clone();
    for _ in 0..2 {
        act(&mut decline, Action::ChooseOptional { accept: false });
    }
    assert_eq!(decline.state.rng, game.state.rng);
    assert_eq!(decline.state.players[player.index()].library.len(), 10);
    let mut find_none = game.clone();
    act(&mut find_none, Action::ChooseOptional { accept: true });
    act(&mut find_none, Action::FinishSearch);
    act(&mut find_none, Action::ChooseOptional { accept: false });
    let mut unsuccessful_rng = game.state.rng;
    unsuccessful_rng.shuffle(&mut [0; 10]);
    assert_eq!(find_none.state.rng, unsuccessful_rng);
    for selected in [first, second] {
        act(&mut game, Action::ChooseOptional { accept: true });
        act(&mut game, Action::ChooseSearch { card: selected });
        act(&mut game, Action::FinishSearch);
        assert!(game.on_battlefield(selected));
        assert!(game.object_view(selected).tapped);
    }
    assert_eq!(game.state.rng, expected_rng);
    assert!(game.keywords(source).contains(Keywords::VIGILANCE));

    let mut even = scenario(player);
    let source = put(&mut even, player, "Claim Jumper", Zone::Hand);
    cast(&mut even, source, None);
    pass_pair(&mut even);
    assert!(even.state.stack.is_empty());
}

#[test]
fn colossal_rattlewurm_uses_desert_timing_and_exiles_its_graveyard_incarnation() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Colossal Rattlewurm", Zone::Hand);
    game.state.active = player.other();
    game.state.step = Step::Upkeep;
    reject(&mut game, Action::AnnounceCast { card: spell });
    let desert = put(&mut game, player, "Bristling Backwoods", Zone::Battlefield);
    cast(&mut game, spell, None);
    assert!(game.keywords(spell).contains(Keywords::FLASH));
    resolve(&mut game);
    assert!(game.keywords(spell).contains(Keywords::TRAMPLE));
    game.move_object(spell, Zone::Graveyard);
    game.move_object(desert, Zone::Library);
    game.state.priority = player;
    announce_activation(&mut game, spell, 0);
    pay(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseSearch { card: desert });
    act(&mut game, Action::FinishSearch);
    assert!(game.on_battlefield(desert) && game.object_view(desert).tapped);
}

#[test]
fn consuming_ashes_uses_pre_exile_mana_value_and_needs_a_legal_target() {
    let player = Player::P1;
    for (name, surveils) in [("Ankle Biter", true), ("Hill Giant", false)] {
        let mut game = scenario(player);
        let ashes = put(&mut game, player, "Consuming Ashes", Zone::Hand);
        let target = put(&mut game, player.other(), name, Zone::Battlefield);
        cast(&mut game, ashes, Some(Target::Card(target)));
        let mut invalid = game.clone();
        invalid.move_object(target, Zone::Hand);
        resolve(&mut invalid);
        assert!(invalid.observe(player).library_choice.is_none());
        pass_pair(&mut game);
        assert_eq!(game.object_view(target).zone, Zone::Exile);
        assert_eq!(game.observe(player).library_choice.is_some(), surveils);
    }
}

#[test]
fn dance_searches_before_it_freezes_the_green_elementals_size() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Dance of the Tumbleweeds", Zone::Hand);
    let old = put(&mut game, player, "Forest", Zone::Battlefield);
    let forest = game.top_library_card(player).unwrap();
    spree(&mut game, spell, 2, &[]);
    assert_eq!(
        game.state
            .stack
            .last()
            .unwrap()
            .cast
            .unwrap()
            .payment
            .into_iter()
            .sum::<u32>(),
        6
    );
    pass_pair(&mut game);
    act(&mut game, Action::ChooseSearch { card: forest });
    act(&mut game, Action::FinishSearch);
    assert!(game.on_battlefield(forest) && !game.object_view(forest).tapped);
    let elemental = tokens(&game, player, Card::from_catalogue_id(5614))[0];
    assert_eq!(stats(&game, elemental), (2, 2));
    assert_eq!(game.object_view(elemental).colors, 16);
    game.move_object(old, Zone::Graveyard);
    assert_eq!(stats(&game, elemental), (2, 2));
}

#[test]
fn deepmuck_mills_once_above_the_crime_committing_spell() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Deepmuck Desperado", Zone::Battlefield);
    let victim = player.other();
    let library = game.state.players[victim.index()].library.len();
    for i in 0..2 {
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(victim)));
        if i == 0 {
            assert_eq!(game.state.stack.last().unwrap().source.unwrap().id, source);
            pass_pair(&mut game);
            assert_eq!(game.state.players[victim.index()].life, 20);
        }
        resolve(&mut game);
        assert_eq!(
            game.state.players[victim.index()].library.len(),
            library - 3
        );
    }
}

#[test]
fn deserts_due_samples_only_current_controlled_deserts() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Desert's Due", Zone::Hand);
    let victim = put(
        &mut game,
        player.other(),
        "Calamity, Galloping Inferno",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Creosote Heath", Zone::Battlefield);
    let second = put(&mut game, player, "Bristling Backwoods", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Bristling Backwoods",
        Zone::Battlefield,
    );
    cast(&mut game, spell, Some(Target::Card(victim)));
    game.move_object(first, Zone::Graveyard);
    resolve(&mut game);
    assert_eq!(stats(&game, victim), (1, 3));
    game.move_object(second, Zone::Graveyard);
    assert_eq!(stats(&game, victim), (1, 3));
}

#[test]
fn double_down_copies_only_outlaw_casts_and_a_permanent_copy_enters_as_a_token() {
    let player = Player::P1;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Double Down", Zone::Battlefield);
    let other = put(&mut game, player, "Ankle Biter", Zone::Hand);
    let outlaw = put(&mut game, player, "Boneyard Desecrator", Zone::Hand);
    cast(&mut game, other, None);
    assert_eq!(game.state.stack.len(), 1);
    resolve(&mut game);
    cast(&mut game, outlaw, None);
    assert_eq!(
        game.state.stack.last().unwrap().source.unwrap().id,
        provider
    );
    pass_pair(&mut game);
    assert!(game.state.stack.last().unwrap().is_copy);
    resolve(&mut game);
    let copies = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|o| {
            o.zone == Zone::Battlefield && o.is_token() && o.card.name() == "Boneyard Desecrator"
        })
        .count();
    assert_eq!(copies, 1);
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 2);
}

#[test]
fn duelist_counts_draws_and_lets_its_controller_choose_one_crime_loot() {
    let player = Player::P0;
    let mut game = scenario(player);
    let duelist = put(&mut game, player, "Duelist of the Mind", Zone::Battlefield);
    let added = game.top_library_card(player).unwrap();
    game.move_object(added, Zone::Hand);
    assert_eq!(stats(&game, duelist), (0, 3));
    assert!(
        game.keywords(duelist)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    resolve(&mut declined);
    assert_eq!(stats(&declined, duelist).0, 0);
    act(&mut game, Action::ChooseOptional { accept: true });
    act(&mut game, Action::Discard { card: added });
    resolve(&mut game);
    assert_eq!(stats(&game, duelist), (1, 3));
}

#[test]
fn dust_animus_uses_five_untapped_lands_for_both_entry_counter_kinds() {
    let player = Player::P1;
    for untapped in [4, 5] {
        let mut game = scenario(player);
        let animus = put(&mut game, player, "Dust Animus", Zone::Hand);
        let mut plot = game.clone();
        act(&mut plot, Action::AnnouncePlot { card: animus });
        assert_eq!(
            plot.observe(player).plot_payment.unwrap().cost,
            ManaCost {
                generic: 1,
                colored: [1, 0, 0, 0, 0, 0],
            }
        );
        for land in 0..5 {
            let id = put(&mut game, player, "Plains", Zone::Battlefield);
            game.state.objects[id as usize].tapped = land >= untapped;
        }
        cast(&mut game, animus, None);
        resolve(&mut game);
        let countered = untapped == 5;
        assert_eq!(
            game.counter_count(animus, CounterKind::PlusOne),
            if countered { 2 } else { 0 }
        );
        assert_eq!(
            game.counter_count(animus, CounterKind::Lifelink),
            u64::from(countered)
        );
        assert_eq!(
            game.keywords(animus).contains(Keywords::LIFELINK),
            countered
        );
        assert!(game.keywords(animus).contains(Keywords::FLYING));
    }
}
