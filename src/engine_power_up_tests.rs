//! Representative paid Power-up consumers, exact-object history and owned replay.
use super::assurance::big_helpers::{cast, checked, next_turn, settle, setup, spawn};
use super::*;
fn put(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn announce(id: CardId) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn reject(g: &mut Game, action: Action) {
    let snapshot = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
}
fn pay(g: &mut Game) {
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, action);
    }
    panic!("unfinished Power-up payment");
}
fn activate(g: &mut Game, id: CardId) {
    checked(g, announce(id));
    pay(g);
    settle(g);
}
#[test]
fn power_up_aerial_paid_entry_discount_once_control_blink_and_owned_replay() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = put(&mut g, p, "Aerial Doombot", Zone::Hand);
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(id).power, 1);
        assert!(g.object_view(id).keywords.contains(Keywords::FLYING));
        assert!(g.object_view(id).entered_this_turn);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        reject(&mut g, announce(id));
        g.state.players[p.index()].mana[5] = 5;
        checked(&mut g, announce(id));
        let payment = g.announcement_payment_request(p).unwrap();
        assert_eq!((payment.required, payment.generic), ([0; 6], 5));
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        pay(&mut g);
        assert_eq!(g.object_view(id).power_up_uses[0].count, 1);
        assert_eq!(g.object_view(id).exhausted_abilities, 0);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 3);
        assert_eq!(g.object_view(id).power, 4);
        reject(&mut g, announce(id));
        g.state.objects[id as usize].controller = p.other();
        g.state.active = p.other();
        g.state.priority = p.other();
        reject(&mut g, announce(id));
        g.state.objects[id as usize].controller = p;
        g.state.active = p;
        g.state.priority = p;
        next_turn(&mut g);
        next_turn(&mut g);
        reject(&mut g, announce(id));
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        assert!(g.object_view(id).power_up_uses.is_empty());
        assert!(g.object_view(id).entered_this_turn);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        activate(&mut g, id);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 3);
        assert_eq!(
            g.observe(p)
                .objects
                .iter()
                .find(|o| o.id == id)
                .unwrap()
                .power_up_uses,
            g.observe(p.other())
                .objects
                .iter()
                .find(|o| o.id == id)
                .unwrap()
                .power_up_uses
        );
    }
}
#[test]
fn power_up_entry_turn_expires_and_copy_has_independent_use() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = put(&mut g, p, "Aerial Doombot", Zone::Battlefield);
        next_turn(&mut g);
        next_turn(&mut g);
        assert!(!g.object_view(id).entered_this_turn);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        reject(&mut g, announce(id));
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 5];
        activate(&mut g, id);
        let definition = g.state.objects[id as usize].card;
        let copies = g.create_copy_groups_inner(
            p,
            &[TokenGroupView {
                token: definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(definition))],
            None,
        );
        g.advance_entries();
        let copied = copies[0].card;
        assert!(g.object_view(copied).power_up_uses.is_empty());
        assert!(g.object_view(copied).entered_this_turn);
        assert_eq!(g.counter_count(copied, CounterKind::PlusOne), 0);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        activate(&mut g, copied);
        assert_eq!(g.counter_count(copied, CounterKind::PlusOne), 3);
        assert_eq!(g.power_up_count(g.object_ref(id), definition, 0, None), 1);
        assert_eq!(
            g.power_up_count(g.object_ref(copied), definition, 0, None),
            1
        );
    }
}
#[test]
fn power_up_wonder_man_counts_uses_without_refreshing_replacement_providers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = put(&mut g, p, "Aerial Doombot", Zone::Battlefield);
        let wonder = put(&mut g, p, "Wonder Man, Hollywood Hero", Zone::Battlefield);
        assert_eq!(g.power_up_limit(wonder), 2); // Includes itself.
        activate(&mut g, id);
        activate(&mut g, id);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 6);
        reject(&mut g, announce(id));
        g.move_object(wonder, Zone::Graveyard);
        let replacement = put(&mut g, p, "Wonder Man, Hollywood Hero", Zone::Battlefield);
        reject(&mut g, announce(id));
        activate(&mut g, replacement);
        activate(&mut g, replacement);
        assert_eq!(g.counter_count(replacement, CounterKind::PlusOne), 4);
        reject(&mut g, announce(replacement));
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(replacement)]);
        settle(&mut g);
        assert!(!g.printed_abilities_available(replacement));
        assert_eq!(g.power_up_limit(id), 1);
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(g.power_up_limit(id), 2);
        reject(&mut g, announce(id));
        // Multiple independent modifiers add; inspect the simultaneous world
        // before the next SBA applies the legend rule.
        let extra = put(&mut g, p, "Wonder Man, Hollywood Hero", Zone::Battlefield);
        assert_eq!(g.power_up_limit(id), 3);
        g.move_object(extra, Zone::Exile);
        assert_eq!(g.power_up_limit(id), 2);
    }
}
#[test]
fn power_up_hulk_other_creatures_generic_only_suppression_and_locked_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hulk = put(&mut g, p, "Hulk, Gamma Goliath", Zone::Battlefield);
        let id = put(&mut g, p, "Aerial Doombot", Zone::Battlefield);
        let definition = g.state.objects[id as usize].card;
        assert_eq!(
            g.activation_mana_costs(hulk, g.state.objects[hulk as usize].card, 0, 0)
                .collect::<Vec<_>>(),
            vec![ManaCost {
                colored: [0; 6],
                generic: 3
            }]
        );
        assert!(g.object_view(hulk).keywords.contains(Keywords::REACH));
        assert!(g.object_view(hulk).keywords.contains(Keywords::TRAMPLE));
        assert_eq!(
            g.activation_mana_costs(id, definition, 0, 0)
                .collect::<Vec<_>>(),
            vec![ManaCost {
                colored: [0; 6],
                generic: 2
            }]
        );
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(hulk)]);
        settle(&mut g);
        assert_eq!(
            g.activation_mana_costs(id, definition, 0, 0)
                .next()
                .unwrap()
                .generic,
            5
        );
        g.move_object(aura, Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, announce(id));
        // Total cost is locked when the activation starts. Departing providers
        // cannot raise it during the mana-ability/payment window.
        g.move_object(hulk, Zone::Exile);
        assert_eq!(g.announcement_payment_request(p).unwrap().generic, 2);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 3);
        g.move_object(hulk, Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        activate(&mut g, hulk);
        assert_eq!(g.counter_count(hulk, CounterKind::PlusOne), 5);
        assert_eq!(g.object_view(hulk).power, 11);
    }
}
#[test]
fn power_up_source_departure_does_not_rebind_the_old_packet() {
    let p = Player::P0;
    let mut g = setup(p);
    let id = put(&mut g, p, "Aerial Doombot", Zone::Battlefield);
    checked(&mut g, announce(id));
    pay(&mut g);
    let old = g.object_ref(id);
    g.move_object(id, Zone::Exile);
    g.move_object(id, Zone::Battlefield);
    assert_ne!(old, g.object_ref(id));
    settle(&mut g);
    assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
    assert!(g.object_view(id).power_up_uses.is_empty());
    activate(&mut g, id);
    assert_eq!(g.counter_count(id, CounterKind::PlusOne), 3);
}
#[test]
fn power_up_typed_reduction_hybrid_choices_excess_colorless_and_x_order() {
    use power_up::reduce;
    let base = ManaCost {
        colored: [0, 1, 0, 0, 0, 1],
        generic: 1,
    };
    let reduction = ManaCost {
        colored: [0, 0, 0, 3, 0, 2],
        generic: 2,
    };
    // RRR has no red cost to remove: it reduces generic, as does excess C.
    assert_eq!(
        reduce(base, reduction, 1, 10),
        Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 4
        })
    );
    assert_eq!(
        reduce(base, reduction, 1, 0),
        Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 0
        })
    );
    let options = [
        ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        },
        ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 1,
        },
    ];
    let base = ManaCost {
        colored: [0, 1, 0, 0, 0, 0],
        generic: 4,
    };
    assert_eq!(
        reduce(base, options[0], 0, 0),
        Some(ManaCost {
            colored: [0; 6],
            generic: 3
        })
    );
    assert_eq!(
        reduce(base, options[1], 0, 0),
        Some(ManaCost {
            colored: base.colored,
            generic: 2
        })
    );
    // A copied {X} activated slot demonstrates the actual saved-announcement X
    // domain: the source's mana cost is a reduction, with X=0 on the battlefield.
    let mut g = setup(Player::P0);
    // Exercise the existing saved-recipe accessor directly. This checks the
    // reduction query without admitting an in-place copying card or effect.
    let source = put(&mut g, Player::P0, "Aerial Doombot", Zone::Battlefield);
    let aerial = g.state.objects[source as usize].card;
    g.set_copy_recipe(
        g.object_ref(source),
        CopyRecipe::new(Card::from_name("Hulk, Gamma Goliath").unwrap()),
    );
    assert_eq!(
        g.activation_mana_costs(source, aerial, 0, 0)
            .collect::<Vec<_>>(),
        vec![ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 0
        }]
    );
    g.clear_copy_recipe(g.object_ref(source));
    assert_eq!(
        g.activation_mana_costs(source, aerial, 0, 0)
            .next()
            .unwrap()
            .generic,
        5
    );
    let id = put(&mut g, Player::P0, "Steel Hellkite", Zone::Battlefield);
    let item = g.activation_item(Player::P0, id, 1);
    g.open_payment_announcement(item, Counters::default(), None);
    g.state.announcing.as_mut().unwrap().power_up_costs = Some(Box::new(power_up::PowerUpCosts {
        reductions: vec![ManaCost {
            colored: [0; 6],
            generic: 6,
        }],
    }));
    g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
    assert_eq!(
        g.x_choice().unwrap().intervals,
        vec![XInterval {
            minimum: 0,
            maximum: 8
        }]
    );
    checked(&mut g, Action::ChooseX { value: 8 });
    assert_eq!(
        g.announcement_payment_request(Player::P0).unwrap().generic,
        2
    );
    pay(&mut g);
    settle(&mut g);
}
#[test]
fn power_up_funds_with_real_land_mana_and_preserves_the_entry_discount() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = put(&mut g, p, "Aerial Doombot", Zone::Battlefield);
        let lands: Vec<_> = (0..5)
            .map(|_| put(&mut g, p, "Forest", Zone::Battlefield))
            .collect();
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, announce(id));
        assert_eq!(g.announcement_payment_request(p).unwrap().generic, 5);
        for card in lands {
            checked(&mut g, Action::TapMana { card });
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 3);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn power_up_inherited_ability_uses_its_own_slot_and_the_actual_source_mana_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let donor = put(&mut g, p.other(), "Aerial Doombot", Zone::Battlefield);
        let forge = put(&mut g, p, "Territory Forge", Zone::Hand);
        cast(&mut g, forge, None, &[]);
        settle(&mut g);
        // Territory Forge's entry trigger is compulsory; select its artifact.
        // The ordinary settle helper already chooses the first legal target.
        assert_eq!(g.object_view(donor).zone, Zone::Exile);
        let descriptor = g
            .borrowed_ability_views()
            .into_iter()
            .find(|a| a.source.card == forge && a.exiled.card == donor)
            .unwrap();
        let action = Action::AnnounceBorrowedAbility {
            card: forge,
            exiled: descriptor.exiled,
            ability: 0,
            counter_payment: Counters::default(),
        };
        let (colored, generic) = g.state.objects[forge as usize].card.cost();
        let hulk = put(&mut g, p, "Hulk, Gamma Goliath", Zone::Battlefield);
        assert_eq!(
            g.power_up_generic_discount(forge, descriptor.definition, 0),
            0
        );
        g.move_object(hulk, Zone::Exile);
        let expected = power_up::reduce(
            ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 5,
            },
            ManaCost { colored, generic },
            0,
            0,
        )
        .unwrap();
        g.state.players[p.index()].mana = [10; 6];
        checked(&mut g, action.clone());
        let payment = g.announcement_payment_request(p).unwrap();
        assert_eq!(
            (payment.required, payment.generic),
            (expected.colored, expected.generic)
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(forge, CounterKind::PlusOne), 3);
        assert_eq!(
            g.power_up_count(
                g.object_ref(forge),
                descriptor.definition,
                0,
                Some(descriptor.slot)
            ),
            1
        );
        reject(&mut g, action.clone());
        next_turn(&mut g);
        next_turn(&mut g);
        reject(&mut g, action.clone());
        let wonder = put(&mut g, p, "Wonder Man, Hollywood Hero", Zone::Battlefield);
        g.state.players[p.index()].mana = [10; 6];
        checked(&mut g, action.clone());
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(forge, CounterKind::PlusOne), 6);
        reject(&mut g, action.clone());
        g.move_object(wonder, Zone::Exile);
        g.move_object(donor, Zone::Hand);
        reject(&mut g, action);
        assert!(g.borrowed_ability_views().is_empty());
        // A copied entry packet can grant a distinct instance of the same
        // Power-up definition: its lifetime allowance belongs to its slot.
        g.move_object(donor, Zone::Battlefield);
        let mut packet = g.activation_item(p, forge, 0);
        packet.targets = vec![g.bind_target(Target::Card(donor))];
        g.exile_and_borrow_activated_abilities(&packet);
        let second = g
            .borrowed_ability_views()
            .into_iter()
            .find(|a| a.source.card == forge)
            .unwrap();
        assert_ne!(second.slot, descriptor.slot);
        let second_action = Action::AnnounceBorrowedAbility {
            card: forge,
            exiled: second.exiled,
            ability: 0,
            counter_payment: Counters::default(),
        };
        checked(&mut g, second_action);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(
            g.power_up_count(
                g.object_ref(forge),
                descriptor.definition,
                0,
                Some(second.slot)
            ),
            1
        );
    }
}
