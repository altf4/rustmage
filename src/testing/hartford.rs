//! Ordinary success paths for the Hartford card features.
use super::*;

#[test]
fn restless_reef_entry_mana_animation_and_attack_mill() {
    let mut g = game();
    let reef = put(&mut g, Player::P0, "Restless Reef", Zone::Hand);
    act(&mut g, Action::PlayLand { card: reef });
    assert!(g.object_view(reef).tapped);
    for color in [1, 2] {
        g.state.objects[reef as usize].tapped = false;
        g.state.players[0].mana = [0; 6];
        let mut production = [0; 6];
        production[color] = 1;
        choose(
            &mut g,
            |a| matches!(a, Action::ActivateMana {card, production: m, ..} if *card==reef && *m==production),
        );
        assert_eq!(g.state.players[0].mana, production);
    }
    g.state.objects[reef as usize].tapped = false;
    g.state.players[0].mana = [0, 1, 1, 0, 0, 2];
    activate(&mut g, reef, 2, &[]);
    resolve(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(stats(&g, reef), (4, 4));
    assert!(g.is_creature(reef));
    assert_ne!(g.object_view(reef).kinds & 2, 0);
    assert_eq!(g.object_view(reef).colors, 2 | 4);
    assert!(g.has_subtype(reef, "Shark"));
    assert!(g.keywords(reef).contains(Keywords::DEATHTOUCH));
    // Arrange the attack feature as a land controlled since the previous turn.
    g.state.objects[reef as usize].summoning_sick = false;
    let before = g.state.players[1].library.len();
    attack(&mut g, reef);
    let target = Action::ChooseTarget {
        target: Target::Player(Player::P1),
    };
    reach(&mut g, &target);
    act(&mut g, target);
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    resolve(&mut g);
    assert_eq!(g.state.players[1].library.len(), before - 4);
    g.finish_cleanup();
    assert!(!g.is_creature(reef));
}

#[test]
fn cryogen_relic_draws_on_entry_and_sacrifice_and_stuns_its_target() {
    let mut g = game();
    put(&mut g, Player::P1, "Vengeful Tracker", Zone::Battlefield);
    let target = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.objects[target as usize].tapped = true;
    let relic = put(&mut g, Player::P0, "Cryogen Relic", Zone::Hand);
    cast(&mut g, relic, &[]);
    resolve(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    activate(&mut g, relic, 0, &[Target::Card(target)]);
    resolve(&mut g);
    assert_eq!(g.object_view(relic).zone, Zone::Graveyard);
    assert_eq!(g.counter_count(target, CounterKind::Stun), 1);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(
        g.state.players[0].life, 18,
        "Tracker deals two for the opponent's artifact sacrifice"
    );
}

#[test]
fn outrageous_robbery_exiles_two_and_allows_land_and_any_color_spell_play() {
    let mut g = game();
    let land = put(&mut g, Player::P1, "Island", Zone::Library);
    let bear = put(&mut g, Player::P1, "Bear Cub", Zone::Library);
    let robbery = put(&mut g, Player::P0, "Outrageous Robbery", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: robbery });
    act(&mut g, Action::ChooseX { value: 2 });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    pay(&mut g);
    resolve(&mut g);
    for card in [land, bear] {
        assert_eq!(g.object_view(card).zone, Zone::Exile);
        assert!(g.observe(Player::P0).objects.iter().any(|o| o.id == card));
        assert!(!g.observe(Player::P1).objects.iter().any(|o| o.id == card));
    }
    act(&mut g, Action::PlayLand { card: land });
    g.state.players[0].mana = [0, 0, 0, 2, 0, 0];
    act(&mut g, Action::AnnounceExile { card: bear });
    pay(&mut g);
    resolve(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert_eq!(g.object_view(bear).controller, Player::P0);
    assert_eq!(g.object_view(land).controller, Player::P0);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn bloodghast_returns_on_landfall_and_gains_haste_at_ten_life() {
    let mut g = game();
    let bloodghast = put(&mut g, Player::P0, "Bloodghast", Zone::Graveyard);
    let land = put(&mut g, Player::P0, "Forest", Zone::Hand);
    act(&mut g, Action::PlayLand { card: land });
    let accept = Action::ChooseOptional { accept: true };
    reach(&mut g, &accept);
    act(&mut g, accept);
    resolve(&mut g);
    assert_eq!(g.object_view(bloodghast).zone, Zone::Battlefield);
    g.state.players[1].life = 11;
    assert!(!g.keywords(bloodghast).contains(Keywords::HASTE));
    g.state.players[1].life = 10;
    assert!(g.keywords(bloodghast).contains(Keywords::HASTE));
}

#[test]
fn sunspine_lynx_counts_nonbasic_lands_and_stops_lifegain_and_prevention() {
    let mut g = game();
    put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    put(&mut g, Player::P0, "Abraded Bluffs", Zone::Battlefield);
    put(&mut g, Player::P1, "Abraded Bluffs", Zone::Battlefield);
    put(&mut g, Player::P1, "Bristling Backwoods", Zone::Battlefield);
    let lynx = put(&mut g, Player::P0, "Sunspine Lynx", Zone::Hand);
    cast(&mut g, lynx, &[]);
    resolve(&mut g);
    assert_eq!([g.state.players[0].life, g.state.players[1].life], [19, 18]);
    g.gain_life(Player::P0, 3);
    assert_eq!(g.state.players[0].life, 19);
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let fog = put(&mut g, Player::P1, "Fog Bank", Zone::Battlefield);
    attack(&mut g, bear);
    let block = Action::Block {
        blocker: fog,
        attacker: bear,
    };
    reach(&mut g, &block);
    act(&mut g, block);
    act(&mut g, Action::FinishBlockers);
    until(&mut g, |g| g.object_view(fog).zone == Zone::Graveyard);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
}

#[test]
fn vren_exiles_opposing_deaths_and_makes_scaling_rats_at_end_step() {
    let mut g = game();
    put(
        &mut g,
        Player::P0,
        "Vren, the Relentless",
        Zone::Battlefield,
    );
    for _ in 0..2 {
        let bear = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let murder = put(&mut g, Player::P0, "Murder", Zone::Hand);
        cast(&mut g, murder, &[Target::Card(bear)]);
        resolve(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
    }
    g.begin_step(Step::End);
    resolve(&mut g);
    let rats: Vec<_> = g
        .observe(Player::P0)
        .objects
        .into_iter()
        .filter(|o| o.zone == Zone::Battlefield && o.is_token() && o.has_subtype("Rat"))
        .collect();
    assert_eq!(rats.len(), 2);
    assert!(rats.iter().all(|o| (o.power, o.toughness) == (3, 3)));
}

#[test]
fn fire_crystal_reduces_red_costs_grants_haste_and_copies_until_end_step() {
    let mut g = game();
    let crystal = put(&mut g, Player::P0, "The Fire Crystal", Zone::Battlefield);
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    assert!(g.keywords(bear).contains(Keywords::HASTE));
    let strike = put(&mut g, Player::P0, "Lightning Strike", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 1, 0, 0];
    cast(&mut g, strike, &[Target::Card(enemy)]);
    resolve(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
    g.state.players[0].mana = [0, 0, 0, 2, 0, 4];
    activate(&mut g, crystal, 0, &[Target::Card(bear)]);
    resolve(&mut g);
    assert!(g.object_view(crystal).tapped);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    let copies: Vec<_> = g
        .observe(Player::P0)
        .objects
        .into_iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.is_token()
                && o.card == Card::from_name("Bear Cub").unwrap()
        })
        .collect();
    assert_eq!(copies.len(), 1);
    assert_eq!((copies[0].power, copies[0].toughness), (2, 2));
    let copy = copies[0].id;
    g.begin_step(Step::End);
    resolve(&mut g);
    assert!(!g.on_battlefield(copy));
}

#[test]
fn tidebinder_counters_an_entry_trigger_and_blanks_its_source_while_present() {
    let mut g = game();
    let relic = put(&mut g, Player::P0, "Cryogen Relic", Zone::Hand);
    cast(&mut g, relic, &[]);
    until(&mut g, |g| {
        g.state.pending == Pending::Priority
            && g.state
                .stack
                .last()
                .is_some_and(|s| s.kind != StackKind::Spell)
    });
    let ability = g.state.stack.last().unwrap().id;
    if g.actor() != Some(Player::P1) {
        act(&mut g, Action::Pass);
    }
    let binder = put(&mut g, Player::P1, "Tishana's Tidebinder", Zone::Hand);
    cast(&mut g, binder, &[]);
    let target = Action::ChooseTarget {
        target: Target::Stack(ability),
    };
    reach(&mut g, &target);
    act(&mut g, target);
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    resolve(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    assert!(!g.printed_abilities_available(relic));
    let murder = put(&mut g, Player::P0, "Murder", Zone::Hand);
    cast(&mut g, murder, &[Target::Card(binder)]);
    resolve(&mut g);
    assert!(g.printed_abilities_available(relic));
}

#[test]
fn winternight_normal_cast_draws_three_then_discards_one_creature_or_two_cards() {
    for creature_route in [true, false] {
        let mut g = game();
        let spell = put(&mut g, Player::P0, "Winternight Stories", Zone::Hand);
        let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Hand);
        cast(&mut g, spell, &[]);
        let discard = Action::Discard { card: bear };
        reach(&mut g, &discard);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
        if !creature_route {
            act(&mut g, Action::DeclineEffectPayment);
        }
        act(&mut g, discard);
        if !creature_route {
            choose(&mut g, |a| matches!(a, Action::Discard { .. }));
        }
        resolve(&mut g);
        assert_eq!(g.hand_count(Player::P0), if creature_route { 3 } else { 2 });
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        covered(if creature_route {
            "card/5088/normal/creature"
        } else {
            "card/5088/normal/two"
        });
    }
}

#[test]
fn carnage_reanimates_a_creature_that_attacks_then_sacrifices_after_hitting_a_player() {
    let mut g = game();
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    let carnage = put(&mut g, Player::P0, "Carnage, Crimson Chaos", Zone::Hand);
    cast(&mut g, carnage, &[]);
    let target = Action::ChooseTarget {
        target: Target::Card(bear),
    };
    reach(&mut g, &target);
    act(&mut g, target);
    resolve(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert!(g.attacks_each_combat(bear));
    g.state.objects[bear as usize].summoning_sick = false;
    attack(&mut g, bear);
    reach(&mut g, &Action::FinishBlockers);
    act(&mut g, Action::FinishBlockers);
    until(&mut g, |g| g.object_view(bear).zone == Zone::Graveyard);
    assert_eq!(g.state.players[1].life, 18);
}

#[test]
fn carnage_mayhem_casts_for_black_red_after_discard() {
    let mut g = game();
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    let carnage = put(&mut g, Player::P0, "Carnage, Crimson Chaos", Zone::Hand);
    let thrill = put(&mut g, Player::P0, "Thrill of Possibility", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: thrill });
    let discard = Action::ChooseAdditionalCost {
        object: g.object_ref(carnage),
    };
    for _ in 0..16 {
        if g.legal_actions().contains(&discard) {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::PayCosts { .. } | Action::ChooseCastCost { .. })
        });
    }
    act(&mut g, discard);
    for _ in 0..16 {
        if g.state.announcing.is_none() {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::PayCosts { .. } | Action::ChooseCastCost { .. })
        });
    }
    assert!(g.state.announcing.is_none());
    resolve(&mut g);
    g.state.players[0].mana = [0, 0, 1, 1, 0, 0];
    act(&mut g, Action::AnnounceMayhem { card: carnage });
    pay(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    let target = Action::ChooseTarget {
        target: Target::Card(bear),
    };
    reach(&mut g, &target);
    act(&mut g, target);
    resolve(&mut g);
    assert_eq!(g.object_view(carnage).zone, Zone::Battlefield);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
}
