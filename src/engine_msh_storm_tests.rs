//! Focused paid rules for Storm's non-target cast subjects and combat restrictions.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const STORM: &str = "Storm, Windrider";
fn hand(g: &mut Game, p: Player, name: &str) -> CardId {
    if name == STORM && !Card::from_name(name).unwrap().is_supported() {
        // Bounded gated fixture: ordinary paid actions, rejected accepted-pool definition.
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
fn age(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    until(g, |g| g.state.active == p && g.state.step == Step::Main1);
    priority(g, p);
}
fn activation(g: &mut Game, id: CardId, targets: &[Target]) {
    let p = g.state.objects[id as usize].controller;
    priority(g, p);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
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

fn storm_packet(g: &Game, id: CardId) -> &SpellView {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .find(|item| item.card == id && matches!(item.kind, StackKind::TriggeredAbility { .. }))
        .unwrap()
}
fn granted(g: &Game, id: CardId) -> bool {
    g.keywords(id).contains(Keywords::FLYING)
}
fn resolve_grant(g: &mut Game, id: CardId) {
    until(g, |g| {
        g.state.roles.as_ref().is_some_and(|r| {
            r.keyword_effects
                .iter()
                .any(|e| e.recipient == g.object_ref(id) && e.keywords.contains(Keywords::FLYING))
        })
    });
}
fn combat(g: &mut Game, p: Player) {
    age(g, p);
    until(g, |g| {
        g.state.pending == Pending::Attackers && g.state.active == p
    });
}
#[test]
fn msh_storm_cast_own_and_opponent_creatures_frozen_subjects_not_targets() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            let mut g = setup(p);
            let s = paid(&mut g, p, STORM);
            let a = named(&mut g, owner, "Hill Giant", Zone::Battlefield);
            let spell = packet(&mut g, p, "Shock", None, &[Target::Card(a)]);
            let trigger = storm_packet(&g, s);
            assert!(trigger.targets.is_empty());
            let facts = trigger.cast_creature_subjects.as_ref().unwrap();
            assert_eq!(facts.objects, vec![g.object_ref(a)]);
            assert_eq!(
                facts.spell,
                g.state.stack.iter().find(|x| x.card == spell).unwrap().id
            );
            resolve_grant(&mut g, a);
            assert!(granted(&g, a));
            assert_eq!(g.object_view(a).damage, 0);
            settle(&mut g);
            assert_eq!(g.object_view(a).damage, 2);
            next_turn(&mut g);
            assert!(!granted(&g, a));
        }
    }
}
#[test]
fn msh_storm_multigroup_deduplicates_exact_subjects_and_gives_both_controllers_flying() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        packet(
            &mut g,
            p,
            "Epic Fight",
            Some(2),
            &[Target::Card(a), Target::Card(a), Target::Card(b)],
        );
        assert_eq!(
            storm_packet(&g, s)
                .cast_creature_subjects
                .as_ref()
                .unwrap()
                .objects,
            vec![g.object_ref(a), g.object_ref(b)]
        );
        resolve_grant(&mut g, a);
        assert!(granted(&g, b));
        assert_eq!(g.stats(a).power, 3);
        settle(&mut g);
    }
}
#[test]
fn msh_storm_noncreature_player_and_enemy_casts_have_no_subject_payload() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let pole = paid(&mut g, p, "Fishing Pole");
        packet(&mut g, p, "Abrade", Some(1), &[Target::Card(pole)]);
        assert!(g.state.pending_triggers.iter().all(|x| x.card != s));
        assert!(
            g.state
                .stack
                .iter()
                .all(|x| x.cast_creature_subjects.is_none())
        );
        settle(&mut g);
        packet(&mut g, p, "Shock", None, &[Target::Player(p.other())]);
        assert!(g.state.pending_triggers.iter().all(|x| x.card != s));
        settle(&mut g);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        packet(&mut g, p.other(), "Shock", None, &[Target::Card(a)]);
        assert!(g.state.pending_triggers.iter().all(|x| x.card != s));
        settle(&mut g);
        assert!(!granted(&g, a));
    }
}
#[test]
fn msh_storm_original_countered_still_grants_and_source_departure_does_not_cancel() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let s = paid(&mut g, p, STORM);
            let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let spell = packet(&mut g, p, "Shock", None, &[Target::Card(a)]);
            if depart {
                packet(&mut g, p.other(), "Murder", None, &[Target::Card(s)]);
            } else {
                packet(&mut g, p.other(), "Negate", None, &[Target::Spell(spell)]);
            }
            settle(&mut g);
            assert!(granted(&g, a));
            if depart {
                assert_eq!(g.object_view(s).zone, Zone::Graveyard);
            } else {
                assert_eq!(g.object_view(a).damage, 0);
            }
        }
    }
}
#[test]
fn msh_storm_subject_departure_reentry_never_rebinds_and_owned_packet_restores() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        packet(&mut g, p, "Shock", None, &[Target::Card(a)]);
        let old = g.object_ref(a);
        g.move_object(a, Zone::Hand);
        g.move_object(a, Zone::Battlefield);
        assert_ne!(old, g.object_ref(a));
        assert_eq!(
            storm_packet(&g, s)
                .cast_creature_subjects
                .as_ref()
                .unwrap()
                .objects,
            vec![old]
        );
        g.prepare_triggers();
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            .unwrap();
        g = orphan(g, action);
        settle(&mut g);
        assert!(!granted(&g, a));
        assert_eq!(g.object_view(a).damage, 0);
    }
}
#[test]
fn msh_storm_subject_stays_frozen_after_original_retargets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let original = packet(&mut g, p, "Shock", None, &[Target::Card(a)]);
        packet(
            &mut g,
            p.other(),
            "Bolt Bend",
            None,
            &[Target::Spell(original)],
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        assert_eq!(
            storm_packet(&g, s)
                .cast_creature_subjects
                .as_ref()
                .unwrap()
                .objects,
            vec![g.object_ref(a)]
        );
        settle(&mut g);
        assert!(granted(&g, a));
        assert!(!granted(&g, b));
        assert_eq!(g.object_view(b).damage, 2);
    }
}
#[test]
fn msh_storm_flying_attack_player_forbidden_planeswalker_allowed_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, STORM);
        let flyer = paid(&mut g, p.other(), "Serra Angel");
        let walker = named(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 6;
        combat(&mut g, p.other());
        reject(&mut g, Action::Attack { card: flyer });
        let action = Action::AttackPlaneswalker {
            card: flyer,
            defender: g.object_ref(walker),
        };
        assert!(g.legal_actions().contains(&action));
        g = orphan(g, action);
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.life(p), 20);
        assert!(!g.object_view(flyer).tapped); // Serra Angel has vigilance.
    }
}
#[test]
fn msh_storm_flying_blocker_forbidden_ground_blocker_legal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let own = paid(&mut g, p, "Hill Giant");
        let flyer = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let ground = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        combat(&mut g, p);
        checked(&mut g, Action::Attack { card: own });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: flyer,
                attacker: own,
            },
        );
        assert!(g.can_declare_attacker(p, s));
        checked(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: own,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert!(g.object_view(ground).zone == Zone::Graveyard);
        assert_eq!(g.life(p.other()), 20);
    }
}
#[test]
fn msh_storm_controller_relation_allows_own_flying_blocker_against_opponents_attack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let enemy = paid(&mut g, p.other(), "Hill Giant");
        combat(&mut g, p.other());
        checked(&mut g, Action::Attack { card: enemy });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: s,
                attacker: enemy,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.life(p), 20);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_storm_mandatory_flying_attacker_no_defender_can_skip_but_walker_requires_attack() {
    for p in [Player::P0, Player::P1] {
        for walker in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, STORM);
            let phoenix = paid(&mut g, p.other(), "Flamewake Phoenix");
            let defender = walker.then(|| {
                let id = named(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
                g.state.objects[id as usize].counters.loyalty = 6;
                id
            });
            combat(&mut g, p.other());
            assert!(
                !g.legal_actions()
                    .contains(&Action::Attack { card: phoenix })
            );
            if let Some(id) = defender {
                reject(&mut g, Action::FinishAttackers);
                let reference = g.object_ref(id);
                checked(
                    &mut g,
                    Action::AttackPlaneswalker {
                        card: phoenix,
                        defender: reference,
                    },
                );
            }
            checked(&mut g, Action::FinishAttackers);
        }
    }
}
#[test]
fn msh_storm_suppression_before_cast_removes_observer_and_combat_restriction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        packet(&mut g, p, "Witness Protection", None, &[Target::Card(s)]);
        settle(&mut g);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        packet(&mut g, p, "Shock", None, &[Target::Card(a)]);
        assert!(g.state.pending_triggers.iter().all(|x| x.card != s));
        settle(&mut g);
        assert!(!granted(&g, a));
        let flyer = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        assert!(g.can_attack_defender(flyer, CombatDefender::Player(p)));
    }
}

#[test]
fn msh_storm_paid_body_timing_insufficient_mana_and_no_old_payload() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        assert_eq!(
            g.stats(s),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert!(granted(&g, s));
        assert!(g.object_view(s).is_legendary());
        assert!(g.has_subtype(s, "Mutant") && g.has_subtype(s, "Hero"));
        let mut g = setup(p);
        let id = hand(&mut g, p, STORM);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.players[p.index()].mana = [10; 6];
        g.state.step = Step::Upkeep;
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.step = Step::Main1;
        let shock = packet(&mut g, p, "Shock", None, &[Target::Player(p.other())]);
        let item = g.state.stack.iter().find(|x| x.card == shock).unwrap();
        assert!(item.cast_creature_subjects.is_none());
        assert!(
            serde_json::to_value(item)
                .unwrap()
                .get("cast_creature_subjects")
                .is_none()
        );
    }
}
#[test]
fn msh_storm_payload_heap_bytes_both_views_orphan_and_library_privacy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let hidden = named(&mut g, p, "Murder", Zone::Hand);
        packet(&mut g, p, "Shock", None, &[Target::Card(a)]);
        let item = storm_packet(&g, s);
        let mut without = item.clone();
        without.cast_creature_subjects = None;
        assert_eq!(
            item.owned_bytes() - without.owned_bytes(),
            std::mem::size_of::<CastCreatureSubjectsView>() + std::mem::size_of::<ObjectRef>()
        );
        for viewer in [Player::P0, Player::P1] {
            let observation = serde_json::to_value(g.observe(viewer)).unwrap();
            let serialized = observation.to_string();
            assert!(serialized.contains("cast_creature_subjects"));
        }

        assert_eq!(g.state.objects[hidden as usize].zone, Zone::Hand);
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|view| view.reference.card == hidden)
        );
        let mut other = g.clone();
        other.state.objects[hidden as usize].card = Card::from_name("Negate").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(other.observe(p.other())).unwrap()
        );
        g.prepare_triggers();
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            .unwrap();
        g = orphan(g, action);
        settle(&mut g);
        assert!(granted(&g, a));
    }
}
#[test]
fn msh_storm_unrelated_static_provider_does_not_restrict_flying_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "The Ten Rings");
        let flyer = paid(&mut g, p.other(), "Serra Angel");
        assert!(g.can_attack_defender(flyer, CombatDefender::Player(p)));
        assert!(!g.can_block_pair(flyer, flyer));
    }
}

#[test]
fn msh_storm_copied_trigger_owns_same_nontargets_and_cannot_retarget_them() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let echo = paid(&mut g, p, "Echo, Perceptive Prodigy");
        age(&mut g, p);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        packet(&mut g, p, "Shock", None, &[Target::Card(a)]);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|i| i.card == s && matches!(i.kind, StackKind::TriggeredAbility { .. }))
        });
        let original = storm_packet(&g, s).clone();
        activation(&mut g, echo, &[Target::Stack(original.id)]);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|item| item.card == s && item.is_copy)
        });
        let copy = g
            .state
            .stack
            .iter()
            .find(|item| item.card == s && item.is_copy)
            .unwrap();
        assert!(copy.is_copy);
        assert_ne!(copy.id, original.id);
        assert_eq!(copy.cast_creature_subjects, original.cast_creature_subjects);
        assert!(copy.targets.is_empty());
        assert!(g.retarget_view().is_none());
        reject(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(a),
            },
        );
        g = orphan(g, Action::Pass);
        settle(&mut g);
        let count = g
            .state
            .roles
            .as_ref()
            .unwrap()
            .keyword_effects
            .iter()
            .filter(|r| r.recipient == g.object_ref(a) && r.keywords.contains(Keywords::FLYING))
            .count();
        assert_eq!(count, 2);
    }
}
#[test]
fn msh_storm_target_later_noncreature_still_gains_flying_bounded_copy_transition() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, STORM);
        let mascot = paid(&mut g, p, "Three Tree Mascot");
        packet(&mut g, p, "Shock", None, &[Target::Card(mascot)]);
        // Bounded characteristic transition: Shuri is sorcery-only, so no paid
        // instant-speed existing-object copy producer is claimed for this boundary.
        let reference = g.object_ref(mascot);
        let recipe = CopyRecipe::new(Card::from_name("Arc Reactor").unwrap());
        g.apply_temporary_copy(reference, recipe, CopyDeadline::EndOfTurn);
        resolve_grant(&mut g, mascot);
        assert!(!g.is_creature(mascot));
        assert_eq!(g.current_kinds(mascot) & 16, 16);
        assert!(granted(&g, mascot));
        settle(&mut g);
        assert_eq!(g.object_view(mascot).damage, 0);
    }
}

#[test]
fn msh_storm_source_blank_after_cast_keeps_queued_grant_but_not_restriction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        packet(&mut g, p, "Shock", None, &[Target::Card(a)]);
        // Bounded exact continuous-layer transition while the already acquired trigger waits.
        let recipe = CopyRecipe::new(Card::from_name("Arc Reactor").unwrap());
        let source = g.object_ref(s);
        g.apply_temporary_copy(source, recipe, CopyDeadline::EndOfTurn);
        settle(&mut g);
        assert!(granted(&g, a));
        assert!(!g.is_creature(s));
        assert!(g.can_attack_defender(a, CombatDefender::Player(p)));
    }
}
#[test]
fn msh_storm_current_flying_and_control_rules_change_declaration_not_existing_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let a = paid(&mut g, p.other(), "Hill Giant");
        combat(&mut g, p.other());
        checked(&mut g, Action::Attack { card: a });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            g.state.pending == Pending::Priority && g.state.step == Step::Attackers
        });
        // The attacker was legally committed before this later resolution-created gain.
        let reference = g.object_ref(s);
        g.grant_keyword_effects(&[a], reference, Keywords::FLYING);
        assert!(g.state.attackers.iter().any(|x| x.card == a));
        assert!(!g.can_attack_defender(a, CombatDefender::Player(p)));
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.life(p), 17);
        let reference = g.object_ref(s);
        g.record_control(reference, p.other(), None);
        assert!(g.can_attack_defender(a, CombatDefender::Player(p)));
        assert!(!g.can_attack_defender(a, CombatDefender::Player(p.other())));
    }
}
#[test]
fn msh_storm_current_flying_gain_prevents_block_and_later_gain_does_not_unblock() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let a = paid(&mut g, p, "Hill Giant");
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        combat(&mut g, p);
        checked(&mut g, Action::Attack { card: a });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: a,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        let reference = g.object_ref(s);
        g.grant_keyword_effects(&[b], reference, Keywords::FLYING);
        assert!(!g.can_block_pair(b, a));
        assert!(
            g.state
                .blocks
                .iter()
                .any(|&(blocker, attacker)| blocker == b && attacker == a)
        );
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.life(p.other()), 20);
    }
}

#[test]
fn msh_storm_paid_flying_evasion_and_four_combat_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let ground = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        combat(&mut g, p);
        checked(&mut g, Action::Attack { card: s });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: s,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.life(p.other()), 16);
        assert!(g.object_view(s).tapped);
    }
}
#[test]
fn msh_storm_ability_targeting_creature_is_not_spell_cast_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = paid(&mut g, p, STORM);
        let arc = paid(&mut g, p, "Arc Reactor");
        let mascot = paid(&mut g, p, "Three Tree Mascot");
        let shuri = paid(&mut g, p, "Shuri, Wakandan Inventor");
        age(&mut g, p);
        activation(&mut g, shuri, &[Target::Card(mascot), Target::Card(arc)]);
        assert!(g.state.pending_triggers.iter().all(|x| x.card != s));
        settle(&mut g);
        assert!(!granted(&g, mascot));
    }
}
#[test]
fn msh_storm_non_target_grant_never_triggers_second_ward() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, STORM);
        let target = named(&mut g, p.other(), "Thornfist Striker", Zone::Battlefield);
        let shock = packet(&mut g, p, "Shock", None, &[Target::Card(target)]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::DeclineResolutionMana)
        });
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        assert!(granted(&g, target));
        assert_eq!(g.object_view(target).damage, 0);
    }
}
