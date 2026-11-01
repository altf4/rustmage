//! Ordinary printed outcomes for FRA abilities and casting choices.
use super::*;

fn jace(g: &Game) -> CardId {
    g.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.card == Card::JacePlaneswalkerToken)
                .then_some(i as CardId)
        })
        .unwrap()
}

#[test]
fn ajani_resolute_loyalty_modes_and_lifegain_trigger() {
    for ability in 0..3 {
        let mut g = game();
        let own = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let ajani = put(&mut g, Player::P0, "Ajani Resolute", Zone::Hand);
        cast(&mut g, ajani, &[]);
        resolve(&mut g);
        g.state.objects[ajani as usize].counters.loyalty = [2, 5, 11][ability as usize];
        activate(&mut g, ajani, ability, &[]);
        resolve(&mut g);
        match ability {
            0 => {
                assert_eq!(g.state.players[0].life, 21);
                assert_eq!(g.counter_count(ajani, CounterKind::Loyalty), 3);
            }
            1 => {
                let mate = g
                    .observe(Player::P0)
                    .objects
                    .into_iter()
                    .find(|o| {
                        o.zone == Zone::Battlefield
                            && o.is_token()
                            && o.card == Card::from_name("@token/fra-ajani-pridemate").unwrap()
                    })
                    .unwrap();
                assert_eq!((mate.power, mate.toughness), (2, 2));
                g.gain_life(Player::P0, 1);
                resolve(&mut g);
                assert_eq!(g.counter_count(mate.id, CounterKind::PlusOne), 1);
            }
            2 => {
                assert_eq!(stats(&g, own), (4, 4));
                assert_eq!(stats(&g, enemy), (2, 2));
            }
            _ => unreachable!(),
        }
        covered(&format!("card/111/loyalty/{ability}"));
    }
}

#[test]
fn ajani_unrelenting_plus_one_pumps_grants_haste_and_creates_a_cadet() {
    let mut g = game();
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let ajani = put(&mut g, Player::P0, "Ajani Unrelenting", Zone::Hand);
    cast(&mut g, ajani, &[]);
    resolve(&mut g);
    activate(&mut g, ajani, 0, &[]);
    resolve(&mut g);
    assert_eq!(stats(&g, bear), (3, 2));
    assert!(g.keywords(bear).contains(Keywords::HASTE));
    let cadets: Vec<_> = g
        .observe(Player::P0)
        .objects
        .into_iter()
        .filter(|o| o.zone == Zone::Battlefield && o.is_token() && o.name == "Cadet")
        .collect();
    assert_eq!(cadets.len(), 1);
    assert_eq!((cadets[0].power, cadets[0].toughness), (3, 2));
    g.finish_cleanup();
    assert_eq!(stats(&g, bear), (2, 2));
}

#[test]
fn hall_of_echoes_produces_colorless_and_copies_a_legend_without_losing_either() {
    let mut g = game();
    let hall = put(&mut g, Player::P0, "Hall of Echoes", Zone::Battlefield);
    let legend = put(
        &mut g,
        Player::P0,
        "Caelorna, Coral Tyrant",
        Zone::Battlefield,
    );
    g.state.players[0].mana = [0; 6];
    choose(&mut g, |a| {
        matches!(a,Action::TapMana {card} if *card==hall)
            || matches!(a,Action::ActivateMana {card,production:[0,0,0,0,0,1],..} if *card==hall)
    });
    assert_eq!(g.state.players[0].mana, [0, 0, 0, 0, 0, 1]);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 5];
    activate(&mut g, hall, 1, &[Target::Card(legend)]);
    resolve(&mut g);
    assert_eq!(g.object_name(hall), "Caelorna, Coral Tyrant");
    assert!(g.is_creature(hall));
    assert!(g.on_battlefield(hall) && g.on_battlefield(legend));
    assert!(g.state.players[0].legend_rule_ignored_this_turn);
    g.finish_cleanup();
    assert_eq!(g.object_name(hall), "Hall of Echoes");
    assert!(!g.is_creature(hall));
    assert!(!g.state.players[0].legend_rule_ignored_this_turn);
}

#[test]
fn machinations_empowers_eight_and_permits_a_loyalty_activation_on_the_opponents_turn() {
    let mut g = game();
    let spell = put(&mut g, Player::P0, "Jace's Machinations", Zone::Hand);
    cast(&mut g, spell, &[]);
    resolve(&mut g);
    let source = jace(&g);
    assert_eq!(g.counter_count(source, CounterKind::Loyalty), 8);
    g.state.active = Player::P1;
    g.state.priority = Player::P0;
    g.state.step = Step::BeginCombat;
    activate(&mut g, source, 1, &[]);
    resolve(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    g.finish_cleanup();
    assert!(!g.state.players[0].jace_loyalty_anytime);
}

#[test]
fn solitary_cell_exiles_until_departure_and_discards_a_legend_to_draw() {
    let mut g = game();
    let bear = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let cell = put(&mut g, Player::P0, "Solitary Cell", Zone::Hand);
    cast(&mut g, cell, &[]);
    let target = Action::ChooseTarget {
        target: Target::Card(bear),
    };
    reach(&mut g, &target);
    act(&mut g, target);
    resolve(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Exile);
    let legend = put(&mut g, Player::P0, "Caelorna, Coral Tyrant", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    act(
        &mut g,
        Action::AnnounceActivate {
            card: cell,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = g.object_ref(legend);
    act(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    resolve(&mut g);
    assert!(g.object_view(cell).tapped);
    assert_eq!(g.object_view(legend).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 1];
    let disenchant = put(&mut g, Player::P0, "Disenchant", Zone::Hand);
    cast(&mut g, disenchant, &[Target::Card(cell)]);
    resolve(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert_eq!(g.object_view(bear).controller, Player::P1);
}

#[test]
fn vanguard_enters_prepared_and_seed_suture_places_a_counter_and_gains_life() {
    let mut g = game();
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let source = put(
        &mut g,
        Player::P0,
        "Vigorbloom Vanguard // Seed Suture",
        Zone::Hand,
    );
    cast(&mut g, source, &[]);
    resolve(&mut g);
    let reference = g.object_ref(source);
    assert!(g.is_prepared(reference));
    let copy = g
        .state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.permanent == reference)
        .unwrap()
        .spell_copy;
    g.state.players[0].mana = [0, 0, 0, 0, 1, 0];
    act(&mut g, Action::AnnounceExile { card: copy.card });
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        act(&mut g, Action::ChooseSpellCost { cost: 0 });
    }
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    resolve(&mut g);
    assert!(!g.is_prepared(reference));
    assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
    assert_eq!(g.state.players[0].life, 21);
    assert!(g.keywords(bear).contains(Keywords::VIGILANCE));
}

#[test]
fn way_of_the_mind_sculptor_empowers_five_and_draws_for_a_real_loyalty_payment() {
    let mut g = game();
    let way = put(&mut g, Player::P0, "Way of the Mind Sculptor", Zone::Hand);
    cast(&mut g, way, &[]);
    resolve(&mut g);
    let source = jace(&g);
    assert_eq!(g.counter_count(source, CounterKind::Loyalty), 5);
    activate(&mut g, source, 1, &[]);
    resolve(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(g.counter_count(source, CounterKind::Loyalty), 2);
}

#[test]
fn craftwork_crusher_executes_all_three_pairs_of_modes() {
    for mode in 0..3 {
        let mut g = game();
        let turtle = put(&mut g, Player::P1, "Aegis Turtle", Zone::Battlefield);
        let crusher = put(&mut g, Player::P0, "Craftwork Crusher", Zone::Hand);
        cast(&mut g, crusher, &[]);
        until(&mut g, |g| matches!(g.state.pending, Pending::Modes { .. }));
        act(&mut g, Action::ChooseMode { mode });
        if mode != 2 {
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(turtle),
                },
            );
        }
        resolve(&mut g);
        assert_eq!(g.object_view(turtle).damage, if mode == 2 { 0 } else { 4 });
        let cadets = g
            .observe(Player::P0)
            .objects
            .into_iter()
            .filter(|o| o.zone == Zone::Battlefield && o.is_token() && o.name == "Cadet")
            .count();
        assert_eq!(cadets, usize::from(mode != 1));
        assert_eq!(
            g.state.players[0].cards_drawn_this_turn,
            u64::from(mode != 0)
        );
        covered(&format!("card/877/entry_pair/{mode}"));
    }
}

#[test]
fn gallia_exiles_another_graveyard_creature_to_return_tapped_with_a_counter() {
    let mut g = game();
    let gallia = put(&mut g, Player::P0, "Gallia, Tragic Host", Zone::Graveyard);
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    g.state.players[0].mana = [0, 0, 1, 0, 0, 4];
    act(
        &mut g,
        Action::AnnounceActivate {
            card: gallia,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = g.object_ref(bear);
    act(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    resolve(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Exile);
    assert_eq!(g.object_view(gallia).zone, Zone::Battlefield);
    assert!(g.object_view(gallia).tapped);
    assert_eq!(g.counter_count(gallia, CounterKind::PlusOne), 1);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn ingris_attack_triggers_damage_each_opponent_before_combat_damage() {
    let mut g = game();
    let ingris = put(&mut g, Player::P0, "Ingris Stingerquill", Zone::Battlefield);
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: ingris });
    act(&mut g, Action::Attack { card: bear });
    act(&mut g, Action::FinishAttackers);
    resolve(&mut g);
    assert_eq!(g.state.players[1].life, 18);
}

#[test]
fn null_summoner_exiles_a_nonland_and_casts_it_at_threshold_with_other_mana() {
    let mut g = game();
    let spell = put(&mut g, Player::P1, "Spectral Sailor", Zone::Hand);
    let source = put(&mut g, Player::P0, "Null Summoner", Zone::Hand);
    cast(&mut g, source, &[]);
    let target = Action::ChooseTarget {
        target: Target::Player(Player::P1),
    };
    reach(&mut g, &target);
    act(&mut g, target);
    let choice = Action::ChooseHandCard {
        object: Some(g.object_ref(spell)),
    };
    reach(&mut g, &choice);
    act(&mut g, choice);
    resolve(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Exile);
    assert!(!g.can_play_exiled(Player::P0, spell));
    for _ in 0..7 {
        put(&mut g, Player::P0, "Forest", Zone::Graveyard);
    }
    g.state.players[0].mana = [0, 0, 0, 1, 0, 0];
    act(&mut g, Action::AnnounceExile { card: spell });
    pay(&mut g);
    resolve(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(g.object_view(spell).controller, Player::P0);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn sphinxs_approach_draws_two_and_exiles_four_to_search_for_a_sphinx() {
    let mut g = game();
    let graves: Vec<_> = (0..4)
        .map(|_| put(&mut g, Player::P0, "Sphinx's Approach", Zone::Graveyard))
        .collect();
    let sphinx = put(
        &mut g,
        Player::P0,
        "Sphinx of the Final Word",
        Zone::Library,
    );
    put(&mut g, Player::P0, "Island", Zone::Library);
    put(&mut g, Player::P0, "Island", Zone::Library);
    let spell = put(&mut g, Player::P0, "Sphinx's Approach", Zone::Hand);
    cast(&mut g, spell, &[]);
    let first = Action::SelectResolutionExile {
        object: g.object_ref(graves[0]),
    };
    reach(&mut g, &first);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    for &id in &graves {
        let object = g.object_ref(id);
        act(&mut g, Action::SelectResolutionExile { object });
    }
    for id in graves {
        assert_eq!(g.object_view(id).zone, Zone::Exile);
    }
    assert_eq!(g.object_view(spell).zone, Zone::Exile);
    act(&mut g, Action::ChooseSearch { card: sphinx });
    act(&mut g, Action::FinishSearch);
    resolve(&mut g);
    assert_eq!(g.object_view(sphinx).zone, Zone::Battlefield);
}

#[test]
fn sphinxs_approach_can_decline_exile_and_keep_the_drawn_cards() {
    let mut g = game();
    for _ in 0..4 {
        put(&mut g, Player::P0, "Sphinx's Approach", Zone::Graveyard);
    }
    let spell = put(&mut g, Player::P0, "Sphinx's Approach", Zone::Hand);
    cast(&mut g, spell, &[]);
    reach(&mut g, &Action::DeclineEffectPayment);
    act(&mut g, Action::DeclineEffectPayment);
    resolve(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.hand_count(Player::P0), 2);
}
