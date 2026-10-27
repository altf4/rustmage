//! Whole physical Bilbo/Adventure wiring over atomic control and relational legality.
use super::*;
const BILBO: &str = "Bilbo, Luckwearer // Burglar's Plot";
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if ready(g) {
            return;
        }
        if let Some(r) = g.damage_assignment_range() {
            act(
                g,
                Action::AssignDamage {
                    target: r.target,
                    amount: r.minimum,
                },
            );
        } else {
            choose(g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
                )
            });
        }
    }
    panic!("Bilbo budget {:?} {:?}", g.state.step, g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn bilbo(g: &mut Game, p: Player, zone: Zone) -> CardId {
    put(g, p, BILBO, zone)
}
fn plot(g: &mut Game, p: Player, a: CardId, b: CardId) -> CardId {
    let s = bilbo(g, p, Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
    act(g, Action::AnnounceAdventure { card: s });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    reject(g, Action::FinishTargets);
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    let spell = g.state.stack.last().unwrap();
    assert_eq!(spell.source_card.name(), "Burglar's Plot");
    assert_eq!(spell.mana_value(), 5);
    s
}
fn private(g: &Game, p: Player) {
    for (id, _) in g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Hand && o.owner == p)
    {
        assert!(g.observe(p).objects.iter().any(|o| o.id == id as CardId));
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .all(|o| o.id != id as CardId)
        );
    }
}
fn loot(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    assert!(
        !g.legal_actions()
            .contains(&Action::ChooseOptional { accept: false })
    );
    choose(g, |a| matches!(a, Action::Discard { .. }));
}
fn damage(g: &mut Game, s: CardId, target: Target, amount: i128, combat: bool) {
    let source = g.damage_source(s);
    g.apply_damage_batch(&[DamageEvent {
        source,
        target,
        amount,
        combat,
        preventable: true,
    }]);
    g.prepare_triggers();
}
#[test]
fn hob_luckwearer_paid_front_real_unblockable_combat_and_mandatory_private_loot() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = bilbo(&mut g, p, Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, s, None);
        finish(&mut g);
        assert_eq!(stats(&g, s), (1, 1));
        assert!(g.has_subtype(s, "Halfling") && g.has_subtype(s, "Rogue"));
        assert!(g.object_view(s).is_legendary());
        assert_eq!(g.object_view(s).name(), "Bilbo, Luckwearer");
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: s });
        g.state.pending = Pending::Priority;
        g.state.step = Step::Cleanup;
        g.finish_cleanup();
        g.state.step = Step::Cleanup;
        g.finish_cleanup();
        assert_eq!(g.state.active, p);
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: s });
        act(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: s,
            },
        );
        act(&mut g, Action::FinishBlockers);
        loot(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.hand_count(p), 0);
    }
}
#[test]
fn hob_luckwearer_zero_noncombat_and_creature_damage_do_not_loot() {
    let p = Player::P0;
    let mut g = scenario(p);
    let s = bilbo(&mut g, p, Zone::Battlefield);
    let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    for (target, amount, combat) in [
        (Target::Player(p.other()), 0, true),
        (Target::Player(p.other()), 1, false),
        (Target::Card(bear), 1, true),
    ] {
        damage(&mut g, s, target, amount, combat);
        finish(&mut g);
    }
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    assert_eq!(g.state.players[p.other().index()].life, 19);
}
#[test]
fn hob_luckwearer_combat_packet_controller_source_departure_staff_and_ability_suppression() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = bilbo(&mut g, p.other(), Zone::Battlefield);
        g.change_controller(g.object_ref(s), p);
        let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
        g.state.objects[staff as usize].attached_to = Some(g.object_ref(s));
        damage(&mut g, s, Target::Player(p.other()), 2, true);
        g.move_object(s, Zone::Exile);
        loot(&mut g);
        loot(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
        let s = bilbo(&mut g, p, Zone::Battlefield);
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(s));
        assert!(!g.keywords(s).contains(Keywords::CANT_BE_BLOCKED));
        damage(&mut g, s, Target::Player(p.other()), 1, true);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}
#[test]
fn hob_luckwearer_plot_exact_two_nonland_type_legality_and_unfunded_rejections() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = bilbo(&mut g, p, Zone::Hand);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let wrong = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        reject(&mut g, Action::AnnounceAdventure { card: s });
        let b = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceAdventure { card: s });
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
        act(&mut g, Action::AnnounceAdventure { card: s });
        for id in [land, wrong] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        reject(&mut g, Action::FinishTargets);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(wrong),
            },
        );
        contracts::branch_contract(g.clone());
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(b),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(a).controller, p.other());
        assert_eq!(g.object_view(b).controller, p);
    }
}
#[test]
fn hob_luckwearer_shared_card_type_not_subtype_supertype_or_color_and_multitype_pair() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Adaptive Automaton", Zone::Battlefield);
    let s = plot(&mut g, p, a, b);
    finish(&mut g);
    assert_eq!(g.object_view(a).controller, p.other());
    assert_eq!(g.object_view(b).controller, p);
    assert_eq!(g.object_view(s).zone, Zone::Exile);
    let mut g = scenario(p);
    let body = bilbo(&mut g, p, Zone::Battlefield);
    let ring = put(&mut g, p.other(), "The Arkenstone", Zone::Battlefield);
    let s = bilbo(&mut g, p, Zone::Hand);
    assert!(g.object_view(body).is_legendary() && g.object_view(ring).is_legendary());
    reject(&mut g, Action::AnnounceAdventure { card: s });
}
#[test]
fn hob_luckwearer_same_controller_targets_legal_no_exchange_and_adventure_can_cast_body() {
    for p in [Player::P0, Player::P1] {
        for controller in [p, p.other()] {
            let mut g = scenario(p);
            let a = put(&mut g, controller, "Bear Cub", Zone::Battlefield);
            let b = put(&mut g, controller, "Goblin Piker", Zone::Battlefield);
            let s = plot(&mut g, p, a, b);
            finish(&mut g);
            assert_eq!(g.object_view(a).controller, controller);
            assert_eq!(g.object_view(b).controller, controller);
            assert_eq!(g.object_view(s).zone, Zone::Exile);
            assert_eq!(g.object_view(s).card.name(), "Bilbo, Luckwearer");
            reject(&mut g, Action::AnnounceAdventure { card: s });
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
            act(&mut g, Action::AnnounceExile { card: s });
            pay(&mut g);
            finish(&mut g);
            assert_eq!(stats(&g, s), (1, 1));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn hob_luckwearer_one_illegal_target_exchange_is_atomic_but_adventure_resolves_to_exile() {
    for p in [Player::P0, Player::P1] {
        for changed in 0..3 {
            let mut g = scenario(p);
            let a = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            g.change_object_form(
                g.object_ref(a),
                crate::cards::AnimationSpec {
                    artifact: false,
                    creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                    base: Some((2, 2)),
                    keywords: Keywords::NONE,
                    duration: crate::cards::AnimationDuration::Permanent,
                },
                characteristic_layers::AnimationTypeMode::AddCreature,
            );
            assert_eq!(g.current_kinds(a), 17);
            let b = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
            let s = plot(&mut g, p, a, b);
            match changed {
                0 => {
                    g.move_object(a, Zone::Hand);
                }
                1 => {
                    g.move_object(a, Zone::Graveyard);
                    g.move_object(a, Zone::Battlefield);
                }
                _ => {
                    g.state.objects[b as usize].gained_keywords = Keywords::HEXPROOF;
                }
            }
            if changed < 2 {
                let lki = g
                    .state
                    .stack
                    .last()
                    .unwrap()
                    .target_kinds_lki
                    .as_ref()
                    .unwrap();
                assert!(lki.iter().any(|(_, k)| *k == 17));
            }
            finish(&mut g);
            assert_eq!(g.object_view(b).controller, p.other());
            assert_eq!(g.object_view(s).zone, Zone::Exile);
            if changed != 0 {
                assert_eq!(g.object_view(a).controller, p);
            }
        }
    }
}
#[test]
fn hob_luckwearer_both_illegal_or_no_longer_shared_card_type_does_not_resolve_adventure() {
    for p in [Player::P0, Player::P1] {
        for type_change in [false, true] {
            let mut g = scenario(p);
            let a = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            let b = put(&mut g, p.other(), "Adaptive Automaton", Zone::Battlefield);
            let s = plot(&mut g, p, a, b);
            if type_change {
                let aura = put(&mut g, p, "Witness Protection", Zone::Battlefield);
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(b));
                assert_eq!(g.current_kinds(b), 1);
            } else {
                g.move_object(a, Zone::Hand);
                g.move_object(b, Zone::Hand);
            }
            finish(&mut g);
            assert_eq!(g.object_view(s).zone, Zone::Graveyard);
            assert!(
                !g.legal_actions()
                    .contains(&Action::AnnounceCast { card: s })
            );
            assert_eq!(g.object_view(b).controller, p.other());
        }
    }
}
#[test]
fn hob_luckwearer_becoming_land_invalidates_only_that_target_and_preserves_partner_legality() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
    let s = plot(&mut g, p, a, b);
    let aura = put(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(a));
    assert_eq!(g.current_kinds(a), 2);
    finish(&mut g);
    assert_eq!(
        g.object_view(s).zone,
        Zone::Graveyard,
        "neither current target shares a card type after the type change"
    );
    assert_eq!(g.object_view(b).controller, p.other());
}
#[test]
fn hob_luckwearer_exchange_uses_resolution_controllers_and_persists_after_types_or_source_leave() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        let s = plot(&mut g, p, a, b);
        g.record_control(g.object_ref(a), p.other(), None);
        g.record_control(g.object_ref(b), p, None);
        finish(&mut g);
        assert_eq!(g.object_view(a).controller, p);
        assert_eq!(g.object_view(b).controller, p.other());
        g.move_object(s, Zone::Hand);
        let aura = put(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(a));
        g.state.step = Step::Cleanup;
        g.finish_cleanup();
        assert_eq!(g.object_view(a).controller, p);
        assert_eq!(g.object_view(b).controller, p.other());
        g.move_object(a, Zone::Exile);
        g.move_object(a, Zone::Battlefield);
        assert_eq!(g.object_view(a).controller, p);
        assert_eq!(g.object_view(b).controller, p.other());
    }
}
#[test]
fn hob_luckwearer_exchange_same_controller_at_resolution_does_not_create_new_control_effect() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
    plot(&mut g, p, a, b);
    g.record_control(g.object_ref(b), p, None);
    let count = g.state.controls.as_ref().unwrap().effects.len();
    finish(&mut g);
    assert_eq!(g.state.controls.as_ref().unwrap().effects.len(), count);
    assert_eq!(g.object_view(a).controller, p);
    assert_eq!(g.object_view(b).controller, p);
}
#[test]
fn hob_luckwearer_fixed_exchange_supersedes_older_temporary_control_and_later_effects_win() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
    g.record_control(g.object_ref(a), p, Some(g.state.turn));
    assert_eq!(g.object_view(a).controller, p);
    plot(&mut g, p, a, b);
    finish(&mut g);
    assert_eq!(g.object_view(a).controller, p.other());
    assert_eq!(g.object_view(b).controller, p);
    g.state.step = Step::Cleanup;
    g.finish_cleanup();
    assert_eq!(g.object_view(a).controller, p.other());
    assert_eq!(g.object_view(b).controller, p);
    g.record_control(g.object_ref(b), p.other(), None);
    assert_eq!(g.object_view(b).controller, p.other());
}
#[test]
fn hob_luckwearer_exchange_aura_dependencies_and_equipment_remain_independent() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
    let pick = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
    g.state.objects[pick as usize].attached_to = Some(g.object_ref(a));
    let s = plot(&mut g, p, a, b);
    finish(&mut g);
    assert_eq!(g.object_view(a).controller, p.other());
    assert_eq!(g.object_view(b).controller, p);
    assert_eq!(g.object_view(pick).controller, p);
    assert_eq!(g.object_view(pick).attached_to, Some(g.object_ref(a)));
    assert_eq!(g.object_view(s).zone, Zone::Exile);
    let x = put(&mut g, p, "Confiscate", Zone::Battlefield);
    g.state.objects[x as usize].attached_to = Some(g.object_ref(a));
    g.refresh_controllers();
    assert_eq!(g.object_view(a).controller, p);
    let y = put(&mut g, p.other(), "Confiscate", Zone::Battlefield);
    g.state.objects[y as usize].attached_to = Some(g.object_ref(b));
    g.refresh_controllers();
    assert_eq!(g.object_view(b).controller, p.other());
    plot(&mut g, p, x, y);
    finish(&mut g);
    assert_eq!(g.object_view(x).controller, p.other());
    assert_eq!(g.object_view(y).controller, p);
    assert_eq!(g.object_view(a).controller, p.other());
    assert_eq!(g.object_view(b).controller, p);
}
#[test]
fn hob_luckwearer_loot_private_raw_index_restore_and_miracle_pause() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = bilbo(&mut g, p, Zone::Battlefield);
        put(&mut g, p, "Shock", Zone::Hand);
        put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let old = g.state.players[p.index()].library.clone();
        for id in old {
            g.move_object(id, Zone::Outside);
        }
        put(&mut g, p, "Shock", Zone::Library);
        damage(&mut g, s, Target::Player(p.other()), 1, true);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::MiracleReveal { .. })
        });
        assert!(g.observe(p).miracle.is_some());
        assert!(g.observe(p.other()).miracle.is_none());
        g.move_object(s, Zone::Exile);
        act(&mut g, Action::DeclineMiracle);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        private(&g, p);
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let actor = g.actor().unwrap();
        let rev = g.revision();
        let actions = g.legal_actions();
        let i = actions
            .iter()
            .position(|a| matches!(a, Action::Discard { .. }))
            .unwrap();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        act(&mut raw, actions[i].clone());
        indexed.apply_index(actor, rev, i).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        finish(&mut raw);
        finish(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        let expected = raw.snapshot();
        raw.restore(&saved).unwrap();
        act(&mut raw, actions[i].clone());
        finish(&mut raw);
        assert_eq!(raw.snapshot(), expected);
    }
}
#[test]
fn hob_luckwearer_stale_relation_owned_snapshot_replay_and_private_lki_wire_omission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        let s = plot(&mut g, p, a, b);
        let old = g.object_ref(a);
        g.move_object(a, Zone::Hand);
        g.move_object(a, Zone::Battlefield);
        assert!(
            g.state
                .stack
                .last()
                .unwrap()
                .target_kinds_lki
                .as_ref()
                .unwrap()
                .iter()
                .any(|(r, _)| *r == old)
        );
        assert!(
            serde_json::to_value(g.state.stack.last().unwrap())
                .unwrap()
                .get("target_kinds_lki")
                .is_none()
        );
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut child = g.clone();
        drop(g);
        finish(&mut child);
        assert_eq!(child.object_view(s).zone, Zone::Exile);
        assert_eq!(child.object_view(b).controller, p.other());
        let expected = child.snapshot();
        child.restore(&saved).unwrap();
        finish(&mut child);
        assert_eq!(child.snapshot(), expected);
    }
}
#[test]
fn hob_luckwearer_actual_paid_spell_copy_retarget_preserves_pair_constraint_and_no_extra_adventure_right()
 {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
    let c = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
    let d = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
    let s = plot(&mut g, p, a, b);
    let copy = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 0];
    act(&mut g, Action::AnnounceCast { card: copy });
    act(&mut g, Action::ChooseMode { mode: 0 });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(s),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::Retarget { .. })
    });
    act(
        &mut g,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(c),
        },
    );
    reject(&mut g, Action::FinishStackTargets);
    act(
        &mut g,
        Action::SetStackTarget {
            index: 1,
            target: Target::Card(d),
        },
    );
    contracts::branch_contract(g.clone());
    act(&mut g, Action::FinishStackTargets);
    finish(&mut g);
    assert_eq!(g.object_view(a).controller, p.other());
    assert_eq!(g.object_view(b).controller, p);
    assert_eq!(g.object_view(c).controller, p.other());
    assert_eq!(g.object_view(d).controller, p);
    assert_eq!(g.object_view(s).zone, Zone::Exile);
    assert_eq!(
        g.observe(p)
            .adventures
            .iter()
            .filter(|r| r.object.card == s)
            .count(),
        1
    );
}
#[test]
fn hob_luckwearer_countered_plot_has_no_adventure_permission_and_exile_reentry_clears_right() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        let s = plot(&mut g, p, a, b);
        act(&mut g, Action::Pass);
        let n = put(&mut g, p.other(), "Negate", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, n, Some(Target::Spell(s)));
        finish(&mut g);
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        assert!(!g.can_play_adventure(p, s));
        assert_eq!(g.object_view(a).controller, p);
        assert_eq!(g.object_view(b).controller, p.other());
        g.state.priority = p;
        g.state.active = p;
        let s = plot(&mut g, p, a, b);
        finish(&mut g);
        assert!(g.can_play_adventure(p, s));
        let old = g.object_ref(s);
        g.move_object(s, Zone::Hand);
        g.move_object(s, Zone::Exile);
        assert_ne!(old, g.object_ref(s));
        assert!(!g.can_play_adventure(p, s));
        g.state.players[p.index()].mana = [8; 6];
        reject(&mut g, Action::AnnounceExile { card: s });
    }
}
#[test]
fn hob_luckwearer_late_loot_failure_rolls_back_action_and_keeps_frozen_damage_trigger() {
    let p = Player::P0;
    let mut g = scenario(p);
    let s = bilbo(&mut g, p, Zone::Battlefield);
    damage(&mut g, s, Target::Player(p.other()), 1, true);
    until(&mut g, |g| !g.state.stack.is_empty());
    g.state.players[p.index()].cards_drawn_this_turn = u64::MAX;
    act(&mut g, Action::Pass);
    let saved = g.snapshot();
    let rev = g.revision();
    let actor = g.actor().unwrap();
    assert!(matches!(
        g.apply(actor, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
    assert_eq!(g.state.players[p.other().index()].life, 19);
    assert_eq!(g.state.stack.last().unwrap().controller, p);
}
#[test]
fn hob_luckwearer_existing_different_controller_lki_retains_departed_exact_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        let old = g.object_ref(a);
        let s = put(&mut g, p, "Run Away Together", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceCast { card: s });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(b),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        g.move_object(a, Zone::Exile);
        g.move_object(a, Zone::Battlefield);
        g.change_controller(g.object_ref(a), p.other());
        let stack = g.state.stack.last().unwrap();
        assert!(
            stack
                .target_controller_lki
                .as_ref()
                .unwrap()
                .iter()
                .any(|(r, controller)| *r == old && *controller == p)
        );
        assert!(stack.target_kinds_lki.is_none());
        finish(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        assert_eq!(g.object_view(a).controller, p.other());
        assert_eq!(g.object_view(b).zone, Zone::Hand);
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        assert_eq!(g.object_view(a).owner, p);
        assert_eq!(g.object_view(b).owner, p.other());
    }
}
#[test]
fn hob_luckwearer_actual_target_treasure_mana_payment_keeps_announcement_lki_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(
            &mut g,
            p,
            "@token/b60f6cd3b44b22bd289fe4e2",
            Zone::Battlefield,
        );
        let b = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let s = bilbo(&mut g, p, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        act(&mut g, Action::AnnounceAdventure { card: s });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(b),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        let old = g.object_ref(a);
        act(
            &mut g,
            Action::ActivateMana {
                card: a,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        assert_ne!(g.object_view(a).zone, Zone::Battlefield);
        assert!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .target_kinds_lki
                .as_ref()
                .unwrap()
                .iter()
                .any(|(r, k)| *r == old && *k == 16)
        );
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        assert_eq!(g.snapshot(), saved);
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        finish(&mut g);
        assert_eq!(g.object_view(b).controller, p.other());
        assert_eq!(g.object_view(s).zone, Zone::Exile);
    }
}
