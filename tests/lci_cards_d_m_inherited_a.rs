//! Printed obligations of previously admitted D–G LCI cards.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, attacker: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
}

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn sacrifice_cast(game: &mut Game, spell: CardId, sacrifice: CardId, invalid: CardId) {
    act(game, Action::AnnounceCast { card: spell });
    let invalid = Action::ChooseAdditionalCost {
        object: game.object_ref(invalid),
    };
    let payment = Action::ChooseAdditionalCost {
        object: game.object_ref(sacrifice),
    };
    for _ in 0..8 {
        if game.state.announcing.is_none() {
            assert_eq!(game.object_view(sacrifice).zone, Zone::Graveyard);
            return;
        }
        match game.state.pending {
            Pending::AdditionalCost { .. } => {
                reject(game, invalid.clone());
                act(game, payment.clone());
            }
            Pending::PayCosts { .. } => pay(game),
            Pending::CastCosts { .. } => {
                choose(game, |action| {
                    matches!(action, Action::ChooseCastCost { .. })
                });
            }
            _ => panic!("expected a spell cost"),
        }
    }
    panic!("expected committed cast");
}

#[test]
fn descend_four_uses_physical_permanent_cards_for_each_inherited_static_effect() {
    let p = Player::P0;
    let mut game = scenario(p);
    for name in ["Forest", "Island", "Short Bow", "Shock"] {
        put(&mut game, p, name, Zone::Graveyard);
    }
    let echo = put(&mut game, p, "Echo of Dusk", Zone::Battlefield);
    let wurm = put(&mut game, p, "Frilled Cave-Wurm", Zone::Battlefield);
    let didact = put(&mut game, p, "Didact Echo", Zone::Hand);
    let top = game.top_library_card(p).unwrap();
    game.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
    cast(&mut game, didact, None);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert_eq!(game.hand_count(p), 1);
    assert!(!game.keywords(didact).contains(Keywords::FLYING));
    assert!(!game.keywords(echo).contains(Keywords::LIFELINK));
    assert_eq!(stats(&game, echo), (2, 2));
    assert_eq!(stats(&game, wurm), (2, 5));

    let fourth = put(&mut game, p, "Grizzly Bears", Zone::Library);
    game.move_object(fourth, Zone::Graveyard);
    assert!(game.keywords(didact).contains(Keywords::FLYING));
    assert!(game.keywords(echo).contains(Keywords::LIFELINK));
    assert_eq!(stats(&game, echo), (3, 3));
    assert_eq!(stats(&game, wurm), (4, 5));
    let mut removed = game.clone();
    removed.move_object(fourth, Zone::Exile);
    assert!(!removed.keywords(didact).contains(Keywords::FLYING));
    assert!(!removed.keywords(echo).contains(Keywords::LIFELINK));
    assert_eq!(stats(&removed, wurm), (2, 5));

    attack(&mut game, echo);
    for _ in 0..8 {
        if game.state.players[p.index()].life == STARTING_LIFE + 3 {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::FinishBlockers)
        });
    }
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE + 3);
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 3
    );
}

#[test]
fn dinotomaton_grants_temporary_menace_only_to_a_controlled_creature() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Dinotomaton", Zone::Hand);
    let friend = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    reject(&mut game, target(enemy));
    assert!(game.legal_actions().contains(&target(source)));
    act(&mut game, target(friend));
    settle(&mut game);
    assert!(game.keywords(source).contains(Keywords::MENACE));
    assert!(game.keywords(friend).contains(Keywords::MENACE));
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert!(game.keywords(source).contains(Keywords::MENACE));
    assert!(!game.keywords(friend).contains(Keywords::MENACE));
}

#[test]
fn disruptor_wanderglyph_attacks_to_exile_an_opponents_graveyard_card() {
    let p = Player::P0;
    let mut game = scenario(p);
    let glyph = put(&mut game, p, "Disruptor Wanderglyph", Zone::Battlefield);
    let own = put(&mut game, p, "Forest", Zone::Graveyard);
    let enemy = put(&mut game, p.other(), "Shock", Zone::Graveyard);
    attack(&mut game, glyph);
    reject(&mut game, target(own));
    act(&mut game, target(enemy));
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Exile);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
}

#[test]
fn dreadmaws_ire_rewards_combat_damage_with_only_the_damaged_players_artifact() {
    let p = Player::P0;
    let mut game = scenario(p);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let idle = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let own = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Basilisk Collar", Zone::Battlefield);
    let spell = put(&mut game, p, "Dreadmaw's Ire", Zone::Hand);
    attack(&mut game, creature);
    game.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(idle));
    act(&mut game, target(creature));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, creature), (4, 4));
    assert!(game.keywords(creature).contains(Keywords::TRAMPLE));
    for _ in 0..8 {
        if game.legal_actions().contains(&target(enemy)) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::FinishBlockers)
        });
    }
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 4
    );
    reject(&mut game, target(own));
    act(&mut game, target(enemy));
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(game.object_view(own).zone, Zone::Battlefield);
}

#[test]
fn dusk_rose_reliquary_sacrifices_a_cost_and_keeps_its_target_exiled_only_while_present() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Dusk Rose Reliquary", Zone::Hand);
    let material = put(&mut game, p, "Gleaming Barrier", Zone::Battlefield);
    let invalid = put(&mut game, p, "Forest", Zone::Battlefield);
    let opposing = put(&mut game, p.other(), "Basilisk Collar", Zone::Battlefield);
    let disenchant = put(&mut game, p.other(), "Disenchant", Zone::Hand);
    sacrifice_cast(&mut game, source, material, invalid);
    for _ in 0..16 {
        if game.legal_actions().contains(&target(opposing)) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(&mut game, target(opposing));
    let mut gone = game.clone();
    gone.move_object(source, Zone::Hand);
    settle(&mut gone);
    assert_eq!(gone.object_view(opposing).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(opposing).zone, Zone::Exile);

    game.state.players[p.other().index()].mana = [1, 0, 0, 0, 0, 1];
    act(&mut game, Action::Pass);
    cast(&mut game, disenchant, Some(Target::Card(source)));
    pass_pair(&mut game);
    if game
        .legal_actions()
        .contains(&Action::DeclineResolutionMana)
    {
        act(&mut game, Action::DeclineResolutionMana);
    }
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert_eq!(game.object_view(opposing).zone, Zone::Exile);
    game.move_object(source, Zone::Hand);
    assert_eq!(game.object_view(opposing).zone, Zone::Battlefield);
}

#[test]
fn eaten_by_piranhas_flashes_in_and_removes_old_types_and_abilities_without_touching_counters() {
    let p = Player::P0;
    let mut game = scenario(p);
    let aura = put(&mut game, p, "Eaten by Piranhas", Zone::Hand);
    let victim = put(&mut game, p.other(), "Gleaming Barrier", Zone::Battlefield);
    game.add_counter(victim, CounterKind::PlusOne, 1);
    game.state.active = p.other();
    game.state.step = Step::Upkeep;
    cast(&mut game, aura, Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(victim))
    );
    let view = game.object_view(victim);
    assert_eq!(view.kinds, 1);
    assert_eq!(view.colors, 4);
    assert!(view.has_subtype("Skeleton"));
    assert!(!view.has_subtype("Wall"));
    assert!(!game.keywords(victim).contains(Keywords::DEFENDER));
    assert_eq!(game.counter_count(victim, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, victim), (2, 2));
    game.move_object(aura, Zone::Hand);
    assert_eq!(game.object_view(victim).kinds, 1 | 16);
    assert!(game.keywords(victim).contains(Keywords::DEFENDER));
    assert_eq!(stats(&game, victim), (1, 5));
}

#[test]
fn envoy_of_okinec_ahau_pays_to_create_one_colorless_gnome_artifact_creature() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Envoy of Okinec Ahau", Zone::Battlefield);
    game.state.step = Step::Upkeep;
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
    reject(&mut game, activate(source));
    game.state.players[p.index()].mana = [1, 0, 0, 0, 0, 4];
    let token = game.state.objects.len() as CardId;
    act(&mut game, activate(source));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.objects.len(), token as usize + 1);
    let view = game.object_view(token);
    assert!(view.is_token() && view.has_subtype("Gnome"));
    assert_eq!(view.kinds, 1 | 16);
    assert_eq!(view.colors, 0);
    assert_eq!(stats(&game, token), (1, 1));
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn family_reunions_modes_grant_only_the_casting_players_team_and_expire() {
    let p = Player::P0;
    for mode in [0, 1] {
        let mut game = scenario(p);
        let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
        let opponent = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut game, p, "Family Reunion", Zone::Hand);
        let shock = put(&mut game, p.other(), "Shock", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseMode { mode });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, own), if mode == 0 { (3, 3) } else { (2, 2) });
        assert_eq!(stats(&game, opponent), (2, 2));
        assert_eq!(game.keywords(own).contains(Keywords::HEXPROOF), mode == 1);
        assert!(!game.keywords(opponent).contains(Keywords::HEXPROOF));
        if mode == 1 {
            let mut enemy_cast = game.clone();
            act(&mut enemy_cast, Action::Pass);
            act(&mut enemy_cast, Action::AnnounceCast { card: shock });
            reject(&mut enemy_cast, target(own));
            assert!(enemy_cast.legal_actions().contains(&target(opponent)));
        }
        game.state.step = Step::End;
        pass_pair(&mut game);
        assert_eq!(stats(&game, own), (2, 2));
        assert!(!game.keywords(own).contains(Keywords::HEXPROOF));
    }
}

#[test]
fn fanatical_offering_sacrifices_one_multitype_permanent_draws_twice_and_makes_a_map() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Fanatical Offering", Zone::Hand);
    let material = put(&mut game, p, "Gleaming Barrier", Zone::Battlefield);
    let invalid = put(&mut game, p, "Forest", Zone::Battlefield);
    let explorer = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, p, "Island", Zone::Library);
    let first = put(&mut game, p, "Opt", Zone::Library);
    let second = put(&mut game, p, "Shock", Zone::Library);
    sacrifice_cast(&mut game, spell, material, invalid);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert_eq!(game.object_view(second).zone, Zone::Hand);
    assert_eq!(game.object_view(land).zone, Zone::Library);
    let map = game
        .state
        .objects
        .iter()
        .position(|object| object.card == Card(5284) && object.zone == Zone::Battlefield)
        .unwrap() as CardId;
    act(&mut game, activate(map));
    act(&mut game, target(explorer));
    pay(&mut game);
    settle(&mut game);
    assert!(!game.on_battlefield(map));
    assert_eq!(game.object_view(land).zone, Zone::Hand);
}

#[test]
fn forest_and_island_play_untapped_and_standard_allows_unlimited_copies() {
    assert!(
        Game::new(Config {
            decks: [vec![Card::Forest; 60], vec![Card::Island; 60]],
            seed: 22,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .is_ok()
    );
    for (player, name, mana) in [
        (Player::P0, "Forest", [0, 0, 0, 0, 1, 0]),
        (Player::P1, "Island", [0, 1, 0, 0, 0, 0]),
    ] {
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        game.state.players[player.index()].mana = [0; 6];
        act(&mut game, Action::PlayLand { card: land });
        assert!(!game.object_view(land).tapped);
        act(&mut game, Action::TapMana { card: land });
        assert_eq!(game.state.players[player.index()].mana, mana);
    }
}

#[test]
fn gargantuan_leech_reduces_only_generic_cost_for_controlled_and_graveyard_caves() {
    let p = Player::P0;
    let mut game = scenario(p);
    let leech = put(&mut game, p, "Gargantuan Leech", Zone::Hand);
    let cave = put(&mut game, p, "Captivating Cave", Zone::Battlefield);
    put(&mut game, p, "Promising Vein", Zone::Graveyard);
    put(&mut game, p.other(), "Captivating Cave", Zone::Battlefield);
    game.state.objects[cave as usize].tapped = true;
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
    reject(&mut game, Action::AnnounceCast { card: leech });
    game.state.players[p.index()].mana = [0, 0, 1, 0, 0, 5];
    let mut too_few_caves = game.clone();
    too_few_caves.move_object(cave, Zone::Exile);
    reject(&mut too_few_caves, Action::AnnounceCast { card: leech });
    cast(&mut game, leech, None);
    settle(&mut game);
    assert_eq!(game.object_view(leech).zone, Zone::Battlefield);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
    assert!(game.keywords(leech).contains(Keywords::LIFELINK));
}

#[test]
fn get_lost_gives_maps_even_when_a_legal_target_is_indestructible() {
    let p = Player::P0;
    let mut game = scenario(p);
    let target_creature = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, p, "Get Lost", Zone::Hand);
    game.grant_keyword_effects(
        &[target_creature],
        game.object_ref(target_creature),
        Keywords::INDESTRUCTIBLE,
    );
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(artifact));
    act(&mut game, target(target_creature));
    pay(&mut game);
    let maps = |game: &Game| {
        game.state
            .objects
            .iter()
            .filter(|object| {
                object.card == Card(5284)
                    && object.zone == Zone::Battlefield
                    && object.controller == p.other()
            })
            .count()
    };
    let mut invalidated = game.clone();
    invalidated.move_object(target_creature, Zone::Hand);
    settle(&mut invalidated);
    assert_eq!(maps(&invalidated), 0);
    settle(&mut game);
    assert_eq!(game.object_view(target_creature).zone, Zone::Battlefield);
    assert_eq!(maps(&game), 2);
}
