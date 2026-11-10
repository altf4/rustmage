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
        for card in discards {
            act(&mut branch, Action::Discard { card });
            assert_eq!(branch.object_view(card).zone, Zone::Graveyard);
        }
        settle(&mut branch);
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
