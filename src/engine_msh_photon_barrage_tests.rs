//! Paid X-copy cast scenarios through ordinary support-checked setup.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
const PHOTON: &str = "Photon Blast Barrage";
fn hand(g: &mut Game, p: Player, name: &str) -> CardId {
    named(g, p, name, Zone::Hand)
}

fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str, mode: Option<u16>, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = hand(g, p, name);
    let (colored, generic) = g.state.objects[id as usize].card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, mode, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    if g.state.active != p {
        next_turn(g);
        until(g, |g| g.state.active == p && g.state.step == Step::Main1);
    }
    let id = packet(g, p, name, None, &[]);
    settle(g);
    id
}
fn until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..200 {
        if done(g) {
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
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unhandled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("continuation did not complete");
}
fn reject(g: &mut Game, a: Action) {
    let snapshot = g.snapshot();
    let bytes = g.state_bytes();
    let views = serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap(),
        views
    );
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut o = g.observe(p);
        o.revision = 0;
        serde_json::to_value(o).unwrap()
    })
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let before = g.snapshot();
    let before_views = views(&g);
    let index = g.legal_actions().iter().position(|x| *x == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, rev, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}

fn finish(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
}
fn photon(g: &mut Game, p: Player, x: u32, target: CardId) -> CardId {
    priority(g, p);
    let id = hand(g, p, PHOTON);
    g.state.players[p.index()].mana = [0, 0, 0, 2, 0, x];
    checked(g, Action::AnnounceCast { card: id });
    checked(g, Action::ChooseX { value: x });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn retarget(g: &mut Game) {
    until(g, |g| matches!(g.state.pending, Pending::Retarget { .. }));
}
fn finish_copies(g: &mut Game, n: u32) {
    for _ in 0..n {
        retarget(g);
        checked(g, Action::FinishStackTargets);
    }
}
#[test]
fn msh_photon_paid_x_zero_no_copies_and_exact_one_creature_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let b = photon(&mut g, p, 0, h);
        settle(&mut g);
        assert_eq!(g.object_view(h).damage, 1);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
    }
}
#[test]
fn msh_photon_paid_x_three_retains_optional_targets_and_chosen_x_without_recasting() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let b = photon(&mut g, p, 3, h);
        finish_copies(&mut g, 3);
        let copies = g
            .state
            .stack
            .iter()
            .filter(|s| s.is_copy && s.source_card == Card::from_name(PHOTON).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(copies.len(), 3);
        assert!(
            copies
                .iter()
                .all(|s| s.x == Some(3) && s.targets[0].target == Target::Card(h))
        );
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
        settle(&mut g);
        assert_eq!(g.object_view(h).zone, Zone::Graveyard);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn msh_photon_each_copy_targets_independently_without_changing_original() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let original = photon(&mut g, p, 2, a);
        retarget(&mut g);
        reject(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        g = orphan(g, Action::FinishStackTargets);
        retarget(&mut g);
        checked(&mut g, Action::FinishStackTargets);
        assert_eq!(
            g.state
                .stack
                .iter()
                .find(|s| s.card == original)
                .unwrap()
                .targets[0]
                .target,
            Target::Card(a)
        );
        settle(&mut g);
        assert_eq!(g.object_view(a).damage, 2);
        assert_eq!(g.object_view(b).damage, 1);
    }
}
#[test]
fn msh_photon_countered_original_keeps_frozen_spell_and_x_for_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let b = photon(&mut g, p, 2, h);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.card == b && s.kind != StackKind::Spell)
                && g.state.pending == Pending::Priority
        });
        packet(&mut g, p.other(), "Negate", None, &[Target::Spell(b)]);
        finish_copies(&mut g, 2);
        settle(&mut g);
        assert_eq!(g.object_view(h).damage, 2);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_photon_illegal_old_target_may_be_kept_or_one_copy_can_be_retargeted() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let other = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        photon(&mut g, p, 2, h);
        until(&mut g, |g| {
            g.state.stack.len() == 2 && g.state.pending == Pending::Priority
        });
        packet(&mut g, p, "Unsummon", None, &[Target::Card(h)]);
        retarget(&mut g);
        checked(&mut g, Action::FinishStackTargets);
        retarget(&mut g);
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(other),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(other).damage, 1);
        assert_eq!(g.object_view(h).zone, Zone::Hand);
    }
}
#[test]
fn msh_photon_x_cost_timing_target_failures_and_nonactor_privacy_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let b = hand(&mut g, p, PHOTON);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 1];
        checked(&mut g, Action::AnnounceCast { card: b });
        reject(&mut g, Action::ChooseX { value: 2 });
        checked(&mut g, Action::ChooseX { value: 1 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(h),
            },
        );
        finish(&mut g);
        let hidden = named(&mut g, p.other(), "Murder", Zone::Hand);
        retarget(&mut g);
        let mut alt = g.clone();
        alt.state.objects[hidden as usize].card = Card::from_name("Negate").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(alt.observe(p)).unwrap()
        );
        g = orphan(g, Action::FinishStackTargets);
        settle(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        priority(&mut g, p);
        let b = hand(&mut g, p, PHOTON);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: b });
    }
}
#[test]
fn msh_photon_copy_target_ward_choices_are_independent_real_payments() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let original = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let ward = named(&mut g, p.other(), "Thornfist Striker", Zone::Battlefield);
        photon(&mut g, p, 2, original);
        for _ in 0..2 {
            retarget(&mut g);
            checked(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(ward),
                },
            );
            checked(&mut g, Action::FinishStackTargets);
        }
        for pay in [false, true] {
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::DeclineResolutionMana)
            });
            if pay {
                g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
                g = orphan(
                    g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 1],
                    },
                );
            } else {
                g = orphan(g, Action::DeclineResolutionMana);
            }
        }
        settle(&mut g);
        assert_eq!(g.object_view(ward).damage, 1);
        assert_eq!(g.object_view(original).damage, 1);
    }
}
#[test]
fn msh_photon_cast_observer_fires_once_and_real_noncast_spell_copy_has_no_cast_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, "Fin Fang Foom");
        let h = named(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
        g.state.objects[h as usize].counters.plus_one = 3;
        let b = photon(&mut g, p, 1, h);
        finish_copies(&mut g, 2);
        let copies = g
            .state
            .stack
            .iter()
            .filter(|s| s.is_copy && s.source_card == Card::from_name(PHOTON).unwrap())
            .count();
        assert_eq!(copies as i128 + g.object_view(h).damage, 2);
        settle(&mut g);
        assert_eq!(g.object_view(fin).counters.plus_one, 2);
        assert_eq!(g.object_view(h).damage, 3);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
    }
}
