//! Independent rules proofs for the previously admitted green and multicolor identities.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn advance(game: &mut Game, until: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if until(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("scenario did not reach the requested state");
}

fn order(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

fn targets(game: &mut Game, target: Target) {
    order(game);
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn activate(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    advance(game, |g| g.state.pending == Pending::Attackers);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn tokens(game: &Game, player: Player, subtype: &str) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == player
                && o.is_token()
                && o.has_subtype(subtype)
        })
        .collect()
}

#[test]
fn meltstriders_resolve_fights_and_limits_the_accepted_blockers_to_one() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foe = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let other_foes = ["Hill Giant", "Savannah Lions"]
        .map(|name| put(&mut game, player.other(), name, Zone::Battlefield));
    let aura = put(&mut game, player, "Meltstrider's Resolve", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(creature)));
    passes(&mut game);
    order(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::FinishTargets);
    settle(&mut declined);
    assert_eq!(declined.object_view(foe).zone, Zone::Battlefield);
    targets(&mut game, Target::Card(foe));
    settle(&mut game);
    assert_eq!(game.object_view(foe).zone, Zone::Graveyard);
    assert_eq!(stats(&game, creature), (2, 4));
    attack(&mut game, &[creature]);
    advance(&mut game, |g| g.state.pending == Pending::Blockers);
    act(
        &mut game,
        Action::Block {
            blocker: other_foes[0],
            attacker: creature,
        },
    );
    reject(
        &mut game,
        Action::Block {
            blocker: other_foes[1],
            attacker: creature,
        },
    );
}

#[test]
fn mightform_doubles_the_targets_live_power_and_uses_the_later_exile_permission() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Mightform Harmonizer", Zone::Hand);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::AnnounceWarp { card: source });
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(source).cast_for_warp);
    act(&mut game, Action::PlayLand { card: land });
    order(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    targets(&mut game, Target::Card(creature));
    game.add_counter(creature, CounterKind::PlusOne, 1);
    settle(&mut game);
    assert_eq!(stats(&game, creature), (6, 3));
    let turn = game.state.turn;
    advance(&mut game, |g| {
        g.state.active == player && g.state.turn > turn && g.state.step == Step::Main1
    });
    assert_eq!(stats(&game, creature), (3, 3));
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    game.state.players[player.index()].mana = [8; 6];
    act(&mut game, Action::AnnounceExile { card: source });
    pay(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().origin,
        Zone::Exile
    );
    settle(&mut game);
    assert!(!game.object_view(source).cast_for_warp);
}

#[test]
fn samis_curiosity_and_agrarian_create_landers_and_the_searched_land_grows_agrarian() {
    let player = Player::P0;
    let mut game = scenario(player);
    let agrarian = put(&mut game, player, "Seedship Agrarian", Zone::Battlefield);
    let ship = put(&mut game, player, "Wurmwall Sweeper", Zone::Battlefield);
    let spell = put(&mut game, player, "Sami's Curiosity", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(tokens(&game, player, "Lander").len(), 1);
    activate(&mut game, ship, 0);
    let object = game.object_ref(agrarian);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    order(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 3);
    let landers = tokens(&game, player, "Lander");
    assert_eq!(landers.len(), 2);
    activate(&mut game, landers[0].id, 0);
    pay(&mut game);
    passes(&mut game);
    let basic = *game.state.players[player.index()].library.last().unwrap();
    act(&mut game, Action::ChooseSearch { card: basic });
    act(&mut game, Action::FinishSearch);
    order(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
    assert!(game.object_view(basic).tapped);
    assert_eq!(stats(&game, agrarian), (4, 4));
}

#[test]
fn shattered_wings_checks_the_flyer_artifact_or_enchantment_union_and_surveils() {
    let player = Player::P0;
    let mut game = scenario(player);
    let flyer = put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    let ground = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let artifact = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let enchantment = put(
        &mut game,
        player.other(),
        "Phyrexian Arena",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Shattered Wings", Zone::Hand);
    let card = *game.state.players[player.index()].library.last().unwrap();
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(ground),
        },
    );
    for victim in [flyer, artifact, enchantment] {
        let mut branch = game.clone();
        targets(&mut branch, Target::Card(victim));
        pay(&mut branch);
        passes(&mut branch);
        assert_eq!(branch.object_view(victim).zone, Zone::Graveyard);
        assert!(branch.observe(player.other()).library_choice.is_none());
        act(
            &mut branch,
            Action::ArrangeLibrary {
                card,
                destination: LibraryDestination::Graveyard,
            },
        );
        assert_eq!(branch.object_view(card).zone, Zone::Graveyard);
    }
}

#[test]
fn thawbringer_surveils_on_entry_and_death_but_not_exile() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Thawbringer", Zone::Hand);
    cast(&mut game, source, None);
    passes(&mut game);
    order(&mut game);
    passes(&mut game);
    let first = *game.state.players[player.index()].library.last().unwrap();
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: first,
            destination: LibraryDestination::Graveyard,
        },
    );
    let mut exile = game.clone();
    let spell = put(&mut exile, player, "Gravkill", Zone::Hand);
    cast(&mut exile, spell, Some(Target::Card(source)));
    settle(&mut exile);
    assert!(exile.observe(player).library_choice.is_none());
    let spell = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(source)));
    passes(&mut game);
    order(&mut game);
    passes(&mut game);
    let second = *game.state.players[player.index()].library.last().unwrap();
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: second,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_ne!(first, second);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
}

#[test]
fn biomechan_and_biotech_land_on_entry_and_the_artifact_sacrifice_damages_the_opponent() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let engineer = put(&mut game, player, "Biomechan Engineer", Zone::Hand);
    let specialist = put(&mut game, player, "Biotech Specialist", Zone::Hand);
    for card in [engineer, specialist] {
        cast(&mut game, card, None);
        settle(&mut game);
    }
    let landers = tokens(&game, player, "Lander");
    assert_eq!(landers.len(), 2);
    activate(&mut game, landers[0].id, 0);
    pay(&mut game);
    order(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    targets(&mut game, Target::Player(opponent));
    passes(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    passes(&mut game);
    let basic = *game.state.players[player.index()].library.last().unwrap();
    act(&mut game, Action::ChooseSearch { card: basic });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    let before = game.state.players[player.index()].library.len();
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 8];
    activate(&mut game, engineer, 0);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].library.len(), before - 2);
    assert_eq!(tokens(&game, player, "Robot").len(), 1);
}

#[test]
fn cosmogoyf_counts_cards_owned_in_exile_and_updates_when_control_changes() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Cosmogoyf", Zone::Hand);
    put(&mut game, player, "Island", Zone::Exile);
    let second = put(&mut game, player, "Forest", Zone::Exile);
    put(&mut game, player.other(), "Mountain", Zone::Exile);
    assert_eq!(stats(&game, source), (2, 3));
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (2, 3));
    game.move_object(second, Zone::Hand);
    assert_eq!(stats(&game, source), (1, 2));
    put(&mut game, player.other(), "Forest", Zone::Exile);
    let exact = game.object_ref(source);
    game.change_controller(exact, player.other());
    assert_eq!(stats(&game, source), (2, 3));
}

#[test]
fn genemorph_replaces_base_stats_at_six_lands_without_removing_counters() {
    let player = Player::P0;
    for (lands, power) in [(4, 4), (5, 7)] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Genemorph Imago", Zone::Battlefield);
        let victim = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        game.add_counter(victim, CounterKind::PlusOne, 1);
        for _ in 0..lands {
            put(&mut game, player, "Forest", Zone::Battlefield);
        }
        let entering = put(&mut game, player, "Island", Zone::Hand);
        assert!(game.object_view(source).keywords.contains(Keywords::FLYING));
        act(&mut game, Action::PlayLand { card: entering });
        targets(&mut game, Target::Card(victim));
        settle(&mut game);
        assert_eq!(stats(&game, victim), (power, power));
    }
}

#[test]
fn ascendant_cadet_marks_at_entry_and_attack_and_draws_once_for_a_shared_damage_event() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let haliya = put(&mut game, player, "Haliya, Ascendant Cadet", Zone::Hand);
    cast(&mut game, haliya, None);
    passes(&mut game);
    targets(&mut game, Target::Card(first));
    settle(&mut game);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 1);
    game.state.objects[haliya as usize].summoning_sick = false;
    attack(&mut game, &[first, second, haliya]);
    targets(&mut game, Target::Card(second));
    settle(&mut game);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 1);
    let library = game.state.players[player.index()].library.len();
    advance(&mut game, |g| {
        g.state.players[player.index()].library.len() < library
    });
    assert_eq!(
        game.state.players[player.index()].library.len(),
        library - 1
    );
    assert_eq!(game.state.players[player.other().index()].life, 11);
}

#[test]
fn uthros_exile_marks_either_players_creature_only_when_its_controllers_artifact_enters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Mm'menon, Uthros Exile",
        Zone::Battlefield,
    );
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    assert!(game.object_view(source).keywords.contains(Keywords::FLYING));
    let spell = put(&mut game, player, "Nutrient Block", Zone::Hand);
    cast(&mut game, spell, None);
    passes(&mut game);
    targets(&mut game, Target::Card(victim));
    settle(&mut game);
    assert_eq!(game.counter_count(victim, CounterKind::PlusOne), 1);
    let entering = put(&mut game, player.other(), "Nutrient Block", Zone::Graveyard);
    game.move_object(entering, Zone::Battlefield);
    assert!(game.state.pending_triggers.is_empty());
}

#[test]
fn ships_engineer_makes_a_tapped_robot_at_the_qualified_own_end_step() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Sami, Ship's Engineer",
        Zone::Battlefield,
    );
    let creatures =
        ["Grizzly Bears", "Hill Giant"].map(|name| put(&mut game, player, name, Zone::Battlefield));
    let mut no_second = game.clone();
    no_second.state.objects[creatures[0] as usize].tapped = true;
    for creature in creatures {
        game.state.objects[creature as usize].tapped = true;
    }
    for (mut branch, expected) in [(game, 1), (no_second, 0)] {
        let turn = branch.state.turn;
        advance(&mut branch, |g| g.state.turn != turn);
        let robots = tokens(&branch, player, "Robot");
        assert_eq!(robots.len(), expected);
        assert!(
            robots
                .iter()
                .all(|robot| robot.tapped && (robot.power, robot.toughness) == (2, 2))
        );
    }
}

#[test]
fn wildcat_captain_reduces_generic_for_each_artifact_and_strikes_twice_without_tapping() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sami = put(
        &mut game,
        player,
        "Sami, Wildcat Captain",
        Zone::Battlefield,
    );
    for artifact in ["Nutrient Block", "Swiftfoot Boots"] {
        put(&mut game, player, artifact, Zone::Battlefield);
    }
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    cast(&mut game, spell, None);
    let paid = game.state.stack.last().unwrap().cast.unwrap().payment;
    assert_eq!(paid, [0, 1, 0, 0, 0, 0]);
    settle(&mut game);
    attack(&mut game, &[sami]);
    assert!(!game.object_view(sami).tapped);
    advance(&mut game, |g| {
        g.state.players[player.other().index()].life <= 12
    });
    assert_eq!(game.state.players[player.other().index()].life, 12);
}

#[test]
fn space_time_anomaly_uses_its_controllers_live_life_total_for_the_target_player() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    game.state.players[player.index()].life = 6;
    let spell = put(&mut game, player, "Space-Time Anomaly", Zone::Hand);
    let response = put(&mut game, opponent, "Lightning Strike", Zone::Hand);
    let library = game.state.players[opponent.index()].library.len();
    cast(&mut game, spell, Some(Target::Player(opponent)));
    act(&mut game, Action::Pass);
    cast(&mut game, response, Some(Target::Player(player)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 3);
    assert_eq!(
        game.state.players[opponent.index()].library.len(),
        library - 3
    );
}

#[test]
fn lucent_vondam_boosts_other_creatures_at_entry_and_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let source = put(&mut game, player, "Syr Vondam, the Lucent", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, ally), (3, 2));
    assert!(
        game.object_view(ally)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::DEATHTOUCH.union(Keywords::LIFELINK))
    );
    game.state.objects[source as usize].summoning_sick = false;
    attack(&mut game, &[source]);
    order(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, ally), (4, 2));
    assert_eq!(stats(&game, source), (4, 4));
    assert_eq!(stats(&game, foreign), (2, 2));
}
