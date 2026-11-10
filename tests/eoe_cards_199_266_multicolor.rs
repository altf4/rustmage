//! Card-level proofs for multicolor effects, a land observer and the large Spacecraft.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn advance(game: &mut Game, until: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
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

fn order_trigger(game: &mut Game) {
    if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

fn announce_ability(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

#[test]
fn mutinous_massacre_destroys_the_chosen_parity_then_steals_and_untaps_survivors() {
    let player = Player::P0;
    let enemy = player.other();
    let mut game = scenario(player);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let even = put(&mut game, enemy, "Grizzly Bears", Zone::Battlefield);
    let odd = put(&mut game, enemy, "Savannah Lions", Zone::Battlefield);
    for id in [own, even, odd] {
        game.state.objects[id as usize].tapped = true;
    }
    let spell = put(&mut game, player, "Mutinous Massacre", Zone::Hand);
    cast(&mut game, spell, None);
    passes(&mut game);
    for (parity, dead, alive, allied_alive) in [
        (ManaValueParity::Odd, odd, even, true),
        (ManaValueParity::Even, even, odd, false),
    ] {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseManaValueParity { parity });
        assert_eq!(branch.object_view(dead).zone, Zone::Graveyard);
        let stolen = branch.object_view(alive);
        assert_eq!(stolen.controller, player);
        assert!(!stolen.tapped && stolen.keywords.contains(Keywords::HASTE));
        let ally = branch.object_view(own);
        assert_eq!(ally.zone == Zone::Battlefield, allied_alive);
        if allied_alive {
            assert!(!ally.tapped && ally.keywords.contains(Keywords::HASTE));
        }
        advance(&mut branch, |g| g.state.active == enemy);
        assert_eq!(branch.object_view(alive).controller, enemy);
        assert!(!branch.object_view(alive).keywords.contains(Keywords::HASTE));
    }
}

#[test]
fn ragost_adds_food_to_owned_artifacts_and_untaps_after_gaining_life() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let ragost = put(
        &mut game,
        player,
        "Ragost, Deft Gastronaut",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let second = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
    assert!(game.object_view(first).has_subtype("Food"));
    assert!(game.object_view(first).has_subtype("Equipment"));
    assert!(!game.object_view(foreign).has_subtype("Food"));
    let grantor = game.object_ref(ragost);
    let source = game.object_ref(first);
    let foreign = game.object_ref(foreign);
    reject(
        &mut game,
        Action::AnnounceArtifactFood {
            source: foreign,
            grantor,
            ability: 0,
        },
    );
    act(
        &mut game,
        Action::AnnounceArtifactFood {
            source,
            grantor,
            ability: 0,
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 23);
    announce_ability(&mut game, ragost, 1);
    let object = game.object_ref(second);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 17);
    assert!(game.object_view(ragost).tapped);
    advance(&mut game, |g| g.state.active == opponent);
    assert!(!game.object_view(ragost).tapped);
}

#[test]
fn seedship_broodtender_mills_three_and_returns_a_noncreature_spacecraft() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Seedship Broodtender", Zone::Hand);
    let ship = put(&mut game, player, "Warmaker Gunship", Zone::Graveyard);
    let wrong = put(&mut game, player, "Nutrient Block", Zone::Graveyard);
    let library = game.state.players[player.index()].library.len();
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        library - 3
    );
    announce_ability(&mut game, source, 0);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(wrong),
        },
    );
    target(&mut game, Target::Card(ship));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.object_view(ship).zone, Zone::Battlefield);
}

#[test]
fn singularity_rupture_destroys_all_creatures_even_without_mill_targets() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(
        &mut game,
        player.other(),
        "Nutrient Block",
        Zone::Battlefield,
    );
    put(&mut game, player, "Forest", Zone::Library);
    let spell = put(&mut game, player, "Singularity Rupture", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    for count in [0, 2] {
        let mut branch = game.clone();
        if count == 2 {
            for p in [player, player.other()] {
                target(&mut branch, Target::Player(p));
            }
        }
        if branch.legal_actions().contains(&Action::FinishTargets) {
            act(&mut branch, Action::FinishTargets);
        }
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.object_view(creature).zone, Zone::Graveyard);
        assert_eq!(branch.object_view(artifact).zone, Zone::Battlefield);
        assert_eq!(
            branch.state.players[player.index()].library.len(),
            if count == 2 { 5 } else { 9 }
        );
        assert_eq!(
            branch.state.players[player.other().index()].library.len(),
            if count == 2 { 4 } else { 8 }
        );
    }
}

#[test]
fn syr_vondam_uses_actual_death_or_exile_and_frozen_departure_power() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Syr Vondam, Sunstar Exemplar",
        Zone::Battlefield,
    );
    let bounce = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let dying = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let exiled = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let victim = put(
        &mut game,
        player.other(),
        "Warmaker Gunship",
        Zone::Battlefield,
    );
    let view = game.object_view(source);
    assert!(
        view.keywords
            .contains(Keywords::VIGILANCE.union(Keywords::MENACE))
    );
    let unsummon = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, unsummon, Some(Target::Card(bounce)));
    settle(&mut game);
    assert_eq!(stats(&game, source), (2, 2));
    for (id, effect) in [(dying, "Murder"), (exiled, "Gravkill")] {
        let spell = put(&mut game, player, effect, Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(id)));
        settle(&mut game);
    }
    assert_eq!(stats(&game, source), (4, 4));
    assert_eq!(game.state.players[player.index()].life, 22);
    let spell = put(&mut game, player, "Gravkill", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(source)));
    passes(&mut game);
    order_trigger(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    target(&mut game, Target::Card(victim));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
}

#[test]
fn tannuk_damages_on_every_land_entry_and_draws_on_exactly_the_second_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Tannuk, Memorial Ensign",
        Zone::Battlefield,
    );
    for _ in 0..2 {
        put(&mut game, player, "Icetill Explorer", Zone::Battlefield);
    }
    let library = game.state.players[player.index()].library.len();
    for entry in 1..=3 {
        let land = put(&mut game, player, "Forest", Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        advance(&mut game, |g| {
            g.state.stack.is_empty() && g.state.pending == Pending::Priority
        });
        assert_eq!(game.state.players[player.other().index()].life, 20 - entry);
        assert_eq!(
            game.state.players[player.index()].library.len(),
            library - 2 * entry as usize - usize::from(entry >= 2)
        );
    }
}

#[test]
fn dawnsire_triggers_at_ten_for_another_attacker_and_becomes_a_creature_at_twenty() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(
        &mut game,
        player,
        "Dawnsire, Sunstar Dreadnought",
        Zone::Battlefield,
    );
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(ship, CounterKind::Charge, 10);
    assert!(!game.object_view(ship).is_creature());
    advance(&mut game, |g| g.state.pending == Pending::Attackers);
    reject(&mut game, Action::Attack { card: ship });
    act(&mut game, Action::Attack { card: own });
    act(&mut game, Action::FinishAttackers);
    order_trigger(&mut game);
    target(&mut game, Target::Card(victim));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    game.set_counters(ship, Counters::default());
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    game.add_counter(ship, CounterKind::Charge, 20);
    assert_eq!(stats(&game, ship), (20, 20));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING));
}

#[test]
fn dominion_bracelet_equips_then_exiles_itself_with_the_equipped_power_discount() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let bracelet = put(
        &mut game,
        player,
        "The Dominion Bracelet",
        Zone::Battlefield,
    );
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let hidden = put(&mut game, opponent, "Island", Zone::Hand);
    announce_ability(&mut game, bracelet, 0);
    target(&mut game, Target::Card(creature));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, creature), (3, 3));
    let grantor = game.object_ref(bracelet);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 12];
    act(
        &mut game,
        Action::AnnounceAttachedAbility {
            grantor,
            ability: 1,
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    target(&mut game, Target::Player(opponent));
    pay(&mut game);
    assert_eq!(game.object_view(bracelet).zone, Zone::Exile);
    assert_eq!(stats(&game, creature), (2, 2));
    settle(&mut game);
    assert_eq!(
        game.observe(player).player_control.unwrap().pending[opponent.index()],
        Some(player)
    );
    assert!(!game.observe(player).objects.iter().any(|o| o.id == hidden));
    advance(&mut game, |g| g.state.active == opponent);
    assert!(game.observe(player).objects.iter().any(|o| o.id == hidden));
}

#[test]
fn endstone_observes_a_land_play_and_a_cast_then_sets_half_starting_life() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "The Endstone", Zone::Battlefield);
    let library = game.state.players[player.index()].library.len();
    let land = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    order_trigger(&mut game);
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        library - 1
    );
    let other = put(&mut game, player, "Forest", Zone::Graveyard);
    game.move_object(other, Zone::Battlefield);
    assert!(game.state.pending_triggers.is_empty());
    let spell = put(&mut game, player, "Nutrient Block", Zone::Hand);
    cast(&mut game, spell, None);
    order_trigger(&mut game);
    passes(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Stack);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        library - 2
    );
    settle(&mut game);
    let mut low = game.clone();
    low.state.players[player.index()].life = 4;
    for branch in [&mut game, &mut low] {
        advance(branch, |g| g.state.active != player);
        assert_eq!(branch.state.players[player.index()].life, 10);
    }
}

#[test]
fn eternity_elevator_produces_three_colorless_or_one_chosen_color_at_twenty() {
    let player = Player::P0;
    let mut game = scenario(player);
    let elevator = put(
        &mut game,
        player,
        "The Eternity Elevator",
        Zone::Battlefield,
    );
    game.state.players[player.index()].mana = [0; 6];
    game.add_counter(elevator, CounterKind::Charge, 19);
    let mana = |ability, production| Action::ActivateMana {
        card: elevator,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    };
    reject(&mut game, mana(2, [0, 0, 0, 0, 19, 0]));
    let mut ordinary = game.clone();
    act(&mut ordinary, mana(0, [0, 0, 0, 0, 0, 3]));
    assert_eq!(
        ordinary.state.players[player.index()].mana,
        [0, 0, 0, 0, 0, 3]
    );
    game.add_counter(elevator, CounterKind::Charge, 1);
    reject(&mut game, mana(2, [0, 0, 0, 0, 19, 1]));
    act(&mut game, mana(2, [0, 0, 0, 0, 20, 0]));
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 20, 0]);
    assert!(!game.object_view(elevator).is_creature());
}

#[test]
fn extinguisher_sweeps_creatures_even_if_its_legal_indestructible_target_survives() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "Extinguisher Battleship", Zone::Hand);
    let block = put(
        &mut game,
        player.other(),
        "Nutrient Block",
        Zone::Battlefield,
    );
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, ship, None);
    passes(&mut game);
    order_trigger(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    target(&mut game, Target::Card(block));
    let mut invalid = game.clone();
    invalid.move_object(block, Zone::Exile);
    settle(&mut invalid);
    assert_eq!(invalid.object_view(own).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(block).zone, Zone::Battlefield);
    for victim in [own, enemy] {
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    }
    game.add_counter(ship, CounterKind::Charge, 5);
    assert_eq!(stats(&game, ship), (10, 10));
    assert!(
        game.object_view(ship)
            .keywords
            .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
    );
}
