//! Independent paid outcomes for five inherited HOB artifacts; no definition changes.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("artifact decision budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    for _ in 0..48 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            )
        });
    }
    panic!("artifact resolution budget {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn bind(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    if g.legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        pay(g);
    }
}
fn announce(g: &mut Game, id: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    for id in g.state.players[p.index()].library.iter().copied() {
        g.state.objects[id as usize].zone = Zone::Outside;
    }
    g.state.players[p.index()].library.clear();
    names
        .iter()
        .map(|name| put(g, p, name, Zone::Library))
        .collect()
}
fn private(g: &Game, p: Player, ids: &[CardId]) {
    for &id in ids {
        assert!(g.observe(p).objects.iter().any(|o| o.id == id));
        assert!(g.observe(p.other()).objects.iter().all(|o| o.id != id));
    }
}
#[test]
fn hob_inherited_artifacts_four_paid_equips_keep_stats_control_restrictions_and_timing() {
    for p in [Player::P0, Player::P1] {
        for (name, cost, expected) in [
            ("Crude Bent Blade", 2, (4, 3)),
            ("Dwarven Mattock", 3, (4, 4)),
            ("Ragged Short Spear", 3, (4, 2)),
            ("Well-Worn Spatula", 1, (3, 3)),
        ] {
            let mut g = scenario(p);
            let equipment = put(&mut g, p, name, Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[equipment as usize].owner = p.other();
            g.state.objects[bear as usize].owner = p.other();
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, cost - 1];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: equipment,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, cost];
            g.state.active = p.other();
            g.state.priority = p;
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: equipment,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.active = p;
            g.state.priority = p;
            announce(&mut g, equipment);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            bind(&mut g, Target::Card(bear));
            finish(&mut g);
            assert_eq!(
                g.object_view(equipment).attached_to,
                Some(g.object_ref(bear)),
                "{name}"
            );
            assert_eq!(stats(&g, bear), expected, "{name}");
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            g.move_object(equipment, Zone::Graveyard);
            assert_eq!(stats(&g, bear), (2, 2));
            assert_eq!(g.state.objects[equipment as usize].owner, p.other());
        }
    }
}
#[test]
fn hob_inherited_artifacts_blade_targeted_opponent_chooses_controlled_creature_not_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mine = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let stolen = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[stolen as usize].owner = p;
        let spared = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let blade = paid(&mut g, p, "Crude Bent Blade", [0, 0, 1, 0, 0, 2]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        bind(&mut g, Target::Player(p.other()));
        g.move_object(blade, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        assert_eq!(g.actor(), Some(p.other()));
        let revision = g.revision();
        let before = g.snapshot();
        assert_eq!(
            g.apply(
                p,
                Action::Sacrifice {
                    object: g.object_ref(stolen)
                }
            ),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        let own_ref = g.object_ref(mine);
        reject(&mut g, Action::Sacrifice { object: own_ref });
        let land_ref = g.object_ref(land);
        reject(&mut g, Action::Sacrifice { object: land_ref });
        contracts::branch_contract(g.clone());
        let object = g.object_ref(stolen);
        act(&mut g, Action::Sacrifice { object });
        finish(&mut g);
        assert_eq!(g.object_view(stolen).zone, Zone::Graveyard);
        assert_eq!(g.object_view(stolen).owner, p);
        assert_eq!(g.object_view(spared).zone, Zone::Battlefield);
        assert_eq!(g.object_view(mine).zone, Zone::Battlefield);
    }
}
#[test]
fn hob_inherited_artifacts_blade_empty_opponent_and_stale_equip_target_do_not_redirect() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blade = paid(&mut g, p, "Crude Bent Blade", [0, 0, 1, 0, 0, 2]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        bind(&mut g, Target::Player(p.other()));
        finish(&mut g);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut g, blade);
        bind(&mut g, Target::Card(bear));
        let old = g.object_ref(bear);
        g.move_object(bear, Zone::Hand);
        g.move_object(bear, Zone::Battlefield);
        assert_ne!(g.object_ref(bear), old);
        finish(&mut g);
        assert_eq!(g.object_view(blade).attached_to, None);
        assert_eq!(stats(&g, bear), (2, 2));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn hob_inherited_artifacts_mattock_automatic_attachment_only_targets_own_dwarf() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = scenario(p);
            let dwarf = put(&mut g, p, "Axgard Cavalry", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Axgard Cavalry", Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let mattock = paid(&mut g, p, "Dwarven Mattock", [0, 0, 0, 0, 0, 2]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            bind(&mut g, Target::Card(dwarf));
            if stale {
                g.move_object(dwarf, Zone::Hand);
                g.move_object(dwarf, Zone::Battlefield);
            }
            finish(&mut g);
            assert_eq!(
                g.object_view(mattock).attached_to,
                if stale {
                    None
                } else {
                    Some(g.object_ref(dwarf))
                }
            );
            assert_eq!(stats(&g, dwarf), if stale { (2, 2) } else { (4, 4) });
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mattock = paid(&mut g, p, "Dwarven Mattock", [0, 0, 0, 0, 0, 2]);
        finish(&mut g);
        assert_eq!(g.object_view(mattock).attached_to, None);
    }
}
#[test]
fn hob_inherited_artifacts_mattock_grants_ward_one_to_recipient_and_queued_ward_survives_source() {
    for p in [Player::P0, Player::P1] {
        for pay_ward in [false, true] {
            let mut g = scenario(p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let mattock = put(&mut g, p, "Dwarven Mattock", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            announce(&mut g, mattock);
            bind(&mut g, Target::Card(bear));
            finish(&mut g);
            assert_eq!(stats(&g, bear), (4, 4));
            let own = put(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            cast(&mut g, own, Some(Target::Card(bear)));
            finish(&mut g);
            assert_eq!(
                g.state.objects[bear as usize].damage, 2,
                "own spell does not invoke ward"
            );
            g.state.objects[bear as usize].damage = 0;
            g.state.priority = p.other();
            let spell = put(&mut g, p.other(), "Shock", Zone::Hand);
            g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 1];
            cast(&mut g, spell, Some(Target::Card(bear)));
            if !pay_ward {
                g.move_object(mattock, Zone::Graveyard);
            }
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            assert_eq!(g.actor(), Some(p.other()));
            assert_eq!(
                g.observe(p.other())
                    .resolution_payment
                    .unwrap()
                    .cost
                    .generic,
                1
            );
            reject(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 2],
                },
            );
            if pay_ward {
                act(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 1],
                    },
                );
            } else {
                act(&mut g, Action::DeclineResolutionMana);
            }
            finish(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
            assert_eq!(
                g.state.objects[bear as usize].damage,
                if pay_ward { 2 } else { 0 }
            );
            assert_eq!(
                g.state.players[p.other().index()].mana,
                [0, 0, 0, 0, 0, u32::from(!pay_ward)]
            );
        }
    }
}
#[test]
fn hob_inherited_artifacts_spear_optional_discard_precedes_two_private_draws_and_empty_hand_declines()
 {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let discard = (mode != 2).then(|| put(&mut g, p, "Forest", Zone::Hand));
            let enemy = put(&mut g, p.other(), "Shock", Zone::Hand);
            let drawn = [
                put(&mut g, p, "Island", Zone::Library),
                put(&mut g, p, "Forest", Zone::Library),
            ];
            let spear = paid(&mut g, p, "Ragged Short Spear", [0, 0, 0, 1, 0, 1]);
            if let Some(discard) = discard {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectChoice { .. })
                });
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
                private(&g, p, &[discard]);
                reject(&mut g, Action::Discard { card: enemy });
                reject(&mut g, Action::Discard { card: spear });
                if mode == 1 {
                    contracts::branch_contract(g.clone());
                    act(&mut g, Action::Discard { card: discard });
                } else {
                    act(&mut g, Action::DeclineEffectPayment);
                }
            }
            finish(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                if mode == 1 { 2 } else { 0 }
            );
            if mode == 1 {
                private(&g, p, &drawn);
                assert_eq!(g.object_view(discard.unwrap()).zone, Zone::Graveyard);
            } else {
                assert!(
                    drawn
                        .iter()
                        .all(|id| g.object_view(*id).zone == Zone::Library)
                );
            }
        }
    }
}
#[test]
fn hob_inherited_artifacts_map_search_reveals_only_selected_basic_and_shuffles_even_fail_to_find() {
    for p in [Player::P0, Player::P1] {
        for find in [false, true] {
            let mut g = scenario(p);
            let ids = library(
                &mut g,
                p,
                &["Shock", "Forest", "Island", "Goblin-town", "Bear Cub"],
            );
            let map = paid(&mut g, p, "Thrór's Map", [0, 0, 0, 0, 0, 2]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            assert!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .all(|o| !ids.contains(&o.id))
            );
            reject(&mut g, Action::ChooseSearch { card: ids[0] });
            reject(&mut g, Action::ChooseSearch { card: ids[3] });
            let before_rng = g.state.rng;
            if find {
                contracts::branch_contract(g.clone());
                act(&mut g, Action::ChooseSearch { card: ids[1] });
            }
            if g.legal_actions().contains(&Action::FinishSearch) {
                act(&mut g, Action::FinishSearch);
            }
            finish(&mut g);
            assert_ne!(g.state.rng, before_rng, "search always shuffles");
            let mut expected = ids.clone();
            if find {
                expected.retain(|id| *id != ids[1]);
            }
            expected.sort_unstable();
            let mut remaining = g.state.players[p.index()].library.to_vec();
            remaining.sort_unstable();
            assert_eq!(remaining, expected);
            assert_eq!(g.object_view(map).zone, Zone::Battlefield);
            if find {
                assert_eq!(g.object_view(ids[1]).zone, Zone::Hand);
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .any(|o| o.id == ids[1] && o.card == Card::Forest)
                );
            } else {
                assert!(
                    g.state.players[p.index()].library.contains(&ids[1]),
                    "qualified hidden-zone search may legally fail to find"
                );
            }
        }
    }
}
#[test]
fn hob_inherited_artifacts_map_two_and_tap_loots_at_instant_speed_after_paid_entry() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [false, true] {
            let mut g = scenario(p);
            let map = paid(&mut g, p, "Thrór's Map", [0, 0, 0, 0, 0, 2]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            act(&mut g, Action::FinishSearch);
            finish(&mut g);
            let drawn = put(&mut g, p, "Island", Zone::Library);
            let enemy = put(&mut g, p.other(), "Shock", Zone::Hand);
            g.state.active = p.other();
            g.state.priority = p;
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: map,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            announce(&mut g, map);
            reject(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 1, 0, 1],
                },
            );
            pay(&mut g);
            assert!(g.state.objects[map as usize].tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            if source_leaves {
                g.move_object(map, Zone::Graveyard);
            }
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            private(&g, p, &[drawn]);
            reject(&mut g, Action::Discard { card: enemy });
            act(&mut g, Action::Discard { card: drawn });
            finish(&mut g);
            assert_eq!(g.object_view(drawn).zone, Zone::Graveyard);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: map,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
        }
    }
}
#[test]
fn hob_inherited_artifacts_spatula_paid_entry_gains_two_even_after_source_departure() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [false, true] {
            let mut g = scenario(p);
            let e = paid(&mut g, p, "Well-Worn Spatula", [0, 0, 0, 0, 0, 1]);
            until(&mut g, |g| {
                g.state
                    .stack
                    .iter()
                    .any(|s| s.source.is_some_and(|source| source.id == e) && matches!(s.kind, StackKind::TriggeredAbility { .. }))
            });
            if source_leaves {
                g.move_object(e, Zone::Graveyard);
            }
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].life, 22);
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
}
