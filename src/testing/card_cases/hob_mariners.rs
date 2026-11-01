//! Focused printed outcomes; shared engine invariants live in core tests.
//! Both paid faces, exact batch blink targets and Adventure permission outcomes.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Mariners decision budget: {:?}", g.state.pending);
}
fn drive(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn target(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
}
fn start(g: &mut Game, p: Player) -> CardId {
    let spell = put(g, p, "Lake-town Mariners", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
    act(g, Action::AnnounceAdventure { card: spell });
    spell
}
fn finish(g: &mut Game, p: Player) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn permitted_body(g: &mut Game, p: Player, spell: CardId) {
    assert_eq!(g.object_view(spell).zone, Zone::Exile);
    assert!(g.can_play_adventure(p, spell));
    assert!(!g.can_play_adventure(p.other(), spell));
    g.state.active = p;
    g.state.step = Step::Main1;
    g.give_priority();
    reject(g, Action::AnnounceAdventure { card: spell });
    g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
    act(g, Action::AnnounceExile { card: spell });
    pay(g);
    drive(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, spell), (6, 5));
    assert!(g.keywords(spell).contains(Keywords::VIGILANCE));
    assert!(!g.can_play_adventure(p, spell));
}
#[test]
fn mariners_paid_normal_vigilance_attack_and_exact_ward_two_pay_or_decline() {
    {
        let p = Player::P0;
        for paying in [false, true] {
            let mut g = scenario(p);
            let body = put(&mut g, p, "Lake-town Mariners", Zone::Hand);
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
            cast(&mut g, body, None);
            drive(&mut g);
            assert_eq!(stats(&g, body), (6, 5));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            // Begin a later own combat; vigilance must leave the real attacker untapped.
            g.state.objects[body as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: body });
            act(&mut g, Action::FinishAttackers);
            assert!(!g.object_view(body).tapped);
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            act(&mut g, Action::FinishBlockers);
            until(&mut g, |g| g.state.step == Step::EndCombat);
            assert_eq!(g.state.players[p.other().index()].life, 14);
            assert!(!g.object_view(body).tapped);
            // Ordinary priority after combat isolates the opponent's ward payment.
            g.state.step = Step::Main2;
            g.state.pending = Pending::Priority;
            g.state.priority = p.other();
            let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
            g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 2];
            cast(&mut g, shock, Some(Target::Card(body)));
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            assert_eq!(g.actor(), Some(p.other()));
            reject(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 1],
                },
            );
            act(
                &mut g,
                if paying {
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 2],
                    }
                } else {
                    Action::DeclineResolutionMana
                },
            );
            drive(&mut g);
            assert_eq!(
                g.state.objects[body as usize].damage,
                if paying { 2 } else { 0 }
            );
            assert_eq!(
                g.state.players[p.other().index()].mana,
                [0, 0, 0, 0, 0, if paying { 0 } else { 2 }]
            );
            assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
            // Our own targeting does not trigger ward or charge two extra mana.
            g.state.priority = p;
            let own_shock = put(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            cast(&mut g, own_shock, Some(Target::Card(body)));
            drive(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.state.objects[body as usize].damage,
                if paying { 4 } else { 2 }
            );
        }
    }
}
#[test]
fn mariners_instant_two_distinct_creature_land_targets_all_kind_combinations() {
    {
        let p = Player::P0;
        for names in [
            ["Bear Cub", "Grizzly Bears"],
            ["Forest", "Swamp"],
            ["Bear Cub", "Forest"],
        ] {
            let mut g = scenario(p);
            let ids = names.map(|n| put(&mut g, p, n, Zone::Battlefield));
            let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            for id in ids {
                g.state.objects[id as usize].tapped = true;
            }
            g.add_counter(ids[0], CounterKind::PlusOne, 2);
            let refs = ids.map(|id| g.object_ref(id));
            g.state.active = p.other(); // Gone Fishing is an instant; the body is not.
            let spell = start(&mut g, p);
            for invalid in [foreign, artifact] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(invalid),
                    },
                );
            }
            reject(&mut g, Action::FinishTargets);
            target(&mut g, ids[0]);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(ids[0]),
                },
            );
            reject(&mut g, Action::FinishTargets);

            target(&mut g, ids[1]);
            finish(&mut g, p);
            drive(&mut g);
            for (i, id) in ids.into_iter().enumerate() {
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
                assert_eq!(g.object_view(id).controller, p);
                assert!(!g.object_view(id).tapped);
                assert_ne!(g.object_ref(id), refs[i]);
                assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
            }
            permitted_body(&mut g, p, spell);
        }
    }
}
#[test]
fn mariners_blink_returns_stolen_creature_to_owner_and_triggers_real_entries() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Dancing from Dark to Dawn", Zone::Battlefield);
        let stolen = put(
            &mut g,
            p.other(),
            "Dori, Bearer of Friends",
            Zone::Battlefield,
        );
        g.state.objects[stolen as usize].controller = p;
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let spell = start(&mut g, p);
        target(&mut g, stolen);
        target(&mut g, land);
        finish(&mut g, p);
        drive(&mut g);
        assert_eq!(g.object_view(stolen).controller, p.other());
        let count = |p: Player, subtype: &str| {
            g.state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype(subtype)
                })
                .count()
        };
        assert_eq!(count(p.other(), "Treasure"), 1);
        assert_eq!(count(p, "Treasure"), 0);
        assert_eq!(count(p, "Bear"), 1);
        assert_eq!(count(p.other(), "Bear"), 0);
        assert!(g.can_play_adventure(p, spell));
    }
}
