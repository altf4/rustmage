use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
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
    g.prepare_triggers();
    for _ in 0..150 {
        if g.state.stack.is_empty() && g.state.pending == Pending::Priority {
            return;
        }
        let p = g.actor().unwrap();
        if g.observe(p).library_choice.is_some() {
            assert!(g.observe(p.other()).library_choice.is_none());
            assert!(g.observe(p.other()).actions.is_empty());
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a, Action::ChooseSearch { .. }))
            .cloned()
            .or_else(|| {
                actions.into_iter().find(|a| match a {
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    } => target.is_none_or(|t| t == *id),
                    Action::ChooseTarget {
                        target: Target::Player(who),
                    } => *who == p.other(),
                    Action::ChooseSearch { .. }
                    | Action::FinishSearch
                    | Action::PutTrigger { .. }
                    | Action::Pass
                    | Action::ArrangeLibrary { .. }
                    | Action::Sacrifice { .. }
                    | Action::FinishTargets => true,
                    _ => false,
                })
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
fn factory(subtype: &str) -> Card {
    Card::catalogue()
        .find(|c| c.is_token() && c.has_subtype(subtype))
        .unwrap()
}
fn treasure(g: &mut Game, p: Player) -> CardId {
    let c = factory("Treasure");
    g.create_tokens(p, c, 1, false);
    tokens(g, p, "Treasure").last().unwrap().id
}
fn pending(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .filter(|s| s.card == id)
        .count()
}
#[test]
fn sacrifice_filters_use_explicit_cause_actor_types_and_other_not_graveyard_destination() {
    for p in [Player::P0, Player::P1] {
        for (name, yes, no) in [
            ("Biotech Specialist", "Gravestone Strider", "Bear Cub"),
            ("Crime Novelist", "Nutrient Block", "Bear Cub"),
            (
                "Experimental Confectioner",
                "Nutrient Block",
                "Gravestone Strider",
            ),
            ("Gleaming Geardrake", "Gravestone Strider", "Bear Cub"),
            ("Lightless Evangel", "Gravestone Strider", "Forest"),
            ("Pirate Peddlers", "Forest", "Forest"),
            ("Tolls of War", "Forest", "Forest"),
        ] {
            let mut g = setup(p);
            let source = named(&mut g, p, name, Zone::Battlefield);
            for zone in [Zone::Graveyard, Zone::Hand, Zone::Exile] {
                let id = named(&mut g, p, yes, Zone::Battlefield);
                let e = g.move_object(id, zone);
                assert_eq!(e.sacrificed_by, None);
                assert_eq!(pending(&g, source), 0, "{name}");
            }
            let hand = named(&mut g, p, yes, Zone::Hand);
            g.discard_batch(&[hand], None);
            assert_eq!(pending(&g, source), 0);
            let foreign = named(&mut g, p.other(), yes, Zone::Battlefield);
            g.sacrifice_batch(&[(p.other(), foreign)]);
            assert_eq!(pending(&g, source), 0);
            if name != "Pirate Peddlers" && name != "Tolls of War" {
                let wrong = named(&mut g, p, no, Zone::Battlefield);
                g.sacrifice_batch(&[(p, wrong)]);
                assert_eq!(pending(&g, source), 0);
            }
            let id = named(&mut g, p, yes, Zone::Battlefield);
            g.sacrifice_batch(&[(p, id)]);
            assert_eq!(pending(&g, source), 1, "{name}: multitype is one event");
            if name == "Lightless Evangel" || name == "Pirate Peddlers" {
                g.sacrifice_batch(&[(p, source)]);
                assert_eq!(pending(&g, source), 1);
            }
        }
    }
}
#[test]
fn all_new_creatures_and_tolls_cast_with_complete_entries_and_keywords() {
    for p in [Player::P0, Player::P1] {
        for (name, pt, keyword, token) in [
            ("Biotech Specialist", (1, 3), Keywords::NONE, Some("Lander")),
            ("Crime Novelist", (1, 3), Keywords::NONE, None),
            (
                "Experimental Confectioner",
                (2, 3),
                Keywords::NONE,
                Some("Food"),
            ),
            ("Gleaming Geardrake", (1, 1), Keywords::FLYING, Some("Clue")),
            ("Lightless Evangel", (2, 2), Keywords::NONE, None),
            ("Pirate Peddlers", (2, 2), Keywords::DEATHTOUCH, None),
            ("Tolls of War", (0, 0), Keywords::NONE, Some("Clue")),
        ] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, id, None);
            settle(&mut g, None);
            let v = g.object_view(id);
            assert_eq!(v.zone, Zone::Battlefield);
            assert_eq!((v.power, v.toughness), pt);
            assert!(v.keywords.contains(keyword));
            if let Some(kind) = token {
                assert_eq!(tokens(&g, p, kind).len(), 1, "{name}");
            }
        }
    }
}
#[test]
fn confectioner_food_cost_creates_complete_rat_above_independent_life_gain() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Experimental Confectioner", Zone::Hand);
        cast(&mut g, id, None);
        settle(&mut g, None);
        let food = tokens(&g, p, "Food")[0].id;
        activate(&mut g, food, 0);
        assert_eq!(g.object_view(food).zone, Zone::Outside);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.stack.len(), 2);
        assert_eq!(g.state.stack.last().unwrap().card, id);
        pass_pair(&mut g);
        let rats = tokens(&g, p, "Rat");
        assert_eq!(rats.len(), 1);
        assert_eq!((rats[0].power, rats[0].toughness), (1, 1));
        assert!(rats[0].keywords.contains(Keywords::CANT_BLOCK));
        assert_eq!(rats[0].card.colors(), 4);
        assert_eq!(g.state.players[p.index()].life, 20);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].life, 23);
    }
}
#[test]
fn geardrake_clue_cost_grows_before_its_private_draw_and_replays() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Gleaming Geardrake", Zone::Hand);
        cast(&mut g, id, None);
        settle(&mut g, None);
        let clue = tokens(&g, p, "Clue")[0].id;
        activate(&mut g, clue, 0);
        assert_eq!(g.state.stack.len(), 2);
        assert_eq!(g.state.stack.last().unwrap().card, id);
        pass_pair(&mut g);
        assert_eq!(g.object_view(id).counters.plus_one, 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .all(|o| o.owner != p || o.zone != Zone::Hand)
        );
    }
}
#[test]
fn biotech_lander_cost_queues_targeted_damage_above_private_tapped_land_search() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Biotech Specialist", Zone::Hand);
        cast(&mut g, id, None);
        settle(&mut g, None);
        let lander = tokens(&g, p, "Lander")[0].id;
        activate(&mut g, lander, 0);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert!(
            g.state.objects.iter().any(|o| o.owner == p
                && o.zone == Zone::Battlefield
                && o.card.is_land()
                && o.tapped)
        );
    }
}
#[test]
fn crime_novelist_triggered_mana_cannot_fund_the_cost_that_triggered_it() {
    let p = Player::P0;
    let mut g = setup(p);
    let novelist = named(&mut g, p, "Crime Novelist", Zone::Battlefield);
    let token = treasure(&mut g, p);
    let expensive = named(&mut g, p, "Divination", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: expensive })
    );
    let spell = named(&mut g, p, "Shock", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    let mana=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==token&&production[3]==1)).unwrap();
    replay(&mut g, mana);
    assert_eq!(g.state.players[0].mana, [0, 0, 0, 1, 0, 0]);
    assert_eq!(g.object_view(novelist).counters.plus_one, 0);
    assert_eq!(pending(&g, novelist), 1);
    assert!(g.state.stack.is_empty());
    assert_eq!(g.state.objects[token as usize].zone, Zone::Graveyard);
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, pay);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(g.object_view(novelist).counters.plus_one, 0);
    settle(&mut g, None);
    assert_eq!(g.object_view(novelist).counters.plus_one, 1);
    assert_eq!(g.state.players[0].mana[3], 1);
    assert_eq!(g.state.players[1].life, 18);
}
#[test]
fn additional_spell_cost_sacrifice_is_paid_once_before_lower_spell_and_replays() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let evangel = named(&mut g, p, "Lightless Evangel", Zone::Battlefield);
        let pirate = named(&mut g, p, "Pirate Peddlers", Zone::Battlefield);
        let body = named(&mut g, p, "Gravestone Strider", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Corrupted Conviction", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: spell });
        let foreign_ref = g.object_ref(foreign);
        reject(
            &mut g,
            Action::ChooseAdditionalCost {
                object: foreign_ref,
            },
        );
        let body_ref = g.object_ref(body);
        replay(&mut g, Action::ChooseAdditionalCost { object: body_ref });
        assert_eq!(g.object_view(body).zone, Zone::Battlefield);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, pay);
        assert_eq!(g.object_view(body).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        settle(&mut g, None);
        assert_eq!(g.object_view(evangel).counters.plus_one, 1);
        assert_eq!(g.object_view(pirate).counters.plus_one, 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.creatures_died_this_turn, 1);
    }
}
#[test]
fn sacrifice_source_effect_stays_on_battlefield_until_resolution_and_ignores_indestructible() {
    let p = Player::P0;
    let mut g = setup(p);
    let pirate = named(&mut g, p, "Pirate Peddlers", Zone::Battlefield);
    let vigil = named(&mut g, p, "Hopeful Vigil", Zone::Battlefield);
    activate(&mut g, vigil, 0);
    assert_eq!(g.object_view(vigil).zone, Zone::Battlefield);
    assert_eq!(g.object_view(pirate).counters.plus_one, 0);
    settle(&mut g, None);
    assert_eq!(g.object_view(pirate).counters.plus_one, 1);
    let n = named(&mut g, p, "Nutrient Block", Zone::Battlefield);
    let spell = named(&mut g, p, "Disenchant", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(n)));
    settle(&mut g, None);
    assert_eq!(g.object_view(n).zone, Zone::Battlefield);
    assert_eq!(g.object_view(pirate).counters.plus_one, 1);
    activate(&mut g, n, 0);
    settle(&mut g, None);
    assert_eq!(g.object_view(n).zone, Zone::Graveyard);
    assert_eq!(g.object_view(pirate).counters.plus_one, 2);
}
#[test]
fn tolls_turn_gate_and_once_ledger_survive_source_departure_and_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let toll = named(&mut g, p, "Tolls of War", Zone::Battlefield);
        g.state.active = p.other();
        let a = named(&mut g, p, "Forest", Zone::Battlefield);
        g.sacrifice_batch(&[(p, a)]);
        assert_eq!(pending(&g, toll), 0);
        g.state.active = p;
        let a = named(&mut g, p, "Forest", Zone::Battlefield);
        g.sacrifice_batch(&[(p, a), (p, toll)]);
        assert_eq!(pending(&g, toll), 1);
        settle(&mut g, None);
        assert_eq!(tokens(&g, p, "Ally").len(), 1);
        let ally = tokens(&g, p, "Ally")[0].clone();
        assert_eq!((ally.power, ally.toughness), (1, 1));
        assert_eq!(ally.card.colors(), 1);
        g.move_object(toll, Zone::Battlefield);
        settle(&mut g, None);
        let b = named(&mut g, p, "Forest", Zone::Battlefield);
        g.sacrifice_batch(&[(p, b)]);
        assert_eq!(pending(&g, toll), 1);
        settle(&mut g, None);
        assert_eq!(tokens(&g, p, "Ally").len(), 2);
        let b = named(&mut g, p, "Forest", Zone::Battlefield);
        g.sacrifice_batch(&[(p, b)]);
        assert_eq!(pending(&g, toll), 0);
        g.begin_turn();
        let next = named(&mut g, p, "Forest", Zone::Battlefield);
        g.sacrifice_batch(&[(p, next)]);
        assert_eq!(pending(&g, toll), 1);
        settle(&mut g, None);
        assert_eq!(tokens(&g, p, "Ally").len(), 3);
    }
}
#[test]
fn simultaneous_effect_sacrifices_use_dying_observers_for_both_players_in_apnap_order() {
    for active in [Player::P0, Player::P1] {
        let mut g = setup(active);
        let mut ids = [Vec::new(), Vec::new()];
        for p in [active, active.other()] {
            for name in ["Lightless Evangel", "Pirate Peddlers"] {
                ids[p.index()].push(named(&mut g, p, name, Zone::Battlefield));
            }
        }
        let gore = named(&mut g, active, "Abyssal Gorestalker", Zone::Hand);
        cast(&mut g, gore, None);
        pass_pair(&mut g);
        pass_pair(&mut g);
        for p in [active, active.other()] {
            for &id in &ids[p.index()] {
                assert_eq!(g.actor(), Some(p));
                let object = g.object_ref(id);
                replay(&mut g, Action::Sacrifice { object });
            }
        }
        assert_eq!(g.state.creatures_died_this_turn, 4);
        let live: Vec<_> = g
            .state
            .stack
            .iter()
            .chain(g.state.pending_triggers.iter())
            .chain(g.state.ordering_triggers.iter())
            .collect();
        assert_eq!(live.len(), 4);
        assert_eq!(g.actor(), Some(active));
        assert!(
            live.iter()
                .all(|s| ids[s.controller.index()].contains(&s.card))
        );
        settle(&mut g, None);
        for p in [active, active.other()] {
            for &id in &ids[p.index()] {
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
                assert_eq!(g.object_view(id).counters.plus_one, 0);
            }
        }
    }
}
#[test]
fn removed_source_trigger_cannot_grow_a_returned_incarnation_but_still_adds_mana() {
    let p = Player::P0;
    let mut g = setup(p);
    let id = named(&mut g, p, "Crime Novelist", Zone::Battlefield);
    let t = treasure(&mut g, p);
    g.sacrifice_batch(&[(p, t)]);
    let old = g.object_ref(id);
    g.move_object(id, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    assert_ne!(old, g.object_ref(id));
    let before = g.state.players[0].mana[3];
    settle(&mut g, None);
    assert_eq!(g.object_view(id).counters.plus_one, 0);
    assert_eq!(g.state.players[0].mana[3], before + 1);
}
#[test]
fn sacrifice_queue_capacity_failure_rolls_back_paid_cost_token_pool_and_revision() {
    let p = Player::P0;
    let mut g = setup(p);
    named(&mut g, p, "Crime Novelist", Zone::Battlefield);
    let t = treasure(&mut g, p);
    g.state.next_stack_id = u64::MAX;
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==t))
        .unwrap();
    let saved = g.snapshot();
    let r = g.revision();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(p, a.clone()),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), r);
        assert_eq!(g.object_view(t).zone, Zone::Battlefield);
    }
}
#[test]
fn self_artifact_sacrifice_uses_frozen_source_and_counter_overflow_does_not_add_mana() {
    let p = Player::P0;
    let mut g = setup(p);
    let drake = named(&mut g, p, "Gleaming Geardrake", Zone::Battlefield);
    let novelist = named(&mut g, p, "Crime Novelist", Zone::Battlefield);
    g.sacrifice_batch(&[(p, drake)]);
    assert_eq!(pending(&g, drake), 1);
    assert_eq!(pending(&g, novelist), 1);
    settle(&mut g, None);
    assert_eq!(g.object_view(drake).counters.plus_one, 0);
    assert_eq!(g.object_view(novelist).counters.plus_one, 1);
    g.state.objects[novelist as usize].counters.plus_one = u64::MAX;
    let t = treasure(&mut g, p);
    g.sacrifice_batch(&[(p, t)]);
    g.prepare_triggers();
    act(&mut g, Action::Pass);
    let saved = g.snapshot();
    let r = g.revision();
    let actor = g.actor().unwrap();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(actor, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), r);
    }
}
