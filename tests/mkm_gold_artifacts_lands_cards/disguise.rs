use super::support::*;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn disguise(game: &mut Game, id: CardId) -> ObjectRef {
    let source = announce_hand_disguise(game, id);
    pay(game);
    settle(game);
    let object = game.object_ref(source.card);
    assert!(game.is_face_down(object));
    object
}

fn turn_up(game: &mut Game, object: ObjectRef, colored: Mana, generic: u32) {
    act(
        game,
        Action::BeginFaceUp {
            object,
            cost: ManaCost { colored, generic },
        },
    );
    choose(game, |a| matches!(a, Action::PayFaceUp { .. }));
    assert_eq!(game.object_ref(object.card), object);
    assert!(!game.is_face_down(object));
}

fn finish_combat_damage(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
    for _ in 0..12 {
        if game.state.step == Step::CombatDamage {
            return;
        }
        choose(game, |a| matches!(a, Action::Pass | Action::FinishBlockers));
    }
    panic!("combat did not deal damage");
}

#[test]
fn rakish_scoundrel_protects_on_both_entry_and_turn_up() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bear = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Rakish Scoundrel", Zone::Hand);
    cast(&mut game, first, None);
    pass_pair(&mut game);
    target(&mut game, Target::Card(bear));
    settle(&mut game);
    assert!(game.keywords(first).contains(Keywords::DEATHTOUCH));
    assert!(game.keywords(bear).contains(Keywords::INDESTRUCTIBLE));

    let second = put(&mut game, player, "Rakish Scoundrel", Zone::Hand);
    let face = disguise(&mut game, second);
    let second = face.card;
    turn_up(&mut game, face, [0, 0, 1, 0, 1, 0], 4);
    target(&mut game, Target::Card(second));
    settle(&mut game);
    assert!(game.keywords(second).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn riftburst_hellion_and_sanguine_savior_pay_their_mixed_disguise_costs() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hellion = put(&mut game, player, "Riftburst Hellion", Zone::Hand);
    let face = disguise(&mut game, hellion);
    let hellion = face.card;
    turn_up(&mut game, face, [0, 0, 0, 1, 1, 0], 4);
    assert_eq!(game.stats(hellion).power, 6);
    assert_eq!(game.stats(hellion).toughness, 7);
    assert!(game.keywords(hellion).contains(Keywords::REACH));

    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let savior = put(&mut game, player, "Sanguine Savior", Zone::Hand);
    let face = disguise(&mut game, savior);
    let savior = face.card;
    turn_up(&mut game, face, [1, 0, 1, 0, 0, 0], 0);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(savior),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    target(&mut game, Target::Card(hellion));
    settle(&mut game);
    assert!(
        game.keywords(savior)
            .contains(Keywords::FLYING.union(Keywords::LIFELINK))
    );
    assert!(game.keywords(hellion).contains(Keywords::LIFELINK));
}

#[test]
fn shady_informant_only_deals_death_damage_face_up() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let informant = put(&mut game, player, "Shady Informant", Zone::Hand);
    let face = disguise(&mut game, informant);
    let informant = face.card;
    let mut hidden = game.clone();
    turn_up(&mut game, face, [0, 0, 1, 1, 0, 0], 2);
    for (branch, is_hidden) in [(&mut game, false), (&mut hidden, true)] {
        branch.state.priority = opponent;
        let shock = put(branch, opponent, "Shock", Zone::Hand);
        cast(branch, shock, Some(Target::Card(informant)));
        if is_hidden {
            pass_pair(branch);
            choose(branch, |a| matches!(a, Action::PayResolutionMana { .. }));
            settle(branch);
        } else {
            pass_pair(branch);
            target(branch, Target::Player(opponent));
            settle(branch);
        }
        assert_eq!(branch.object_view(informant).zone, Zone::Graveyard);
        assert_eq!(
            branch.state.players[opponent.index()].life,
            if is_hidden { 20 } else { 18 }
        );
    }
}

#[test]
fn undercover_crocodelf_investigates_only_after_its_combat_damage() {
    let player = Player::P0;
    let mut game = scenario(player);
    let croc = put(&mut game, player, "Undercover Crocodelf", Zone::Hand);
    let face = disguise(&mut game, croc);
    let croc = face.card;
    turn_up(&mut game, face, [0, 1, 0, 0, 1, 0], 3);
    assert_eq!(game.stats(croc).power, 5);
    game.state.objects[croc as usize].summoning_sick = false;
    assert_eq!(game.controlled_token_ids(player).count(), 0);
    finish_combat_damage(&mut game, &[croc]);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 15);
    let clue = game.controlled_token_ids(player).next().unwrap();
    assert!(game.object_view(clue).has_subtype("Clue"));
}

#[test]
fn sumala_sentry_counts_the_exact_permanent_and_keeps_its_own_counter_if_that_card_leaves() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sumala = put(&mut game, player, "Sumala Sentry", Zone::Battlefield);
    assert!(game.keywords(sumala).contains(Keywords::REACH));
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Library);
    game.cloak_library_top(player, player);
    assert!(!game.advance_entries());
    let exact = game.object_ref(bear);
    turn_up(&mut game, exact, [0, 0, 0, 0, 1, 0], 1);
    let mut gone = game.clone();
    gone.move_object(bear, Zone::Exile);
    settle(&mut gone);
    assert_eq!(gone.counter_count(sumala, CounterKind::PlusOne), 1);
    settle(&mut game);
    assert_eq!(game.counter_count(sumala, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(bear, CounterKind::PlusOne), 1);
}

#[test]
fn tin_street_gossip_produces_two_restricted_mana_and_keeps_vigilance() {
    let player = Player::P0;
    let mut game = scenario(player);
    let gossip = put(&mut game, player, "Tin Street Gossip", Zone::Battlefield);
    assert!(game.keywords(gossip).contains(Keywords::VIGILANCE));
    let hellion = put(&mut game, player, "Riftburst Hellion", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    reject(&mut game, Action::AnnounceCast { card: shock });
    let hellion = announce_hand_disguise(&mut game, hellion).card;
    act(
        &mut game,
        Action::ActivateMana {
            card: gossip,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 1, 1, 0],
        },
    );
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 1, 1],
        },
    );
    settle(&mut game);
    assert!(game.is_face_down(game.object_ref(hellion)));
    assert!(game.object_view(gossip).tapped);
}

#[test]
fn yarus_grants_other_creatures_haste_draws_once_and_returns_an_exact_death() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let yarus = put(
        &mut game,
        player,
        "Yarus, Roar of the Old Gods",
        Zone::Battlefield,
    );
    assert!(!game.keywords(yarus).contains(Keywords::HASTE));
    let mut hidden = Vec::new();
    for name in ["Grizzly Bears", "Bear Cub"] {
        let card = put(&mut game, player, name, Zone::Library);
        game.cloak_library_top(player, player);
        assert!(!game.advance_entries());
        assert!(game.keywords(card).contains(Keywords::HASTE));
        hidden.push(card);
    }
    finish_combat_damage(&mut game, &hidden);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.state.players[opponent.index()].life, 16);
    let first = hidden[0];
    let old = game.object_ref(first);
    game.state.priority = opponent;
    game.state.players[opponent.index()].mana = [0, 0, 0, 1, 0, 2];
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(first)));
    pass_pair(&mut game);
    choose(&mut game, |a| matches!(a, Action::PayResolutionMana { .. }));
    settle(&mut game);
    assert!(game.on_battlefield(first));
    assert_ne!(game.object_ref(first), old);
    assert!(!game.is_face_down(game.object_ref(first)));
}
