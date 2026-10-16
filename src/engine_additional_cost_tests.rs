use super::tests::{act, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, owner, card, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(saved, g.snapshot());
    assert_eq!(revision, g.revision());
}
fn choose(g: &mut Game, id: CardId) {
    replay(
        g,
        Action::ChooseAdditionalCost {
            object: g.object_ref(id),
        },
    );
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
#[test]
fn mandatory_costs_have_no_direct_cast_or_payment_bypass_and_reject_invalid_objects() {
    for (name, permanent) in [
        ("Corrupted Conviction", "Grizzly Bears"),
        ("Final Vengeance", "Banishing Light"),
        ("Thrill of Possibility", "Forest"),
    ] {
        let mut g = scenario();
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        g.state.players[0].mana = [4; 6];
        let target = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        let zone = if name == "Thrill of Possibility" {
            Zone::Hand
        } else {
            Zone::Battlefield
        };
        let cost = named(&mut g, Player::P0, permanent, zone);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        reject(
            &mut g,
            Action::Cast {
                card: spell,
                target: None,
                payment: [0; 6],
            },
        );
        reject(
            &mut g,
            Action::BeginCast {
                card: spell,
                payment: [0; 6],
            },
        );
        replay(&mut g, Action::AnnounceCast { card: spell });
        if name == "Final Vengeance" {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        }
        assert_eq!(g.observe(Player::P0).decision, "additional_cost");
        assert!(g.observe(Player::P0).payment_request.is_none());
        assert!(g.observe(Player::P1).additional_cost.is_none());
        let view = g.observe(Player::P0).additional_cost.unwrap();
        assert_eq!(view.candidates, vec![g.object_ref(cost)]);
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        reject(&mut g, Action::Pass);
        for bad in [
            g.object_ref(spell),
            g.object_ref(target),
            ObjectRef {
                card: u32::MAX,
                generation: 0,
            },
            ObjectRef {
                card: cost,
                generation: g.object_ref(cost).generation + 1,
            },
        ] {
            reject(&mut g, Action::ChooseAdditionalCost { object: bad });
        }
        choose(&mut g, cost);
        assert_eq!(g.object_view(cost).zone, zone);
        assert!(g.state.stack.is_empty());
        assert_eq!(
            g.observe(Player::P0).additional_cost.unwrap().selected,
            Some(g.object_ref(cost))
        );
        pay(&mut g);
        assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
        assert_eq!(g.state.stack.len(), 1);
        pass_pair(&mut g);
        if name == "Final Vengeance" {
            assert_eq!(g.object_view(target).zone, Zone::Exile);
        } else {
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.owner == Player::P0 && o.zone == Zone::Hand)
                    .count(),
                2
            );
        }
    }
}
#[test]
fn sacrifice_target_can_fizzle_but_cost_is_never_refunded() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Final Vengeance", Zone::Hand);
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.state.players[0].mana[2] = 1;
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    choose(&mut g, bear);
    pay(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].mana[2], 0);
}
#[test]
fn reserved_creature_can_tap_for_mana_then_be_sacrificed() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Corrupted Conviction", Zone::Hand);
    let elf = named(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    g.state.objects[elf as usize].summoning_sick = false;
    // Synthetic green enclosing cost exercises reservation independently of the printed black cost.
    let spec = Card::from_name("Corrupted Conviction")
        .unwrap()
        .additional_cost()
        .unwrap();
    assert!(g.can_fund(FundingGoal {
        actor: Player::P0,
        required: [0, 0, 0, 0, 1, 0],
        generic: 0,
        source: None,
        reserve: Some((g.object_ref(elf), spec))
    }));
    g.state.players[0].mana[2] = 1;
    replay(&mut g, Action::AnnounceCast { card: spell });
    choose(&mut g, elf);
    // A mana step that doesn't help this printed cost isn't offered; the native funding state still retains a tappable reservation.
    let mut a = g.state.announcing.as_ref().unwrap().clone();
    a.cost = Some(ManaCost {
        colored: [0, 0, 0, 0, 1, 0],
        generic: 0,
    });
    // Replace the synthetic locked total too; changing an equivalent after
    // declaration no longer changes the already locked payment requirement.
    a.total_cost = a.cost;
    g.state.announcing = Some(a);
    g.state.players[0].mana = [0; 6];
    replay(&mut g, Action::TapMana { card: elf });
    assert!(g.object_view(elf).tapped);
    pay(&mut g);
    assert_eq!(g.object_view(elf).zone, Zone::Graveyard);
}
#[test]
fn reserved_treasure_cannot_be_spent_twice_and_prefunded_reservations_are_validated() {
    let mut g = scenario();
    let factory = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.name == "Treasure Token"))
        .unwrap();
    let token = spawn(&mut g, Player::P0, factory, Zone::Battlefield);
    let spec = AdditionalCost {
        kind: EffectChoiceKind::Sacrifice,
        filter: SacrificeFilter {
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 16,
            nonland: false,
            other: false,
            tokens: None,
            subtype: None,
        },
    };
    let goal = FundingGoal {
        actor: Player::P0,
        required: [0, 0, 1, 0, 0, 0],
        generic: 0,
        source: None,
        reserve: Some((g.object_ref(token), spec)),
    };
    assert!(!g.can_fund(goal));
    let second = spawn(&mut g, Player::P0, factory, Zone::Battlefield);
    assert!(g.can_fund(goal));
    // Two otherwise identical resources must retain distinct reservation roles in search keys.
    g.move_object(second, Zone::Graveyard);
    g.state.players[0].mana[2] = 1;
    assert!(g.can_fund(goal));
    g.move_object(token, Zone::Graveyard);
    assert!(!g.can_fund(goal));
    g.move_object(token, Zone::Battlefield);
    assert!(!g.can_fund(goal));
}
#[test]
fn paying_cost_can_suspend_for_returned_land_entry_before_cast_completion() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Final Vengeance", Zone::Hand);
    let light = named(&mut g, Player::P0, "Banishing Light", Zone::Battlefield);
    let held = named(&mut g, Player::P0, "Temple Garden", Zone::Exile);
    let target = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    // Explicit synthetic link: Banishing Light itself does not target lands.
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(light),
        exiled: g.object_ref(held),
        return_zone: Zone::Battlefield,
    });
    g.state.players[0].mana[2] = 1;
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    choose(&mut g, light);
    pay(&mut g);
    assert_eq!(g.observe(Player::P0).decision, "entry_life");
    assert!(g.observe(Player::P0).payment_request.is_none());
    assert!(g.observe(Player::P1).payment_request.is_none());
    assert!(g.state.stack.is_empty());
    assert!(g.state.announcing.as_ref().unwrap().additional_paid);
    assert_eq!(g.state.players[0].mana[2], 0);
    assert_eq!(g.object_view(light).zone, Zone::Graveyard);
    replay(&mut g, Action::ChooseEntryLife { pay: true });
    assert!(g.state.announcing.is_none());
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.players[0].life, 18);
    assert!(!g.object_view(held).tapped);
    assert_eq!(g.state.players[0].mana[2], 0);
    pass_pair(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Exile);
}
#[test]
fn discard_is_private_until_payment_and_its_trigger_waits_until_spell_is_cast() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Thrill of Possibility", Zone::Hand);
    let discard = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    let archer = named(&mut g, Player::P0, "Firebrand Archer", Zone::Battlefield);
    g.state.players[0].mana = [0, 0, 0, 2, 0, 0];
    replay(&mut g, Action::AnnounceCast { card: spell });
    choose(&mut g, discard);
    let opponent = g.observe(Player::P1);
    assert!(opponent.additional_cost.is_none());
    assert!(!opponent.objects.iter().any(|o| o.id == discard));
    assert!(g.state.pending_triggers.is_empty());
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    pay(&mut g);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert_eq!(g.state.stack[0].card, spell);
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == discard && o.zone == Zone::Graveyard)
    );
    let waiting: Vec<_> = g.state.ordering_triggers.iter().map(|s| s.card).collect();
    assert!(waiting.contains(&mako) && waiting.contains(&archer));
    while let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        replay(&mut g, a);
    }
    assert!(g.state.stack.iter().any(|s|s.card==mako&&s.discard_event.is_some_and(|e|e.amount==1&&!e.cycling)));
    while !g.state.stack.is_empty() {
        pass_pair(&mut g);
    }
    assert_eq!(g.object_view(mako).counters.plus_one, 1);
    assert_eq!(g.state.players[1].life, 19);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
}
#[test]
fn returned_aura_finishes_entry_before_completed_cast_observers() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Final Vengeance", Zone::Hand);
    let light = named(&mut g, Player::P0, "Banishing Light", Zone::Battlefield);
    let aura = named(&mut g, Player::P0, "Ethereal Armor", Zone::Exile);
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let target = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(light),
        exiled: g.object_ref(aura),
        return_zone: Zone::Battlefield,
    });
    g.state.players[0].mana[2] = 1;
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    choose(&mut g, light);
    pay(&mut g);
    assert_eq!(g.observe(Player::P0).decision, "aura_entry");
    assert!(g.observe(Player::P0).payment_request.is_none());
    assert!(g.observe(Player::P1).payment_request.is_none());
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    assert!(g.state.stack.is_empty());
    let object = g.object_ref(bear);
    replay(&mut g, Action::ChooseAuraAttachment { object });
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(bear)));
    assert!(g.state.announcing.is_none());
}

#[test]
fn opponent_entry_chooser_never_receives_the_casters_paid_payment_request() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Final Vengeance", Zone::Hand);
    let light = named(&mut g, Player::P0, "Banishing Light", Zone::Battlefield);
    let held = named(&mut g, Player::P1, "Temple Garden", Zone::Exile);
    let target = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(light),
        exiled: g.object_ref(held),
        return_zone: Zone::Battlefield,
    });
    g.state.players[0].mana[2] = 1;
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    choose(&mut g, light);
    pay(&mut g);
    assert_eq!(g.actor(), Some(Player::P1));
    assert_eq!(g.observe(Player::P1).decision, "entry_life");
    assert!(g.observe(Player::P0).payment_request.is_none());
    assert!(g.observe(Player::P1).payment_request.is_none());
    assert!(g.observe(Player::P1).additional_cost.is_none());
    replay(&mut g, Action::ChooseEntryLife { pay: true });
    assert_eq!(g.state.players[1].life, 18);
    assert_eq!(g.state.players[0].mana[2], 0);
    assert_eq!(g.actor(), Some(Player::P0));
    assert_eq!(g.state.stack.len(), 1);
}
