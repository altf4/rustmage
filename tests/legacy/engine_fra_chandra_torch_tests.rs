use super::*;

fn torch_activate(g: &mut Game, id: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
}

#[test]
fn torch_decline_empty_and_land_deal_damage_even_after_source_departure() {
    for player in [Player::P0, Player::P1] {
        for top in [None, Some("Mountain"), Some("Bear Cub")] {
            let mut g = scenario(player);
            let chandra = put(
                &mut g,
                player,
                "Chandra, Torch of Defiance",
                Zone::Battlefield,
            );
            g.state.objects[chandra as usize].counters.loyalty = 4;
            if let Some(name) = top {
                put(&mut g, player, name, Zone::Library);
            } else {
                let cards = g.state.players[player.index()].library.clone();
                for id in cards {
                    g.move_object(id, Zone::Exile);
                }
            }
            let life = g.state.players[player.other().index()].life;
            torch_activate(&mut g, chandra, 0);
            g.move_object(chandra, Zone::Graveyard);
            let saved = g.snapshot();
            for _ in 0..2 {
                g.resolve_top();
                if top.is_some() {
                    assert!(matches!(g.state.pending, Pending::ResolutionCasts { .. }));
                    act(&mut g, Action::FinishResolutionCasts);
                }
                assert_eq!(g.state.players[player.other().index()].life, life - 2);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn torch_paid_cast_commits_result_and_snapshot_replays() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        let bear = put(&mut g, player, "Bear Cub", Zone::Library);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
        let life = g.state.players[player.other().index()].life;
        torch_activate(&mut g, chandra, 0);
        g.resolve_top();
        let object = g.object_ref(bear);
        let bad = Action::CastExiledDuringResolution {
            object,
            face: CastFace::Back,
        };
        let before = g.snapshot();
        reject(&mut g, bad);
        assert_eq!(g.snapshot(), before);
        for _ in 0..2 {
            act(
                &mut g,
                Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Normal,
                },
            );
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .casts_remaining,
                Some(0)
            );
            act(&mut g, Action::FinishResolutionCasts);
            assert_eq!(g.state.players[player.other().index()].life, life);
            assert_eq!(g.object_view(bear).zone, Zone::Stack);
            g.restore(&before).unwrap();
        }
    }
}

#[test]
fn torch_mana_and_damage_modes_obey_loyalty_and_targets() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        g.state.players[player.index()].mana = [0; 6];
        torch_activate(&mut g, chandra, 1);
        g.resolve_top();
        assert_eq!(g.state.players[player.index()].mana[3], 2);
        assert_eq!(g.counter_count(chandra, CounterKind::Loyalty), 5);
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        let victim = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[victim as usize].growth.toughness = 4;
        g.state.objects[chandra as usize].counters.loyalty = 4;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: chandra,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        pay(&mut g);
        g.resolve_top();
        assert_eq!(g.object_view(victim).damage, 4);
    }
}

#[test]
fn torch_emblem_survives_source_and_triggers_for_its_controllers_casts() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 8;
        torch_activate(&mut g, chandra, 3);
        g.resolve_top();
        g.move_object(chandra, Zone::Graveyard);
        assert!(
            g.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Command && o.card == Card::test_index(5465))
        );
        let spell = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, spell, None);
        g.prepare_triggers();
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        let trigger = g
            .state
            .stack
            .iter()
            .find(|s| {
                s.source
                    .is_some_and(|source| source.card == Card::test_index(5465))
            })
            .expect("emblem cast trigger");
        assert_eq!(trigger.controller, player);
        let life = g.state.players[player.other().index()].life;
        g.resolve_top();
        assert_eq!(g.state.players[player.other().index()].life, life - 5);
    }
}

#[test]
fn torch_unfundable_cast_is_rejected_atomically_and_can_be_declined() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        let bear = put(&mut g, player, "Bear Cub", Zone::Library);
        g.state.players[player.index()].mana = [0; 6];
        let life = g.state.players[player.other().index()].life;
        torch_activate(&mut g, chandra, 0);
        g.resolve_top();
        let before = g.snapshot();
        let object = g.object_ref(bear);
        reject(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        assert_eq!(g.snapshot(), before);
        act(&mut g, Action::FinishResolutionCasts);
        assert_eq!(g.state.players[player.other().index()].life, life - 2);
    }
}

#[test]
fn torch_copied_ability_resolves_separately_for_its_controller() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        let other_top = put(&mut g, player.other(), "Mountain", Zone::Library);
        let own_top = put(&mut g, player, "Mountain", Zone::Library);
        let p_life = g.state.players[player.index()].life;
        let other_life = g.state.players[player.other().index()].life;
        torch_activate(&mut g, chandra, 0);
        let ability = g.state.stack.last().unwrap().clone();
        let mut copier = ability.clone();
        copier.controller = player.other();
        copier.targets = vec![BoundTarget {
            group: 0,
            target: Target::Stack(ability.id),
            object: None,
        }];
        g.copy_target_ability(&copier, 0);
        g.move_object(chandra, Zone::Graveyard);
        g.resolve_top();
        assert_eq!(g.object_view(other_top).zone, Zone::Exile);
        act(&mut g, Action::FinishResolutionCasts);
        assert_eq!(g.state.players[player.index()].life, p_life - 2);
        g.resolve_top();
        assert_eq!(g.object_view(own_top).zone, Zone::Exile);
        act(&mut g, Action::FinishResolutionCasts);
        assert_eq!(g.state.players[player.other().index()].life, other_life - 2);
    }
}

#[test]
fn torch_can_cast_exiled_disguise_for_three_and_remembers_commit() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        // Bubble Smuggler is the engine's pinned test-only disguise fixture.
        let creature = put(&mut g, player, "Bubble Smuggler", Zone::Library);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        let life = g.state.players[player.other().index()].life;
        torch_activate(&mut g, chandra, 0);
        g.resolve_top();
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceDisguise { card: creature });
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .casts_remaining,
                Some(0)
            );
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().method,
                CastMethod::Disguise
            );
            act(&mut g, Action::FinishResolutionCasts);
            assert_eq!(g.state.players[player.other().index()].life, life);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn torch_countered_committed_cast_still_avoids_damage() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        let bear = put(&mut g, player, "Bear Cub", Zone::Library);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
        let life = g.state.players[player.other().index()].life;
        torch_activate(&mut g, chandra, 0);
        g.resolve_top();
        let object = g.object_ref(bear);
        act(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        pay(&mut g);
        let stack_id = g.state.stack.last().unwrap().id;
        act(&mut g, Action::FinishResolutionCasts);
        assert!(g.counter_stack_item(stack_id, false));
        assert_eq!(g.state.players[player.other().index()].life, life);
    }
}

#[test]
fn torch_foreign_entry_choice_can_reverse_unfundable_cast_before_declining() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let silver = put(
            &mut g,
            player,
            "Silverquill, the Disputant",
            Zone::Battlefield,
        );
        g.state.objects[silver as usize].growth.power = -1;
        let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let aura = put(
            &mut g,
            player.other(),
            "Witness Protection",
            Zone::Battlefield,
        );
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(enemy));
        let prison = put(&mut g, player, "Banishing Light", Zone::Hand);
        super::super::big_helpers::cast(&mut g, prison, None, &[]);
        for _ in 0..40 {
            if g.state.pending != Pending::Priority {
                break;
            }
            act(&mut g, Action::Pass);
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(aura),
            },
        );
        super::super::big_helpers::settle(&mut g);
        assert_eq!(g.object_view(aura).zone, Zone::Exile);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        let spell = put(&mut g, player, "Candy Grapple", Zone::Library);
        let life = g.state.players[player.other().index()].life;
        torch_activate(&mut g, chandra, 0);
        g.resolve_top();
        let before = g.snapshot();
        let object = g.object_ref(spell);
        act(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        let sacrifice = g.object_ref(prison);
        act(
            &mut g,
            Action::ChooseBargain {
                sacrifice: Some(sacrifice),
            },
        );
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        pay(&mut g);
        let clause = g
            .observe(player)
            .cast_costs
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::Additional && !row.paid)
            .unwrap();
        act(
            &mut g,
            Action::ChooseCastCost {
                clause: clause as u16,
            },
        );
        assert_eq!(g.actor(), Some(player.other()));
        let silver_object = g.object_ref(silver);
        act(
            &mut g,
            Action::ChooseAuraAttachment {
                object: silver_object,
            },
        );
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        act(&mut g, Action::FinishResolutionCasts);
        assert_eq!(g.state.players[player.other().index()].life, life - 2);
    }
}

#[test]
fn torch_paid_offer_preserves_chosen_x_and_normal_mana_cost() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        let spell = put(&mut g, player, "Ajani's Anguish", Zone::Library);
        g.state.players[player.index()].mana = [0, 0, 0, 1, 0, 2];
        torch_activate(&mut g, chandra, 0);
        g.resolve_top();
        let object = g.object_ref(spell);
        act(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        act(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        assert_eq!(g.state.stack.last().unwrap().x, Some(2));
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
        act(&mut g, Action::FinishResolutionCasts);
    }
}

#[test]
fn torch_paid_offer_keeps_modal_spell_choices_and_opponent_tax() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        put(
            &mut g,
            player.other(),
            "Thalia, the Survivor",
            Zone::Battlefield,
        );
        let charm = put(&mut g, player, "Konstrari Charm", Zone::Library);
        g.state.players[player.index()].mana = [0, 0, 0, 1, 1, 1];
        torch_activate(&mut g, chandra, 0);
        g.resolve_top();
        let object = g.object_ref(charm);
        act(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        act(&mut g, Action::ChooseMode { mode: 2 });
        pay(&mut g);
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
        assert_eq!(g.state.stack.last().unwrap().mode, Some(2));
        act(&mut g, Action::FinishResolutionCasts);
    }
}

#[test]
fn torch_emblem_ignores_opponent_casts_and_uncast_spell_copies() {
    for owner in [Player::P0, Player::P1] {
        let opponent = owner.other();
        let mut g = scenario(opponent);
        g.create_emblem(owner, Card::test_index(5465));
        let spell = put(&mut g, opponent, "Bear Cub", Zone::Hand);
        cast(&mut g, spell, None);
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.state.ordering_triggers.is_empty());
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, owner);
        g.state.stack.push(copy);
        g.prepare_triggers();
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.state.ordering_triggers.is_empty());
        assert_eq!(g.state.stack.len(), 2);
    }
}

#[test]
fn torch_emblem_trigger_obeys_countering_and_damage_prevention() {
    for owner in [Player::P0, Player::P1] {
        for counter in [false, true] {
            for prevent in [false, true] {
                let mut g = scenario(owner);
                g.create_emblem(owner, Card::test_index(5465));
                let emblem = g
                    .state
                    .objects
                    .iter()
                    .position(|o| o.zone == Zone::Command && o.card == Card::test_index(5465))
                    .unwrap() as CardId;
                let spell = put(&mut g, owner, "Bear Cub", Zone::Hand);
                let life = g.state.players[owner.other().index()].life;
                cast(&mut g, spell, None);
                g.prepare_triggers();
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(owner.other()),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                let trigger_id = g.state.stack.last().unwrap().id;
                if prevent {
                    let source = put(&mut g, owner.other(), "Forest", Zone::Battlefield);
                    let source = g.object_ref(source);
                    let recipient = g.object_ref(emblem);
                    let timestamp = g.next_role_timestamp();
                    g.state
                        .roles
                        .get_or_insert_default()
                        .source_linked_effects
                        .push(SourceLinkedEffectView {
                            source,
                            recipient,
                            controller: owner.other(),
                            kind: SourceLinkedEffectKind::PreventDamageBy,
                            timestamp,
                        });
                }
                let saved = g.snapshot();
                for _ in 0..2 {
                    if counter {
                        assert!(g.counter_stack_item(trigger_id, false));
                    } else {
                        g.resolve_top();
                    }
                    assert_eq!(
                        g.state.players[owner.other().index()].life,
                        life - if counter || prevent { 0 } else { 5 }
                    );
                    g.restore(&saved).unwrap();
                }
            }
        }
    }
}
