//! Ordinary paid hand-interaction scenarios using complete typed definitions.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
use crate::CreatureType;
const KLAW: &str = "Klaw, Sonic Subjugator";
const CLOAK: &str = "Cloak and Dagger, Entwined";
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut v = g.observe(p);
        v.revision = 0;
        serde_json::to_value(v).unwrap()
    })
}
fn reject(g: &mut Game, a: Action) {
    let before = (g.snapshot(), g.revision(), g.state_bytes(), views(g));
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(
        (g.snapshot(), g.revision(), g.state_bytes(), views(g)),
        before
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let ix = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let before = g.snapshot();
    let oldviews = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, ix).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), oldviews);
    assert_eq!(raw.apply_index(actor, rev, ix), Err(Error::StaleRevision));
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn main(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.passes = 0;
}
fn moved(g: &mut Game, ids: &[CardId], to: Zone) {
    let before = g.death_trigger_sources();
    let events = g.move_objects_raw(ids, to);
    g.finish_zone_events(&events, &before);
}
fn clear_hand(g: &mut Game, p: Player) {
    let ids = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Hand && o.owner == p)
        .map(|(i, _)| i as CardId)
        .collect::<Vec<_>>();
    moved(g, &ids, Zone::Outside);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..160 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("hand state stalled {:?}", g.state.pending)
}
fn arrive(g: &mut Game, p: Player, name: &str) -> CardId {
    main(g, p);
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "whole-card definition must be supported");
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, n) = c.cost();
    mana[5] += n;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    until(g, |g| g.on_battlefield(id));
    id
}
fn targets(g: &mut Game, targets: &[Target]) {
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    for &target in targets {
        if !g.legal_actions().contains(&Action::ChooseTarget { target }) {
            if let Some(a) = &g.state.announcing {
                panic!(
                    "missing target {target:?} at {:?} group{} spec{:?} chosen{:?} base={} relation={}",
                    g.state.pending,
                    a.group,
                    a.spec,
                    a.item.targets,
                    g.item_target_matches(&a.item, a.spec.rule, target),
                    g.target_group_compatible(a.spec.rule, a.group, &a.item.targets, target)
                );
            }
            panic!(
                "missing target {target:?} without announcement at {:?}",
                g.state.pending
            );
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    panic!("trigger not announced")
}
fn hand(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::HandCardChoice { .. })
    });
}
fn choose(g: &mut Game, card: Option<CardId>) {
    let object = card.map(|id| g.object_ref(id));
    checked(g, Action::ChooseHandCard { object });
}
fn known(g: &Game, id: CardId) -> bool {
    g.state.known_hand.contains(&g.object_ref(id))
}
fn cloak(g: &mut Game, p: Player, creature: Option<CardId>) -> CardId {
    let s = arrive(g, p, CLOAK);
    let targets_: Vec<_> = std::iter::once(Target::Player(p.other()))
        .chain(creature.map(Target::Card))
        .collect();
    targets(g, &targets_);
    s
}
fn packet(g: &Game, s: CardId) -> &SpellView {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .find(|v| v.card == s)
        .unwrap()
}
#[test]
fn msh_hand_choices_klaw_body_actual_cost_and_two_phase_subset_discard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p.other());
        named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let a = named(&mut g, p.other(), "Shock", Zone::Hand);
        let b = named(&mut g, p.other(), "Forest", Zone::Hand);
        let hidden = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
        let s = arrive(&mut g, p, KLAW);
        let v = g.object_view(s);
        assert_eq!((v.power, v.toughness), (2, 2));
        assert!(v.card.is_legendary());
        for t in ["Human", "Rogue", "Villain"] {
            assert!(v.has_subtype(t));
        }
        targets(&mut g, &[Target::Player(p.other())]);
        hand(&mut g);
        assert_eq!(g.actor(), Some(p.other()));
        assert_eq!(g.observe(p.other()).hand_choice.unwrap().required, 2);
        assert!(g.observe(p).hand_choice.is_none());
        choose(&mut g, Some(a));
        assert!(!known(&g, a) && !known(&g, b) && !known(&g, hidden));
        let duplicate = g.object_ref(a);
        reject(
            &mut g,
            Action::ChooseHandCard {
                object: Some(duplicate),
            },
        );
        reject(&mut g, Action::ChooseHandCard { object: None });
        choose(&mut g, Some(b));
        assert_eq!(g.actor(), Some(p));
        assert!(known(&g, a) && known(&g, b) && !known(&g, hidden));
        let bad = g.object_ref(hidden);
        reject(&mut g, Action::ChooseHandCard { object: Some(bad) });
        choose(&mut g, Some(a));
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_view(b).zone, Zone::Hand);
        assert_eq!(g.object_view(hidden).zone, Zone::Hand);
    }
}
#[test]
fn msh_hand_choices_klaw_empty_short_all_and_land_mandatory_discard() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1, 2] {
            let mut g = setup(p);
            clear_hand(&mut g, p.other());
            for _ in 0..4 {
                named(&mut g, p, "Bear Cub", Zone::Graveyard);
            }
            let ids = (0..count)
                .map(|_| named(&mut g, p.other(), "Forest", Zone::Hand))
                .collect::<Vec<_>>();
            arrive(&mut g, p, KLAW);
            targets(&mut g, &[Target::Player(p.other())]);
            if count == 0 {
                settle(&mut g);
                assert!(g.observe(p).hand_choice.is_none());
            } else {
                hand(&mut g);
                assert_eq!(g.actor(), Some(p));
                assert_eq!(
                    g.observe(p).hand_choice.unwrap().kind,
                    HandChoiceKind::DiscardRevealed
                );
                for &id in &ids {
                    assert!(known(&g, id));
                }
                reject(&mut g, Action::ChooseHandCard { object: None });
                choose(&mut g, Some(ids[0]));
                settle(&mut g);
                assert_eq!(g.object_view(ids[0]).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn msh_hand_choices_klaw_self_target_and_resolution_graveyard_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p);
        let a = named(&mut g, p, "Shock", Zone::Hand);
        let b = named(&mut g, p, "Forest", Zone::Hand);
        let c = named(&mut g, p, "Mountain", Zone::Hand);
        let s = arrive(&mut g, p, KLAW);
        targets(&mut g, &[Target::Player(p)]);
        moved(&mut g, &[s], Zone::Graveyard);
        hand(&mut g);
        let view = g.observe(p).hand_choice.unwrap();
        assert_eq!(
            (view.kind, view.required),
            (HandChoiceKind::RevealSubset, 2)
        );
        choose(&mut g, Some(a));
        choose(&mut g, Some(b));
        assert!(!known(&g, c));
        choose(&mut g, Some(b));
        settle(&mut g);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.object_view(a).zone, Zone::Hand);
    }
}
#[test]
fn msh_hand_choices_klaw_graveyard_owner_card_kind_and_token_filter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p.other());
        let creature = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.state.objects[creature as usize].controller = p.other();
        named(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
        named(&mut g, p, "Shock", Zone::Graveyard);
        let token = named(&mut g, p, "@token/msh-robot-villain", Zone::Graveyard);
        assert!(g.state.objects[token as usize].is_token());
        for _ in 0..4 {
            named(&mut g, p.other(), "Forest", Zone::Hand);
        }
        arrive(&mut g, p, KLAW);
        targets(&mut g, &[Target::Player(p.other())]);
        hand(&mut g);
        assert_eq!(g.observe(p.other()).hand_choice.unwrap().required, 2);
        settle(&mut g);
    }
}
#[test]
fn msh_hand_choices_klaw_source_control_blink_and_captured_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p.other());
        let a = named(&mut g, p.other(), "Shock", Zone::Hand);
        let b = named(&mut g, p.other(), "Forest", Zone::Hand);
        let s = arrive(&mut g, p, KLAW);
        targets(&mut g, &[Target::Player(p.other())]);
        let old = g.object_ref(s);
        g.change_controller(old, p.other());
        moved(&mut g, &[s], Zone::Exile);
        hand(&mut g);
        assert_eq!(g.actor(), Some(p.other()));
        choose(&mut g, Some(a));
        assert_eq!(g.actor(), Some(p));
        choose(&mut g, Some(a));
        settle(&mut g);
        assert_eq!(g.object_view(b).zone, Zone::Hand);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert!(!g.on_battlefield(s));
    }
}
#[test]
fn msh_hand_choices_klaw_discard_caused_by_controller_preserves_replacements() {
    for p in [Player::P0, Player::P1] {
        for rip in [false, true] {
            let mut g = setup(p);
            clear_hand(&mut g, p.other());
            if rip {
                named(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            let d = named(
                &mut g,
                p.other(),
                if rip { "Shock" } else { "Wilt-Leaf Liege" },
                Zone::Hand,
            );
            arrive(&mut g, p, KLAW);
            targets(&mut g, &[Target::Player(p.other())]);
            hand(&mut g);
            choose(&mut g, Some(d));
            settle(&mut g);
            assert_eq!(
                g.object_view(d).zone,
                if rip { Zone::Exile } else { Zone::Battlefield }
            );
        }
    }
}
#[test]
fn msh_hand_choices_klaw_illegal_player_target_has_no_reveal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = named(&mut g, p.other(), "Shock", Zone::Hand);
        arrive(&mut g, p, KLAW);
        targets(&mut g, &[Target::Player(p.other())]);
        named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
        settle(&mut g);
        assert!(!known(&g, d));
        assert_eq!(g.object_view(d).zone, Zone::Hand);
    }
}
#[test]
fn msh_hand_choices_klaw_private_selection_counterfactual_orphan_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p.other());
        let a = named(&mut g, p.other(), "Shock", Zone::Hand);
        let b = named(&mut g, p.other(), "Forest", Zone::Hand);
        arrive(&mut g, p, KLAW);
        targets(&mut g, &[Target::Player(p.other())]);
        hand(&mut g);
        let mut counter = g.clone();
        counter.state.objects[a as usize].card = Card::from_name("Giant Growth").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(counter.observe(p)).unwrap()
        );
        let reveal = Action::ChooseHandCard {
            object: Some(g.object_ref(a)),
        };
        g = orphan(g, reveal);
        assert!(known(&g, a) && !known(&g, b));
        let discard = Action::ChooseHandCard {
            object: Some(g.object_ref(a)),
        };
        g = orphan(g, discard);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_hand_choices_cloak_full_reveal_optional_decline_and_nonland_filter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p.other(), "Forest", Zone::Hand);
        let a = named(&mut g, p.other(), "Shock", Zone::Hand);
        let foreign = named(&mut g, p, "Giant Growth", Zone::Hand);
        let s = cloak(&mut g, p, None);
        hand(&mut g);
        let v = g.object_view(s);
        assert_eq!((v.power, v.toughness), (2, 2));
        assert!(v.card.is_legendary() && v.has_subtype("Hero") && v.has_subtype("Human"));
        assert!(v
            .keywords
            .contains(Keywords::DEATHTOUCH.union(Keywords::LIFELINK)));
        assert!(known(&g, land) && known(&g, a));
        for id in [land, foreign] {
            let r = g.object_ref(id);
            reject(&mut g, Action::ChooseHandCard { object: Some(r) });
        }
        choose(&mut g, None);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Hand);
        assert!(g.state.exile_links.is_empty());
    }
}
#[test]
fn msh_hand_choices_cloak_hand_exile_exact_link_returns_to_hand_on_source_leave() {
    for p in [Player::P0, Player::P1] {
        for zone in [Zone::Hand, Zone::Exile, Zone::Graveyard] {
            let mut g = setup(p);
            let a = named(&mut g, p.other(), "Shock", Zone::Hand);
            let s = cloak(&mut g, p, None);
            let old = g.object_ref(s);
            hand(&mut g);
            choose(&mut g, Some(a));
            settle(&mut g);
            assert_eq!(g.object_view(a).zone, Zone::Exile);
            assert_eq!(g.state.exile_links.len(), 1);
            assert_eq!(g.state.exile_links[0].return_zone, Zone::Hand);
            g.change_controller(old, p.other());
            moved(&mut g, &[s], zone);
            settle(&mut g);
            assert_eq!(g.object_view(a).zone, Zone::Hand);
            assert_eq!(g.object_view(a).owner, p.other());
            assert!(g.state.exile_links.is_empty());
        }
    }
}
#[test]
fn msh_hand_choices_cloak_creature_exile_returns_owner_new_object_without_counters_or_equipment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.change_controller(g.object_ref(victim), p.other());
        g.state.objects[victim as usize].counters.plus_one = 3;
        let old = g.object_ref(victim);
        let sword = named(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        g.attach_equipment(g.object_ref(sword), old);
        let s = cloak(&mut g, p, Some(victim));
        hand(&mut g);
        choose(&mut g, Some(victim));
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        assert!(g.state.objects[sword as usize].attached_to.is_none());
        moved(&mut g, &[s], Zone::Exile);
        settle(&mut g);
        assert!(g.on_battlefield(victim));
        assert_eq!(g.object_view(victim).controller, p);
        assert_ne!(g.object_ref(victim), old);
        assert_eq!(g.counter_count(victim, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_hand_choices_cloak_partial_targets_independent_player_or_creature_legality() {
    for p in [Player::P0, Player::P1] {
        for invalid_player in [false, true] {
            for invalid_creature in [false, true] {
                let mut g = setup(p);
                let a = named(&mut g, p.other(), "Shock", Zone::Hand);
                let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                cloak(&mut g, p, Some(victim));
                if invalid_player {
                    named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
                }
                if invalid_creature {
                    g.change_controller(g.object_ref(victim), p);
                }
                if invalid_player && invalid_creature {
                    settle(&mut g);
                    assert!(!known(&g, a));
                    assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
                    continue;
                }
                hand(&mut g);
                assert_eq!(known(&g, a), !invalid_player);
                let selected = if invalid_player { victim } else { a };
                choose(&mut g, Some(selected));
                settle(&mut g);
                assert_eq!(g.object_view(selected).zone, Zone::Exile);
            }
        }
    }
}
#[test]
fn msh_hand_choices_cloak_source_gone_or_reentered_still_reveals_without_exile() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = setup(p);
            let a = named(&mut g, p.other(), "Shock", Zone::Hand);
            let s = cloak(&mut g, p, None);
            let old = g.object_ref(s);
            moved(&mut g, &[s], Zone::Exile);
            if reenter {
                moved(&mut g, &[s], Zone::Battlefield);
                assert_ne!(g.object_ref(s), old);
                // The new incarnation gets its own actual ETB, which resolves
                // first. Decline that new effect, then resolve the old packet.
                targets(&mut g, &[Target::Player(p.other())]);
                hand(&mut g);
                choose(&mut g, None);
            }
            settle(&mut g);
            assert!(known(&g, a));
            assert_eq!(g.object_view(a).zone, Zone::Hand);
            assert!(g.state.exile_links.is_empty());
            assert_ne!(g.object_ref(s), old);
        }
    }
}

#[test]
fn msh_hand_choices_cloak_target_relationship_mandatory_opponent_and_optional_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let theirs = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        arrive(&mut g, p, CLOAK);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(theirs),
            },
        );
        settle(&mut g);
        assert!(g.on_battlefield(theirs));
    }
}
#[test]
fn msh_hand_choices_cloak_exiled_token_ceases_and_never_returns() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.create_tokens(
            p.other(),
            Card::from_name("@token/msh-robot-villain").unwrap(),
            1,
            false,
        );
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
            .unwrap()
            .0 as CardId;
        let s = cloak(&mut g, p, Some(token));
        hand(&mut g);
        choose(&mut g, Some(token));
        settle(&mut g);
        assert!(!g.on_battlefield(token));
        moved(&mut g, &[s], Zone::Exile);
        settle(&mut g);
        assert!(!g.on_battlefield(token));
        assert!(g.state.exile_links.is_empty());
    }
}
#[test]
fn msh_hand_choices_cloak_old_exile_object_moved_and_source_blink_do_not_rebind() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p.other(), "Shock", Zone::Hand);
        let s = cloak(&mut g, p, None);
        hand(&mut g);
        choose(&mut g, Some(a));
        settle(&mut g);
        let exiled = g.object_ref(a);
        moved(&mut g, &[a], Zone::Graveyard);
        moved(&mut g, &[a], Zone::Exile);
        assert_ne!(g.object_ref(a), exiled);
        moved(&mut g, &[s], Zone::Exile);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
        assert!(g.state.exile_links.is_empty());
    }
}
#[test]
fn msh_hand_choices_cloak_public_exile_choice_owned_orphan_and_atomic_failures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p.other(), "Shock", Zone::Hand);
        let foreign = named(&mut g, p, "Giant Growth", Zone::Hand);
        cloak(&mut g, p, None);
        hand(&mut g);
        assert!(g.observe(p.other()).hand_choice.is_none());
        let bad = g.object_ref(foreign);
        reject(&mut g, Action::ChooseHandCard { object: Some(bad) });
        let action = Action::ChooseHandCard {
            object: Some(g.object_ref(a)),
        };
        g = orphan(g, action);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
    }
}
#[test]
fn msh_hand_choices_cloak_ward_countered_ability_does_not_reveal_hand() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p.other(), "Shock", Zone::Hand);
        let v = named(&mut g, p.other(), "Tolarian Terror", Zone::Battlefield);
        let s = arrive(&mut g, p, CLOAK);
        g.state.players[p.index()].mana = [0; 6];
        targets(&mut g, &[Target::Player(p.other()), Target::Card(v)]);
        settle(&mut g);
        assert!(g.on_battlefield(s) && g.on_battlefield(v));
        assert!(!known(&g, a));
        assert!(g.state.exile_links.is_empty());
    }
}
#[test]
fn msh_hand_choices_actual_consumer_costs_require_colored_mana() {
    for p in [Player::P0, Player::P1] {
        for name in [KLAW, CLOAK] {
            let mut g = setup(p);
            main(&mut g, p);
            let s = named(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            reject(&mut g, Action::AnnounceCast { card: s });
            let (mut mana, n) = g.object_view(s).card.cost();
            mana[5] += n;
            g.state.players[p.index()].mana = mana;
            cast(&mut g, s, None, &[]);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn msh_hand_choices_cloak_copied_packets_own_separate_choices_and_shared_exact_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p.other());
        let a = named(&mut g, p.other(), "Shock", Zone::Hand);
        let b = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
        let s = cloak(&mut g, p, None);
        // Explicit queued-ability copy boundary; no copy-card admission is claimed.
        let mut copy = packet(&g, s).clone();
        copy.id = g.next_stack_id();
        copy.is_copy = true;
        g.state.stack.push(copy);
        for id in [a, b] {
            hand(&mut g);
            choose(&mut g, Some(id));
        }
        settle(&mut g);
        assert_eq!(g.state.exile_links.len(), 2);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
        assert_eq!(g.object_view(b).zone, Zone::Exile);
        moved(&mut g, &[s], Zone::Exile);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Hand);
        assert_eq!(g.object_view(b).zone, Zone::Hand);
        assert!(g.state.exile_links.is_empty());
    }
}
#[test]
fn msh_hand_choices_klaw_stale_private_candidate_is_not_new_hand_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p.other());
        let a = named(&mut g, p.other(), "Shock", Zone::Hand);
        let b = named(&mut g, p.other(), "Forest", Zone::Hand);
        arrive(&mut g, p, KLAW);
        targets(&mut g, &[Target::Player(p.other())]);
        hand(&mut g);
        let old = g.object_ref(a);
        // Production movement during a suspended choice is an exact-reference boundary
        // probe, not an assertion that a player can respond midway through resolution.
        moved(&mut g, &[a], Zone::Exile);
        moved(&mut g, &[a], Zone::Hand);
        let new = g.object_ref(a);
        assert_ne!(new, old);
        for reference in [old, new] {
            reject(
                &mut g,
                Action::ChooseHandCard {
                    object: Some(reference),
                },
            );
        }
        choose(&mut g, Some(b));
        choose(&mut g, Some(b));
        settle(&mut g);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.object_view(a).zone, Zone::Hand);
    }
}
#[test]
fn msh_hand_choices_cloak_return_entry_choice_is_owned_by_card_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p, "Adaptive Automaton", Zone::Battlefield);
        g.change_controller(g.object_ref(victim), p.other());
        let s = cloak(&mut g, p, Some(victim));
        hand(&mut g);
        choose(&mut g, Some(victim));
        settle(&mut g);
        // A real paid source bounce pumps the ordinary entry continuation.
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, bounce, None, &[Target::Card(s)]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        assert!(matches!(g.state.pending,Pending::EntryValue{player} if player==p));
        let value = EntryChoiceValue::CreatureType(CreatureType::from_name("Hero").unwrap());
        g = orphan(g, Action::ChooseEntryValue { value });
        settle(&mut g);
        assert_eq!(g.object_view(victim).controller, p);
        assert!(g.has_subtype(victim, "Hero"));
    }
}
#[test]
fn msh_hand_choices_cloak_actual_combat_deathtouch_lifelink_after_own_untap() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let s = cloak(&mut g, p, None);
        settle(&mut g);
        next_turn(&mut g);
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        g.begin_step(Step::BeginCombat);
        until(&mut g, |g| matches!(g.state.pending, Pending::Attackers));
        checked(&mut g, Action::Attack { card: s });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| matches!(g.state.pending, Pending::Blockers));
        checked(
            &mut g,
            Action::Block {
                blocker: victim,
                attacker: s,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        for _ in 0..30 {
            if !g.on_battlefield(s) && !g.on_battlefield(victim) {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::AssignDamage { .. }))
                .unwrap();
            checked(&mut g, a);
        }
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 22);
    }
}
