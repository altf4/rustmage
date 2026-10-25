//! Ordinary supported paid producer scenarios; primitive seams are labeled.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const BRUCE: &str = "Bruce Banner";
const HULK: &str = "The Incredible Hulk";
const KANG: &str = "Kang the Conqueror";
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn hand(g: &mut Game, p: Player, name: &str) -> CardId {
    assert!(Card::from_name(name).unwrap().is_supported());
    named(g, p, name, Zone::Hand)
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
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseActivationCost { .. }
                )
            })
            .expect("payment");
        checked(g, a);
    }
    panic!("unpaid");
}
fn packet(g: &mut Game, p: Player, name: &str, back: bool, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = hand(g, p, name);
    let c = if back {
        g.state.objects[id as usize].card.other_face().unwrap()
    } else {
        g.state.objects[id as usize].card
    };
    let (mut mana, generic) = c.cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    if back {
        checked(g, Action::AnnounceBack { card: id });
        pay(g);
    } else {
        cast(g, id, None, targets);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, back: bool, targets: &[Target]) -> CardId {
    let id = packet(g, p, name, back, targets);
    settle(g);
    id
}
fn announce(id: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability,
        counter_payment: Counters::default(),
    }
}
fn activate(g: &mut Game, p: Player, id: CardId, ability: u16, mana: Mana) {
    priority(g, p);
    g.state.players[p.index()].mana = mana;
    checked(g, announce(id, ability));
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn kang(g: &mut Game, p: Player) -> CardId {
    paid(g, p, KANG, false, &[])
}
fn power(g: &mut Game, p: Player, k: CardId, entry: bool) {
    activate(
        g,
        p,
        k,
        0,
        if entry {
            [0, 1, 0, 0, 0, 3]
        } else {
            [0, 3, 0, 0, 0, 5]
        },
    );
    settle(g);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..180 {
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
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::Discard { .. }
                        | Action::ChooseTarget {
                            target: Target::Player(_)
                        }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn main(g: &mut Game) {
    until(g, |g| {
        g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
    settle(g);
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap())
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let v = views(g);
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(views(g), v);
    assert_eq!(g.revision(), rev);
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let before = g.snapshot();
    let seen = views(&g);
    let ix = g.legal_actions().iter().position(|x| *x == a).unwrap();
    let (mut raw, mut idx) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    idx.apply_index(actor, rev, ix).unwrap();
    let after = raw.snapshot();
    assert_eq!(after, idx.snapshot());
    assert_eq!(views(&raw), views(&idx));
    raw.restore(&before).unwrap();
    let mut expect = seen;
    for v in &mut expect {
        v["revision"] = serde_json::json!(raw.revision());
    }
    assert_eq!(views(&raw), expect);
    assert_eq!(raw.apply_index(actor, rev, ix), Err(Error::StaleRevision));
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    idx
}
#[test]
fn msh_bruce_kang_modal_paid_faces_costs_and_intrinsic_obligations() {
    for p in [Player::P0, Player::P1] {
        for back in [false, true] {
            let mut g = setup(p);
            let b = paid(&mut g, p, BRUCE, back, &[]);
            let v = g.object_view(b);
            assert_eq!(v.name(), if back { HULK } else { BRUCE });
            assert_eq!((v.power, v.toughness), if back { (8, 8) } else { (1, 1) });
            assert_eq!(v.card.fixed_mana_value(), if back { 6 } else { 1 });
            assert!(v.is_legendary());
            assert!(v.has_subtype("Hero"));
            if back {
                assert!(
                    v.keywords
                        .contains(Keywords::REACH.union(Keywords::TRAMPLE))
                );
                assert!(v.has_subtype("Gamma") && v.has_subtype("Berserker"));
            } else {
                assert!(v.has_subtype("Human") && v.has_subtype("Scientist"));
            }
            g.move_object(b, Zone::Hand);
            assert_eq!(g.object_view(b).name(), BRUCE);
        }
    }
}
#[test]
fn msh_bruce_kang_bruce_paid_xx_draws_one_x_at_zero_small_and_upper_domain() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 1, 2, 4] {
            let mut g = setup(p);
            let b = paid(&mut g, p, BRUCE, false, &[]);
            g.state.objects[b as usize].summoning_sick = false;
            priority(&mut g, p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 8];
            let before = g.hand_count(p);
            checked(&mut g, announce(b, 0));
            let domain = g.x_choice().unwrap();
            assert_eq!(
                domain.intervals,
                vec![XInterval {
                    minimum: 0,
                    maximum: 4
                }]
            );
            reject(&mut g, Action::ChooseX { value: 5 });
            reject(&mut g, Action::ChooseX { value: u32::MAX });
            g = orphan(g, Action::ChooseX { value: x });
            let req = g.announcement_payment_request(p).unwrap();
            assert_eq!(req.generic, 2 * x);
            pay(&mut g);
            assert!(g.object_view(b).tapped);
            assert_eq!(g.state.players[p.index()].mana[5], 8 - 2 * x);
            settle(&mut g);
            assert_eq!(g.hand_count(p), before + x as usize);
        }
    }
}
#[test]
fn msh_bruce_kang_bruce_sorcery_tap_sickness_odd_budget_and_ordinary_scalar_funding() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, BRUCE, false, &[]);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        reject(&mut g, announce(b, 0));
        g.state.objects[b as usize].summoning_sick = false;
        g.state.step = Step::BeginCombat;
        reject(&mut g, announce(b, 0));
        g.state.step = Step::Main1;
        checked(&mut g, announce(b, 0));
        assert_eq!(g.x_choice().unwrap().intervals[0].maximum, 2);
        checked(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        settle(&mut g);
        priority(&mut g, p);
        reject(&mut g, announce(b, 0));
    }
}
#[test]
fn msh_bruce_kang_bruce_paid_xx_can_float_lands_and_owned_cost_payment_is_exact() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, BRUCE, false, &[]);
        g.state.objects[b as usize].summoning_sick = false;
        let lands = (0..4)
            .map(|_| named(&mut g, p, "Forest", Zone::Battlefield))
            .collect::<Vec<_>>();
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, announce(b, 0));
        assert_eq!(g.x_choice().unwrap().intervals[0].maximum, 2);
        checked(&mut g, Action::ChooseX { value: 2 });
        for l in lands {
            checked(&mut g, Action::TapMana { card: l });
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        g = orphan(g, a);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert!(g.object_view(b).tapped);
    }
}
#[test]
fn msh_bruce_kang_bruce_transform_paid_sorcery_serial_source_incarnation_and_counters() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let b = paid(&mut g, p, BRUCE, false, &[]);
            let old = g.object_ref(b);
            g.add_counter(b, CounterKind::PlusOne, 2);
            activate(&mut g, p, b, 1, [0, 0, 0, 2, 2, 2]);
            if mode == 1 {
                g.move_object(b, Zone::Hand);
                g.move_object(b, Zone::Battlefield);
            } else if mode == 2 {
                // Bounded duplicate of a genuinely paid packet tests the existing transform serial guard.
                let mut copy = g.state.stack.last().unwrap().clone();
                copy.id = g.next_stack_id();
                g.state.stack.push(copy);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(b).name(),
                if mode == 1 { BRUCE } else { HULK }
            );
            if mode != 1 {
                assert_eq!(g.object_ref(b), old);
                assert_eq!(
                    g.stats(b),
                    Stats {
                        power: 10,
                        toughness: 10
                    }
                );
                assert_eq!(g.object_view(b).face.unwrap().transforms, 1);
            }
        }
    }
}
#[test]
fn msh_bruce_kang_hulk_paid_noncombat_damage_counter_only_and_prevention_no_trigger() {
    for p in [Player::P0, Player::P1] {
        for prevent in [false, true] {
            let mut g = setup(p);
            let h = paid(&mut g, p, BRUCE, true, &[]);
            if prevent {
                // Explicit public keyword-boundary fixture, not a new card admission.
                g.state.objects[h as usize].gained_keywords = Keywords::PROTECTION_EVERYTHING;
            }
            if prevent {
                let before = g.counter_count(h, CounterKind::PlusOne);
                let ds = g.damage_source(h);
                g.queue_hit(ds, Target::Card(h), 2, false);
                let hits = std::mem::take(&mut g.state.hits);
                g.apply_damage_batch(&hits);
                g.state_based_actions();
                settle(&mut g);
                assert_eq!(g.counter_count(h, CounterKind::PlusOne), before);
            } else {
                paid(&mut g, p, "Shock", false, &[Target::Card(h)]);
                assert_eq!(g.counter_count(h, CounterKind::PlusOne), 1);
                assert_eq!(
                    g.stats(h),
                    Stats {
                        power: 9,
                        toughness: 9
                    }
                );
            }
            assert_eq!(
                g.state
                    .combat_rules
                    .as_ref()
                    .map_or(0, |r| r.additional_combats),
                0
            );
        }
    }
}
fn attack(g: &mut Game, p: Player, h: CardId) {
    g.state.objects[h as usize].summoning_sick = false;
    g.begin_step(Step::BeginCombat);
    until(g, |g| g.state.pending == Pending::Attackers);
    checked(g, Action::Attack { card: h });
    checked(g, Action::FinishAttackers);
    assert!(g.object_view(h).tapped);
    assert_eq!(g.state.active, p);
}
#[test]
fn msh_bruce_kang_hulk_paid_attacking_damage_untaps_and_actual_extra_combat_is_taken() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, BRUCE, true, &[]);
        attack(&mut g, p, h);
        paid(&mut g, p, "Shock", false, &[Target::Card(h)]);
        assert_eq!(g.counter_count(h, CounterKind::PlusOne), 1);
        assert!(!g.object_view(h).tapped);
        assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 1);
        until(&mut g, |g| g.state.step == Step::EndCombat);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: h });
        checked(&mut g, Action::FinishAttackers);
        assert!(g.object_view(h).tapped);
        assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 0);
    }
}
#[test]
fn msh_bruce_kang_hulk_paid_attacking_stun_prevents_untap_but_not_extra_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, BRUCE, true, &[]);
        attack(&mut g, p, h);
        g.add_counter(h, CounterKind::Stun, 1);
        paid(&mut g, p, "Shock", false, &[Target::Card(h)]);
        assert!(g.object_view(h).tapped);
        assert_eq!(g.counter_count(h, CounterKind::Stun), 0);
        assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 1);
    }
}
#[test]
fn msh_bruce_kang_hulk_attacking_source_departure_before_enrage_resolves_no_extra() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, BRUCE, true, &[]);
        attack(&mut g, p, h);
        packet(&mut g, p, "Shock", false, &[Target::Card(h)]);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|i| i.card == h && matches!(i.kind, StackKind::TriggeredAbility { .. }))
        });
        packet(&mut g, p, "Unsummon", false, &[Target::Card(h)]);
        settle(&mut g);
        assert_eq!(g.object_view(h).zone, Zone::Hand);
        assert_eq!(g.object_view(h).name(), BRUCE);
        assert_eq!(
            g.state
                .combat_rules
                .as_ref()
                .map_or(0, |r| r.additional_combats),
            0
        );
    }
}
#[test]
fn msh_bruce_kang_kang_paid_entry_reduction_once_flying_and_public_exact_queue() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = kang(&mut g, p);
        assert_eq!(
            g.stats(k),
            Stats {
                power: 4,
                toughness: 5
            }
        );
        assert!(g.keywords(k).contains(Keywords::FLYING));
        power(&mut g, p, k, true);
        assert_eq!(
            g.stats(k),
            Stats {
                power: 5,
                toughness: 6
            }
        );
        let q = g.observe(p).turn_schedule.unwrap();
        assert_eq!(
            q.pending,
            vec![ExtraTurnView {
                player: p,
                power_up_prohibited: true
            }]
        );
        assert!(q.current.is_none());
        assert_eq!(q.normal_successor, p.other());
        assert_eq!(
            serde_json::to_value(g.observe(p).turn_schedule).unwrap(),
            serde_json::to_value(g.observe(p.other()).turn_schedule).unwrap()
        );
        priority(&mut g, p);
        g.state.players[p.index()].mana = [30; 6];
        reject(&mut g, announce(k, 0));
    }
}
#[test]
fn msh_bruce_kang_kang_paid_later_full_price_and_extra_then_original_normal_successor() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = kang(&mut g, p);
        next_turn(&mut g);
        next_turn(&mut g);
        main(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
        reject(&mut g, announce(k, 0));
        power(&mut g, p, k, false);
        let turn = g.state.turn;
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        assert_eq!(g.state.turn, turn + 1);
        assert!(g.power_up_prohibited_this_turn());
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.state.turn, turn + 2);
        assert!(g.observe(p).turn_schedule.is_none());
        assert!(!g.power_up_prohibited_this_turn());
    }
}
#[test]
fn msh_bruce_kang_kang_source_leaves_control_changes_before_resolution_turn_stays_captured() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = setup(p);
            let k = kang(&mut g, p);
            activate(&mut g, p, k, 0, [0, 1, 0, 0, 0, 3]);
            if leave {
                packet(&mut g, p, "Unsummon", false, &[Target::Card(k)]);
            } else {
                g.change_controller(g.object_ref(k), p.other());
            }
            settle(&mut g);
            assert_eq!(g.observe(p).turn_schedule.unwrap().pending[0].player, p);
            next_turn(&mut g);
            assert_eq!(g.state.active, p);
            assert!(g.power_up_prohibited_this_turn());
        }
    }
}
#[test]
fn msh_bruce_kang_designated_extra_turn_blocks_all_players_powerups_and_allows_ordinary_abilities()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = kang(&mut g, p);
        let own = named(&mut g, p, "Human Torch, Johnny Storm", Zone::Battlefield);
        let enemy = named(
            &mut g,
            p.other(),
            "Human Torch, Johnny Storm",
            Zone::Battlefield,
        );
        let b = paid(&mut g, p, BRUCE, false, &[]);
        power(&mut g, p, k, true);
        next_turn(&mut g);
        main(&mut g);
        assert!(g.power_up_prohibited_this_turn());
        for (owner, t) in [(p, own), (p.other(), enemy)] {
            let mut branch = g.clone();
            priority(&mut branch, owner);
            branch.state.players[owner.index()].mana = [30; 6];
            reject(&mut branch, announce(t, 0));
        }
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, announce(b, 0));
        checked(&mut g, Action::ChooseX { value: 0 });
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(b).tapped);
    }
}
#[test]
fn msh_bruce_kang_two_paid_controllers_lifo_extra_turns_resume_original_normal_schedule() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = kang(&mut g, p);
        next_turn(&mut g);
        main(&mut g);
        let second = kang(&mut g, p.other());
        power(&mut g, p, first, false);
        power(&mut g, p.other(), second, true);
        assert_eq!(g.state.active, p.other());
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        assert!(g.observe(p).turn_schedule.is_none());
    }
}
#[test]
fn msh_bruce_kang_skipped_queued_turn_consumes_its_prohibition_without_turn_events() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = kang(&mut g, p);
        power(&mut g, p, k, true);
        let turn = g.state.turn; // Bounded preexisting skip-effect state; skips themselves are covered by the admitted coin-flip producer.
        g.state.turn_skips = Some(Box::new(if p == Player::P0 { [1, 0] } else { [0, 1] }));
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.state.turn, turn + 1);
        assert!(!g.power_up_prohibited_this_turn());
        assert!(g.observe(p).turn_schedule.is_none());
        assert!(g.state.turn_skips.is_none());
    }
}
#[test]
fn msh_bruce_kang_paid_time_stop_ends_exact_extra_turn_and_does_not_spill_ban() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = kang(&mut g, p);
        power(&mut g, p, k, true);
        next_turn(&mut g);
        main(&mut g);
        assert!(g.power_up_prohibited_this_turn());
        paid(&mut g, p, "Time Stop", false, &[]);
        assert_eq!(g.state.active, p.other());
        assert!(!g.power_up_prohibited_this_turn());
        assert!(g.observe(p).turn_schedule.is_none());
    }
}
#[test]
fn msh_bruce_kang_bounded_nested_extra_turn_and_queue_owned_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = kang(&mut g, p);
        power(&mut g, p, k, true);
        next_turn(&mut g);
        main(&mut g); // Bounded generic scheduler consumer: no additional card admitted.
        g.add_extra_turn(p.other(), false);
        g.add_extra_turn(p, true);
        let a = Action::Pass;
        g = orphan(g, a);
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        assert!(g.power_up_prohibited_this_turn());
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert!(!g.power_up_prohibited_this_turn());
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert!(g.observe(p).turn_schedule.is_none());
    }
}

#[test]
fn msh_bruce_kang_bruce_xx_restricted_artifact_spell_mana_cannot_pay_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, BRUCE, false, &[]);
        g.state.objects[b as usize].summoning_sick = false;
        let lands = (0..2)
            .map(|_| named(&mut g, p, "Castle Doom", Zone::Battlefield))
            .collect::<Vec<_>>();
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        for l in lands {
            checked(
                &mut g,
                Action::ActivateMana {
                    card: l,
                    ability: 1,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [0, 1, 0, 0, 0, 0],
                },
            );
        }
        checked(&mut g, announce(b, 0));
        assert_eq!(
            g.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: 0
            }]
        );
        reject(&mut g, Action::ChooseX { value: 1 });
        checked(&mut g, Action::ChooseX { value: 0 });
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 2, 0, 0, 0, 0]);
        settle(&mut g);
    }
}
#[test]
fn msh_bruce_kang_extra_turn_prohibition_checks_real_borrowed_power_up_template() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = kang(&mut g, p);
        power(&mut g, p, k, true);
        paid(&mut g, p, "I Am Iron Man", false, &[Target::Card(k)]);
        let forge = packet(&mut g, p, "Territory Forge", false, &[]);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == forge)
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(k),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(k).zone, Zone::Exile);
        let descriptor = g
            .borrowed_ability_views()
            .into_iter()
            .find(|v| v.source.card == forge && v.definition == Card::from_name(KANG).unwrap())
            .unwrap();
        assert!(g.power_up_available(
            descriptor.source,
            descriptor.definition,
            descriptor.ability,
            Some(descriptor.slot)
        ));
        next_turn(&mut g);
        main(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [30; 6];
        reject(
            &mut g,
            Action::AnnounceBorrowedAbility {
                card: forge,
                exiled: descriptor.exiled,
                ability: descriptor.ability,
                counter_payment: Counters::default(),
            },
        );
        next_turn(&mut g);
        main(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 3, 0, 0, 0, 5];
        checked(
            &mut g,
            Action::AnnounceBorrowedAbility {
                card: forge,
                exiled: descriptor.exiled,
                ability: descriptor.ability,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.observe(p).turn_schedule.unwrap().pending[0].player, p);
    }
}
#[test]
fn msh_bruce_kang_extra_turn_prohibition_checks_new_actual_copy_power_up_origin() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = kang(&mut g, p);
        let arc = paid(&mut g, p, "Arc Reactor", false, &[]);
        let shuri = paid(&mut g, p, "Shuri, Wakandan Inventor", false, &[]);
        power(&mut g, p, k, true);
        next_turn(&mut g);
        main(&mut g);
        paid(&mut g, p, "I Am Iron Man", false, &[Target::Card(k)]);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(&mut g, announce(shuri, 0));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(arc),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(k),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(arc).name(), KANG);
        assert!(!g.object_view(arc).is_legendary());
        assert!(g.copy_ability_origin(g.object_ref(arc)).is_some());
        priority(&mut g, p);
        g.state.players[p.index()].mana = [30; 6];
        reject(&mut g, announce(arc, 0));
        assert_eq!(
            g.power_up_count(g.object_ref(arc), Card::from_name(KANG).unwrap(), 0, None),
            0
        );
    }
}
#[test]
fn msh_bruce_kang_hulk_actual_reach_blocks_flying_and_enrage_does_not_add_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, BRUCE, true, &[]);
        let flyer = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        g.state.active = p.other();
        g.begin_step(Step::BeginCombat);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: flyer });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: h,
                attacker: flyer,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.step == Step::EndCombat);
        assert_eq!(g.counter_count(h, CounterKind::PlusOne), 1);
        assert_eq!(
            g.state
                .combat_rules
                .as_ref()
                .map_or(0, |r| r.additional_combats),
            0
        );
        assert_eq!(g.object_view(flyer).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_bruce_kang_hulk_actual_trample_damage_and_lethal_enrage_source_lookback() {
    for p in [Player::P0, Player::P1] {
        for lethal in [false, true] {
            let mut g = setup(p);
            let h = paid(&mut g, p, BRUCE, true, &[]);
            let b = named(
                &mut g,
                p.other(),
                if lethal {
                    "Thornweald Archer"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            let life = g.state.players[p.other().index()].life;
            attack(&mut g, p, h);
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            checked(
                &mut g,
                Action::Block {
                    blocker: b,
                    attacker: h,
                },
            );
            checked(&mut g, Action::FinishBlockers);
            until(&mut g, |g| g.state.pending == Pending::Damage);
            checked(
                &mut g,
                Action::AssignDamage {
                    target: Target::Card(b),
                    amount: if lethal { 1 } else { 2 },
                },
            );
            // The sole remaining defender receives the forced remainder.

            until(&mut g, |g| g.state.step == Step::EndCombat);
            assert_eq!(
                g.state.players[p.other().index()].life,
                life - if lethal { 7 } else { 6 }
            );
            if lethal {
                assert_eq!(g.object_view(h).zone, Zone::Graveyard);
                assert_eq!(
                    g.state
                        .combat_rules
                        .as_ref()
                        .map_or(0, |r| r.additional_combats),
                    0
                );
            } else {
                assert_eq!(g.counter_count(h, CounterKind::PlusOne), 1);
                assert!(!g.object_view(h).tapped);
                assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 1);
            }
        }
    }
}

#[test]
fn msh_bruce_kang_duplicate_paid_packet_extra_turns_skip_one_and_preserve_next_context() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = kang(&mut g, p);
        activate(&mut g, p, k, 0, [0, 1, 0, 0, 0, 3]); // Bounded ability-copy seam: clone a genuinely paid packet, not a new card.
        let mut copy = g.state.stack.last().unwrap().clone();
        copy.id = g.next_stack_id();
        g.state.stack.push(copy);
        settle(&mut g);
        assert_eq!(g.counter_count(k, CounterKind::PlusOne), 2);
        assert_eq!(g.observe(p).turn_schedule.unwrap().pending.len(), 2);
        g.state.turn_skips = Some(Box::new(if p == Player::P0 { [1, 0] } else { [0, 1] }));
        let turn = g.state.turn;
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        assert_eq!(g.state.turn, turn + 1);
        assert!(g.power_up_prohibited_this_turn());
        assert!(g.observe(p).turn_schedule.unwrap().pending.is_empty());
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert!(g.observe(p).turn_schedule.is_none());
    }
}
#[test]
fn msh_bruce_kang_multiple_hulk_enrage_combats_survive_later_source_destruction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, BRUCE, true, &[]);
        attack(&mut g, p, h);
        paid(&mut g, p, "Shock", false, &[Target::Card(h)]);
        paid(&mut g, p, "Shock", false, &[Target::Card(h)]);
        assert_eq!(g.counter_count(h, CounterKind::PlusOne), 2);
        assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 2);
        paid(&mut g, p, "Murder", false, &[Target::Card(h)]);
        assert_eq!(g.object_view(h).zone, Zone::Graveyard);
        assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 2);
        until(&mut g, |g| g.state.step == Step::EndCombat);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 1);
    }
}
