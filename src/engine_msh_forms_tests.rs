//! Whole-card paid rules evidence for Panther Pounce, Super Suit and Super-Skrull.
use super::assurance::big_helpers::{checked, named, next_turn, settle, setup};
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
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::ChooseActivationCost { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("unfinished payment");
}
fn cast(g: &mut Game, card: CardId, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card });
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let card = named(g, p, name, Zone::Hand);
    cast(g, card, &[]);
    settle(g);
    card
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..50 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn activate(g: &mut Game, card: CardId, ability: u16, target: Option<Target>) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            let id = id as CardId;
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id, subtype))
            .then_some(id)
        })
        .collect()
}
fn visible(g: &Game, viewer: Player, id: CardId) -> bool {
    g.observe(viewer)
        .objects
        .iter()
        .any(|o| o.reference.card == id)
}
fn attackers(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
}

#[test]
fn msh_forms_panther_pounce_targeted_investigate_buff_untap_expiry_and_paid_clue() {
    for p in [Player::P0, Player::P1] {
        for recipient in [p, p.other()] {
            let mut g = setup(p);
            let creature = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.objects[creature as usize].tapped = true;
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            let card = named(&mut g, p, "Panther Pounce", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(recipient),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert_eq!(
                g.stats(creature),
                Stats {
                    power: 4,
                    toughness: 3
                }
            );
            assert!(g.keywords(creature).contains(Keywords::FLYING));
            assert!(!g.object_view(creature).tapped);
            let clues = tokens(&g, recipient, "Clue");
            assert_eq!(clues.len(), 1);
            assert!(tokens(&g, recipient.other(), "Clue").is_empty());
            if g.actor() != Some(recipient) {
                checked(&mut g, Action::Pass);
            }
            let hand = g.hand_count(recipient);
            let drawn = *g.state.players[recipient.index()].library.last().unwrap();
            g.state.players[recipient.index()].mana = [0, 0, 0, 0, 0, 2];
            activate(&mut g, clues[0], 0, None);
            settle(&mut g);
            assert_eq!(g.state.players[recipient.index()].mana, [0; 6]);
            assert_eq!(g.hand_count(recipient), hand + 1);
            assert_eq!(g.object_view(clues[0]).zone, Zone::Outside);
            assert!(visible(&g, recipient, drawn));
            assert!(!visible(&g, recipient.other(), drawn));
            next_turn(&mut g);
            assert_eq!(
                g.stats(creature),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
            assert!(!g.keywords(creature).contains(Keywords::FLYING));
        }
    }
}

#[test]
fn msh_forms_panther_pounce_partial_targets_and_token_replacement_use_target_player() {
    for p in [Player::P0, Player::P1] {
        for invalid in 0..4 {
            let mut g = setup(p);
            // Only the chosen recipient's replacement may double investigation.
            named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            let creature = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.state.objects[creature as usize].tapped = true;
            let card = named(&mut g, p, "Panther Pounce", Zone::Hand);
            cast(
                &mut g,
                card,
                &[Target::Player(p.other()), Target::Card(creature)],
            );
            if invalid & 1 != 0 {
                // A live public player-hexproof fixture invalidates only group0.
                named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
            }
            if invalid & 2 != 0 {
                g.move_object(creature, Zone::Hand);
                g.move_object(creature, Zone::Battlefield);
                g.state.objects[creature as usize].tapped = true;
            }
            settle(&mut g);
            assert_eq!(
                tokens(&g, p.other(), "Clue").len(),
                if invalid & 1 == 0 { 2 } else { 0 }
            );
            assert_eq!(
                g.stats(creature).power,
                if invalid & 2 == 0 { 4 } else { 3 }
            );
            assert_eq!(
                g.keywords(creature).contains(Keywords::FLYING),
                invalid & 2 == 0
            );
            assert_eq!(g.object_view(creature).tapped, invalid & 2 != 0);
            assert_eq!(g.object_view(card).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn msh_forms_super_suit_flash_attach_untap_exact_source_and_controller_changes() {
    for p in [Player::P0, Player::P1] {
        for change in 0..4 {
            let mut g = setup(p);
            let creature = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            g.state.objects[creature as usize].tapped = true;
            g.state.active = p.other();
            g.state.priority = p;
            let suit = named(&mut g, p, "Super Suit", Zone::Hand);
            cast(&mut g, suit, &[]);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(creature),
                })
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
                    target: Target::Card(land),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            finish(&mut g);
            match change {
                1 => {
                    g.move_object(suit, Zone::Graveyard);
                }
                2 => {
                    g.move_object(creature, Zone::Hand);
                    g.move_object(creature, Zone::Battlefield);
                    g.state.objects[creature as usize].tapped = true;
                }
                3 => {
                    let object = g.object_ref(suit);
                    g.change_controller(object, p.other());
                }
                _ => {}
            }
            settle(&mut g);
            assert_eq!(g.object_view(creature).tapped, change == 2);
            assert_eq!(
                g.stats(creature),
                if change == 0 || change == 3 {
                    Stats {
                        power: 4,
                        toughness: 5,
                    }
                } else {
                    Stats {
                        power: 3,
                        toughness: 3,
                    }
                }
            );
            assert_eq!(
                g.object_view(suit).attached_to,
                if change == 0 || change == 3 {
                    Some(g.object_ref(creature))
                } else {
                    None
                }
            );
        }
    }
}

#[test]
fn msh_forms_super_suit_equip_paid_sorcery_retarget_static_bonus_and_layers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let second = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let suit = named(&mut g, p, "Super Suit", Zone::Hand);
        cast(&mut g, suit, &[]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(first),
            })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        finish(&mut g);
        settle(&mut g);
        let equip = Action::AnnounceActivate {
            card: suit,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.active = p.other();
        g.state.priority = p;
        reject(&mut g, equip.clone());
        g.state.active = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        g.state.objects[second as usize].tapped = true;
        checked(&mut g, equip);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(second),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.stats(first),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert_eq!(
            g.stats(second),
            Stats {
                power: 4,
                toughness: 5
            }
        );
        assert!(g.object_view(second).tapped); // Equip itself does not untap.
        let reference = g.object_ref(second);
        g.change_controller(reference, p.other());
        assert_eq!(
            g.stats(second),
            Stats {
                power: 4,
                toughness: 5
            }
        );
        g.state.players[p.index()].mana = [30; 6];
        let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        cast(&mut g, moon, &[Target::Card(second)]);
        settle(&mut g);
        assert_eq!(g.current_kinds(second), 2);
        assert_eq!(g.object_view(suit).attached_to, None);
    }
}

#[test]
fn msh_forms_super_skrull_flying_wall_recipe_mana_repeatability_and_defender() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Super-Skrull");
        assert_eq!(
            g.stats(source),
            Stats {
                power: 4,
                toughness: 5
            }
        );
        assert!(g.keywords(source).contains(Keywords::FLYING));
        g.state.objects[source as usize].tapped = true;
        g.state.objects[source as usize].summoning_sick = true;
        for _ in 0..2 {
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
            activate(&mut g, source, 0, None);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
        let walls = tokens(&g, p, "Wall");
        assert_eq!(walls.len(), 2);
        for &wall in &walls {
            assert_eq!(
                g.stats(wall),
                Stats {
                    power: 0,
                    toughness: 4
                }
            );
            assert_eq!(g.object_view(wall).colors, 0);
            assert!(g.keywords(wall).contains(Keywords::DEFENDER));
            assert!(!g.object_view(wall).card.info().legendary);
            g.state.objects[wall as usize].summoning_sick = false;
        }
        attackers(&mut g, p);
        reject(&mut g, Action::Attack { card: walls[0] });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        let attacker = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        attackers(&mut g, p.other());
        checked(&mut g, Action::Attack { card: attacker });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: walls[0],
                attacker,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}

#[test]
fn msh_forms_super_skrull_pump_repeatability_cleanup_and_stale_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Super-Skrull");
        for (power, toughness) in [(8, 9), (12, 13)] {
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
            activate(&mut g, source, 1, None);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.stats(source), Stats { power, toughness });
        }
        next_turn(&mut g);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 4,
                toughness: 5
            }
        );
        g.state.priority = p;
        g.state.players[p.index()].mana = [30; 6];
        activate(&mut g, source, 1, None);
        let original = g.object_ref(source);
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        settle(&mut g);
        assert_ne!(g.object_ref(source), original);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 4,
                toughness: 5
            }
        );
    }
}

#[test]
fn msh_forms_super_skrull_damage_targets_and_departure_lki_without_rebinding() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Super-Skrull");
            let victim = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 2,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
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
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            g.move_object(source, Zone::Graveyard);
            if blink {
                g.move_object(victim, Zone::Hand);
                g.move_object(victim, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(g.object_view(victim).damage, if blink { 0 } else { 4 });
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn msh_forms_super_skrull_target_player_draw_four_privacy_controller_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for recipient in [p, p.other()] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Super-Skrull");
            let hidden = named(&mut g, recipient, "Hill Giant", Zone::Hand);
            let top = *g.state.players[recipient.index()].library.last().unwrap();
            let hand = g.hand_count(recipient);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 5];
            activate(&mut g, source, 3, Some(Target::Player(recipient)));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let reference = g.object_ref(source);
            g.change_controller(reference, p.other());
            g.move_object(source, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.hand_count(recipient), hand + 4);
            assert!(visible(&g, recipient, top));
            assert!(!visible(&g, recipient.other(), top));
            assert!(!visible(&g, recipient.other(), hidden));
        }
    }
}

#[test]
fn msh_forms_super_skrull_wall_controller_capture_and_illegal_mana_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Super-Skrull");
        g.state.players[p.index()].mana = [0; 6];
        for ability in 0..4 {
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability,
                    counter_payment: Counters::default(),
                },
            );
        }
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
        activate(&mut g, source, 0, None);
        let reference = g.object_ref(source);
        g.change_controller(reference, p.other());
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(tokens(&g, p, "Wall").len(), 1);
        assert!(tokens(&g, p.other(), "Wall").is_empty());
    }
}
