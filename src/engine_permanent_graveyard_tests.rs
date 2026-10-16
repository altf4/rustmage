use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
const WATCHERS: [&str; 5] = [
    "Ashiok's Reaper",
    "Knight of Doves",
    "Savior of the Sleeping",
    "Warehouse Tabby",
    "Wicked Visitor",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn settle(g: &mut Game, target: Option<CardId>) {
    for _ in 0..100 {
        if g.state.stack.is_empty() && g.state.pending == Pending::Priority {
            return;
        }
        let p = g.actor().unwrap();
        if g.observe(p).library_choice.is_some() {
            assert!(g.observe(p.other()).library_choice.is_none());
            assert!(g.observe(p.other()).actions.is_empty());
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| match a {
                Action::ChooseTarget {
                    target: Target::Card(id),
                } => target.is_none_or(|t| t == *id),
                Action::PutTrigger { .. }
                | Action::Pass
                | Action::ArrangeLibrary { .. }
                | Action::Sacrifice { .. }
                | Action::FinishTargets => true,
                _ => false,
            })
            .expect("fixture resolution choice");
        replay(g, a);
    }
    panic!("not settled");
}
fn activate(g: &mut Game, id: CardId, ability: u16) {
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,ability:a,target:None,..}if *card==id&&*a==ability)).expect("fixture activation");
    replay(g, a);
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<ObjectView> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.owner == p
                && o.card.is_token()
                && o.card.has_subtype(subtype)
        })
        .map(|(i, _)| g.object_view(i as CardId))
        .collect()
}

#[test]
fn enchantment_watchers_require_owned_battlefield_departures_and_keep_creature_history_separate() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in WATCHERS.iter().enumerate() {
            let mut g = setup(p);
            let source = named(&mut g, p, name, Zone::Battlefield);
            let enchant = named(&mut g, p, "Goblin Oriflamme", Zone::Battlefield);
            g.move_object(enchant, Zone::Hand);
            g.move_object(enchant, Zone::Graveyard);
            g.move_object(enchant, Zone::Exile);
            g.move_object(enchant, Zone::Battlefield);
            let milled = named(&mut g, p, "Goblin Oriflamme", Zone::Library);
            g.move_object(milled, Zone::Graveyard);
            let enemy = named(&mut g, p.other(), "Goblin Oriflamme", Zone::Battlefield);
            g.move_object(enemy, Zone::Graveyard);
            let artifact = named(&mut g, p, "World Map", Zone::Battlefield);
            g.move_object(artifact, Zone::Graveyard);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.move_object(bear, Zone::Graveyard);
            assert!(g.state.pending_triggers.is_empty());
            assert_eq!(g.state.creatures_died_this_turn, 1);
            let old = g.object_ref(enchant);
            g.move_object(enchant, Zone::Graveyard);
            assert_eq!(g.state.pending_triggers.len(), 1);
            assert_eq!(g.state.pending_triggers[0].card, source);
            assert_ne!(g.object_ref(enchant), old);
            assert_eq!(g.state.creatures_died_this_turn, 1);
            g.prepare_triggers();
            settle(&mut g, None);
            match i {
                0 => {
                    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
                    assert_eq!(g.hand_count(p), 1);
                    assert!(g.state.known_hand.is_empty());
                }
                1 => {
                    let birds = tokens(&g, p, "Bird");
                    assert_eq!(birds.len(), 1);
                    assert_eq!((birds[0].power, birds[0].toughness), (1, 1));
                    assert_eq!(birds[0].card.colors(), 1);
                    assert!(birds[0].keywords.contains(Keywords::FLYING));
                    assert!(birds[0].summoning_sick);
                }
                2 => {
                    assert_eq!(g.object_view(source).counters.plus_one, 1);
                    assert_eq!((g.stats(source).power, g.stats(source).toughness), (3, 4));
                    assert!(g.keywords(source).contains(Keywords::VIGILANCE));
                }
                3 => {
                    let rats = tokens(&g, p, "Rat");
                    assert_eq!(rats.len(), 1);
                    assert_eq!((rats[0].power, rats[0].toughness), (1, 1));
                    assert_eq!(rats[0].card.colors(), 4);
                    assert!(rats[0].keywords.contains(Keywords::CANT_BLOCK));
                    activate(&mut g, source, 0);
                    settle(&mut g, None);
                    assert!(g.keywords(source).contains(Keywords::DEATHTOUCH));
                }
                _ => {
                    assert_eq!(g.state.players[p.other().index()].life, 19);
                    assert_eq!(g.state.players[p.index()].life, 20);
                }
            }
        }
    }
}

#[test]
fn mephitic_draught_draws_and_loses_life_on_actual_entry_and_artifact_destruction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Mephitic Draught", Zone::Hand);
        cast(&mut g, id, None);
        settle(&mut g, None);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, 19);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        let discarded = named(&mut g, p, "Mephitic Draught", Zone::Hand);
        g.move_object(discarded, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
        let removal = named(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, removal, Some(Target::Card(id)));
        settle(&mut g, None);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.creatures_died_this_turn, 0);
        assert!(g.state.known_hand.is_empty());
    }
}

#[test]
fn nutrient_block_resists_destroy_but_its_paid_sacrifice_triggers_before_the_life_gain() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Nutrient Block", Zone::Hand);
        cast(&mut g, id, None);
        settle(&mut g, None);
        assert!(g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
        let removal = named(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, removal, Some(Target::Card(id)));
        settle(&mut g, None);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.hand_count(p), 0);
        activate(&mut g, id, 0);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.stack.len(), 2);
        assert!(matches!(
            g.state.stack.last().unwrap().kind,
            StackKind::TriggeredAbility { .. }
        ));
        pass_pair(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[p.index()].life, 20);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].life, 23);
        assert_eq!(g.state.creatures_died_this_turn, 0);
    }
}

#[test]
fn hopeful_vigil_enters_with_full_knight_and_sacrifices_only_on_ability_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Hopeful Vigil", Zone::Hand);
        cast(&mut g, id, None);
        settle(&mut g, None);
        let knights = tokens(&g, p, "Knight");
        assert_eq!(knights.len(), 1);
        assert_eq!((knights[0].power, knights[0].toughness), (2, 2));
        assert_eq!(knights[0].card.colors(), 1);
        assert!(knights[0].keywords.contains(Keywords::VIGILANCE));
        let old = g.object_ref(id);
        activate(&mut g, id, 0);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.object_ref(id), old);
        pass_pair(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert!(g.observe(p).effect_choice.is_some());
        replay(&mut g, Action::Sacrifice { object: old });
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_ne!(g.object_ref(id), old);
        pass_pair(&mut g);
        assert!(g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        assert!(g.observe(p.other()).actions.is_empty());
        settle(&mut g, None);
        assert_eq!(g.state.creatures_died_this_turn, 0);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
}

#[test]
fn dying_watchers_see_simultaneous_enchantments_in_frozen_pre_event_views() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mut ids = Vec::new();
        for owner in [p, p.other()] {
            ids.push(named(&mut g, owner, "Ashiok's Reaper", Zone::Battlefield));
            ids.push(named(&mut g, owner, "Wicked Visitor", Zone::Battlefield));
            ids.push(named(&mut g, owner, "Goblin Oriflamme", Zone::Battlefield));
        }
        let sources = g.death_trigger_sources();
        let old: Vec<_> = ids.iter().map(|&id| g.object_ref(id)).collect();
        let events = g.move_objects_raw(&ids, Zone::Graveyard);
        g.finish_zone_events(&events, &sources);
        assert_eq!(g.state.pending_triggers.len(), 4);
        assert_eq!(g.state.creatures_died_this_turn, 4);
        for trigger in &g.state.pending_triggers {
            assert!(old.contains(&trigger.object));
            assert_ne!(g.object_ref(trigger.card), trigger.object);
        }
        g.prepare_triggers();
        assert_eq!(g.actor(), Some(p));
        settle(&mut g, None);
        for s in &g.state.players {
            assert_eq!(s.cards_drawn_this_turn, 1);
            assert_eq!(s.life, 19);
        }
    }
}

#[test]
fn aura_cleanup_and_direct_destruction_both_use_noncreature_departure_triggers() {
    for kill_creature in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        named(&mut g, p, "Ashiok's Reaper", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let aura = named(&mut g, p, "Reach for the Sky", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(bear)));
        settle(&mut g, None);
        let v = g.object_view(bear);
        assert_eq!((v.power, v.toughness), (5, 4));
        assert!(v.keywords.contains(Keywords::REACH));
        assert_eq!(g.object_view(aura).attached_to, Some(v.reference));
        assert!(
            g.object_view(aura)
                .card
                .keywords()
                .contains(Keywords::FLASH)
        );
        let removal = named(
            &mut g,
            p,
            if kill_creature { "Fell" } else { "Disenchant" },
            Zone::Hand,
        );
        cast(
            &mut g,
            removal,
            Some(Target::Card(if kill_creature { bear } else { aura })),
        );
        settle(&mut g, None);
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
        assert_eq!(g.state.creatures_died_this_turn, u64::from(kill_creature));
        if !kill_creature {
            let v = g.object_view(bear);
            assert_eq!((v.power, v.toughness), (2, 2));
            assert!(!v.keywords.contains(Keywords::REACH));
        }
    }
}

#[test]
fn long_feng_matches_owned_lands_or_other_creatures_but_not_itself_or_foreign_objects() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Long Feng, Grand Secretariat", Zone::Battlefield);
        let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let artifact = named(&mut g, p, "World Map", Zone::Battlefield);
        g.move_object(artifact, Zone::Graveyard);
        for name in ["Bear Cub", "Forest"] {
            let id = named(&mut g, p.other(), name, Zone::Battlefield);
            g.move_object(id, Zone::Graveyard);
        }
        assert!(g.state.pending_triggers.is_empty());
        for name in ["Forest", "Bear Cub"] {
            let id = named(&mut g, p, name, Zone::Battlefield);
            g.move_object(id, Zone::Graveyard);
            g.prepare_triggers();
            let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foreign),
                },
            );
            settle(&mut g, Some(target));
        }
        assert_eq!(g.object_view(target).counters.plus_one, 2);
        g.move_object(source, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
    }
}

#[test]
fn source_and_target_counter_effects_cannot_follow_new_incarnations() {
    let mut g = setup(Player::P0);
    let source = named(
        &mut g,
        Player::P0,
        "Savior of the Sleeping",
        Zone::Battlefield,
    );
    let enchant = named(&mut g, Player::P0, "Goblin Oriflamme", Zone::Battlefield);
    g.move_object(enchant, Zone::Graveyard);
    let old = g.object_ref(source);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    g.prepare_triggers();
    settle(&mut g, None);
    assert_ne!(g.object_ref(source), old);
    assert_eq!(g.object_view(source).counters.plus_one, 0);
    let feng = named(
        &mut g,
        Player::P0,
        "Long Feng, Grand Secretariat",
        Zone::Battlefield,
    );
    let land = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    g.move_object(land, Zone::Graveyard);
    g.prepare_triggers();
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    let old = g.object_ref(source);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    settle(&mut g, None);
    assert_ne!(g.object_ref(source), old);
    assert_eq!(g.object_view(source).counters.plus_one, 0);
    assert_eq!(g.object_view(feng).counters.plus_one, 0);
}

#[test]
fn counter_capacity_failure_restores_pending_departure_trigger_and_every_registry() {
    let mut g = setup(Player::P0);
    let source = named(
        &mut g,
        Player::P0,
        "Savior of the Sleeping",
        Zone::Battlefield,
    );
    g.state.objects[source as usize].counters.plus_one = u64::MAX;
    let enchant = named(&mut g, Player::P0, "Goblin Oriflamme", Zone::Battlefield);
    g.move_object(enchant, Zone::Graveyard);
    g.prepare_triggers();
    act(&mut g, Action::Pass);
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(p, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), s);
        assert_eq!(g.revision(), r);
        assert_eq!(g.object_view(enchant).zone, Zone::Graveyard);
    }
}

#[test]
fn creature_only_observers_stay_separate_and_multitype_departures_trigger_each_ability_once() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, "Midnight Reaper", Zone::Battlefield);
    named(&mut g, Player::P0, "Ashiok's Reaper", Zone::Battlefield);
    let feng = named(
        &mut g,
        Player::P0,
        "Long Feng, Grand Secretariat",
        Zone::Battlefield,
    );
    for (name, count, drawn, life, deaths) in [
        ("World Map", 0, 0, 20, 0),
        ("Goblin Oriflamme", 1, 1, 20, 0),
        ("Fear of Surveillance", 3, 3, 19, 1),
        ("Gravestone Strider", 2, 4, 18, 2),
    ] {
        let id = named(&mut g, Player::P0, name, Zone::Battlefield);
        g.move_object(id, Zone::Graveyard);
        assert_eq!(g.state.pending_triggers.len(), count);
        g.prepare_triggers();
        settle(&mut g, Some(feng));
        assert_eq!(g.state.players[0].cards_drawn_this_turn, drawn);
        assert_eq!(g.state.players[0].life, life);
        assert_eq!(g.state.creatures_died_this_turn, deaths);
    }
    assert_eq!(g.object_view(feng).counters.plus_one, 2);
}

#[test]
fn source_only_artifact_departure_does_not_fire_for_exile_bounce_or_private_zone_moves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Nutrient Block", Zone::Battlefield);
        for zone in [
            Zone::Exile,
            Zone::Hand,
            Zone::Graveyard,
            Zone::Library,
            Zone::Graveyard,
            Zone::Exile,
            Zone::Battlefield,
        ] {
            g.move_object(source, zone);
            assert!(g.state.pending_triggers.is_empty());
        }
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.state.pending_triggers.len(), 1);
        g.prepare_triggers();
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.state.creatures_died_this_turn, 0);
    }
}

#[test]
fn hopeful_vigil_old_sacrifice_effect_cannot_consume_returned_incarnation() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Hopeful Vigil", Zone::Battlefield);
    activate(&mut g, source, 0);
    let old = g.object_ref(source);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    g.prepare_triggers();
    settle(&mut g, None);
    assert_ne!(g.object_ref(source), old);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    assert_eq!(tokens(&g, Player::P0, "Knight").len(), 1);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
}

#[test]
fn nutrient_block_paid_old_abilities_finish_after_a_new_source_incarnation_enters() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Nutrient Block", Zone::Battlefield);
    let old = g.object_ref(source);
    activate(&mut g, source, 0);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    settle(&mut g, None);
    assert_ne!(g.object_ref(source), old);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.state.players[0].life, 23);
    assert_eq!(g.state.creatures_died_this_turn, 0);
}
