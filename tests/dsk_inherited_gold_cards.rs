//! Paid scenarios for the admitted green and multicolored Duskmourn cards.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OTHER: Player = Player::P1;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn attack(game: &mut Game, attacker: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
}

fn finish_search(game: &mut Game, selected: CardId, invalid: CardId) {
    reject(game, Action::ChooseSearch { card: invalid });
    act(game, Action::ChooseSearch { card: selected });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(game, Action::FinishSearch);
    }
    assert_eq!(game.object_view(selected).zone, Zone::Hand);
    assert!(
        game.observe(OTHER)
            .known_hand
            .contains(&game.object_ref(selected))
    );
}

#[test]
fn baseball_bat_attaches_at_entry_and_taps_a_target_when_its_new_bearer_attacks() {
    let mut game = scenario(PLAYER);
    let equipment = put(&mut game, PLAYER, "Baseball Bat", Zone::Hand);
    let first = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let opponent = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    cast(&mut game, equipment, None);
    pass_pair(&mut game);
    reject(&mut game, target(opponent));
    act(&mut game, target(first));
    settle(&mut game);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(first))
    );
    assert_eq!(stats(&game, first), (3, 3));

    game.state.players[PLAYER.index()].mana = [0, 0, 0, 0, 0, 3];
    act(&mut game, activation(equipment, 0));
    act(&mut game, target(second));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(second))
    );
    assert_eq!(stats(&game, first), (2, 2));
    attack(&mut game, second);
    let mut decline = game.clone();
    act(&mut decline, Action::FinishTargets);
    settle(&mut decline);
    assert!(!decline.object_view(opponent).tapped);
    act(&mut game, target(opponent));
    settle(&mut game);
    assert!(game.object_view(opponent).tapped);
}

#[test]
fn flesh_burrower_and_the_two_combat_buffs_grant_their_exact_keywords() {
    let mut game = scenario(PLAYER);
    let insect = put(&mut game, PLAYER, "Flesh Burrower", Zone::Battlefield);
    let bear = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    assert!(game.keywords(insect).contains(Keywords::DEATHTOUCH));
    attack(&mut game, insect);
    reject(&mut game, target(insect));
    reject(&mut game, target(enemy));
    act(&mut game, target(bear));
    settle(&mut game);
    assert!(game.keywords(bear).contains(Keywords::DEATHTOUCH));

    let mut game = scenario(OTHER);
    let bear = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let aura = put(&mut game, PLAYER, "Frantic Strength", Zone::Hand);
    let trick = put(&mut game, PLAYER, "Horrid Vigor", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, aura, Some(Target::Card(bear)));
    settle(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(bear))
    );
    assert_eq!(stats(&game, bear), (4, 4));
    assert!(game.keywords(bear).contains(Keywords::TRAMPLE));
    act(&mut game, Action::Pass);
    cast(&mut game, trick, Some(Target::Card(bear)));
    settle(&mut game);
    assert!(
        game.keywords(bear)
            .contains(Keywords::DEATHTOUCH.union(Keywords::INDESTRUCTIBLE))
    );
}

#[test]
fn grasping_longneck_and_wary_watchdog_resolve_their_actual_entry_and_death_effects() {
    let mut game = scenario(PLAYER);
    let watchdog = put(&mut game, PLAYER, "Wary Watchdog", Zone::Hand);
    let longneck = put(&mut game, PLAYER, "Grasping Longneck", Zone::Battlefield);
    let wrath = put(&mut game, PLAYER, "Day of Judgment", Zone::Hand);
    let top = [
        put(&mut game, PLAYER, "Shock", Zone::Library),
        put(&mut game, PLAYER, "Opt", Zone::Library),
    ];
    assert!(game.keywords(longneck).contains(Keywords::REACH));
    cast(&mut game, watchdog, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(PLAYER).library_choice.unwrap().surveil);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top[1],
            destination: LibraryDestination::Graveyard,
        },
    );

    cast(&mut game, wrath, None);
    for _ in 0..12 {
        if game.observe(PLAYER).library_choice.is_some() {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert!(game.observe(OTHER).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top[0],
            destination: LibraryDestination::Graveyard,
        },
    );
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].life, 22);
    assert_eq!(game.object_view(watchdog).zone, Zone::Graveyard);
    assert_eq!(game.object_view(longneck).zone, Zone::Graveyard);
    assert!(
        top.into_iter()
            .all(|card| game.object_view(card).zone == Zone::Graveyard)
    );
}

#[test]
fn insidious_fungus_pays_the_sacrifice_and_executes_all_three_modes() {
    let mut base = scenario(PLAYER);
    let fungus = put(&mut base, PLAYER, "Insidious Fungus", Zone::Battlefield);
    let artifact = put(&mut base, OTHER, "Bear Trap", Zone::Battlefield);
    let enchantment = put(&mut base, OTHER, "Nowhere to Run", Zone::Battlefield);
    let land = put(&mut base, PLAYER, "Island", Zone::Hand);
    let draw = put(&mut base, PLAYER, "Shock", Zone::Library);
    base.state.players[PLAYER.index()].mana = [0, 0, 0, 0, 0, 2];
    for (mode, victim, invalid) in [
        (0, Some(artifact), enchantment),
        (1, Some(enchantment), artifact),
        (2, None, land),
    ] {
        let mut game = base.clone();
        act(&mut game, activation(fungus, 0));
        act(&mut game, Action::ChooseMode { mode });
        if let Some(victim) = victim {
            reject(&mut game, target(invalid));
            act(&mut game, target(victim));
        }
        pay(&mut game);
        assert_eq!(game.object_view(fungus).zone, Zone::Graveyard);
        assert_eq!(game.state.players[PLAYER.index()].mana, [0; 6]);
        pass_pair(&mut game);
        if let Some(victim) = victim {
            assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
            assert_eq!(game.object_view(invalid).zone, Zone::Battlefield);
        } else {
            assert_eq!(game.object_view(draw).zone, Zone::Hand);
            let mut decline = game.clone();
            act(&mut decline, Action::DeclineHandEntry);
            assert_eq!(decline.object_view(land).zone, Zone::Hand);
            let object = game.object_ref(land);
            act(&mut game, Action::PutHandLand { object });
            assert_eq!(game.object_view(land).zone, Zone::Battlefield);
            assert!(game.object_view(land).tapped);
        }
    }
}

#[test]
fn peer_past_the_veil_counts_types_including_its_actual_discard_batch() {
    let mut game = scenario(PLAYER);
    let spell = put(&mut game, PLAYER, "Peer Past the Veil", Zone::Hand);
    let old = ["Friendly Teddy", "Nowhere to Run", "Forest"]
        .map(|name| put(&mut game, PLAYER, name, Zone::Hand));
    cast(&mut game, spell, None);
    settle(&mut game);
    assert!(
        old.into_iter()
            .all(|card| game.object_view(card).zone == Zone::Graveyard)
    );
    assert_eq!(game.hand_count(PLAYER), 4);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 4);
}

const EXTRA_MAIN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CombatAndMainAfterCurrentMain(1)],
    ),
    ..CardRules::EMPTY
};

#[test]
fn cautious_survivor_and_shrewd_storyteller_trigger_only_on_the_second_main() {
    let extra = Card::from_name("Full Throttle").unwrap();
    let _fixture = fixture_rules::activate(&[(extra, &EXTRA_MAIN)]);
    for name in ["Cautious Survivor", "Shrewd Storyteller"] {
        let mut game = scenario(PLAYER);
        let survivor = put(&mut game, PLAYER, name, Zone::Battlefield);
        let target_card = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
        let grant = spawn(&mut game, PLAYER, extra, Zone::Hand);
        game.begin_step(Step::Main1);
        cast(&mut game, grant, None);
        settle(&mut game);
        attack(&mut game, survivor);
        for _ in 0..60 {
            if game.state.step == Step::Main2 {
                break;
            }
            choose(&mut game, |action| {
                matches!(action, Action::Pass | Action::FinishBlockers)
            });
        }
        assert_eq!(game.state.step, Step::Main2);
        if name == "Shrewd Storyteller" {
            act(&mut game, target(target_card));
        }
        settle(&mut game);
        let life = if name == "Cautious Survivor" { 22 } else { 20 };
        let counters = u64::from(name == "Shrewd Storyteller");
        assert_eq!(game.state.players[PLAYER.index()].life, life);
        assert_eq!(
            game.counter_count(target_card, CounterKind::PlusOne),
            counters
        );

        for _ in 0..60 {
            if game.state.step == Step::End {
                break;
            }
            choose(&mut game, |action| {
                matches!(
                    action,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            });
        }
        assert_eq!(game.state.step, Step::End);
        assert!(game.object_view(survivor).tapped);
        assert_eq!(game.state.players[PLAYER.index()].life, life);
        assert_eq!(
            game.counter_count(target_card, CounterKind::PlusOne),
            counters
        );
    }
}

#[test]
fn shroudstomper_drains_and_draws_at_both_entry_and_attack() {
    let mut game = scenario(PLAYER);
    let horror = put(&mut game, PLAYER, "Shroudstomper", Zone::Hand);
    cast(&mut game, horror, None);
    settle(&mut game);
    assert!(game.keywords(horror).contains(Keywords::DEATHTOUCH));
    assert_eq!(game.state.players[PLAYER.index()].life, 22);
    assert_eq!(game.state.players[OTHER.index()].life, 18);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 1);
    game.state.objects[horror as usize].summoning_sick = false;
    attack(&mut game, horror);
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].life, 24);
    assert_eq!(game.state.players[OTHER.index()].life, 16);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 2);
}

#[test]
fn forestcycling_reveals_nonbasic_forests_and_centipedes_entry_reveals_only_basics() {
    let mut game = scenario(PLAYER);
    let branchsnapper = put(&mut game, PLAYER, "Slavering Branchsnapper", Zone::Hand);
    let centipede = put(&mut game, PLAYER, "Spineseeker Centipede", Zone::Hand);
    let dual = put(&mut game, PLAYER, "Commercial District", Zone::Library);
    let basic = put(&mut game, PLAYER, "Island", Zone::Library);
    let mut normal = game.clone();
    cast(&mut normal, branchsnapper, None);
    settle(&mut normal);
    assert!(normal.keywords(branchsnapper).contains(Keywords::TRAMPLE));

    let mut cycling = game.clone();
    act(&mut cycling, activation(branchsnapper, 0));
    pay(&mut cycling);
    pass_pair(&mut cycling);
    assert_eq!(cycling.object_view(branchsnapper).zone, Zone::Graveyard);
    finish_search(&mut cycling, dual, basic);

    cast(&mut game, centipede, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    finish_search(&mut game, basic, dual);
    let wickerfolk = put(&mut game, PLAYER, "Wildfire Wickerfolk", Zone::Battlefield);
    put(&mut game, PLAYER, "Friendly Teddy", Zone::Graveyard);
    put(&mut game, PLAYER, "Shock", Zone::Graveyard);
    assert_eq!(stats(&game, centipede), (2, 1));
    assert_eq!(stats(&game, wickerfolk), (3, 2));
    assert!(game.keywords(wickerfolk).contains(Keywords::HASTE));
    let fourth = put(&mut game, PLAYER, "Forest", Zone::Graveyard);
    assert_eq!(stats(&game, centipede), (3, 3));
    assert!(game.keywords(centipede).contains(Keywords::VIGILANCE));
    assert_eq!(stats(&game, wickerfolk), (4, 3));
    assert!(game.keywords(wickerfolk).contains(Keywords::TRAMPLE));
    game.move_object(fourth, Zone::Exile);
    assert_eq!(stats(&game, centipede), (2, 1));
    assert_eq!(stats(&game, wickerfolk), (3, 2));
}

fn pay_if_needed(game: &mut Game) {
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn kaito_enters_attacking_via_ninjutsu_and_has_three_distinct_loyalty_routes() {
    let mut combat = scenario(PLAYER);
    let attacker = put(&mut combat, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let reserve = put(&mut combat, PLAYER, "Bear Cub", Zone::Battlefield);
    let kaito = put(&mut combat, PLAYER, "Kaito, Bane of Nightmares", Zone::Hand);
    attack(&mut combat, attacker);
    pass_pair(&mut combat);
    act(&mut combat, Action::FinishBlockers);
    combat.state.players[PLAYER.index()].mana = [0, 1, 1, 0, 0, 1];
    act(&mut combat, activation(kaito, 1));
    let other = combat.object_ref(reserve);
    reject(&mut combat, Action::ChooseAdditionalCost { object: other });
    let returner = combat.object_ref(attacker);
    act(
        &mut combat,
        Action::ChooseAdditionalCost { object: returner },
    );
    pay(&mut combat);
    assert_eq!(combat.object_view(attacker).zone, Zone::Hand);
    settle(&mut combat);
    assert_eq!(combat.object_view(kaito).zone, Zone::Battlefield);
    assert!(combat.object_view(kaito).tapped);
    assert!(combat.state.attackers.iter().any(|row| row.card == kaito));
    assert_eq!(stats(&combat, kaito), (3, 4));
    assert_eq!(combat.counter_count(kaito, CounterKind::Loyalty), 4);

    let mut base = scenario(PLAYER);
    let kaito = put(&mut base, PLAYER, "Kaito, Bane of Nightmares", Zone::Hand);
    let victim = put(&mut base, OTHER, "Hill Giant", Zone::Battlefield);
    let shock = put(&mut base, PLAYER, "Shock", Zone::Hand);
    cast(&mut base, kaito, None);
    settle(&mut base);
    assert!(base.object_view(kaito).has_subtype("Ninja"));
    assert!(base.keywords(kaito).contains(Keywords::HEXPROOF));

    let mut plus = base.clone();
    act(&mut plus, activation(kaito, 0));
    pay_if_needed(&mut plus);
    settle(&mut plus);
    assert_eq!(plus.counter_count(kaito, CounterKind::Loyalty), 5);
    assert_eq!(stats(&plus, kaito), (4, 5));
    assert!(
        plus.observe(PLAYER)
            .objects
            .iter()
            .any(|object| object.zone == Zone::Command && object.emblem.is_some())
    );
    reject(&mut plus, activation(kaito, 2));
    plus.begin_step(Step::End);
    pass_pair(&mut plus);
    assert_eq!(plus.state.active, OTHER);
    assert!(!plus.object_view(kaito).is_creature());
    assert!(!plus.keywords(kaito).contains(Keywords::HEXPROOF));

    let mut minus = base.clone();
    act(&mut minus, activation(kaito, 3));
    act(&mut minus, target(victim));
    pay_if_needed(&mut minus);
    settle(&mut minus);
    assert_eq!(minus.counter_count(kaito, CounterKind::Loyalty), 2);
    assert_eq!(minus.counter_count(victim, CounterKind::Stun), 2);
    assert!(minus.object_view(victim).tapped);

    for lost_life in [false, true] {
        let mut game = base.clone();
        if lost_life {
            cast(&mut game, shock, Some(Target::Player(OTHER)));
            settle(&mut game);
        }
        act(&mut game, activation(kaito, 2));
        pay_if_needed(&mut game);
        pass_pair(&mut game);
        assert!(game.observe(PLAYER).library_choice.unwrap().surveil);
        assert!(game.observe(OTHER).library_choice.is_none());
        for _ in 0..2 {
            choose(&mut game, |action| {
                matches!(
                    action,
                    Action::ArrangeLibrary {
                        destination: LibraryDestination::Top,
                        ..
                    }
                )
            });
        }
        assert_eq!(
            game.state.players[PLAYER.index()].cards_drawn_this_turn,
            u64::from(lost_life)
        );
    }
}
