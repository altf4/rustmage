use super::tests::{act, cast, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn funded(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    let (mut pool, generic) = g.state.objects[id as usize].card.cost();
    pool[5] += generic;
    g.state.players[0].mana = pool;
    id
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let mut fork = g.clone();
    act(g, a.clone());
    act(&mut fork, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn settle(g: &mut Game) {
    for _ in 0..1000 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = actions[crate::policy::smoke_index(g, &actions)].clone();
        assert_ne!(a, Action::Concede);
        replay(g, a);
    }
    panic!("fixture didn't settle");
}
fn resolve(g: &mut Game, spell: CardId) {
    cast(g, spell, None);
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn token(g: &mut Game, p: Player) -> CardId {
    let c = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Goblin"))
        .unwrap();
    spawn(g, p, c, Zone::Battlefield)
}
#[test]
fn day_of_judgment_freezes_simultaneous_deaths_and_ignores_hexproof() {
    let mut g = scenario();
    let reaper = named(&mut g, Player::P0, "Midnight Reaper", Zone::Battlefield);
    let own = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Buzz Bots", Zone::Battlefield);
    g.state.objects[enemy as usize].gained_keywords = Keywords::HEXPROOF;
    let protected = named(
        &mut g,
        Player::P1,
        "Zetalpa, Primal Dawn",
        Zone::Battlefield,
    );
    let artifact = named(&mut g, Player::P0, "Mox Jasper", Zone::Battlefield);
    let t = token(&mut g, Player::P0);
    let spell = funded(&mut g, "Day of Judgment");
    resolve(&mut g, spell);
    for id in [reaper, own, enemy] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    assert_eq!(g.object_view(t).zone, Zone::Outside);
    for id in [protected, artifact] {
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
    let queued: Vec<_> = g
        .state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .collect();
    assert_eq!(queued.iter().filter(|s| s.card == reaper).count(), 2);
    assert_eq!(queued.iter().filter(|s| s.card == enemy).count(), 1);
    settle(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 1);
    assert_eq!(g.state.players[0].life, 18);
}
#[test]
fn exile_groups_ignore_indestructible_and_do_not_trigger_deaths() {
    let mut g = scenario();
    let reaper = named(&mut g, Player::P0, "Midnight Reaper", Zone::Battlefield);
    let protected = named(
        &mut g,
        Player::P1,
        "Zetalpa, Primal Dawn",
        Zone::Battlefield,
    );
    let t = token(&mut g, Player::P1);
    // Future-type synthetic object; no Spacecraft body becomes supported here.
    let spacecraft = Card::catalogue()
        .find(|c| !c.is_creature() && c.has_subtype("Spacecraft"))
        .unwrap();
    let ship = spawn(&mut g, Player::P1, spacecraft, Zone::Battlefield);
    let artifact = named(&mut g, Player::P0, "Mox Jasper", Zone::Battlefield);
    let spell = funded(&mut g, "Beyond the Quiet");
    resolve(&mut g, spell);
    for id in [reaper, protected, ship] {
        assert_eq!(g.object_view(id).zone, Zone::Exile);
    }
    assert_eq!(g.object_view(t).zone, Zone::Outside);
    assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
    assert!(g.state.stack.is_empty());
    assert!(g.state.pending_triggers.is_empty());
}
#[test]
fn fumigate_counts_tokens_but_not_indestructible_or_later_aura_sbas() {
    let mut g = scenario();
    let own = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let aura = named(&mut g, Player::P0, "Ethereal Armor", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(own));
    let t = token(&mut g, Player::P1);
    let protected = named(
        &mut g,
        Player::P1,
        "Zetalpa, Primal Dawn",
        Zone::Battlefield,
    );
    let spell = funded(&mut g, "Fumigate");
    resolve(&mut g, spell);
    assert_eq!(g.state.players[0].life, 23);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 1);
    for id in [own, enemy, aura] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    assert_eq!(g.object_view(t).zone, Zone::Outside);
    assert_eq!(g.object_view(protected).zone, Zone::Battlefield);
    let mut empty = scenario();
    let spell = funded(&mut empty, "Fumigate");
    resolve(&mut empty, spell);
    assert_eq!(empty.state.players[0].life_gain_events_this_turn, 0);
}
#[test]
fn result_count_survives_linked_return_entry_choice_and_new_entrants_escape_the_frozen_set() {
    let mut g = scenario();
    let light = named(&mut g, Player::P0, "Banishing Light", Zone::Battlefield);
    let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let pool = named(&mut g, Player::P1, "Breeding Pool", Zone::Exile);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(light),
        exiled: g.object_ref(pool),
        return_zone: Zone::Battlefield,
    });
    let spell = funded(&mut g, "Death Begets Life");
    resolve(&mut g, spell);
    assert!(matches!(
        g.state.pending,
        Pending::EntryLife { player: Player::P1 }
    ));
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    assert_eq!(g.observe(Player::P0).group_move_count, Some(2));
    assert_eq!(g.observe(Player::P1).group_move_count, Some(2));
    assert_eq!(g.object_view(light).zone, Zone::Graveyard);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    replay(&mut g, Action::ChooseEntryLife { pay: false });
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(g.object_view(pool).zone, Zone::Battlefield);
    assert!(g.object_view(pool).tapped);
    assert_eq!(g.observe(Player::P0).group_move_count, None);
    // A real creature source that exiles tapped creatures: the return is a new entrant,
    // so it isn't part of the Day of Judgment set captured before source departure.
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Henchbots", Zone::Battlefield);
    let exiled = named(&mut g, Player::P1, "Bear Cub", Zone::Exile);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(source),
        exiled: g.object_ref(exiled),
        return_zone: Zone::Battlefield,
    });
    let spell = funded(&mut g, "Day of Judgment");
    resolve(&mut g, spell);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.object_view(exiled).zone, Zone::Battlefield);
}
#[test]
fn group_result_life_overflow_restores_every_departure_and_unresolved_stack() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    token(&mut g, Player::P1);
    let spell = funded(&mut g, "Fumigate");
    g.state.players[0].life = i128::MAX - 2;
    cast(&mut g, spell, None);
    act(&mut g, Action::Pass);
    let saved = g.snapshot();
    let rev = g.revision();
    let views = [
        serde_json::to_value(g.observe(Player::P0)).unwrap(),
        serde_json::to_value(g.observe(Player::P1)).unwrap(),
    ];
    for _ in 0..2 {
        assert_eq!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(crate::CapacityExceeded {
                resource: "life total exhausted"
            }))
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
        assert_eq!(
            serde_json::to_value(g.observe(Player::P0)).unwrap(),
            views[0]
        );
        assert_eq!(
            serde_json::to_value(g.observe(Player::P1)).unwrap(),
            views[1]
        );
    }
}
#[test]
fn damage_groups_have_one_source_aware_batch_and_skip_flying_only_when_required() {
    for (name, amount) in [
        ("Pyroclasm", 2),
        ("Fuel the Flames", 2),
        ("Seismic Rupture", 2),
        ("Tectonic Hazard", 1),
    ] {
        let mut g = scenario();
        let own = named(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        let flyer = named(&mut g, Player::P1, "Cat-Owl", Zone::Battlefield);
        let spell = funded(&mut g, name);
        resolve(&mut g, spell);
        assert_eq!(
            g.object_view(own).damage,
            if name == "Tectonic Hazard" { 0 } else { amount }
        );
        assert_eq!(g.object_view(enemy).damage, amount);
        assert_eq!(
            g.object_view(flyer).zone,
            if name == "Seismic Rupture" {
                Zone::Battlefield
            } else if amount >= 3 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.object_view(flyer).damage,
            if name == "Seismic Rupture" { 0 } else { amount }
        );
        assert_eq!(
            g.state.players[1].life,
            if name == "Tectonic Hazard" { 19 } else { 20 }
        );
        assert_eq!(g.state.players[0].life, 20);
    }
    // Synthetic spell keyword grant tests the shared IR's source-aware batching,
    // without admitting cards that grant lifelink/deathtouch to a spell.
    let mut g = scenario();
    let one = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let two = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let spell = funded(&mut g, "Tectonic Hazard");
    cast(&mut g, spell, None);
    g.state.objects[spell as usize].gained_keywords =
        Keywords::LIFELINK.union(Keywords::DEATHTOUCH);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.state.players[0].life, 23);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 1);
    assert_eq!(g.state.players[1].life, 19);
    for id in [one, two] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn all_instructions_run_before_sbas_even_after_failed_draws() {
    let mut g = scenario();
    let ids = g.state.players[0].library.to_vec();
    for id in ids.into_iter().skip(1) {
        g.move_object(id, Zone::Exile);
    }
    assert_eq!(g.state.players[0].library.len(), 1);
    g.state.players[0].life = 3;
    g.state.players[1].life = 2;
    let spell = funded(&mut g, "Cast Away Doubt");
    resolve(&mut g, spell);
    assert_eq!(g.state.players[0].life, 1);
    assert_eq!(g.state.players[1].life, 0);
    assert_eq!(g.outcome(), Some(Outcome::Draw));
}
#[test]
fn pest_control_uses_mana_value_and_nonland_not_tokens_or_colored_costs_alone() {
    let mut g = scenario();
    let low = named(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let artifact = named(&mut g, Player::P1, "Mox Jasper", Zone::Battlefield);
    let big = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let land = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let t = token(&mut g, Player::P0);
    g.state.objects[low as usize].gained_keywords = Keywords::HEXPROOF;
    let spell = funded(&mut g, "Pest Control");
    resolve(&mut g, spell);
    for id in [low, artifact] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    for id in [big, land] {
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
    assert_eq!(g.object_view(t).zone, Zone::Outside);
}
#[test]
fn death_begets_life_counts_a_creature_enchantment_once_and_draws_exactly() {
    let mut g = scenario();
    let enchantment_creature = named(
        &mut g,
        Player::P0,
        "Fear of Surveillance",
        Zone::Battlefield,
    );
    let enchantment = named(&mut g, Player::P1, "Phyrexian Arena", Zone::Battlefield);
    let creature = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let protected = named(
        &mut g,
        Player::P0,
        "Zetalpa, Primal Dawn",
        Zone::Battlefield,
    );
    let spell = funded(&mut g, "Death Begets Life");
    resolve(&mut g, spell);
    for id in [enchantment_creature, enchantment, creature] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    assert_eq!(g.object_view(protected).zone, Zone::Battlefield);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
}
#[test]
fn every_new_modal_mode_preserves_its_whole_affected_set_and_effect() {
    for name in ["Split Up", "Slagstorm", "Ruinous Rampage"] {
        for mode in 0..2 {
            let mut g = scenario();
            let tapped = named(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
            let untapped = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
            g.state.objects[tapped as usize].tapped = true;
            for id in [tapped, untapped] {
                g.state.objects[id as usize].counters.plus_one = 1;
            }
            let low = named(&mut g, Player::P1, "Mox Jasper", Zone::Battlefield);
            let protected = named(
                &mut g,
                Player::P0,
                "Vibranium Energy Daggers",
                Zone::Battlefield,
            );
            let big = named(&mut g, Player::P1, "Gilded Lotus", Zone::Battlefield);
            let spell = funded(&mut g, name);
            replay(&mut g, Action::AnnounceCast { card: spell });
            replay(&mut g, Action::ChooseMode { mode });
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            replay(&mut g, pay);
            replay(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            match name {
                "Split Up" => {
                    assert_eq!(
                        g.object_view(tapped).zone,
                        if mode == 0 {
                            Zone::Graveyard
                        } else {
                            Zone::Battlefield
                        }
                    );
                    assert_eq!(
                        g.object_view(untapped).zone,
                        if mode == 1 {
                            Zone::Graveyard
                        } else {
                            Zone::Battlefield
                        }
                    );
                }
                "Slagstorm" => {
                    for id in [tapped, untapped] {
                        assert_eq!(g.object_view(id).damage, if mode == 0 { 3 } else { 0 });
                    }
                    for player in &g.state.players {
                        assert_eq!(player.life, if mode == 1 { 17 } else { 20 });
                    }
                }
                "Ruinous Rampage" => {
                    for id in [low, protected] {
                        assert_eq!(
                            g.object_view(id).zone,
                            if mode == 1 {
                                Zone::Exile
                            } else {
                                Zone::Battlefield
                            }
                        );
                    }
                    assert_eq!(g.state.players[1].life, if mode == 0 { 17 } else { 20 });
                    assert_eq!(g.object_view(big).zone, Zone::Battlefield);
                }
                _ => unreachable!(),
            }
        }
    }
}
#[test]
fn extended_absence_has_one_target_and_suppresses_both_tails_if_it_fizzles() {
    for invalid in [false, true] {
        let mut g = scenario();
        let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let spell = funded(&mut g, "Extended Absence");
        cast(&mut g, spell, Some(Target::Card(target)));
        if invalid {
            g.move_object(target, Zone::Hand);
            g.move_object(target, Zone::Battlefield);
        }
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert_eq!(
            g.object_view(target).zone,
            if invalid {
                Zone::Battlefield
            } else {
                Zone::Exile
            }
        );
        assert_eq!(g.state.players[0].life, if invalid { 20 } else { 21 });
        assert_eq!(g.state.players[1].life, if invalid { 20 } else { 19 });
    }
}
#[test]
fn cycling_keeps_both_new_spell_bodies_off_the_stack() {
    for name in ["Fuel the Flames", "Pest Control"] {
        let mut g = scenario();
        let victim = named(&mut g, Player::P1, "Llanowar Elves", Zone::Battlefield);
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Activate {card,..}if *card==spell))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
        assert_eq!(g.object_view(victim).damage, 0);
    }
}
