//! Shared core regressions, retained for the three whole-card consumers.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named as supported_named, settle, setup};
use super::*;
const DOC: &str = "Doc Samson, Super Psychiatrist";
const HYDE: &str = "Mister Hyde, Monster Within";
const ANT: &str = "The Astonishing Ant-Man";
// Whole-card admissions now use the ordinary support-checked fixture path.
pub(super) fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    supported_named(g, p, name, zone)
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let b = g.state_bytes();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
    assert_eq!(g.state_bytes(), b);
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut o = g.observe(p);
        o.revision = 0;
        serde_json::to_value(o).unwrap()
    })
}
fn orphan(g: Game, a: Action) -> Game {
    let who = g.actor().unwrap();
    let rev = g.revision();
    let i = g.legal_actions().iter().position(|x| x == &a).unwrap();
    let before = g.snapshot();
    let bv = views(&g);
    let (mut raw, mut index) = (g.clone(), g.clone());
    drop(g);
    raw.apply(who, a.clone()).unwrap();
    index.apply_index(who, rev, i).unwrap();
    let after = raw.snapshot();
    assert_eq!(after, index.snapshot());
    assert_eq!(views(&raw), views(&index));
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), bv);
    assert_eq!(raw.apply_index(who, rev, i), Err(Error::StaleRevision));
    raw.apply(who, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    index
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, generic) = g.object_view(id).card.cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, &[]);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
                )
            })
            .unwrap_or_else(|| panic!("unhandled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending)
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, a);
}
fn activate(id: CardId) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn ready(g: &mut Game, id: CardId) {
    g.state.objects[id as usize].summoning_sick = false;
}
fn mana(id: CardId, color: usize, n: u32) -> Action {
    let mut production = [0; 6];
    production[color] = n;
    Action::ActivateMana {
        card: id,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}
fn upkeep(g: &mut Game, p: Player, mode: u16) {
    g.state.active = p;
    g.state.priority = p;
    g.begin_step(Step::Upkeep);
    until(g, |g| matches!(g.state.pending, Pending::Modes { .. }));
    reject(g, Action::ChooseMode { mode: 2 });
    checked(g, Action::ChooseMode { mode });
}
fn counter_prompt(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    assert_eq!(
        g.state
            .resolving
            .as_ref()
            .unwrap()
            .choice
            .as_ref()
            .unwrap()
            .kind,
        EffectChoiceKind::RemoveCreatureCounters(1)
    );
}
fn insects(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::from_name("@token/3512fba4bfa3f3fcf0f1eb9a").unwrap())
            .then_some(i as CardId)
        })
        .collect()
}
fn ant_announce(g: &mut Game, p: Player, id: CardId, n: u64) {
    g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
    checked(g, activate(id));
    assert!(g.x_choice().is_none());
    assert_eq!(
        g.source_counter_choice().unwrap().kind,
        CounterKind::PlusOne
    );
    checked(g, Action::ChooseSourceCounterAmount { amount: n });
}
#[test]
fn msh_counter_core_real_consumers_are_supported_with_original_paid_stats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for (name, stats) in [(DOC, (3, 6)), (HYDE, (2, 2)), (ANT, (1, 1))] {
            assert!(Card::from_name(name).unwrap().is_supported());
            let id = paid(&mut g, p, name);
            assert_eq!((g.stats(id).power, g.stats(id).toughness), stats);
        }
    }
}
#[test]
fn msh_counter_core_doc_placer_and_recipient_controller_permanent_matrix() {
    for p in [Player::P0, Player::P1] {
        for owner_other in [false, true] {
            for recipient_other in [false, true] {
                for placer_other in [false, true] {
                    let mut g = setup(p);
                    paid(&mut g, p, DOC);
                    let r = named(
                        &mut g,
                        if owner_other { p.other() } else { p },
                        "Gilded Lotus",
                        Zone::Battlefield,
                    );
                    let controller = if recipient_other { p.other() } else { p };
                    g.change_controller(g.object_ref(r), controller);
                    let placer = if placer_other { p.other() } else { p };
                    g.effect_counter_by(
                        r,
                        CounterKind::Page,
                        1,
                        placer,
                        replacements::ResumeMode::Pending,
                    );
                    g.advance_replacements(false);
                    settle(&mut g);
                    assert_eq!(
                        g.counter_count(r, CounterKind::Page),
                        if !recipient_other && !placer_other {
                            2
                        } else {
                            1
                        }
                    );
                }
            }
        }
    }
}
#[test]
fn msh_counter_core_doc_positive_kinds_and_zero_across_permanent_kinds() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Forest", "Gilded Lotus", "Claim the Kingdom"] {
            let mut g = setup(p);
            paid(&mut g, p, DOC);
            let r = named(&mut g, p, name, Zone::Battlefield);
            for kind in [
                CounterKind::Stun,
                CounterKind::Page,
                CounterKind::Indestructible,
                CounterKind::Finality,
            ] {
                g.effect_counter_by(r, kind, 2, p, replacements::ResumeMode::Pending);
                g.advance_replacements(false);
                settle(&mut g);
                assert_eq!(g.counter_count(r, kind), 3);
            }
            let before = g.snapshot();
            g.effect_counter_by(
                r,
                CounterKind::PlusOne,
                0,
                p,
                replacements::ResumeMode::Pending,
            );
            g.advance_replacements(false);
            assert_eq!(g.snapshot(), before);
        }
    }
}
#[test]
fn msh_counter_core_doc_mixed_batch_replacements_choose_order_and_commit_together() {
    for p in [Player::P0, Player::P1] {
        for doc_first in [false, true] {
            let mut g = setup(p);
            let doc = paid(&mut g, p, DOC);
            let season = named(&mut g, p, "Doubling Season", Zone::Battlefield);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.effect_counters(
                vec![
                    (
                        g.object_ref(a),
                        Counters::of(CounterKind::PlusOne, 1).with(CounterKind::Stun, 2),
                    ),
                    (g.object_ref(b), Counters::of(CounterKind::PlusOne, 1)),
                ],
                replacements::ResumeMode::Pending,
            );
            g.advance_replacements(false);
            assert_eq!(g.actor(), Some(p));
            assert_eq!(g.counter_count(a, CounterKind::PlusOne), 0);
            assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
            let source = g.object_ref(if doc_first { doc } else { season });
            let ability = if doc_first { 0 } else { 1 };
            g = orphan(g, Action::ChooseReplacement { source, ability });
            if matches!(g.state.pending, Pending::Replacement { .. }) {
                assert_eq!(g.counter_count(a, CounterKind::PlusOne), 0);
                assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
            }
            while matches!(g.state.pending, Pending::Replacement { .. }) {
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::ChooseReplacement { .. }))
                    .unwrap();
                checked(&mut g, a);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(a, CounterKind::PlusOne),
                if doc_first { 4 } else { 3 }
            );
            assert_eq!(
                g.counter_count(a, CounterKind::Stun),
                if doc_first { 6 } else { 5 }
            );
            assert!(g.counter_count(b, CounterKind::PlusOne) >= 3);
        }
    }
}
#[test]
fn msh_counter_core_doc_general_incoming_static_never_applies_to_own_or_other_simultaneous_entry() {
    for p in [Player::P0, Player::P1] {
        for live in [false, true] {
            let mut g = setup(p);
            if live {
                paid(&mut g, p, DOC);
            }
            let a = named(&mut g, p, DOC, Zone::Hand);
            // Both incoming sources share a controller. Inspect the simultaneous
            // entry commit before the subsequent legend-rule SBA.
            let b = named(&mut g, p, DOC, Zone::Hand);
            let mut ma = g.entry_move(a, Zone::Battlefield, false);
            ma.effect_counter = Some((CounterKind::PlusOne, 1));
            let mut mb = g.entry_move(b, Zone::Battlefield, false);
            mb.effect_counter = Some((CounterKind::PlusOne, 1));
            g.stage_entries(vec![ma, mb], None);
            g.advance_entries();
            assert_eq!(
                g.counter_count(a, CounterKind::PlusOne),
                if live { 2 } else { 1 }
            );
            assert_eq!(
                g.counter_count(b, CounterKind::PlusOne),
                if live { 2 } else { 1 }
            );
        }
    }
}
#[test]
fn msh_counter_core_doc_suppression_departure_and_current_provider_controller() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let d = paid(&mut g, p, DOC);
            let r = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            match change {
                0 => {
                    g.move_object(d, Zone::Exile);
                }
                1 => g.change_controller(g.object_ref(d), p.other()),
                _ => {
                    let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
                    g.state.players[p.index()].mana = [30; 6];
                    cast(&mut g, aura, None, &[Target::Card(d)]);
                    settle(&mut g);
                }
            }
            g.effect_counter_by(
                r,
                CounterKind::PlusOne,
                1,
                p,
                replacements::ResumeMode::Pending,
            );
            g.advance_replacements(false);
            settle(&mut g);
            assert_eq!(g.counter_count(r, CounterKind::PlusOne), 1);
        }
    }
}
#[test]
fn msh_counter_core_doc_five_one_color_outputs_and_illegal_split_colorless_amounts() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = setup(p);
            let d = paid(&mut g, p, DOC);
            assert_eq!(g.power(d), 3);
            reject(&mut g, mana(d, color, 3));
            ready(&mut g, d);
            reject(&mut g, mana(d, 5, 3));
            reject(&mut g, mana(d, color, 2));
            let mut split = [0; 6];
            split[color] = 2;
            split[(color + 1) % 5] = 1;
            reject(
                &mut g,
                Action::ActivateMana {
                    card: d,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: split,
                },
            );
            g = orphan(g, mana(d, color, 3));
            assert_eq!(g.state.players[p.index()].mana[color], 3);
            assert_eq!(g.state.players[p.index()].mana.iter().sum::<u32>(), 3);
            assert!(g.object_view(d).tapped);
            assert!(g.state.stack.is_empty());
            reject(&mut g, mana(d, color, 3));
        }
    }
}
#[test]
fn msh_counter_core_doc_actual_funding_is_one_color_and_uses_effective_power() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = paid(&mut g, p, DOC);
        ready(&mut g, d);
        let mixed = named(&mut g, p, ANT, Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: mixed });
        let div = named(&mut g, p, "Divination", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: div });
        checked(&mut g, mana(d, 1, 3));
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        let mut h = setup(p);
        let d = paid(&mut h, p, DOC);
        ready(&mut h, d);
        h.effect_counter_by(
            d,
            CounterKind::PlusOne,
            1,
            p,
            replacements::ResumeMode::Pending,
        );
        h.advance_replacements(false);
        settle(&mut h);
        assert_eq!(h.stats(d).power, 5);
        checked(&mut h, mana(d, 4, 5));
        assert_eq!(h.state.players[p.index()].mana[4], 5);
    }
}
#[test]
fn msh_counter_core_doc_zero_negative_power_is_one_zero_action() {
    for p in [Player::P0, Player::P1] {
        for power in [-2, 0] {
            let mut g = setup(p);
            let d = paid(&mut g, p, DOC);
            ready(&mut g, d);
            g.state.objects[d as usize].growth.power = power - 3;
            let acts: Vec<_> = g
                .legal_actions()
                .into_iter()
                .filter(|a| matches!(a,Action::ActivateMana{card,..} if *card==d))
                .collect();
            assert_eq!(acts.len(), 1);
            checked(&mut g, acts[0].clone());
            assert!(g.object_view(d).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn msh_counter_core_hyde_resolution_selection_is_mandatory_eligible_and_untargeted() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, HYDE);
        let r = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let empty = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.set_counters(r, Counters::of(CounterKind::Flying, 2));
        g.state.objects[r as usize].gained_keywords = Keywords::HEXPROOF;
        g.set_counters(enemy, Counters::of(CounterKind::Flying, 2));
        upkeep(&mut g, p, 1);
        counter_prompt(&mut g);
        assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
        let hand = g.hand_count(p);
        let v = g.observe(p).effect_choice.unwrap();
        assert!(!v.optional);
        assert_eq!(v.candidates, vec![g.object_ref(r)]);
        reject(&mut g, Action::DeclineEffectPayment);
        for id in [empty, enemy] {
            let object = g.object_ref(id);
            reject(
                &mut g,
                Action::ChooseCreatureCounterCost {
                    object,
                    counter: CounterKind::Flying,
                },
            );
        }
        let object = g.object_ref(r);
        reject(
            &mut g,
            Action::ChooseCreatureCounterCost {
                object,
                counter: CounterKind::Page,
            },
        );
        g = orphan(
            g,
            Action::ChooseCreatureCounterCost {
                object,
                counter: CounterKind::Flying,
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(r, CounterKind::Flying), 1);
        assert_eq!(g.hand_count(p), hand + 1);
        assert!(g.keywords(r).contains(Keywords::FLYING));
    }
}
#[test]
fn msh_counter_core_hyde_all_counter_families_include_self() {
    for p in [Player::P0, Player::P1] {
        for kind in CounterKind::ALL {
            let mut g = setup(p);
            let h = paid(&mut g, p, HYDE);
            g.set_counters(h, Counters::of(kind, 1));
            upkeep(&mut g, p, 1);
            counter_prompt(&mut g);
            let object = g.object_ref(h);
            checked(
                &mut g,
                Action::ChooseCreatureCounterCost {
                    object,
                    counter: kind,
                },
            );
            settle(&mut g);
            assert_eq!(g.counter_count(h, kind), 0);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        }
    }
}
#[test]
fn msh_counter_core_hyde_no_eligible_counter_means_no_prompt_or_draw_but_mode_is_legal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, HYDE);
        let a = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        g.set_counters(a, Counters::of(CounterKind::Page, 1));
        upkeep(&mut g, p, 1);
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::Page), 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn msh_counter_core_hyde_recomputes_after_mode_and_retains_old_controller_after_source_departure() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let h = paid(&mut g, p, HYDE);
            let r = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            upkeep(&mut g, p, 1);
            match change {
                0 => g.change_controller(g.object_ref(h), p.other()),
                1 => {
                    g.move_object(h, Zone::Exile);
                }
                _ => {
                    g.move_object(h, Zone::Exile);
                    g.move_object(h, Zone::Battlefield);
                }
            }
            g.set_counters(r, Counters::of(CounterKind::Page, 1));
            counter_prompt(&mut g);
            assert_eq!(g.actor(), Some(p));
            let object = g.object_ref(r);
            checked(
                &mut g,
                Action::ChooseCreatureCounterCost {
                    object,
                    counter: CounterKind::Page,
                },
            );
            settle(&mut g);
            assert_eq!(g.counter_count(r, CounterKind::Page), 0);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
        }
    }
}
#[test]
fn msh_counter_core_hyde_stale_reference_and_both_observation_privacy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, HYDE);
        let r = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.set_counters(r, Counters::of(CounterKind::Stun, 1));
        let secret = named(&mut g, p, "Shock", Zone::Hand);
        upkeep(&mut g, p, 1);
        counter_prompt(&mut g);
        let mut b = g.clone();
        b.state.objects[secret as usize].card = Card::from_name("Forest").unwrap();
        assert_eq!(views(&g)[p.other().index()], views(&b)[p.other().index()]);
        assert_eq!(g.legal_actions(), b.legal_actions());
        let old = g.object_ref(r);
        g.move_object(r, Zone::Exile);
        g.move_object(r, Zone::Battlefield);
        g.set_counters(r, Counters::of(CounterKind::Stun, 1));
        reject(
            &mut g,
            Action::ChooseCreatureCounterCost {
                object: old,
                counter: CounterKind::Stun,
            },
        );
        let object = g.object_ref(r);
        checked(
            &mut g,
            Action::ChooseCreatureCounterCost {
                object,
                counter: CounterKind::Stun,
            },
        );
        settle(&mut g);
    }
}
#[test]
fn msh_counter_core_hyde_draw_pause_is_owned_and_removal_precedes_draw_then_sba() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        paid(&mut g, p, HYDE);
        let r = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[r as usize].growth = Stats {
            power: -2,
            toughness: -2,
        };
        g.set_counters(r, Counters::of(CounterKind::PlusOne, 1));
        upkeep(&mut g, p, 1);
        counter_prompt(&mut g);
        let object = g.object_ref(r);
        checked(
            &mut g,
            Action::ChooseCreatureCounterCost {
                object,
                counter: CounterKind::PlusOne,
            },
        );
        assert!(matches!(g.state.pending, Pending::MiracleReveal { .. }));
        assert_eq!(g.counter_count(r, CounterKind::PlusOne), 0);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert!(g.observe(p.other()).miracle.is_none());
        g = orphan(g, Action::DeclineMiracle);
        settle(&mut g);
        assert_eq!(g.object_view(r).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_counter_core_ant_actual_cost_zero_through_five_freezes_quantity_and_exact_tokens() {
    for p in [Player::P0, Player::P1] {
        for n in 0..=5 {
            let mut g = setup(p);
            let a = paid(&mut g, p, ANT);
            ready(&mut g, a);
            g.set_counters(a, Counters::of(CounterKind::PlusOne, 5));
            ant_announce(&mut g, p, a, n);
            assert_eq!(g.counter_count(a, CounterKind::PlusOne), 5);
            assert!(
                !g.state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .item
                    .source_counter_payment
                    .unwrap()
                    .paid
            );
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            pay(&mut g);
            assert_eq!(g.counter_count(a, CounterKind::PlusOne), 5 - n);
            assert!(g.object_view(a).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let item = g.state.stack.last().unwrap();
            assert_eq!(item.x, None);
            assert_eq!(
                item.source_counter_payment,
                Some(SourceCounterPaymentView {
                    kind: CounterKind::PlusOne,
                    amount: n,
                    paid: true
                })
            );
            settle(&mut g);
            assert_eq!(insects(&g, p).len(), n as usize);
            for t in insects(&g, p) {
                let v = g.object_view(t);
                assert_eq!((v.power, v.toughness, v.colors), (1, 1, 16));
                assert!(v.has_subtype("Insect"));
                assert_eq!(v.keywords, Keywords::NONE);
            }
        }
    }
}
#[test]
fn msh_counter_core_ant_u64_domain_interior_not_enumerated_is_owned_and_no_token_probe() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, ANT);
        ready(&mut g, a);
        g.set_counters(a, Counters::of(CounterKind::PlusOne, u64::MAX));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        checked(&mut g, activate(a));
        assert_eq!(g.source_counter_choice().unwrap().maximum, u64::MAX);
        assert!(g.legal_actions().len() <= 4);
        assert!(g.observe(p.other()).source_counter_choice.is_none());
        assert!(g.observe(p).source_counter_choice.is_some());
        let amount = u64::MAX - 2;
        let actor = g.actor().unwrap();
        let before = g.snapshot();
        let mut raw = {
            let parent = g.clone();
            parent.clone()
        };
        drop(g);
        raw.apply(actor, Action::ChooseSourceCounterAmount { amount })
            .unwrap();
        assert_eq!(
            raw.state
                .announcing
                .as_ref()
                .unwrap()
                .counter_payment
                .get(CounterKind::PlusOne),
            amount
        );
        raw.restore(&before).unwrap();
        raw.apply(actor, Action::ChooseSourceCounterAmount { amount: 0 })
            .unwrap();
        pay(&mut raw);
        settle(&mut raw);
        assert!(insects(&raw, p).is_empty());
    }
}
#[test]
fn msh_counter_core_ant_invalid_amount_wrong_kind_source_and_funding_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, ANT);
        ready(&mut g, a);
        g.set_counters(a, Counters::of(CounterKind::Page, 3));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        reject(&mut g, activate(a));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        reject(&mut g, activate(a));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        checked(&mut g, activate(a));
        assert_eq!(g.source_counter_choice().unwrap().maximum, 0);
        reject(&mut g, Action::ChooseSourceCounterAmount { amount: 1 });
        reject(&mut g, Action::ChooseX { value: 0 });
        g = orphan(g, Action::ChooseSourceCounterAmount { amount: 0 });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::Page), 3);
        reject(&mut g, activate(a));
    }
}
#[test]
fn msh_counter_core_ant_paid_quantity_survives_changed_counters_departure_blink_and_control() {
    for p in [Player::P0, Player::P1] {
        for change in 0..4 {
            let mut g = setup(p);
            let a = paid(&mut g, p, ANT);
            ready(&mut g, a);
            g.set_counters(a, Counters::of(CounterKind::PlusOne, 4));
            ant_announce(&mut g, p, a, 2);
            pay(&mut g);
            match change {
                0 => g.set_counters(a, Counters::of(CounterKind::PlusOne, 9)),
                1 => {
                    g.move_object(a, Zone::Graveyard);
                }
                2 => {
                    g.move_object(a, Zone::Exile);
                    g.move_object(a, Zone::Battlefield);
                }
                _ => g.change_controller(g.object_ref(a), p.other()),
            };
            settle(&mut g);
            assert_eq!(insects(&g, p).len(), 2);
            assert!(insects(&g, p.other()).is_empty());
            if change == 2 {
                assert_eq!(g.counter_count(a, CounterKind::PlusOne), 0);
            }
        }
    }
}
#[test]
fn msh_counter_core_ant_token_replacement_and_mana_before_committed_counter_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, ANT);
        ready(&mut g, a);
        g.set_counters(a, Counters::of(CounterKind::PlusOne, 2));
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let lotus = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, activate(a));
        checked(&mut g, Action::ChooseSourceCounterAmount { amount: 2 });
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
        checked(
            &mut g,
            Action::ActivateMana {
                card: lotus,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 3, 0],
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(insects(&g, p).len(), 4);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 0);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn msh_counter_core_doc_overflow_rolls_back_complete_resolving_action() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, DOC);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.set_counters(bear, Counters::of(CounterKind::PlusOne, u64::MAX));
        let spell = named(&mut g, p, "Snakeskin Veil", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, spell, None, &[Target::Card(bear)]);
        let mut failed = false;
        for _ in 0..8 {
            let before = g.snapshot();
            let views_before = views(&g);
            let revision = g.revision();
            match g.apply(g.actor().unwrap(), Action::Pass) {
                Ok(()) => (),
                Err(Error::CapacityExceeded(_)) => {
                    assert_eq!(g.snapshot(), before);
                    assert_eq!(views(&g), views_before);
                    assert_eq!(g.revision(), revision);
                    failed = true;
                    break;
                }
                other => panic!("unexpected {other:?}"),
            }
        }
        assert!(failed);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), u64::MAX);
    }
}

#[test]
fn msh_counter_core_doc_noneffect_cost_counters_become_effect_before_doubling() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, DOC);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let r = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        g.activation_cost_counters(g.object_ref(r), CounterKind::Page, 1, p);
        g.advance_replacements(false);
        settle(&mut g);
        assert_eq!(g.counter_count(r, CounterKind::Page), 4);
        let enemy = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        g.activation_cost_counters(g.object_ref(enemy), CounterKind::Page, 1, p);
        g.advance_replacements(false);
        settle(&mut g);
        assert_eq!(g.counter_count(enemy, CounterKind::Page), 1);
    }
}
