//! Paid whole-card behavior with existing owned raw/index replay helpers.
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
    for _ in 0..80 {
        if stop(g) {
            return;
        }
        let action = if g.state.pending == Pending::Priority {
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
                .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending))
        };
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn trigger_target(g: &mut Game, target: CardId) {
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target),
        })
    });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    until(g, |g| g.state.announcing.is_none());
}
fn token(index: usize) -> Card { Card::catalogue().nth(index).unwrap() }
fn tokens(g: &Game, p: Player, definition: Card) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.card == definition)
        .count()
}

#[test]
fn fdn_targeted_indrik_paid_optional_fight_uses_current_power_and_exact_source() {
    for p in [Player::P0, Player::P1] {
        for (accept, source_leaves) in [(false, false), (true, false), (true, true)] {
            let mut g = setup(p);
            let foe = named(
                &mut g,
                p.other(),
                "Quakestrider Ceratops",
                Zone::Battlefield,
            );
            let spell = named(&mut g, p, "Affectionate Indrik", Zone::Hand);
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            cast(&mut g, spell, None, &[]);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                mana - 6
            );
            trigger_target(&mut g, foe);
            // The exact live source uses its current power, not entry-time power.
            g.state.objects[spell as usize].growth.power = 20;
            if source_leaves {
                g.move_object(spell, Zone::Exile);
                g.move_object(spell, Zone::Battlefield);
                // The returned incarnation gets a distinct entry trigger; decline it.
                trigger_target(&mut g, foe);
                until(&mut g, |g| matches!(g.state.pending, Pending::EffectOptional { .. }));
                checked(&mut g, Action::ChooseOptional { accept: false });
            }
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            checked(&mut g, Action::ChooseOptional { accept });
            settle(&mut g);
            assert_eq!(
                g.object_view(foe).zone,
                if accept && !source_leaves {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.object_view(spell).zone,
                if accept && !source_leaves {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn fdn_targeted_indrik_targets_enemy_creatures_and_revalidates_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Affectionate Indrik", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        for bad in [own, land, spell] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foe),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        g.state.objects[foe as usize].controller = p;
        settle(&mut g);
        assert_eq!(g.object_view(foe).zone, Zone::Battlefield);
        assert_eq!(g.object_view(foe).damage, 0);
    }
}
#[test]
fn fdn_targeted_offer_counter_uses_spell_controller_not_owner_and_no_stale_tokens() {
    for p in [Player::P0, Player::P1] {
        for removed in [false, true] {
            let mut g = setup(p);
            // Foreign ownership is retained while the actual casting player gets tokens.
            let victim = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
            g.state.objects[victim as usize].controller = p;
            // Give the owner/caster an owned target and let that player cast normally.
            g.state.active = p.other();
            g.state.priority = p.other();
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            cast(&mut g, victim, None, &[Target::Card(bear)]);
            // Stack controller, rather than owner, is the authoritative recipient.
            g.state.stack.last_mut().unwrap().controller = p;
            checked(&mut g, Action::Pass);
            let offer = named(&mut g, p, "An Offer You Can't Refuse", Zone::Hand);
            if g.actor() != Some(p) {
                checked(&mut g, Action::Pass);
            }
            cast(&mut g, offer, None, &[Target::Spell(victim)]);
            if removed {
                g.move_object(victim, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            assert_eq!(tokens(&g, p, token(5302)), if removed { 0 } else { 2 });
            assert_eq!(tokens(&g, p.other(), token(5302)), 0);
            if !removed {
                if g.actor() != Some(p) { checked(&mut g, Action::Pass); }
                let treasure = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .find(|(_, o)| {
                        o.zone == Zone::Battlefield && o.controller == p && o.card == token(5302)
                    })
                    .unwrap()
                    .0 as CardId;
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==treasure))
                    .unwrap();
                checked(&mut g, action);
                assert_eq!(g.object_view(treasure).zone, Zone::Outside);
            }
        }
    }
}
#[test]
fn fdn_targeted_offer_rejects_creature_spells_and_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None, &[]);
        checked(&mut g, Action::Pass);
        let offer = named(&mut g, p.other(), "An Offer You Can't Refuse", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: offer });
        assert_eq!(g.object_view(bear).zone, Zone::Stack);
    }
}
#[test]
fn fdn_targeted_heartfire_prowess_paid_sacrifice_uses_last_power_and_planeswalker_target() {
    for p in [Player::P0, Player::P1] {
        for planeswalker in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Heartfire Immolator", Zone::Hand);
            cast(&mut g, source, None, &[]);
            settle(&mut g);
            let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
            cast(&mut g, growth, None, &[Target::Card(source)]);
            settle(&mut g);
            assert_eq!(g.stats(source).power, 6); // +1 prowess, +3 Growth
            let victim = named(
                &mut g,
                p.other(),
                if planeswalker {
                    "Ajani, Caller of the Pride"
                } else {
                    "Quakestrider Ceratops"
                },
                Zone::Battlefield,
            );
            if planeswalker {
                g.state.objects[victim as usize].counters.loyalty = 10;
            }
            let mana = g.state.players[p.index()].mana[3];
            let activation = Action::Activate {
                card: source,
                ability: 0,
                target: Some(Target::Card(victim)),
                payment: [0, 0, 0, 1, 0, 0],
                counter_payment: Counters::default(),
            };
            let mut wrong = activation.clone();
            if let Action::Activate { ref mut target, .. } = wrong {
                *target = Some(Target::Player(p.other()));
            }
            reject(&mut g, wrong);
            checked(&mut g, activation);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana[3], mana - 1);
            settle(&mut g);
            if planeswalker {
                assert_eq!(g.object_view(victim).counters.loyalty, 4);
            } else {
                assert_eq!(g.object_view(victim).damage, 6);
            }
        }
    }
}
#[test]
fn fdn_targeted_rite_checks_exact_life_at_resolution_and_can_target_self() {
    for p in [Player::P0, Player::P1] {
        for (initial, resolved) in [(9, 10), (10, 9), (10, 11), (11, 10)] {
            let mut g = setup(p);
            let spell = named(&mut g, p, "Hidetsugu's Second Rite", Zone::Hand);
            g.state.players[p.other().index()].life = initial;
            cast(&mut g, spell, None, &[Target::Player(p.other())]);
            g.state.players[p.other().index()].life = resolved;
            super::tests::pass_pair(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if resolved == 10 { 0 } else { resolved }
            );
            assert_eq!(
                g.outcome(),
                if resolved == 10 {
                    Some(Outcome::Winner(p))
                } else {
                    None
                }
            );
        }
        let mut g = setup(p);
        g.state.players[p.index()].life = 10;
        let spell = named(&mut g, p, "Hidetsugu's Second Rite", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Player(p)]);
        super::tests::pass_pair(&mut g);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
    }
}
#[test]
fn fdn_targeted_stroke_nonland_owner_controller_indestructible_and_target_invalidation() {
    for p in [Player::P0, Player::P1] {
        for (indestructible, removed) in [(false, false), (true, false), (false, true)] {
            let mut g = setup(p);
            let victim = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[victim as usize].controller = p.other();
            if indestructible {
                let protection = named(&mut g, p, "Adamant Will", Zone::Hand);
                cast(&mut g, protection, None, &[Target::Card(victim)]);
                settle(&mut g);
            }
            let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
            let spell = named(&mut g, p, "Stroke of Midnight", Zone::Hand);
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
                    target: Target::Card(victim),
                },
            );
            until(&mut g, |g| g.state.announcing.is_none());
            if removed {
                g.move_object(victim, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(tokens(&g, p.other(), token(5254)), usize::from(!removed));
            assert_eq!(tokens(&g, p, token(5254)), 0);
            assert_eq!(
                g.object_view(victim).zone,
                if removed {
                    Zone::Exile
                } else if indestructible {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn fdn_targeted_high_fae_flash_flying_and_cast_permission_follows_live_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: bear });
        let fae = named(&mut g, p, "High Fae Trickster", Zone::Hand);
        cast(&mut g, fae, None, &[]);
        settle(&mut g);
        assert!(g.keywords(fae).contains(Keywords::FLYING));
        if g.actor() != Some(p) { checked(&mut g, Action::Pass); }
        cast(&mut g, bear, None, &[]);
        settle(&mut g);
        if g.actor() != Some(p) { checked(&mut g, Action::Pass); }
        let sorcery = named(&mut g, p, "Divination", Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: sorcery })
        );
        g.state.objects[fae as usize].controller = p.other();
        reject(&mut g, Action::AnnounceCast { card: sorcery });
    }
}
#[test]
fn fdn_targeted_harbinger_optional_flash_surcharge_tapped_target_revalidation() {
    for p in [Player::P0, Player::P1] {
        for untapped_at_resolution in [false, true] {
            let mut g = setup(p);
            g.state.active = p.other();
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[foe as usize].tapped = true;
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[own as usize].tapped = true;
            let spell = named(&mut g, p, "Harbinger of the Tides", Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: spell });
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
            reject(&mut g, Action::AnnounceFlashSurcharge { card: spell });
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 2];
            checked(&mut g, Action::AnnounceFlashSurcharge { card: spell });
            until(&mut g, |g| g.state.announcing.is_none());
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
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
                    target: Target::Card(foe),
                },
            );
            until(&mut g, |g| g.state.announcing.is_none());
            if untapped_at_resolution {
                g.state.objects[foe as usize].tapped = false;
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(foe).zone,
                if untapped_at_resolution {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                }
            );
        }
    }
}

#[test]
fn fdn_targeted_heartfire_can_tap_mana_during_announced_sacrifice_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Heartfire Immolator", Zone::Battlefield);
        let mountain = named(&mut g, p, "Mountain", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
        checked(&mut g, Action::ChooseTarget { target: Target::Card(foe) });
        if g.legal_actions().contains(&Action::FinishTargets) { checked(&mut g, Action::FinishTargets); }
        checked(&mut g, Action::TapMana { card: mountain });
        let payment = g.legal_actions().into_iter().find(|a| matches!(a, Action::PayCosts { .. })).unwrap();
        checked(&mut g, payment);
        assert_eq!(g.state.players[p.index()].mana, [0;6]);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
    }
}
