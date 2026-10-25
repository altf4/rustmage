//! Paid temporary activated instances; bounded copy transitions are labeled.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
const FIST: &str = "Iron Fist, Living Weapon";
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

fn age(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    until(g, |g| g.state.active == p && g.state.step == Step::Main1);
    priority(g, p);
}
fn arm(g: &mut Game, p: Player, f: CardId) -> u64 {
    let a = named(g, p, "Hill Giant", Zone::Battlefield);
    packet(g, p, "Shock", None, &[Target::Card(a)]);
    settle(g);
    let rows = g.state.self_granted_abilities.as_ref().unwrap();
    rows.iter()
        .rev()
        .find(|r| r.recipient == g.object_ref(f))
        .unwrap()
        .timestamp
}
fn action(g: &Game, f: CardId, t: u64) -> Action {
    Action::AnnounceSelfGrantedAbility {
        source: g.object_ref(f),
        grant_timestamp: t,
    }
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
fn punch(g: &mut Game, p: Player, f: CardId, t: u64, target: Target) -> SpellView {
    priority(g, p);
    g.state.players[p.index()].mana = [0; 6];
    checked(g, action(g, f, t));
    checked(g, Action::ChooseTarget { target });
    finish(g);
    g.state
        .stack
        .iter()
        .find(|i| i.card == f && matches!(i.kind, StackKind::ActivatedAbility { .. }))
        .unwrap()
        .clone()
}
fn ready(p: Player) -> (Game, CardId, u64) {
    let mut g = setup(p);
    let f = paid(&mut g, p, FIST);
    age(&mut g, p);
    let t = arm(&mut g, p, f);
    (g, f, t)
}
#[test]
fn msh_self_activation_paid_body_and_no_unacquired_printed_template() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let f = paid(&mut g, p, FIST);
        assert_eq!(
            g.stats(f),
            Stats {
                power: 2,
                toughness: 3
            }
        );
        assert!(g.object_view(f).is_legendary());
        for ty in ["Human", "Warrior", "Hero"] {
            assert!(g.has_subtype(f, ty));
        }
        assert_eq!(g.object_view(f).colors, 8);
        age(&mut g, p);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::AnnounceSelfGrantedAbility { .. }))
        );
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: f,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert!(g.state.self_granted_abilities.is_none());
    }
}
#[test]
fn msh_self_activation_paid_tap_damage_and_distinct_owned_context() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        let before = g.state.objects[f as usize].used_abilities;
        let item = punch(&mut g, p, f, t, Target::Player(p.other()));
        assert_eq!(
            item.self_granted_activation
                .as_ref()
                .unwrap()
                .grant_timestamp,
            t
        );
        assert_eq!(item.source_card, Card::from_name(FIST).unwrap());
        assert!(g.state.objects[f as usize].tapped);
        assert_eq!(g.state.objects[f as usize].used_abilities, before);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert!(g.on_battlefield(f));
        assert!(!g.legal_actions().contains(&action(&g, f, t)));
        let json = serde_json::to_value(g.self_granted_ability_views().unwrap()).unwrap();
        assert!(json[0].get("effect_timestamp").is_none());
    }
}
#[test]
fn msh_self_activation_spell_relation_and_target_negatives() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let f = paid(&mut g, p, FIST);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        packet(&mut g, p, "Shock", None, &[Target::Card(enemy)]);
        settle(&mut g);
        packet(&mut g, p.other(), "Shock", None, &[Target::Card(enemy)]);
        settle(&mut g);
        packet(&mut g, p, "Shock", None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert!(g.state.self_granted_abilities.is_none());
        let token = named(&mut g, p, "@token/msh-moloid", Zone::Battlefield);
        packet(&mut g, p, "Shock", None, &[Target::Card(token)]);
        settle(&mut g);
        assert_eq!(g.state.self_granted_abilities.as_ref().unwrap().len(), 1);
        assert!(g.on_battlefield(f));
    }
}
#[test]
fn msh_self_activation_other_target_atomic_self_land_and_artifact_rejection() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let art = named(&mut g, p, "Arc Reactor", Zone::Battlefield);
        let a = action(&g, f, t);
        checked(&mut g, a);
        for id in [f, land, art] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 18);
    }
}
#[test]
fn msh_self_activation_any_other_creature_and_planeswalker_damage() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            let (mut g, f, t) = ready(p);
            let a = named(&mut g, owner, "Hill Giant", Zone::Battlefield);
            punch(&mut g, p, f, t, Target::Card(a));
            settle(&mut g);
            assert_eq!(g.object_view(a).damage, 2);
        }
        let (mut g, f, t) = ready(p);
        let a = named(
            &mut g,
            p.other(),
            "Liliana, Dreadhorde General",
            Zone::Battlefield,
        );
        g.set_counters(a, Counters::of(CounterKind::Loyalty, 6));
        punch(&mut g, p, f, t, Target::Card(a));
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::Loyalty), 4);
    }
}
#[test]
fn msh_self_activation_multiple_grants_untap_reuses_same_instance_without_once_history() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        let u = arm(&mut g, p, f);
        assert_ne!(t, u);
        punch(&mut g, p, f, t, Target::Player(p.other()));
        settle(&mut g);
        packet(&mut g, p, "Shore Up", None, &[Target::Card(f)]);
        settle(&mut g);
        assert!(!g.state.objects[f as usize].tapped);
        assert_eq!(g.state.self_granted_abilities.as_ref().unwrap().len(), 3);
        assert!(g.legal_actions().contains(&action(&g, f, t)));
        assert!(g.legal_actions().contains(&action(&g, f, u)));
        punch(&mut g, p, f, t, Target::Player(p.other()));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 15);
        assert_eq!(g.state.objects[f as usize].used_abilities, 0);
    }
}
#[test]
fn msh_self_activation_summoning_sickness_then_real_haste_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cavalry = paid(&mut g, p, "Axgard Cavalry");
        age(&mut g, p);
        let f = paid(&mut g, p, FIST);
        let t = arm(&mut g, p, f);
        assert!(!g.legal_actions().contains(&action(&g, f, t)));
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: cavalry,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(f),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert!(g.keywords(f).contains(Keywords::HASTE));
        punch(&mut g, p, f, t, Target::Player(p.other()));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_self_activation_current_power_is_resolution_time_not_announcement() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        punch(&mut g, p, f, t, Target::Player(p.other()));
        packet(&mut g, p, "Giant Growth", None, &[Target::Card(f)]);
        settle(&mut g);
        assert_eq!(g.stats(f).power, 5);
        assert_eq!(g.state.players[p.other().index()].life, 15);
    }
}
#[test]
fn msh_self_activation_departure_lki_and_new_incarnation_do_not_rebind() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        packet(&mut g, p, "Undying Malice", None, &[Target::Card(f)]);
        settle(&mut g);
        let reference = g.object_ref(f);
        punch(&mut g, p, f, t, Target::Player(p.other()));
        packet(&mut g, p, "Giant Growth", None, &[Target::Card(f)]);
        until(&mut g, |g| {
            g.stats(f).power == 5 && g.state.resolving.is_none()
        });
        packet(&mut g, p.other(), "Murder", None, &[Target::Card(f)]);
        settle(&mut g);
        assert_ne!(g.object_ref(f), reference);
        assert!(g.on_battlefield(f));
        assert_eq!(g.stats(f).power, 3);
        assert_eq!(g.state.players[p.other().index()].life, 15);
        assert!(g.state.self_granted_abilities.is_none());
    }
}
#[test]
fn msh_self_activation_bounded_copy_preserves_granted_program_and_current_power() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        let reference = g.object_ref(f);
        // Bounded shared-copy helper transition; payment and damage below are actual actions.
        g.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(g.object_ref(f), reference);
        let item = punch(&mut g, p, f, t, Target::Player(p.other()));
        assert_eq!(item.source_card.name(), FIST);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}
#[test]
fn msh_self_activation_layer6_removal_and_restoration_preserve_exact_instances() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        let aura = packet(&mut g, p, "Witness Protection", None, &[Target::Card(f)]);
        settle(&mut g);
        assert!(!g.legal_actions().contains(&action(&g, f, t)));
        assert!(
            g.self_granted_ability_views()
                .unwrap()
                .iter()
                .all(|r| !r.available)
        );
        let retained = g.state.self_granted_abilities.clone();
        packet(&mut g, p, "Disenchant", None, &[Target::Card(aura)]);
        settle(&mut g);
        assert_eq!(g.state.self_granted_abilities, retained);
        assert!(g.legal_actions().contains(&action(&g, f, t)));
        punch(&mut g, p, f, t, Target::Player(p.other()));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_self_activation_original_spell_countered_still_acquires_independent_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let f = paid(&mut g, p, FIST);
        age(&mut g, p);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let spell = packet(&mut g, p, "Shock", None, &[Target::Card(a)]);
        assert!(g.state.stack.iter().any(|i| i.card == spell));
        packet(&mut g, p.other(), "Negate", None, &[Target::Spell(spell)]);
        settle(&mut g);
        assert_eq!(g.object_view(a).damage, 0);
        let t = g.state.self_granted_abilities.as_ref().unwrap()[0].timestamp;
        punch(&mut g, p, f, t, Target::Player(p.other()));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_self_activation_source_gone_before_grant_cannot_bestow_on_returned_object() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let f = paid(&mut g, p, FIST);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        packet(&mut g, p, "Shock", None, &[Target::Card(a)]);
        packet(&mut g, p.other(), "Murder", None, &[Target::Card(f)]);
        settle(&mut g);
        assert!(!g.on_battlefield(f));
        assert!(g.state.self_granted_abilities.is_none());
    }
}
#[test]
fn msh_self_activation_negative_current_power_clamps_to_zero() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        // Bounded signed-power input, preserving positive toughness.
        g.state.objects[f as usize].growth.power = -4;
        assert_eq!(g.stats(f).power, -2);
        punch(&mut g, p, f, t, Target::Player(p.other()));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn msh_self_activation_opponent_turn_and_end_turn_expiry_hide_template() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let f = paid(&mut g, p, FIST);
        age(&mut g, p);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        let t = arm(&mut g, p, f);
        punch(&mut g, p, f, t, Target::Player(p.other()));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        packet(&mut g, p, "Time Stop", None, &[]);
        settle(&mut g);
        assert!(g.state.self_granted_abilities.is_none());
        priority(&mut g, p);
        let a = action(&g, f, t);
        reject(&mut g, a);
    }
}
#[test]
fn msh_self_activation_owned_announcement_target_payment_and_wrong_stamp_atomicity() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        let a = action(&g, f, t + 100);
        reject(&mut g, a);
        let stale = Action::AnnounceSelfGrantedAbility {
            source: ObjectRef {
                card: f,
                generation: g.object_ref(f).generation + 1,
            },
            grant_timestamp: t,
        };
        reject(&mut g, stale);
        let a = action(&g, f, t);
        g = orphan(g, a);
        assert!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .self_granted_activation
                .is_some()
        );
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            g = orphan(g, a);
        }
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_self_activation_copied_packet_preserves_origin_and_independent_targets_without_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let f = paid(&mut g, p, FIST);
        let echo = paid(&mut g, p, "Echo, Perceptive Prodigy");
        age(&mut g, p);
        let t = arm(&mut g, p, f);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let original = punch(&mut g, p, f, t, Target::Player(p.other()));
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(original.id),
            },
        );
        finish(&mut g);
        until(&mut g, |g| g.retarget_view().is_some());
        let copy = g.retarget_view().unwrap();
        assert_eq!(
            copy.item.self_granted_activation,
            original.self_granted_activation
        );
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(bear),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert!(!g.on_battlefield(bear));
        assert!(g.on_battlefield(f));
        assert!(g.state.objects[f as usize].tapped);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.objects[f as usize].used_abilities, 0);
    }
}
#[test]
fn msh_self_activation_full_nonactor_privacy_and_old_packet_context_absence() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        let secret = named(&mut g, p, "Murder", Zone::Hand);
        let mut other = g.clone();
        other.state.objects[secret as usize].card = Card::from_name("Negate").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(other.observe(p.other())).unwrap()
        );
        let item = punch(&mut g, p, f, t, Target::Player(p.other()));
        let other_item = punch(&mut other, p, f, t, Target::Player(p.other()));
        assert_eq!(item, other_item);
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(other.observe(p.other())).unwrap()
        );
        settle(&mut g);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        packet(&mut g, p, "Shock", None, &[Target::Card(a)]);
        for i in &g.state.stack {
            assert!(
                serde_json::to_value(i)
                    .unwrap()
                    .get("self_granted_activation")
                    .is_none()
            );
        }
    }
}

#[test]
fn msh_self_activation_cast_mana_and_sorcery_timing_failures_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let f = hand(&mut g, p, FIST);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: f });
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        reject(&mut g, Action::AnnounceCast { card: f });
    }
}
#[test]
fn msh_self_activation_ward_counters_paid_ability_without_moving_source() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        let ward = named(&mut g, p.other(), "Thornfist Striker", Zone::Battlefield);
        let item = punch(&mut g, p, f, t, Target::Card(ward));
        until(&mut g, |g| g.observe(p).resolution_payment.is_some());
        assert_eq!(g.actor(), Some(p));
        g = orphan(g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(ward).damage, 0);
        assert!(g.on_battlefield(f));
        assert!(g.object_view(f).tapped);
        assert!(g.state.stack.iter().all(|i| i.id != item.id));
        assert_eq!(g.state.self_granted_abilities.as_ref().unwrap().len(), 1);
    }
}
#[test]
fn msh_self_activation_source_tap_is_reserved_in_actual_and_borrowed_funding() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        let reference = g.object_ref(f);
        // Bounded in-place copy supplies a real printed mana ability on the same recipient.
        g.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Llanowar Elves").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        g.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: Some((
                reference,
                0,
                Counters::default(),
                Card::from_name(FIST).unwrap(),
                None,
            )),
        };
        assert!(g.can_fund(goal.clone()));
        let mut impossible = goal;
        impossible.required = [0, 0, 0, 0, 1, 0];
        assert!(!g.can_fund(impossible));
        let before = g.snapshot();
        let world = FundingState::from_game(
            &g,
            &FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            },
        );
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&world).stats(f),
            g.stats(f)
        );
        assert_eq!(g.snapshot(), before);
        punch(&mut g, p, f, t, Target::Player(p.other()));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        reject(&mut g, Action::TapMana { card: f });
    }
}
#[test]
fn msh_self_activation_copiable_values_do_not_copy_resolved_activation_grants() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, _) = ready(p);
        let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        // Bounded flat-recipe transition exercises copied printed templates independently of grants.
        let recipe = g.copiable_recipe(g.object_ref(f));
        g.apply_temporary_copy(g.object_ref(other), recipe, CopyDeadline::EndOfTurn);
        assert!(g.legal_actions().iter().all(
            |a| !matches!(a,Action::AnnounceSelfGrantedAbility{source,..}if source.card==other)
        ));
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: other,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn msh_self_activation_controller_change_preserves_packet_controller_and_origin() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        let item = punch(&mut g, p, f, t, Target::Player(p.other()));
        // Bounded continuous-control transition; the already paid packet retains its controller.
        g.record_control(g.object_ref(f), p.other(), None);
        assert_eq!(g.state.objects[f as usize].controller, p.other());
        assert_eq!(
            g.state
                .stack
                .iter()
                .find(|i| i.id == item.id)
                .unwrap()
                .controller,
            p
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(
            g.state.self_granted_abilities.as_ref().unwrap()[0].timestamp,
            t
        );
    }
}
#[test]
fn msh_self_activation_real_loki_observes_target_acquisition_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let f = paid(&mut g, p, FIST);
        let loki = paid(&mut g, p, "Loki, God of Mischief");
        age(&mut g, p);
        let t = arm(&mut g, p, f);
        let hand = g.hand_count(p);
        punch(&mut g, p, f, t, Target::Player(p.other()));
        assert!(
            g.state
                .pending_triggers
                .iter()
                .chain(&g.state.ordering_triggers)
                .chain(&g.state.stack)
                .any(|i| i.card == loki)
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_self_activation_exiled_printed_template_is_not_borrowed_by_forge() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, _) = ready(p);
        // Bounded copy makes the actual Forge target an artifact; exile restores physical Iron Fist.
        g.apply_temporary_copy(
            g.object_ref(f),
            CopyRecipe::new(Card::from_name("Three Tree Mascot").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let forge = packet(&mut g, p, "Territory Forge", None, &[]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(f),
            })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(f),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(f).zone, Zone::Exile);
        assert_eq!(g.state.objects[f as usize].card.name(), FIST);
        assert!(
            g.borrowed_ability_views()
                .iter()
                .all(|v| v.source.card != forge)
        );
        assert!(g.state.self_granted_abilities.is_none());
    }
}

#[test]
fn msh_self_activation_multiple_target_groups_acquire_one_instance_per_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let f = paid(&mut g, p, FIST);
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        packet(
            &mut g,
            p,
            "Epic Fight",
            Some(2),
            &[Target::Card(own), Target::Card(own), Target::Card(enemy)],
        );
        settle(&mut g);
        assert_eq!(g.state.self_granted_abilities.as_ref().unwrap().len(), 1);
        assert_eq!(
            g.state.self_granted_abilities.as_ref().unwrap()[0].recipient,
            g.object_ref(f)
        );
    }
}
#[test]
fn msh_self_activation_departed_target_fizzles_without_moving_source_or_repaying_tap() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f, t) = ready(p);
        let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        punch(&mut g, p, f, t, Target::Card(target));
        packet(&mut g, p.other(), "Murder", None, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.object_view(target).damage, 0);
        assert!(g.on_battlefield(f));
        assert!(g.object_view(f).tapped);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn msh_self_activation_current_global_activation_prohibition_applies_to_acquired_instances() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let f = paid(&mut g, p, FIST);
        age(&mut g, p);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        let t = arm(&mut g, p, f);
        paid(&mut g, p.other(), "Grand Abolisher");
        priority(&mut g, p);
        let a = action(&g, f, t);
        reject(&mut g, a);
        assert!(g.self_granted_ability_views().unwrap()[0].available);
        // The restriction is on activation, not a removal of the acquired ability.
        assert_eq!(g.state.active, p.other());
        assert!(!g.object_view(f).tapped);
    }
}
