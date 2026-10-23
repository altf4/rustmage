//! Ninja Teen's grant and composition of Sneak with printed costs.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;
fn ninja(g: &mut Game, p: Player) -> CardId {
    let source = named(g, p, "Ninja Teen", Zone::Hand);
    super::ability_timeline_tests::cast(g, source, None, &[]);
    settle(g);
    super::tmt_class_tests::level(g, source, 0);
    super::tmt_class_tests::level(g, source, 1);
    source
}
fn attack(g: &mut Game, p: Player, name: &str) -> CardId {
    let a = named(g, p, name, Zone::Battlefield);
    super::tmt_sneak_tests::unblocked(g, a);
    a
}
fn grave_sneak(g: &mut Game, card: CardId, permission: CardId, attacker: CardId) {
    let permission = g.object_ref(permission);
    let attacker = g.object_ref(attacker);
    checked(
        g,
        Action::AnnounceGraveyardSpell {
            card,
            permission,
            permanent_type: None,
            method: CastMethod::Sneak,
            attacker: Some(attacker),
            flash_surcharge: false,
        },
    );
}
fn pay(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap_or_else(|| panic!("payment at {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("payment did not finish");
}
fn illegal(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(before, g.snapshot());
    assert_eq!(revision, g.revision());
}
#[test]
fn tmt_ninja_graveyard_grant_requires_level_three_and_uses_exact_origin_and_no_finality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let lion = named(&mut g, p, "Savannah Lions", Zone::Graveyard);
        let hand = named(&mut g, p, "Savannah Lions", Zone::Hand);
        let foreign = named(&mut g, p.other(), "Savannah Lions", Zone::Graveyard);
        let a = attack(&mut g, p, "Grizzly Bears");
        for card in [hand, foreign] {
            let permission = g.object_ref(n);
            let attacker = g.object_ref(a);
            illegal(
                &mut g,
                Action::AnnounceGraveyardSpell {
                    card,
                    permission,
                    permanent_type: None,
                    method: CastMethod::Sneak,
                    attacker: Some(attacker),
                    flash_surcharge: false,
                },
            );
        }
        let before = g.state.players[p.other().index()].life;
        grave_sneak(&mut g, lion, n, a);
        assert_eq!(g.announcement_payment_request(p).unwrap().generic, 3);
        assert_eq!(
            g.announcement_payment_request(p).unwrap().required,
            [0, 0, 1, 0, 0, 0]
        );
        pay(&mut g);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before - 1);
        assert_eq!(g.counter_count(lion, CounterKind::Finality), 0);
        assert_eq!(g.stats(lion).power, 3);
        assert!(g.keywords(lion).contains(Keywords::MENACE));
        assert!(g.state.attackers.iter().any(|a| a.card == lion));
    }
}
#[test]
fn tmt_ninja_native_and_granted_sneak_choices_share_permission_without_finality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let card = named(&mut g, p, "Oroku Saki, Shredder Rising", Zone::Graveyard);
        let a = attack(&mut g, p, "Grizzly Bears");
        grave_sneak(&mut g, card, n, a);
        assert!(matches!(g.state.pending, Pending::SpellCost { .. }));
        let costs = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cost_options()
            .collect::<Vec<_>>();
        assert_eq!(costs.len(), 2);
        assert_eq!(costs[0].generic, 1);
        assert_eq!(costs[1].generic, 3);
        checked(&mut g, Action::ChooseSpellCost { cost: 1 });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(card, CounterKind::Finality), 0);
    }
}
#[test]
fn tmt_ninja_sneak_return_and_sacrifice_are_both_mandatory_in_either_order() {
    for p in [Player::P0, Player::P1] {
        for after_return in [false, true] {
            let mut g = setup(p);
            let n = ninja(&mut g, p);
            let card = named(&mut g, p, "Arbiter of Woe", Zone::Graveyard);
            let spare = named(&mut g, p, "Savannah Lions", Zone::Battlefield);
            let a = attack(&mut g, p, "Grizzly Bears");
            grave_sneak(&mut g, card, n, a);
            assert!(matches!(g.state.pending, Pending::SneakCostOrder { .. }));
            assert!(g.observe(p.other()).sneak_additional_cost.is_none());
            checked(&mut g, Action::ChooseSneakCostOrder { after_return });
            if after_return {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, action);
                assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
                assert!(matches!(
                    g.state.pending,
                    Pending::SneakAdditionalCost { .. }
                ));
                assert!(!g.legal_actions().iter().any(|a| matches!(
                    a,
                    Action::TapMana { .. } | Action::AnnounceActivate { .. }
                )));
            }
            let witness = g.object_ref(spare);
            checked(
                &mut g,
                Action::ChooseSneakAdditionalCost { object: witness },
            );
            pay(&mut g);
            assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
            assert_eq!(g.state.objects[spare as usize].zone, Zone::Graveyard);
            assert!(g.state.stack.iter().any(|i| i.card == card));
        }
    }
}
#[test]
fn tmt_ninja_sacrifice_cannot_consume_the_only_return_attacker() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let card = named(&mut g, p, "Arbiter of Woe", Zone::Graveyard);
        let a = attack(&mut g, p, "Grizzly Bears");
        let permission = g.object_ref(n);
        let attacker = g.object_ref(a);
        illegal(
            &mut g,
            Action::AnnounceGraveyardSpell {
                card,
                permission,
                permanent_type: None,
                method: CastMethod::Sneak,
                attacker: Some(attacker),
                flash_surcharge: false,
            },
        );
    }
}
#[test]
fn tmt_ninja_champion_return_first_exiles_the_fresh_hand_incarnation_and_links_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let card = named(&mut g, p, "Champions of the Perfect", Zone::Graveyard);
        let a = attack(&mut g, p, "Llanowar Elves");
        let old = g.object_ref(a);
        grave_sneak(&mut g, card, n, a);
        assert_eq!(
            g.observe(p).sneak_additional_cost.unwrap().order_options,
            vec![true]
        );
        checked(&mut g, Action::ChooseSneakCostOrder { after_return: true });
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, action);
        let fresh = g.object_ref(a);
        assert_ne!(fresh, old);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
        illegal(&mut g, Action::ChooseSneakAdditionalCost { object: old });
        checked(&mut g, Action::ChooseSneakAdditionalCost { object: fresh });
        assert_eq!(g.state.objects[a as usize].zone, Zone::Exile);
        settle(&mut g);
        g.move_object(card, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
    }
}
#[test]
fn tmt_ninja_behold_can_reveal_the_attacker_before_return_or_pay_extra_mana() {
    for p in [Player::P0, Player::P1] {
        for mana_instead in [false, true] {
            let mut g = setup(p);
            let n = ninja(&mut g, p);
            let card = named(&mut g, p, "Kinsbaile Aspirant", Zone::Graveyard);
            let a = attack(
                &mut g,
                p,
                if mana_instead {
                    "Grizzly Bears"
                } else {
                    "Eclipsed Kithkin"
                },
            );
            grave_sneak(&mut g, card, n, a);
            let options = g
                .state
                .announcing
                .as_ref()
                .unwrap()
                .cost_options()
                .collect::<Vec<_>>();
            assert_eq!(options.len(), 2);
            assert_eq!(options[1].generic, 5);
            assert_eq!(options[1].colored, [0, 0, 1, 0, 0, 0]);
            checked(
                &mut g,
                Action::ChooseSpellCost {
                    cost: u16::from(mana_instead),
                },
            );
            if !mana_instead {
                checked(
                    &mut g,
                    Action::ChooseSneakCostOrder {
                        after_return: false,
                    },
                );
                let witness = g.object_ref(a);
                checked(
                    &mut g,
                    Action::ChooseSneakAdditionalCost { object: witness },
                );
            }
            pay(&mut g);
            assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
            settle(&mut g);
        }
    }
}
#[test]
fn tmt_ninja_kicker_and_offspring_are_added_to_the_alternative_cost() {
    for p in [Player::P0, Player::P1] {
        for offspring in [false, true] {
            let mut g = setup(p);
            let n = ninja(&mut g, p);
            let card = named(
                &mut g,
                p,
                if offspring {
                    "Flowerfoot Swordmaster"
                } else {
                    "Gnarlid Colony"
                },
                Zone::Graveyard,
            );
            let a = attack(&mut g, p, "Grizzly Bears");
            grave_sneak(&mut g, card, n, a);
            let choice = if offspring {
                Action::ChooseOffspring { pay: true }
            } else {
                Action::ChooseKicker { pay: true }
            };
            checked(&mut g, choice);
            let request = g.announcement_payment_request(p).unwrap();
            assert_eq!(request.generic, 5);
            assert_eq!(request.required[2], 1);
            assert_eq!(request.required[4], u32::from(!offspring));
            pay(&mut g);
            settle(&mut g);
            if offspring {
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.zone == Zone::Battlefield
                            && o.controller == p
                            && o.is_token()
                            && o.card.name().contains("Flowerfoot"))
                        .count(),
                    1
                );
            } else {
                assert_eq!(g.counter_count(card, CounterKind::PlusOne), 2);
            }
        }
    }
}
#[test]
fn tmt_ninja_fixed_alternative_sets_x_to_zero_and_never_offers_another_alternative_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let card = named(&mut g, p, "Wildwood Scourge", Zone::Graveyard);
        let a = attack(&mut g, p, "Grizzly Bears");
        grave_sneak(&mut g, card, n, a);
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(0));
        illegal(&mut g, Action::ChooseX { value: 1 });
        pay(&mut g);
        settle(&mut g);
        // The level-two boost is +1/+0; it cannot save a 0-toughness Scourge.
        assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn tmt_ninja_level_two_grants_live_menace_and_boost_only_while_abilities_exist() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = super::tests::spawn(
            &mut g,
            p,
            Card::from_name("Ninja Teen").unwrap(),
            Zone::Battlefield,
        );
        let a = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        assert_eq!(g.stats(a).power, 2);
        assert!(!g.keywords(a).contains(Keywords::MENACE));
        super::tmt_class_tests::level(&mut g, source, 0);
        assert_eq!(g.stats(a).power, 3);
        assert_eq!(g.stats(b).power, 3);
        assert!(!g.keywords(b).contains(Keywords::MENACE));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: a });
        checked(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        illegal(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: a,
            },
        );
        let c = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        checked(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: a,
            },
        );
        illegal(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                blocker: c,
                attacker: a,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        g.move_object(source, Zone::Hand);
        assert_eq!(g.stats(a).power, 2);
        assert!(!g.keywords(a).contains(Keywords::MENACE));
        assert_eq!(g.class_level(g.object_ref(source)), 1);
    }
}
#[test]
fn tmt_ninja_returning_its_animated_source_preserves_the_captured_grant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let card = named(&mut g, p, "Savannah Lions", Zone::Graveyard);
        let item = g.activation_item(p, n, 0);
        g.animate_source(
            &item,
            crate::cards::AnimationSpec {
                artifact: false,
                creature_types: crate::cards::AnimationCreatureTypes::AddNamed(&["Ninja"]),
                base: Some((3, 3)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
        );
        super::tmt_sneak_tests::unblocked(&mut g, n);
        grave_sneak(&mut g, card, n, n);
        pay(&mut g);
        assert_eq!(g.state.objects[n as usize].zone, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.objects[card as usize].zone, Zone::Battlefield);
        assert_eq!(g.stats(card).power, 2);
        assert!(!g.keywords(card).contains(Keywords::MENACE));
        assert_eq!(g.counter_count(card, CounterKind::Finality), 0);
    }
}
#[test]
fn tmt_ninja_independent_graveyard_permissions_preserve_finality_and_muldrotha_quota() {
    for p in [Player::P0, Player::P1] {
        for route in 0..3 {
            let mut g = setup(p);
            let n = ninja(&mut g, p);
            let samurai = named(&mut g, p, "Leonardo, Sewer Samurai", Zone::Battlefield);
            let m = named(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
            let card = named(&mut g, p, "Savannah Lions", Zone::Graveyard);
            let a = attack(&mut g, p, "Grizzly Bears");
            let permission = g.object_ref([n, samurai, m][route]);
            let attacker = g.object_ref(a);
            checked(
                &mut g,
                Action::AnnounceGraveyardSpell {
                    card,
                    permission,
                    permanent_type: (route == 2).then_some(PermanentType::Creature),
                    method: CastMethod::Sneak,
                    attacker: Some(attacker),
                    flash_surcharge: false,
                },
            );
            pay(&mut g);
            settle(&mut g);
            assert_eq!(
                g.counter_count(card, CounterKind::Finality),
                u64::from(route == 1)
            );
            assert_eq!(
                g.state.graveyard_type_uses.as_ref().map_or(0, |v| v.len()),
                usize::from(route == 2)
            );
        }
    }
}
#[test]
fn tmt_ninja_losing_and_regaining_printed_abilities_keeps_levels_but_gates_all_effects() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let card = named(&mut g, p, "Savannah Lions", Zone::Graveyard);
        let friend = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let item = g.activation_item(p, n, 0);
        g.animate_source(
            &item,
            crate::cards::AnimationSpec {
                artifact: false,
                creature_types: crate::cards::AnimationCreatureTypes::AddNamed(&["Ninja"]),
                base: Some((3, 3)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
        );
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        super::ability_timeline_tests::cast(&mut g, aura, None, &[Target::Card(n)]);
        settle(&mut g);
        assert_eq!(g.class_level(g.object_ref(n)), 3);
        assert_eq!(g.stats(friend).power, 2);
        assert!(!g.keywords(friend).contains(Keywords::MENACE));
        assert!(g.granted_sneak_cost(p, card).is_none());
        let bounce = named(&mut g, p, "Into the Roil", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: bounce });
        checked(&mut g, Action::ChooseKicker { pay: false });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(aura),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.class_level(g.object_ref(n)), 3);
        assert_eq!(g.stats(friend).power, 3);
        assert!(g.keywords(friend).contains(Keywords::MENACE));
        assert!(g.granted_sneak_cost(p, card).is_some());
    }
}
#[test]
fn tmt_ninja_convoke_can_tap_the_reserved_vigilant_attacker_before_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let spell = named(&mut g, p, "Merrow Skyswimmer", Zone::Graveyard);
        let black = named(&mut g, p, "Vampire Nighthawk", Zone::Battlefield);
        let b = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let c = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let a = attack(&mut g, p, "Serra Angel");
        assert!(!g.state.objects[a as usize].tapped);
        g.state.players[p.index()].mana = [0; 6];
        grave_sneak(&mut g, spell, n, a);
        assert!(matches!(g.state.pending, Pending::PayCosts { .. }));
        checked(&mut g, Action::BeginConvoke);
        for (id, contribution) in [
            (black, ConvokeContribution::Black),
            (a, ConvokeContribution::Generic),
            (b, ConvokeContribution::Generic),
            (c, ConvokeContribution::Generic),
        ] {
            let object = g.object_ref(id);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
        }
        pay(&mut g);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
        assert!(g.state.objects[black as usize].tapped);
        assert!(g.state.objects[b as usize].tapped);
        assert!(g.state.objects[c as usize].tapped);
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn tmt_ninja_before_return_witness_can_appear_during_a_mana_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let treasure = super::tests::spawn(
            &mut g,
            p,
            Card::catalogue()
                .find(|c| c.token_info().is_some_and(|i| i.name == "Treasure Token"))
                .unwrap(),
            Zone::Battlefield,
        );
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Exile);
        g.state.exile_links.push(ExileLink {
            source: g.object_ref(treasure),
            exiled: g.object_ref(elf),
            return_zone: Zone::Battlefield,
        });
        let card = named(&mut g, p, "Champions of the Perfect", Zone::Graveyard);
        let a = attack(&mut g, p, "Grizzly Bears");
        g.state.players[p.index()].mana = [0, 0, 0, 0, 3, 0];
        grave_sneak(&mut g, card, n, a);
        checked(
            &mut g,
            Action::ChooseSneakCostOrder {
                after_return: false,
            },
        );
        assert!(matches!(
            g.state.pending,
            Pending::SneakAdditionalCost { .. }
        ));
        assert!(
            g.observe(p)
                .sneak_additional_cost
                .unwrap()
                .candidates
                .is_empty()
        );
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana {card,production,..} if *card==treasure && *production==[0,0,1,0,0,0])).expect("pre-cost mana ability returning witness");
        checked(&mut g, action);
        assert_eq!(g.state.objects[elf as usize].zone, Zone::Battlefield);
        let object = g.object_ref(elf);
        checked(&mut g, Action::ChooseSneakAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.state.objects[elf as usize].zone, Zone::Exile);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
        assert!(g.state.stack.iter().any(|item| item.card == card));
    }
}
#[test]
fn tmt_ninja_composed_payment_rolls_back_mana_sacrifice_and_queued_triggers_if_return_fails() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let card = named(&mut g, p, "Arbiter of Woe", Zone::Graveyard);
        let spare = named(&mut g, p, "Savannah Lions", Zone::Battlefield);
        let a = attack(&mut g, p, "Grizzly Bears");
        g.state.objects[a as usize].generation = u64::MAX;
        grave_sneak(&mut g, card, n, a);
        checked(
            &mut g,
            Action::ChooseSneakCostOrder {
                after_return: false,
            },
        );
        let object = g.object_ref(spare);
        checked(&mut g, Action::ChooseSneakAdditionalCost { object });
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .expect("first cost has a legal return witness");
        let before = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(p, action),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(before, g.snapshot());
        assert_eq!(revision, g.revision());
        assert_eq!(g.state.objects[spare as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn tmt_ninja_champion_cannot_exile_returned_token_or_foreign_owned_elf() {
    for p in [Player::P0, Player::P1] {
        for token in [false, true] {
            let mut g = setup(p);
            let n = ninja(&mut g, p);
            let card = named(&mut g, p, "Champions of the Perfect", Zone::Graveyard);
            let a = if token {
                named(
                    &mut g,
                    p,
                    "@token/ca8c0a7df61e962ddae7596f",
                    Zone::Battlefield,
                )
            } else {
                named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield)
            };
            if !token {
                g.state.objects[a as usize].controller = p;
            }
            super::tmt_sneak_tests::unblocked(&mut g, a);
            let permission = g.object_ref(n);
            let attacker = g.object_ref(a);
            illegal(
                &mut g,
                Action::AnnounceGraveyardSpell {
                    card,
                    permission,
                    permanent_type: None,
                    method: CastMethod::Sneak,
                    attacker: Some(attacker),
                    flash_surcharge: false,
                },
            );
        }
    }
}
#[test]
fn tmt_ninja_level_three_is_required_and_opponents_turn_never_opens_sneak() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = super::tests::spawn(
            &mut g,
            p,
            Card::from_name("Ninja Teen").unwrap(),
            Zone::Battlefield,
        );
        super::tmt_class_tests::level(&mut g, n, 0);
        let card = named(&mut g, p, "Savannah Lions", Zone::Graveyard);
        let a = attack(&mut g, p, "Grizzly Bears");
        let permission = g.object_ref(n);
        let attacker = g.object_ref(a);
        illegal(
            &mut g,
            Action::AnnounceGraveyardSpell {
                card,
                permission,
                permanent_type: None,
                method: CastMethod::Sneak,
                attacker: Some(attacker),
                flash_surcharge: false,
            },
        );
        assert!(g.granted_sneak_cost(p, card).is_none());
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let card = named(&mut g, p, "Savannah Lions", Zone::Graveyard);
        let a = attack(&mut g, p, "Grizzly Bears");
        g.state.active = p.other();
        let permission = g.object_ref(n);
        let attacker = g.object_ref(a);
        illegal(
            &mut g,
            Action::AnnounceGraveyardSpell {
                card,
                permission,
                permanent_type: None,
                method: CastMethod::Sneak,
                attacker: Some(attacker),
                flash_surcharge: false,
            },
        );
    }
}
#[test]
fn tmt_ninja_another_alternative_cost_cannot_be_combined_with_sneak() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let card = named(&mut g, p, "Wistfulness", Zone::Graveyard);
        let a = attack(&mut g, p, "Grizzly Bears");
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
        grave_sneak(&mut g, card, n, a);
        assert_eq!(
            g.state.announcing.as_ref().unwrap().cost_options().count(),
            1
        );
        assert!(
            !g.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .cast
                .unwrap()
                .evoked
        );
        illegal(&mut g, Action::ChooseSpellCost { cost: 1 });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.objects[card as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn tmt_ninja_cost_order_changes_which_creature_departures_its_source_observes() {
    for p in [Player::P0, Player::P1] {
        for after_return in [false, true] {
            let mut g = setup(p);
            let n = ninja(&mut g, p);
            let card = named(&mut g, p, "Arbiter of Woe", Zone::Graveyard);
            let spare = named(&mut g, p, "Savannah Lions", Zone::Battlefield);
            let item = g.activation_item(p, n, 0);
            g.animate_source(
                &item,
                crate::cards::AnimationSpec {
                    artifact: false,
                    creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                    base: Some((3, 3)),
                    keywords: Keywords::NONE,
                    duration: crate::cards::AnimationDuration::Permanent,
                },
            );
            super::tmt_sneak_tests::unblocked(&mut g, n);
            grave_sneak(&mut g, card, n, n);
            checked(&mut g, Action::ChooseSneakCostOrder { after_return });
            if after_return {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, action);
            }
            let object = g.object_ref(spare);
            checked(&mut g, Action::ChooseSneakAdditionalCost { object });
            pay(&mut g);
            let triggers = g
                .state
                .stack
                .iter()
                .chain(&g.state.pending_triggers)
                .chain(&g.state.ordering_triggers)
                .chain(g.state.announcing.iter().map(|a| &a.item))
                .filter(|i| {
                    i.source_card.name() == "Ninja Teen"
                        && matches!(i.kind, StackKind::TriggeredAbility { .. })
                })
                .count();
            assert_eq!(triggers, if after_return { 1 } else { 2 });
        }
    }
}
#[test]
fn tmt_ninja_base_counts_each_owned_creature_in_a_simultaneous_destruction_including_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        super::tests::spawn(
            &mut g,
            p,
            Card::from_name("Ninja Teen").unwrap(),
            Zone::Battlefield,
        );
        named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        named(&mut g, p, "@token/ninja_1_1_black", Zone::Battlefield);
        named(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
        let spell = named(&mut g, p, "Day of Judgment", Zone::Hand);
        let before = g.state.players[p.other().index()].life;
        let own = g.state.players[p.index()].life;
        super::ability_timeline_tests::cast(&mut g, spell, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before - 2);
        assert_eq!(g.state.players[p.index()].life, own);
    }
}
#[test]
fn tmt_ninja_base_triggers_for_a_bounced_token_but_not_a_bounced_noncreature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        super::tests::spawn(
            &mut g,
            p,
            Card::from_name("Ninja Teen").unwrap(),
            Zone::Battlefield,
        );
        let token = named(&mut g, p, "@token/ninja_1_1_black", Zone::Battlefield);
        let artifact = named(&mut g, p, "Sewer-veillance Cam", Zone::Battlefield);
        let before = g.state.players[p.other().index()].life;
        for (target, loss) in [(artifact, 0), (token, 1)] {
            let spell = named(&mut g, p, "Into the Roil", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseKicker { pay: false });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, before - loss);
        }
        assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
    }
}
#[test]
fn tmt_ninja_missing_witness_is_not_created_by_irrelevant_tap_mana_sources() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        for _ in 0..4 {
            named(&mut g, p, "Hedron Archive", Zone::Battlefield);
        }
        let card = named(&mut g, p, "Champions of the Perfect", Zone::Graveyard);
        let a = attack(&mut g, p, "Grizzly Bears");
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        let permission = g.object_ref(n);
        let attacker = g.object_ref(a);
        illegal(
            &mut g,
            Action::AnnounceGraveyardSpell {
                card,
                permission,
                permanent_type: None,
                method: CastMethod::Sneak,
                attacker: Some(attacker),
                flash_surcharge: false,
            },
        );
    }
}
#[test]
fn tmt_ninja_additional_witness_checks_keep_the_locked_cost_when_a_reducer_enters_during_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = ninja(&mut g, p);
        let treasure = super::tests::spawn(
            &mut g,
            p,
            Card::catalogue()
                .find(|c| c.token_info().is_some_and(|i| i.name == "Treasure Token"))
                .unwrap(),
            Zone::Battlefield,
        );
        let reducer = named(&mut g, p, "Honest Rutstein", Zone::Exile);
        g.state.exile_links.push(ExileLink {
            source: g.object_ref(treasure),
            exiled: g.object_ref(reducer),
            return_zone: Zone::Battlefield,
        });
        let card = named(&mut g, p, "Arbiter of Woe", Zone::Graveyard);
        let a = attack(&mut g, p, "Grizzly Bears");
        g.state.players[p.index()].mana = [0, 0, 0, 0, 3, 0];
        grave_sneak(&mut g, card, n, a);
        checked(
            &mut g,
            Action::ChooseSneakCostOrder {
                after_return: false,
            },
        );
        let locked = g.state.announcing.as_ref().unwrap().total_cost.unwrap();
        assert_eq!(locked.generic, 3);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana {card,production,..} if *card==treasure && *production==[0,0,1,0,0,0])).unwrap();
        checked(&mut g, action);
        assert_eq!(g.state.objects[reducer as usize].zone, Zone::Battlefield);
        assert_eq!(
            g.state.announcing.as_ref().unwrap().total_cost,
            Some(locked)
        );
        // Compare the funding predicate just below the fixed cost. The new
        // reducer must not make this insufficient pool fund the old proposal.
        let pool = g.state.players[p.index()].mana;
        g.state.players[p.index()].mana = [0, 0, 1, 0, 2, 0];
        let announcement = g.state.announcing.as_ref().unwrap();
        let mana = announcement.cost_options().next().unwrap();
        let witness = g.object_ref(reducer);
        assert!(!g.sneak_extra_fundable(announcement, mana, Some(false), Some(witness)));
        g.state.players[p.index()].mana = pool;
        checked(
            &mut g,
            Action::ChooseSneakAdditionalCost { object: witness },
        );
        pay(&mut g);
        assert_eq!(g.state.objects[reducer as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
