//! Real optional conspire costs, trigger packets and exact target-copy choices.
use super::ability_timeline_tests::{checked, named, settle, setup};
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
fn schemes(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Raiding Schemes", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 1, 3];
    checked(g, Action::AnnounceCast { card: id });
    while let Some(c) = g.conspire_view() {
        let Some(i) = c.instances.iter().find(|i| i.pay.is_none()) else {
            break;
        };
        let source = i.source;
        checked(g, Action::ChooseConspire { source, pay: false });
    }
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
    settle(g);
    id
}
fn intent(g: &mut Game, source: CardId, pay: bool) {
    let source = g.object_ref(source);
    checked(g, Action::ChooseConspire { source, pay });
}
fn pay_pair(g: &mut Game, source: CardId, first: CardId, second: CardId) {
    let (source, first, second) = (
        g.object_ref(source),
        g.object_ref(first),
        g.object_ref(second),
    );
    let (first, second) = if first.card < second.card {
        (first, second)
    } else {
        (second, first)
    };
    checked(
        g,
        Action::PayConspire {
            source,
            first,
            second,
        },
    );
}
fn printed(g: &mut Game) {
    if g.legal_actions().contains(&Action::PayConspirePrintedCosts) {
        checked(g, Action::PayConspirePrintedCosts);
    }
}
fn finish_copies(g: &mut Game) {
    for _ in 0..100 {
        if matches!(g.state.pending, Pending::Priority)
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishStackTargets | Action::Pass | Action::PutTrigger { .. }
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("conspire did not settle {:?}", g.state.pending);
}
#[test]
fn ecl_raiding_paid_body_and_decline_without_two_matching_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = schemes(&mut g, p);
        named(&mut g, p, "Enraged Flamecaster", Zone::Battlefield);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        let object = g.object_ref(source);
        reject(
            &mut g,
            Action::ChooseConspire {
                source: object,
                pay: true,
            },
        );
        intent(&mut g, source, false);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn ecl_raiding_two_optional_instances_pay_four_fresh_creatures_and_copy_actual_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = schemes(&mut g, p);
        let other = schemes(&mut g, p);
        let donors = (0..4)
            .map(|_| named(&mut g, p, "Enraged Flamecaster", Zone::Battlefield))
            .collect::<Vec<_>>();
        for &id in &donors {
            g.state.objects[id as usize].summoning_sick = true;
        }
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        intent(&mut g, source, true);
        intent(&mut g, other, true);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        assert!(matches!(g.state.pending, Pending::ConspirePayment { .. }));
        assert!(g.state.stack.is_empty());
        pay_pair(&mut g, source, donors[0], donors[1]);
        let r = g.object_ref(donors[0]);
        let s = g.object_ref(source);
        let fresh = g.object_ref(donors[2]);
        reject(
            &mut g,
            Action::PayConspire {
                source: s,
                first: r,
                second: fresh,
            },
        );
        pay_pair(&mut g, other, donors[2], donors[3]);
        if matches!(g.state.pending, Pending::ConspirePayment { .. }) {
            printed(&mut g);
        }
        assert!(
            donors
                .iter()
                .all(|id| g.object_view(*id).tapped && g.object_view(*id).summoning_sick)
        );
        finish_copies(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 14);
        assert!(g.conspire_view().is_none());
    }
}
#[test]
fn ecl_raiding_single_instance_uses_matching_colors_and_rejects_duplicates_and_foreign_pairs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = schemes(&mut g, p);
        let a = named(&mut g, p, "Eclipsed Boggart", Zone::Battlefield);
        let b = named(&mut g, p, "Eclipsed Flamekin", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Enraged Flamecaster", Zone::Battlefield);
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        intent(&mut g, source, true);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        let (s, r, f) = (g.object_ref(source), g.object_ref(a), g.object_ref(enemy));
        reject(
            &mut g,
            Action::PayConspire {
                source: s,
                first: r,
                second: r,
            },
        );
        reject(
            &mut g,
            Action::PayConspire {
                source: s,
                first: r,
                second: f,
            },
        );
        pay_pair(&mut g, source, a, b);
        printed(&mut g);
        finish_copies(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
#[test]
fn ecl_raiding_taps_may_precede_printed_sacrifice_and_cannot_double_spend_a_sacrificed_donor() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = schemes(&mut g, p);
        let a = named(&mut g, p, "Eclipsed Boggart", Zone::Battlefield);
        let b = named(&mut g, p, "Shadow Urchin", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Eaten Alive", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        intent(&mut g, source, true);
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        let object = g.object_ref(a);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, 0],
            },
        );
        reject(&mut g, Action::PayConspirePrintedCosts);
        pay_pair(&mut g, source, a, b);
        printed(&mut g);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Graveyard);
        assert!(g.state.objects[b as usize].tapped);
        finish_copies(&mut g);
        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Exile);
    }
}
#[test]
fn ecl_raiding_printed_sacrifice_returns_a_fresh_donor_before_the_conspire_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = schemes(&mut g, p);
        let hunter = named(&mut g, p, "Henchbots", Zone::Battlefield);
        let auntie = named(&mut g, p, "Heirloom Auntie", Zone::Exile);
        let hunter_ref = g.object_ref(hunter);
        g.state.exile_links.push(ExileLink {
            source: hunter_ref,
            exiled: g.object_ref(auntie),
            return_zone: Zone::Battlefield,
        });
        let donor = named(&mut g, p, "Eclipsed Boggart", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Eaten Alive", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        intent(&mut g, source, true);
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(&mut g, Action::ChooseAdditionalCost { object: hunter_ref });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, 0],
            },
        );
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayConspire { .. }))
        );
        printed(&mut g);
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::ConspirePayment { .. }) {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(g.object_view(auntie).zone, Zone::Battlefield);
        pay_pair(&mut g, source, donor, auntie);
        assert!(g.state.announcing.is_none());
        finish_copies(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Exile);
    }
}
#[test]
fn ecl_raiding_departed_provider_and_countered_original_leave_the_copy_trigger_and_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = schemes(&mut g, p);
        let a = named(&mut g, p, "Enraged Flamecaster", Zone::Battlefield);
        let b = named(&mut g, p, "Enraged Flamecaster", Zone::Battlefield);
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        intent(&mut g, source, true);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        pay_pair(&mut g, source, a, b);
        printed(&mut g);
        let original = g
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Spell)
            .unwrap()
            .id;
        assert!(g.counter_stack_item(original, false));
        g.move_object(source, Zone::Graveyard);
        finish_copies(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn ecl_raiding_kindred_spell_conspire_is_not_multiplied_by_elemental_permanent_travelers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = schemes(&mut g, p);
        named(&mut g, p, "Twinflame Travelers", Zone::Battlefield);
        let a = named(&mut g, p, "Eclipsed Elf", Zone::Battlefield);
        let b = named(&mut g, p, "Eclipsed Boggart", Zone::Battlefield);
        let victim = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let spell = named(&mut g, p, "Nameless Inversion", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        intent(&mut g, source, true);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, 1],
            },
        );
        pay_pair(&mut g, source, a, b);
        printed(&mut g);
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|s| s.kind == StackKind::Conspire)
                .count(),
            1
        );
        finish_copies(&mut g);
        assert_eq!(
            g.stats(victim),
            Stats {
                power: 16,
                toughness: 4
            }
        );
    }
}
#[test]
fn ecl_raiding_kirol_copies_the_intrinsic_conspire_trigger_without_paying_conspire_again() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = schemes(&mut g, p);
        let a = named(&mut g, p, "Enraged Flamecaster", Zone::Battlefield);
        let b = named(&mut g, p, "Enraged Flamecaster", Zone::Battlefield);
        let kirol = named(&mut g, p, "Kirol, Attentive First-Year", Zone::Battlefield);
        let witness = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        intent(&mut g, source, true);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        pay_pair(&mut g, source, a, b);
        printed(&mut g);
        let id = g
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Conspire)
            .unwrap()
            .id;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: kirol,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(id),
            },
        );
        for card in [kirol, witness] {
            let object = g.object_ref(card);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        finish_copies(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}
