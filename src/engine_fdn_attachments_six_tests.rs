//! Six complete FDN cards and source-bound granted mana through paid owned replay.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. }
                    )
                })
                .unwrap_or_else(|| panic!("unexpected {:?}: {actions:?}", g.state.pending))
                .clone()
        };
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn target_trigger(g: &mut Game, id: CardId) {
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(id),
        })
    });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    until(g, |g| g.state.announcing.is_none());
}
fn activate(g: &mut Game, id: CardId, target: Option<CardId>, payment: Mana) {
    checked(
        g,
        Action::Activate {
            card: id,
            ability: if g.state.objects[id as usize].card.name() == "Cryptic Caves" {
                1
            } else {
                0
            },
            target: target.map(Target::Card),
            payment,
            counter_payment: Counters::default(),
        },
    );
}

#[test]
fn prayer_paid_flash_optional_exile_life_and_exact_link_return() {
    for p in [Player::P0, Player::P1] {
        for source_leaves in [false, true] {
            let mut g = setup(p);
            g.state.active = p.other(); // Flash really permits the opponent's turn.
            let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = named(&mut g, p, "Prayer of Binding", Zone::Hand);
            let life = g.state.players[p.index()].life;
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            cast(&mut g, spell, None, &[]);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                mana - 4
            );
            target_trigger(&mut g, victim);
            if source_leaves {
                g.move_object(spell, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, life + 2);
            assert_eq!(
                g.object_view(victim).zone,
                if source_leaves {
                    Zone::Battlefield
                } else {
                    Zone::Exile
                }
            );
            if !source_leaves {
                let exiled = g.object_ref(victim);
                g.move_object(spell, Zone::Graveyard);
                assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
                assert_ne!(g.object_ref(victim), exiled);
            }
        }
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let spell = named(&mut g, p, "Prayer of Binding", Zone::Hand);
        let life = g.state.players[p.index()].life;
        cast(&mut g, spell, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        for id in [own, land] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        checked(&mut g, Action::FinishTargets);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 2);
    }
}

#[test]
fn prayer_illegal_chosen_target_counters_the_whole_trigger_including_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Prayer of Binding", Zone::Hand);
        let life = g.state.players[p.index()].life;
        cast(&mut g, spell, None, &[]);
        target_trigger(&mut g, victim);
        g.move_object(victim, Zone::Hand);
        g.move_object(victim, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
    }
}

#[test]
fn starlight_paid_aura_taps_with_lki_and_prohibits_only_controller_untap_step() {
    for p in [Player::P0, Player::P1] {
        for leaves in [false, true] {
            let mut g = setup(p);
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let aura = named(&mut g, p, "Starlight Snare", Zone::Hand);
            cast(&mut g, aura, None, &[Target::Card(bear)]);
            until(&mut g, |g| {
                g.state
                    .stack
                    .last()
                    .is_some_and(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
            });
            if leaves {
                g.move_object(aura, Zone::Graveyard);
            }
            settle(&mut g);
            assert!(g.object_view(bear).tapped);
            g.untap_objects(&[bear], untap_rules::UntapCause::ControllerStep(p.other()));
            assert_eq!(g.object_view(bear).tapped, !leaves);
            g.untap_objects(&[bear], untap_rules::UntapCause::Effect);
            assert!(!g.object_view(bear).tapped);
            if !leaves {
                g.state.objects[bear as usize].controller = p;
                g.state.objects[bear as usize].tapped = true;
                g.untap_objects(&[bear], untap_rules::UntapCause::ControllerStep(p));
                assert!(g.object_view(bear).tapped);
            }
        }
        let mut g = setup(p);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let aura = named(&mut g, p, "Starlight Snare", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: aura });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
    }
}

#[test]
fn cutlass_paid_pirate_entry_attachment_and_sorcery_equip_any_owned_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pirate = named(&mut g, p, "Spectral Sailor", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Spectral Sailor", Zone::Battlefield);
        let cutlass = named(&mut g, p, "Pirate's Cutlass", Zone::Hand);
        cast(&mut g, cutlass, None, &[]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(pirate),
            })
        });
        for id in [bear, foe] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(pirate),
            },
        );
        settle(&mut g);
        assert_eq!(
            g.object_view(cutlass).attached_to,
            Some(g.object_ref(pirate))
        );
        assert_eq!((g.stats(pirate).power, g.stats(pirate).toughness), (3, 2));
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        activate(&mut g, cutlass, Some(bear), [2, 0, 0, 0, 0, 0]);
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            before - 2
        );
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (4, 3));
        assert_eq!(g.stats(pirate).power, 1);
        g.state.active = p.other();
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: cutlass,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.move_object(bear, Zone::Hand);
        g.move_object(bear, Zone::Battlefield);
        assert_ne!(g.object_view(cutlass).attached_to, Some(g.object_ref(bear)));
        assert_eq!(g.stats(bear).power, 2);
    }
}

#[test]
fn felidar_paid_up_to_two_other_targets_partial_illegality_and_lifelink() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = setup(p);
            let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let cat = named(&mut g, p, "Felidar Savior", Zone::Hand);
            cast(&mut g, cat, None, &[]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            for id in [cat, foe] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            if count > 0 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
            }
            if count > 1 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(b),
                    },
                );
            } else {
                checked(&mut g, Action::FinishTargets);
            }
            until(&mut g, |g| g.state.announcing.is_none());
            if count == 2 {
                g.move_object(b, Zone::Hand);
                g.move_object(b, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(g.object_view(a).counters.plus_one, u64::from(count > 0));
            assert_eq!(g.object_view(b).counters.plus_one, 0);
            assert!(g.keywords(cat).contains(Keywords::LIFELINK));
            let life = g.state.players[p.index()].life;
            let event = DamageEvent {
                preventable: true,
                source: g.damage_source(cat),
                target: Target::Player(p.other()),
                amount: 2,
                combat: false,
            };
            g.apply_damage_batch(&[event]);
            assert_eq!(g.state.players[p.index()].life, life + 2);
        }
    }
}

#[test]
fn caves_five_land_start_guard_cost_sacrifice_draw_private_and_captured_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let caves = named(&mut g, p, "Cryptic Caves", Zone::Battlefield);
        for _ in 0..3 {
            named(&mut g, p, "Forest", Zone::Battlefield);
        }
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: caves,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        let fifth = named(&mut g, p, "Forest", Zone::Battlefield);
        checked(
            &mut g,
            Action::ActivateMana {
                card: caves,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        assert!(g.object_view(caves).tapped);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: caves,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        g.state.objects[caves as usize].tapped = false;
        let hand = g.hand_count(p);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        activate(&mut g, caves, None, [1, 0, 0, 0, 0, 0]);
        assert_eq!(g.object_view(caves).zone, Zone::Graveyard);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 1
        );
        g.move_object(fifth, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .all(|o| o.zone != Zone::Hand || o.owner != p)
        );
    }
}

fn horizons(g: &mut Game, p: Player, land: CardId) -> CardId {
    let body = named(g, p, "Bear Cub", Zone::Battlefield);
    let aura = named(g, p, "New Horizons", Zone::Hand);
    cast(g, aura, None, &[Target::Card(land)]);
    target_trigger(g, body);
    settle(g);
    assert_eq!(g.object_view(body).counters.plus_one, 1);
    aura
}
#[test]
fn horizons_paid_counter_choice_two_mana_payment_reachability_control_and_departure() {
    for p in [Player::P0, Player::P1] {
        for own in [false, true] {
            let mut g = setup(p);
            let controller = if own { p } else { p.other() };
            let land = named(&mut g, controller, "Forest", Zone::Battlefield);
            let aura = horizons(&mut g, p, land);
            let grantor = g.object_ref(aura);
            g.state.priority = controller;
            g.state.players[controller.index()].mana = [0; 6];
            let grants: Vec<_> = g
                .legal_actions()
                .into_iter()
                .filter(|a| matches!(a,Action::TapGrantedMana{card,..}if *card==land))
                .collect();
            assert_eq!(grants.len(), 5);
            reject(
                &mut g,
                Action::TapGrantedMana {
                    card: land,
                    grantor,
                    production: [1, 1, 0, 0, 0, 0],
                },
            );
            reject(
                &mut g,
                Action::TapGrantedMana {
                    card: land,
                    grantor,
                    production: [0, 0, 0, 0, 0, 2],
                },
            );
            let action = Action::TapGrantedMana {
                card: land,
                grantor,
                production: [0, 2, 0, 0, 0, 0],
            };
            checked(&mut g, action.clone());
            assert_eq!(g.state.players[controller.index()].mana, [0, 2, 0, 0, 0, 0]);
            assert!(g.state.stack.is_empty());
            reject(&mut g, action);
            g.state.objects[land as usize].tapped = false;
            g.state.players[controller.index()].mana = [0; 6];
            g.move_object(aura, Zone::Graveyard);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::TapGrantedMana { .. }))
            );
        }
        let mut g = setup(p);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let aura = horizons(&mut g, p, land);
        let grantor = g.object_ref(aura);
        g.state.players[p.index()].mana = [0; 6];
        let spell = named(&mut g, p, "Negate", Zone::Hand);
        // A two-mana sorcery needs the grant even before its announcement is legal.
        let creature = named(&mut g, p, "Bear Cub", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: creature });
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: land,
                grantor,
                production: [0, 0, 0, 0, 2, 0],
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
    }
}

#[test]
fn horizons_layer_six_removal_timestamps_and_granted_mana_failure_atomicity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        g.state.objects[land as usize].tapped = false;
        let aura = horizons(&mut g, p, land);
        let grantor = g.object_ref(aura);
        g.state.players[p.index()].mana = [0; 6];
        let action = Action::TapGrantedMana {
            card: land,
            grantor,
            production: [0, 2, 0, 0, 0, 0],
        };
        assert!(g.legal_actions().contains(&action));
        g.state.players[p.index()].mana[1] = u32::MAX;
        let before = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(p, action),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
    }
}

#[test]
fn granted_mana_layer_six_obeys_recipient_removal_timestamps() {
    use super::ability_order::*;
    for grant_later in [false, true] {
        let source = ObjectRef {
            card: 0,
            generation: 1,
        };
        let grant = GrantedManaAbility {
            source,
            production: ManaProduction::Choice {
                mask: 31,
                amount: 2,
            },
        };
        let objects = vec![
            LayerSixObject {
                printed: true,
                keywords: Keywords::NONE,
                grants: Vec::new(),
                mana: Vec::new()
            };
            3
        ];
        let effects = [
            LayerSixEffect {
                provider: Some(0),
                started_earlier: false,
                timestamp: if grant_later { 3 } else { 1 },
                recipients: vec![],
                action: LayerSixAction::GrantEach(vec![LayerSixGrant {
                    recipient: 2,
                    keywords: Keywords::NONE,
                    abilities: Vec::new(),
                    mana: vec![grant],
                }]),
            },
            LayerSixEffect {
                provider: Some(1),
                started_earlier: false,
                timestamp: 2,
                recipients: vec![2],
                action: LayerSixAction::Remove,
            },
        ];
        let result = apply_layer_six(objects, &effects);
        assert!(!result.objects[2].printed);
        assert_eq!(result.objects[2].mana.len(), usize::from(grant_later));
    }
}

#[test]
fn horizons_mana_breaks_consecutive_passes_before_stack_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let aura = horizons(&mut g, p, land);
        let body = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(body)]);
        checked(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p.other()));
        let grantor = g.object_ref(aura);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: land,
                grantor,
                production: [0, 0, 0, 0, 2, 0],
            },
        );
        assert!(g.object_view(land).tapped);
        assert_eq!(g.state.passes, 0);
        checked(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        assert_eq!(g.stats(body).power, 2);
        checked(&mut g, Action::Pass);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.stats(body).power, 5);
    }
}
