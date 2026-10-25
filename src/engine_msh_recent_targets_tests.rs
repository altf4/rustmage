//! Paid whole-card scenarios; direct movement probes are explicitly bounded fixtures.
use super::assurance::big_helpers::{checked, named, setup};
use super::*;
const NURSE: &str = "Night Nurse, Healer of Heroes";
const REST: &str = "Restorative Technique";
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..500 {
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
                        | Action::PutTrigger { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unexpected recent-target choice {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("recent-target continuation stalled");
}
fn settle(g: &mut Game) {
    until(g, quiet);
}
fn packet(g: &Game, source: CardId) -> bool {
    g.state.announcing.as_ref().is_some_and(|a| {
        a.item.card == source && matches!(a.item.kind, StackKind::TriggeredAbility { .. })
    })
}
fn finish(g: &mut Game, id: u64) {
    for _ in 0..25 {
        if g.state.announcing.as_ref().is_none_or(|a| a.item.id != id) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseGift { beneficiary: None }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("unpaid recent-target packet {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("unpaid recent-target packet");
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (colored, generic) = g.object_view(id).card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    checked(g, Action::AnnounceCast { card: id });
    let spell = g.state.announcing.as_ref().unwrap().item.id;
    for &target in targets {
        for _ in 0..4 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    finish(g, spell);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn fresh(g: &mut Game, p: Player, name: &str, from: Zone) -> CardId {
    let id = named(g, p, name, from);
    g.move_object(id, Zone::Graveyard);
    assert!(g.graveyard_entry_this_turn(g.object_ref(id)));
    id
}
fn start_nurse(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, NURSE, &[]);
    until(g, |g| packet(g, id));
    id
}
fn choose_nurse(g: &mut Game, id: CardId, target: CardId) {
    assert!(packet(g, id));
    let a = g.state.announcing.as_ref().unwrap().item.id;
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(g, a);
}
fn start_rest(g: &mut Game, p: Player, victim: Player, creature: Option<CardId>) -> CardId {
    let mut targets = vec![Target::Player(victim)];
    if let Some(id) = creature {
        targets.push(Target::Card(id));
    }
    paid(g, p, REST, &targets)
}
fn search(g: &mut Game, player: Player) {
    until(
        g,
        |g| matches!(g.state.pending,Pending::EffectSearch{player:p} if p==player),
    );
    assert_eq!(g.actor(), Some(player));
}
fn find(g: &mut Game, land: Option<CardId>) {
    if let Some(card) = land {
        checked(g, Action::ChooseSearch { card });
    }
    checked(g, Action::FinishSearch);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    let bytes = g.state_bytes();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
    assert_eq!(g.state_bytes(), bytes);
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut o = g.observe(p);
        o.revision = 0;
        serde_json::to_value(o).unwrap()
    })
}
fn orphan(g: Game, action: Action) -> Game {
    let who = g.actor().unwrap();
    let rev = g.revision();
    let before = g.snapshot();
    let before_views = views(&g);
    let at = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(who, action.clone()).unwrap();
    indexed.apply_index(who, rev, at).unwrap();
    let after = raw.snapshot();
    assert_eq!(indexed.snapshot(), after);
    assert_eq!(views(&raw), views(&indexed));
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(raw.apply_index(who, rev, at), Err(Error::StaleRevision));
    raw.apply(who, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
#[test]
fn msh_recent_nurse_paid_flash_after_real_murder_and_lifelink_combat() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(q);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(&mut g, q, "Murder", &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        let n = start_nurse(&mut g, p);
        choose_nurse(&mut g, n, bear);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
        assert_eq!((g.stats(n).power, g.stats(n).toughness), (2, 1));
        assert!(
            g.keywords(n)
                .contains(Keywords::FLASH.union(Keywords::LIFELINK))
        );
        let turn = g.state.turn;
        until(&mut g, |g| {
            g.state.turn > turn && g.state.active == p && g.state.pending == Pending::Attackers
        });
        checked(&mut g, Action::Attack { card: n });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.step == Step::Main2 && quiet(g));
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[q.index()].life, 18);
    }
}
#[test]
fn msh_recent_nurse_all_inventory_permanent_kinds_and_owner_hand_knowledge() {
    // The frozen catalogue has no Battle rows; the reused Permanent predicate
    // already includes that kind, without inventing a playable Battle fixture.
    for p in [Player::P0, Player::P1] {
        for name in [
            "Forest",
            "Bear Cub",
            "Arc Reactor",
            "Heroic Feast",
            "Ajani, Caller of the Pride",
        ] {
            let mut g = setup(p);
            let x = fresh(&mut g, p, name, Zone::Hand);
            let n = start_nurse(&mut g, p);
            choose_nurse(&mut g, n, x);
            settle(&mut g);
            assert_eq!(g.object_view(x).zone, Zone::Hand);
            assert!(g.observe(p.other()).known_hand.contains(&g.object_ref(x)));
        }
    }
}
#[test]
fn msh_recent_nurse_any_origin_uses_actual_final_graveyard_transition() {
    for p in [Player::P0, Player::P1] {
        for from in [
            Zone::Library,
            Zone::Hand,
            Zone::Battlefield,
            Zone::Exile,
            Zone::Stack,
        ] {
            // Bounded production movement probes supplement the real paid
            // death, discard and mill scenarios below; Stack is not a fake cast.
            let mut g = setup(p);
            let x = fresh(&mut g, p, "Bear Cub", from);
            let n = start_nurse(&mut g, p);
            choose_nurse(&mut g, n, x);
            settle(&mut g);
            assert_eq!(g.object_view(x).zone, Zone::Hand);
        }
    }
}
#[test]
fn msh_recent_nurse_rejects_old_opponent_instant_and_wrong_zone_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let valid = fresh(&mut g, p, "Forest", Zone::Hand);
        let old = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let enemy = fresh(&mut g, p.other(), "Arc Reactor", Zone::Hand);
        let instant = fresh(&mut g, p, "Murder", Zone::Hand);
        let hand = named(&mut g, p, "Arc Reactor", Zone::Hand);
        let n = start_nurse(&mut g, p);
        for x in [old, enemy, instant, hand, n] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(x),
                },
            );
        }
        reject(&mut g, Action::FinishTargets);
        choose_nurse(&mut g, n, valid);
        settle(&mut g);
    }
}
#[test]
fn msh_recent_nurse_turn_reset_and_new_incarnation_history_are_exact() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = fresh(&mut g, p, "Forest", Zone::Hand);
        let old_ref = g.object_ref(old);
        let turn = g.state.turn;
        until(&mut g, |g| {
            g.state.turn > turn
                && g.state.active == p.other()
                && g.state.step == Step::Main1
                && quiet(g)
        });
        assert!(!g.graveyard_entry_this_turn(old_ref));
        g.move_object(old, Zone::Exile);
        g.move_object(old, Zone::Graveyard);
        assert_ne!(old_ref, g.object_ref(old));
        assert!(g.graveyard_entry_this_turn(g.object_ref(old)));
        let n = start_nurse(&mut g, p);
        choose_nurse(&mut g, n, old);
        settle(&mut g);
        assert_eq!(g.object_view(old).zone, Zone::Hand);
    }
}
#[test]
fn msh_recent_nurse_target_departure_and_same_slot_reentry_do_not_rebind() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = setup(p);
            let x = fresh(&mut g, p, "Arc Reactor", Zone::Hand);
            let was = g.object_ref(x);
            let n = start_nurse(&mut g, p);
            choose_nurse(&mut g, n, x);
            g.move_object(x, Zone::Exile);
            if reenter {
                g.move_object(x, Zone::Graveyard);
            }
            assert_ne!(was, g.object_ref(x));
            settle(&mut g);
            assert_eq!(
                g.object_view(x).zone,
                if reenter {
                    Zone::Graveyard
                } else {
                    Zone::Exile
                }
            );
        }
    }
}
#[test]
fn msh_recent_nurse_queued_controller_survives_source_death_or_theft() {
    for p in [Player::P0, Player::P1] {
        for die in [false, true] {
            let mut g = setup(p);
            let x = fresh(&mut g, p, "Forest", Zone::Hand);
            let n = start_nurse(&mut g, p);
            choose_nurse(&mut g, n, x);
            if die {
                paid(&mut g, p.other(), "Murder", &[Target::Card(n)]);
                until(&mut g, |g| g.object_view(n).zone == Zone::Graveyard);
            } else {
                g.change_controller(g.object_ref(n), p.other());
            }
            settle(&mut g);
            assert_eq!(g.object_view(x).zone, Zone::Hand);
            assert_eq!(g.object_view(x).owner, p);
        }
    }
}
#[test]
fn msh_recent_nurse_public_target_private_hand_and_orphan_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let x = fresh(&mut g, p, "Arc Reactor", Zone::Hand);
        let secret = named(&mut g, p.other(), "Murder", Zone::Hand);
        let n = start_nurse(&mut g, p);
        let mut hidden = g.clone();
        hidden.state.objects[secret as usize].card = Card::from_name("Shock").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(hidden.observe(p)).unwrap()
        );
        assert_eq!(g.legal_actions(), hidden.legal_actions());
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(x),
            },
        );
        let a = g.state.announcing.as_ref().map(|a| a.item.id);
        if let Some(a) = a {
            finish(&mut g, a);
        }
        assert!(!packet(&g, n));
        g = orphan(g, Action::Pass);
        g = orphan(g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.object_view(x).zone, Zone::Hand);
    }
}
#[test]
fn msh_recent_nurse_real_duress_discard_then_flash_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let x = named(&mut g, p, "Arc Reactor", Zone::Hand);
        paid(&mut g, p.other(), "Duress", &[Target::Player(p)]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseRevealedDiscard {
                object: g.object_ref(x),
            })
        });
        let object = g.object_ref(x);
        checked(&mut g, Action::ChooseRevealedDiscard { object });
        settle(&mut g);
        assert!(g.graveyard_entry_this_turn(g.object_ref(x)));
        let n = start_nurse(&mut g, p);
        choose_nurse(&mut g, n, x);
        settle(&mut g);
        assert_eq!(g.object_view(x).zone, Zone::Hand);
    }
}
#[test]
fn msh_recent_nurse_real_paid_bloodseeker_mill_and_instant_exclusion() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let x = named(&mut g, p, "Arc Reactor", Zone::Library);
        let instant = named(&mut g, p, "Murder", Zone::Library);
        let miller = paid(&mut g, p, "Desperate Bloodseeker", &[]);
        until(&mut g, |g| packet(g, miller));
        let a = g.state.announcing.as_ref().unwrap().item.id;
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        finish(&mut g, a);
        settle(&mut g);
        assert_eq!(g.object_view(x).zone, Zone::Graveyard);
        assert_eq!(g.object_view(instant).zone, Zone::Graveyard);
        let n = start_nurse(&mut g, p);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(instant),
            },
        );
        choose_nurse(&mut g, n, x);
        settle(&mut g);
        assert_eq!(g.object_view(x).zone, Zone::Hand);
    }
}
#[test]
fn msh_recent_nurse_replaced_graveyard_entry_creates_no_eligible_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let x = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.move_object(x, Zone::Graveyard);
        assert_eq!(g.object_view(x).zone, Zone::Exile);
        assert!(!g.graveyard_entry_this_turn(g.object_ref(x)));
        let n = paid(&mut g, p, NURSE, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(n));
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn msh_recent_nurse_graveyard_ownership_ignores_prior_battlefield_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let x = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.change_controller(g.object_ref(x), p.other());
        paid(&mut g, p, "Murder", &[Target::Card(x)]);
        settle(&mut g);
        let n = start_nurse(&mut g, p);
        choose_nurse(&mut g, n, x);
        settle(&mut g);
        assert_eq!(g.object_view(x).zone, Zone::Hand);
    }
}
#[test]
fn msh_recent_rest_paid_player_and_optional_creature_groups_are_independent() {
    for p in [Player::P0, Player::P1] {
        for victim in [p, p.other()] {
            for recipient in [None, Some(p), Some(p.other())] {
                let mut g = setup(p);
                let c = recipient.map(|q| named(&mut g, q, "Bear Cub", Zone::Battlefield));
                let land = named(&mut g, victim, "Island", Zone::Library);
                start_rest(&mut g, p, victim, c);
                search(&mut g, victim);
                assert_eq!(g.state.players[victim.index()].life, 22);
                find(&mut g, Some(land));
                settle(&mut g);
                assert_eq!(g.object_view(land).zone, Zone::Battlefield);
                assert_eq!(g.object_view(land).controller, victim);
                assert!(g.object_view(land).tapped);
                if let Some(c) = c {
                    assert_eq!(g.counter_count(c, CounterKind::PlusOne), 1);
                }
            }
        }
    }
}
#[test]
fn msh_recent_rest_all_six_basic_land_recipes_and_normal_play_allowance() {
    for p in [Player::P0, Player::P1] {
        for name in ["Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes"] {
            let mut g = setup(p);
            let normal = named(&mut g, p, "Forest", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: normal });
            let x = named(&mut g, p, name, Zone::Library);
            start_rest(&mut g, p, p, None);
            search(&mut g, p);
            find(&mut g, Some(x));
            settle(&mut g);
            assert!(g.on_battlefield(x));
            assert!(g.object_view(x).tapped);
            let other = named(&mut g, p, "Forest", Zone::Hand);
            reject(&mut g, Action::PlayLand { card: other });
        }
    }
}
#[test]
fn msh_recent_rest_targeted_search_actor_private_choices_and_owned_commit() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        let land = named(&mut g, q, "Island", Zone::Library);
        let hidden = named(&mut g, q, "Murder", Zone::Library);
        start_rest(&mut g, p, q, None);
        search(&mut g, q);
        let mut alternate = g.clone();
        alternate.state.objects[hidden as usize].card = Card::from_name("Shock").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(alternate.observe(p)).unwrap()
        );
        assert_eq!(g.legal_actions(), alternate.legal_actions());
        g = orphan(g, Action::ChooseSearch { card: land });
        let rng = g.state.rng;
        g = orphan(g, Action::FinishSearch);
        settle(&mut g);
        assert_ne!(g.state.rng, rng);
        assert!(g.on_battlefield(land));
        assert_eq!(g.object_view(land).controller, q);
    }
}
#[test]
fn msh_recent_rest_search_rejects_nonbasic_foreign_duplicate_and_excess() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        let land = named(&mut g, q, "Island", Zone::Library);
        let nonbasic = named(&mut g, q, "A.I.M. Labs", Zone::Library);
        let foreign = named(&mut g, p, "Island", Zone::Library);
        let other = named(&mut g, q, "Forest", Zone::Library);
        start_rest(&mut g, p, q, None);
        search(&mut g, q);
        for card in [nonbasic, foreign] {
            reject(&mut g, Action::ChooseSearch { card });
        }
        checked(&mut g, Action::ChooseSearch { card: land });
        for card in [land, other] {
            reject(&mut g, Action::ChooseSearch { card });
        }
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
    }
}
#[test]
fn msh_recent_rest_fail_to_find_with_or_without_eligible_land_still_shuffles() {
    for p in [Player::P0, Player::P1] {
        for none in [false, true] {
            let mut g = setup(p);
            if none {
                let lib = g.state.players[p.index()].library.to_vec();
                for id in lib {
                    g.move_object(id, Zone::Exile);
                }
                named(&mut g, p, "Bear Cub", Zone::Library);
                named(&mut g, p, "Arc Reactor", Zone::Library);
            }
            let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            start_rest(&mut g, p, p, Some(c));
            search(&mut g, p);
            let rng = g.state.rng;
            find(&mut g, None);
            settle(&mut g);
            assert_ne!(rng, g.state.rng);
            assert_eq!(g.counter_count(c, CounterKind::PlusOne), 1);
        }
    }
}
#[test]
fn msh_recent_rest_life_prevention_and_actual_gain_replacement_do_not_skip_search() {
    for p in [Player::P0, Player::P1] {
        for prevent in [false, true] {
            let q = p.other();
            let mut g = setup(p);
            named(
                &mut g,
                q,
                if prevent {
                    "Giant Cindermaw"
                } else {
                    "Angel of Vitality"
                },
                Zone::Battlefield,
            );
            let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let x = named(&mut g, q, "Forest", Zone::Library);
            start_rest(&mut g, p, q, Some(c));
            search(&mut g, q);
            assert_eq!(
                g.state.players[q.index()].life,
                if prevent { 20 } else { 23 }
            );
            find(&mut g, Some(x));
            settle(&mut g);
            assert!(g.on_battlefield(x));
            assert_eq!(g.counter_count(c, CounterKind::PlusOne), 1);
        }
    }
}
fn truce(g: &mut Game, q: Player) {
    let id = paid(g, q, "Dawn's Truce", &[]);
    until(g, |g| g.object_view(id).zone == Zone::Graveyard);
}
#[test]
fn msh_recent_rest_paid_player_hexproof_response_leaves_other_creature_effect() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        start_rest(&mut g, p, q, Some(c));
        truce(&mut g, q);
        settle(&mut g);
        assert_eq!(g.state.players[q.index()].life, 20);
        assert_eq!(g.counter_count(c, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_recent_rest_illegal_creature_does_not_cancel_player_and_search() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let old = g.object_ref(c);
        let x = named(&mut g, p, "Forest", Zone::Library);
        start_rest(&mut g, p, p, Some(c));
        g.move_object(c, Zone::Exile);
        g.move_object(c, Zone::Battlefield);
        assert_ne!(old, g.object_ref(c));
        search(&mut g, p);
        find(&mut g, Some(x));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.counter_count(c, CounterKind::PlusOne), 0);
        assert!(g.on_battlefield(x));
    }
}
#[test]
fn msh_recent_rest_paid_truce_makes_all_targets_illegal_without_any_effect() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        let c = named(&mut g, q, "Bear Cub", Zone::Battlefield);
        let lib = g.state.players[q.index()].library.clone();
        start_rest(&mut g, p, q, Some(c));
        truce(&mut g, q);
        settle(&mut g);
        assert_eq!(g.state.players[q.index()].life, 20);
        assert_eq!(g.state.players[q.index()].library, lib);
        assert_eq!(g.counter_count(c, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_recent_rest_counter_replacement_and_real_life_event_wait_for_search() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let feast = named(&mut g, p, "Heroic Feast", Zone::Battlefield);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let x = named(&mut g, p, "Forest", Zone::Library);
        start_rest(&mut g, p, p, Some(c));
        search(&mut g, p);
        assert!(!packet(&g, feast));
        assert_eq!(g.counter_count(c, CounterKind::PlusOne), 0);
        find(&mut g, Some(x));
        until(&mut g, |g| packet(g, feast));
        assert_eq!(g.counter_count(c, CounterKind::PlusOne), 2);
        assert!(g.on_battlefield(x));
        assert!(g.object_view(x).tapped);
        assert_eq!(g.state.announcing.as_ref().unwrap().spec.max, 2);
        let a = g.state.announcing.as_ref().unwrap().item.id;
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(c),
            },
        );
        finish(&mut g, a);
        settle(&mut g);
        assert_eq!(g.counter_count(c, CounterKind::PlusOne), 4);
    }
}
#[test]
fn msh_recent_rest_late_counter_failure_rolls_back_search_commit_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[c as usize].counters.plus_one = u64::MAX;
        let x = named(&mut g, p, "Forest", Zone::Library);
        start_rest(&mut g, p, p, Some(c));
        search(&mut g, p);
        checked(&mut g, Action::ChooseSearch { card: x });
        let before = g.snapshot();
        let before_views = views(&g);
        let rev = g.revision();
        assert!(matches!(
            g.apply(p, Action::FinishSearch),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(views(&g), before_views);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.object_view(x).zone, Zone::Library);
        assert_eq!(g.state.players[p.index()].life, 22);
    }
}
