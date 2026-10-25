//! Baron pays real costs; synthetic producers use ordinary owned effect rows.
//! Kindred metadata is never cast or admitted; copied packets are engine fixtures.
use super::assurance::big_helpers::{checked, named, next_turn, settle, setup};
use super::*;
const BARON: &str = "Baron Strucker, HYDRA Overlord";
const VILLAIN: &str = "Ghost, Spectral Saboteur";
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn finish(g: &mut Game) {
    for _ in 0..15 {
        if g.state.announcing.is_none() && !matches!(g.state.pending, Pending::ManaLotSpend { .. })
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::ChooseSpellCost { .. }
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished payment {:?}", g.state.pending);
}
fn cast(g: &mut Game, id: CardId, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = [30; 6];
    cast(g, id, &[]);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..150 {
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
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn entry(g: &mut Game, p: Player) -> CardId {
    priority(g, p);
    let id = named(g, p, VILLAIN, Zone::Hand);
    g.state.players[p.index()].mana = [30; 6];
    cast(g, id, &[]);
    until(g, |g| g.on_battlefield(id));
    id
}
fn optional(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
}
fn accept(g: &mut Game) {
    optional(g);
    checked(g, Action::ChooseOptional { accept: true });
}
fn discard(g: &mut Game, p: Player, id: CardId) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    assert_eq!(g.actor(), Some(p));
    checked(g, Action::Discard { card: id });
    settle(g);
}
fn moved(g: &mut Game, ids: &[CardId], zone: Zone) {
    let events = g.move_objects_raw(ids, zone);
    g.finish_zone_events(&events, &[]);
}
fn packet(g: &Game, source: CardId) -> &SpellView {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .find(|s| s.card == source && s.event_object.is_some())
        .unwrap()
}
fn set_types(g: &mut Game, id: CardId, types: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    assert!(timestamp > 0);
    g.state
        .roles
        .get_or_insert_default()
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types,
            additional: false,
            timestamp,
        });
}
fn metadata_entrant(g: &mut Game, p: Player, kindred: bool, villain: bool) -> CardId {
    let id = named(g, p, "Hill Giant", Zone::Graveyard);
    if kindred {
        let card = Card::from_name("Boggart Mischief").unwrap();
        assert!(!card.is_supported());
        assert_eq!(card.kind_mask(), 160);
        g.state.objects[id as usize].card = card;
    }
    let events = g.move_objects_raw(&[id], Zone::Battlefield);
    // Synthetic earlier-layer subtype producer before public entry dispatch.
    if villain {
        set_types(g, id, &["Villain"]);
    }
    g.finish_zone_events(&events, &[]);
    id
}
fn blank(g: &mut Game, p: Player, id: CardId) {
    let aura = named(g, p, "Stop Cold", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(id));
}
fn visible(g: &Game, p: Player, id: CardId) -> bool {
    g.observe(p).objects.iter().any(|o| o.reference.card == id)
}
#[test]
fn msh_baron_paid_body_discount_and_generic_colored_floor() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = named(&mut g, p, BARON, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card: b });
        g.state.players[p.index()].mana[5] = 2;
        cast(&mut g, b, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let v = g.object_view(b);
        assert_eq!((v.power, v.toughness, v.colors), (2, 2, 4));
        assert!(v.card.is_legendary() && v.has_subtype("Human") && v.has_subtype("Villain"));
        assert_eq!(g.keywords(b), Keywords::NONE);
        let who = named(&mut g, p, VILLAIN, Zone::Hand);
        let card = g.object_view(who).card;
        assert!(card.has_subtype("Villain"));
        let (colored, generic) = card.cost();
        let cost = g.reduced_spell_cost(p, who, ManaCost { colored, generic });
        assert_eq!(cost.colored, colored);
        assert_eq!(cost.generic, generic - 1);
        g.state.players[p.index()].mana = colored;
        g.state.players[p.index()].mana[5] = generic - 1;
        cast(&mut g, who, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.object_view(who).card.fixed_mana_value(),
            card.fixed_mana_value()
        );
        let zero = named(&mut g, p, "Duress", Zone::Hand);
        assert_eq!(
            g.reduced_spell_cost(
                p,
                zero,
                ManaCost {
                    colored: [0, 0, 1, 0, 0, 0],
                    generic: 0
                }
            ),
            ManaCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 0
            }
        );
    }
}
#[test]
fn msh_baron_discount_current_controller_suppression_other_spells_and_funding() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, BARON);
        let who = named(&mut g, p, VILLAIN, Zone::Hand);
        let (colored, generic) = g.object_view(who).card.cost();
        let base = ManaCost { colored, generic };
        assert_eq!(g.reduced_spell_cost(p, who, base).generic, generic - 1);
        g.change_controller(g.object_ref(b), p.other());
        assert_eq!(g.reduced_spell_cost(p, who, base), base);
        let theirs = named(&mut g, p.other(), VILLAIN, Zone::Hand);
        assert_eq!(
            g.reduced_spell_cost(p.other(), theirs, base).generic,
            generic - 1
        );
        blank(&mut g, p, b);
        assert_eq!(g.reduced_spell_cost(p.other(), theirs, base), base);
        let non = named(&mut g, p.other(), "Hill Giant", Zone::Hand);
        assert_eq!(g.spell_reduction(p.other(), non), 0);
        moved(&mut g, &[b], Zone::Exile);
        assert_eq!(g.reduced_spell_cost(p.other(), theirs, base), base);
        // The real Hideout produces Villain-only mana; it can fund Baron, never a generic non-Villain spell.
        let mut g = setup(p);
        let h = named(&mut g, p, "Villainous Hideout", Zone::Battlefield);
        let b = named(&mut g, p, BARON, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(
            &mut g,
            Action::ActivateMana {
                card: h,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 1, 0, 0, 0],
            },
        );
        let non = named(&mut g, p, "Hill Giant", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: non });
        cast(&mut g, b, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(b));
        assert!(g.mana_lots(p).is_empty());
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_baron_entry_filter_kindred_current_types_source_exclusion_and_untargeted() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, BARON);
        assert!(g.state.pending_triggers.is_empty());
        metadata_entrant(&mut g, p, false, false);
        metadata_entrant(&mut g, p.other(), true, true);
        assert!(!g.state.pending_triggers.iter().any(|s| s.card == b));
        let who = metadata_entrant(&mut g, p, true, true);
        let original = g.object_ref(who);
        assert!(!g.is_creature(who));
        assert!(g.has_subtype(who, "Villain"));
        let s = packet(&g, b);
        assert!(s.targets.is_empty());
        assert_eq!(s.entering_object, Some(original));
        assert_eq!(
            s.event_object.as_ref().unwrap().facts,
            EventObjectFacts::Controller(p)
        );
        let d = named(&mut g, p, "Hill Giant", Zone::Hand);
        accept(&mut g);
        discard(&mut g, p, d);
        assert_eq!(g.counter_count(who, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
        assert!(g.put_counter_on_source_this_turn(original, p));
    }
}
#[test]
fn msh_baron_decline_accept_queued_limiter_turn_reset_and_exact_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, BARON);
        let first = entry(&mut g, p);
        optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        assert_eq!(g.counter_count(first, CounterKind::PlusOne), 0);
        let a = metadata_entrant(&mut g, p, false, true);
        let z = metadata_entrant(&mut g, p, false, true);
        let d = named(&mut g, p, "Hill Giant", Zone::Hand);
        let hand = g.hand_count(p);
        accept(&mut g);
        discard(&mut g, p, d);
        assert_eq!(g.hand_count(p), hand);
        assert_eq!(
            g.counter_count(a, CounterKind::PlusOne) + g.counter_count(z, CounterKind::PlusOne),
            1
        );
        let later = metadata_entrant(&mut g, p, false, true);
        settle(&mut g);
        assert_eq!(g.counter_count(later, CounterKind::PlusOne), 0);
        assert!(!g.state.pending_triggers.iter().any(|s| s.card == b));
        next_turn(&mut g);
        let later = metadata_entrant(&mut g, p, false, true);
        let d = named(&mut g, p, "Hill Giant", Zone::Hand);
        accept(&mut g);
        discard(&mut g, p, d);
        assert_eq!(g.counter_count(later, CounterKind::PlusOne), 1);
        moved(&mut g, &[b], Zone::Graveyard);
        moved(&mut g, &[b], Zone::Battlefield);
        let later = metadata_entrant(&mut g, p, false, true);
        let d = named(&mut g, p, "Hill Giant", Zone::Hand);
        accept(&mut g);
        discard(&mut g, p, d);
        assert_eq!(g.counter_count(later, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_baron_live_control_change_uses_conniver_player_and_private_discard() {
    for p in [Player::P0, Player::P1] {
        for kindred in [false, true] {
            let mut g = setup(p);
            let b = paid(&mut g, p, BARON);
            let who = if kindred {
                metadata_entrant(&mut g, p, true, true)
            } else {
                entry(&mut g, p)
            };
            let original = g.object_ref(who);
            g.change_controller(original, p.other());
            let d = named(&mut g, p.other(), "Hill Giant", Zone::Hand);
            let foreign = named(&mut g, p, "Hill Giant", Zone::Hand);
            let old = g.hand_count(p);
            let new = g.hand_count(p.other());
            optional(&mut g);
            assert_eq!(g.actor(), Some(p));
            checked(&mut g, Action::ChooseOptional { accept: true });
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(g.actor(), Some(p.other()));
            assert_eq!(g.hand_count(p), old);
            assert_eq!(g.hand_count(p.other()), new + 1);
            assert!(!visible(&g, p, d));
            assert!(!visible(&g, p.other(), foreign));
            reject(&mut g, Action::Discard { card: foreign });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(who),
                },
            );
            checked(&mut g, Action::Discard { card: d });
            settle(&mut g);
            assert_eq!(g.counter_count(who, CounterKind::PlusOne), 1);
            assert!(g.put_counter_on_source_this_turn(original, p.other()));
            assert!(!g.put_counter_on_source_this_turn(original, p));
            assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
        }
    }
}
#[test]
fn msh_baron_latest_departure_all_zones_owner_and_reentry_never_rebind() {
    for p in [Player::P0, Player::P1] {
        for zone in [Zone::Hand, Zone::Graveyard, Zone::Exile] {
            for reenter in [false, true] {
                let mut g = setup(p);
                let b = paid(&mut g, p, BARON);
                let who = entry(&mut g, p);
                let original = g.object_ref(who);
                g.change_controller(original, p.other());
                moved(&mut g, &[who], zone);
                assert_eq!(
                    packet(&g, b).event_object.as_ref().unwrap().facts,
                    EventObjectFacts::Controller(p.other())
                );
                if reenter {
                    moved(&mut g, &[who], Zone::Battlefield);
                    assert_ne!(g.object_ref(who), original);
                    assert_eq!(g.object_view(who).controller, p);
                }
                let d = named(&mut g, p.other(), "Hill Giant", Zone::Hand);
                accept(&mut g);
                discard(&mut g, p.other(), d);
                assert_eq!(g.counter_count(who, CounterKind::PlusOne), 0);
                assert!(!g.put_counter_on_source_this_turn(original, p.other()));
            }
        }
    }
}
#[test]
fn msh_baron_recipient_subtype_and_creature_loss_do_not_cancel_connive() {
    for p in [Player::P0, Player::P1] {
        for noncreature in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, BARON);
            let who = entry(&mut g, p);
            if noncreature {
                let a = named(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
                g.state.objects[a as usize].attached_to = Some(g.object_ref(who));
                assert!(!g.is_creature(who));
                assert!(!g.has_subtype(who, "Villain"));
            } else {
                set_types(&mut g, who, &["Hero"]);
                assert!(!g.has_subtype(who, "Villain"));
            }
            let d = named(&mut g, p, "Hill Giant", Zone::Hand);
            accept(&mut g);
            discard(&mut g, p, d);
            assert_eq!(g.counter_count(who, CounterKind::PlusOne), 1);
        }
    }
}
#[test]
fn msh_baron_counter_replacement_and_opponent_discard_deferred_placer() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            for deferred in [false, true] {
                let mut g = setup(p);
                paid(&mut g, p, BARON);
                let who = entry(&mut g, p);
                let original = g.object_ref(who);
                g.change_controller(original, p.other());
                if doubled {
                    named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
                }
                let d = named(
                    &mut g,
                    p.other(),
                    if deferred {
                        "Wilt-Leaf Liege"
                    } else {
                        "Hill Giant"
                    },
                    Zone::Hand,
                );
                accept(&mut g);
                discard(&mut g, p.other(), d);
                assert_eq!(
                    g.object_view(d).zone,
                    if deferred {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(
                    g.counter_count(who, CounterKind::PlusOne),
                    if doubled { 2 } else { 1 }
                );
                assert!(g.put_counter_on_source_this_turn(original, p.other()));
                assert!(!g.put_counter_on_source_this_turn(original, p));
            }
        }
    }
}
#[test]
fn msh_baron_source_theft_departure_and_suppression_preserve_queued_choice() {
    for p in [Player::P0, Player::P1] {
        for action in 0..3 {
            let mut g = setup(p);
            let b = paid(&mut g, p, BARON);
            let who = entry(&mut g, p);
            match action {
                0 => g.change_controller(g.object_ref(b), p.other()),
                1 => moved(&mut g, &[b], Zone::Exile),
                _ => blank(&mut g, p, b),
            }
            let d = named(&mut g, p, "Hill Giant", Zone::Hand);
            optional(&mut g);
            assert_eq!(g.actor(), Some(p));
            checked(&mut g, Action::ChooseOptional { accept: true });
            discard(&mut g, p, d);
            assert_eq!(g.counter_count(who, CounterKind::PlusOne), 1);
        }
        let mut g = setup(p);
        let b = paid(&mut g, p, BARON);
        blank(&mut g, p, b);
        let who = entry(&mut g, p);
        settle(&mut g);
        assert_eq!(g.counter_count(who, CounterKind::PlusOne), 0);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn msh_baron_copied_packet_shared_once_and_copied_source_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, BARON);
        let who = entry(&mut g, p);
        let mut copy = packet(&g, b).clone();
        copy.id = g.next_stack_id();
        copy.is_copy = true;
        copy.controller = p.other();
        g.state.pending_triggers.push(copy);
        // All other hand cards are lands, so auto-discard still proves exactly one draw cycle.
        let old = g.hand_count(p);
        settle(&mut g);
        assert_eq!(g.hand_count(p), old);
        assert_eq!(
            g.state
                .triggered_this_turn
                .iter()
                .filter(|u| u.source == g.object_ref(b))
                .count(),
            1
        );
        assert_eq!(g.counter_count(who, CounterKind::PlusOne), 0);
        let mut g = setup(p);
        let b = paid(&mut g, p, BARON);
        let def = g.object_view(b).card;
        let copied = g.create_copy_groups_inner(
            p.other(),
            &[TokenGroupView {
                token: def,
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(def))],
            None,
        )[0]
        .card;
        g.advance_entries();
        settle(&mut g);
        assert_eq!(g.object_view(copied).controller, p.other());
        let who = metadata_entrant(&mut g, p.other(), false, true);
        let d = named(&mut g, p.other(), "Hill Giant", Zone::Hand);
        accept(&mut g);
        discard(&mut g, p.other(), d);
        assert_eq!(g.counter_count(who, CounterKind::PlusOne), 1);
        assert!(
            g.state
                .triggered_this_turn
                .iter()
                .any(|u| u.source == g.object_ref(copied))
        );
        assert!(
            !g.state
                .triggered_this_turn
                .iter()
                .any(|u| u.source == g.object_ref(b))
        );
    }
}
#[test]
fn msh_baron_land_discard_no_counter_and_no_unrelated_payload() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, BARON);
        let who = entry(&mut g, p);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        accept(&mut g);
        discard(&mut g, p, land);
        assert_eq!(g.counter_count(who, CounterKind::PlusOne), 0);
        // A paid, unrelated self-conniver must retain its old absent event payload.
        let mut g = setup(p);
        let recruit = named(&mut g, p, "Red Room Recruit", Zone::Hand);
        cast(&mut g, recruit, &[]);
        until(&mut g, |g| g.on_battlefield(recruit));
        assert!(
            g.state
                .pending_triggers
                .iter()
                .filter(|s| s.card == recruit)
                .all(|s| s.event_object.is_none())
        );
        settle(&mut g);
        assert!(!g.observe(p).stack.iter().any(|s| s.card == b));
    }
}
#[test]
fn msh_baron_paid_bounce_updates_recipient_last_controller_and_source_stays_separate() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, BARON);
        let who = entry(&mut g, p);
        let original = g.object_ref(who);
        // Put the real entry trigger on the stack, then cast and pay an instant.
        until(&mut g, |g| g.state.stack.iter().any(|s| s.card == b));
        g.change_controller(original, p.other());
        priority(&mut g, p);
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, bounce, &[Target::Card(who)]);
        until(&mut g, |g| g.object_view(who).zone == Zone::Hand);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(who).owner, p);
        assert_eq!(
            packet(&g, b).event_object.as_ref().unwrap().facts,
            EventObjectFacts::Controller(p.other())
        );
        let d = named(&mut g, p.other(), "Hill Giant", Zone::Hand);
        accept(&mut g);
        discard(&mut g, p.other(), d);
        assert_eq!(g.counter_count(who, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_baron_context_orphan_privacy_restore_and_latest_control_is_branch_local() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, BARON);
        let who = entry(&mut g, p);
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
        orphan.change_controller(orphan.object_ref(who), p.other());
        moved(&mut orphan, &[who], Zone::Exile);
        assert_eq!(g.snapshot(), before);
        assert_eq!(
            packet(&g, b).event_object.as_ref().unwrap().facts,
            EventObjectFacts::Controller(p)
        );
        assert_eq!(
            packet(&orphan, b).event_object.as_ref().unwrap().facts,
            EventObjectFacts::Controller(p.other())
        );
        let d = named(&mut orphan, p.other(), "Hill Giant", Zone::Hand);
        optional(&mut orphan);
        let boundary = orphan.snapshot();
        checked(&mut orphan, Action::ChooseOptional { accept: true });
        discard(&mut orphan, p.other(), d);
        let after = orphan.snapshot();
        orphan.restore(&boundary).unwrap();
        checked(&mut orphan, Action::ChooseOptional { accept: true });
        discard(&mut orphan, p.other(), d);
        assert_eq!(orphan.snapshot(), after);
        assert_eq!(orphan.observe(p).stack, orphan.observe(p.other()).stack);
        assert!(!visible(&orphan, p.other(), secret)); // Exile recipient never exposes unrelated private hands.
    }
}
#[test]
fn msh_baron_token_ceases_same_slot_new_incarnation_does_not_receive_old_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, BARON);
        let definition = Card::from_name(VILLAIN).unwrap();
        let who = g.create_copy_groups_inner(
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
        let original = g.object_ref(who);
        g.change_controller(original, p.other());
        moved(&mut g, &[who], Zone::Exile);
        g.state_based_actions();
        assert_ne!(g.object_ref(who), original);
        let replacement = g.create_copy_groups_inner(
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
        assert_ne!(g.object_ref(replacement), original);
        assert_eq!(
            packet(&g, b).event_object.as_ref().unwrap().facts,
            EventObjectFacts::Controller(p.other())
        );
        let d = named(&mut g, p.other(), "Hill Giant", Zone::Hand);
        accept(&mut g);
        discard(&mut g, p.other(), d);
        assert_eq!(g.counter_count(replacement, CounterKind::PlusOne), 0);
    }
}
