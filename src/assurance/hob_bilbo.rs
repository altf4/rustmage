//! Paid resolution casts retain chosen-face, origin, cost and exact-object facts.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Bilbo budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn bilbo(g: &mut Game, p: Player) -> CardId {
    put(g, p, "Bilbo, Thief in the Night", Zone::Battlefield)
}
fn attack(g: &mut Game, p: Player, id: CardId) {
    g.state.active = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: id });
    act(g, Action::FinishAttackers);
    g.prepare_triggers();
    until(g, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
    assert_eq!(g.actor(), Some(p));
}
fn select(g: &mut Game, id: CardId, face: CastFace) {
    let object = g.object_ref(id);
    act(g, Action::CastExiledDuringResolution { object, face });
}
fn finish_offer(g: &mut Game) {
    act(g, Action::FinishResolutionCasts);
    finish(g);
}
fn offered(g: &Game, id: CardId, face: CastFace) -> bool {
    g.legal_actions()
        .contains(&Action::CastExiledDuringResolution {
            object: g.object_ref(id),
            face,
        })
}
#[test]
fn hob_bilbo_paid_body_and_hand_casts_keep_printed_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = put(&mut g, p, "Bilbo, Thief in the Night", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, id, None);
        finish(&mut g);
        assert_eq!(stats(&g, id), (2, 2));
        assert!(g.has_subtype(id, "Halfling") && g.has_subtype(id, "Rogue"));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: id });
        g.state.step = Step::Main1;
        g.state.pending = Pending::Priority;
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card: pick });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        cast(&mut g, pick, None);
        finish(&mut g);
        assert_eq!(g.object_view(pick).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn hob_bilbo_optional_nontargeting_choice_filters_controller_kind_and_affordability() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        let creature = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let enchant = put(&mut g, p, "Witness Protection", Zone::Graveyard);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        let foreign = put(&mut g, p.other(), "Goldvein Pick", Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        attack(&mut g, p, b);
        for id in [artifact, creature, enchant, land, foreign] {
            assert!(!offered(&g, id, CastFace::Normal));
        }
        let public =
            serde_json::to_value(g.observe(p.other()).resolution_free_casts.unwrap()).unwrap();
        assert_eq!(public["paid"]["kinds"], 28);
        assert_eq!(public["paid"]["exile_instants_sorceries"], true);
        assert_eq!(public["casts_remaining"], 1);
        assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        finish_offer(&mut g);
        for id in [artifact, creature, enchant, land, foreign] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
    }
}
#[test]
fn hob_bilbo_paid_artifact_or_instant_one_cast_and_exile_only_spells() {
    for p in [Player::P0, Player::P1] {
        for name in ["Goldvein Pick", "Shock"] {
            for counter in [false, true] {
                let mut g = scenario(p);
                let b = bilbo(&mut g, p);
                let id = put(&mut g, p, name, Zone::Graveyard);
                let unused = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
                let artifact = name == "Goldvein Pick";
                g.state.players[p.index()].mana = if artifact {
                    [0, 0, 0, 0, 0, 1]
                } else {
                    [0, 0, 0, 1, 0, 0]
                };
                attack(&mut g, p, b);
                assert!(offered(&g, id, CastFace::Normal));
                select(&mut g, id, CastFace::Normal);
                if !artifact {
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Player(p.other()),
                        },
                    );
                }
                reject(&mut g, Action::PayCosts { payment: [0; 6] });
                pay(&mut g);
                let spell = g
                    .state
                    .stack
                    .iter()
                    .find(|s| s.source.is_some_and(|source| source.id == id))
                    .unwrap();
                assert_eq!(spell.cast.unwrap().origin, Zone::Graveyard);
                assert_eq!(spell.cast.unwrap().method, CastMethod::Normal);
                assert_eq!(spell.cast.unwrap().exile_on_graveyard, !artifact);
                let stack = spell.id;
                assert!(!offered(&g, unused, CastFace::Normal));
                assert_eq!(
                    g.state
                        .resolution_free_casts
                        .as_ref()
                        .unwrap()
                        .casts_remaining,
                    Some(0)
                );
                act(&mut g, Action::FinishResolutionCasts);
                if counter {
                    g.counter_stack_item(stack, false);
                }
                finish(&mut g);
                assert_eq!(
                    g.object_view(id).zone,
                    if artifact {
                        if counter {
                            Zone::Graveyard
                        } else {
                            Zone::Battlefield
                        }
                    } else {
                        Zone::Exile
                    }
                );
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    if !artifact && !counter { 18 } else { 20 }
                );
                if artifact && !counter {
                    g.move_object(id, Zone::Graveyard);
                    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
                }
            }
        }
    }
}
#[test]
fn hob_bilbo_paid_x_sorcery_keeps_x_and_public_draws_are_private() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let spell = put(&mut g, p, "Mind Spring", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 2];
        attack(&mut g, p, b);
        select(&mut g, spell, CastFace::Normal);
        act(&mut g, Action::ChooseX { value: 3 });
        pay(&mut g);
        assert_eq!(g.state.stack.last().unwrap().x, Some(3));
        finish_offer(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        let hand: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.owner == p && o.zone == Zone::Hand)
            .map(|(id, _)| id as CardId)
            .collect();
        assert_eq!(hand.len(), 3);
        assert!(
            hand.iter()
                .all(|id| g.observe(p.other()).objects.iter().all(|o| o.id != *id))
        );
    }
}
#[test]
fn hob_bilbo_printed_mana_alternative_or_additional_sacrifice_both_work() {
    for p in [Player::P0, Player::P1] {
        for alternate in [false, true] {
            let mut g = scenario(p);
            let b = bilbo(&mut g, p);
            let spell = put(&mut g, p, "Stir Up Trouble", Zone::Graveyard);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let fodder = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            g.state.players[p.index()].mana = if alternate {
                [0, 0, 1, 0, 0, 3]
            } else {
                [0, 0, 1, 0, 0, 0]
            };
            attack(&mut g, p, b);
            select(&mut g, spell, CastFace::Normal);
            act(
                &mut g,
                Action::ChooseSpellCost {
                    cost: if alternate { 1 } else { 0 },
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if !alternate {
                let object = g.object_ref(fodder);
                act(&mut g, Action::ChooseAdditionalCost { object });
            }
            pay(&mut g);
            finish_offer(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(fodder).zone,
                if alternate {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn hob_bilbo_face_specific_permissions_include_adventure_not_creature_and_real_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let goblin = put(
            &mut g,
            p,
            "Great Ugly-Looking Goblin // Clap! Snap!",
            Zone::Graveyard,
        );
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        attack(&mut g, p, b);
        assert!(!offered(&g, goblin, CastFace::Normal));
        assert!(offered(&g, goblin, CastFace::Adventure));
        select(&mut g, goblin, CastFace::Adventure);
        pay(&mut g);
        let spell = g.state.stack.last().unwrap();
        assert_eq!(spell.source.unwrap().card.name(), "Clap! Snap!");
        assert!(spell.cast.unwrap().exile_on_graveyard);
        finish_offer(&mut g);
        assert_eq!(g.object_view(goblin).zone, Zone::Exile);
        assert!(
            g.state
                .objects
                .iter()
                .enumerate()
                .any(|(id, o)| o.zone == Zone::Battlefield
                    && o.controller == p
                    && g.has_subtype(id as CardId, "Army"))
        );
    }
}
#[test]
fn hob_bilbo_frame_keeps_exact_incarnation_and_source_departure_only_removes_discount() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        attack(&mut g, p, b);
        let old = g.object_ref(pick);
        g.move_object(pick, Zone::Exile);
        g.move_object(pick, Zone::Graveyard);
        assert!(!offered(&g, pick, CastFace::Normal));
        reject(
            &mut g,
            Action::CastExiledDuringResolution {
                object: old,
                face: CastFace::Normal,
            },
        );
        finish_offer(&mut g);
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        attack(&mut g, p, b);
        g.move_object(b, Zone::Hand);
        assert!(!offered(&g, pick, CastFace::Normal));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        assert!(offered(&g, pick, CastFace::Normal));
        select(&mut g, pick, CastFace::Normal);
        pay(&mut g);
        finish_offer(&mut g);
        assert_eq!(g.object_view(pick).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn hob_bilbo_countered_trigger_grants_nothing_and_suppressed_source_does_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: b });
        act(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
        });
        let id = g.state.stack.last().unwrap().id;
        g.counter_stack_item(id, false);
        finish(&mut g);
        assert!(g.state.resolution_free_casts.is_none());
        assert_eq!(g.object_view(pick).zone, Zone::Graveyard);
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(b));
        assert!(!g.printed_abilities_available(b));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        g.state.objects[b as usize].tapped = false;
        g.state.attackers.clear();
        act(&mut g, Action::Attack { card: b });
        act(&mut g, Action::FinishAttackers);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn hob_bilbo_nonhand_discount_applies_exile_and_graveyard_current_controller_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let exiled = put(&mut g, p, "Goldvein Pick", Zone::Exile);
        g.grant_exile_play_permission(p, g.object_ref(exiled), false);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceExile { card: exiled });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(exiled).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.change_controller(g.object_ref(b), p.other());
        let exiled = put(&mut g, p, "Goldvein Pick", Zone::Exile);
        g.grant_exile_play_permission(p, g.object_ref(exiled), false);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceExile { card: exiled });
    }
}
#[test]
fn hob_bilbo_owned_paid_choice_and_selected_payment_survive_parent_disposal_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        attack(&mut g, p, b);
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let action = Action::CastExiledDuringResolution {
            object: g.object_ref(pick),
            face: CastFace::Normal,
        };
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        act(&mut raw, action.clone());
        assert_eq!(raw.snapshot(), indexed.snapshot());
        contracts::branch_contract(raw.clone());
        reject(&mut raw, Action::PayCosts { payment: [0; 6] });
        pay(&mut raw);
        finish_offer(&mut raw);
        let expected = raw.snapshot();
        raw.restore(&saved).unwrap();
        act(&mut raw, action);
        pay(&mut raw);
        finish_offer(&mut raw);
        assert_eq!(raw.snapshot(), expected);
        pay(&mut indexed);
        finish_offer(&mut indexed);
        assert_eq!(indexed.snapshot(), expected);
    }
}
#[test]
fn hob_bilbo_mana_payment_sacrificing_provider_preserves_locked_reduced_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(b)));
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        attack(&mut g, p, b);
        // Lotus Ring grants vigilance, so the actual attacker remains untapped.
        assert!(!g.object_view(b).tapped);
        assert!(offered(&g, pick, CastFace::Normal));
        select(&mut g, pick, CastFace::Normal);
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            1
        );
        let grantor = g.object_ref(ring);
        act(
            &mut g,
            Action::TapGrantedMana {
                card: b,
                grantor,
                production: [0, 0, 3, 0, 0, 0],
            },
        );
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, 0],
            },
        );
        finish_offer(&mut g);
        assert_eq!(g.object_view(pick).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 2, 0, 0, 0]);
    }
}
#[test]
fn hob_bilbo_library_origin_reduction_floors_generic_and_preserves_colored() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        bilbo(&mut g, p);
        put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Library);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        act(&mut g, Action::AnnounceCast { card: bear });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let shock = put(&mut g, p, "Shock", Zone::Exile);
        g.grant_exile_play_permission(p, g.object_ref(shock), false);
        reject(&mut g, Action::AnnounceExile { card: shock });
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        act(&mut g, Action::AnnounceExile { card: shock });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(
            g.object_view(shock).zone,
            Zone::Graveyard,
            "ordinary exile permission has no Bilbo replacement"
        );
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn hob_bilbo_doubled_attack_packets_each_offer_exactly_one_paid_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(staff), g.object_ref(b)));
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        let shock = put(&mut g, p, "Shock", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        attack(&mut g, p, b);
        select(&mut g, pick, CastFace::Normal);
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        assert!(!offered(&g, shock, CastFace::Normal));
        act(&mut g, Action::FinishResolutionCasts);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionCasts { .. })
        });
        select(&mut g, shock, CastFace::Normal);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        finish_offer(&mut g);
        assert_eq!(g.object_view(pick).zone, Zone::Battlefield);
        assert_eq!(g.object_view(shock).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn hob_bilbo_stale_spell_target_fizzles_but_still_exiles_exact_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let shock = put(&mut g, p, "Shock", Zone::Graveyard);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        attack(&mut g, p, b);
        select(&mut g, shock, CastFace::Normal);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        act(&mut g, Action::FinishResolutionCasts);
        g.move_object(bear, Zone::Hand);
        g.move_object(bear, Zone::Battlefield);
        finish(&mut g);
        assert_eq!(g.object_view(shock).zone, Zone::Exile);
        assert_eq!(g.object_view(bear).damage, 0);
        // The replaced old spell incarnation grants no replacement to its later hand cast.
        g.move_object(shock, Zone::Hand);
        g.state.step = Step::Main1;
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        cast(&mut g, shock, Some(Target::Player(p.other())));
        finish(&mut g);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
    }
}
#[test]
fn hob_bilbo_paid_resolution_permission_bypasses_timing_but_not_prohibitions() {
    // A generic resolution-cast contract: an already-created frame is independent
    // of the active seat. Normal Bilbo attacks necessarily start in its own turn.
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bilbo(&mut g, p);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        attack(&mut g, p, b);
        g.state.active = p.other();
        assert!(offered(&g, pick, CastFace::Normal));
        g.state.casting_prohibited = true;
        assert!(!offered(&g, pick, CastFace::Normal));
        g.state.casting_prohibited = false;
        select(&mut g, pick, CastFace::Normal);
        pay(&mut g);
        finish_offer(&mut g);
        assert_eq!(g.object_view(pick).zone, Zone::Battlefield);
    }
}
