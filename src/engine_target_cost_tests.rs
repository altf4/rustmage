use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 19] = [
    "Ajani's Response",
    "Brush Off",
    "Dark Endurance",
    "Depower",
    "Dire Downdraft",
    "Fate of the Sun-Cryst",
    "Grounded for Life",
    "Grow Extra Arms",
    "Luminous Rebuke",
    "Magnificent End",
    "Out of Air",
    "Quicksand Whirlpool",
    "Ride's End",
    "Roadside Blowout",
    "Run Behind",
    "Run Over",
    "Seized from Slumber",
    "Swampsnare Trap",
    "Uneasy Partings",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn replay(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    assert!(
        g.legal_actions().contains(&a),
        "illegal {a:?}: {:?}",
        g.state.pending
    );
    let old = g.snapshot();
    let rev = g.revision();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(f.snapshot(), expected);
    g.restore(&old).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let old = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), old);
    assert_eq!(g.revision(), rev);
}
fn finish(g: &mut Game) {
    for _ in 0..100 {
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseMode { mode: 0 }
                        | Action::ChooseSpellCost { cost: 0 }
                        | Action::ChooseOffspring { pay: false }
                        | Action::ChooseForageRoute {
                            route: ForageRoute::Black
                        }
                )
            })
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::ChooseTarget { .. }
                            | Action::FinishTargets
                            | Action::ChooseAdditionalCost { .. }
                            | Action::ChooseForage { .. }
                            | Action::PayCosts { .. }
                    )
                })
            })
            .unwrap_or_else(|| panic!("announcement {:?}", actions));
        replay(g, a.clone());
    }
    panic!("stuck announcement");
}
fn resolve(g: &mut Game) {
    for _ in 0..100 {
        if g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        if g.state.announcing.is_some() {
            finish(g);
            continue;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::ChooseLibraryDestination { .. }
                        | Action::ArrangeLibrary { .. }
                )
            })
            .unwrap();
        replay(g, a);
    }
    panic!("stuck resolution");
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g
}
fn qualifier(g: &mut Game, p: Player, i: usize) -> CardId {
    if matches!(i, 1 | 10) {
        let id = named(
            g,
            p.other(),
            if i == 1 { "Shock" } else { "Bear Cub" },
            Zone::Hand,
        );
        let item = g.cast_item(p.other(), id, [0; 6]);
        g.state.stack.push(item);
        g.state.priority = p;
        return id;
    }
    let id = named(
        g,
        if i == 15 { p } else { p.other() },
        if i == 7 || i == 15 {
            "Three Tree Mascot"
        } else if i == 13 {
            "Llanowar Elves"
        } else if i == 17 {
            "Malevolent Chandelier"
        } else {
            "Bear Cub"
        },
        Zone::Battlefield,
    );
    if matches!(i, 0 | 4 | 5 | 6 | 8 | 9 | 11 | 12 | 16) {
        g.state.objects[id as usize].tapped = true;
    }
    if matches!(i, 3 | 4 | 14 | 18) {
        g.state.attackers.push(Attacker {
            card: id,
            defender: Some(CombatDefender::Player(p)),
            blocked: false,
        });
    }
    if i == 2 {
        g.state.blocking.push(id);
    }
    if i == 15 {
        named(g, p.other(), "Bear Cub", Zone::Battlefield);
    }
    id
}
#[test]
fn all_nineteen_discounted_only_announcements_lock_real_colored_and_generic_payments_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let target = qualifier(&mut g, p, i);
            let spell = named(&mut g, p, name, Zone::Hand);
            let c = g.state.objects[spell as usize].card;
            let (mut colored, base) = c.cost();
            let discount = c.target_cost_reduction().unwrap().discount;
            for (n, d) in colored.iter_mut().zip(discount.colored) {
                *n = n.saturating_sub(d);
            }
            let generic = base.saturating_sub(discount.generic);
            colored[5] += generic;
            g.state.players[p.index()].mana = colored;
            let b = g.snapshot();
            assert!(
                g.legal_actions()
                    .contains(&Action::AnnounceCast { card: spell }),
                "{name}"
            );
            assert_eq!(g.snapshot(), b);
            replay(&mut g, Action::AnnounceCast { card: spell });
            finish(&mut g);
            let stack = g.state.stack.iter().find(|s| s.card == spell).unwrap();
            assert_eq!(stack.cast.unwrap().method, CastMethod::Normal);
            assert_eq!(stack.cast.unwrap().origin, Zone::Hand);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            resolve(&mut g);
            assert_eq!(
                g.state.objects[spell as usize].zone,
                if i == 17 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                },
                "{name}"
            );
            if matches!(i, 0 | 1 | 5 | 6 | 8 | 10 | 16) {
                assert_eq!(
                    g.state.objects[target as usize].zone,
                    Zone::Graveyard,
                    "{name}"
                );
            }
            if matches!(i, 4 | 14 | 18) {
                assert_eq!(g.state.objects[target as usize].zone, Zone::Library);
            }
            if matches!(i, 11 | 12) {
                assert_eq!(g.state.objects[target as usize].zone, Zone::Exile);
            }
            if i == 13 {
                assert_eq!(g.state.objects[target as usize].zone, Zone::Hand);
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            }
            if i == 3 {
                assert_eq!(g.stats(target).power, -2);
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            }
            if i == 2 {
                assert_eq!(g.stats(target).power, 4);
                assert!(g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
            }
            if i == 7 {
                assert_eq!(
                    g.stats(target),
                    Stats {
                        power: 6,
                        toughness: 5
                    }
                );
            }
            if i == 17 {
                assert_eq!(
                    g.stats(target),
                    Stats {
                        power: -1,
                        toughness: 1
                    }
                );
                assert_eq!(
                    g.state.objects[spell as usize].attached_to,
                    Some(g.object_ref(target))
                );
            }
        }
    }
}
#[test]
fn target_options_cannot_strand_discounted_only_cast_and_direct_payment_is_exact() {
    let mut g = setup(Player::P0);
    let tapped = qualifier(&mut g, Player::P0, 0);
    let plain = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Ajani's Response", Zone::Hand);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 1];
    assert!(g.legal_actions().contains(&Action::Cast {
        card: spell,
        target: Some(Target::Card(tapped)),
        payment: [1, 0, 0, 0, 0, 1]
    }));
    reject(
        &mut g,
        Action::Cast {
            card: spell,
            target: Some(Target::Card(plain)),
            payment: [1, 0, 0, 0, 0, 1],
        },
    );
    replay(&mut g, Action::AnnounceCast { card: spell });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(plain),
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(tapped),
        },
    );
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .unwrap()
            .generic,
        1
    );
    finish(&mut g);
    resolve(&mut g);
}
#[test]
fn hypothetical_mana_tap_cannot_create_a_discount_and_locked_total_survives_later_tap() {
    let mut g = setup(Player::P0);
    let elf = named(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Ajani's Response", Zone::Hand);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 0];
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell })
    );
    g.state.objects[elf as usize].tapped = true;
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell })
    );
    g.state.players[0].mana[5] = 1;
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(elf),
        },
    );
    let total = g.state.announcing.as_ref().unwrap().total_cost;
    g.state.objects[elf as usize].tapped = false;
    assert_eq!(
        g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
        1
    );
    assert_eq!(g.state.announcing.as_ref().unwrap().total_cost, total);
    finish(&mut g);
    resolve(&mut g);
}
#[test]
fn real_mana_tap_cannot_reprice_a_total_locked_on_an_untapped_target() {
    let mut g = setup(Player::P0);
    let elf = named(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Ajani's Response", Zone::Hand);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 3];
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(elf),
        },
    );
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .unwrap()
            .generic,
        4
    );
    replay(&mut g, Action::TapMana { card: elf });
    assert_eq!(
        g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
        4
    );
    reject(
        &mut g,
        Action::PayCosts {
            payment: [1, 0, 0, 0, 1, 0],
        },
    );
    replay(
        &mut g,
        Action::PayCosts {
            payment: [1, 0, 0, 0, 1, 3],
        },
    );
    resolve(&mut g);
    assert_eq!(g.state.objects[elf as usize].zone, Zone::Graveyard);
}
#[test]
fn partial_multi_target_completion_requires_one_qualifier_before_finish_and_no_more() {
    let mut g = setup(Player::P0);
    let tapped = qualifier(&mut g, Player::P0, 0);
    let first = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let second = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Ajani's Response", Zone::Hand);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 1];
    replay(&mut g, Action::AnnounceCast { card: spell });
    // A synthetic typed cardinality isolates the reusable completion solver; no
    // extra Oracle body or dynamic definition enters the admitted catalogue.
    let spec = TargetSpec {
        rule: TargetRule::Creature,
        min: 1,
        max: 2,
    };
    let a = g.state.announcing.as_mut().unwrap();
    a.plan = TargetPlan::Single(spec);
    a.spec = spec;
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    reject(&mut g, Action::FinishTargets);
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(second),
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(tapped),
        },
    );
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .unwrap()
            .generic,
        1
    );
    let locked = g.state.announcing.as_ref().unwrap().total_cost;
    g.sacrifice_batch(&[(Player::P1, tapped)]);
    assert_eq!(g.state.announcing.as_ref().unwrap().total_cost, locked);
    assert_eq!(
        g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
        1
    );
}
#[test]
fn brush_off_colored_discount_and_stack_kinds_are_not_generic_credit() {
    let mut g = setup(Player::P0);
    let creature = qualifier(&mut g, Player::P0, 10);
    let instant = qualifier(&mut g, Player::P0, 1);
    let spell = named(&mut g, Player::P0, "Brush Off", Zone::Hand);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 1];
    replay(&mut g, Action::AnnounceCast { card: spell });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(creature),
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(instant),
        },
    );
    let total = g.state.announcing.as_ref().unwrap().total_cost.unwrap();
    assert_eq!(
        total,
        ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1
        }
    );
    finish(&mut g);
    let bound = g.bind_target(Target::Spell(instant));
    assert_eq!(
        g.target_spell_cost(
            Player::P0,
            spell,
            ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 0
            },
            &[bound]
        ),
        ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 0
        }
    );
}
#[test]
fn static_and_target_reductions_stack_at_zero_without_erasing_printed_values() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, "Stormcatch Mentor", Zone::Battlefield);
    named(&mut g, Player::P0, "Stormcatch Mentor", Zone::Battlefield);
    let target = qualifier(&mut g, Player::P0, 0);
    let spell = named(&mut g, Player::P0, "Ajani's Response", Zone::Hand);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 0];
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    assert_eq!(
        g.state.announcing.as_ref().unwrap().total_cost.unwrap(),
        ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 0
        }
    );
    finish(&mut g);
    let s = g.state.stack.last().unwrap();
    assert_eq!(s.source_card.fixed_mana_value(), 5);
    assert_eq!(s.source_card.colors(), 1);
    resolve(&mut g);
}
