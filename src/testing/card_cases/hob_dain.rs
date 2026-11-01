//! Focused printed outcomes; shared engine invariants live in core tests.
//! Conditional attack costs and exact paid declarations.
use super::*;
fn story(g: &mut Game, p: Player) -> CardId {
    let d = put(g, p, "Dáin, Lord of the Iron Hills", Zone::Battlefield);
    put(g, p, "Goldvein Pick", Zone::Battlefield);
    put(g, p, "Goldvein Pick", Zone::Battlefield);
    g.refresh_enduring_stories();
    assert!(g.state.players[p.index()].enduring_story);
    d
}
fn attackers(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
}
#[test]
fn dain_per_attacker_cost_locks_after_taps_and_is_paid_with_mana_abilities() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let dain = story(&mut g, p.other());
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let x = put(&mut g, p, "Forest", Zone::Battlefield);
        let y = put(&mut g, p, "Forest", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        attackers(&mut g, p);
        act(&mut g, Action::Attack { card: a });
        act(&mut g, Action::Attack { card: b });
        act(&mut g, Action::FinishAttackers);
        assert!(g.object_view(a).tapped && g.object_view(b).tapped);
        assert!(!g.state.attackers.iter().any(|v| v.card == a));
        assert_eq!(g.state.attack_payment.as_ref().unwrap().generic, 2);

        g.move_object(dain, Zone::Graveyard);
        assert_eq!(g.state.attack_payment.as_ref().unwrap().generic, 2);
        act(&mut g, Action::TapMana { card: x });
        act(&mut g, Action::TapMana { card: y });
        reject(
            &mut g,
            Action::PayAttackCost {
                payment: [0, 0, 0, 0, 1, 0],
            },
        );
        act(
            &mut g,
            Action::PayAttackCost {
                payment: [0, 0, 0, 0, 2, 0],
            },
        );
        assert_eq!(g.state.attackers.len(), 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn dain_planeswalker_untaxed_and_cannot_fund_from_nontvigiliant_attacker() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        story(&mut g, p.other());
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        g.state.players[p.index()].mana = [0; 6];
        attackers(&mut g, p);
        act(&mut g, Action::Attack { card: elf });
        assert!(!g.legal_actions().contains(&Action::FinishAttackers));
        act(&mut g, Action::CancelAttackDeclaration);
        let defender = g.object_ref(walker);
        act(
            &mut g,
            Action::AttackPlaneswalker {
                card: elf,
                defender,
            },
        );
        act(&mut g, Action::FinishAttackers);
        assert!(g.object_view(elf).tapped);
        assert!(g.state.attack_payment.is_none());
    }
}

#[test]
fn dain_current_controller_story_and_vigilance() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let d = put(&mut g, p, "Dáin, Lord of the Iron Hills", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        cast(&mut g, d, None);
        settle(&mut g);
        assert_eq!(stats(&g, d), (2, 2));

        assert_eq!(g.attack_controller_tax(p), 0);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        assert!(g.state.players[p.index()].enduring_story);
        assert_eq!(g.attack_controller_tax(p), 1);
        let reference = g.object_ref(d);
        g.record_control(reference, p.other(), None);
        assert_eq!(g.attack_controller_tax(p), 0);
        assert_eq!(g.attack_controller_tax(p.other()), 0);
        g.state.players[p.other().index()].enduring_story = true;
        assert_eq!(g.attack_controller_tax(p.other()), 1);
        g.move_object(d, Zone::Graveyard);
        assert!(g.state.players[p.index()].enduring_story);
        assert_eq!(g.attack_controller_tax(p.other()), 0);
    }
}

#[test]
fn dain_suppression_preserves_story_but_removes_tax() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = story(&mut g, p);
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, aura, Some(Target::Card(source)));
        settle(&mut g);
        assert!(!g.printed_abilities_available(source));
        assert!(g.has_attack_tax_provider());
        assert!(g.state.players[p.index()].enduring_story);
        assert_eq!(g.attack_controller_tax(p), 0);
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(g.attack_controller_tax(p), 1);
    }
}

#[test]
fn dain_entering_attacking_does_not_pay_declaration_tax_again() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        story(&mut g, p.other());
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let elektra = put(&mut g, p, "Elektra, Daughter of the Hand", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        attackers(&mut g, p);
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        act(
            &mut g,
            Action::PayAttackCost {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        for _ in 0..10 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            act(&mut g, Action::Pass);
        }
        act(&mut g, Action::FinishBlockers);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        act(
            &mut g,
            Action::AnnounceSneak {
                card: elektra,
                permission: None,
            },
        );
        let object = g.object_ref(attacker);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        settle(&mut g);
        assert_eq!(g.object_view(attacker).zone, Zone::Hand);
        assert!(
            g.object_view(elektra).tapped && g.state.attackers.iter().any(|a| a.card == elektra)
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.state.attack_payment.is_none());
    }
}

#[test]
fn dain_restricted_mana_cannot_pay_and_mana_ability_costs_compete() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        story(&mut g, p.other());
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let hydro = put(&mut g, p, "Hydro-Channeler", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        act(
            &mut g,
            Action::ActivateMana {
                card: hydro,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        attackers(&mut g, p);
        act(&mut g, Action::Attack { card: creature });
        assert!(!g.legal_actions().contains(&Action::FinishAttackers));
        act(&mut g, Action::CancelAttackDeclaration);

        let mut g = scenario(p);
        story(&mut g, p.other());
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let faun = put(&mut g, p, "Rootrider Faun", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        attackers(&mut g, p);
        act(&mut g, Action::Attack { card: a });
        act(&mut g, Action::Attack { card: b });
        act(&mut g, Action::FinishAttackers);
        act(
            &mut g,
            Action::ActivateMana {
                card: faun,
                ability: 1,
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        act(&mut g, Action::TapMana { card: land });
        act(
            &mut g,
            Action::PayAttackCost {
                payment: [0, 0, 0, 0, 2, 0],
            },
        );
        assert_eq!(g.state.attackers.len(), 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

// A synthetic tax predicate models an ability enabled by the attacker's tapped
// state. The preview must evaluate it after tapping, without mutating the game.

#[test]
fn dain_mana_activation_preserves_unrestricted_mana_for_tax() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        story(&mut g, p.other());
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let faun = put(&mut g, p, "Rootrider Faun", Zone::Battlefield);
        let land = put(&mut g, p, "Secluded Courtyard", Zone::Battlefield);
        // A continuous animation makes the Satyr an Elf for the mana restriction.
        g.change_object_form(
            g.object_ref(faun),
            crate::cards::AnimationSpec {
                base: None,
                artifact: false,
                creature_types: crate::cards::AnimationCreatureTypes::Named(&["Elf"]),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        let land_ref = g.object_ref(land);
        g.set_entry_value(
            land_ref,
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Elf").unwrap()),
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        act(
            &mut g,
            Action::ActivateMana {
                card: land,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        attackers(&mut g, p);
        act(&mut g, Action::Attack { card: a });
        act(&mut g, Action::Attack { card: b });
        act(&mut g, Action::FinishAttackers);
        act(
            &mut g,
            Action::ActivateMana {
                card: faun,
                ability: 1,
                payment: [0, 0, 0, 0, 1, 0],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        assert!(
            g.state.attack_payment.is_some(),
            "allocation must preserve a fundable attack"
        );
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseManaLotSpend { .. }))
        {
            act(&mut g, Action::ChooseManaLotSpend { amount: 1 });
        }
        act(
            &mut g,
            Action::PayAttackCost {
                payment: [0, 0, 0, 0, 2, 0],
            },
        );
        assert_eq!(g.state.attackers.len(), 2);
    }
}

#[test]
fn dain_foreign_entry_during_nested_mana_can_reverse_or_keep_earlier_mana() {
    {
        let p = Player::P0;
        for (keep, irreversible) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut g = scenario(p);
            story(&mut g, p.other());
            let army: Vec<_> = (0..if irreversible { 7 } else { 4 })
                .map(|_| put(&mut g, p, "Bear Cub", Zone::Battlefield))
                .collect();
            let forest = put(&mut g, p, "Forest", Zone::Battlefield);
            let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let colossus = if irreversible {
                let colossus = put(&mut g, p, "Darksteel Colossus", Zone::Battlefield);
                let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
                assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(colossus)));
                Some((colossus, ring))
            } else {
                None
            };
            let company = put(&mut g, p, "Bolg's Company", Zone::Battlefield);
            let prison = put(&mut g, p, "Banishing Light", Zone::Battlefield);
            let aura = put(&mut g, p.other(), "Witness Protection", Zone::Exile);
            g.state.exile_links.push(ExileLink {
                source: g.object_ref(prison),
                exiled: g.object_ref(aura),
                return_zone: Zone::Battlefield,
            });
            g.change_object_form(
                g.object_ref(prison),
                crate::cards::AnimationSpec {
                    base: Some((2, 2)),
                    artifact: false,
                    creature_types: crate::cards::AnimationCreatureTypes::Named(&["Goblin"]),
                    keywords: Keywords::NONE,
                    duration: crate::cards::AnimationDuration::Permanent,
                },
                characteristic_layers::AnimationTypeMode::AddCreature,
            );
            g.state.players[p.index()].mana = [0; 6];
            attackers(&mut g, p);
            for a in army {
                act(&mut g, Action::Attack { card: a });
            }
            act(&mut g, Action::FinishAttackers);
            act(&mut g, Action::TapMana { card: forest });
            let irreversible_library = if let Some((colossus, ring)) = colossus {
                let grantor = g.object_ref(ring);
                act(
                    &mut g,
                    Action::TapGrantedMana {
                        card: colossus,
                        grantor,
                        production: [0, 0, 3, 0, 0, 0],
                    },
                );
                Some((g.state.players[p.index()].library.clone(), g.state.rng))
            } else {
                None
            };
            act(
                &mut g,
                Action::AnnounceMana {
                    card: company,
                    ability: 0,
                    counter_payment: Counters::default(),
                    production: [0, 0, 1, 1, 0, 0],
                },
            );
            let object = g.object_ref(prison);
            act(&mut g, Action::ChooseAdditionalCost { object });
            let payment = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            act(&mut g, payment);
            assert_eq!(g.actor(), Some(p.other()));

            let object = g.object_ref(elf);
            act(&mut g, Action::ChooseAuraAttachment { object });
            assert!(matches!(g.state.pending, Pending::ManaRollback { .. }));

            act(&mut g, Action::ChooseManaRollback { keep });
            if let Some((library, rng)) = irreversible_library {
                let view = g.observe(p).mana_rollback.unwrap();
                assert!(view.irreversible && view.can_keep && !view.can_reverse);
                reject(&mut g, Action::ChooseManaRollback { keep: false });
                act(&mut g, Action::ChooseManaRollback { keep: true });
                assert_eq!(g.object_view(colossus.unwrap().0).zone, Zone::Library);
                assert_eq!(g.state.players[p.index()].library, library);
                assert_eq!(g.state.rng, rng);
            }
            assert_eq!(g.object_view(forest).tapped, keep);
            assert_eq!(
                g.state.players[p.index()].mana,
                [
                    0,
                    0,
                    if irreversible { 3 } else { 0 },
                    0,
                    u32::from(keep),
                    0
                ]
            );
            assert_eq!(g.object_view(prison).zone, Zone::Battlefield);
            assert_eq!(g.object_view(aura).zone, Zone::Exile);
            assert!(g.state.attack_payment.is_none());
        }
    }
}
