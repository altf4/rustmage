//! Source-identified death replacement and independently resolving Wolf packets.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Hunt budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn head(g: &mut Game, p: Player) -> CardId {
    put(g, p, "Head of the Hunt", Zone::Battlefield)
}
fn wolves(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card.index() == 5412
        })
        .map(|(i, _)| i as CardId)
        .collect()
}
fn packet(g: &mut Game) {
    g.prepare_triggers();
    until(g, |g| {
        g.state
            .stack
            .iter()
            .any(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
    });
}
fn shock(g: &mut Game, p: Player, target: CardId) {
    g.state.priority = p;
    let s = put(g, p, "Shock", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    cast(g, s, Some(Target::Card(target)));
}
fn option(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::ZoneReplacement { .. })
    });
}
fn replacement(source: ObjectRef) -> ZoneReplacementKind {
    ZoneReplacementKind::ExileReflexive { source, ability: 0 }
}
#[test]
fn hob_hunt_flash_paid_body_real_death_defers_token_and_source_removal_keeps_packet() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.active = p.other();
        let s = put(&mut g, p, "Head of the Hunt", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 2];
        cast(&mut g, s, None);
        finish(&mut g);
        assert_eq!(stats(&g, s), (4, 3));
        assert!(g.keywords(s).contains(Keywords::FLASH));
        assert!(g.has_subtype(s, "Wolf"));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let v = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        shock(&mut g, p.other(), v);
        packet(&mut g);
        assert_eq!(g.object_view(v).zone, Zone::Exile);
        assert!(wolves(&g, p).is_empty());
        assert_eq!(g.state.stack.last().unwrap().object, g.object_ref(s));
        g.move_object(s, Zone::Hand);
        finish(&mut g);
        let w = wolves(&g, p);
        assert_eq!(w.len(), 1);
        assert_eq!(stats(&g, w[0]), (2, 2));
        assert!(g.has_subtype(w[0], "Wolf"));
        assert_eq!(g.object_view(w[0]).colors, 16);
        assert!(!g.state.objects[w[0] as usize].tapped);
        assert!(g.state.objects[w[0] as usize].summoning_sick);
    }
}
#[test]
fn hob_hunt_exact_current_type_control_tokens_and_non_death_qualifications() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        head(&mut g, p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.move_object(own, Zone::Graveyard);
        let artifact = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        g.move_object(artifact, Zone::Graveyard);
        let hand = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        g.move_object(hand, Zone::Graveyard);
        let exile = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(exile, Zone::Exile);
        assert!(g.state.pending_triggers.is_empty());
        let stolen = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.change_controller(g.object_ref(stolen), p.other());
        g.move_object(stolen, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.object_view(stolen).zone, Zone::Exile);
        assert_eq!(wolves(&g, p).len(), 1);
        let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        g.change_object_form(
            g.object_ref(land),
            crate::cards::AnimationSpec {
                artifact: false,
                creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                base: Some((1, 1)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        g.move_object(land, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Exile);
        assert_eq!(wolves(&g, p).len(), 2);
        g.create_tokens(
            p.other(),
            Card::from_name("@token/hob-bear").unwrap(),
            1,
            false,
        );
        let token = (g.state.objects.len() - 1) as CardId;
        let old = g.object_ref(token);
        g.move_object(token, Zone::Graveyard);
        finish(&mut g);
        assert_ne!(g.object_ref(token), old);
        assert_eq!(wolves(&g, p).len(), 3);
    }
}
#[test]
fn hob_hunt_multiple_replacements_are_distinct_owned_choices_only_selected_head_triggers() {
    for p in [Player::P0, Player::P1] {
        for choose_second in [false, true] {
            let mut g = scenario(p);
            let a = head(&mut g, p);
            let b = head(&mut g, p);
            let v = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.change_controller(g.object_ref(v), p.other());
            shock(&mut g, p.other(), v);
            option(&mut g);
            assert_eq!(g.actor(), Some(p.other()), "controller chooses, not owner");
            let options = g
                .observe(p.other())
                .zone_replacement_choice
                .unwrap()
                .choices;
            assert_eq!(options.len(), 2);
            assert!(
                options.contains(&replacement(g.object_ref(a)))
                    && options.contains(&replacement(g.object_ref(b)))
            );
            assert!(g.observe(p).zone_replacement_choice.is_none());
            assert_ne!(
                serde_json::to_value(options[0]).unwrap(),
                serde_json::to_value(options[1]).unwrap()
            );
            contracts::branch_contract(g.clone());
            let chosen = if choose_second { b } else { a };
            let effect = replacement(g.object_ref(chosen));
            act(&mut g, Action::ChooseZoneReplacement { effect });
            packet(&mut g);
            assert_eq!(g.state.stack.len(), 1);
            assert_eq!(g.state.stack[0].object, g.object_ref(chosen));
            finish(&mut g);
            assert_eq!(wolves(&g, p).len(), 1);
        }
    }
}
#[test]
fn hob_hunt_rip_and_colossus_compete_and_nonchosen_replacement_never_creates_wolf() {
    for p in [Player::P0, Player::P1] {
        for rip in [false, true] {
            for hunt in [false, true] {
                let mut g = scenario(p);
                let h = head(&mut g, p);
                if rip {
                    put(&mut g, p, "Rest in Peace", Zone::Battlefield);
                }
                let v = put(
                    &mut g,
                    p.other(),
                    if rip {
                        "Bear Cub"
                    } else {
                        "Darksteel Colossus"
                    },
                    Zone::Battlefield,
                );
                // Actual granted tap/sacrifice mana cost; indestructible does not stop it.
                let ring = put(&mut g, p.other(), "Lotus Ring", Zone::Battlefield);
                assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(v)));
                g.state.priority = p.other();
                let grantor = g.object_ref(ring);
                act(
                    &mut g,
                    Action::TapGrantedMana {
                        card: v,
                        grantor,
                        production: [0, 0, 3, 0, 0, 0],
                    },
                );
                option(&mut g);
                let effect = if hunt {
                    replacement(g.object_ref(h))
                } else if rip {
                    ZoneReplacementKind::Exile
                } else {
                    ZoneReplacementKind::RevealShuffle
                };
                act(&mut g, Action::ChooseZoneReplacement { effect });
                finish(&mut g);
                assert_eq!(wolves(&g, p).len(), usize::from(hunt));
                assert_eq!(
                    g.object_view(v).zone,
                    if hunt || rip {
                        Zone::Exile
                    } else {
                        Zone::Library
                    }
                );
            }
        }
    }
}
#[test]
fn hob_hunt_simultaneous_source_departure_and_opposing_heads_use_frozen_pre_event_sources() {
    for p in [Player::P0, Player::P1] {
        for other_head in [false, true] {
            let mut g = scenario(p);
            let a = head(&mut g, p);
            let b = if other_head {
                head(&mut g, p.other())
            } else {
                put(&mut g, p.other(), "Bear Cub", Zone::Battlefield)
            };
            let observers = g.death_trigger_sources();
            let events = g.move_objects_raw(&[a, b], Zone::Graveyard);
            g.finish_zone_events(&events, &observers);
            finish(&mut g);
            assert_eq!(wolves(&g, p).len(), 1);
            assert_eq!(wolves(&g, p.other()).len(), usize::from(other_head));
            assert_eq!(g.object_view(b).zone, Zone::Exile);
            assert_eq!(
                g.object_view(a).zone,
                if other_head {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn hob_hunt_ability_suppression_reacquisition_and_token_doubling_are_current() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let h = head(&mut g, p);
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, aura, Some(Target::Card(h)));
        finish(&mut g);
        let v = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(v, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.object_view(v).zone, Zone::Graveyard);
        assert!(wolves(&g, p).is_empty());
        g.move_object(aura, Zone::Graveyard);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(b, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(wolves(&g, p).len(), 2);
    }
}
#[test]
fn hob_hunt_reflexive_can_be_countered_and_staff_multiplies_actual_packets() {
    for p in [Player::P0, Player::P1] {
        for counter in [false, true] {
            let mut g = scenario(p);
            let h = head(&mut g, p);
            let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(staff), g.object_ref(h)));
            let v = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.move_object(v, Zone::Graveyard);
            packet(&mut g);
            assert_eq!(g.state.stack.len() + g.state.ordering_triggers.len(), 2);
            if counter {
                let id = g.state.stack.last().unwrap().id;
                assert!(g.counter_stack_item(id, false));
            }
            finish(&mut g);
            assert_eq!(wolves(&g, p).len(), if counter { 1 } else { 2 });
        }
    }
}
#[test]
fn hob_hunt_copied_provider_and_old_incarnation_packet_survive_later_source_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let h = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.apply_temporary_copy(
            g.object_ref(h),
            CopyRecipe::new(Card::from_name("Head of the Hunt").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let v = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(v, Zone::Graveyard);
        packet(&mut g);
        let old = g.object_ref(h);
        assert_eq!(g.state.stack.last().unwrap().object, old);
        assert_eq!(
            g.state.stack.last().unwrap().source_card,
            Card::from_name("Head of the Hunt").unwrap()
        );
        g.move_object(h, Zone::Hand);
        g.move_object(h, Zone::Battlefield);
        assert_ne!(g.object_ref(h), old);
        assert_eq!(g.object_view(h).name(), "Bear Cub");
        contracts::branch_contract(g.clone());
        finish(&mut g);
        assert_eq!(wolves(&g, p).len(), 1);
    }
}
