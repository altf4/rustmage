//! Focused printed outcomes; shared engine invariants live in core tests.
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
    assert_eq!(spell.source.unwrap().card.name(), "Burglar's Plot");
    assert_eq!(spell.mana_value(), 5);
    s
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
fn hob_luckwearer_plot_exact_two_nonland_type_legality_and_unfunded_rejections() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn hob_luckwearer_both_illegal_or_no_longer_shared_card_type_does_not_resolve_adventure() {
    {
        let p = Player::P0;
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
fn hob_luckwearer_exchange_uses_resolution_controllers_and_persists_after_types_or_source_leave() {
    {
        let p = Player::P0;
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
