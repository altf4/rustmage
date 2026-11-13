use crate::{reach, support::*};
use rustmage_core::{cards::*, engine::*};

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn begin_attack(game: &mut Game) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
}

fn choose_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn token_count(game: &Game, subtype: &str) -> usize {
    (0..game.state.objects.len())
        .map(|id| game.object_view(id as CardId))
        .filter(|object| {
            object.is_token() && object.zone == Zone::Battlefield && object.has_subtype(subtype)
        })
        .count()
}

#[test]
fn noncreature_casts_grow_boar_q_pine_and_cunning_maneuver_creates_a_clue() {
    let player = Player::P0;
    let mut game = scenario(player);
    let porcupine = put(&mut game, player, "Boar-q-pine", Zone::Battlefield);
    let creature = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, creature, None);
    settle(&mut game);
    assert_eq!(game.counter_count(porcupine, CounterKind::PlusOne), 0);
    let trick = put(&mut game, player, "Cunning Maneuver", Zone::Hand);
    cast(&mut game, trick, Some(Target::Card(porcupine)));
    settle(&mut game);
    assert_eq!(game.counter_count(porcupine, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, porcupine), (6, 4));
    assert_eq!(token_count(&game, "Clue"), 1);
}

#[test]
fn combustion_technique_and_firebending_lesson_use_their_full_damage_rules() {
    let player = Player::P1;
    let mut game = scenario(player);
    let opponent = player.other();
    let victim = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let technique = put(&mut game, player, "Combustion Technique", Zone::Hand);
    put(&mut game, player, "Firebending Lesson", Zone::Graveyard);
    cast(&mut game, technique, Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);

    let dragon = put(&mut game, opponent, "Shivan Dragon", Zone::Battlefield);
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Hand);
    for kicked in [false, true] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: lesson });
        act(&mut branch, Action::ChooseKicker { pay: kicked });
        choose_target(&mut branch, Target::Card(dragon));
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(
            branch.object_view(dragon).zone,
            if kicked {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        if !kicked {
            assert_eq!(branch.state.objects[dragon as usize].damage, 2);
        }
    }
    let bolt = put(&mut game, player, "Lightning Strike", Zone::Hand);
    cast(&mut game, bolt, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 17);
}

#[test]
fn fire_sages_and_rough_rhino_cavalry_produce_combat_mana_and_grow() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sages = put(&mut game, player, "Fire Sages", Zone::Battlefield);
    let rhino = put(&mut game, player, "Rough Rhino Cavalry", Zone::Battlefield);
    for card in [sages, rhino] {
        act(&mut game, activate(card, 0));
        pay(&mut game);
        settle(&mut game);
    }
    assert_eq!(stats(&game, sages), (3, 3));
    assert_eq!(stats(&game, rhino), (7, 7));
    assert!(game.keywords(rhino).contains(Keywords::TRAMPLE));
    reject(&mut game, activate(rhino, 0));
    begin_attack(&mut game);
    for card in [sages, rhino] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    reach(&mut game, |g| {
        g.state.stack.is_empty() && g.state.pending == Pending::Priority
    });
    assert_eq!(game.state.players[player.index()].combat_red_mana, 3);
}

#[test]
fn mongoose_lizard_uses_mountaincycling_or_its_menace_and_entry_damage() {
    let player = Player::P1;
    let mut game = scenario(player);
    let lizard = put(&mut game, player, "Mongoose Lizard", Zone::Hand);
    let dual = put(&mut game, player, "Steam Vents", Zone::Library);
    let mut cycle = game.clone();
    act(&mut cycle, activate(lizard, 0));
    pay(&mut cycle);
    reach(&mut cycle, |g| {
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: dual })
    });
    act(&mut cycle, Action::ChooseSearch { card: dual });
    act(&mut cycle, Action::FinishSearch);
    settle(&mut cycle);
    assert_eq!(cycle.object_view(lizard).zone, Zone::Graveyard);
    assert_eq!(cycle.object_view(dual).zone, Zone::Hand);
    cast(&mut game, lizard, None);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    choose_target(&mut game, Target::Player(player.other()));
    settle(&mut game);
    assert!(game.keywords(lizard).contains(Keywords::MENACE));
    assert_eq!(game.state.players[player.other().index()].life, 19);
}

#[test]
fn the_treetop_allies_trigger_the_protestors_and_raider_only_raids_after_an_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let raider = put(&mut game, player, "Fire Nation Raider", Zone::Hand);
    let fighters = put(&mut game, player, "Treetop Freedom Fighters", Zone::Hand);
    let protestor = put(&mut game, player, "Wartime Protestors", Zone::Battlefield);
    let mut no_raid = game.clone();
    cast(&mut no_raid, raider, None);
    settle(&mut no_raid);
    assert_eq!(token_count(&no_raid, "Clue"), 0);

    cast(&mut game, fighters, None);
    reach(&mut game, |g| {
        g.state.stack.is_empty() && g.state.pending == Pending::Priority
    });
    assert!(game.keywords(fighters).contains(Keywords::HASTE));
    assert!(game.keywords(protestor).contains(Keywords::HASTE));
    assert_eq!(game.counter_count(fighters, CounterKind::PlusOne), 1);
    let ally = (0..game.state.objects.len())
        .map(|id| game.object_view(id as CardId))
        .find(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.has_subtype("Ally")
        })
        .unwrap();
    assert_eq!((ally.power, ally.toughness, ally.colors), (2, 2, 1));
    assert!(game.keywords(ally.id).contains(Keywords::HASTE));
    begin_attack(&mut game);
    act(&mut game, Action::Attack { card: ally.id });
    act(&mut game, Action::FinishAttackers);
    for _ in 0..16 {
        if game.state.step == Step::Main2 && game.actor() == Some(player) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::FinishBlockers)
        });
    }
    assert_eq!(game.state.step, Step::Main2);
    let mountain = put(&mut game, player, "Mountain", Zone::Hand);
    act(&mut game, Action::PlayLand { card: mountain });
    act(&mut game, Action::TapMana { card: mountain });
    game.state.players[player.index()].mana[3] = 8;
    cast(&mut game, raider, None);
    settle(&mut game);
    assert_eq!(token_count(&game, "Clue"), 1);
}

#[test]
fn tiger_dillo_needs_another_power_four_creature() {
    let player = Player::P1;
    let mut game = scenario(player);
    let dillo = put(&mut game, player, "Tiger-Dillo", Zone::Battlefield);
    let smaller = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    begin_attack(&mut game);
    reject(&mut game, Action::Attack { card: dillo });
    game.add_counter(smaller, CounterKind::PlusOne, 1);
    act(&mut game, Action::Attack { card: dillo });
}

#[test]
fn twin_blades_flashes_in_attaches_gives_double_strike_and_later_equips() {
    let player = Player::P1;
    let mut game = scenario(player);
    let blades = put(&mut game, player, "Twin Blades", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.state.active = player.other();
    cast(&mut game, blades, None);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    choose_target(&mut game, Target::Card(first));
    settle(&mut game);
    assert_eq!(stats(&game, first), (3, 3));
    assert!(game.keywords(first).contains(Keywords::DOUBLE_STRIKE));
    game.state.active = player;
    game.state.priority = player;
    game.state.step = Step::Main1;
    act(&mut game, activate(blades, 0));
    choose_target(&mut game, Target::Card(second));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, first), (2, 2));
    assert_eq!(stats(&game, second), (3, 3));
    assert!(!game.keywords(second).contains(Keywords::DOUBLE_STRIKE));
}

#[test]
fn yuyan_archers_offer_an_optional_rummage_and_have_reach() {
    let player = Player::P0;
    let mut game = scenario(player);
    let archers = put(&mut game, player, "Yuyan Archers", Zone::Hand);
    let discard = put(&mut game, player, "Mountain", Zone::Hand);
    cast(&mut game, archers, None);
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::Discard { card: discard })
    });
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    settle(&mut decline);
    assert_eq!(decline.object_view(discard).zone, Zone::Hand);
    assert_eq!(
        decline.state.players[player.index()].cards_drawn_this_turn,
        0
    );
    act(&mut game, Action::Discard { card: discard });
    if game.legal_actions().contains(&Action::FinishEffectChoice) {
        act(&mut game, Action::FinishEffectChoice);
    }
    settle(&mut game);
    assert!(game.keywords(archers).contains(Keywords::REACH));
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn zuko_plays_exiled_lands_this_turn_and_firebends_only_on_attack() {
    let player = Player::P1;
    let mut game = scenario(player);
    let zuko = put(&mut game, player, "Zuko, Exiled Prince", Zone::Battlefield);
    let land = put(&mut game, player, "Mountain", Zone::Library);
    act(&mut game, activate(zuko, 0));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 0);
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    begin_attack(&mut game);
    act(&mut game, Action::Attack { card: zuko });
    act(&mut game, Action::FinishAttackers);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 3);
}
