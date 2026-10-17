//! Whole-card paid zone/control obligations with owned raw/index replay and restore.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::tests::spawn;
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
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. }
                    )
                })
                .unwrap_or_else(|| panic!("unexpected choice {:?}", g.state.pending))
        };
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn pay(g: &mut Game) {
    until(g, |g| g.state.announcing.is_none());
}
fn treasure(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.card
                    .token_info()
                    .is_some_and(|t| t.subtypes == "Treasure")
        })
        .count()
}
#[test]
fn fdn_zone_employment_paid_control_untap_haste_treasure_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[victim as usize].tapped = true;
        let spell = named(&mut g, p, "Involuntary Employment", Zone::Hand);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        cast(&mut g, spell, None, &[Target::Card(victim)]);
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 4
        );
        assert_eq!(g.object_view(victim).controller, p);
        assert_eq!(g.object_view(victim).owner, p.other());
        assert!(!g.object_view(victim).tapped);
        assert!(g.keywords(victim).contains(Keywords::HASTE));
        assert_eq!(treasure(&g, p), 1);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: victim });
        checked(&mut g, Action::FinishAttackers);
        assert!(g.state.attackers.iter().any(|a| a.card == victim));
        next_turn(&mut g);
        assert_eq!(g.object_view(victim).controller, p.other());
        assert!(!g.keywords(victim).contains(Keywords::HASTE));
        assert!(g.state.temporary_controls.is_none());
        assert_eq!(treasure(&g, p), 1);
    }
}
#[test]
fn fdn_zone_employment_overlapping_effects_combat_removal_and_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        for leaves in [false, true] {
            let mut g = setup(p);
            let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            // A controller change removes even an existing attacker from combat.
            g.state.attackers.push(AttackDeclaration {
                card: victim,
                defender: Some(CombatDefender::Player(p)),
                blocked: false,
            });
            let first = named(&mut g, p, "Involuntary Employment", Zone::Hand);
            cast(&mut g, first, None, &[Target::Card(victim)]);
            settle(&mut g);
            assert!(g.state.attackers.iter().all(|a| a.card != victim));
            let second = named(&mut g, p, "Involuntary Employment", Zone::Hand);
            cast(&mut g, second, None, &[Target::Card(victim)]);
            settle(&mut g);
            assert_eq!(treasure(&g, p), 2);
            if leaves {
                let old = g.object_ref(victim);
                g.move_object(victim, Zone::Exile);
                g.move_object(victim, Zone::Battlefield);
                assert_ne!(g.object_ref(victim), old);
                assert_eq!(g.object_view(victim).controller, p.other());
                assert!(!g.keywords(victim).contains(Keywords::HASTE));
            }
            next_turn(&mut g);
            assert_eq!(g.object_view(victim).controller, p.other());
            assert!(g.state.temporary_controls.is_none());
        }
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Involuntary Employment", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(victim)]);
        g.move_object(victim, Zone::Exile);
        settle(&mut g);
        assert_eq!(treasure(&g, p), 0); // All targets illegal: no independent Treasure.
    }
}
#[test]
fn fdn_zone_banshee_morbid_checked_at_resolution_and_opponent_only() {
    for p in [Player::P0, Player::P1] {
        for death in [false, true] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let victim = named(
                &mut g,
                p.other(),
                "Quakestrider Ceratops",
                Zone::Battlefield,
            );
            let spell = named(&mut g, p, "Tragic Banshee", Zone::Hand);
            cast(&mut g, spell, None, &[]);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(victim),
                })
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
                    target: Target::Card(victim),
                },
            );
            pay(&mut g);
            assert_eq!(g.state.creatures_died_this_turn, 0);
            if death {
                g.move_object(own, Zone::Graveyard);
            }
            settle(&mut g);
            if death {
                assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            } else {
                assert_eq!(g.stats(victim).power, 11);
                assert_eq!(g.stats(victim).toughness, 7);
                next_turn(&mut g);
                assert_eq!(g.stats(victim).power, 12);
                assert_eq!(g.stats(victim).toughness, 8);
            }
        }
    }
}
#[test]
fn fdn_zone_primal_paid_x_optional_fight_boost_precedes_damage() {
    for p in [Player::P0, Player::P1] {
        for (x, fight) in [(0, false), (3, false), (3, true)] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = named(&mut g, p, "Primal Might", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, x];
            checked(&mut g, Action::AnnounceCast { card: spell });
            reject(&mut g, Action::ChooseX { value: x + 1 });
            checked(&mut g, Action::ChooseX { value: x });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            if fight {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foe),
                    },
                );
            }
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.stats(own).power, 2 + i128::from(x));
            assert_eq!(
                g.object_view(foe).zone,
                if fight {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(own).damage, if fight { 2 } else { 0 });
            next_turn(&mut g);
            assert_eq!(g.stats(own).power, 2);
            assert_eq!(g.object_view(own).damage, 0);
        }
    }
}
#[test]
fn fdn_zone_primal_partial_targets_preserve_boost_but_never_fight_stale_objects() {
    for p in [Player::P0, Player::P1] {
        for remove_own in [false, true] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = named(&mut g, p, "Primal Might", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseX { value: 3 });
            checked(
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
            pay(&mut g);
            let moved = if remove_own { own } else { foe };
            g.move_object(moved, Zone::Exile);
            g.move_object(moved, Zone::Battlefield);
            settle(&mut g);
            assert_eq!(g.object_view(own).zone, Zone::Battlefield);
            assert_eq!(g.object_view(foe).zone, Zone::Battlefield);
            assert_eq!(g.stats(own).power, if remove_own { 2 } else { 5 });
            assert_eq!(g.object_view(own).damage, 0);
            assert_eq!(g.object_view(foe).damage, 0);
        }
    }
}
#[test]
fn fdn_zone_run_away_target_controllers_rechecked_with_departed_last_controller() {
    for p in [Player::P0, Player::P1] {
        for (change, leave) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let other_own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = named(&mut g, p, "Run Away Together", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(other_own),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            pay(&mut g);
            if change {
                g.state.objects[foe as usize].controller = p;
            }
            if leave {
                g.move_object(foe, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(own).zone,
                if change {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                }
            );
            assert_eq!(
                g.object_view(foe).zone,
                if leave {
                    Zone::Graveyard
                } else if change {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                }
            );
            if !change {
                assert!(
                    g.observe(p.other())
                        .known_hand
                        .iter()
                        .any(|r| r.card == own)
                );
            }
        }
    }
}
#[test]
fn fdn_zone_raise_returns_only_own_mv_two_or_less_simultaneously_with_entry_triggers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Dwynen's Elite", Zone::Graveyard);
        let b = named(&mut g, p, "Dwynen's Elite", Zone::Graveyard);
        let bear = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let high = named(&mut g, p, "Elvish Archdruid", Zone::Graveyard);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let land = named(&mut g, p, "Forest", Zone::Graveyard);
        let old = g.object_ref(a);
        let spell = named(&mut g, p, "Raise the Past", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        settle(&mut g);
        for id in [a, b, bear] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(g.object_view(id).controller, p);
        }
        assert_ne!(g.object_ref(a), old);
        for id in [high, foe, land] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        // Each Elite sees the other in the simultaneous entry event.
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card.is_token()
                    && o.card.has_subtype("Elf"))
                .count(),
            2
        );
    }
}
#[test]
fn fdn_zone_rise_returns_both_owners_under_caster_and_entry_counters_static_layers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let captain = named(&mut g, p.other(), "Corsair Captain", Zone::Graveyard);
        let pirate = named(&mut g, p, "Spectral Sailor", Zone::Graveyard);
        let counters = named(&mut g, p.other(), "Heroes' Bane", Zone::Graveyard);
        let stone = named(&mut g, p, "Burdened Stoneback", Zone::Graveyard);
        let land = named(&mut g, p.other(), "Forest", Zone::Graveyard);
        let spell = named(&mut g, p, "Rise of the Dark Realms", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        settle(&mut g);
        for id in [captain, pirate, counters, stone] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(g.object_view(id).controller, p);
        }
        assert_eq!(g.object_view(captain).owner, p.other());
        assert_eq!(g.stats(pirate).power, 2);
        assert_eq!(g.stats(pirate).toughness, 2);
        assert_eq!(g.object_view(stone).counters.minus_one, 2);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        g.move_object(captain, Zone::Hand);
        assert_eq!(g.object_view(captain).controller, p.other());
        assert_eq!(g.stats(pirate).power, 1);
    }
}
#[test]
fn fdn_zone_pulse_same_names_across_controllers_non_target_protection_and_token_factories() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let c = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.grant_keyword_effects(&[b], g.object_ref(b), Keywords::HEXPROOF);
        g.grant_keyword_effects(&[c], g.object_ref(c), Keywords::INDESTRUCTIBLE);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let spell = named(&mut g, p, "Maelstrom Pulse", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.object_view(c).zone, Zone::Battlefield);
        let mut groups = std::collections::BTreeMap::<&str, Vec<Card>>::new();
        for card in Card::catalogue() {
            if card.is_supported()
                && let Some(t) = card.token_info()
                && card.is_creature()
            {
                groups.entry(t.name).or_default().push(card);
            }
        }
        let definitions = groups
            .values()
            .find(|v| v.len() > 1)
            .expect("two token factories sharing an actual name");
        let ta = spawn(&mut g, p, definitions[0], Zone::Battlefield);
        let tb = spawn(&mut g, p.other(), definitions[1], Zone::Battlefield);
        assert_ne!(definitions[0], definitions[1]);
        let spell = named(&mut g, p, "Maelstrom Pulse", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(ta)]);
        settle(&mut g);
        assert_ne!(g.object_view(ta).zone, Zone::Battlefield);
        assert_ne!(g.object_view(tb).zone, Zone::Battlefield);
    }
}
#[test]
fn fdn_zone_pulse_illegal_target_does_not_destroy_other_same_named_permanents() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Maelstrom Pulse", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(a)]);
        g.move_object(a, Zone::Exile);
        g.move_object(a, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        assert_eq!(g.object_view(b).zone, Zone::Battlefield);
    }
}
