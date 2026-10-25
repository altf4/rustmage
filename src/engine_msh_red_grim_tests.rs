//! Paid complete Red Hulk / Grim Reaper scenarios; explicit production boundary probes are labelled.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{checked, named, setup};
use super::*;
const RED: &str = "Red Hulk";
const GRIM: &str = "Grim Reaper, Lethal Legionnaire";
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    let bytes = g.state_bytes();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.state_bytes(), bytes);
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut observation = g.observe(p);
        // Restoring deliberately advances the branch revision.
        observation.revision = 0;
        serde_json::to_value(observation).unwrap()
    })
}
fn orphan(g: Game, action: Action) -> Game {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    let before_views = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    let after = raw.snapshot();
    assert_eq!(indexed.snapshot(), after);
    assert_eq!(views(&raw), views(&indexed));
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn finish_packet(g: &mut Game, packet: u64) {
    for _ in 0..20 {
        if g.state
            .announcing
            .as_ref()
            .is_none_or(|a| a.item.id != packet)
        {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseSpellCost { .. }
                )
            })
            .unwrap_or_else(|| panic!("packet {packet} stalled at {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("packet {packet} did not finish");
}
// Never guess modal choices or choose optional/multi targets. Only later Daredevil
// attacks are declined by default. Every nontrivial packet is stopped explicitly.
fn drive(g: &mut Game, stop: impl Fn(&Game) -> bool) -> usize {
    let mut declined = 0;
    g.prepare_triggers();
    for _ in 0..600 {
        if stop(g) {
            return declined;
        }
        assert!(g.outcome().is_none(), "unexpected terminal state");
        let action = if matches!(g.state.pending, Pending::EffectOptional { .. }) {
            declined += 1;
            Action::ChooseOptional { accept: false }
        } else if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
            Action::DeclineResolutionMana
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishAttackers
                            | Action::FinishBlockers
                    ) || matches!(a, Action::FinishTargets)
                        && g.state.announcing.as_ref().is_some_and(|a| a.spec.max == 0)
                })
                .unwrap_or_else(|| {
                    panic!(
                        "unhandled combined decision {:?} at {:?}",
                        g.state.pending, g.state.step
                    )
                })
        };
        checked(g, action);
    }
    panic!("combined progression stalled");
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn settle(g: &mut Game) {
    drive(g, quiet);
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn next_own_main(g: &mut Game, p: Player) {
    let turn = g.state.turn;
    drive(g, |g| {
        g.state.turn > turn
            && g.state.active == p
            && g.state.step == Step::Main1
            && quiet(g)
            && g.actor() == Some(p)
    });
}
fn trigger(g: &Game, id: CardId, ability: u16) -> bool {
    g.state.announcing.as_ref().is_some_and(|a| {
        a.item.card == id && a.item.kind == StackKind::TriggeredAbility { ability }
    })
}

fn cast(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, generic) = g.object_view(id).card.cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    checked(g, Action::AnnounceCast { card: id });
    let packet = g.state.announcing.as_ref().unwrap().item.id;
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish_packet(g, packet);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = cast(g, p, name, &[]);
    settle(g);
    id
}
fn fresh(p: Player) -> Game {
    let mut g = setup(p);
    for pl in &mut g.state.players {
        pl.mana = [0; 6];
    }
    g
}
fn select(g: &mut Game, target: Target) {
    let packet = g.state.announcing.as_ref().unwrap().item.id;
    checked(g, Action::ChooseTarget { target });
    finish_packet(g, packet);
}
fn red_child(g: &mut Game, p: Player, r: CardId) {
    cast(g, p, "Shock", &[Target::Card(r)]);
    drive(g, |g| trigger(g, r, 1));
}
fn hit(g: &mut Game, dealer: CardId, target: CardId, amount: i128) {
    let source = g.damage_source(dealer);
    g.apply_damage_batch(&[DamageEvent {
        source,
        target: Target::Card(target),
        amount,
        combat: false,
        preventable: true,
    }]);
}
fn aura(g: &mut Game, p: Player, name: &str, target: CardId) -> CardId {
    let id = named(g, p, name, Zone::Battlefield);
    g.state.objects[id as usize].attached_to = Some(g.object_ref(target));
    id
}
fn attack(g: &mut Game, p: Player, id: CardId) {
    next_own_main(g, p);
    drive(g, |g| {
        g.state.active == p && g.state.pending == Pending::Attackers
    });
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
}
fn grim_pay(g: &mut Game, p: Player, id: CardId) {
    drive(g, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
    assert_eq!(g.actor(), Some(p));
    g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
    checked(
        g,
        Action::PayResolutionMana {
            payment: [0, 0, 1, 0, 0, 3],
        },
    );
    drive(g, |g| trigger(g, id, 1));
}
#[test]
fn msh_red_grim_red_paid_body_counter_then_any_other_player_and_current_quantity() {
    for p in [Player::P0, Player::P1] {
        for victim in [p, p.other()] {
            let mut g = fresh(p);
            let r = paid(&mut g, p, RED);
            assert_eq!((g.stats(r).power, g.stats(r).toughness), (6, 7));
            assert!(
                g.keywords(r)
                    .contains(Keywords::REACH.union(Keywords::TRAMPLE))
            );
            red_child(&mut g, p, r);
            assert_eq!(g.counter_count(r, CounterKind::PlusOne), 1);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(r),
                },
            );
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            g = orphan(
                g,
                Action::ChooseTarget {
                    target: Target::Player(victim),
                },
            );
            g.add_counter(r, CounterKind::PlusOne, 2);
            settle(&mut g);
            assert_eq!(g.life(victim), 17);
        }
    }
}
#[test]
fn msh_red_grim_red_latest_departure_counter_lki_ignores_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, RED);
        red_child(&mut g, p, r);
        select(&mut g, Target::Player(p.other()));
        g.add_counter(r, CounterKind::PlusOne, 3);
        g.move_object(r, Zone::Exile);
        g.move_object(r, Zone::Battlefield);
        g.set_counters(r, Counters::of(CounterKind::PlusOne, 20));
        settle(&mut g);
        assert_eq!(g.life(p.other()), 16);
    }
}
#[test]
fn msh_red_grim_red_dynamic_removed_counters_and_noncreature_source() {
    for p in [Player::P0, Player::P1] {
        for moon in [false, true] {
            let mut g = fresh(p);
            let r = paid(&mut g, p, RED);
            red_child(&mut g, p, r);
            select(&mut g, Target::Player(p.other()));
            if moon {
                aura(&mut g, p, "Imprisoned in the Moon", r);
            } else {
                g.set_counters(r, Counters::default());
            }
            settle(&mut g);
            assert_eq!(g.life(p.other()), if moon { 19 } else { 20 });
        }
    }
}
#[test]
fn msh_red_grim_red_lethal_or_departed_parent_cannot_place_or_reflex() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = fresh(p);
            let r = paid(&mut g, p, RED);
            let dealer = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            hit(&mut g, dealer, r, if mode == 0 { 20 } else { 1 });
            if mode > 0 {
                g.move_object(r, Zone::Exile);
                if mode == 2 {
                    g.move_object(r, Zone::Battlefield);
                }
            }
            settle(&mut g);
            assert_eq!(g.counter_count(r, CounterKind::PlusOne), 0);
            assert_eq!(g.life(p.other()), 20);
        }
    }
}
#[test]
fn msh_red_grim_red_simultaneous_dealers_one_occurrence_other_victim_ignored() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, RED);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let x = g.damage_source(a);
        let y = g.damage_source(b);
        g.apply_damage_batch(&[
            DamageEvent {
                source: x,
                target: Target::Card(r),
                amount: 1,
                combat: false,
                preventable: true,
            },
            DamageEvent {
                source: y,
                target: Target::Card(r),
                amount: 2,
                combat: false,
                preventable: true,
            },
            DamageEvent {
                source: x,
                target: Target::Card(b),
                amount: 1,
                combat: false,
                preventable: true,
            },
        ]);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|i| i.card == r)
                .count(),
            1
        );
        drive(&mut g, |g| trigger(g, r, 1));
        select(&mut g, Target::Player(p.other()));
        settle(&mut g);
        assert_eq!(g.counter_count(r, CounterKind::PlusOne), 1);
        assert_eq!(g.life(p.other()), 19);
    }
}
#[test]
fn msh_red_grim_red_actual_doubled_counter_one_child_and_replacement_orphan() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, RED);
        paid(&mut g, p, "Doubling Season");
        paid(&mut g, p, "Doubling Season");
        cast(&mut g, p, "Shock", &[Target::Card(r)]);
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        assert_eq!(g.counter_count(r, CounterKind::PlusOne), 0);
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .any(|i| i.card == r && i.kind == StackKind::TriggeredAbility { ability: 1 })
        );
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        g = orphan(g, a);
        drive(&mut g, |g| trigger(g, r, 1));
        let n = g.counter_count(r, CounterKind::PlusOne);
        assert!([3, 4].contains(&n));
        select(&mut g, Target::Player(p.other()));
        settle(&mut g);
        assert_eq!(g.life(p.other()), 20 - i128::from(n));
    }
}
#[test]
fn msh_red_grim_red_zero_action_never_infers_success_from_prior_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, RED);
        red_child(&mut g, p, r);
        let item = g.state.announcing.as_ref().unwrap().item.clone();
        select(&mut g, Target::Player(p.other()));
        settle(&mut g);
        let before = g.state.pending_triggers.len();
        g.put_source_counter_reflexive(&item, CounterKind::PlusOne, 0, 1);
        assert_eq!(g.state.pending_triggers.len(), before);
        let dealer = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        hit(&mut g, dealer, r, 0);
        assert_eq!(g.state.pending_triggers.len(), before);
    }
}
#[test]
fn msh_red_grim_red_counter_on_control_changed_source_attributed_original_actor() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, RED);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        hit(&mut g, a, r, 1);
        g.record_control(g.object_ref(r), p.other(), None);
        drive(&mut g, |g| trigger(g, r, 1));
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.counter_count(r, CounterKind::PlusOne), 1);
        select(&mut g, Target::Player(p));
        settle(&mut g);
        assert_eq!(g.life(p), 19);
    }
}
#[test]
fn msh_red_grim_red_suppression_before_damage_and_after_queue() {
    for p in [Player::P0, Player::P1] {
        for before in [true, false] {
            let mut g = fresh(p);
            let r = paid(&mut g, p, RED);
            let dealer = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            if before {
                aura(&mut g, p, "Imprisoned in the Moon", r);
            }
            hit(&mut g, dealer, r, 1);
            if !before {
                aura(&mut g, p, "Imprisoned in the Moon", r);
                drive(&mut g, |g| trigger(g, r, 1));
                select(&mut g, Target::Player(p.other()));
            }
            settle(&mut g);
            assert_eq!(g.counter_count(r, CounterKind::PlusOne), u64::from(!before));
            assert_eq!(g.life(p.other()), if before { 20 } else { 19 });
        }
    }
}
#[test]
fn msh_red_grim_red_noncombat_prevented_before_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, RED);
        paid(&mut g, p, "Crystal Barricade");
        let q = p.other();
        cast(&mut g, q, "Shock", &[Target::Card(r)]);
        settle(&mut g);
        assert_eq!(g.object_view(r).damage, 0);
        assert_eq!(g.counter_count(r, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_red_grim_red_child_target_departure_and_new_source_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, RED);
        red_child(&mut g, p, r);
        g.move_object(r, Zone::Exile);
        g.move_object(r, Zone::Battlefield);
        select(&mut g, Target::Card(r));
        drive(&mut g, |g| trigger(g, r, 1));
        assert_eq!(g.object_view(r).damage, 1);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        select(&mut g, Target::Card(bear));
        g.move_object(bear, Zone::Exile);
        g.move_object(bear, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(bear).damage, 0);
    }
}
#[test]
fn msh_red_grim_grim_paid_attack_decline_no_early_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        attack(&mut g, p, r);
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(dead),
            },
        );
        reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(dead).zone, Zone::Graveyard);
        assert_eq!((g.stats(r).power, g.stats(r).toughness), (3, 4));
    }
}
#[test]
fn msh_red_grim_grim_paid_return_tapped_attacking_finality_and_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        attack(&mut g, p, r);
        grim_pay(&mut g, p, r);
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(dead),
            },
        );
        settle(&mut g);
        let v = g.object_view(dead);
        assert_eq!(v.zone, Zone::Battlefield);
        assert!(v.tapped && v.summoning_sick);
        assert_eq!(g.counter_count(dead, CounterKind::Finality), 1);
        assert!(
            g.state
                .attackers
                .iter()
                .any(|a| a.card == dead && a.defender == Some(CombatDefender::Player(p.other())))
        );
        drive(&mut g, |g| g.state.step == Step::Main2 && quiet(g));
        assert_eq!(g.life(p.other()), 15);
        cast(&mut g, p, "Murder", &[Target::Card(dead)]);
        settle(&mut g);
        assert_eq!(g.object_view(dead).zone, Zone::Exile);
    }
}
#[test]
fn msh_red_grim_grim_defender_independent_and_truthful_public_owned_menu() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let pw = named(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.set_counters(pw, Counters::of(CounterKind::Loyalty, 4));
        attack(&mut g, p, r);
        grim_pay(&mut g, p, r);
        select(&mut g, Target::Card(dead));
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::HandDefender { .. })
        });
        for viewer in [p, p.other()] {
            let choice = g.observe(viewer).effect_choice.unwrap();
            assert_eq!(
                choice.kind,
                EffectChoiceKind::TargetAttackingReturn {
                    counter: CounterKind::Finality
                }
            );
            assert!(choice.defender_choice);
            assert!(choice.candidates.is_empty());
        }
        reject(
            &mut g,
            Action::ChooseHandDefender {
                defender: CombatDefender::Player(p),
            },
        );
        let defender = CombatDefender::Planeswalker(g.object_ref(pw));
        g = orphan(g, Action::ChooseHandDefender { defender });
        settle(&mut g);
        assert!(
            g.state
                .attackers
                .iter()
                .any(|a| a.card == r && a.defender == Some(CombatDefender::Player(p.other())))
        );
        assert!(
            g.state
                .attackers
                .iter()
                .any(|a| a.card == dead && a.defender == Some(defender))
        );
    }
}
#[test]
fn msh_red_grim_grim_child_source_departure_or_control_does_not_change_return_actor() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = fresh(p);
            let r = paid(&mut g, p, GRIM);
            let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            attack(&mut g, p, r);
            grim_pay(&mut g, p, r);
            if blink {
                g.move_object(r, Zone::Exile);
                g.move_object(r, Zone::Battlefield);
            } else {
                g.record_control(g.object_ref(r), p.other(), None);
            }
            assert_eq!(g.actor(), Some(p));
            select(&mut g, Target::Card(dead));
            settle(&mut g);
            assert_eq!(g.object_view(dead).controller, p);
            assert!(g.state.attackers.iter().any(|a| a.card == dead));
        }
    }
}
#[test]
fn msh_red_grim_grim_target_legality_stale_and_noncreature() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let instant = named(&mut g, p, "Shock", Zone::Graveyard);
        attack(&mut g, p, r);
        grim_pay(&mut g, p, r);
        for id in [r, other, instant] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        select(&mut g, Target::Card(dead));
        g.move_object(dead, Zone::Exile);
        g.move_object(dead, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(dead).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_red_grim_grim_finality_at_entry_doubled_and_no_declared_attack_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        paid(&mut g, p, "Doubling Season");
        let dead = named(&mut g, p, GRIM, Zone::Graveyard);
        attack(&mut g, p, r);
        grim_pay(&mut g, p, r);
        select(&mut g, Target::Card(dead));
        drive(&mut g, |g| g.on_battlefield(dead));
        assert_eq!(g.counter_count(dead, CounterKind::Finality), 2);
        assert!(!g.state.pending_triggers.iter().any(|i| i.card == dead));
        let keep = g.object_ref(dead);
        checked(&mut g, Action::KeepLegend { object: keep });
        settle(&mut g);
        assert_eq!(g.object_view(r).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_red_grim_grim_outside_combat_child_returns_without_attack() {
    for p in [Player::P0, Player::P1] {
        for active in [p, p.other()] {
            let mut g = fresh(p);
            let r = paid(&mut g, p, GRIM);
            let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            attack(&mut g, p, r);
            grim_pay(&mut g, p, r);
            select(&mut g, Target::Card(dead)); // Deliberate resolution-boundary copy/late-combat fixture.
            g.state.step = Step::Main2;
            g.state.active = active;
            g.state.attackers.clear();
            settle(&mut g);
            assert!(g.on_battlefield(dead));
            assert!(g.object_view(dead).tapped);
            assert_eq!(g.counter_count(dead, CounterKind::Finality), 1);
            assert!(!g.state.attackers.iter().any(|a| a.card == dead));
        }
    }
}
#[test]
fn msh_red_grim_grim_finality_removed_bounce_reentry_and_ability_loss() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = fresh(p);
            let r = paid(&mut g, p, GRIM);
            let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            attack(&mut g, p, r);
            grim_pay(&mut g, p, r);
            select(&mut g, Target::Card(dead));
            settle(&mut g);
            if mode == 0 {
                g.set_counters(dead, Counters::default());
            } else if mode == 1 {
                g.move_object(dead, Zone::Hand);
                g.move_object(dead, Zone::Battlefield);
            } else {
                aura(&mut g, p, "Witness Protection", dead);
            }
            g.move_object(dead, Zone::Graveyard);
            assert_eq!(
                g.object_view(dead).zone,
                if mode == 2 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn msh_red_grim_full_both_views_hidden_counterfactual_and_owned_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let secret = named(&mut g, p.other(), "Shock", Zone::Hand);
        attack(&mut g, p, r);
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        let mut hidden = g.clone();
        hidden.state.objects[secret as usize].card = Card::from_name("Giant Growth").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(hidden.observe(p)).unwrap()
        );
        assert_eq!(g.legal_actions(), hidden.legal_actions());
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
        g = orphan(
            g,
            Action::PayResolutionMana {
                payment: [0, 0, 1, 0, 0, 3],
            },
        );
        drive(&mut g, |g| trigger(g, r, 1));
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(dead),
            },
        );
        settle(&mut g);
        assert!(g.on_battlefield(dead));
    }
}

#[test]
fn msh_red_grim_red_real_reach_block_and_trample_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, RED);
        next_own_main(&mut g, p.other());
        let bird = paid(&mut g, p.other(), "Healer's Hawk");
        next_own_main(&mut g, p);
        drive(&mut g, |g| {
            g.state.active == p.other() && g.state.pending == Pending::Attackers
        });
        checked(&mut g, Action::Attack { card: bird });
        checked(&mut g, Action::FinishAttackers);
        drive(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: r,
                attacker: bird,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        drive(&mut g, |g| trigger(g, r, 1));
        select(&mut g, Target::Player(p.other()));
        drive(&mut g, |g| g.state.step == Step::Main2 && quiet(g));
        assert_eq!(g.object_view(bird).zone, Zone::Graveyard);
        let before = g.life(p.other());
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attack(&mut g, p, r);
        drive(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: r,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        drive(&mut g, |g| {
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::AssignDamage { .. }))
        });
        checked(
            &mut g,
            Action::AssignDamage {
                target: Target::Card(bear),
                amount: 2,
            },
        );
        drive(&mut g, |g| trigger(g, r, 1));
        select(&mut g, Target::Player(p.other()));
        drive(&mut g, |g| g.state.step == Step::Main2 && quiet(g));
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.life(p.other()), before - 7);
    }
}
#[test]
fn msh_red_grim_red_child_current_controller_lifelink_and_departure_lki() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = fresh(p);
            let r = paid(&mut g, p, RED);
            red_child(&mut g, p, r);
            select(&mut g, Target::Player(p));
            g.grant_keyword_effects(&[r], g.object_ref(r), Keywords::LIFELINK);
            g.record_control(g.object_ref(r), p.other(), None);
            if leave {
                g.move_object(r, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(g.life(p), 19);
            assert_eq!(g.life(p.other()), 21);
        }
    }
}
#[test]
fn msh_red_grim_red_parent_and_child_packet_copies_keep_independent_success() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, RED);
        let dealer = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        hit(&mut g, dealer, r, 1);
        let mut copy = g
            .state
            .pending_triggers
            .iter()
            .find(|i| i.card == r)
            .unwrap()
            .clone();
        copy.id = g.next_stack_id();
        g.state.pending_triggers.push(copy);
        for i in 0..2 {
            drive(&mut g, |g| trigger(g, r, 1));
            select(&mut g, Target::Player(p.other()));
            if i == 1 {
                settle(&mut g);
            }
        }
        assert_eq!(g.counter_count(r, CounterKind::PlusOne), 2);
        red_child(&mut g, p, r);
        select(&mut g, Target::Player(p.other()));
        let mut copy = g.state.stack.last().unwrap().clone();
        copy.id = g.next_stack_id();
        g.state.stack.push(copy);
        let before = g.life(p.other());
        settle(&mut g);
        assert_eq!(g.life(p.other()), before - 6);
        assert_eq!(g.counter_count(r, CounterKind::PlusOne), 3);
    }
}
#[test]
fn msh_red_grim_grim_source_gone_before_actual_payment_still_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        attack(&mut g, p, r);
        g.move_object(r, Zone::Exile);
        grim_pay(&mut g, p, r);
        select(&mut g, Target::Card(dead));
        settle(&mut g);
        assert!(g.on_battlefield(dead));
        assert_eq!(g.object_view(dead).controller, p);
    }
}
#[test]
fn msh_red_grim_grim_pending_entry_color_choice_owns_finality_and_attacking() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        let dead = named(&mut g, p, "Diamond Mare", Zone::Graveyard);
        attack(&mut g, p, r);
        grim_pay(&mut g, p, r);
        select(&mut g, Target::Card(dead));
        drive(&mut g, |g| {
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseEntryValue { .. }))
        });
        assert_eq!(g.object_view(dead).zone, Zone::Graveyard);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
            .unwrap();
        g = orphan(g, a);
        settle(&mut g);
        assert_eq!(g.counter_count(dead, CounterKind::Finality), 1);
        assert!(g.object_view(dead).tapped);
        assert!(g.state.attackers.iter().any(|a| a.card == dead));
    }
}
#[test]
fn msh_red_grim_grim_defender_disappears_during_entry_choice_no_invalid_attacker() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        let dead = named(&mut g, p, "Diamond Mare", Zone::Graveyard);
        let pw = named(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.set_counters(pw, Counters::of(CounterKind::Loyalty, 4));
        attack(&mut g, p, r);
        grim_pay(&mut g, p, r);
        select(&mut g, Target::Card(dead));
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::HandDefender { .. })
        });
        let defender = CombatDefender::Planeswalker(g.object_ref(pw));
        checked(&mut g, Action::ChooseHandDefender { defender });
        drive(&mut g, |g| {
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseEntryValue { .. }))
        }); // Production boundary probe: the selected exact defender is no longer on the battlefield.
        g.move_object(pw, Zone::Exile);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
            .unwrap();
        checked(&mut g, a);
        settle(&mut g);
        assert!(g.on_battlefield(dead));
        assert!(!g.state.attackers.iter().any(|a| a.card == dead));
    }
}
#[test]
fn msh_red_grim_grim_stale_defender_choice_rejects_exact_old_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let pw = named(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.set_counters(pw, Counters::of(CounterKind::Loyalty, 4));
        attack(&mut g, p, r);
        grim_pay(&mut g, p, r);
        select(&mut g, Target::Card(dead));
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::HandDefender { .. })
        });
        let old = g.object_ref(pw);
        g.move_object(pw, Zone::Exile);
        g.move_object(pw, Zone::Battlefield);
        reject(
            &mut g,
            Action::ChooseHandDefender {
                defender: CombatDefender::Planeswalker(old),
            },
        );
        checked(
            &mut g,
            Action::ChooseHandDefender {
                defender: CombatDefender::Player(p.other()),
            },
        );
        settle(&mut g);
        assert!(g.on_battlefield(dead));
    }
}
#[test]
fn msh_red_grim_grim_finality_entry_replacement_choice_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        paid(&mut g, p, "Doubling Season");
        paid(&mut g, p, "Doubling Season");
        let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        attack(&mut g, p, r);
        grim_pay(&mut g, p, r);
        select(&mut g, Target::Card(dead));
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        assert!(!g.on_battlefield(dead));
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        g = orphan(g, a);
        settle(&mut g);
        assert_eq!(g.counter_count(dead, CounterKind::Finality), 4);
    }
}

#[test]
fn msh_red_grim_grim_real_mana_abilities_pay_optional_cost_before_later_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let swamp = named(&mut g, p, "Swamp", Zone::Battlefield);
        let wastes: [CardId; 3] =
            std::array::from_fn(|_| named(&mut g, p, "Wastes", Zone::Battlefield));
        attack(&mut g, p, r);
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        for card in std::iter::once(swamp).chain(wastes) {
            let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::TapMana { card: id } | Action::ActivateMana { card: id, .. } if *id == card)).expect("actual basic mana action");
            checked(&mut g, action);
        }
        g = orphan(
            g,
            Action::PayResolutionMana {
                payment: [0, 0, 1, 0, 0, 3],
            },
        );
        drive(&mut g, |g| trigger(g, r, 1));
        select(&mut g, Target::Card(dead));
        settle(&mut g);
        assert_eq!(g.counter_count(dead, CounterKind::Finality), 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_red_grim_red_successful_counter_cancellation_still_announces_zero_damage_child() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, RED);
        g.set_counters(r, Counters::of(CounterKind::MinusOne, 1));
        red_child(&mut g, p, r);
        assert_eq!(g.counter_count(r, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(r, CounterKind::MinusOne), 0);
        select(&mut g, Target::Player(p.other()));
        settle(&mut g);
        assert_eq!(g.life(p.other()), 20);
    }
}
#[test]
fn msh_red_grim_red_any_other_planeswalker_and_opponent_hexproof_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, RED);
        let pw = named(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.set_counters(pw, Counters::of(CounterKind::Loyalty, 4));
        red_child(&mut g, p, r);
        select(&mut g, Target::Card(pw));
        settle(&mut g);
        assert_eq!(g.counter_count(pw, CounterKind::Loyalty), 3);
        let wall = named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
        red_child(&mut g, p, r);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        select(&mut g, Target::Player(p));
        settle(&mut g);
        assert_eq!(g.life(p), 18);
        assert!(g.on_battlefield(wall));
    }
}
#[test]
fn msh_red_grim_grim_copied_child_keeps_exact_target_and_fizzles_after_first_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let r = paid(&mut g, p, GRIM);
        let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        attack(&mut g, p, r);
        grim_pay(&mut g, p, r);
        select(&mut g, Target::Card(dead)); // Explicit independent copied ability packet, no new copy producer admission.
        let mut copy = g.state.stack.last().unwrap().clone();
        copy.id = g.next_stack_id();
        g.state.stack.push(copy);
        settle(&mut g);
        assert_eq!(g.counter_count(dead, CounterKind::Finality), 1);
        assert_eq!(
            g.state.attackers.iter().filter(|a| a.card == dead).count(),
            1
        );
        assert_eq!(g.object_view(dead).controller, p);
    }
}
