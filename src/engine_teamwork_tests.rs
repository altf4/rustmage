//! Teamwork costs through paid legal actions, signed power, visibility and owned replay.
use super::big_helpers::checked;
use super::*;

fn begin(g: &mut Game, card: CardId, paid: bool, target: CardId) {
    checked(g, Action::AnnounceCast { card });
    checked(g, Action::ChooseTeamwork { pay: paid });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
}
fn team(g: &mut Game, objects: &[CardId]) {
    assert!(matches!(g.state.pending, Pending::CastCosts { .. }));
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
        .position(|r| matches!(r.clause, CastCostClause::Teamwork(_)))
        .unwrap() as u16;
    checked(g, Action::ChooseCastCost { clause });
    for &id in objects {
        checked(
            g,
            Action::ChooseTeammate {
                object: g.object_ref(id),
            },
        );
    }
    checked(g, Action::FinishTeamwork);
}

#[test]
fn msh_teamwork_decline_and_pay_are_distinct_cast_choices() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = scenario(p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[bear as usize].summoning_sick = true;
            g.state.objects[bear as usize].gained_keywords = Keywords::HEXPROOF;
            let spell = put(&mut g, p, "Team Tactics", Zone::Hand);
            begin(&mut g, spell, paid, bear);
            if paid {
                team(&mut g, &[bear]);
            }
            assert_eq!(g.object_view(bear).tapped, paid);
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().teamwork,
                Some(paid)
            );
            settle(&mut g);
            assert!(g.keywords(bear).contains(Keywords::DOUBLE_STRIKE));
            assert_eq!(g.keywords(bear).contains(Keywords::TRAMPLE), paid);
        }
    }
}

#[test]
fn msh_teamwork_signed_power_invalid_choices_and_private_prefix() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let negative = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[negative as usize].growth.power = -5;
        let big = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let extra = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Team Tactics", Zone::Hand);
        begin(&mut g, spell, true, big);
        checked(&mut g, Action::ChooseCastCost { clause: 0 });
        let wrong = g.object_ref(enemy);
        reject(&mut g, Action::ChooseTeammate { object: wrong });
        let object = g.object_ref(negative);
        checked(&mut g, Action::ChooseTeammate { object });
        reject(&mut g, Action::ChooseTeammate { object });
        reject(&mut g, Action::FinishTeamwork);
        assert!(!g.object_view(negative).tapped);
        assert_eq!(
            g.observe(p).teamwork_choice.unwrap().selected_power,
            Some(-3)
        );
        assert!(g.observe(p.other()).teamwork_choice.is_none());
        let object = g.object_ref(big);
        checked(&mut g, Action::ChooseTeammate { object });
        reject(&mut g, Action::FinishTeamwork);
        let object = g.object_ref(extra);
        checked(&mut g, Action::ChooseTeammate { object });
        checked(&mut g, Action::FinishTeamwork);
        assert!(
            g.object_view(negative).tapped
                && g.object_view(big).tapped
                && g.object_view(extra).tapped
        );
        assert!(!g.object_view(enemy).tapped);
        settle(&mut g);
    }
}

#[test]
fn msh_teamwork_maria_triggers_only_for_successful_teamwork_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let maria = put(&mut g, p, "Agent Maria Hill", Zone::Battlefield);
        let spell = put(&mut g, p, "Team Tactics", Zone::Hand);
        begin(&mut g, spell, true, maria);
        let hand = g.hand_count(p);
        team(&mut g, &[maria]);
        assert_eq!(g.counter_count(maria, CounterKind::PlusOne), 0);
        settle(&mut g);
        assert_eq!(g.counter_count(maria, CounterKind::PlusOne), 1);
        assert_eq!(g.hand_count(p), hand + 1);
        g.state.objects[maria as usize].tapped = false;
        g.tap_objects(&[maria]);
        assert!(g.state.pending_triggers.is_empty());
    }
}

#[test]
fn msh_teamwork_target_expansion_and_all_invalid_fizzle() {
    for p in [Player::P0, Player::P1] {
        for invalid in [false, true] {
            let mut g = scenario(p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let big = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let spell = put(&mut g, p, "Cruel Alliance", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseTeamwork { pay: true });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(big),
                },
            );
            pay(&mut g);
            team(&mut g, &[bear]);
            if invalid {
                g.move_object(big, Zone::Hand);
                g.move_object(big, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(big).zone,
                if invalid {
                    Zone::Battlefield
                } else {
                    Zone::Exile
                }
            );
            assert_eq!(
                g.state.players[p.index()].life,
                if invalid { 20 } else { 23 }
            );
        }
    }
}

#[test]
fn msh_teamwork_normal_mana_value_restriction_remains_on_resolution() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let big = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut g, p, "Cruel Alliance", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: spell });
    checked(&mut g, Action::ChooseTeamwork { pay: false });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(big),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Exile);
    assert_eq!(g.state.players[p.index()].life, 20);
}

#[test]
fn msh_teamwork_only_expanded_targets_still_allow_announcement() {
    let p = Player::P0;
    let mut g = scenario(p);
    let big = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut g, p, "Cruel Alliance", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: spell });
    reject(&mut g, Action::ChooseTeamwork { pay: false });
    checked(&mut g, Action::ChooseTeamwork { pay: true });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(big),
        },
    );
    pay(&mut g);
    team(&mut g, &[big]);
    settle(&mut g);
    assert_eq!(g.object_view(big).zone, Zone::Exile);
}

#[test]
fn msh_teamwork_mana_and_tap_costs_cannot_double_spend_one_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.state.objects[elf as usize].growth.power = 1;
        let spell = put(&mut g, p, "Cruel Alliance", Zone::Hand);
        // The only creature can supply at most one of the two generic mana;
        // no illegal proposal should be offered, even before the teamwork choice.
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        g.state.players[p.index()].mana[2] = 2;
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::ChooseTeamwork { pay: true });
        assert!(!g.object_view(elf).tapped);
        checked(&mut g, Action::ChooseTeamwork { pay: false });
    }
}

#[test]
fn msh_teamwork_copied_spell_keeps_paid_choice_without_repaying_cost() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, p, "Team Tactics", Zone::Hand);
    begin(&mut g, spell, true, bear);
    team(&mut g, &[bear]);
    let sparks = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
    checked(&mut g, Action::AnnounceCast { card: sparks });
    checked(&mut g, Action::ChooseMode { mode: 0 });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(spell),
        },
    );
    pay(&mut g);
    for _ in 0..8 {
        if matches!(g.state.pending, Pending::Retarget { .. }) {
            break;
        }
        act(&mut g, Action::Pass);
    }
    assert!(matches!(g.state.pending, Pending::Retarget { .. }));
    checked(&mut g, Action::FinishStackTargets);
    let copies = g
        .state
        .stack
        .iter()
        .filter(|s| s.is_copy)
        .collect::<Vec<_>>();
    assert_eq!(copies.len(), 1);
    assert_eq!(copies[0].cast.unwrap().teamwork, Some(true));
    settle(&mut g);
    assert!(g.keywords(bear).contains(Keywords::TRAMPLE));
}

#[test]
fn msh_teamwork_paid_modal_spell_requires_both_and_revalidates_each_group() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = scenario(p);
            let giant = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Widow's Bite", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            assert!(matches!(g.state.pending, Pending::TeamworkIntent { .. }));
            checked(&mut g, Action::ChooseTeamwork { pay: paid });
            assert_eq!(g.observe(p).mode_choices.len(), if paid { 1 } else { 2 });
            reject(
                &mut g,
                Action::ChooseMode {
                    mode: if paid { 0 } else { 2 },
                },
            );
            checked(
                &mut g,
                Action::ChooseMode {
                    mode: if paid { 2 } else { 0 },
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(giant),
                },
            );
            if paid {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(bear),
                    },
                );
            }
            pay(&mut g);
            if paid {
                team(&mut g, &[giant]);
            }
            settle(&mut g);
            assert!(g.keywords(giant).contains(Keywords::DEATHTOUCH));
            assert_eq!(
                g.object_view(bear).zone,
                if paid {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
        let mut g = scenario(p);
        let giant = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Widow's Bite", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseTeamwork { pay: true });
        checked(&mut g, Action::ChooseMode { mode: 2 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(giant),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        team(&mut g, &[giant]);
        g.move_object(giant, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    }
}

#[test]
fn msh_teamwork_free_cast_still_pays_optional_nonmana_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Omniscience", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Team Tactics", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceFree {
                card: spell,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        checked(&mut g, Action::ChooseTeamwork { pay: true });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        team(&mut g, &[bear]);
        settle(&mut g);
        assert!(g.object_view(bear).tapped);
        assert!(g.keywords(bear).contains(Keywords::TRAMPLE));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
