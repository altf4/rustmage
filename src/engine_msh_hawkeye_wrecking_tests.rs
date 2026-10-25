//! Paid finite modal triggers, captured limits, and exact owned choices.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
const H: &str = "Hawkeye, Master Marksman";
const W: &str = "The Ruinous Wrecking Crew";
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn pay(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets | Action::PayCosts { .. } | Action::ChooseCastCost { .. }
                )
            })
            .unwrap_or_else(|| panic!("unpaid {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("unpaid");
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (mut m, n) = g.state.objects[id as usize].card.cost();
    m[5] += n;
    g.state.players[p.index()].mana = m;
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    id
}
fn until(g: &mut Game, f: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..150 {
        if f(g) {
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
                        | Action::Discard { .. }
                        | Action::Sacrifice { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled");
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        views
    );
}
fn mode(g: &mut Game, m: u16, targets: &[Target]) {
    checked(g, Action::ChooseMode { mode: m });
    if m == 0 {
        assert!(g.state.announcing.is_none());
        return;
    }
    for &target in targets {
        while !g.legal_actions().contains(&Action::ChooseTarget { target }) {
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    while g.state.announcing.is_some() {
        checked(g, Action::FinishTargets);
    }
}
fn crew_entry(g: &mut Game, p: Player, x: u32) -> CardId {
    priority(g, p);
    let id = named(g, p, W, Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 1, 1, 0, x];
    checked(g, Action::AnnounceCast { card: id });
    checked(g, Action::ChooseX { value: x });
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    until(g, |g| matches!(g.state.pending, Pending::Modes { .. }));
    id
}
fn hawkeye_x(g: &mut Game, p: Player) -> CardId {
    let h = paid(g, p, H);
    let q = paid(g, p, "Dependable Quinjet");
    let bear = paid(g, p, "Grizzly Bears");
    priority(g, p);
    g.state.players[p.index()].mana = [0; 6];
    checked(
        g,
        Action::AnnounceActivate {
            card: q,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseCrewer {
            object: g.object_ref(h),
        },
    );
    checked(
        g,
        Action::ChooseCrewer {
            object: g.object_ref(bear),
        },
    );
    checked(g, Action::FinishCrew);
    until(g, |g| {
        matches!(g.state.pending, Pending::ResolutionX { .. })
    });
    h
}
fn arrows(g: &mut Game, p: Player, n: u32) {
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, n];
    checked(g, Action::ChooseX { value: n });
    if n > 0 {
        checked(
            g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, n],
            },
        );
        until(g, |g| matches!(g.state.pending, Pending::Modes { .. }));
    }
}
fn token(g: &mut Game, p: Player) -> CardId {
    // Bounded canonical Treasure recipe fixture, using the actual central token-entry batch.
    let before = g.state.objects.len();
    g.create_tokens(
        p,
        Card::catalogue().find(|c| c.index() == 5302).unwrap(),
        1,
        false,
    );
    assert_eq!(g.state.objects.len(), before + 1);
    before as CardId
}
fn orphan(g: Game, a: Action) -> Game {
    let before = g.snapshot();
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let i = g.legal_actions().iter().position(|x| *x == a).unwrap();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    drop(g);
    raw.apply(actor, a).unwrap();
    indexed.apply_index(actor, rev, i).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for p in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(raw.observe(p)).unwrap(),
            serde_json::to_value(indexed.observe(p)).unwrap()
        );
    }
    indexed.restore(&before).unwrap();
    assert_eq!(indexed.snapshot(), before);
    raw
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_all_eight_paid_combinations() {
    for p in [Player::P0, Player::P1] {
        for m in 0u16..8 {
            let mut g = setup(p);
            let h = hawkeye_x(&mut g, p);
            let target = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let discard = named(&mut g, p, "Island", Zone::Hand);
            let lib = g.state.players[p.index()].library.len();
            let life = g.state.players[p.other().index()].life;
            arrows(&mut g, p, 3);
            let mut ts = vec![];
            if m & 1 != 0 {
                ts.push(Target::Card(target));
            }
            if m & 2 != 0 {
                ts.push(Target::Player(p.other()));
            }
            mode(&mut g, m, &ts);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                life - if m & 2 != 0 { 2 } else { 0 }
            );
            assert_eq!(
                g.keywords(target).contains(Keywords::CANT_BLOCK),
                m & 1 != 0
            );
            if m & 4 != 0 {
                assert_eq!(g.state.objects[discard as usize].zone, Zone::Graveyard);
                assert_eq!(g.state.players[p.index()].library.len(), lib - 1);
            } else {
                assert_eq!(g.state.objects[discard as usize].zone, Zone::Hand);
            }
            assert_eq!(
                g.stats(h),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert!(
                g.keywords(h)
                    .contains(Keywords::REACH.union(Keywords::FIRST_STRIKE))
            );
        }
    }
}
#[test]
fn msh_hawkeye_wrecking_crew_all_sixteen_paid_combinations() {
    for p in [Player::P0, Player::P1] {
        for m in 0u16..16 {
            let mut g = setup(p);
            let t = token(&mut g, p.other());
            let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let discard = named(&mut g, p, "Island", Zone::Hand);
            let lib = g.state.players[p.index()].library.len();
            let life = g.state.players[p.other().index()].life;
            let w = crew_entry(&mut g, p, 4);
            assert_eq!(g.counter_count(w, CounterKind::PlusOne), 4);
            assert_eq!(
                g.stats(w),
                Stats {
                    power: 6,
                    toughness: 6
                }
            );
            let mut ts = vec![];
            if m & 2 != 0 {
                ts.push(Target::Player(p.other()));
            }
            if m & 4 != 0 {
                ts.push(Target::Card(t));
            }
            mode(&mut g, m, &ts);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                life - if m & 2 != 0 { 2 } else { 0 }
            );
            assert_eq!(g.on_battlefield(t), m & 4 == 0);
            assert_eq!(g.on_battlefield(w), m & 8 == 0);
            assert_eq!(g.on_battlefield(victim), m & 8 == 0);
            if m & 1 != 0 {
                assert_eq!(g.state.objects[discard as usize].zone, Zone::Graveyard);
                assert_eq!(g.state.players[p.index()].library.len(), lib - 1);
            } else {
                assert_eq!(g.state.objects[discard as usize].zone, Zone::Hand);
            }
        }
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_zero_and_decline_have_no_reflexive() {
    for p in [Player::P0, Player::P1] {
        for decline in [false, true] {
            let mut g = setup(p);
            hawkeye_x(&mut g, p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            if decline {
                checked(&mut g, Action::ChooseX { value: 2 });
                checked(&mut g, Action::DeclineResolutionMana);
            } else {
                checked(&mut g, Action::ChooseX { value: 0 });
            }
            assert!(g.state.resolving.is_none());
            assert!(g.state.pending_triggers.is_empty());
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 3]);
            settle(&mut g);
        }
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_limits_payment_then_targets() {
    for p in [Player::P0, Player::P1] {
        for n in 1u32..=3 {
            let mut g = setup(p);
            hawkeye_x(&mut g, p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 10];
            assert_eq!(g.x_choice().unwrap().intervals[0].maximum, 3);
            reject(&mut g, Action::ChooseX { value: 4 });
            arrows(&mut g, p, n);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let modes = g.observe(p).mode_choices;
            for m in 0u16..8 {
                let legal = m.count_ones() <= n;
                assert_eq!(modes.iter().any(|x| x.mode == m), legal, "{m} / {n}");
            }
            reject(
                &mut g,
                Action::ChooseMode {
                    mode: if n < 3 { 7 } else { 8 },
                },
            );
            mode(&mut g, 0, &[]);
            settle(&mut g);
        }
    }
}
#[test]
fn msh_hawkeye_wrecking_crew_x_bounds_and_zero_removal() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 1, 2, 3, 5] {
            let mut g = setup(p);
            token(&mut g, p);
            let w = crew_entry(&mut g, p, x);
            assert_eq!(g.stats(w).power, 2 + i128::from(x));
            for m in 0u16..16 {
                assert_eq!(
                    g.observe(p).mode_choices.iter().any(|v| v.mode == m),
                    m.count_ones() <= x
                );
            }
            if x < 4 {
                reject(&mut g, Action::ChooseMode { mode: 15 });
            }
            mode(&mut g, 0, &[]);
            assert_eq!(g.state.pending, Pending::Priority);
            assert!(g.state.stack.is_empty());
        }
    }
}
#[test]
fn msh_hawkeye_wrecking_crew_source_departure_preserves_x() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let w = crew_entry(&mut g, p, 2);
        g.move_object(w, Zone::Graveyard);
        assert!(g.observe(p).mode_choices.iter().any(|m| m.mode == 3));
        mode(&mut g, 2, &[Target::Player(p.other())]);
        let life = g.state.players[p.other().index()].life;
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
    }
}
#[test]
fn msh_hawkeye_wrecking_crew_destroy_any_token_but_not_card() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            let mut g = setup(p);
            let t = token(&mut g, owner);
            let card = named(&mut g, owner, "Arc Reactor", Zone::Battlefield);
            crew_entry(&mut g, p, 1);
            checked(&mut g, Action::ChooseMode { mode: 4 });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(t),
                },
            );
            while g.state.announcing.is_some() {
                checked(&mut g, Action::FinishTargets);
            }
            settle(&mut g);
            assert!(!g.on_battlefield(t));
            assert!(g.on_battlefield(card));
        }
    }
}
#[test]
fn msh_hawkeye_wrecking_crew_opponent_target_and_all_illegal_cancels_rummage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = token(&mut g, p);
        let d = named(&mut g, p, "Island", Zone::Hand);
        let lib = g.state.players[p.index()].library.len();
        crew_entry(&mut g, p, 2);
        checked(&mut g, Action::ChooseMode { mode: 5 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(t),
            },
        );
        while g.state.announcing.is_some() {
            checked(&mut g, Action::FinishTargets);
        }
        g.move_object(t, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.state.objects[d as usize].zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].library.len(), lib);
        let mut g = setup(p);
        crew_entry(&mut g, p, 1);
        checked(&mut g, Action::ChooseMode { mode: 2 });
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
        while g.state.announcing.is_some() {
            checked(&mut g, Action::FinishTargets);
        }
        settle(&mut g);
    }
}
#[test]
fn msh_hawkeye_wrecking_crew_partial_illegal_target_keeps_other_modes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = token(&mut g, p);
        let life = g.state.players[p.other().index()].life;
        let d = named(&mut g, p, "Island", Zone::Hand);
        crew_entry(&mut g, p, 3);
        mode(&mut g, 7, &[Target::Player(p.other()), Target::Card(t)]);
        g.move_object(t, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        assert_eq!(g.state.objects[d as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_source_lki_and_self_player() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = hawkeye_x(&mut g, p);
        g.move_object(h, Zone::Graveyard);
        arrows(&mut g, p, 1);
        let life = g.state.players[p.index()].life;
        mode(&mut g, 2, &[Target::Player(p)]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 2);
    }
}
#[test]
fn msh_hawkeye_wrecking_empty_hand_rummage_draws() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        crew_entry(&mut g, p, 1);
        let n = g.hand_count(p);
        assert_eq!(n, 0);
        let lib = g.state.players[p.index()].library.len();
        mode(&mut g, 1, &[]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
        assert_eq!(g.state.players[p.index()].library.len(), lib - 1);
    }
}
#[test]
fn msh_hawkeye_wrecking_owned_orphans_x_payment_modes_and_private_discard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        hawkeye_x(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        g = orphan(g, Action::ChooseX { value: 2 });
        g = orphan(
            g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        until(&mut g, |g| matches!(g.state.pending, Pending::Modes { .. }));
        assert!(g.observe(p.other()).mode_choices.is_empty());
        g = orphan(g, Action::ChooseMode { mode: 4 });
        while g.state.announcing.is_some() {
            checked(&mut g, Action::FinishTargets);
        }
        let d = named(&mut g, p, "Island", Zone::Hand);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        g = orphan(g, Action::Discard { card: d });
        settle(&mut g);
        assert_eq!(g.state.objects[d as usize].zone, Zone::Graveyard);
    }
}
fn packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, n) = g.state.objects[id as usize].card.cost();
    mana[5] += n;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
#[test]
fn msh_hawkeye_wrecking_crew_counter_replacement_does_not_increase_mode_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Doubling Season");
        let w = crew_entry(&mut g, p, 1);
        assert_eq!(g.counter_count(w, CounterKind::PlusOne), 2);
        reject(&mut g, Action::ChooseMode { mode: 3 });
        g.add_counter(w, CounterKind::PlusOne, 20);
        reject(&mut g, Action::ChooseMode { mode: 3 });
        mode(&mut g, 2, &[Target::Player(p.other())]);
        settle(&mut g);
    }
}
#[test]
fn msh_hawkeye_wrecking_crew_noncast_return_x_zero_and_counter_reset() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let w = crew_entry(&mut g, p, 4);
        mode(&mut g, 0, &[]);
        packet(&mut g, p, "Murder", &[Target::Card(w)]);
        settle(&mut g);
        packet(&mut g, p, "Zombify", &[Target::Card(w)]);
        until(&mut g, |g| matches!(g.state.pending, Pending::Modes { .. }));
        assert_eq!(g.counter_count(w, CounterKind::PlusOne), 0);
        assert_eq!(g.observe(p).mode_choices.len(), 1);
        mode(&mut g, 0, &[]);
        assert!(g.on_battlefield(w));
    }
}
#[test]
fn msh_hawkeye_wrecking_crew_actual_copy_token_x_zero() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let w = crew_entry(&mut g, p, 3);
        mode(&mut g, 0, &[]);
        packet(&mut g, p, "Self-Reflection", &[Target::Card(w)]);
        until(&mut g, |g| {
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::KeepLegend { .. }))
        });
        let a = g
            .legal_actions()
            .into_iter()
            .find(
                |a| matches!(a,Action::KeepLegend{object} if g.object_view(object.card).is_token()),
            )
            .unwrap();
        checked(&mut g, a);
        until(&mut g, |g| matches!(g.state.pending, Pending::Modes { .. }));
        let c = g.state.announcing.as_ref().unwrap().item.card;
        assert!(g.object_view(c).is_token());
        assert_eq!(g.counter_count(c, CounterKind::PlusOne), 0);
        assert_eq!(g.observe(p).mode_choices.len(), 1);
        mode(&mut g, 0, &[]);
        assert!(g.on_battlefield(c));
    }
}
#[test]
fn msh_hawkeye_wrecking_crew_apnap_sacrifices_are_one_simultaneous_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let w = crew_entry(&mut g, p, 1);
        mode(&mut g, 8, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let own_ref = g.object_ref(own);
        let enemy_ref = g.object_ref(enemy);
        assert_eq!(g.actor(), Some(p));
        reject(&mut g, Action::Sacrifice { object: enemy_ref });
        checked(&mut g, Action::Sacrifice { object: own_ref });
        assert!(g.on_battlefield(own) && g.on_battlefield(enemy));
        assert_eq!(g.actor(), Some(p.other()));
        g = orphan(g, Action::Sacrifice { object: enemy_ref });
        assert_eq!(g.state.objects[own as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
        assert!(g.on_battlefield(w));
        settle(&mut g);
    }
}
#[test]
fn msh_hawkeye_wrecking_crew_destroy_creature_token_before_sacrifice_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = Card::catalogue()
            .find(|c| {
                c.token_info().is_some()
                    && c.is_supported()
                    && matches!(c.effect(), Effect::Creature(_, _))
            })
            .unwrap();
        let n = g.state.objects.len();
        g.create_tokens(p.other(), c, 1, false);
        let t = n as CardId;
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let w = crew_entry(&mut g, p, 2);
        mode(&mut g, 12, &[Target::Card(t)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let w_ref = g.object_ref(w);
        let t_ref = g.object_ref(t);
        let enemy_ref = g.object_ref(enemy);
        assert!(!g.on_battlefield(t));
        checked(&mut g, Action::Sacrifice { object: w_ref });
        reject(&mut g, Action::Sacrifice { object: t_ref });
        checked(&mut g, Action::Sacrifice { object: enemy_ref });
        settle(&mut g);
        assert!(!g.on_battlefield(w));
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_real_response_removal_preserves_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = hawkeye_x(&mut g, p);
        arrows(&mut g, p, 1);
        mode(&mut g, 2, &[Target::Player(p.other())]);
        let life = g.state.players[p.other().index()].life;
        packet(&mut g, p, "Murder", &[Target::Card(h)]);
        settle(&mut g);
        assert_eq!(g.state.objects[h as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_tap_transitions_and_captured_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = hawkeye_x(&mut g, p);
        arrows(&mut g, p, 0);
        settle(&mut g);
        g.tap_objects(&[h]);
        assert!(g.state.pending_triggers.is_empty());
        g.untap_objects(&[h], untap_rules::UntapCause::Effect);
        g.tap_objects(&[h]);
        g.change_controller(g.object_ref(h), p.other());
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionX { .. })
        });
        assert_eq!(g.actor(), Some(p));
        arrows(&mut g, p, 1);
        mode(&mut g, 2, &[Target::Player(p.other())]);
        let life = g.state.players[p.other().index()].life;
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        assert_eq!(g.state.objects[h as usize].controller, p.other());
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_payment_uses_actual_mana_abilities_and_lot_rules() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        hawkeye_x(&mut g, p);
        let castle = named(&mut g, p, "Castle Doom", Zone::Battlefield);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        assert_eq!(g.x_choice().unwrap().intervals[0].maximum, 3); // Quinjet is also an untapped mana producer.
        checked(&mut g, Action::ChooseX { value: 1 });
        reject(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        checked(
            &mut g,
            Action::ActivateMana {
                card: castle,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        reject(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        checked(&mut g, Action::TapMana { card: forest });
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 1, 0],
            },
        );
        until(&mut g, |g| matches!(g.state.pending, Pending::Modes { .. }));
        mode(&mut g, 0, &[]);
        settle(&mut g);
        assert!(g.state.objects[castle as usize].tapped && g.state.objects[forest as usize].tapped);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_all_illegal_cancels_boomerang() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        hawkeye_x(&mut g, p);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let d = named(&mut g, p, "Island", Zone::Hand);
        arrows(&mut g, p, 2);
        mode(&mut g, 5, &[Target::Card(victim)]);
        g.move_object(victim, Zone::Exile);
        let lib = g.state.players[p.index()].library.len();
        settle(&mut g);
        assert_eq!(g.state.objects[d as usize].zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].library.len(), lib);
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_net_blocks_actual_block_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        hawkeye_x(&mut g, p);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let attacker = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        arrows(&mut g, p, 1);
        mode(&mut g, 1, &[Target::Card(victim)]);
        settle(&mut g);
        until(&mut g, |g| matches!(g.state.pending, Pending::Attackers));
        checked(&mut g, Action::Attack { card: attacker });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| matches!(g.state.pending, Pending::Blockers));
        reject(
            &mut g,
            Action::Block {
                blocker: victim,
                attacker,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        let turn = g.state.turn;
        until(&mut g, |g| g.state.turn != turn);
        assert!(!g.keywords(victim).contains(Keywords::CANT_BLOCK));
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_reach_blocks_flyer_first_strike_kills_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, H);
        let flyer = named(&mut g, p.other(), "Healer's Hawk", Zone::Battlefield);
        let ground = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        until(&mut g, |g| {
            g.state.active == p.other() && matches!(g.state.pending, Pending::Attackers)
        });
        checked(&mut g, Action::Attack { card: flyer });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| matches!(g.state.pending, Pending::Blockers));
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: flyer,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: h,
                attacker: flyer,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.step == Step::CombatDamage);
        assert_eq!(g.state.objects[flyer as usize].zone, Zone::Graveyard);
        assert!(g.on_battlefield(h));
        assert_eq!(g.state.objects[h as usize].damage, 0);
    }
}
#[test]
fn msh_hawkeye_wrecking_crew_exact_colored_payment_and_stack_x() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let w = named(&mut g, p, W, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 10];
        reject(&mut g, Action::AnnounceCast { card: w });
        g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 2];
        checked(&mut g, Action::AnnounceCast { card: w });
        assert_eq!(g.x_choice().unwrap().intervals[0].maximum, 2);
        reject(&mut g, Action::ChooseX { value: 3 });
        checked(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.stack.last().unwrap().mana_value(), 4);
        until(&mut g, |g| matches!(g.state.pending, Pending::Modes { .. }));
        mode(&mut g, 0, &[]);
        assert_eq!(g.object_view(w).card.fixed_mana_value(), 2);
    }
}
fn wasp_target(g: &mut Game, p: Player, target: CardId) {
    let wasp = packet(g, p, "The Wondrous Wasp", &[]);
    until(g, |g| {
        g.state
            .announcing
            .as_ref()
            .is_some_and(|a| a.item.card == wasp)
    });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    while g.state.announcing.is_some() {
        checked(g, Action::FinishTargets);
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_tap_packet_survives_real_wasp_suppression() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, H);
        wasp_target(&mut g, p, h);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionX { .. })
        });
        assert!(!g.printed_abilities_available(h));
        arrows(&mut g, p, 1);
        mode(&mut g, 2, &[Target::Player(p.other())]);
        let life = g.state.players[p.other().index()].life;
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_net_is_rule_restriction_after_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        hawkeye_x(&mut g, p);
        let t = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        arrows(&mut g, p, 1);
        mode(&mut g, 1, &[Target::Card(t)]);
        settle(&mut g);
        wasp_target(&mut g, p, t);
        settle(&mut g);
        assert!(!g.printed_abilities_available(t));
        assert!(g.keywords(t).contains(Keywords::CANT_BLOCK));
    }
}
#[test]
fn msh_hawkeye_wrecking_hawkeye_ward_counters_one_entire_reflexive_packet() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        hawkeye_x(&mut g, p);
        let t = named(
            &mut g,
            p.other(),
            "Titania, Rugged Rumbler",
            Zone::Battlefield,
        );
        arrows(&mut g, p, 2);
        mode(&mut g, 3, &[Target::Card(t), Target::Player(p.other())]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        let life = g.state.players[p.other().index()].life;
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert!(!g.keywords(t).contains(Keywords::CANT_BLOCK));
        assert_eq!(g.state.players[p.other().index()].life, life);
    }
}
