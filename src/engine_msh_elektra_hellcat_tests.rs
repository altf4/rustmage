//! Paid card scenarios with exact recipients, public menus and owned action replay.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
const ELEKTRA: &str = "Elektra, Daughter of the Hand";
const HELLCAT: &str = "Hellcat, Undying Vigilante";
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn pay(g: &mut Game) {
    for _ in 0..12 {
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
            .unwrap_or_else(|| panic!("payment {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("unpaid");
}
pub(super) fn packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (mut m, n) = g.state.objects[id as usize].card.cost();
    m[5] += n;
    g.state.players[p.index()].mana = m;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
pub(super) fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = packet(g, p, name, targets);
    settle(g);
    id
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let v = views(g);
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(views(g), v);
    assert_eq!(g.revision(), r);
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap())
}
pub(super) fn until(g: &mut Game, f: impl Fn(&Game) -> bool) {
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
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::Discard { .. }
                )
            })
            .unwrap_or_else(|| panic!("pending {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled");
}
fn main(g: &mut Game) {
    until(g, |g| {
        g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
    settle(g);
}
fn unblocked(g: &mut Game, p: Player, attacker: CardId, defender: CombatDefender) {
    until(g, |g| matches!(g.state.pending, Pending::Attackers));
    checked(
        g,
        match defender {
            CombatDefender::Player(_) => Action::Attack { card: attacker },
            CombatDefender::Planeswalker(defender) => Action::AttackPlaneswalker {
                card: attacker,
                defender,
            },
        },
    );
    checked(g, Action::FinishAttackers);
    until(g, |g| matches!(g.state.pending, Pending::Blockers));
    checked(g, Action::FinishBlockers);
    priority(g, p);
}
fn sneak(g: &mut Game, p: Player, id: CardId, attacker: CardId) {
    priority(g, p);
    g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
    checked(
        g,
        Action::AnnounceSneak {
            card: id,
            permission: None,
        },
    );
    checked(
        g,
        Action::ChooseAdditionalCost {
            object: g.object_ref(attacker),
        },
    );
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
#[test]
fn msh_elektra_hellcat_ordinary_paid_hellcat_returns_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HELLCAT, &[]);
        assert!(g.keywords(h).contains(Keywords::HASTE));
        paid(&mut g, p, "Murder", &[Target::Card(h)]);
        assert!(g.on_battlefield(h));
        assert_eq!(
            g.stats(h),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(!g.printed_abilities_available(h));
        assert!(g.keywords(h).contains(Keywords::HASTE));
        assert_eq!(g.resolved_ability_modification_views().unwrap().len(), 1);
        paid(&mut g, p, "Murder", &[Target::Card(h)]);
        assert_eq!(g.state.objects[h as usize].zone, Zone::Graveyard);
        assert!(g.resolved_ability_modification_views().is_none());
    }
}
#[test]
fn msh_elektra_hellcat_ordinary_paid_elektra_etb() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let e = paid(&mut g, p, ELEKTRA, &[]);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
        assert!(!g.state.objects[e as usize].tapped);
        assert_eq!(
            g.stats(e),
            Stats {
                power: 3,
                toughness: 3
            }
        );
    }
}
#[test]
fn msh_elektra_hellcat_paid_sneak_returns_attacker_and_enters_attacking() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HELLCAT, &[]);
        let e = named(&mut g, p, ELEKTRA, Zone::Hand);
        unblocked(&mut g, p, h, CombatDefender::Player(p.other()));
        sneak(&mut g, p, e, h);
        assert_eq!(g.state.objects[h as usize].zone, Zone::Hand);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Sneak
        );
        settle(&mut g);
        assert!(g.on_battlefield(e));
        assert!(g.state.objects[e as usize].tapped);
        assert!(
            g.state
                .attackers
                .iter()
                .any(|a| a.card == e && a.defender == Some(CombatDefender::Player(p.other())))
        );
    }
}
fn trigger(g: &mut Game, id: CardId, target: CardId) {
    until(g, |g| {
        g.state
            .announcing
            .as_ref()
            .is_some_and(|a| a.item.card == id)
    });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
}
fn orphan(g: Game, a: Action) -> Game {
    let p = g.actor().unwrap();
    let r = g.revision();
    let before = g.snapshot();
    let seen = views(&g);
    let ix = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let (mut raw, mut index) = (g.clone(), g.clone());
    drop(g);
    raw.apply(p, a.clone()).unwrap();
    index.apply_index(p, r, ix).unwrap();
    assert_eq!(raw.snapshot(), index.snapshot());
    assert_eq!(views(&raw), views(&index));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    let mut seen = seen;
    for v in &mut seen {
        v["revision"] = serde_json::json!(raw.revision());
    }
    assert_eq!(views(&raw), seen);
    assert_eq!(raw.apply_index(p, r, ix), Err(Error::StaleRevision));
    raw.apply(p, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    raw
}
fn fresh_sneak(p: Player) -> (Game, CardId, CardId) {
    let mut g = setup(p);
    let h = paid(&mut g, p, HELLCAT, &[]);
    let e = named(&mut g, p, ELEKTRA, Zone::Hand);
    unblocked(&mut g, p, h, CombatDefender::Player(p.other()));
    (g, h, e)
}
#[test]
fn msh_elektra_hellcat_sneak_timing_blocked_and_other_controller_are_rejected() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HELLCAT, &[]);
        let e = named(&mut g, p, ELEKTRA, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: None,
            },
        );
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: h });
        checked(&mut g, Action::FinishAttackers);
        priority(&mut g, p);
        reject(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: None,
            },
        );
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: h,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: None,
            },
        );
        paid(&mut g, p, "Murder", &[Target::Card(b)]);
        reject(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: None,
            },
        );
    }
}
#[test]
fn msh_elektra_hellcat_sneak_short_mana_and_illegal_return_atomic() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, e) = fresh_sneak(p);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: None,
            },
        );
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        checked(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: None,
            },
        );
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        for id in [other, own, e] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let r = g.object_ref(h);
        checked(&mut g, Action::ChooseAdditionalCost { object: r });
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        pay(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(e));
    }
}
#[test]
fn msh_elektra_hellcat_sneak_countered_spell_keeps_return_cost_paid() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, e) = fresh_sneak(p);
        sneak(&mut g, p, e, h);
        let c = paid(&mut g, p.other(), "Essence Scatter", &[Target::Spell(e)]);
        assert_eq!(g.state.objects[c as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[e as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[h as usize].zone, Zone::Hand);
        assert!(g.state.attackers.is_empty());
    }
}
#[test]
fn msh_elektra_hellcat_sneak_captured_planeswalker_departure_does_not_redirect() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = setup(p);
            let h = paid(&mut g, p, HELLCAT, &[]);
            let pw = named(
                &mut g,
                p.other(),
                "Liliana, Dreadhorde General",
                Zone::Battlefield,
            );
            // The ordinary battlefield fixture does not apply entering loyalty.
            g.state.objects[pw as usize].counters.loyalty = 6;
            let e = named(&mut g, p, ELEKTRA, Zone::Hand);
            let r = g.object_ref(pw);
            unblocked(&mut g, p, h, CombatDefender::Planeswalker(r));
            sneak(&mut g, p, e, h);
            if leave {
                paid(&mut g, p, "Hero's Downfall", &[Target::Card(pw)]);
            }
            settle(&mut g);
            assert!(g.state.objects[e as usize].tapped);
            assert_eq!(
                g.state
                    .attackers
                    .iter()
                    .find(|a| a.card == e)
                    .and_then(|a| a.defender),
                if leave {
                    None
                } else {
                    Some(CombatDefender::Planeswalker(r))
                }
            );
        }
    }
}
#[test]
fn msh_elektra_hellcat_sneak_nonhand_permissions_and_prohibition() {
    for p in [Player::P0, Player::P1] {
        for zone in [Zone::Graveyard, Zone::Exile, Zone::Library] {
            let (mut g, h, e) = fresh_sneak(p);
            g.move_object(e, zone);
            g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
            reject(
                &mut g,
                Action::AnnounceSneak {
                    card: e,
                    permission: None,
                },
            );
            let r = g.object_ref(e);
            match zone {
                Zone::Graveyard => g.grant_graveyard_permission(p, r, GraveyardCastKind::Cast),
                Zone::Exile => g.grant_exile_play_permission(p, r, false),
                Zone::Library => {
                    // Bounded public top-library permission provider, using its admitted typed static.
                    named(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
                    let lib = &mut g.state.players[p.index()].library;
                    lib.retain(|v| *v != e);
                    lib.push(e);
                }
                _ => unreachable!(),
            };
            sneak(&mut g, p, e, h);
            assert_eq!(g.state.stack.last().unwrap().cast.unwrap().origin, zone);
            settle(&mut g);
            assert!(g.on_battlefield(e));
        }
    }
}
#[test]
fn msh_elektra_hellcat_sneak_actual_lands_and_non_targeted_protection_payment() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, e) = fresh_sneak(p); // Bounded keyword seam: return is not targeting.
        g.grant_keyword_effects(&[h], g.object_ref(h), Keywords::PROTECTION_EVERYTHING);
        let lands = (0..3)
            .map(|_| named(&mut g, p, "Swamp", Zone::Battlefield))
            .collect::<Vec<_>>();
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: None,
            },
        );
        let object = g.object_ref(h);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        for l in lands {
            checked(&mut g, Action::TapMana { card: l });
        }
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert!(g.on_battlefield(e));
    }
}
#[test]
fn msh_elektra_hellcat_sneak_owned_return_payment_and_hidden_hand() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, e) = fresh_sneak(p);
        let hidden = named(&mut g, p, "Divination", Zone::Hand);
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == e || o.id == hidden)
        );
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        checked(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: None,
            },
        );
        assert!(g.observe(p.other()).additional_cost.is_none());
        let a = Action::ChooseAdditionalCost {
            object: g.object_ref(h),
        };
        g = orphan(g, a);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        g = orphan(g, a);
        settle(&mut g);
        assert!(g.on_battlefield(e));
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == hidden));
    }
}
#[test]
fn msh_elektra_hellcat_sneak_does_not_declare_another_attack() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, e) = fresh_sneak(p);
        let count = g.state.attacked_this_turn.len();
        sneak(&mut g, p, e, h);
        settle(&mut g);
        assert_eq!(g.state.attacked_this_turn.len(), count);
        assert!(!g.state.attacked_this_turn.contains(&g.object_ref(e)));
        assert!(g.state.attackers.iter().any(|a| a.card == e));
    }
}
#[test]
fn msh_elektra_hellcat_etb_targets_current_opponent_power_and_rechecks() {
    for p in [Player::P0, Player::P1] {
        for grow in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let giant = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let e = packet(&mut g, p, ELEKTRA, &[]);
            until(&mut g, |g| {
                g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.item.card == e)
            });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            ); // Hill Giant power3 is legal; actual growth makes it illegal later.
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            pay(&mut g);
            if grow {
                paid(&mut g, p.other(), "Giant Growth", &[Target::Card(bear)]);
            }
            settle(&mut g);
            assert_eq!(g.on_battlefield(bear), grow);
            assert!(g.on_battlefield(giant));
        }
    }
}
#[test]
fn msh_elektra_hellcat_etb_large_power_illegal_negative_power_legal() {
    for p in [Player::P0, Player::P1] {
        for negative in [false, true] {
            let mut g = setup(p);
            let large = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let zero = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            // Bounded characteristic seam keeps a negative-power creature alive.
            if negative {
                g.state.objects[zero as usize].growth.power = -2;
            }
            let e = packet(&mut g, p, ELEKTRA, &[]);
            until(&mut g, |g| {
                g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.item.card == e)
            });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(large),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(zero),
                },
            );
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.state.objects[zero as usize].zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_elektra_hellcat_etb_independent_of_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let e = packet(&mut g, p, ELEKTRA, &[]);
        trigger(&mut g, e, b);
        paid(&mut g, p, "Murder", &[Target::Card(e)]);
        assert_eq!(g.state.objects[b as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn msh_elektra_hellcat_hellcat_stolen_dies_returns_to_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HELLCAT, &[]);
        super::assurance::big_helpers::next_turn(&mut g);
        main(&mut g);
        assert_eq!(g.state.active, p.other());
        paid(
            &mut g,
            p.other(),
            "Involuntary Employment",
            &[Target::Card(h)],
        );
        assert_eq!(g.state.objects[h as usize].controller, p.other());
        paid(&mut g, p.other(), "Murder", &[Target::Card(h)]);
        assert!(g.on_battlefield(h));
        assert_eq!(g.state.objects[h as usize].controller, p);
        assert!(!g.printed_abilities_available(h));
    }
}
#[test]
fn msh_elektra_hellcat_hellcat_counter_doubling_and_owner_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Doubling Season", &[]);
        let h = paid(&mut g, p, HELLCAT, &[]);
        paid(&mut g, p, "Murder", &[Target::Card(h)]);
        assert_eq!(g.counter_count(h, CounterKind::PlusOne), 2);
        assert_eq!(
            g.stats(h),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert_eq!(g.keywords(h), Keywords::HASTE);
    }
}
#[test]
fn msh_elektra_hellcat_hellcat_leaving_graveyard_does_not_rebind() {
    for p in [Player::P0, Player::P1] {
        for back in [false, true] {
            let mut g = setup(p);
            let h = paid(&mut g, p, HELLCAT, &[]);
            packet(&mut g, p, "Murder", &[Target::Card(h)]);
            until(&mut g, |g| g.state.stack.iter().any(|s| s.card == h));
            let old = g.object_ref(h); // Bounded public movement while the real paid death trigger waits.
            g.move_object(h, Zone::Exile);
            if back {
                g.move_object(h, Zone::Graveyard);
            }
            assert_ne!(g.object_ref(h), old);
            settle(&mut g);
            assert!(!g.on_battlefield(h));
            assert!(g.resolved_ability_modification_views().is_none());
        }
    }
}
#[test]
fn msh_elektra_hellcat_hellcat_exile_replacement_no_death_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Rest in Peace", &[]);
        let h = paid(&mut g, p, HELLCAT, &[]);
        paid(&mut g, p, "Murder", &[Target::Card(h)]);
        assert_eq!(g.state.objects[h as usize].zone, Zone::Exile);
        assert!(g.resolved_ability_modification_views().is_none());
    }
}
#[test]
fn msh_elektra_hellcat_hellcat_blank_lasts_cleanup_copy_and_control_but_blink_resets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HELLCAT, &[]);
        paid(&mut g, p, "Murder", &[Target::Card(h)]);
        let row = g.resolved_ability_modification_views().unwrap()[0]; // Bounded characteristic/control seams on a genuinely returned paid Hellcat.
        let donor = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let recipe = g.copiable_recipe(g.object_ref(donor));
        g.apply_temporary_copy(g.object_ref(h), recipe, CopyDeadline::EndOfTurn);
        assert!(!g.keywords(h).contains(Keywords::FLYING));
        assert_eq!(g.keywords(h), Keywords::HASTE);
        g.change_controller(g.object_ref(h), p.other());
        assert!(!g.printed_abilities_available(h));
        assert_eq!(g.resolved_ability_modification_views().unwrap()[0], row);
        super::assurance::big_helpers::next_turn(&mut g);
        main(&mut g);
        assert!(!g.printed_abilities_available(h));
        paid(&mut g, p.other(), "Unsummon", &[Target::Card(h)]);
        assert!(g.resolved_ability_modification_views().is_none());
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 0];
        cast(&mut g, h, None, &[]);
        settle(&mut g);
        assert!(g.printed_abilities_available(h));
        assert_eq!(g.counter_count(h, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_elektra_hellcat_hellcat_later_grants_and_funding_obey_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HELLCAT, &[]);
        paid(&mut g, p, "Murder", &[Target::Card(h)]); // Bounded layer-one conversion checks actual and speculative mana ability removal.
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let recipe = g.copiable_recipe(g.object_ref(forest));
        g.apply_temporary_copy(g.object_ref(h), recipe, CopyDeadline::EndOfTurn);
        assert!(!g.legal_actions().contains(&Action::TapMana { card: h }));
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 2, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        g.state.players[p.index()].mana = [0; 6];
        assert!(!g.can_fund(goal));
        let source = row_source(&g, h);
        g.grant_keyword_effects(&[h], source, Keywords::FLYING);
        assert!(
            g.keywords(h)
                .contains(Keywords::HASTE.union(Keywords::FLYING))
        );
    }
}
fn row_source(g: &Game, h: CardId) -> ObjectRef {
    g.resolved_ability_modification_views()
        .unwrap()
        .iter()
        .find(|r| r.recipient.card == h)
        .unwrap()
        .source
}
#[test]
fn msh_elektra_hellcat_hellcat_paid_death_owned_pending_and_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HELLCAT, &[]);
        packet(&mut g, p, "Murder", &[Target::Card(h)]);
        until(&mut g, |g| g.state.stack.iter().any(|s| s.card == h));
        let a = Action::Pass;
        g = orphan(g, a);
        settle(&mut g);
        let row = g.resolved_ability_modification_views().unwrap()[0];
        assert_eq!(row.recipient, g.object_ref(h));
        assert_ne!(row.source, row.recipient);
        assert_eq!(
            views(&g)[0]["resolved_ability_modifications"],
            views(&g)[1]["resolved_ability_modifications"]
        );
    }
}
#[test]
fn msh_elektra_hellcat_sneak_muldrotha_consumes_selected_type_permission() {
    for p in [Player::P0, Player::P1] {
        let (mut g, h, e) = fresh_sneak(p);
        g.move_object(e, Zone::Graveyard);
        let m = named(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        let permission = g.object_ref(m);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: None,
            },
        );
        checked(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: Some((permission, PermanentType::Creature)),
            },
        );
        let object = g.object_ref(h);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(e));
        let second = named(&mut g, p, ELEKTRA, Zone::Graveyard);
        reject(
            &mut g,
            Action::AnnounceSneak {
                card: second,
                permission: Some((permission, PermanentType::Creature)),
            },
        );
        assert!(
            g.observe(p)
                .graveyard_type_uses
                .iter()
                .any(|r| r.source == permission && r.permanent_type == PermanentType::Creature)
        );
    }
}
#[test]
fn msh_elektra_hellcat_return_entry_choice_pause_redirect_and_owned_result() {
    for p in [Player::P0, Player::P1] {
        for redirect in [false, true] {
            let mut g = setup(p);
            let a = paid(&mut g, p, "Adaptive Automaton", &[]);
            let donor = named(&mut g, p, HELLCAT, Zone::Graveyard); // Bounded copy seam: a genuine paid creature acquires the real Hellcat death program.
            let recipe = g.copiable_recipe(g.object_ref(donor));
            g.apply_temporary_copy(g.object_ref(a), recipe, CopyDeadline::EndOfTurn);
            packet(&mut g, p, "Murder", &[Target::Card(a)]);
            until(&mut g, |g| {
                g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseEntryValue { .. }))
            });
            assert!(g.resolved_ability_modification_views().is_none());
            let batch = g.state.entry_batch.as_mut().unwrap();
            assert!(batch.moves[0].blank_with_haste_after.is_some());
            if redirect {
                batch.moves[0].destination = Zone::Exile;
            }
            let choice = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
                .unwrap();
            g = orphan(g, choice);
            settle(&mut g);
            assert_eq!(
                g.state.objects[a as usize].card,
                Card::from_name("Adaptive Automaton").unwrap()
            );
            assert_eq!(g.on_battlefield(a), !redirect);
            if !redirect {
                assert!(!g.printed_abilities_available(a));
                assert_eq!(g.keywords(a), Keywords::HASTE);
                assert_eq!(g.counter_count(a, CounterKind::PlusOne), 1);
            } else {
                assert!(g.resolved_ability_modification_views().is_none());
            }
        }
    }
}
#[test]
fn msh_elektra_hellcat_paid_copy_token_has_copiable_death_ability_but_cannot_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HELLCAT, &[]);
        paid(&mut g, p, "Murder", &[Target::Card(h)]);
        packet(&mut g, p, "Self-Reflection", &[Target::Card(h)]);
        until(&mut g, |g| {
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::KeepLegend { .. }))
        });
        let t = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(i, o)| *i != h as usize && o.zone == Zone::Battlefield && o.is_copy_token())
            .map(|(i, _)| i as CardId)
            .unwrap();
        assert!(g.printed_abilities_available(t));
        assert_eq!(g.state.objects[t as usize].card.triggers().len(), 1);
        let object = g.object_ref(h);
        checked(&mut g, Action::KeepLegend { object });
        settle(&mut g);
        assert!(!g.on_battlefield(t));
        assert!(
            g.resolved_ability_modification_views()
                .unwrap()
                .iter()
                .all(|r| r.recipient.card != t)
        );
    }
}
#[test]
fn msh_elektra_hellcat_return_entry_trigger_freezes_before_lasting_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = paid(&mut g, p, "Inspiring Overseer", &[]);
        let donor = named(&mut g, p, HELLCAT, Zone::Graveyard); // Bounded copy acquisition on an actual paid entrant; return uses its physical definition.
        let recipe = g.copiable_recipe(g.object_ref(donor));
        g.apply_temporary_copy(g.object_ref(c), recipe, CopyDeadline::EndOfTurn);
        let library = g.state.players[p.index()].library.len();
        let life = g.state.players[p.index()].life;
        paid(&mut g, p, "Murder", &[Target::Card(c)]);
        assert!(g.on_battlefield(c));
        assert_eq!(
            g.state.objects[c as usize].card,
            Card::from_name("Inspiring Overseer").unwrap()
        );
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
        assert_eq!(g.state.players[p.index()].life, life + 1);
        assert_eq!(
            g.stats(c),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert!(!g.printed_abilities_available(c));
        assert_eq!(g.keywords(c), Keywords::HASTE);
    }
}
#[test]
fn msh_elektra_hellcat_sneak_mana_attacker_can_tap_before_its_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, "Llanowar Elves", &[]); // Bounded vigilance/sickness setup isolates a real tap-mana and return cost sharing one creature.
        g.state.objects[h as usize].summoning_sick = false;
        let r = g.object_ref(h);
        g.grant_keyword_effects(&[h], r, Keywords::VIGILANCE);
        let e = named(&mut g, p, ELEKTRA, Zone::Hand);
        let a = named(&mut g, p, "Swamp", Zone::Battlefield);
        let b = named(&mut g, p, "Swamp", Zone::Battlefield);
        unblocked(&mut g, p, h, CombatDefender::Player(p.other()));
        assert!(!g.state.objects[h as usize].tapped);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: None,
            },
        );
        let object = g.object_ref(h);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        for card in [a, b, h] {
            checked(&mut g, Action::TapMana { card });
        }
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert!(g.on_battlefield(e));
        assert_eq!(g.state.objects[h as usize].zone, Zone::Hand);
    }
}
#[test]
fn msh_elektra_hellcat_sneak_actual_copy_token_return_is_payable() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HELLCAT, &[]);
        packet(&mut g, p, "Molten Duplication", &[Target::Card(h)]);
        for _ in 0..100 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.announcing.is_none()
            {
                break;
            }
            let actions = g.legal_actions();
            let action=actions.iter().find(|a|matches!(a,Action::KeepLegend{object} if g.object_view(object.card).is_token())).or_else(||actions.iter().find(|a|matches!(a,Action::Pass|Action::PutTrigger{..}|Action::FinishTargets))).unwrap().clone();
            checked(&mut g, action);
        }
        let t = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_copy_token())
            .map(|(i, _)| i as CardId)
            .unwrap();
        let e = named(&mut g, p, ELEKTRA, Zone::Hand);
        unblocked(&mut g, p, t, CombatDefender::Player(p.other()));
        sneak(&mut g, p, e, t);
        settle(&mut g);
        assert!(g.on_battlefield(e));
        assert!(!g.on_battlefield(t));
        assert_ne!(g.state.objects[t as usize].zone, Zone::Hand);
    }
}
#[test]
fn msh_elektra_hellcat_sneak_is_unavailable_after_blockers_step() {
    for p in [Player::P0, Player::P1] {
        let (mut g, _h, e) = fresh_sneak(p);
        until(&mut g, |g| {
            g.state.step == Step::EndCombat && g.state.pending == Pending::Priority
        });
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceSneak {
                card: e,
                permission: None,
            },
        );
    }
}
