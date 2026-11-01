//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::PutTrigger { .. } | Action::Pass));
    }
    panic!("landfall choice budget");
}
fn drive(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::PutTrigger { .. } | Action::Pass));
    }
    panic!("landfall resolution budget");
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn land(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Forest", Zone::Hand);
    act(g, Action::PlayLand { card: id });
    id
}
fn extra_entry(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Forest", Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    g.give_priority();
    g.prepare_triggers();
    id
}
#[test]
fn hob_inherited_lands_paid_growth_each_own_entry_cleanup_and_exact_source() {
    {
        let p = Player::P0;
        for (name, mana, base, delta) in [
            ("Attercop", [0, 0, 0, 0, 1, 1], (2, 1), 1),
            ("Boughside Wanderers", [0, 0, 0, 0, 2, 4], (4, 4), 2),
        ] {
            let mut g = scenario(p);
            let source = paid(&mut g, p, name, mana);
            if name == "Boughside Wanderers" {
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::FinishLibrarySelection)
                });
                act(&mut g, Action::FinishLibrarySelection);
            }
            drive(&mut g);
            assert_eq!(stats(&g, source), base);
            if name == "Attercop" {
                assert!(
                    g.keywords(source)
                        .contains(Keywords::REACH.union(Keywords::DEATHTOUCH))
                );
            }
            land(&mut g, p);
            if name == "Attercop" {
                let mut stale = g.clone();
                stale.move_object(source, Zone::Hand);
                stale.move_object(source, Zone::Battlefield);
                stale.give_priority();
                stale.prepare_triggers();
                drive(&mut stale);
                assert_eq!(
                    stats(&stale, source),
                    base,
                    "old growth cannot bind a new incarnation"
                );
            }
            drive(&mut g);
            assert_eq!(stats(&g, source), (base.0 + delta, base.1 + delta));
            extra_entry(&mut g, p);
            drive(&mut g);
            assert_eq!(stats(&g, source), (base.0 + 2 * delta, base.1 + 2 * delta));
            extra_entry(&mut g, p.other());
            drive(&mut g);
            assert_eq!(stats(&g, source), (base.0 + 2 * delta, base.1 + 2 * delta));
            g.finish_cleanup();
            assert_eq!(stats(&g, source), base);
        }
    }
}

#[test]
fn hob_inherited_lands_meditator_optional_base_setting_layers_and_expires() {
    {
        let p = Player::P0;
        for accept in [false, true] {
            let mut g = scenario(p);
            let source = paid(&mut g, p, "Mirkwood Meditator", [0, 1, 0, 0, 0, 2]);
            drive(&mut g);
            assert_eq!(stats(&g, source), (2, 4));
            g.state.objects[source as usize].counters.plus_one = 1;
            extra_entry(&mut g, p.other());
            drive(&mut g);
            assert_eq!(stats(&g, source), (3, 5));
            land(&mut g, p);
            until(&mut g, |g| {
                g.legal_actions()
                    .contains(&Action::ChooseOptional { accept })
            });
            act(&mut g, Action::ChooseOptional { accept });
            drive(&mut g);
            let expected = if accept { (5, 3) } else { (3, 5) };
            assert_eq!(stats(&g, source), expected);
            extra_entry(&mut g, p);
            until(&mut g, |g| {
                g.legal_actions()
                    .contains(&Action::ChooseOptional { accept })
            });
            act(&mut g, Action::ChooseOptional { accept });
            drive(&mut g);
            assert_eq!(
                stats(&g, source),
                expected,
                "base setting does not add another4/2"
            );
            let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
            cast(&mut g, growth, Some(Target::Card(source)));
            drive(&mut g);
            assert_eq!(stats(&g, source), (expected.0 + 3, expected.1 + 3));
            g.finish_cleanup();
            assert_eq!(
                stats(&g, source),
                (3, 5),
                "counter persists; base setting and growth expire"
            );
        }
    }
}

#[test]
fn hob_inherited_lands_pathmaker_cda_all_zones_live_control_and_owned_views() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let one = put(&mut g, p, "Forest", Zone::Battlefield);
        let two = put(&mut g, p, "Forest", Zone::Battlefield);
        let steal = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let source = put(&mut g, p, "Mirkwood Pathmaker", Zone::Hand);
        assert_eq!(stats(&g, source), (2, 2));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        cast(&mut g, source, None);
        assert_eq!(g.object_view(source).zone, Zone::Stack);
        assert_eq!(stats(&g, source), (2, 2));
        drive(&mut g);
        assert_eq!(stats(&g, source), (2, 2));
        land(&mut g, p);
        drive(&mut g);
        assert_eq!(stats(&g, source), (3, 3));
        let parent = g.clone();
        let steal_ref = g.object_ref(steal);
        g.record_control(steal_ref, p, None);
        assert_eq!(stats(&g, source), (4, 4));
        assert_eq!(stats(&parent, source), (3, 3));
        let one_ref = g.object_ref(one);
        g.record_control(one_ref, p.other(), None);
        assert_eq!(stats(&g, source), (3, 3));
        let source_ref = g.object_ref(source);
        g.record_control(source_ref, p.other(), None);
        assert_eq!(stats(&g, source), (2, 2), "use current controller's lands");
        g.record_control(source_ref, p, None);
        assert_eq!(stats(&g, source), (3, 3));
        g.move_object(two, Zone::Hand);
        g.give_priority();
        assert_eq!(stats(&g, source), (2, 2));
        g.move_object(source, Zone::Hand);
        assert_eq!(stats(&g, source), (2, 2));
        g.move_object(source, Zone::Graveyard);
        assert_eq!(stats(&g, source), (2, 2));
        let mut zero = scenario(p);
        let zero_source = paid(&mut zero, p, "Mirkwood Pathmaker", [0, 0, 0, 0, 1, 2]);
        assert_eq!(stats(&zero, zero_source), (0, 0));
        drive(&mut zero);
        assert_eq!(zero.object_view(zero_source).zone, Zone::Graveyard);
        assert_eq!(stats(&zero, zero_source), (0, 0));
    }
}
