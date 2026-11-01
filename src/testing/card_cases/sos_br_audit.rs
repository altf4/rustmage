//! Focused printed outcomes; shared engine invariants live in core tests.
//! Printed obligations of the 25 black/red SOS members admitted before this set batch.
use super::*;
use big_helpers::{cast as paid, checked, settle as finish};
fn pay_checked(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, a);
}
fn until_choice(g: &mut Game) {
    for _ in 0..40 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing choice");
}
fn activate(g: &mut Game, id: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
#[test]
fn sos_audit_br_converge_uses_actual_colors_zero_one_five_and_available_hand() {
    {
        let p = Player::P0;
        for colors in [0, 1, 5] {
            for self_target in [false, true] {
                let mut g = scenario(p);
                let recipient = if self_target { p } else { p.other() };
                let a = put(&mut g, recipient, "Bear Cub", Zone::Hand);
                let b = put(&mut g, recipient, "Mountain", Zone::Hand);
                let spell = put(&mut g, p, "Arcane Omens", Zone::Hand);
                if colors == 0 {
                    put(&mut g, p, "Omniscience", Zone::Battlefield);
                    checked(
                        &mut g,
                        Action::AnnounceFree {
                            card: spell,
                            face: CastFace::Normal,
                            flash_surcharge: false,
                        },
                    );
                } else {
                    g.state.players[p.index()].mana = if colors == 1 {
                        [0, 0, 1, 0, 0, 4]
                    } else {
                        [1, 1, 1, 1, 1, 0]
                    };
                    checked(&mut g, Action::AnnounceCast { card: spell });
                }
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(recipient),
                    },
                );
                pay_checked(&mut g);
                assert_eq!(
                    g.state.stack.last().unwrap().cast.unwrap().colors_spent(),
                    colors
                );
                finish(&mut g);
                let discarded = [a, b]
                    .iter()
                    .filter(|&&id| g.object_view(id).zone == Zone::Graveyard)
                    .count();
                assert_eq!(discarded, (colors as usize).min(2));
            }
        }
    }
}
#[test]
fn sos_audit_br_draw_life_optional_counter_groups_and_partial_failure() {
    {
        let p = Player::P0;
        for target_creature in [false, true] {
            for leave in [false, true] {
                let mut g = scenario(p);
                let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let spell = put(&mut g, p, "Cost of Brilliance", Zone::Hand);
                let targets = if target_creature {
                    vec![Target::Player(p.other()), Target::Card(bear)]
                } else {
                    vec![Target::Player(p.other())]
                };
                paid(&mut g, spell, None, &targets);
                if leave {
                    g.move_object(bear, Zone::Hand);
                }
                finish(&mut g);
                assert_eq!(g.state.players[p.other().index()].life, 18);
                assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 2);
                assert_eq!(
                    g.counter_count(bear, CounterKind::PlusOne),
                    u64::from(target_creature && !leave)
                );
            }
        }
    }
}
#[test]
fn sos_audit_br_burrog_paid_repeat_pump_deathtouch_and_cleanup() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let frog = put(&mut g, p, "Burrog Banemaker", Zone::Hand);
        paid(&mut g, frog, None, &[]);
        finish(&mut g);
        assert_eq!(stats(&g, frog), (1, 1));

        for size in [2, 3] {
            activate(&mut g, frog);
            pay_checked(&mut g);
            finish(&mut g);
            assert_eq!(stats(&g, frog), (size, size));
        }
        g.finish_cleanup();
        assert_eq!(stats(&g, frog), (1, 1));
    }
}

#[test]
fn sos_audit_br_eternal_student_grave_exile_cost_frozen_tokens_and_zone_restriction() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let student = put(&mut g, p, "Eternal Student", Zone::Hand);
        paid(&mut g, student, None, &[]);
        finish(&mut g);
        assert_eq!(stats(&g, student), (4, 2));
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: student,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.move_object(student, Zone::Graveyard);
        activate(&mut g, student);
        pay_checked(&mut g);
        assert_eq!(g.object_view(student).zone, Zone::Exile);
        finish(&mut g);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
            .map(|(id, _)| id as CardId)
            .collect();
        assert_eq!(tokens.len(), 2);
        for id in tokens {
            assert_eq!(stats(&g, id), (1, 1));
            assert!(g.keywords(id).contains(Keywords::FLYING));
            assert_eq!(g.object_view(id).card.colors(), 0b00101);
        }
    }
}
#[test]
fn sos_audit_br_opus_all_three_consumers_use_paid_five_threshold_and_exclude_creatures() {
    {
        let p = Player::P0;
        for high in [false, true] {
            let mut g = scenario(p);
            let dancer = put(&mut g, p, "Expressive Firedancer", Zone::Hand);
            let artist = put(&mut g, p, "Tackle Artist", Zone::Hand);
            let drummer = put(&mut g, p, "Thunderdrum Soloist", Zone::Hand);
            for id in [dancer, artist, drummer] {
                paid(&mut g, id, None, &[]);
                finish(&mut g);
            }

            assert_eq!(g.counter_count(artist, CounterKind::PlusOne), 0);
            let spell = put(
                &mut g,
                p,
                if high { "Arcane Omens" } else { "Tome Blast" },
                Zone::Hand,
            );
            paid(&mut g, spell, None, &[Target::Player(p.other())]);
            finish(&mut g);
            assert_eq!(stats(&g, dancer), (3, 3));
            assert_eq!(g.keywords(dancer).contains(Keywords::DOUBLE_STRIKE), high);
            assert_eq!(
                g.counter_count(artist, CounterKind::PlusOne),
                if high { 2 } else { 1 }
            );
            assert_eq!(g.state.players[p.other().index()].life, 17);
            g.finish_cleanup();
            assert_eq!(stats(&g, dancer), (2, 2));
            assert!(!g.keywords(dancer).contains(Keywords::DOUBLE_STRIKE));
        }
    }
}
#[test]
fn sos_audit_br_repartee_mandatory_targets_frozen_source_and_one_event_per_cast() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let forum = put(&mut g, p, "Forum Necroscribe", Zone::Hand);
        let scorn = put(&mut g, p, "Lecturing Scornmage", Zone::Hand);
        let poet = put(&mut g, p, "Melancholic Poet", Zone::Hand);
        for id in [forum, scorn, poet] {
            paid(&mut g, id, None, &[]);
            finish(&mut g);
        }
        let dead = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Last Gasp", Zone::Hand);
        paid(&mut g, spell, None, &[Target::Card(enemy)]);
        g.move_object(poet, Zone::Exile);
        finish(&mut g);
        assert_eq!(g.object_view(dead).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(scorn, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        let no_creature = put(&mut g, p, "Tome Blast", Zone::Hand);
        paid(&mut g, no_creature, None, &[Target::Player(p.other())]);
        finish(&mut g);
        assert_eq!(g.counter_count(scorn, CounterKind::PlusOne), 1);
    }
}
#[test]
fn sos_audit_br_forum_ward_discard_pay_decline_unaffordable() {
    {
        let p = Player::P0;
        for choice in [0, 1, 2] {
            let mut g = scenario(p);
            let forum = put(&mut g, p.other(), "Forum Necroscribe", Zone::Battlefield);
            let spare = if choice < 2 {
                Some(put(&mut g, p, "Mountain", Zone::Hand))
            } else {
                None
            };
            let spell = put(&mut g, p, "Wander Off", Zone::Hand);
            paid(&mut g, spell, None, &[Target::Card(forum)]);
            if choice == 2 {
                // An empty hand cannot pay the ward cost, so no private choice opens.
                finish(&mut g);
                assert!(g.observe(p).effect_choice.is_none());
            } else {
                until_choice(&mut g);
                if choice == 0 {
                    checked(
                        &mut g,
                        Action::Discard {
                            card: spare.unwrap(),
                        },
                    );
                } else {
                    checked(&mut g, Action::DeclineEffectPayment);
                }
                finish(&mut g);
            }
            assert_eq!(
                g.object_view(forum).zone,
                if choice == 0 {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
            if choice == 0 {
                assert_eq!(g.object_view(spare.unwrap()).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn sos_audit_br_masterful_and_last_gasp_control_restriction_indestructible_and_toughness() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Masterful Flourish", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay_checked(&mut g);
        finish(&mut g);
        assert_eq!(stats(&g, own), (3, 2));
        assert!(g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
        let blast = put(&mut g, p, "Tome Blast", Zone::Hand);
        paid(&mut g, blast, None, &[Target::Card(own)]);
        finish(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        let gasp = put(&mut g, p, "Last Gasp", Zone::Hand);
        paid(&mut g, gasp, None, &[Target::Card(own)]);
        finish(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
    }
}
#[test]
fn sos_audit_br_infusion_entry_checks_resolution_and_live_scaling_resets_next_turn() {
    {
        let p = Player::P0;
        for timing in [0, 1, 2] {
            let mut g = scenario(p);
            let shop = put(&mut g, p, "Ulna Alley Shopkeep", Zone::Hand);
            paid(&mut g, shop, None, &[]);
            finish(&mut g);
            assert_eq!(stats(&g, shop), (2, 3));

            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let apprentice = put(&mut g, p, "Poisoner's Apprentice", Zone::Hand);
            if timing == 1 {
                g.gain_life(p, 1);
            }
            paid(&mut g, apprentice, None, &[]);
            until_choice(&mut g);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(shop),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            if timing == 2 {
                g.gain_life(p, 1);
            }
            finish(&mut g);
            assert_eq!(
                g.object_view(enemy).zone,
                if timing == 0 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(stats(&g, shop), if timing == 0 { (2, 3) } else { (4, 3) });
            g.finish_cleanup();
            assert_eq!(stats(&g, shop), (2, 3));
        }
    }
}

#[test]
fn sos_audit_br_paid_etb_flying_drain_haste_reach_and_instant_sorcery_recursion() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let vampire = put(&mut g, p, "Sneering Shadewriter", Zone::Hand);
        paid(&mut g, vampire, None, &[]);
        finish(&mut g);
        assert_eq!(stats(&g, vampire), (3, 3));

        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        let horse = put(&mut g, p, "Rearing Embermare", Zone::Hand);
        paid(&mut g, horse, None, &[]);
        finish(&mut g);
        assert_eq!(stats(&g, horse), (4, 5));
        assert!(
            g.keywords(horse)
                .contains(Keywords::HASTE.union(Keywords::REACH))
        );
        let instant = put(&mut g, p, "Last Gasp", Zone::Graveyard);
        let foreign = put(&mut g, p.other(), "Tome Blast", Zone::Graveyard);
        let creature = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let giant = put(&mut g, p, "Zealous Lorecaster", Zone::Hand);
        paid(&mut g, giant, None, &[]);
        until_choice(&mut g);
        for id in [foreign, creature] {
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
                target: Target::Card(instant),
            },
        );
        finish(&mut g);
        assert_eq!(g.object_view(instant).zone, Zone::Hand);
        assert_eq!(stats(&g, giant), (4, 4));
    }
}
#[test]
fn sos_audit_br_tome_flashback_fixed_five_and_exile_on_counter_or_resolution() {
    {
        let p = Player::P0;
        for countered in [false, true] {
            let mut g = scenario(p);
            let blast = put(&mut g, p, "Tome Blast", Zone::Hand);
            paid(&mut g, blast, None, &[Target::Player(p.other())]);
            finish(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 18);
            checked(&mut g, Action::AnnounceFlashback { card: blast });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay_checked(&mut g);
            assert_eq!(g.state.stack.last().unwrap().cast.unwrap().mana_spent(), 5);
            if countered {
                let counter = put(&mut g, p, "Negate", Zone::Hand);
                paid(&mut g, counter, None, &[Target::Spell(blast)]);
            }
            finish(&mut g);
            assert_eq!(g.object_view(blast).zone, Zone::Exile);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if countered { 18 } else { 16 }
            );
        }
    }
}

#[test]
fn sos_audit_br_wither_distinguishes_infusion_from_shrink_and_indestructibility() {
    {
        let p = Player::P0;
        for infusion in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let large = put(&mut g, p.other(), "Rearing Embermare", Zone::Battlefield);
            let indestructible = put(&mut g, p.other(), "Darksteel Colossus", Zone::Battlefield);
            if infusion {
                g.gain_life(p, 1);
            }
            let spell = put(&mut g, p, "Withering Curse", Zone::Hand);
            paid(&mut g, spell, None, &[]);
            finish(&mut g);
            assert_eq!(g.object_view(own).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(large).zone,
                if infusion {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(indestructible).zone, Zone::Battlefield);
            if !infusion {
                assert_eq!(stats(&g, large), (2, 3));
                let newcomer = put(&mut g, p, "Bear Cub", Zone::Hand);
                paid(&mut g, newcomer, None, &[]);
                finish(&mut g);
                assert_eq!(stats(&g, newcomer), (2, 2));
            }
        }
    }
}
