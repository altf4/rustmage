//! Complete remaining expressible teamwork spells through paid actions and owned replay.
use super::big_helpers::checked;
use super::*;

fn cast_team(
    g: &mut Game,
    spell: CardId,
    paid: bool,
    mode: Option<u16>,
    targets: &[Target],
    helpers: &[CardId],
) {
    checked(g, Action::AnnounceCast { card: spell });
    checked(g, Action::ChooseTeamwork { pay: paid });
    if let Some(mode) = mode {
        reject(
            g,
            Action::ChooseMode {
                mode: if paid { 0 } else { 2 },
            },
        );
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
    let payment = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, payment);
    if paid {
        let clause = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|c| matches!(c.clause, CastCostClause::Teamwork(_)))
            .unwrap() as u16;
        checked(g, Action::ChooseCastCost { clause });
        for &id in helpers {
            {
                let object = g.object_ref(id);
                checked(g, Action::ChooseTeammate { object });
            }
        }
        checked(g, Action::FinishTeamwork);
    }
}
fn choice(g: &mut Game) {
    for _ in 0..12 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("no resolution choice");
}

#[test]
fn msh_teamwork_go_nuts_all_modes_and_fight_uses_post_counter_power() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let s = put(&mut g, p, "Go Nuts!", Zone::Hand);
            let targets = match mode {
                0 => vec![Target::Card(own)],
                1 => vec![Target::Card(own), Target::Card(enemy)],
                _ => vec![Target::Card(own), Target::Card(own), Target::Card(enemy)],
            };
            cast_team(&mut g, s, mode == 2, Some(mode), &targets, &[own]);
            settle(&mut g);
            assert_eq!(
                g.counter_count(own, CounterKind::PlusOne),
                if mode == 1 { 0 } else { 1 }
            );
            assert_eq!(
                g.object_view(enemy).zone,
                if mode == 0 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(own).zone,
                if mode == 1 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            if mode == 2 {
                assert_eq!(g.object_view(own).damage, 3);
            }
        }
    }
}

#[test]
fn msh_teamwork_go_nuts_partial_invalid_fight_needs_both_objects() {
    for invalid in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let counter = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let own = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let s = put(&mut g, p, "Go Nuts!", Zone::Hand);
        cast_team(
            &mut g,
            s,
            true,
            Some(2),
            &[
                Target::Card(counter),
                Target::Card(own),
                Target::Card(enemy),
            ],
            &[own],
        );
        let id = [counter, own, enemy][invalid];
        g.move_object(id, Zone::Hand);
        g.move_object(id, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(
            g.counter_count(counter, CounterKind::PlusOne),
            u64::from(invalid != 0)
        );
        if invalid != 0 {
            assert_eq!(g.object_view(own).damage, 0);
            assert_eq!(g.object_view(enemy).damage, 0);
        } else {
            assert_eq!(g.object_view(own).zone, Zone::Graveyard);
            assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn msh_teamwork_hulk_smash_all_modes_and_creature_is_damage_source() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Serra Angel", Zone::Battlefield);
            g.state.objects[own as usize].gained_keywords = Keywords::LIFELINK;
            let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let artifact = put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
            let s = put(&mut g, p, "HULK SMASH!", Zone::Hand);
            let targets = match mode {
                0 => vec![Target::Card(artifact)],
                1 => vec![Target::Card(own), Target::Card(enemy)],
                _ => vec![
                    Target::Card(artifact),
                    Target::Card(own),
                    Target::Card(enemy),
                ],
            };
            cast_team(&mut g, s, mode == 2, Some(mode), &targets, &[own]);
            settle(&mut g);
            assert_eq!(
                g.object_view(artifact).zone,
                if mode == 1 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(enemy).zone,
                if mode == 0 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.object_view(own).damage, 0);
            assert_eq!(
                g.state.players[p.index()].life,
                if mode == 0 { 20 } else { 24 }
            );
        }
    }
}

#[test]
fn msh_teamwork_hulk_smash_partial_invalid_and_relationship_revalidation() {
    for invalid in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let own = put(&mut g, p, "Serra Angel", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let artifact = put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let s = put(&mut g, p, "HULK SMASH!", Zone::Hand);
        cast_team(
            &mut g,
            s,
            true,
            Some(2),
            &[
                Target::Card(artifact),
                Target::Card(own),
                Target::Card(enemy),
            ],
            &[own],
        );
        if invalid == 0 {
            g.move_object(artifact, Zone::Hand);
        } else {
            g.state.objects[if invalid == 1 { own } else { enemy } as usize].controller =
                if invalid == 1 { p.other() } else { p };
        }
        settle(&mut g);
        assert_eq!(
            g.object_view(enemy).zone,
            if invalid == 0 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.object_view(artifact).zone,
            if invalid == 0 {
                Zone::Hand
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn msh_teamwork_murdock_all_modes_and_thresholds() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let helper = put(&mut g, p, "Serra Angel", Zone::Battlefield);
            let creature = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let enchantment = put(&mut g, p.other(), "Omniscience", Zone::Battlefield);
            let s = put(&mut g, p, "Murdock's Crusade", Zone::Hand);
            let targets = match mode {
                0 => vec![Target::Card(creature)],
                1 => vec![Target::Card(enchantment)],
                _ => vec![Target::Card(creature), Target::Card(enchantment)],
            };
            cast_team(&mut g, s, mode == 2, Some(mode), &targets, &[helper]);
            settle(&mut g);
            assert_eq!(
                g.object_view(creature).zone,
                if mode == 1 {
                    Zone::Battlefield
                } else {
                    Zone::Exile
                }
            );
            assert_eq!(
                g.object_view(enchantment).zone,
                if mode == 0 {
                    Zone::Battlefield
                } else {
                    Zone::Exile
                }
            );
        }
    }
    let p = Player::P0;
    let mut g = scenario(p);
    let small = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let big = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
    let s = put(&mut g, p, "Murdock's Crusade", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: s });
    checked(&mut g, Action::ChooseTeamwork { pay: false });
    checked(&mut g, Action::ChooseMode { mode: 0 });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(small),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(big),
        },
    );
    pay(&mut g);
    g.state.objects[big as usize].growth.toughness = -1;
    settle(&mut g);
    assert_eq!(g.object_view(big).zone, Zone::Battlefield);
}

#[test]
fn msh_teamwork_murdock_partial_invalid_enchantment_and_mana_value_floor() {
    let p = Player::P1;
    let mut g = scenario(p);
    let helper = put(&mut g, p, "Serra Angel", Zone::Battlefield);
    let creature = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
    let enchantment = put(&mut g, p.other(), "Omniscience", Zone::Battlefield);
    let low = put(&mut g, p.other(), "Phyrexian Arena", Zone::Battlefield);
    let s = put(&mut g, p, "Murdock's Crusade", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: s });
    checked(&mut g, Action::ChooseTeamwork { pay: true });
    checked(&mut g, Action::ChooseMode { mode: 2 });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(low),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enchantment),
        },
    );
    pay(&mut g);
    checked(&mut g, Action::ChooseCastCost { clause: 0 });
    {
        let object = g.object_ref(helper);
        checked(&mut g, Action::ChooseTeammate { object });
    }
    checked(&mut g, Action::FinishTeamwork);
    g.move_object(enchantment, Zone::Hand);
    settle(&mut g);
    assert_eq!(g.object_view(creature).zone, Zone::Exile);
    assert_eq!(g.object_view(low).zone, Zone::Battlefield);
}

#[test]
fn msh_teamwork_helicarrier_replaces_damage_and_requires_combatant() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for invalid in [false, true] {
                let mut g = scenario(p);
                let helper = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                let enemy = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
                g.state.objects[enemy as usize].growth.toughness = 3;
                let idle = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                g.state.active = p.other();
                g.state.step = Step::Blockers;
                g.state.attackers.push(Attacker {
                    card: enemy,
                    defender: Some(CombatDefender::Player(p)),
                    blocked: false,
                });
                let s = put(&mut g, p, "Helicarrier Strike", Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: s });
                checked(&mut g, Action::ChooseTeamwork { pay: paid });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(idle),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                pay(&mut g);
                if paid {
                    checked(&mut g, Action::ChooseCastCost { clause: 0 });
                    {
                        let object = g.object_ref(helper);
                        checked(&mut g, Action::ChooseTeammate { object });
                    }
                    checked(&mut g, Action::FinishTeamwork);
                }
                if invalid {
                    g.state.attackers.clear();
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(enemy).damage,
                    if invalid {
                        0
                    } else if paid {
                        4
                    } else {
                        2
                    }
                );
            }
        }
    }
}

#[test]
fn msh_teamwork_repulsor_controller_damage_after_lethal_and_all_invalid() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for invalid in [false, true] {
                let mut g = scenario(p);
                let helper = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let s = put(&mut g, p, "Repulsor Blast", Zone::Hand);
                cast_team(&mut g, s, paid, None, &[Target::Card(enemy)], &[helper]);
                if invalid {
                    g.move_object(enemy, Zone::Hand);
                    g.move_object(enemy, Zone::Battlefield);
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(enemy).zone,
                    if invalid {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    if paid && !invalid { 18 } else { 20 }
                );
            }
        }
    }
}

#[test]
fn msh_teamwork_reanimation_restrictions_entry_and_stale_object() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for invalid in [false, true] {
                let mut g = scenario(p);
                let helper = put(&mut g, p, "Serra Angel", Zone::Battlefield);
                let small = put(&mut g, p, "Bear Cub", Zone::Graveyard);
                let big = put(&mut g, p, "Serra Angel", Zone::Graveyard);
                let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
                let s = put(&mut g, p, "Too Evil to Stay Dead", Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: s });
                checked(&mut g, Action::ChooseTeamwork { pay: paid });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                if !paid {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(big),
                        },
                    );
                }
                let target = if paid { big } else { small };
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                pay(&mut g);
                if paid {
                    checked(&mut g, Action::ChooseCastCost { clause: 0 });
                    {
                        let object = g.object_ref(helper);
                        checked(&mut g, Action::ChooseTeammate { object });
                    }
                    checked(&mut g, Action::FinishTeamwork);
                }
                if invalid {
                    g.move_object(target, Zone::Hand);
                    g.move_object(target, Zone::Graveyard);
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(target).zone,
                    if invalid {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
                if !invalid {
                    assert_eq!(g.object_view(target).controller, p);
                    assert!(g.object_view(target).summoning_sick);
                }
            }
        }
    }
}

#[test]
fn msh_teamwork_nay_replaces_tax_and_payer_can_decline_or_pay() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for pays in [false, true] {
                let mut g = scenario(p);
                let helper = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                let victim = put(&mut g, p, "Shock", Zone::Hand);
                cast(&mut g, victim, Some(Target::Player(p.other())));
                let s = put(&mut g, p, "We Say Thee Nay!", Zone::Hand);
                cast_team(&mut g, s, paid, None, &[Target::Spell(victim)], &[helper]);
                choice(&mut g);
                let n = if paid { 4 } else { 2 };
                let view = g.observe(p).resolution_payment.unwrap();
                assert_eq!(view.payer, p);
                assert_eq!(view.cost.generic, n);
                reject(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, n - 1],
                    },
                );
                checked(
                    &mut g,
                    if pays {
                        Action::PayResolutionMana {
                            payment: [0, 0, 0, 0, 0, n],
                        }
                    } else {
                        Action::DeclineResolutionMana
                    },
                );
                assert!(!matches!(g.state.pending, Pending::ResolutionMana { .. }));
                settle(&mut g);
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    if pays { 18 } else { 20 }
                );
            }
        }
    }
}

#[test]
fn msh_teamwork_earths_public_reveal_optional_one_or_many_and_remainder() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for decline in [false, true] {
                let mut g = scenario(p);
                let helper = put(&mut g, p, "Serra Angel", Zone::Battlefield);
                let extra = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                let a = put(&mut g, p, "Bear Cub", Zone::Library);
                let b = put(&mut g, p, "Serra Angel", Zone::Library);
                let land = put(&mut g, p, "Forest", Zone::Library);
                let s = put(&mut g, p, "Earth's Mightiest Heroes", Zone::Hand);
                cast_team(&mut g, s, paid, None, &[], &[helper, extra]);
                choice(&mut g);
                let view = g.observe(p).library_selection.unwrap();
                let other = g.observe(p.other()).library_selection.unwrap();
                assert!(view.public && other.public);
                assert_eq!(view.remaining.len(), 8);
                assert_eq!(view.maximum, if paid { 2 } else { 1 });
                assert_eq!(
                    view.remaining
                        .iter()
                        .map(|o| o.reference)
                        .collect::<Vec<_>>(),
                    other
                        .remaining
                        .iter()
                        .map(|o| o.reference)
                        .collect::<Vec<_>>()
                );
                reject(&mut g, Action::SelectLibrary { card: land });
                if !decline {
                    checked(&mut g, Action::SelectLibrary { card: a });
                    reject(&mut g, Action::SelectLibrary { card: a });
                    if paid {
                        checked(&mut g, Action::SelectLibrary { card: b });
                    } else {
                        reject(&mut g, Action::SelectLibrary { card: b });
                    }
                }
                checked(&mut g, Action::FinishLibrarySelection);
                settle(&mut g);
                assert_eq!(
                    g.object_view(a).zone,
                    if decline {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
                assert_eq!(
                    g.object_view(b).zone,
                    if !decline && paid {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(g.object_view(land).zone, Zone::Graveyard);
                assert_eq!(g.state.players[p.index()].library.len(), 3);
            }
        }
    }
}

#[test]
fn msh_teamwork_atlantis_all_modes_one_or_two_bounces_and_token_recipe() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            for count in 1..=2 {
                let mut g = scenario(p);
                let helper = put(&mut g, p, "Serra Angel", Zone::Battlefield);
                let first = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let second = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
                g.state.objects[second as usize].controller = p.other();
                let spell = put(&mut g, p, "Atlantis Attacks", Zone::Hand);
                let mut targets = Vec::new();
                if mode != 1 {
                    targets.push(Target::Player(p.other()));
                }
                if mode != 0 {
                    targets.push(Target::Card(first));
                    if count == 2 {
                        targets.push(Target::Card(second));
                    }
                }
                cast_team(&mut g, spell, mode == 2, Some(mode), &targets, &[helper]);
                settle(&mut g);
                let tokens: Vec<_> = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| {
                        o.card == Card::MshLeviathanToken && o.zone == Zone::Battlefield
                    })
                    .map(|(id, _)| id as CardId)
                    .collect();
                assert_eq!(tokens.len(), usize::from(mode != 1));
                if let Some(&id) = tokens.first() {
                    let v = g.object_view(id);
                    assert_eq!(v.owner, p.other());
                    assert_eq!(v.controller, p.other());
                    assert_eq!(stats(&g, id), (6, 5));
                    assert!(g.keywords(id).contains(Keywords::HEXPROOF));
                    assert_eq!(g.object_view(id).colors, 2);
                    assert!(g.has_subtype(id, "Leviathan"));
                    assert_eq!(v.name(), "Leviathan Token");
                }
                assert_eq!(
                    g.object_view(first).zone,
                    if mode == 0 {
                        Zone::Battlefield
                    } else {
                        Zone::Hand
                    }
                );
                assert_eq!(
                    g.object_view(second).zone,
                    if mode != 0 && count == 2 {
                        Zone::Hand
                    } else {
                        Zone::Battlefield
                    }
                );
                if mode != 0 && count == 2 {
                    assert_eq!(g.object_view(second).owner, p);
                }
            }
        }
    }
}

#[test]
fn msh_teamwork_atlantis_rejects_land_duplicates_and_keeps_valid_bounce() {
    let p = Player::P0;
    let mut g = scenario(p);
    let helper = put(&mut g, p, "Serra Angel", Zone::Battlefield);
    let a = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
    let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
    let spell = put(&mut g, p, "Atlantis Attacks", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: spell });
    checked(&mut g, Action::ChooseTeamwork { pay: true });
    checked(&mut g, Action::ChooseMode { mode: 2 });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    reject(&mut g, Action::FinishTargets);
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
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    pay(&mut g);
    checked(&mut g, Action::ChooseCastCost { clause: 0 });
    {
        let object = g.object_ref(helper);
        checked(&mut g, Action::ChooseTeammate { object });
    }
    checked(&mut g, Action::FinishTeamwork);
    g.move_object(a, Zone::Hand);
    g.move_object(a, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.object_view(a).zone, Zone::Battlefield);
    assert_eq!(g.object_view(b).zone, Zone::Hand);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.card == Card::MshLeviathanToken
                && o.zone == Zone::Battlefield
                && o.controller == p)
            .count(),
        1
    );
}

#[test]
fn msh_teamwork_nay_taxes_opposing_spell_controller_and_invalid_spell_fizzles() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            for pays in [false, true] {
                let mut g = scenario(p);
                let helper = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                g.state.priority = p.other();
                let victim = put(&mut g, p.other(), "Shock", Zone::Hand);
                cast(&mut g, victim, Some(Target::Player(p)));
                checked(&mut g, Action::Pass);
                let spell = put(&mut g, p, "We Say Thee Nay!", Zone::Hand);
                cast_team(
                    &mut g,
                    spell,
                    paid,
                    None,
                    &[Target::Spell(victim)],
                    &[helper],
                );
                choice(&mut g);
                let n = if paid { 4 } else { 2 };
                assert_eq!(g.actor(), Some(p.other()));
                assert_eq!(g.observe(p).resolution_payment.unwrap().cost.generic, n);
                checked(
                    &mut g,
                    if pays {
                        Action::PayResolutionMana {
                            payment: [0, 0, 0, 0, 0, n],
                        }
                    } else {
                        Action::DeclineResolutionMana
                    },
                );
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].life, if pays { 18 } else { 20 });
            }
        }
    }
    let p = Player::P0;
    let mut g = scenario(p);
    let helper = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let victim = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, victim, Some(Target::Player(p.other())));
    let spell = put(&mut g, p, "We Say Thee Nay!", Zone::Hand);
    cast_team(
        &mut g,
        spell,
        true,
        None,
        &[Target::Spell(victim)],
        &[helper],
    );
    assert!(g.counter_stack_item(
        g.state.stack.iter().find(|s| s.card == victim).unwrap().id,
        false
    ));
    settle(&mut g);
    assert_eq!(g.state.players[p.other().index()].life, 20);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn msh_teamwork_power_up_and_plan_crossing_share_paid_cast_and_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let plan = put(&mut g, p, "Death to Our Enemies", Zone::Hand);
        super::big_helpers::cast(&mut g, plan, None, &[]);
        settle(&mut g);
        for _ in 0..3 {
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            super::big_helpers::cast(&mut g, shock, None, &[Target::Player(p.other())]);
            settle(&mut g);
        }
        assert_eq!(g.counter_count(plan, CounterKind::Plan), 3);
        let aerial = put(&mut g, p, "Aerial Doombot", Zone::Hand);
        super::big_helpers::cast(&mut g, aerial, None, &[]);
        settle(&mut g);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: aerial,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(g.announcement_payment_request(p).unwrap().generic, 5);
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        settle(&mut g);
        assert_eq!(g.object_view(aerial).power, 4);
        assert_eq!(g.counter_count(plan, CounterKind::Plan), 3);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, p, "Cruel Alliance", Zone::Hand);
        cast_team(&mut g, spell, true, None, &[Target::Card(enemy)], &[aerial]);
        for _ in 0..30 {
            if g.state.announcing.as_ref().is_some_and(|a| {
                a.item.card == plan && a.item.kind == (StackKind::TriggeredAbility { ability: 2 })
            }) {
                break;
            }
            let action = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                g.legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap()
            };
            checked(&mut g, action);
        }
        assert!(
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == plan
                    && a.item.kind == (StackKind::TriggeredAbility { ability: 2 }))
        );
        assert_eq!(g.object_view(plan).zone, Zone::Graveyard);
        assert_eq!(
            g.state
                .stack
                .iter()
                .find(|s| s.card == spell)
                .unwrap()
                .cast
                .unwrap()
                .teamwork,
            Some(true)
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(&mut g, Action::FinishTargets);
        reject(&mut g, Action::AllocateTargetAmount { amount: 6 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 7 });
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].life, 23);
        assert_eq!(g.state.players[p.other().index()].life, 7);
        assert!(g.object_view(aerial).tapped);
        assert_eq!(g.object_view(aerial).power_up_uses[0].count, 1);
        let treasure = Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap();
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card == treasure && o.controller == p)
            .collect();
        assert_eq!(tokens.len(), 4);
        assert!(tokens.iter().all(|o| o.tapped));
    }
}
