//! Complete paid Bow/Hulkling representatives for independent event-object facts.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const BOW: &str = "Hawkeye's Bow";
const HULK: &str = "Hulkling, Burgeoning Bruiser";
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn priority(g: &mut Game, player: Player) {
    if g.actor() != Some(player) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(player));
}
fn paid(g: &mut Game, player: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, player);
    let id = named(g, player, name, Zone::Hand);
    let (colored, generic) = g.object_view(id).card.cost();
    g.state.players[player.index()].mana = colored;
    g.state.players[player.index()].mana[5] += generic;
    cast(g, id, None, targets);
    settle(g);
    id
}
fn finish(g: &mut Game) {
    for _ in 0..15 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("unfinished quantity scenario payment");
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("quantity scenario stalled {:?}", g.state.pending);
}
fn declare(g: &mut Game, p: Player, id: CardId) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    // Existing combat scenario boundary; separate body test verifies actual untap.
    g.state.objects[id as usize].summoning_sick = false;
    g.state.objects[id as usize].tapped = false;
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    g.prepare_triggers();
}
fn equip(g: &mut Game, p: Player, equipment: CardId, target: CardId) {
    priority(g, p);
    let definition = g.object_view(equipment).card.activation(0);
    g.state.players[p.index()].mana = definition.cost.colored;
    g.state.players[p.index()].mana[5] += definition.cost.generic;
    checked(
        g,
        Action::AnnounceActivate {
            card: equipment,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(g);
    settle(g);
    assert_eq!(
        g.object_view(equipment).attached_to,
        Some(g.object_ref(target))
    );
}
fn moved(g: &mut Game, ids: &[CardId], zone: Zone) {
    let events = g.move_objects_raw(ids, zone);
    g.finish_zone_events(&events, &[]);
}
fn packet(g: &Game, id: CardId) -> &SpellView {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .find(|item| item.card == id)
        .unwrap()
}

fn armed(g: &mut Game, p: Player, creature: &str) -> (CardId, CardId) {
    let bow = paid(g, p, BOW, &[]);
    let who = paid(g, p, creature, &[]);
    equip(g, p, bow, who);
    (bow, who)
}
fn event_packets(g: &Game) -> impl Iterator<Item = &SpellView> {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
}
fn has_packet(g: &Game, id: CardId) -> bool {
    event_packets(g).any(|s| s.card == id && s.event_object.is_some())
}
fn entry(g: &mut Game, p: Player, name: &str) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (colored, generic) = g.object_view(id).card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, None, &[]);
    until(g, |g| g.on_battlefield(id));
    id
}
fn enters(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Graveyard);
    moved(g, &[id], Zone::Battlefield);
    id
}
fn tap(g: &mut Game, id: CardId) {
    g.tap_objects(&[id]);
}
fn damage_actor(p: &SpellView) -> DamageSource {
    let EventObjectFacts::DamageActor(d) = p.event_object.as_ref().unwrap().facts else {
        panic!("not damage");
    };
    d
}
fn entrant_facts(p: &SpellView) -> CreatureComparisonFacts {
    let EventObjectFacts::CreatureComparison(d) = p.event_object.as_ref().unwrap().facts else {
        panic!("not stats");
    };
    d
}
fn blank(g: &mut Game, p: Player, id: CardId) -> CardId {
    let aura = named(g, p, "Stop Cold", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(id));
    aura
}
fn noncreature(g: &mut Game, p: Player, id: CardId) -> CardId {
    let aura = named(g, p, "Imprisoned in the Moon", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(id));
    assert!(!g.is_creature(id));
    aura
}
#[test]
fn msh_event_objects_paid_equipment_body_equip_cost_targets_timing_atomicity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bow = paid(&mut g, p, BOW, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let own = paid(&mut g, p, "Bear Cub", &[]);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana[5] = 1;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: bow,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bow),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        while g.state.pending != (Pending::PayCosts { player: p }) {
            checked(&mut g, Action::FinishTargets);
        }
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        g.state.players[p.index()].mana[5] = 1;
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!((g.stats(own).power, g.stats(own).toughness), (3, 2));
        assert!(g.keywords(own).contains(Keywords::REACH));
        g.state.step = Step::BeginCombat;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: bow,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn msh_event_objects_bow_reach_blocks_flying_and_actual_combat_boost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (_, bear) = armed(&mut g, p, "Bear Cub");
        let flyer = named(&mut g, p.other(), "Healer's Hawk", Zone::Battlefield);
        declare(&mut g, p.other(), flyer);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: flyer,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.step == Step::CombatDamage);
        until(&mut g, |g| g.object_view(flyer).zone == Zone::Graveyard);
        assert!(g.on_battlefield(bear));
        let mut g = setup(p);
        let (_, bear) = armed(&mut g, p, "Bear Cub");
        declare(&mut g, p, bear);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 19);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.life(p.other()) == 16);
    }
}
#[test]
fn msh_event_objects_bow_mana_tap_funding_and_production_failures_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (bow, who) = armed(&mut g, p, "Topiary Lecturer");
        next_turn(&mut g);
        next_turn(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        let goal = |n| FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, n, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        assert!(g.can_fund(goal(2)));
        assert!(!g.can_fund(goal(3)));
        assert_eq!(g.snapshot(), before);
        reject(
            &mut g,
            Action::ActivateMana {
                card: who,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 3, 0],
            },
        );
        checked(
            &mut g,
            Action::ActivateMana {
                card: who,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 2, 0],
            },
        );
        assert_eq!(g.life(p.other()), 20);
        assert!(has_packet(&g, bow));
        assert_eq!(g.state.players[p.index()].mana[4], 2);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 19);
        tap(&mut g, who);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 19);
    }
}
#[test]
fn msh_event_objects_bow_event_qualification_vigilance_repeated_taps_and_entry_tapped() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (bow, who) = armed(&mut g, p, HULK);
        declare(&mut g, p, who);
        settle(&mut g);
        assert!(!g.object_view(who).tapped);
        assert!(!has_packet(&g, bow));
        assert_eq!(g.life(p.other()), 20);
        tap(&mut g, who);
        tap(&mut g, who);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 19);
        g.state.objects[who as usize].tapped = false;
        tap(&mut g, who);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 18);
        let just_entered = named(&mut g, p, "Whiplash, Vengeful Engineer", Zone::Graveyard);
        moved(&mut g, &[just_entered], Zone::Battlefield);
        assert!(g.object_view(just_entered).tapped);
        assert!(!has_packet(&g, bow));
    }
}
#[test]
fn msh_event_objects_bow_current_actor_controller_lifelink_and_independent_opponents() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (bow, who) = armed(&mut g, p, "Bear Cub");
        tap(&mut g, who);
        g.record_control(g.object_ref(who), p.other(), None);
        g.state.objects[who as usize].gained_keywords = Keywords::LIFELINK;
        assert_eq!(packet(&g, bow).controller, p);
        assert_eq!(packet(&g, bow).card, bow);
        settle(&mut g);
        assert_eq!((g.life(p), g.life(p.other())), (20, 20));
        g.state.objects[who as usize].tapped = false;
        tap(&mut g, who); // Opponent-controlled equipped creature still triggers Bow.
        assert!(has_packet(&g, bow));
        settle(&mut g);
        assert_eq!((g.life(p), g.life(p.other())), (20, 20));
    }
}
#[test]
fn msh_event_objects_bow_detach_reequip_source_theft_and_removal_preserve_actor() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = setup(p);
            let (bow, who) = armed(&mut g, p, "Healer's Hawk");
            let other = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            tap(&mut g, who);
            match mode {
                0 => g.state.objects[bow as usize].attached_to = None,
                1 => assert!(g.attach_equipment(g.object_ref(bow), g.object_ref(other))),
                2 => g.record_control(g.object_ref(bow), p.other(), None),
                _ => moved(&mut g, &[bow], Zone::Exile),
            }
            settle(&mut g);
            assert_eq!((g.life(p), g.life(p.other())), (21, 19));
        }
    }
}
#[test]
fn msh_event_objects_bow_latest_departure_controller_keywords_colors_all_zones_generations() {
    for p in [Player::P0, Player::P1] {
        for zone in [Zone::Graveyard, Zone::Hand, Zone::Library, Zone::Exile] {
            let mut g = setup(p);
            let (bow, who) = armed(&mut g, p, "Bear Cub");
            let original = g.object_ref(who);
            tap(&mut g, who);
            g.state.objects[who as usize].gained_keywords = Keywords::LIFELINK;
            g.record_control(original, p.other(), None);
            moved(&mut g, &[who], zone);
            let actor = damage_actor(packet(&g, bow));
            assert_eq!(actor.reference, original);
            assert_eq!(actor.controller, p.other());
            assert!(actor.keywords.contains(Keywords::LIFELINK));
            assert!(actor.facts.is_creature());
            assert_eq!(
                actor.facts.colors(),
                Card::from_name("Bear Cub").unwrap().colors()
            );
            moved(&mut g, &[who], Zone::Battlefield);
            assert_ne!(g.object_ref(who), original);
            assert!(!g.keywords(who).contains(Keywords::LIFELINK));
            settle(&mut g);
            assert_eq!((g.life(p), g.life(p.other())), (20, 20));
        }
    }
}
#[test]
fn msh_event_objects_bow_simultaneous_departure_preserves_granted_lifelink_all_orders() {
    for p in [Player::P0, Player::P1] {
        for order in 0..3 {
            let mut g = setup(p);
            let (bow, who) = armed(&mut g, p, "Healer's Hawk");
            let violence = named(&mut g, p, "Gratuitous Violence", Zone::Battlefield);
            tap(&mut g, who);
            let ids = match order {
                0 => vec![who, bow],
                1 => vec![bow, who],
                _ => vec![who],
            };
            moved(&mut g, &ids, Zone::Exile);
            assert!(
                damage_actor(packet(&g, bow))
                    .keywords
                    .contains(Keywords::LIFELINK)
            );
            settle(&mut g);
            assert_eq!((g.life(p), g.life(p.other())), (22, 18));
            assert!(g.on_battlefield(violence));
        }
    }
}
#[test]
fn msh_event_objects_bow_suppression_before_event_after_event_and_creature_removal() {
    for p in [Player::P0, Player::P1] {
        for after in [false, true] {
            let mut g = setup(p);
            let (bow, who) = armed(&mut g, p, "Bear Cub");
            if after {
                tap(&mut g, who);
            }
            blank(&mut g, p, bow);
            assert!(!g.printed_abilities_available(bow));
            assert_eq!(g.stats(who).power, 2);
            if !after {
                tap(&mut g, who);
            }
            settle(&mut g);
            assert_eq!(g.life(p.other()), if after { 19 } else { 20 });
        }
        let mut g = setup(p);
        let (bow, who) = armed(&mut g, p, "Bear Cub");
        blank(&mut g, p, who);
        tap(&mut g, who);
        assert!(has_packet(&g, bow));
        settle(&mut g);
        assert_eq!(g.life(p.other()), 19);
    }
}
#[test]
fn msh_event_objects_bow_noncreature_after_tap_still_deals_current_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (bow, who) = armed(&mut g, p, "Healer's Hawk");
        named(&mut g, p, "Gratuitous Violence", Zone::Battlefield);
        tap(&mut g, who);
        noncreature(&mut g, p, who);
        settle(&mut g);
        assert_eq!((g.life(p), g.life(p.other())), (20, 19));
        assert!(!has_packet(&g, bow));
    }
}
#[test]
fn msh_event_objects_hulkling_paid_body_vigilance_and_initial_or_comparison() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HULK, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!((g.stats(h).power, g.stats(h).toughness), (2, 3));
        assert!(g.has_subtype(h, "Kree") && g.has_subtype(h, "Skrull") && g.has_subtype(h, "Hero"));
        declare(&mut g, p, h);
        assert!(!g.object_view(h).tapped);
    }
}
#[test]
fn msh_event_objects_hulkling_initial_power_or_toughness_live_derived_stats() {
    for p in [Player::P0, Player::P1] {
        for (name, want) in [
            ("Hill Giant", true),
            ("Kree Sentinel", true),
            ("Aegis Turtle", true),
            ("Bear Cub", false),
            ("Healer's Hawk", false),
        ] {
            let mut g = setup(p);
            let h = paid(&mut g, p, HULK, &[]);
            entry(&mut g, p, name);
            assert_eq!(has_packet(&g, h), want, "{name}");
            settle(&mut g);
            assert_eq!(g.counter_count(h, CounterKind::PlusOne), u64::from(want));
        }
        let mut g = setup(p);
        let h = paid(&mut g, p, HULK, &[]);
        let enemy = enters(&mut g, p.other(), "Hill Giant");
        assert!(!has_packet(&g, h));
        g.state.objects[enemy as usize].growth.power = 10;
        assert!(!has_packet(&g, h));
    }
}
#[test]
fn msh_event_objects_hulkling_rechecks_either_axis_and_equalities_signed_stats() {
    for p in [Player::P0, Player::P1] {
        for (hp, ht, ep, et, want) in [
            (2, 3, 2, 3, false),
            (2, 3, 3, 2, true),
            (2, 3, 1, 4, true),
            (2, 3, 1, 3, false),
            (-3, 3, -2, 3, true),
            (-2, 3, -3, 3, false),
        ] {
            let mut g = setup(p);
            let h = paid(&mut g, p, HULK, &[]);
            let e = entry(&mut g, p, "Hill Giant");
            g.state.objects[h as usize].growth = Stats {
                power: hp - 2,
                toughness: ht - 3,
            };
            g.state.objects[e as usize].growth = Stats {
                power: ep - 3,
                toughness: et - 3,
            };
            settle(&mut g);
            assert_eq!(g.counter_count(h, CounterKind::PlusOne), u64::from(want));
        }
    }
}
#[test]
fn msh_event_objects_hulkling_entry_counters_simultaneous_entries_and_source_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HULK, &[]);
        let a = named(&mut g, p, "Hill Giant", Zone::Graveyard);
        let b = named(&mut g, p, "Hill Giant", Zone::Graveyard);
        moved(&mut g, &[a, b], Zone::Battlefield);
        assert_eq!(event_packets(&g).filter(|s| s.card == h).count(), 2);
        settle(&mut g);
        assert_eq!(g.counter_count(h, CounterKind::PlusOne), 1);
        let mut g = setup(p);
        let h = named(&mut g, p, HULK, Zone::Graveyard);
        let e = named(&mut g, p, "Hill Giant", Zone::Graveyard);
        moved(&mut g, &[h, e], Zone::Battlefield);
        assert!(has_packet(&g, h));
        settle(&mut g);
        assert_eq!(g.counter_count(h, CounterKind::PlusOne), 1);
        let mut g = setup(p);
        let h = paid(&mut g, p, HULK, &[]);
        entry(&mut g, p, "Heroes' Bane");
        settle(&mut g);
        assert_eq!(g.counter_count(h, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_event_objects_hulkling_event_control_and_source_control_are_historical() {
    for p in [Player::P0, Player::P1] {
        for change_source in [false, true] {
            let mut g = setup(p);
            let h = paid(&mut g, p, HULK, &[]);
            let e = entry(&mut g, p, "Hill Giant");
            g.record_control(
                g.object_ref(if change_source { h } else { e }),
                p.other(),
                None,
            );
            settle(&mut g);
            assert_eq!(g.counter_count(h, CounterKind::PlusOne), 1);
        }
    }
}
#[test]
fn msh_event_objects_hulkling_latest_entrant_departure_and_reentry_all_zones() {
    for p in [Player::P0, Player::P1] {
        for (zone, power, want) in [
            (Zone::Hand, 2, false),
            (Zone::Graveyard, 4, true),
            (Zone::Library, 2, false),
            (Zone::Exile, 4, true),
        ] {
            let mut g = setup(p);
            let h = paid(&mut g, p, HULK, &[]);
            let e = entry(&mut g, p, "Hill Giant");
            let original = g.object_ref(e);
            g.state.objects[e as usize].growth = Stats {
                power: power - 3,
                toughness: 0,
            };
            moved(&mut g, &[e], zone);
            assert_eq!(entrant_facts(packet(&g, h)).power, power);
            moved(&mut g, &[e], Zone::Battlefield);
            assert_ne!(g.object_ref(e), original);
            // Reentry itself creates a second independent trigger. Remove only that new packet from this core fixture.
            g.state
                .pending_triggers
                .retain(|s| s.event_object.as_ref().is_none_or(|v| v.object == original));
            settle(&mut g);
            assert_eq!(g.counter_count(h, CounterKind::PlusOne), u64::from(want));
        }
    }
}
#[test]
fn msh_event_objects_hulkling_both_latest_departures_and_new_source_exclusion() {
    for p in [Player::P0, Player::P1] {
        for source_first in [false, true] {
            let mut g = setup(p);
            let h = paid(&mut g, p, HULK, &[]);
            let e = entry(&mut g, p, "Hill Giant");
            g.state.objects[h as usize].growth = Stats {
                power: 4,
                toughness: 4,
            };
            g.state.objects[e as usize].growth = Stats {
                power: 3,
                toughness: 4,
            };
            moved(
                &mut g,
                &if source_first { vec![h, e] } else { vec![e, h] },
                Zone::Exile,
            );
            let s = packet(&g, h);
            let source = s
                .source_quantity_lki
                .as_ref()
                .unwrap()
                .creature_comparison
                .unwrap();
            assert_eq!((source.power, source.toughness), (6, 7));
            assert_eq!((entrant_facts(s).power, entrant_facts(s).toughness), (6, 7));
            assert!(!g.event_object_guard(s));
            moved(&mut g, &[h], Zone::Battlefield);
            settle(&mut g);
            assert_eq!(g.counter_count(h, CounterKind::PlusOne), 0);
        }
    }
}
#[test]
fn msh_event_objects_hulkling_prebatch_statics_and_entry_suppression() {
    for p in [Player::P0, Player::P1] {
        for order in [false, true] {
            let mut g = setup(p);
            let h = paid(&mut g, p, HULK, &[]);
            let lord = named(&mut g, p, "Anthem of Champions", Zone::Battlefield);
            let e = entry(&mut g, p, "Hill Giant");
            moved(
                &mut g,
                &if order {
                    vec![lord, h, e]
                } else {
                    vec![e, h, lord]
                },
                Zone::Exile,
            );
            let s = packet(&g, h);
            assert_eq!(
                s.source_quantity_lki
                    .as_ref()
                    .unwrap()
                    .creature_comparison
                    .unwrap()
                    .power,
                3
            );
            assert_eq!(entrant_facts(s).power, 4);
            assert!(g.event_object_guard(s));
            settle(&mut g);
        }
        let mut g = setup(p);
        let h = paid(&mut g, p, HULK, &[]);
        named(&mut g, p, "Torpor Orb", Zone::Battlefield);
        enters(&mut g, p, "Hill Giant");
        assert!(!has_packet(&g, h));
    }
}
#[test]
fn msh_event_objects_hulkling_ability_loss_before_and_after_trigger_noncreatures_fail() {
    for p in [Player::P0, Player::P1] {
        for after in [false, true] {
            let mut g = setup(p);
            let h = paid(&mut g, p, HULK, &[]);
            if after {
                entry(&mut g, p, "Hill Giant");
            }
            blank(&mut g, p, h);
            if !after {
                enters(&mut g, p, "Hill Giant");
            }
            settle(&mut g);
            assert_eq!(g.counter_count(h, CounterKind::PlusOne), u64::from(after));
        }
        for source in [false, true] {
            let mut g = setup(p);
            let h = paid(&mut g, p, HULK, &[]);
            let e = entry(&mut g, p, "Hill Giant");
            noncreature(&mut g, p, if source { h } else { e });
            settle(&mut g);
            assert_eq!(g.counter_count(h, CounterKind::PlusOne), 0);
        }
    }
}
#[test]
fn msh_event_objects_paid_removal_freezes_facts_without_disabling_bow_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (bow, who) = armed(&mut g, p, "Healer's Hawk");
        tap(&mut g, who);
        // Put the original trigger on the stack, then cast an actual instant above it.
        until(&mut g, |g| g.state.stack.iter().any(|s| s.card == bow));
        priority(&mut g, p);
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        cast(&mut g, murder, None, &[Target::Card(who)]);
        settle(&mut g);
        assert_eq!(g.object_view(who).zone, Zone::Graveyard);
        assert_eq!((g.life(p), g.life(p.other())), (21, 19));
    }
}
#[test]
fn msh_event_objects_public_contexts_full_nonactor_privacy_and_orphan_restore() {
    for p in [Player::P0, Player::P1] {
        for bow_case in [false, true] {
            let mut g = setup(p);
            let (source, event) = if bow_case {
                let (s, e) = armed(&mut g, p, "Bear Cub");
                tap(&mut g, e);
                (s, e)
            } else {
                let s = paid(&mut g, p, HULK, &[]);
                let e = entry(&mut g, p, "Hill Giant");
                (s, e)
            };
            let secret = named(&mut g, p, "Murder", Zone::Hand);
            let mut different = g.clone();
            different.state.objects[secret as usize].card = Card::from_name("Divination").unwrap();
            assert_eq!(
                serde_json::to_value(g.observe(p.other())).unwrap(),
                serde_json::to_value(different.observe(p.other())).unwrap()
            );
            let before = g.snapshot();
            let mut orphan = {
                let parent = g.clone();
                parent.clone()
            };
            if bow_case {
                orphan.state.objects[event as usize].gained_keywords = Keywords::LIFELINK;
            } else {
                orphan.state.objects[event as usize].growth.power = 5;
            }
            moved(&mut orphan, &[event], Zone::Hand);
            assert_eq!(g.snapshot(), before);
            if bow_case {
                assert!(
                    !damage_actor(packet(&g, source))
                        .keywords
                        .contains(Keywords::LIFELINK)
                );
                assert!(
                    damage_actor(packet(&orphan, source))
                        .keywords
                        .contains(Keywords::LIFELINK)
                );
            } else {
                assert_eq!(entrant_facts(packet(&g, source)).power, 3);
                assert_eq!(entrant_facts(packet(&orphan, source)).power, 8);
            }
            let departure = orphan.snapshot();
            settle(&mut orphan);
            let result = orphan.snapshot();
            orphan.restore(&departure).unwrap();
            settle(&mut orphan);
            assert_eq!(orphan.snapshot(), result);
            assert_eq!(orphan.observe(p).stack, orphan.observe(p.other()).stack);
        }
    }
}
#[test]
fn msh_event_objects_copied_packets_own_exact_references_and_independent_controllers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (bow, who) = armed(&mut g, p, "Healer's Hawk");
        tap(&mut g, who);
        let mut copy = packet(&g, bow).clone();
        copy.id = g.next_stack_id();
        copy.controller = p.other();
        copy.is_copy = true;
        g.state.pending_triggers.push(copy);
        moved(&mut g, &[who], Zone::Exile);
        settle(&mut g);
        assert_eq!((g.life(p), g.life(p.other())), (21, 19)); // Two lifelink gains, one loss each.
        let mut g = setup(p);
        let h = paid(&mut g, p, HULK, &[]);
        entry(&mut g, p, "Hill Giant");
        let mut copy = packet(&g, h).clone();
        copy.id = g.next_stack_id();
        copy.is_copy = true;
        g.state.pending_triggers.push(copy);
        settle(&mut g);
        assert_eq!(g.counter_count(h, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_event_objects_packet_owned_bytes_absent_old_payloads_and_funding_stats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HULK, &[]);
        entry(&mut g, p, "Hill Giant");
        let packet = packet(&g, h);
        let bytes = packet.owned_bytes();
        let mut without = packet.clone();
        without.event_object = None;
        assert_eq!(
            bytes - without.owned_bytes(),
            std::mem::size_of::<EventObjectView>()
        );
        let before = g.snapshot();
        let production = ManaProduction::SourcePower { color: 4 };
        assert_eq!(
            production.in_world(&g, h),
            ManaProduction::Fixed([0, 0, 0, 0, 2, 0])
        );
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(g.can_fund(goal));
        assert_eq!(g.snapshot(), before);
        settle(&mut g);
        assert_eq!(
            production.in_world(&g, h),
            ManaProduction::Fixed([0, 0, 0, 0, 3, 0])
        );
    }
}

#[test]
fn msh_event_objects_bow_crew_multiple_bows_and_controller_batches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (bow, who) = armed(&mut g, p, "Bear Cub");
        let second = paid(&mut g, p, BOW, &[]);
        equip(&mut g, p, second, who);
        g.record_control(g.object_ref(second), p.other(), None);
        let vehicle = named(&mut g, p, "Detention Chariot", Zone::Battlefield);
        g.state.players[p.index()].mana = [30; 6];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: vehicle,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(who);
        checked(&mut g, Action::ChooseCrewer { object });
        assert!(!g.object_view(who).tapped);
        checked(&mut g, Action::FinishCrew);
        assert!(has_packet(&g, bow) && has_packet(&g, second));
        assert_eq!(g.life(p), 20);
        assert_eq!(g.life(p.other()), 20);
        settle(&mut g);
        assert_eq!((g.life(p), g.life(p.other())), (19, 19));
    }
}
#[test]
fn msh_event_objects_bow_teamwork_paid_tap_commits_before_spell_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (bow, who) = armed(&mut g, p, "Bear Cub");
        let other = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Team Tactics", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseTeamwork { pay: true });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(who),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        let clause = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|r| r.clause == CastCostClause::Teamwork(1))
            .unwrap() as u16;
        checked(&mut g, Action::ChooseCastCost { clause });
        for id in [who, other] {
            let object = g.object_ref(id);
            checked(&mut g, Action::ChooseTeammate { object });
        }
        assert!(!has_packet(&g, bow));
        checked(&mut g, Action::FinishTeamwork);
        assert!(has_packet(&g, bow));
        assert_eq!(g.life(p.other()), 20);
        until(&mut g, |g| g.life(p.other()) == 19);
        assert!(g.state.stack.iter().any(|s| s.card == spell));
        settle(&mut g);
    }
}
#[test]
fn msh_event_objects_bow_damage_lifelink_gain_modifiers_and_player_hexproof() {
    for p in [Player::P0, Player::P1] {
        for prevent in [false, true] {
            let mut g = setup(p);
            let (_, who) = armed(&mut g, p, "Healer's Hawk");
            named(
                &mut g,
                p,
                if prevent {
                    "Giant Cindermaw"
                } else {
                    "Angel of Vitality"
                },
                Zone::Battlefield,
            );
            named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
            let priest = named(&mut g, p, "Ajani's Pridemate", Zone::Battlefield);
            tap(&mut g, who);
            settle(&mut g);
            assert_eq!(g.life(p.other()), 19);
            assert_eq!(g.life(p), if prevent { 20 } else { 22 });
            assert_eq!(
                g.counter_count(priest, CounterKind::PlusOne),
                if prevent { 0 } else { 1 }
            );
        }
    }
}
#[test]
fn msh_event_objects_hulkling_departed_noncreature_facts_never_compare_as_zero() {
    for p in [Player::P0, Player::P1] {
        for source in [false, true] {
            let mut g = setup(p);
            let h = paid(&mut g, p, HULK, &[]);
            let e = entry(&mut g, p, "Hill Giant");
            let id = if source { h } else { e };
            let aura = noncreature(&mut g, p, id);
            moved(&mut g, &[id, aura], Zone::Exile);
            let packet = packet(&g, h);
            assert!(!g.event_object_guard(packet));
            if source {
                assert!(
                    !packet
                        .source_quantity_lki
                        .as_ref()
                        .unwrap()
                        .creature_comparison
                        .unwrap()
                        .creature
                );
            } else {
                assert!(!entrant_facts(packet).creature);
            }
            settle(&mut g);
            assert_eq!(g.counter_count(h, CounterKind::PlusOne), 0);
        }
    }
}
#[test]
fn msh_event_objects_copied_permanent_recipe_uses_its_own_ability_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let original = paid(&mut g, p, BOW, &[]);
        let definition = g.object_view(original).card;
        let copy = g.create_copy_groups_inner(
            p.other(),
            &[TokenGroupView {
                token: definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(definition))],
            None,
        )[0]
        .card;
        g.advance_entries();
        let who = named(&mut g, p, "Healer's Hawk", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(copy), g.object_ref(who)));
        tap(&mut g, who);
        let packet = packet(&g, copy);
        assert_eq!(packet.object, g.object_ref(copy));
        assert_eq!(packet.controller, p.other());
        assert_eq!(
            packet.event_object.as_ref().unwrap().object,
            g.object_ref(who)
        );
        assert!(packet.creature_combat_event.is_none() && packet.damage_event.is_none());
        moved(&mut g, &[who, copy], Zone::Exile);
        settle(&mut g);
        assert_eq!((g.life(p), g.life(p.other())), (20, 20));
    }
}

#[test]
fn msh_event_objects_paid_tap_effect_survives_creature_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (_, who) = armed(&mut g, p, "Healer's Hawk");
        paid(&mut g, p, "Stop Cold", &[Target::Card(who)]);
        assert!(g.object_view(who).tapped);
        assert!(!g.printed_abilities_available(who));
        assert_eq!((g.life(p), g.life(p.other())), (20, 19));
    }
}
#[test]
fn msh_event_objects_hulkling_token_entrant_ceases_and_new_token_keeps_original_facts() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HULK, &[]);
        let definition = Card::from_name("Hill Giant").unwrap();
        let token = g.create_copy_groups_inner(
            p,
            &[TokenGroupView {
                token: definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(definition))],
            None,
        )[0]
        .card;
        g.advance_entries();
        assert!(has_packet(&g, h));
        let original = g.object_ref(token);
        g.state.objects[token as usize].growth.power = 4;
        moved(&mut g, &[token], Zone::Exile);
        assert_eq!(entrant_facts(packet(&g, h)).power, 7);
        let replacement = g.create_copy_groups_inner(
            p.other(),
            &[TokenGroupView {
                token: Card::from_name("Healer's Hawk").unwrap(),
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(
                Card::from_name("Healer's Hawk").unwrap(),
            ))],
            None,
        )[0];
        g.advance_entries();
        assert_ne!(replacement, original);
        settle(&mut g);
        assert_eq!(g.counter_count(h, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_event_objects_copied_hulkling_observes_its_controller_and_source_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let original = paid(&mut g, p, HULK, &[]);
        let definition = g.object_view(original).card;
        let copy = g.create_copy_groups_inner(
            p.other(),
            &[TokenGroupView {
                token: definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(definition))],
            None,
        )[0]
        .card;
        g.advance_entries();
        next_turn(&mut g);
        until(&mut g, |g| g.state.step == Step::Main1);
        entry(&mut g, p.other(), "Hill Giant");
        assert!(has_packet(&g, copy));
        assert!(!has_packet(&g, original));
        moved(&mut g, &[original], Zone::Exile);
        settle(&mut g);
        assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 1);
    }
}

#[test]
fn msh_event_objects_bow_new_and_removed_lifelink_grants_freeze_before_entire_batch() {
    for p in [Player::P0, Player::P1] {
        for first in [false, true] {
            let mut g = setup(p);
            let (bow, who) = armed(&mut g, p, "Bear Cub");
            tap(&mut g, who);
            let collar = named(&mut g, p, "Basilisk Collar", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(collar), g.object_ref(who)));
            moved(
                &mut g,
                &if first {
                    vec![collar, bow, who]
                } else {
                    vec![who, bow, collar]
                },
                Zone::Exile,
            );
            assert!(
                damage_actor(packet(&g, bow))
                    .keywords
                    .contains(Keywords::LIFELINK)
            );
            settle(&mut g);
            assert_eq!((g.life(p), g.life(p.other())), (21, 19));
        }
        let mut g = setup(p);
        let (bow, who) = armed(&mut g, p, "Bear Cub");
        let collar = named(&mut g, p, "Basilisk Collar", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(collar), g.object_ref(who)));
        tap(&mut g, who);
        moved(&mut g, &[collar], Zone::Exile);
        moved(&mut g, &[who], Zone::Exile);
        assert!(
            !damage_actor(packet(&g, bow))
                .keywords
                .contains(Keywords::LIFELINK)
        );
        settle(&mut g);
        assert_eq!((g.life(p), g.life(p.other())), (20, 19));
    }
}
struct EventPowerWorld<'a> {
    game: &'a Game,
    objects: Vec<(CardId, Object)>,
}
impl ability_world::CharacteristicWorld for EventPowerWorld<'_> {
    fn game(&self) -> &Game {
        self.game
    }
    fn object(&self, id: CardId) -> &Object {
        self.objects
            .iter()
            .find(|(key, _)| *key == id)
            .map_or(&self.game.state.objects[id as usize], |(_, o)| o)
    }
    fn life(&self, p: Player) -> i128 {
        self.game.life(p)
    }
    fn gained_life(&self, _: Player) -> bool {
        false
    }
    fn attacking(&self, id: CardId) -> bool {
        self.game.state.attackers.iter().any(|a| a.card == id)
    }
}
#[test]
fn msh_event_objects_borrowed_funding_power_recomputes_without_mutating_packets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (bow, who) = armed(&mut g, p, "Topiary Lecturer");
        tap(&mut g, who);
        let before = g.snapshot();
        let mut gone = g.state.objects[bow as usize].clone();
        gone.zone = Zone::Exile;
        let world = EventPowerWorld {
            game: &g,
            objects: vec![(bow, gone)],
        };
        let production = ManaProduction::SourcePower { color: 4 };
        assert_eq!(
            production.in_world(&g, who),
            ManaProduction::Fixed([0, 0, 0, 0, 2, 0])
        );
        assert_eq!(
            production.in_world(&world, who),
            ManaProduction::Fixed([0, 0, 0, 0, 1, 0])
        );
        assert_eq!(g.snapshot(), before);
        let mut g = setup(p);
        let h = paid(&mut g, p, HULK, &[]);
        entry(&mut g, p, "Hill Giant");
        let before = g.snapshot();
        let mut projected = g.state.objects[h as usize].clone();
        projected.counters.plus_one = 5;
        let world = EventPowerWorld {
            game: &g,
            objects: vec![(h, projected)],
        };
        // Test the shared quantity evaluator; Hulkling has no printed mana activation.
        assert_eq!(
            production.in_world(&world, h),
            ManaProduction::Fixed([0, 0, 0, 0, 7, 0])
        );
        assert_eq!(
            production.in_world(&g, h),
            ManaProduction::Fixed([0, 0, 0, 0, 2, 0])
        );
        assert_eq!(g.snapshot(), before);
    }
}
