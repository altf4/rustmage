//! Shared turn-casting rules with ordinary paid supported Saga producers.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
const ARMOR: &str = "Armor Wars";
const HULK: &str = "World War Hulk";
fn free(card: CardId) -> Action {
    Action::AnnounceFree {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}
fn grave_free(card: CardId) -> Action {
    Action::AnnounceFreeCounterGraveyard {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}
fn pay(g: &mut Game) {
    for _ in 0..25 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected grant payment {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("grant payment stalled")
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    let bytes = g.state_bytes();
    let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
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
    let index = g.legal_actions().iter().position(|b| b == &a).unwrap();
    let before = g.snapshot();
    let v = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), v);
    assert_eq!(
        raw.apply_index(actor, rev, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn grant(g: &mut Game, p: Player, n: u32) {
    for _ in 0..n {
        g.grant_next_creature_free(p)
    }
}
fn count(g: &Game, p: Player) -> u64 {
    g.player_cast_grants_view(p.index())
        .map_or(0, |r| r.next_red_or_green_creature_free)
}
fn source(g: &mut Game, p: Player, name: &str) -> CardId {
    assert!([ARMOR, HULK].contains(&name));
    let d = Card::from_name(name).unwrap();
    assert!(d.is_supported());
    let id = named(g, p, name, Zone::Hand);
    let (mut m, n) = d.cost();
    m[5] += n;
    g.state.players[p.index()].mana = m;
    cast(g, id, None, &[]);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn muldrotha_free(g: &Game, card: CardId) -> Action {
    g.legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::AnnounceFreeGraveyardPermanent {card:id,..} if *id==card))
        .unwrap()
}
#[test]
fn turn_cast_grants_supported_consumers_and_absent_wire_are_honest() {
    for p in [Player::P0, Player::P1] {
        let g = setup(p);
        assert!(Card::from_name(ARMOR).unwrap().is_supported());
        assert!(Card::from_name(HULK).unwrap().is_supported());
        for view in g.observe(p).players {
            assert!(view.turn_cast_grants.is_none());
            assert!(
                !serde_json::to_value(view)
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .contains_key("turn_cast_grants")
            );
        }
    }
}
#[test]
fn turn_cast_grants_public_captured_player_counts_stack_and_owned_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let before = g.snapshot();
        let bytes = g.state_bytes();
        g.grant_artifact_spell_reduction(p, 2);
        g.grant_artifact_spell_reduction(p, 3);
        grant(&mut g, p, 2);
        let expected = Some(PlayerCastGrantsView {
            artifact_generic_reduction: 5,
            next_red_or_green_creature_free: 2,
        });
        for viewer in [p, p.other()] {
            let v = g.observe(viewer);
            assert_eq!(v.players[p.index()].turn_cast_grants, expected);
            assert_eq!(v.players[p.other().index()].turn_cast_grants, None);
        }
        assert!(g.state_bytes() > bytes);
        let after = g.snapshot();
        let mut b = g.clone();
        drop(g);
        b.restore(&before).unwrap();
        assert_eq!(b.player_cast_grants_view(p.index()), None);
        b.restore(&after).unwrap();
        assert_eq!(b.player_cast_grants_view(p.index()), expected);
    }
}
#[test]
fn turn_cast_grants_armor_opcode_uses_queued_controller_after_source_departure_or_control_change() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let id = source(&mut g, p, ARMOR);
            let old = g.object_ref(id);
            assert!(g.queue_trigger(&g.object_view(id), 1));
            if depart {
                g.move_object(id, Zone::Graveyard);
                g.move_object(id, Zone::Battlefield);
                assert_ne!(g.object_ref(id), old);
            } else {
                g.state.objects[id as usize].controller = p.other();
            }
            settle(&mut g);
            assert_eq!(
                g.turn_artifact_spell_reduction(p, Card::from_name("Gilded Lotus").unwrap()),
                1
            );
            assert_eq!(g.player_cast_grants_view(p.other().index()), None);
        }
    }
}
#[test]
fn turn_cast_grants_hulk_opcode_survives_saga_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = source(&mut g, p, HULK);
        assert_eq!(count(&g, p), 1);
        // A resolved player grant survives the producer's later departure.
        g.move_object(id, Zone::Graveyard);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, free(bear));
        pay(&mut g);
        assert_eq!(count(&g, p), 0);
        settle(&mut g);
        assert!(g.on_battlefield(bear));
    }
}
#[test]
fn turn_cast_grants_artifact_discount_stacks_generic_only_and_real_payment_locks() {
    for p in [Player::P0, Player::P1] {
        for reduction in 0..=7 {
            let mut g = setup(p);
            g.grant_artifact_spell_reduction(p, reduction);
            let id = named(&mut g, p, "Gilded Lotus", Zone::Hand);
            let needed = 5u32.saturating_sub(reduction);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, needed];
            let before = g.snapshot();
            assert!(
                g.legal_actions()
                    .contains(&Action::AnnounceCast { card: id })
            );
            assert_eq!(g.snapshot(), before);
            checked(&mut g, Action::AnnounceCast { card: id });
            assert_eq!(g.announcement_payment_request(p).unwrap().generic, needed);
            reject(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 0, needed + 1],
                },
            );
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert!(g.on_battlefield(id));
        }
    }
}
#[test]
fn turn_cast_grants_nonartifact_color_and_caster_filters_do_not_receive_discount() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.grant_artifact_spell_reduction(p, 99);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: bear });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, bear, None, &[]);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().payment,
            [0, 0, 0, 0, 1, 1]
        );
        settle(&mut g);
        let iron = named(&mut g, p, "Ironheart, Clever Champion", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: iron });
        assert_eq!(
            g.spell_reduction_for_definition(
                p.other(),
                iron,
                5,
                Card::from_name("Ironheart, Clever Champion").unwrap()
            ),
            0
        );
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: iron });
        pay(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(iron));
    }
}
#[test]
fn turn_cast_grants_free_requires_real_color_creature_and_selected_face() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grant(&mut g, p, 1);
        g.state.players[p.index()].mana = [0; 6];
        for (name, allowed) in [
            ("Bear Cub", true),
            ("Shivan Dragon", true),
            ("Galewind Moose", true),
            ("Serra Angel", false),
            ("Gilded Lotus", false),
            ("Giant Growth", false),
            ("Frolicking Familiar", false),
        ] {
            let id = named(&mut g, p, name, Zone::Hand);
            assert_eq!(g.legal_actions().contains(&free(id)), allowed, "{name}");
            if !allowed {
                reject(&mut g, free(id));
            }
            if name == "Frolicking Familiar" {
                reject(
                    &mut g,
                    Action::AnnounceFree {
                        card: id,
                        face: CastFace::Adventure,
                        flash_surcharge: false,
                    },
                );
            }
        }
        let id = named(&mut g, p, "Monica Rambeau", Zone::Hand);
        assert!(!g.legal_actions().contains(&free(id)));
        let a = Action::AnnounceFree {
            card: id,
            face: CastFace::Back,
            flash_surcharge: false,
        };
        assert!(g.legal_actions().contains(&a));
        checked(&mut g, a);
        pay(&mut g);
        assert_eq!(
            g.state.stack.last().unwrap().source_card.name(),
            "Photon, Living Light"
        );
        assert_eq!(count(&g, p), 0);
    }
}
#[test]
fn turn_cast_grants_paid_next_cast_consumes_all_old_matching_grants_not_queries() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grant(&mut g, p, 3);
        let id = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        let before = g.snapshot();
        for _ in 0..3 {
            assert!(g.legal_actions().contains(&free(id)));
            assert!(g.can_fund(FundingGoal {
                actor: p,
                required: [0, 0, 0, 0, 1, 0],
                generic: 1,
                reserve: None,
                source: None
            }));
        }
        assert_eq!(g.snapshot(), before);
        assert_eq!(count(&g, p), 3);
        checked(&mut g, Action::AnnounceCast { card: id });
        assert_eq!(count(&g, p), 3);
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        assert_eq!(count(&g, p), 3);
        pay(&mut g);
        assert_eq!(count(&g, p), 0);
        let sid = g.state.stack.last().unwrap().id;
        assert!(g.counter_stack_item(sid, false));
        assert_eq!(count(&g, p), 0);
        grant(&mut g, p, 1);
        assert_eq!(count(&g, p), 1);
    }
}
#[test]
fn turn_cast_grants_nonqualifying_cast_and_other_player_do_not_consume() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grant(&mut g, p, 2);
        let id = named(&mut g, p, "Gilded Lotus", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        cast(&mut g, id, None, &[]);
        assert_eq!(count(&g, p), 2);
        settle(&mut g);
        let other = named(&mut g, p.other(), "Galewind Moose", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 2, 4];
        checked(&mut g, Action::Pass);
        cast(&mut g, other, None, &[]);
        assert_eq!(count(&g, p), 2);
    }
}
#[test]
fn turn_cast_grants_copy_of_qualifying_spell_is_not_a_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, id, None, &[]);
        grant(&mut g, p, 2);
        let target = g.state.stack.last().unwrap().clone();
        let mut item = target.clone();
        item.targets = vec![BoundTarget {
            target: Target::Stack(target.id),
            object: Some(target.object),
            group: 0,
        }];
        g.copy_target_spell(&item, 0, false, crate::cards::CopyExceptions::NONE);
        assert_eq!(count(&g, p), 2);
        assert!(g.state.stack.last().unwrap().is_copy);
    }
}
#[test]
fn turn_cast_grants_timing_owner_and_zone_permission_remain_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grant(&mut g, p, 1);
        g.state.players[p.index()].mana = [0; 6];
        for zone in [Zone::Graveyard, Zone::Exile, Zone::Library] {
            let id = named(&mut g, p, "Bear Cub", zone);
            reject(&mut g, free(id));
        }
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        reject(&mut g, free(other));
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        let flash = named(&mut g, p, "Galewind Moose", Zone::Hand);
        g.state.active = p.other();
        g.state.priority = p;
        g.state.step = Step::Upkeep;
        reject(&mut g, free(bear));
        assert!(g.legal_actions().contains(&free(flash)));
        checked(&mut g, free(flash));
        pay(&mut g);
        assert_eq!(count(&g, p), 0);
    }
}
#[test]
fn turn_cast_grants_library_top_normal_permission_and_hidden_next_top_defer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let below = named(&mut g, p, "Forest", Zone::Library);
        let id = named(&mut g, p, "Bear Cub", Zone::Library);
        grant(&mut g, p, 1);
        g.state.players[p.index()].mana = [0; 6];
        let a = free(id);
        assert!(g.legal_actions().contains(&a));
        g = orphan(g, a);
        assert!(g.observe(p).known_library_top[p.index()].is_none());
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        pay(&mut g);
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            g.object_ref(below)
        );
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        assert_eq!(count(&g, p), 0);
    }
}
#[test]
fn turn_cast_grants_exile_normal_permission_and_explicit_flashback_do_not_mix() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grant(&mut g, p, 2);
        g.state.players[p.index()].mana = [0; 6];
        let id = named(&mut g, p, "Bear Cub", Zone::Exile);
        g.grant_exile_play_permission(p, g.object_ref(id), false);
        checked(&mut g, free(id));
        pay(&mut g);
        assert_eq!(count(&g, p), 0);
        settle(&mut g);
        grant(&mut g, p, 1);
        let gy = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.grant_graveyard_permission(p, g.object_ref(gy), GraveyardCastKind::Flashback);
        reject(&mut g, free(gy));
        g.grant_graveyard_permission(p, g.object_ref(gy), GraveyardCastKind::Cast);
        g = orphan(g, free(gy));
        pay(&mut g);
        assert_eq!(count(&g, p), 0);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().origin,
            Zone::Graveyard
        );
    }
}
#[test]
fn turn_cast_grants_greatwurm_free_never_waives_the_six_counter_origin_cost() {
    for p in [Player::P0, Player::P1] {
        for amount in [0, 5, 6, 7] {
            let mut g = setup(p);
            grant(&mut g, p, 1);
            let id = named(&mut g, p, "Quilled Greatwurm", Zone::Graveyard);
            let bearer = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.set_counters(bearer, Counters::of(CounterKind::PlusOne, amount));
            g.state.players[p.index()].mana = [0; 6];
            assert!(!g.legal_actions().contains(&free(id)));
            assert_eq!(g.legal_actions().contains(&grave_free(id)), amount >= 6);
            if amount < 6 {
                reject(&mut g, grave_free(id));
                continue;
            }
            g = orphan(g, grave_free(id));
            assert_eq!(
                g.state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .graveyard_origin_counter_cost(),
                Some(6)
            );
            assert_eq!(count(&g, p), 1);
            for selected in 0..6 {
                assert_eq!(g.counter_count(bearer, CounterKind::PlusOne), amount);
                let invalid_object = g.object_ref(id);
                reject(
                    &mut g,
                    Action::ChooseCreatureCounterCost {
                        object: invalid_object,
                        counter: CounterKind::PlusOne,
                    },
                );
                let a = Action::ChooseCreatureCounterCost {
                    object: g.object_ref(bearer),
                    counter: CounterKind::PlusOne,
                };
                g = orphan(g, a);
                assert_eq!(count(&g, p), 1);
                if selected < 5 {
                    assert!(
                        !g.legal_actions()
                            .iter()
                            .any(|a| matches!(a, Action::PayCosts { .. }))
                    );
                }
            }
            pay(&mut g);
            assert_eq!(g.counter_count(bearer, CounterKind::PlusOne), amount - 6);
            assert_eq!(count(&g, p), 0);
            settle(&mut g);
            assert!(g.on_battlefield(id));
        }
    }
}
#[test]
fn turn_cast_grants_greatwurm_own_route_and_other_permission_are_distinct_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grant(&mut g, p, 1);
        let id = named(&mut g, p, "Quilled Greatwurm", Zone::Graveyard);
        let bearer = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.set_counters(bearer, Counters::of(CounterKind::Stun, 6));
        g.grant_graveyard_permission(p, g.object_ref(id), GraveyardCastKind::Cast);
        g.state.players[p.index()].mana = [0; 6];
        let actions = g.legal_actions();
        assert!(actions.contains(&free(id)));
        assert!(actions.contains(&grave_free(id)));
        checked(&mut g, free(id));
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .graveyard_origin_counter_cost(),
            None
        );
        pay(&mut g);
        assert_eq!(g.counter_count(bearer, CounterKind::Stun), 6);
        assert_eq!(count(&g, p), 0);
    }
}
#[test]
fn turn_cast_grants_muldrotha_exact_permission_type_cost_and_orphan_replay() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let provider = named(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        grant(&mut g, p, 2);
        g.state.players[p.index()].mana = [0; 6];
        let id = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let action = muldrotha_free(&g, id);
        let permission = g.object_ref(provider);
        reject(
            &mut g,
            Action::AnnounceFreeGraveyardPermanent {
                card: id,
                face: CastFace::Normal,
                flash_surcharge: false,
                permission,
                permanent_type: PermanentType::Artifact,
            },
        );
        assert_eq!(count(&g, p), 2);
        assert!(g.state.graveyard_type_uses.is_none());
        g = orphan(g, action);
        assert_eq!(count(&g, p), 2);
        assert_eq!(g.state.graveyard_type_uses.as_ref().unwrap().len(), 1);
        pay(&mut g);
        assert_eq!(count(&g, p), 0);
        settle(&mut g);
        grant(&mut g, p, 1);
        let next = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        assert!(
            !g.legal_actions().iter().any(
                |a| matches!(a,Action::AnnounceFreeGraveyardPermanent{card,..} if *card==next)
            )
        );
        let old = g.object_ref(provider);
        g.move_object(provider, Zone::Exile);
        g.move_object(provider, Zone::Battlefield);
        assert_ne!(g.object_ref(provider), old);
        let _ = muldrotha_free(&g, next);
    }
}
#[test]
fn turn_cast_grants_muldrotha_not_opponents_turn_suppressed_or_missing_mandatory_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let provider = named(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        grant(&mut g, p, 1);
        let id = named(&mut g, p, "Galewind Moose", Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        let a = muldrotha_free(&g, id);
        g.state.active = p.other();
        reject(&mut g, a.clone());
        g.state.active = p;
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, aura, None, &[Target::Card(provider)]);
        settle(&mut g);
        reject(&mut g, a);
        assert_eq!(count(&g, p), 1);
        assert!(g.state.graveyard_type_uses.is_none());
    }
}
#[test]
fn turn_cast_grants_free_titania_mandatory_discard_or_real_mana_alternative() {
    for p in [Player::P0, Player::P1] {
        for extra_mana in [false, true] {
            let mut g = setup(p);
            grant(&mut g, p, 1);
            let id = named(&mut g, p, "Titania, Rugged Rumbler", Zone::Hand);
            g.state.players[p.index()].mana = if extra_mana {
                [0, 0, 0, 0, 0, 2]
            } else {
                [0; 6]
            };
            if !extra_mana {
                reject(&mut g, free(id));
                named(&mut g, p, "Forest", Zone::Hand);
            }
            checked(&mut g, free(id));
            if extra_mana {
                let actions = g.legal_actions();
                if let Some(a) = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
                {
                    checked(&mut g, a);
                }
            }
            for _ in 0..25 {
                if g.state.announcing.is_none() {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| {
                        matches!(
                            a,
                            Action::ChooseSpellCost { .. }
                                | Action::ChooseAdditionalCost { .. }
                                | Action::FinishTargets
                                | Action::PayCosts { .. }
                        )
                    })
                    .unwrap();
                checked(&mut g, a);
            }
            assert!(g.state.announcing.is_none());
            assert_eq!(count(&g, p), 0);
            settle(&mut g);
            assert!(g.on_battlefield(id));
        }
    }
}
#[test]
fn turn_cast_grants_plot_cast_consumes_and_plotting_itself_does_not() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grant(&mut g, p, 2);
        let id = named(&mut g, p, "Slickshot Show-Off", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        checked(&mut g, Action::AnnouncePlot { card: id });
        checked(
            &mut g,
            Action::PayPlot {
                payment: [0, 0, 0, 1, 0, 1],
            },
        );
        assert_eq!(count(&g, p), 2);
        // Plot's next-turn prerequisite is advanced with ordinary turn progression.
        super::assurance::big_helpers::next_turn(&mut g);
        settle(&mut g);
        super::assurance::big_helpers::next_turn(&mut g);
        for _ in 0..30 {
            if g.state.step == Step::Main1 && g.actor() == Some(p) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        grant(&mut g, p, 2);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnouncePlotted { card: id });
        pay(&mut g);
        assert_eq!(count(&g, p), 0);
    }
}
#[test]
fn turn_cast_grants_cleanup_expires_both_players_before_repeated_cleanup_priority() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for player in [p, p.other()] {
            grant(&mut g, player, 2);
            g.grant_artifact_spell_reduction(player, 3);
        }
        g.state.step = Step::Cleanup;
        g.finish_cleanup();
        assert!(g.state.turn_cast_grants.is_none());
        for player in [p, p.other()] {
            assert_eq!(
                g.observe(player).players[player.index()].turn_cast_grants,
                None
            );
        }
        g.finish_cleanup();
        assert!(g.state.turn_cast_grants.is_none());
    }
}
#[test]
fn turn_cast_grants_private_hand_counterfactual_and_illegal_action_do_not_leak() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grant(&mut g, p, 2);
        g.grant_artifact_spell_reduction(p, 1);
        let hidden = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let mut other = g.clone();
        other.state.objects[hidden as usize].card = Card::from_name("Shivan Dragon").unwrap();
        assert_eq!(views(&g)[p.index()], views(&other)[p.index()]);
        assert_ne!(
            views(&g)[p.other().index()],
            views(&other)[p.other().index()]
        );
        assert_eq!(g.legal_actions(), other.legal_actions());
        let id = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        g = orphan(g, free(id));
        reject(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        assert_eq!(count(&g, p), 2);
        pay(&mut g);
        assert_eq!(count(&g, p), 0);
    }
}
#[test]
fn turn_cast_grants_funding_small_exhaustive_discounts_agree_with_paid_menu() {
    for p in [Player::P0, Player::P1] {
        for reduction in 0..=6 {
            for mana in 0..=5 {
                let mut g = setup(p);
                g.grant_artifact_spell_reduction(p, reduction);
                let id = named(&mut g, p, "Gilded Lotus", Zone::Hand);
                g.state.players[p.index()].mana = [0, 0, 0, 0, 0, mana];
                let before = g.snapshot();
                let (colored, generic) = g.object_view(id).card.cost();
                let request = ManaCost { colored, generic };
                let expected = mana >= 5u32.saturating_sub(reduction);
                assert_eq!(g.spell_can_fund(p, id, request), expected);
                assert_eq!(
                    g.legal_actions()
                        .contains(&Action::AnnounceCast { card: id }),
                    expected
                );
                assert_eq!(g.snapshot(), before);
            }
        }
    }
}
#[test]
fn turn_cast_grants_muldrotha_mandatory_cost_failure_does_not_spend_either_allowance() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let provider = named(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
        let id = named(&mut g, p, "Titania, Rugged Rumbler", Zone::Graveyard);
        grant(&mut g, p, 2);
        g.state.players[p.index()].mana = [0; 6];
        let a = Action::AnnounceFreeGraveyardPermanent {
            card: id,
            face: CastFace::Normal,
            flash_surcharge: false,
            permission: g.object_ref(provider),
            permanent_type: PermanentType::Creature,
        };
        reject(&mut g, a.clone());
        assert!(g.state.graveyard_type_uses.is_none());
        assert_eq!(count(&g, p), 2);
        let discard = named(&mut g, p, "Forest", Zone::Hand);
        g = orphan(g, a);
        for _ in 0..20 {
            if g.state.announcing.is_none() {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::ChooseSpellCost { .. }
                            | Action::ChooseAdditionalCost { .. }
                            | Action::FinishTargets
                            | Action::PayCosts { .. }
                    )
                })
                .unwrap();
            checked(&mut g, a);
        }
        assert_eq!(g.state.objects[discard as usize].zone, Zone::Graveyard);
        assert_eq!(count(&g, p), 0);
        assert_eq!(g.state.graveyard_type_uses.as_ref().unwrap().len(), 1);
    }
}
#[test]
fn turn_cast_grants_paid_warp_consumes_without_using_free_alternative() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grant(&mut g, p, 3);
        let id = named(&mut g, p, "Nova Hellkite", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        checked(&mut g, Action::AnnounceWarp { card: id });
        assert_eq!(count(&g, p), 3);
        pay(&mut g);
        assert_eq!(count(&g, p), 0);
        let item = g.state.stack.last().unwrap();
        assert_eq!(item.cast.unwrap().method, CastMethod::Warp);
        assert_eq!(item.cast.unwrap().payment, [0, 0, 0, 1, 0, 2]);
    }
}
#[test]
fn turn_cast_grants_free_x_creature_is_zero_without_a_scalar_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grant(&mut g, p, 1);
        let id = named(&mut g, p, "Wildwood Scourge", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        g = orphan(g, free(id));
        assert!(g.x_choice().is_none());
        reject(&mut g, Action::ChooseX { value: 1 });
        pay(&mut g);
        let item = g.state.stack.last().unwrap();
        assert_eq!(item.x, Some(0));
        assert_eq!(item.mana_value(), 1);
        assert_eq!(count(&g, p), 0);
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn turn_cast_grants_greatwurm_counter_payment_uses_current_creature_controller_and_kind() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grant(&mut g, p, 1);
        let id = named(&mut g, p, "Quilled Greatwurm", Zone::Graveyard);
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.set_counters(other, Counters::of(CounterKind::PlusOne, 6));
        let artifact = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        g.set_counters(artifact, Counters::of(CounterKind::PlusOne, 6));
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, grave_free(id));
        // A bounded owner/control fixture: the cost uses the actual controller.
        g.state.objects[other as usize].controller = p;
        checked(&mut g, grave_free(id));
        let invalid = g.object_ref(artifact);
        reject(
            &mut g,
            Action::ChooseCreatureCounterCost {
                object: invalid,
                counter: CounterKind::PlusOne,
            },
        );
        for _ in 0..6 {
            let object = g.object_ref(other);
            checked(
                &mut g,
                Action::ChooseCreatureCounterCost {
                    object,
                    counter: CounterKind::PlusOne,
                },
            );
        }
        pay(&mut g);
        assert_eq!(g.counter_count(other, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(artifact, CounterKind::PlusOne), 6);
        assert_eq!(count(&g, p), 0);
    }
}
#[test]
fn turn_cast_grants_omniscience_overlap_emits_each_same_face_alternative_once() {
    for p in [Player::P0, Player::P1] {
        for own_provider in [false, true] {
            let mut g = setup(p);
            named(
                &mut g,
                if own_provider { p } else { p.other() },
                "Omniscience",
                Zone::Battlefield,
            );
            grant(&mut g, p, 1);
            g.state.players[p.index()].mana = [0; 6];
            let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
            let monica = named(&mut g, p, "Monica Rambeau", Zone::Hand);
            let white = named(&mut g, p, "Serra Angel", Zone::Hand);
            let back = Action::AnnounceFree {
                card: monica,
                face: CastFace::Back,
                flash_surcharge: false,
            };
            let actions = g.legal_actions();
            assert_eq!(actions.iter().filter(|a| **a == free(bear)).count(), 1);
            assert_eq!(actions.iter().filter(|a| **a == back).count(), 1);
            assert_eq!(actions.contains(&free(white)), own_provider);
            g = orphan(g, free(bear));
            pay(&mut g);
            assert_eq!(count(&g, p), 0);
            settle(&mut g);
            let actions = g.legal_actions();
            assert_eq!(actions.contains(&back), own_provider);
            assert_eq!(actions.contains(&free(white)), own_provider);
            let unique = actions
                .iter()
                .map(|a| serde_json::to_string(a).unwrap())
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(actions.len(), unique.len());
        }
    }
}
