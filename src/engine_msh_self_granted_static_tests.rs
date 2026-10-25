//! Focused static self-grant rules; Armor-kind helper fixtures are explicitly bounded.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
const MARVEL: &str = "Ms. Marvel, Kamala Khan";
fn hand(g: &mut Game, p: Player, name: &str) -> CardId {
    if name == MARVEL && !Card::from_name(name).unwrap().is_supported() {
        let id = named(g, p, "Hill Giant", Zone::Hand);
        g.state.objects[id as usize].card = Card::from_name(name).unwrap();
        id
    } else {
        named(g, p, name, Zone::Hand)
    }
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

fn source_packet(g: &Game, id: CardId) -> SpellView {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .find(|item| item.card == id && matches!(item.kind, StackKind::TriggeredAbility { .. }))
        .unwrap()
        .clone()
}
fn grant_hand(g: &mut Game, p: Player, id: CardId) -> SpellView {
    let target = named(g, p, "Hill Giant", Zone::Battlefield);
    packet(g, p, "Shock", None, &[Target::Card(target)]);
    let item = source_packet(g, id);
    settle(g);
    item
}
fn bonus(g: &mut Game, item: &SpellView) {
    g.grant_self_ability(item, SelfGrantedAbilityKind::ControllerArtifactBonus);
}
fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
#[test]
fn msh_self_static_paid_marvel_draws_then_dynamic_layer7b_counters_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        assert_eq!(
            g.stats(m),
            Stats {
                power: 1,
                toughness: 4
            }
        );
        named(&mut g, p, "Forest", Zone::Hand);
        let count = g.hand_count(p);
        grant_hand(&mut g, p, m);
        assert_eq!(g.hand_count(p), count + 1);
        assert_eq!(g.stats(m).power, (count + 1) as i128);
        assert_eq!(g.object_view(m).damage, 0);
        named(&mut g, p, "Forest", Zone::Hand);
        assert_eq!(g.stats(m).power, (count + 2) as i128);
        g.effect_counter(
            m,
            CounterKind::PlusOne,
            2,
            replacements::ResumeMode::Pending,
        );
        assert_eq!(g.stats(m).power, (count + 4) as i128);
        next_turn(&mut g);
        assert_eq!(
            g.stats(m),
            Stats {
                power: 3,
                toughness: 6
            }
        );
        assert!(g.state.self_granted_abilities.is_none());
    }
}
#[test]
fn msh_self_static_marvel_only_own_cast_targets_current_own_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        packet(&mut g, p, "Shock", None, &[Target::Card(enemy)]);
        assert!(g.state.pending_triggers.iter().all(|i| i.card != m));
        settle(&mut g);
        packet(&mut g, p.other(), "Shock", None, &[Target::Card(m)]);
        assert!(g.state.pending_triggers.iter().all(|i| i.card != m));
        settle(&mut g);
        packet(&mut g, p, "Shock", None, &[Target::Player(p.other())]);
        assert!(g.state.pending_triggers.iter().all(|i| i.card != m));
        settle(&mut g);
        assert!(g.state.self_granted_abilities.is_none());
    }
}
#[test]
fn msh_self_static_hand_setter_orders_against_real_later_and_earlier_animation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        for _ in 0..6 {
            named(&mut g, p, "Forest", Zone::Hand);
        }
        packet(&mut g, p, "I Am Iron Man", None, &[Target::Card(m)]);
        settle(&mut g);
        assert_eq!(
            g.stats(m),
            Stats {
                power: 4,
                toughness: 4
            }
        ); // Later base setter overrides earlier self-grant.
        grant_hand(&mut g, p, m);
        assert_eq!(g.stats(m).power, g.hand_count(p) as i128);
        assert!(g.stats(m).power > 4);
    }
}
#[test]
fn msh_self_static_ability_removal_suppresses_then_aura_departure_restores_old_instance() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        let item = grant_hand(&mut g, p, m);
        packet(&mut g, p, "Witness Protection", None, &[Target::Card(m)]);
        settle(&mut g);
        let aura = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.card.name() == "Witness Protection")
            .unwrap()
            .0 as CardId;
        let rows = g.self_granted_ability_views().unwrap();
        assert!(rows.iter().all(|r| !r.available));
        assert_eq!(
            g.stats(m),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        let retained = g.state.self_granted_abilities.clone();
        named(&mut g, p, "Forest", Zone::Hand);
        assert_eq!(g.stats(m).power, 1);
        packet(&mut g, p, "Disenchant", None, &[Target::Card(aura)]);
        settle(&mut g);
        assert_eq!(g.state.self_granted_abilities, retained);
        assert_eq!(g.stats(m).power, g.hand_count(p) as i128);
        assert!(
            g.self_granted_ability_views()
                .unwrap()
                .iter()
                .all(|r| r.available)
        );
        assert_eq!(item.object, g.object_ref(m));
    }
}
#[test]
fn msh_self_static_later_grant_survives_earlier_removal_bounded_same_effect_time() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        let item = grant_hand(&mut g, p, m);
        packet(&mut g, p, "Witness Protection", None, &[Target::Card(m)]);
        settle(&mut g);
        // Bounded resolving helper: grant newer than the existing removal. Armor's
        // consumer supplies animation and grant at this same continuous-effect time.
        let timestamp = g.next_role_timestamp();
        g.grant_self_ability_at(
            &item,
            SelfGrantedAbilityKind::ControllerArtifactBonus,
            timestamp,
        );
        named(&mut g, p, "Fishing Pole", Zone::Battlefield);
        assert_eq!(
            g.stats(m),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(
            g.self_granted_ability_views()
                .unwrap()
                .iter()
                .any(|r| r.timestamp == timestamp && r.available)
        );
    }
}
#[test]
fn msh_self_static_grants_survive_copy_but_copiable_values_exclude_resolved_grants() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        grant_hand(&mut g, p, m);
        let count = g.hand_count(p) as i128;
        let reference = g.object_ref(m);
        g.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(
            g.stats(m),
            Stats {
                power: count,
                toughness: 3
            }
        );
        let recipe = g.copiable_recipe(g.object_ref(m));
        let copies = g.create_copy_groups_inner(
            p,
            &[TokenGroupView {
                token: recipe.definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(recipe)],
            None,
        );
        g.advance_entries();
        settle(&mut g);
        assert_eq!(
            g.stats(copies[0].card),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert_eq!(g.state.self_granted_abilities.as_ref().unwrap().len(), 1);
    }
}
#[test]
fn msh_self_static_exact_source_departure_new_generation_and_queued_draw_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        packet(&mut g, p, "Shock", None, &[Target::Card(m)]);
        let count = g.hand_count(p);
        let old = g.object_ref(m);
        g.move_object(m, Zone::Hand);
        g.move_object(m, Zone::Battlefield);
        assert_ne!(g.object_ref(m), old);
        settle(&mut g);
        assert_eq!(g.hand_count(p), count + 1);
        assert!(g.state.self_granted_abilities.is_none());
        assert_eq!(g.stats(m).power, 1);
    }
}
#[test]
fn msh_self_static_current_controller_hand_count_and_borrowed_without_hand() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        grant_hand(&mut g, p, m);
        for _ in 0..4 {
            named(&mut g, p.other(), "Forest", Zone::Hand);
        }
        let reference = g.object_ref(m);
        g.record_control(reference, p.other(), None);
        assert_eq!(g.stats(m).power, 4);
        let secret = named(&mut g, p.other(), "Murder", Zone::Hand);
        let before = g.snapshot();
        let query = g.characteristic_query();
        assert_eq!(query.stats_without_hand(m, Some(secret)).power, 4);
        assert_eq!(query.stats(m).power, 5);
        let world = FundingState::from_game(&g, &goal(p.other()));
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&world).stats(m),
            g.stats(m)
        );
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_self_static_artifact_bonus_counts_effective_artifacts_self_and_artifact_land() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        let item = grant_hand(&mut g, p, m);
        bonus(&mut g, &item);
        let h = g.hand_count(p) as i128;
        assert_eq!(
            g.stats(m),
            Stats {
                power: h,
                toughness: 4
            }
        );
        named(&mut g, p, "Fishing Pole", Zone::Battlefield);
        assert_eq!(
            g.stats(m),
            Stats {
                power: h + 1,
                toughness: 5
            }
        );
        let recipe = CopyRecipe::new(Card::from_name("Forest").unwrap())
            .with_exceptions(CopyExceptions::ARTIFACT);
        let copies = g.create_copy_groups_inner(
            p,
            &[TokenGroupView {
                token: recipe.definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(recipe)],
            None,
        );
        g.advance_entries();
        settle(&mut g);
        assert_eq!(g.current_kinds(copies[0].card) & 18, 18);
        assert_eq!(
            g.stats(m),
            Stats {
                power: h + 2,
                toughness: 6
            }
        );
        let reference = g.object_ref(m);
        let mut recipe = g.copiable_recipe(g.object_ref(m));
        recipe = recipe.with_exceptions(CopyExceptions::ARTIFACT);
        g.apply_temporary_copy(reference, recipe, CopyDeadline::EndOfTurn);
        assert_eq!(
            g.stats(m),
            Stats {
                power: h + 3,
                toughness: 7
            }
        );
    }
}
#[test]
fn msh_self_static_artifact_bonus_dynamic_control_double_instances_and_virtual_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        let item = grant_hand(&mut g, p, m);
        bonus(&mut g, &item);
        bonus(&mut g, &item);
        let a = named(&mut g, p, "Fishing Pole", Zone::Battlefield);
        named(&mut g, p.other(), "Arc Reactor", Zone::Battlefield);
        let h = g.hand_count(p) as i128;
        assert_eq!(g.stats(m).power, h + 2);
        let before = g.snapshot();
        let request = goal(p);
        let mut world = FundingState::from_game(&g, &request);
        world.prepare_entry_projection();
        let mut actual = g.clone();
        actual.move_object(a, Zone::Graveyard);
        let expected = actual.object_view(m);
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(a)], Zone::Graveyard, &mut |after| {
            visits += 1;
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(after).object_view(m),
                expected
            );
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), before);
        let reference = g.object_ref(m);
        g.record_control(reference, p.other(), None);
        assert_eq!(
            g.stats(m),
            Stats {
                power: 2,
                toughness: 6
            }
        );
    }
}
#[test]
fn msh_self_static_funding_aura_departure_restores_grant_without_parent_mutation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        let item = grant_hand(&mut g, p, m);
        bonus(&mut g, &item);
        named(&mut g, p, "Fishing Pole", Zone::Battlefield);
        let aura = packet(&mut g, p, "Witness Protection", None, &[Target::Card(m)]);
        settle(&mut g);
        assert_eq!(g.stats(m).power, 1);
        let before = g.snapshot();
        let mut actual = g.clone();
        actual.move_object(aura, Zone::Graveyard);
        let expected = actual.object_view(m);
        let mut world = FundingState::from_game(&g, &goal(p));
        world.prepare_entry_projection();
        let mut visits = 0;
        world.cost_departures(&[g.object_ref(aura)], Zone::Graveyard, &mut |after| {
            visits += 1;
            assert_eq!(
                characteristic_query::CharacteristicQuery::in_world(after).object_view(m),
                expected
            );
            false
        });
        assert_eq!(visits, 1);
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_self_static_owned_raw_index_restore_privacy_bytes_and_absent_old_payload() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        assert!(
            serde_json::to_value(g.observe(p))
                .unwrap()
                .get("self_granted_abilities")
                .is_none()
        );
        let m = paid(&mut g, p, MARVEL);
        let hidden = named(&mut g, p, "Murder", Zone::Hand);
        packet(&mut g, p, "Shock", None, &[Target::Card(m)]);
        until(&mut g, |g| g.state.self_granted_abilities.is_some());
        assert_eq!(
            self_granted::owned_bytes(&g.state.self_granted_abilities),
            std::mem::size_of::<Vec<self_granted::Row>>()
                + std::mem::size_of::<self_granted::Row>()
        );
        let mut other = g.clone();
        other.state.objects[hidden as usize].card = Card::from_name("Negate").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(other.observe(p.other())).unwrap()
        );
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|v| v.reference.card == hidden)
        );
        g = orphan(g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.stats(m).power, g.hand_count(p) as i128);
    }
}

#[test]
fn msh_self_static_marvel_cast_cost_timing_rejections_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = hand(&mut g, p, MARVEL);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: m });
        g.state.players[p.index()].mana = [10; 6];
        g.state.step = Step::Upkeep;
        reject(&mut g, Action::AnnounceCast { card: m });
    }
}

#[test]
fn msh_self_static_physical_retimestamp_moves_static_effects_without_reacquiring_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MARVEL);
        grant_hand(&mut g, p, m);
        packet(&mut g, p, "I Am Iron Man", None, &[Target::Card(m)]);
        settle(&mut g);
        assert_eq!(g.stats(m).power, 4);
        let before = g.state.self_granted_abilities.clone().unwrap();
        let reference = g.object_ref(m);
        // Bounded physical-object retimestamp primitive: this is not a claim that a
        // creature can equip or that a single-faced Marvel can transform.
        g.retimestamp_live_static_source(reference);
        let after = g.state.self_granted_abilities.clone().unwrap();
        assert_eq!(after.len(), before.len());
        for (old, new) in before.iter().zip(after.iter()) {
            assert_eq!(new.timestamp, old.timestamp);
            assert!(new.effect_timestamp > old.effect_timestamp);
        }
        assert!(
            after
                .windows(2)
                .all(|rows| rows[0].effect_timestamp < rows[1].effect_timestamp)
        );
        assert_eq!(g.stats(m).power, g.hand_count(p) as i128);
        packet(&mut g, p, "Witness Protection", None, &[Target::Card(m)]);
        settle(&mut g);
        let acquired: Vec<_> = g
            .state
            .self_granted_abilities
            .as_ref()
            .unwrap()
            .iter()
            .map(|r| r.timestamp)
            .collect();
        g.retimestamp_live_static_source(reference);
        assert_eq!(g.stats(m).power, 1);
        assert_eq!(
            g.state
                .self_granted_abilities
                .as_ref()
                .unwrap()
                .iter()
                .map(|r| r.timestamp)
                .collect::<Vec<_>>(),
            acquired
        );
        assert!(
            g.self_granted_ability_views()
                .unwrap()
                .iter()
                .all(|row| !row.available)
        );
    }
}
