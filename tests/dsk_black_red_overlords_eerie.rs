//! Authored black and red impending, eerie, delirium, and survival interactions.
mod support;

use rustmage::RoomDoor;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn step(game: &mut Game, next: Step) {
    game.begin_step(next);
    game.prepare_triggers();
}

fn settle_triggers(game: &mut Game) {
    for _ in 0..32 {
        match game.state.pending {
            Pending::Priority if game.state.stack.is_empty() && game.state.resolving.is_none() => {
                return;
            }
            Pending::Priority => act(game, Action::Pass),
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }));
            }
            ref pending => panic!("unexpected trigger continuation {pending:?}"),
        }
    }
    panic!("triggers did not settle");
}

fn announce_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn cast_overlord(game: &mut Game, player: Player, name: &str, impending: bool) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    act(game, Action::AnnounceCast { card });
    let cost = game
        .observe(player)
        .cost_choices
        .into_iter()
        .find(|choice| choice.impending == impending)
        .expect("fundable Overlord route")
        .index;
    act(game, Action::ChooseSpellCost { cost });
    pay(game);
    pass_pair(game);
    card
}

fn declare_attack(game: &mut Game, card: CardId) {
    step(game, Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

#[test]
fn balemurk_impending_mills_then_recovers_an_eligible_graveyard_card() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let walker = put(
        &mut game,
        player,
        "Liliana, Dreadhorde General",
        Zone::Graveyard,
    );
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Graveyard);
    let top = [
        "Forest",
        "Opt",
        "Hill Giant",
        "Overlord of the Boilerbilges",
    ]
    .map(|name| put(&mut game, player, name, Zone::Library));
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 1];
    let overlord = cast_overlord(&mut game, player, "Overlord of the Balemurk", true);
    assert_eq!(game.object_view(overlord).kinds, 32);
    assert_eq!(game.counter_count(overlord, CounterKind::Time), 5);
    pass_pair(&mut game);
    assert!(
        top.iter()
            .all(|&card| game.object_view(card).zone == Zone::Graveyard)
    );
    let recover = |game: &Game, card| Action::SelectGraveyardReturn {
        object: game.object_ref(card),
    };
    for card in [foreign, top[3], top[0]] {
        let action = recover(&game, card);
        reject(&mut game, action);
    }
    assert!(game.legal_actions().contains(&recover(&game, top[2])));
    let choice = recover(&game, walker);
    act(&mut game, choice);
    assert_eq!(game.object_view(walker).zone, Zone::Hand);

    step(&mut game, Step::End);
    settle_triggers(&mut game);
    assert_eq!(game.counter_count(overlord, CounterKind::Time), 4);
    game.pay_counters(overlord, &Counters::of(CounterKind::Time, 4));
    assert_eq!(stats(&game, overlord), (5, 5));
    game.state.objects[overlord as usize].summoning_sick = false;
    let count = game.state.players[player.index()].library.len();
    declare_attack(&mut game, overlord);
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.index()].library.len(), count - 4);
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.object_view(top[2]).zone, Zone::Graveyard);
}

#[test]
fn boilerbilges_cast_routes_trigger_on_entry_and_the_creature_triggers_on_attack() {
    let player = Player::P0;
    let opponent = player.other();
    for impending in [true, false] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.state.players[player.index()].mana = [0, 0, 0, 2, 0, if impending { 2 } else { 4 }];
        let overlord = cast_overlord(&mut game, player, "Overlord of the Boilerbilges", impending);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        assert_eq!(
            game.object_view(overlord).kinds,
            if impending { 32 } else { 33 }
        );
        assert_eq!(
            game.counter_count(overlord, CounterKind::Time),
            if impending { 4 } else { 0 }
        );
        if impending {
            announce_target(&mut game, Target::Card(creature));
            settle_triggers(&mut game);
            assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
            step(&mut game, Step::End);
            settle_triggers(&mut game);
            assert_eq!(game.counter_count(overlord, CounterKind::Time), 3);
        } else {
            announce_target(&mut game, Target::Player(opponent));
            settle_triggers(&mut game);
            assert_eq!(game.state.players[opponent.index()].life, 16);
            game.state.objects[overlord as usize].summoning_sick = false;
            declare_attack(&mut game, overlord);
            announce_target(&mut game, Target::Player(opponent));
            settle_triggers(&mut game);
            assert_eq!(game.state.players[opponent.index()].life, 12);
        }
    }
}

#[test]
fn eerie_uses_own_enchantment_entry_and_full_unlock_and_soulrager_counts_doors() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let leech = put(&mut game, player, "Balemurk Leech", Zone::Battlefield);
    let lifelink = put(&mut game, player, "Dashing Bloodsucker", Zone::Battlefield);
    let phantom = put(&mut game, player, "Infernal Phantom", Zone::Battlefield);
    let soulrager = put(&mut game, player, "Rampaging Soulrager", Zone::Battlefield);
    assert_eq!(stats(&game, leech), (2, 2));
    assert_eq!(stats(&game, lifelink), (2, 5));
    assert_eq!(stats(&game, phantom), (2, 3));
    assert_eq!(stats(&game, soulrager), (1, 4));
    let enemy = put(&mut game, opponent, "Deeproot Pilgrimage", Zone::Exile);
    game.move_object(enemy, Zone::Battlefield);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());

    let room = put(
        &mut game,
        player,
        "Charred Foyer // Warped Space",
        Zone::Hand,
    );
    act(
        &mut game,
        Action::AnnounceRoom {
            card: room,
            door: RoomDoor::Left,
            flash_surcharge: false,
        },
    );
    pay(&mut game);
    pass_pair(&mut game);
    settle_triggers(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    assert_eq!(stats(&game, lifelink), (4, 5));
    assert_eq!(stats(&game, phantom), (4, 3));
    assert_eq!(stats(&game, soulrager), (1, 4));

    let object = game.object_ref(room);
    act(
        &mut game,
        Action::BeginRoomUnlock {
            object,
            door: RoomDoor::Right,
            cost: ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 4,
            },
        },
    );
    choose(&mut game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
    settle_triggers(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(stats(&game, lifelink), (6, 5));
    assert!(
        game.object_view(lifelink)
            .keywords
            .contains(Keywords::LIFELINK)
    );
    assert_eq!(stats(&game, phantom), (6, 3));
    assert_eq!(stats(&game, soulrager), (4, 4));
    let mut stolen = game.clone();
    stolen.change_controller(object, opponent);
    assert_eq!(stats(&stolen, soulrager), (1, 4));

    let removal = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(phantom)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(phantom).zone, Zone::Graveyard);
    announce_target(&mut game, Target::Player(opponent));
    settle_triggers(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 12);

    declare_attack(&mut game, lifelink);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 6);
    assert_eq!(game.state.players[player.index()].life, 26);
}

#[test]
fn demonic_counsel_reveals_the_demon_but_delirium_finds_a_private_card() {
    let player = Player::P0;
    let opponent = player.other();
    for delirium in [false, true] {
        let mut game = scenario(player);
        put(&mut game, player, "Gingerbrute", Zone::Graveyard);
        put(&mut game, player, "Forest", Zone::Graveyard);
        if delirium {
            put(&mut game, player, "Opt", Zone::Graveyard);
        }
        let demon = put(&mut game, player, "Bloodletter of Aclazotz", Zone::Library);
        let land = put(&mut game, player, "Island", Zone::Library);
        let spell = put(&mut game, player, "Demonic Counsel", Zone::Hand);
        cast(&mut game, spell, None);
        pass_pair(&mut game);
        let picked = if delirium {
            reject(&mut game, Action::FinishSearch);
            land
        } else {
            reject(&mut game, Action::ChooseSearch { card: land });
            let mut fail_to_find = game.clone();
            act(&mut fail_to_find, Action::FinishSearch);
            assert_eq!(fail_to_find.object_view(demon).zone, Zone::Library);
            demon
        };
        act(&mut game, Action::ChooseSearch { card: picked });
        act(&mut game, Action::FinishSearch);
        assert_eq!(game.object_view(picked).zone, Zone::Hand);
        assert_eq!(
            game.observe(opponent)
                .known_hand
                .contains(&game.object_ref(picked)),
            !delirium
        );
    }
}

#[test]
fn cynical_loner_cannot_be_blocked_by_a_glimmer_and_searches_in_the_second_main() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let loner = put(&mut game, player, "Cynical Loner", Zone::Hand);
    let glimmer = put(&mut game, opponent, "Enduring Curiosity", Zone::Battlefield);
    let normal = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let searched = put(&mut game, player, "Island", Zone::Library);
    step(&mut game, Step::Main1);
    cast(&mut game, loner, None);
    settle_triggers(&mut game);
    assert_eq!(stats(&game, loner), (3, 1));
    game.state.objects[loner as usize].summoning_sick = false;
    declare_attack(&mut game, loner);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: glimmer,
            attacker: loner,
        },
    );
    assert!(game.legal_actions().contains(&Action::Block {
        blocker: normal,
        attacker: loner
    }));
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::Main2);
    assert!(game.object_view(loner).tapped);

    let mut untapped = game.clone();
    untapped.state.objects[loner as usize].tapped = false;
    settle_triggers(&mut untapped);
    assert_eq!(untapped.object_view(searched).zone, Zone::Library);

    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: true });
    reject(&mut game, Action::FinishSearch);
    act(&mut game, Action::ChooseSearch { card: searched });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(searched).zone, Zone::Graveyard);
}
