//! Paid rules proofs for the remaining inherited Wilds of Eldraine E–L cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn target_player(player: Player) -> Action {
    Action::ChooseTarget {
        target: Target::Player(player),
    }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn bargain(game: &mut Game, offering: Option<CardId>) {
    let sacrifice = offering.map(|card| game.object_ref(card));
    act(game, Action::ChooseBargain { sacrifice });
}

fn cast_adventurer(game: &mut Game, card: CardId) {
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    act(game, Action::AnnounceExile { card });
    pay(game);
    settle(game);
    assert_eq!(game.object_view(card).zone, Zone::Battlefield);
}

fn tokens(game: &Game, player: Player, subtype: &str) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|view| {
            view.zone == Zone::Battlefield
                && view.controller == player
                && view.is_token()
                && view.has_subtype(subtype)
        })
        .collect()
}

fn reach_attackers(game: &mut Game) {
    for _ in 0..16 {
        if game.state.pending == Pending::Attackers {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("attackers did not begin");
}

#[test]
fn ice_out_counters_the_exact_spell_at_both_printed_bargain_prices() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let spell = put(&mut game, player, "Ice Out", Zone::Hand);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
    cast(&mut game, shock, Some(Target::Player(player)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: spell });

    for paid in [false, true] {
        let mut branch = game.clone();
        bargain(&mut branch, paid.then_some(offering));
        act(
            &mut branch,
            Action::ChooseTarget {
                target: Target::Spell(shock),
            },
        );
        let domain = branch.mana_payment_domain().unwrap();
        assert_eq!(domain.required, [0, 2, 0, 0, 0, 0]);
        assert_eq!(domain.generic, if paid { 0 } else { 1 });
        pay(&mut branch);
        assert_eq!(
            branch.object_view(offering).zone,
            if paid {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        settle(&mut branch);
        assert_eq!(branch.object_view(shock).zone, Zone::Graveyard);
        assert_eq!(branch.state.players[player.index()].life, 20);
    }
}

#[test]
fn johanns_stopgap_draws_only_when_the_exact_nonland_target_survives() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Johann's Stopgap", Zone::Hand);
    let drawn = put(&mut game, player, "Opt", Zone::Library);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    let permanent = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });

    for paid in [false, true] {
        let mut branch = game.clone();
        bargain(&mut branch, paid.then_some(offering));
        reject(&mut branch, target(land));
        act(&mut branch, target(permanent));
        let domain = branch.mana_payment_domain().unwrap();
        assert_eq!(domain.required, [0, 1, 0, 0, 0, 0]);
        assert_eq!(domain.generic, if paid { 1 } else { 3 });
        pay(&mut branch);

        if paid {
            let mut stale = branch.clone();
            cast(&mut stale, bounce, Some(Target::Card(permanent)));
            settle(&mut stale);
            assert_eq!(stale.object_view(drawn).zone, Zone::Library);
            assert_eq!(stale.object_view(offering).zone, Zone::Graveyard);
        }

        settle(&mut branch);
        assert_eq!(branch.object_view(permanent).zone, Zone::Hand);
        assert_eq!(branch.object_view(drawn).zone, Zone::Hand);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            1
        );
    }
}

#[test]
fn kellans_lightblades_checks_live_combat_and_switches_damage_to_destroy() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let spell = put(&mut game, player, "Kellan's Lightblades", Zone::Hand);
    let attacker = put(&mut game, opponent, "Shivan Dragon", Zone::Battlefield);
    let bystander = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let plains = [(); 2].map(|_| put(&mut game, player, "Plains", Zone::Battlefield));
    act(&mut game, Action::Pass);
    reject(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::Pass);
    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    act(&mut game, Action::Pass);
    for card in plains {
        act(&mut game, Action::TapMana { card });
    }
    act(&mut game, Action::AnnounceCast { card: spell });

    for paid in [false, true] {
        let mut branch = game.clone();
        bargain(&mut branch, paid.then_some(offering));
        reject(&mut branch, target(bystander));
        act(&mut branch, target(attacker));
        let domain = branch.mana_payment_domain().unwrap();
        assert_eq!(domain.required, [1, 0, 0, 0, 0, 0]);
        assert_eq!(domain.generic, 1);
        pay(&mut branch);
        settle(&mut branch);
        let victim = branch.object_view(attacker);
        assert_eq!(
            victim.zone,
            if paid {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        if !paid {
            assert_eq!(victim.damage, 3);
        }
    }
}

#[test]
fn bargain_creatures_distinguish_a_cost_reduction_from_a_paid_entry_payoff() {
    let player = Player::P1;
    let opponent = player.other();
    for (name, keywords, plain, reduced, life_paid, life_unpaid, opponent_paid) in [
        ("Hamlet Glutton", Keywords::TRAMPLE, 5, 3, 23, 23, 20),
        ("High Fae Negotiator", Keywords::FLYING, 3, 3, 23, 20, 17),
    ] {
        for paid in [false, true] {
            let mut game = scenario(player);
            let creature = put(&mut game, player, name, Zone::Hand);
            let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
            act(&mut game, Action::AnnounceCast { card: creature });
            bargain(&mut game, paid.then_some(offering));
            assert_eq!(
                game.mana_payment_domain().unwrap().generic,
                if paid { reduced } else { plain }
            );
            pay(&mut game);
            settle(&mut game);
            let permanent = game.object_view(creature);
            assert_eq!(permanent.zone, Zone::Battlefield);
            assert!(permanent.keywords.contains(keywords));
            assert_eq!(
                game.state.players[player.index()].life,
                if paid { life_paid } else { life_unpaid }
            );
            assert_eq!(
                game.state.players[opponent.index()].life,
                if paid { opponent_paid } else { 20 }
            );
        }
    }
}

#[test]
fn eriettes_whisper_keeps_the_opponents_discard_when_its_role_target_leaves() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let whisper = put(&mut game, player, "Eriette's Whisper", Zone::Hand);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    let discards = [(); 2].map(|_| put(&mut game, opponent, "Forest", Zone::Hand));
    act(&mut game, Action::AnnounceCast { card: whisper });
    reject(&mut game, target_player(player));
    act(&mut game, target_player(opponent));
    reject(&mut game, target(foreign));
    act(&mut game, target(creature));
    pay(&mut game);

    for stale in [false, true] {
        let mut branch = game.clone();
        if stale {
            cast(&mut branch, bounce, Some(Target::Card(creature)));
            pass_pair(&mut branch);
        }
        pass_pair(&mut branch);
        assert_eq!(branch.actor(), Some(opponent));
        act(&mut branch, Action::Discard { card: discards[0] });
        assert_eq!(branch.object_view(discards[0]).zone, Zone::Hand);
        act(&mut branch, Action::Discard { card: discards[1] });
        settle(&mut branch);
        for card in discards {
            assert_eq!(branch.object_view(card).zone, Zone::Graveyard);
        }
        let roles = tokens(&branch, player, "Role");
        if stale {
            assert_eq!(branch.object_view(creature).zone, Zone::Hand);
            assert!(roles.is_empty());
        } else {
            assert_eq!(roles.len(), 1);
            assert_eq!(roles[0].attached_to, Some(branch.object_ref(creature)));
            assert_eq!(stats(&branch, creature), (3, 3));
        }
    }
}

#[test]
fn veteran_and_guard_change_replace_one_role_and_knight_of_doves_observes_the_graveyard() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let veteran = put(&mut game, player, "Embereth Veteran", Zone::Hand);
    let doves = put(&mut game, player, "Knight of Doves", Zone::Hand);
    let werefox = put(
        &mut game,
        player,
        "Ferocious Werefox // Guard Change",
        Zone::Hand,
    );
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    for card in [doves, veteran] {
        cast(&mut game, card, None);
        settle(&mut game);
    }
    act(&mut game, activate(veteran, 0));
    reject(&mut game, target(veteran));
    assert!(game.legal_actions().contains(&target(foreign)));
    act(&mut game, target(bearer));
    assert_eq!(game.mana_payment_domain().unwrap().generic, 1);
    pay(&mut game);
    assert_eq!(game.object_view(veteran).zone, Zone::Graveyard);
    settle(&mut game);
    let roles = tokens(&game, player, "Role");
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].attached_to, Some(game.object_ref(bearer)));

    let mut attacking = game.clone();
    reach_attackers(&mut attacking);
    act(&mut attacking, Action::Attack { card: bearer });
    act(&mut attacking, Action::FinishAttackers);
    settle(&mut attacking);
    assert_eq!(attacking.counter_count(bearer, CounterKind::PlusOne), 1);

    for redirect in [false, true] {
        let mut branch = game.clone();
        if redirect {
            put(&mut branch, opponent, "Rest in Peace", Zone::Battlefield);
        }
        act(&mut branch, Action::AnnounceAdventure { card: werefox });
        reject(&mut branch, target(foreign));
        act(&mut branch, target(bearer));
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(stats(&branch, bearer), (3, 3));
        assert!(
            branch
                .object_view(bearer)
                .keywords
                .contains(Keywords::TRAMPLE)
        );
        assert_eq!(tokens(&branch, player, "Role").len(), 1);
        let birds = tokens(&branch, player, "Bird");
        assert_eq!(birds.len(), usize::from(!redirect));
        if let Some(bird) = birds.first() {
            assert_eq!((bird.power, bird.toughness, bird.colors), (1, 1, 0b00001));
            assert!(bird.keywords.contains(Keywords::FLYING));
        }
        cast_adventurer(&mut branch, werefox);
        assert!(
            branch
                .object_view(werefox)
                .keywords
                .contains(Keywords::TRAMPLE)
        );
    }
}

fn cast_adventure(game: &mut Game, card: CardId, recipient: Option<Target>) {
    act(game, Action::AnnounceAdventure { card });
    if let Some(target) = recipient {
        act(game, Action::ChooseTarget { target });
    }
    pay(game);
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..32 {
        if game.state.turn != turn {
            return;
        }
        let action = match &game.state.pending {
            Pending::Attackers => Action::FinishAttackers,
            Pending::Blockers => Action::FinishBlockers,
            Pending::Priority => Action::Pass,
            pending => panic!("unexpected turn choice {pending:?}"),
        };
        act(game, action);
    }
    panic!("turn did not finish");
}

#[test]
fn puny_snack_and_bakery_raid_produce_food_for_the_scavengers_turn_limit() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let hunter = put(
        &mut game,
        player,
        "Gingerbread Hunter // Puny Snack",
        Zone::Hand,
    );
    let scavenger = put(
        &mut game,
        player,
        "Hollow Scavenger // Bakery Raid",
        Zone::Hand,
    );
    let foe = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);

    cast_adventure(&mut game, hunter, Some(Target::Card(foe)));
    settle(&mut game);
    assert_eq!(stats(&game, foe), (8, 8));
    assert!(tokens(&game, player, "Food").is_empty());
    cast_adventurer(&mut game, hunter);
    assert_eq!(stats(&game, hunter), (5, 5));
    let hunter_food = tokens(&game, player, "Food").pop().unwrap();
    cast_adventure(&mut game, scavenger, None);
    settle(&mut game);
    let food = tokens(&game, player, "Food");
    assert_eq!(food.len(), 2);
    let bakery_food = food
        .into_iter()
        .find(|food| food.id != hunter_food.id)
        .unwrap();
    cast_adventurer(&mut game, scavenger);

    let pump = activate(scavenger, 0);
    act(&mut game, pump.clone());
    let wrong = game.object_ref(bow);
    reject(&mut game, Action::ChooseAdditionalCost { object: wrong });
    act(
        &mut game,
        Action::ChooseAdditionalCost {
            object: hunter_food.reference,
        },
    );
    assert_eq!(game.mana_payment_domain().unwrap().generic, 1);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, scavenger), (5, 4));
    assert_eq!(
        tokens(&game, player, "Food")[0].reference,
        bakery_food.reference
    );
    reject(&mut game, pump.clone());

    finish_turn(&mut game);
    assert_eq!(game.state.active, opponent);
    assert_eq!(stats(&game, scavenger), (3, 2));
    assert_eq!(stats(&game, foe), (10, 10));
    act(&mut game, Action::Pass);
    act(&mut game, Action::TapMana { card: forest });
    act(&mut game, pump);
    act(
        &mut game,
        Action::ChooseAdditionalCost {
            object: bakery_food.reference,
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, scavenger), (5, 4));
    assert!(tokens(&game, player, "Food").is_empty());
}

#[test]
fn thats_mine_creates_a_treasure_and_grabby_giant_draws_with_artifact_or_land() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let giant = put(&mut game, player, "Grabby Giant // That's Mine", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let draw = put(&mut game, player, "Opt", Zone::Library);
    cast_adventure(&mut game, giant, None);
    settle(&mut game);
    let treasures = tokens(&game, player, "Treasure");
    assert_eq!(treasures.len(), 1);
    let treasure = treasures[0].reference;
    cast_adventurer(&mut game, giant);
    assert!(game.object_view(giant).keywords.contains(Keywords::REACH));
    act(&mut game, activate(giant, 0));
    for card in [foreign, bear] {
        let object = game.object_ref(card);
        reject(&mut game, Action::ChooseAdditionalCost { object });
    }
    for object in [treasure, game.object_ref(land)] {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseAdditionalCost { object });
        let domain = branch.mana_payment_domain().unwrap();
        assert_eq!(domain.required, [0, 0, 0, 1, 0, 0]);
        assert_eq!(domain.generic, 2);
        pay(&mut branch);
        assert_ne!(branch.object_view(object.card).zone, Zone::Battlefield);
        assert_eq!(branch.object_view(draw).zone, Zone::Library);
        settle(&mut branch);
        assert_eq!(branch.object_view(draw).zone, Zone::Hand);
    }
}

#[test]
fn galvanic_giant_uses_the_cast_mana_value_and_storm_reading_draws_before_discards() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let giant = put(
        &mut game,
        player,
        "Galvanic Giant // Storm Reading",
        Zone::Hand,
    );
    let reading = put(
        &mut game,
        player,
        "Galvanic Giant // Storm Reading",
        Zone::Hand,
    );
    let foe = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let another = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let low = put(&mut game, player, "Hill Giant", Zone::Hand);
    let five = put(&mut game, player, "Serra Angel", Zone::Hand);
    let drawn = ["Forest", "Island", "Opt", "Shock"]
        .map(|name| put(&mut game, player, name, Zone::Library));
    cast(&mut game, giant, None);
    settle(&mut game);
    cast(&mut game, low, None);
    settle(&mut game);
    assert!(!game.object_view(foe).tapped);
    assert_eq!(game.counter_count(foe, CounterKind::Stun), 0);

    cast(&mut game, five, None);
    reject(&mut game, target(own));
    act(&mut game, target(foe));
    pass_pair(&mut game);
    assert_eq!(game.object_view(five).zone, Zone::Stack);
    assert!(game.object_view(foe).tapped);
    assert_eq!(game.counter_count(foe, CounterKind::Stun), 1);
    settle(&mut game);

    cast_adventure(&mut game, reading, None);
    act(&mut game, target(another));
    pass_pair(&mut game);
    assert!(game.object_view(another).tapped);
    assert_eq!(game.counter_count(another, CounterKind::Stun), 1);
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
    for card in drawn {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
    act(&mut game, Action::Discard { card: drawn[0] });
    assert_eq!(game.object_view(drawn[0]).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: drawn[1] });
    settle(&mut game);
    for card in [drawn[0], drawn[1]] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    assert_eq!(game.hand_count(player), 2);
    assert_eq!(game.object_view(reading).zone, Zone::Exile);
}

#[test]
fn stoke_genius_discards_the_whole_old_hand_then_draws_two_and_discounts_hearth() {
    let player = Player::P1;
    let mut game = scenario(player);
    let hearth = put(
        &mut game,
        player,
        "Hearth Elemental // Stoke Genius",
        Zone::Hand,
    );
    let old =
        ["Opt", "Divination", "Bear Cub"].map(|name| put(&mut game, player, name, Zone::Hand));
    let new = ["Island", "Forest"].map(|name| put(&mut game, player, name, Zone::Library));
    cast_adventure(&mut game, hearth, None);
    settle(&mut game);
    for card in old {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    for card in new {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
    assert_eq!(game.hand_count(player), 2);
    assert_eq!(game.object_view(hearth).zone, Zone::Exile);
    act(&mut game, Action::AnnounceExile { card: hearth });
    assert_eq!(game.mana_payment_domain().unwrap().generic, 3);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, hearth), (4, 5));
}

#[test]
fn frolicking_familiar_grows_for_both_spell_kinds_after_its_paid_adventure() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let familiar = put(
        &mut game,
        player,
        "Frolicking Familiar // Blow Off Steam",
        Zone::Hand,
    );
    let creature = put(&mut game, player, "Bear Cub", Zone::Hand);
    let instant = put(&mut game, player, "Shock", Zone::Hand);
    let sorcery = put(&mut game, player, "Divination", Zone::Hand);
    cast_adventure(&mut game, familiar, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    cast_adventurer(&mut game, familiar);
    assert!(
        game.object_view(familiar)
            .keywords
            .contains(Keywords::FLYING)
    );
    cast(&mut game, creature, None);
    settle(&mut game);
    assert_eq!(stats(&game, familiar), (2, 2));

    cast(&mut game, instant, Some(Target::Player(opponent)));
    pass_pair(&mut game);
    assert_eq!(stats(&game, familiar), (3, 3));
    assert_eq!(game.state.players[opponent.index()].life, 19);
    settle(&mut game);
    cast(&mut game, sorcery, None);
    settle(&mut game);
    assert_eq!(stats(&game, familiar), (4, 4));
    finish_turn(&mut game);
    assert_eq!(stats(&game, familiar), (2, 2));
}

#[test]
fn recruiter_hastes_trained_knights_and_trufflesnout_distinguishes_an_attack_alone() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let boar = put(
        &mut game,
        player,
        "Intrepid Trufflesnout // Go Hog Wild",
        Zone::Hand,
    );
    let recruiter = put(
        &mut game,
        player,
        "Imodane's Recruiter // Train Troops",
        Zone::Hand,
    );
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let late = put(&mut game, player, "Bear Cub", Zone::Hand);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    cast_adventure(&mut game, boar, Some(Target::Card(bear)));
    settle(&mut game);
    assert_eq!(stats(&game, bear), (4, 4));
    cast_adventurer(&mut game, boar);
    cast_adventure(&mut game, recruiter, None);
    settle(&mut game);
    let knights = tokens(&game, player, "Knight");
    assert_eq!(knights.len(), 2);
    for knight in &knights {
        assert_eq!(
            (knight.power, knight.toughness, knight.colors),
            (2, 2, 0b00001)
        );
        assert!(knight.keywords.contains(Keywords::VIGILANCE));
    }
    cast_adventurer(&mut game, recruiter);
    for (card, expected) in [
        (boar, (4, 1)),
        (bear, (5, 4)),
        (recruiter, (3, 2)),
        (knights[0].id, (3, 2)),
        (knights[1].id, (3, 2)),
    ] {
        assert_eq!(stats(&game, card), expected);
        assert!(game.object_view(card).keywords.contains(Keywords::HASTE));
    }
    assert_eq!(stats(&game, enemy), (2, 2));
    cast(&mut game, late, None);
    settle(&mut game);
    assert_eq!(stats(&game, late), (2, 2));
    assert!(!game.object_view(late).keywords.contains(Keywords::HASTE));

    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: boar });
    let mut group = game.clone();
    act(
        &mut group,
        Action::Attack {
            card: knights[0].id,
        },
    );
    act(&mut group, Action::FinishAttackers);
    settle(&mut group);
    assert!(tokens(&group, player, "Food").is_empty());
    assert!(!group.object_view(knights[0].id).tapped);

    act(&mut game, Action::FinishAttackers);
    settle(&mut game);
    assert_eq!(tokens(&game, player, "Food").len(), 1);
    finish_turn(&mut game);
    assert_eq!(stats(&game, bear), (2, 2));
    assert!(
        !game
            .object_view(knights[0].id)
            .keywords
            .contains(Keywords::HASTE)
    );
}
