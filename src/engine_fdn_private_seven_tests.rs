//! Complete private selection, milling, victim LKI and paid printed obligations.
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
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|action| {
                    matches!(
                        action,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. }
                    )
                })
                .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending))
        };
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn visible(g: &Game, viewer: Player, id: CardId) -> bool {
    g.observe(viewer)
        .objects
        .iter()
        .any(|object| object.reference.card == id)
}

#[test]
fn fdn_private_duress_and_pilfer_reveal_full_hand_and_require_legal_chooser_discard() {
    for p in [Player::P0, Player::P1] {
        for (name, allow_creature, cost) in [("Duress", false, 1), ("Pilfer", true, 2)] {
            let mut g = setup(p);
            let creature = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
            let land = named(&mut g, p.other(), "Forest", Zone::Hand);
            let instant = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
            let own = named(&mut g, p, "Giant Growth", Zone::Hand);
            let private_top = named(&mut g, p.other(), "Plains", Zone::Library);
            assert!(!visible(&g, p, creature));
            let spell = named(&mut g, p, name, Zone::Hand);
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            cast(&mut g, spell, None, &[Target::Player(p.other())]);
            assert_eq!(
                mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                cost
            );
            assert!(!visible(&g, p, instant));
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::RevealedDiscard { .. })
            });
            assert_eq!(g.actor(), Some(p));
            assert!(visible(&g, p, land) && visible(&g, p, creature) && visible(&g, p, instant));
            assert!(!visible(&g, p, private_top));
            let reference = g.object_ref(land);
            reject(&mut g, Action::ChooseRevealedDiscard { object: reference });
            let reference = g.object_ref(own);
            reject(&mut g, Action::ChooseRevealedDiscard { object: reference });
            if !allow_creature {
                let reference = g.object_ref(creature);
                reject(&mut g, Action::ChooseRevealedDiscard { object: reference });
            }
            let chosen = if allow_creature { creature } else { instant };
            let reference = g.object_ref(chosen);
            checked(&mut g, Action::ChooseRevealedDiscard { object: reference });
            assert_eq!(g.object_view(chosen).zone, Zone::Graveyard);
            assert_eq!(
                g.state.players[p.other().index()].cards_discarded_this_turn,
                1
            );
            assert!(visible(&g, p, land)); // Previously revealed exact identities remain known.
            g.move_object(land, Zone::Library);
            g.move_object(land, Zone::Hand);
            assert!(!visible(&g, p, land)); // New hidden incarnation is not permanently revealed.
            let new_private = named(&mut g, p.other(), "Island", Zone::Hand);
            assert!(!visible(&g, p, new_private));
        }
    }
}

#[test]
fn fdn_private_discard_no_matching_cards_finishes_and_capacity_is_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p.other(), "Forest", Zone::Hand);
        let creature = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let spell = named(&mut g, p, "Duress", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Hand);
        assert_eq!(g.object_view(creature).zone, Zone::Hand);
        assert!(visible(&g, p, land));
        let spell = named(&mut g, p, "Pilfer", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Player(p.other())]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::RevealedDiscard { .. })
        });
        g.state.players[p.other().index()].cards_discarded_this_turn = u64::MAX;
        let before = g.snapshot();
        let revision = g.revision();
        let object = g.object_ref(creature);
        assert!(matches!(
            g.apply(p, Action::ChooseRevealedDiscard { object }),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        for viewer in [p, p.other()] {
            assert!(visible(&g, viewer, creature));
        }
    }
}

#[test]
fn fdn_private_feed_swarm_paid_destroy_lki_loss_and_indestructible_invalid_target() {
    for p in [Player::P0, Player::P1] {
        for indestructible in [false, true] {
            let mut g = setup(p);
            let foe = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let spell = named(&mut g, p, "Feed the Swarm", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            until(&mut g, |g| g.state.announcing.is_none());
            if indestructible {
                let source = g.object_ref(own);
                g.grant_keyword_effects(&[foe], source, Keywords::INDESTRUCTIBLE);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 15);
            assert_eq!(g.state.players[p.index()].life_lost_this_turn, 5);
            assert_eq!(
                g.object_view(foe).zone,
                if indestructible {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
        let mut g = setup(p);
        let foe = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let spell = named(&mut g, p, "Feed the Swarm", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(foe)]);
        g.move_object(foe, Zone::Exile);
        g.move_object(foe, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}

#[test]
fn fdn_private_inspiration_recovers_old_or_new_grave_card_without_target_and_flashback_exiles() {
    for p in [Player::P0, Player::P1] {
        for old in [false, true] {
            let mut g = setup(p);
            let old_spell = named(&mut g, p, "Giant Growth", Zone::Graveyard);
            let new_spell = named(&mut g, p, "Duress", Zone::Library);
            named(&mut g, p, "Forest", Zone::Library);
            named(&mut g, p, "Bear Cub", Zone::Library);
            let foe = named(&mut g, p.other(), "Giant Growth", Zone::Graveyard);
            let spell = named(&mut g, p, "Inspiration from Beyond", Zone::Hand);
            cast(&mut g, spell, None, &[]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::MilledSelection { .. })
            });
            assert_eq!(g.object_view(new_spell).zone, Zone::Graveyard);
            reject(&mut g, Action::FinishMilledSelection);
            let reference = g.object_ref(foe);
            reject(&mut g, Action::SelectMilled { object: reference });
            let chosen = if old { old_spell } else { new_spell };
            let reference = g.object_ref(chosen);
            checked(&mut g, Action::SelectMilled { object: reference });
            assert_eq!(g.object_view(chosen).zone, Zone::Hand);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            checked(&mut g, Action::AnnounceFlashback { card: spell });
            until(&mut g, |g| g.state.announcing.is_none());
            assert_eq!(
                mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                7
            );
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
        }
    }
}

#[test]
fn fdn_private_dread_summons_counts_both_actual_milled_batches_and_tapped_zombies_x_zero() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 3, 30] {
            let mut g = setup(p);
            for who in [p, p.other()] {
                let library = g.state.players[who.index()].library.clone();
                for id in library {
                    g.move_object(id, Zone::Exile);
                }
                named(&mut g, who, "Bear Cub", Zone::Library);
                named(&mut g, who, "Forest", Zone::Library);
                named(&mut g, who, "Serra Angel", Zone::Library);
            }
            let spell = named(&mut g, p, "Dread Summons", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 2, 0, 0, x];
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseX { value: x });
            until(&mut g, |g| g.state.announcing.is_none());
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            let zombies: Vec<_> = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield && o.card == Card::catalogue().nth(5268).unwrap()
                })
                .collect();
            assert_eq!(zombies.len(), if x == 0 { 0 } else { 4 });
            assert!(zombies.iter().all(|o| o.tapped && o.controller == p));
            assert_eq!(
                g.state.players[p.index()].library.len(),
                if x == 0 { 3 } else { 0 }
            );
            assert_eq!(g.state.outcome, None); // Empty milling never draws or loses the game.
        }
    }
}

#[test]
fn fdn_private_trygon_only_combat_damage_bound_player_targets_optional_destroy_source_leaves() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let predator = named(&mut g, p, "Trygon Predator", Zone::Hand);
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            cast(&mut g, predator, None, &[]);
            assert_eq!(
                mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                3
            );
            settle(&mut g);
            assert!(g.keywords(predator).contains(Keywords::FLYING));
            let enemy = named(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            let own = named(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            let event = DamageEvent {
                source: g.damage_source(predator),
                target: Target::Player(p.other()),
                amount: 2,
                combat: false,
            };
            g.apply_damage_batch(&[event]);
            assert!(g.state.pending_triggers.is_empty());
            let event = DamageEvent {
                source: g.damage_source(predator),
                target: Target::Player(p.other()),
                amount: 2,
                combat: true,
            };
            g.apply_damage_batch(&[event]);
            g.prepare_triggers();
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            until(&mut g, |g| g.state.announcing.is_none());
            g.move_object(predator, Zone::Exile);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            checked(&mut g, Action::ChooseOptional { accept });
            settle(&mut g);
            assert_eq!(
                g.object_view(enemy).zone,
                if accept {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn fdn_private_massacre_wurm_etb_kills_opponents_only_and_simultaneous_lki_deaths_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let wurm = named(&mut g, p, "Massacre Wurm", Zone::Hand);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        cast(&mut g, wurm, None, &[]);
        assert_eq!(
            mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
            6
        );
        settle(&mut g);
        assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        let later = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert_eq!(
            g.stats(later),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        let sources = g.death_trigger_sources();
        let events = g.move_objects_raw(&[wurm, later, own], Zone::Graveyard);
        g.finish_zone_events(&events, &sources);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
