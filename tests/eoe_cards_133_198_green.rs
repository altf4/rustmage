//! Paid green card scenarios for the last twelve EOE 133–198 admissions.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const LANDER: Card = Card::from_catalogue_id(5238);

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishAttackers
                    | Action::FinishBlockers
            )
        });
    }
    panic!("scenario did not reach its stop condition");
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn activate(game: &mut Game, source: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

fn landers(game: &Game) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.is_token() && object.card == LANDER)
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn bioengineered_future_counts_lands_from_before_its_own_entry_and_the_lander_search() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first_land = put(&mut game, player, "Forest", Zone::Hand);
    let searched = put(&mut game, player, "Plains", Zone::Library);
    let future = put(&mut game, player, "Bioengineered Future", Zone::Hand);
    let already = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    act(&mut game, Action::PlayLand { card: first_land });
    cast(&mut game, future, None);
    settle(&mut game);
    let [lander] = landers(&game).try_into().unwrap();

    let first = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, first, None);
    settle(&mut game);
    assert_eq!(game.counter_count(already, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 1);

    activate(&mut game, lander);
    pay(&mut game);
    resolve_top(&mut game);
    act(&mut game, Action::ChooseSearch { card: searched });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    settle(&mut game);
    assert_eq!(game.object_view(searched).zone, Zone::Battlefield);
    assert!(game.object_view(searched).tapped);
    assert_ne!(game.object_view(lander).zone, Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, second, None);
    settle(&mut game);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 1);
}

#[test]
fn biosynthic_burst_adds_a_counter_grants_three_keywords_and_untaps_its_own_target() {
    let player = Player::P1;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.state.objects[own as usize].tapped = true;
    let spell = put(&mut game, player, "Biosynthic Burst", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    target(&mut game, Target::Card(own));
    pay(&mut game);
    settle(&mut game);
    let result = game.object_view(own);
    assert!(!result.tapped);
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 1);
    assert!(
        result.keywords.contains(
            Keywords::REACH
                .union(Keywords::TRAMPLE)
                .union(Keywords::INDESTRUCTIBLE)
        )
    );
}

#[test]
fn broodguard_elite_warps_with_x_and_transfers_all_counters_when_it_leaves() {
    let player = Player::P0;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Broodguard Elite", Zone::Hand);
    let recipient = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let wrong = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    act(&mut game, Action::AnnounceWarp { card });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(card).cast_for_warp);
    assert_eq!(game.counter_count(card, CounterKind::PlusOne), 2);
    game.add_counter(card, CounterKind::Charge, 3);

    until(&mut game, |game| game.state.step == Step::End);
    resolve_top(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(wrong),
        },
    );
    target(&mut game, Target::Card(recipient));
    settle(&mut game);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(recipient, CounterKind::Charge), 3);
}

fn start_encounter(game: &mut Game, spell: CardId, recipient: CardId) {
    act(game, Action::AnnounceCast { card: spell });
    target(game, Target::Card(recipient));
    pay(game);
    choose(game, |action| {
        matches!(action, Action::ChooseCastCost { .. })
    });
}

#[test]
fn close_encounter_uses_live_chosen_power_or_an_exact_same_turn_warp_exile() {
    let player = Player::P0;
    let mut game = scenario(player);
    let foe = put(
        &mut game,
        player.other(),
        "Shivan Dragon",
        Zone::Battlefield,
    );
    let donor = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Close Encounter", Zone::Hand);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);

    start_encounter(&mut game, spell, foe);
    let wrong = game.object_ref(foreign);
    reject(&mut game, Action::ChooseAdditionalCost { object: wrong });
    let object = game.object_ref(donor);
    act(&mut game, Action::ChooseAdditionalCost { object });
    assert_eq!(game.object_ref(donor), object);
    cast(&mut game, growth, Some(Target::Card(donor)));
    settle(&mut game);
    assert_eq!(game.object_view(foe).zone, Zone::Graveyard);

    let mut game = scenario(player);
    let warped = put(&mut game, player, "Possibility Technician", Zone::Hand);
    let ordinary = put(&mut game, player, "Possibility Technician", Zone::Exile);
    let recipient = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Close Encounter", Zone::Hand);
    let mana = [
        put(&mut game, player, "Forest", Zone::Battlefield),
        put(&mut game, player, "Forest", Zone::Battlefield),
    ];
    act(&mut game, Action::AnnounceWarp { card: warped });
    pay(&mut game);
    settle(&mut game);
    let turn = game.state.turn;
    until(&mut game, |game| {
        game.object_view(warped).zone == Zone::Exile
    });
    assert_eq!(game.state.turn, turn);
    for card in mana {
        act(&mut game, Action::TapMana { card });
    }
    start_encounter(&mut game, spell, recipient);
    let wrong = game.object_ref(ordinary);
    reject(&mut game, Action::ChooseAdditionalCost { object: wrong });
    let object = game.object_ref(warped);
    act(&mut game, Action::ChooseAdditionalCost { object });
    settle(&mut game);
    assert_eq!(game.object_ref(warped), object);
    assert_eq!(game.object_view(recipient).zone, Zone::Graveyard);
}

#[test]
fn diplomatic_relations_grows_only_your_creature_before_one_way_damage() {
    let player = Player::P1;
    for bounce in [false, true] {
        let mut game = scenario(player);
        let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Shivan Dragon",
            Zone::Battlefield,
        );
        let spell = put(&mut game, player, "Diplomatic Relations", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        target(&mut game, Target::Card(own));
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        target(&mut game, Target::Card(enemy));
        pay(&mut game);
        if bounce {
            let spell = put(&mut game, player, "Unsummon", Zone::Hand);
            cast(&mut game, spell, Some(Target::Card(enemy)));
        }
        settle(&mut game);
        let creature = game.object_view(own);
        assert_eq!(
            (creature.power, creature.toughness, creature.damage),
            (3, 2, 0)
        );
        assert!(creature.keywords.contains(Keywords::VIGILANCE));
        let enemy = game.object_view(enemy);
        assert!(!enemy.keywords.contains(Keywords::VIGILANCE));
        assert_eq!(enemy.damage, if bounce { 0 } else { 3 });
    }
}

#[test]
fn edge_rover_creates_one_lander_for_each_player_only_when_it_dies() {
    let player = Player::P0;
    for exile in [false, true] {
        let mut game = scenario(player);
        let rover = put(&mut game, player.other(), "Edge Rover", Zone::Battlefield);
        assert!(game.object_view(rover).keywords.contains(Keywords::REACH));
        let name = if exile { "Exorcise" } else { "Shock" };
        let removal = put(&mut game, player, name, Zone::Hand);
        cast(&mut game, removal, Some(Target::Card(rover)));
        settle(&mut game);
        let tokens = landers(&game);
        assert_eq!(tokens.len(), if exile { 0 } else { 2 });
        if !exile {
            for owner in [player, player.other()] {
                assert_eq!(
                    tokens
                        .iter()
                        .filter(|&&id| game.object_view(id).controller == owner)
                        .count(),
                    1
                );
            }
        }
    }
}

#[test]
fn famished_worldsire_devours_lands_looks_privately_and_has_ward_three() {
    let player = Player::P0;
    let mut game = scenario(player);
    let donor = put(&mut game, player, "Forest", Zone::Battlefield);
    let keep = put(&mut game, player, "Island", Zone::Battlefield);
    let wrong = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let land1 = put(&mut game, player, "Plains", Zone::Library);
    let nonland = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let land2 = put(&mut game, player, "Mountain", Zone::Library);
    let worldsire = put(&mut game, player, "Famished Worldsire", Zone::Hand);
    cast(&mut game, worldsire, None);
    resolve_top(&mut game);
    let object = game.object_ref(wrong);
    reject(&mut game, Action::ChooseEntryDevour { object });
    let object = game.object_ref(donor);
    act(&mut game, Action::ChooseEntryDevour { object });
    act(&mut game, Action::FinishEntryDevour);
    assert_eq!(game.object_view(donor).zone, Zone::Graveyard);
    assert_eq!(game.object_view(keep).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(worldsire, CounterKind::PlusOne), 3);
    resolve_top(&mut game);
    let look = game.observe(player).library_selection.unwrap();
    assert_eq!(look.remaining.len(), 3);
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: nonland });
    for card in [land1, land2] {
        act(&mut game, Action::SelectLibrary { card });
    }
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    settle(&mut game);
    for id in [land1, land2] {
        let land = game.object_view(id);
        assert_eq!(land.zone, Zone::Battlefield);
        assert!(land.tapped);
    }
    assert_eq!(game.object_view(nonland).zone, Zone::Library);

    let shock = put(&mut game, player.other(), "Shock", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Card(worldsire)));
    resolve_top(&mut game);
    assert_eq!(game.actor(), Some(player.other()));
    reject(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 2, 0, 0],
        },
    );
    assert!(game.legal_actions().contains(&Action::PayResolutionMana {
        payment: [0, 0, 0, 3, 0, 0]
    }));
    act(&mut game, Action::DeclineResolutionMana);
    settle(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Graveyard);
    assert_eq!(game.object_view(worldsire).damage, 0);
}

#[test]
fn frenzied_baloth_protects_creature_spells_and_only_disables_combat_prevention() {
    let player = Player::P1;
    let mut game = scenario(player);
    let baloth = put(&mut game, player, "Frenzied Baloth", Zone::Hand);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    for card in [baloth, creature] {
        let counter = put(&mut game, player.other(), "Essence Scatter", Zone::Hand);
        cast(&mut game, card, None);
        act(&mut game, Action::Pass);
        cast(&mut game, counter, Some(Target::Spell(card)));
        settle(&mut game);
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }
    assert!(
        game.object_view(baloth)
            .keywords
            .contains(Keywords::HASTE.union(Keywords::TRAMPLE))
    );

    let blocker = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    game.add_counter(blocker, CounterKind::Shield, 1);
    until(&mut game, |game| game.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    until(&mut game, |game| game.state.pending == Pending::Blockers);
    act(&mut game, Action::Block { blocker, attacker });
    act(&mut game, Action::FinishBlockers);
    until(&mut game, |game| game.state.step == Step::Main2);
    assert_eq!(game.object_view(blocker).damage, 2);
    assert_eq!(game.counter_count(blocker, CounterKind::Shield), 0);

    game.add_counter(blocker, CounterKind::Shield, 1);
    act(&mut game, Action::TapMana { card: mountain });
    cast(&mut game, shock, Some(Target::Card(blocker)));
    settle(&mut game);
    assert_eq!(game.object_view(blocker).zone, Zone::Battlefield);
    assert_eq!(game.object_view(blocker).damage, 2);
    assert_eq!(game.counter_count(blocker, CounterKind::Shield), 0);
}

#[test]
fn hemosymbic_mite_targets_another_controlled_creature_and_samples_current_power() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mite = put(&mut game, player, "Hemosymbic Mite", Zone::Battlefield);
    let friend = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foe = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    until(&mut game, |game| game.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: mite });
    act(&mut game, Action::FinishAttackers);
    for id in [mite, foe] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    target(&mut game, Target::Card(friend));
    act(&mut game, Action::TapMana { card: forest });
    cast(&mut game, growth, Some(Target::Card(mite)));
    settle(&mut game);
    assert!(game.object_view(mite).tapped);
    assert_eq!(stats(&game, friend), (6, 6));
}

#[test]
fn larval_scoutlander_accepts_land_or_lander_and_stations_to_seven() {
    let player = Player::P0;
    for sacrifice_token in [false, true] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Larval Scoutlander", Zone::Hand);
        let land = put(&mut game, player, "Forest", Zone::Battlefield);
        let wrong = put(&mut game, player, "Goldvein Pick", Zone::Battlefield);
        let donors = [
            put(&mut game, player, "Shivan Dragon", Zone::Battlefield),
            put(&mut game, player, "Grizzly Bears", Zone::Battlefield),
        ];
        let targets = [
            put(&mut game, player, "Swamp", Zone::Library),
            put(&mut game, player, "Mountain", Zone::Library),
        ];
        game.create_tokens(player, LANDER, 1, false);
        assert!(!game.advance_entries());
        let [lander] = landers(&game).try_into().unwrap();
        cast(&mut game, source, None);
        resolve_top(&mut game);
        resolve_top(&mut game);

        let mut decline = game.clone();
        act(&mut decline, Action::DeclineEffectPayment);
        assert!(
            targets
                .iter()
                .all(|&id| decline.object_view(id).zone == Zone::Library)
        );
        let object = game.object_ref(wrong);
        reject(&mut game, Action::Sacrifice { object });
        let object = game.object_ref(if sacrifice_token { lander } else { land });
        act(&mut game, Action::Sacrifice { object });
        for card in targets {
            act(&mut game, Action::ChooseSearch { card });
        }
        if game.legal_actions().contains(&Action::FinishSearch) {
            act(&mut game, Action::FinishSearch);
        }
        settle(&mut game);
        assert_ne!(game.object_view(object.card).zone, Zone::Battlefield);
        for card in targets {
            let view = game.object_view(card);
            assert_eq!(view.zone, Zone::Battlefield);
            assert!(view.tapped);
        }
        for (donor, charge) in donors.into_iter().zip([5, 7]) {
            activate(&mut game, source);
            let object = game.object_ref(donor);
            act(&mut game, Action::ChooseAdditionalCost { object });
            pay(&mut game);
            settle(&mut game);
            assert_eq!(game.counter_count(source, CounterKind::Charge), charge);
            assert_eq!(game.object_view(source).is_creature(), charge == 7);
        }
        assert_eq!(stats(&game, source), (3, 3));
        assert!(game.object_view(source).keywords.contains(Keywords::FLYING));
    }
}

#[test]
fn lashwhip_predator_needs_three_opposing_creatures_for_two_mana_off() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Lashwhip Predator", Zone::Hand);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    for _ in 0..2 {
        put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
    }
    game.state.players[player.index()].mana = [0, 0, 0, 0, 4, 0];
    reject(&mut game, Action::AnnounceCast { card: spell });
    put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert!(game.object_view(spell).keywords.contains(Keywords::REACH));
}

#[test]
fn meltstrider_eulogist_uses_predeath_controller_and_the_named_counter() {
    let player = Player::P1;
    for exile in [false, true] {
        let mut game = scenario(player);
        put(&mut game, player, "Meltstrider Eulogist", Zone::Battlefield);
        let marked = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let other_counter = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        for id in [marked, enemy] {
            game.add_counter(id, CounterKind::PlusOne, 1);
        }
        game.add_counter(other_counter, CounterKind::Stun, 1);
        if exile {
            put(&mut game, player, "Rest in Peace", Zone::Battlefield);
        }
        let wipe = put(&mut game, player, "Day of Judgment", Zone::Hand);
        cast(&mut game, wipe, None);
        settle(&mut game);
        assert_eq!(game.hand_count(player), usize::from(!exile));
    }
}
