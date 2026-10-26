//! Paid equip outcomes, whole-cost route choices, and global activation history.
use super::*;
fn start(g: &mut Game, card: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn target(g: &mut Game, card: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn route(g: &mut Game, p: Player, wanted: ActivationCostRoute) {
    let choice = g
        .observe(p)
        .cost_choices
        .into_iter()
        .find(|c| c.activation_route == Some(wanted))
        .expect("cost route");
    act(g, Action::ChooseActivationCost { cost: choice.index });
}
fn equip(g: &mut Game, e: CardId, recipient: CardId) {
    start(g, e, 0);
    target(g, recipient);
    pay(g);
    settle(g);
}
fn story(g: &mut Game, p: Player) -> CardId {
    let k = put(g, p, "Kíli the Resourceful", Zone::Battlefield);
    put(g, p, "Goldvein Pick", Zone::Battlefield);
    put(g, p, "Swiftfoot Boots", Zone::Battlefield);
    g.refresh_enduring_stories();
    assert!(g.state.players[p.index()].enduring_story);
    k
}
fn reset(g: &mut Game, p: Player) {
    g.state.active = p;
    g.begin_turn();
    g.state.pending = Pending::Priority;
    g.state.step = Step::Main1;
    g.state.priority = p;
}
fn types(g: &mut Game, id: CardId, types: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types,
            additional: false,
            timestamp: 1,
        });
}
fn sacrifice_mana(g: &mut Game, bolg: CardId, fodder: CardId) {
    act(
        g,
        Action::AnnounceMana {
            card: bolg,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 1, 1, 0, 0],
        },
    );
    let object = g.object_ref(fodder);
    act(g, Action::ChooseAdditionalCost { object });
    pay(g);
}
#[test]
fn mauler_paid_body_and_target_specific_direct_and_announced_generic_reductions() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mauler = put(&mut g, p, "Dwarven Mauler", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        cast(&mut g, mauler, None);
        settle(&mut g);
        assert_eq!(stats(&g, mauler), (2, 1));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Dwarven Mauler", Zone::Battlefield);
        let spear = put(&mut g, p, "Ragged Short Spear", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let paid = Action::Activate {
            card: spear,
            ability: 0,
            target: Some(Target::Card(mauler)),
            payment: [0, 0, 0, 0, 0, 1],
            counter_payment: Counters::default(),
        };
        assert!(g.legal_actions().contains(&paid));
        reject(
            &mut g,
            Action::Activate {
                card: spear,
                ability: 0,
                target: Some(Target::Card(bear)),
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::Activate {
                card: spear,
                ability: 0,
                target: Some(Target::Card(enemy)),
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
            },
        );
        act(&mut g, paid);
        settle(&mut g);
        assert_eq!(g.object_view(spear).attached_to, Some(g.object_ref(mauler)));
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        equip(&mut g, pick, mauler);
        assert_eq!(g.object_view(pick).attached_to, Some(g.object_ref(mauler)));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn mauler_current_control_and_ability_availability_matter_not_ownership() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mauler = put(&mut g, p, "Dwarven Mauler", Zone::Battlefield);
        g.state.objects[mauler as usize].owner = p.other();
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spear = put(&mut g, p, "Ragged Short Spear", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        equip(&mut g, spear, mauler);
        g.blank_and_grant_haste(g.object_ref(mauler), g.object_ref(bear));
        assert!(!g.printed_abilities_available(mauler));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: spear,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        equip(&mut g, spear, mauler);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn mauler_locked_cost_survives_target_sacrifice_and_stale_target_does_not_attach() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mauler = put(&mut g, p, "Dwarven Mauler", Zone::Battlefield);
        types(&mut g, mauler, &["Dwarf", "Goblin"]);
        let bolg = put(&mut g, p, "Bolg's Company", Zone::Battlefield);
        let spear = put(&mut g, p, "Ragged Short Spear", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        start(&mut g, spear, 0);
        target(&mut g, mauler);
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
        sacrifice_mana(&mut g, bolg, mauler);
        contracts::branch_contract(g.clone());
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        pay(&mut g);
        assert!(g.state.players[p.index()].equip_activated_this_turn);
        assert_eq!(g.state.players[p.index()].mana.iter().sum::<u32>(), 1);
        g.move_object(mauler, Zone::Battlefield); // new incarnation cannot rebind
        settle(&mut g);
        assert_eq!(g.object_view(spear).attached_to, None);
    }
}
#[test]
fn kili_paid_entry_storied_is_persistent_and_self_entry_does_not_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let boots = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
        let k = put(&mut g, p, "Kíli the Resourceful", Zone::Hand);
        let hand = g.hand_count(p);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        cast(&mut g, k, None);
        settle(&mut g);
        assert_eq!(stats(&g, k), (1, 2));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.hand_count(p), hand - 1);
        assert!(g.state.players[p.index()].enduring_story);
        g.move_object(pick, Zone::Graveyard);
        g.move_object(boots, Zone::Graveyard);
        assert!(g.state.players[p.index()].enduring_story);
    }
}
#[test]
fn kili_zero_route_replaces_my_precious_life_cost_and_decline_keeps_it() {
    for p in [Player::P0, Player::P1] {
        for free in [false, true] {
            let mut g = scenario(p);
            story(&mut g, p);
            let mauler = put(&mut g, p, "Dwarven Mauler", Zone::Battlefield);
            let precious = put(
                &mut g,
                p,
                "My Precious // Allure of Power",
                Zone::Battlefield,
            );
            let life = g.state.players[p.index()].life;
            g.state.players[p.index()].mana = [0; 6];
            start(&mut g, precious, 0);
            let choices = g.observe(p).cost_choices;
            assert_eq!(choices.len(), 2);
            // The printed route still has generic2 before target-dependent reduction.
            assert!(
                choices
                    .iter()
                    .any(|c| c.activation_route == Some(ActivationCostRoute::Printed))
            );
            assert!(
                choices
                    .iter()
                    .any(|c| c.activation_route == Some(ActivationCostRoute::FirstEquipZero))
            );
            assert!(g.observe(p.other()).cost_choices.is_empty());
            if p == Player::P0 && free {
                contracts::branch_contract(g.clone());
            }
            route(
                &mut g,
                p,
                if free {
                    ActivationCostRoute::FirstEquipZero
                } else {
                    ActivationCostRoute::Printed
                },
            );
            target(&mut g, mauler);
            assert_eq!(
                g.state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .total_cost
                    .unwrap()
                    .generic,
                0
            );
            reject(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 0, 1],
                },
            );
            pay(&mut g);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                life - if free { 0 } else { 2 }
            );
            assert_eq!(
                g.object_view(precious).attached_to,
                Some(g.object_ref(mauler))
            );
            assert!(g.state.players[p.index()].equip_activated_this_turn);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            // Declining consumes the first activation just as accepting does.
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let pick = put(&mut g, p, "Ragged Short Spear", Zone::Battlefield);
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: pick,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            assert_ne!(bear, mauler);
        }
    }
}
#[test]
fn kili_equal_mana_choice_discriminator_and_unrelated_choice_shapes() {
    let p = Player::P0;
    let mut g = scenario(p);
    story(&mut g, p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let precious = put(
        &mut g,
        p,
        "My Precious // Allure of Power",
        Zone::Battlefield,
    );
    // Fixture an already reduced printed mana cost through the captured generic discount.
    start(&mut g, precious, 0);
    g.state.announcing.as_mut().unwrap().activation_discount = 2;
    let views = g.observe(p).cost_choices;
    assert_eq!(views.len(), 2);
    assert!(views.iter().all(|v| v.cost
        == ManaCost {
            colored: [0; 6],
            generic: 0
        }));
    let json = serde_json::to_value(&views).unwrap();
    assert_eq!(json[0]["activation_route"], "printed");
    assert_eq!(json[1]["activation_route"], "first_equip_zero");
    route(&mut g, p, ActivationCostRoute::FirstEquipZero);
    target(&mut g, bear);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, 20);
    // Existing spell and unrelated activation choices retain their old shape,
    // even while an eligible Kíli is present on the battlefield.
    let mut spell_game = scenario(p);
    story(&mut spell_game, p);
    let spell = put(&mut spell_game, p, "Bifur, Melodic Rider", Zone::Hand);
    act(&mut spell_game, Action::AnnounceCast { card: spell });
    let choices = spell_game.observe(p).cost_choices;
    assert!(choices.len() > 1);
    assert!(choices.iter().all(|c| c.activation_route.is_none()));
    assert!(
        serde_json::to_value(&choices)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c.get("activation_route").is_none())
    );
    let mut other = scenario(p);
    story(&mut other, p);
    put(&mut other, p, "Bear Cub", Zone::Battlefield);
    let source = put(&mut other, p, "Trostani, Three Whispers", Zone::Battlefield);
    start(&mut other, source, 1);
    let choices = other.observe(p).cost_choices;
    assert!(choices.len() > 1);
    assert!(choices.iter().all(|c| c.activation_route.is_none()));
    assert!(
        serde_json::to_value(&choices)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c.get("activation_route").is_none())
    );
    assert_eq!(OBSERVATION_SCHEMA, 156);
}
#[test]
fn kili_free_route_available_when_printed_life_is_unpayable_and_timing_still_restricted() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        story(&mut g, p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let e = put(
            &mut g,
            p,
            "My Precious // Allure of Power",
            Zone::Battlefield,
        );
        g.state.players[p.index()].life = 1;
        g.state.players[p.index()].mana = [0; 6];
        g.state.active = p.other();
        g.state.priority = p;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: e,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.active = p;
        start(&mut g, e, 0);
        assert_eq!(g.observe(p).cost_choices.len(), 1);
        assert_eq!(
            g.observe(p).cost_choices[0].activation_route,
            Some(ActivationCostRoute::FirstEquipZero)
        );
        reject(&mut g, Action::ChooseActivationCost { cost: 0 });
        assert!(!g.state.players[p.index()].equip_activated_this_turn);
        route(&mut g, p, ActivationCostRoute::FirstEquipZero);
        target(&mut g, bear);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 1);
    }
}
#[test]
fn equip_history_before_provider_or_story_countered_and_failed_resolution_still_consumes() {
    for p in [Player::P0, Player::P1] {
        for outcome in 0..3 {
            let mut g = scenario(p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let boots = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            start(&mut g, boots, 0);
            target(&mut g, bear);
            pay(&mut g);
            assert!(g.state.players[p.index()].equip_activated_this_turn);
            if outcome == 1 {
                // Real shared counter event; there is no admitted Stifle fixture.
                let id = g.state.stack.last().unwrap().id;
                assert!(g.counter_stack_item(id, false));
            } else {
                if outcome == 2 {
                    g.move_object(bear, Zone::Graveyard);
                }
                settle(&mut g);
            }
            story(&mut g, p);
            let second = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = [0; 6];
            g.state.priority = p;
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: second,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reset(&mut g, p);
            assert!(!g.state.players[p.index()].equip_activated_this_turn);
            start(&mut g, second, 0);
            route(&mut g, p, ActivationCostRoute::FirstEquipZero);
        }
    }
}
#[test]
fn kili_route_survives_provider_sacrifice_and_owned_payment_branches() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let k = story(&mut g, p);
        types(&mut g, k, &["Dwarf", "Goblin"]);
        let bolg = put(&mut g, p, "Bolg's Company", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let e = put(
            &mut g,
            p,
            "My Precious // Allure of Power",
            Zone::Battlefield,
        );
        g.state.players[p.index()].mana = [0; 6];
        start(&mut g, e, 0);
        route(&mut g, p, ActivationCostRoute::FirstEquipZero);
        target(&mut g, bear);
        sacrifice_mana(&mut g, bolg, k);
        contracts::branch_contract(g.clone());
        assert!(!g.state.players[p.index()].equip_activated_this_turn);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 1, 0, 0]);
        assert_eq!(g.object_view(e).attached_to, Some(g.object_ref(bear)));
        contracts::branch_contract(g.clone());
    }
}
#[test]
fn restricted_equip_variants_still_require_legal_targets_and_non_equip_is_excluded() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let k = story(&mut g, p);
        let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
        let mauler = put(&mut g, p, "Dwarven Mauler", Zone::Battlefield);
        let wizard = put(&mut g, p, "Radagast of Rhosgobel", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        start(&mut g, staff, 0);
        route(&mut g, p, ActivationCostRoute::FirstEquipZero);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mauler),
            },
        );
        target(&mut g, wizard);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(staff).attached_to, Some(g.object_ref(wizard)));
        assert!(!g.first_equip_zero_available(
            p,
            Card::from_name("Axgard Cavalry").unwrap().activation(0)
        ));
        // The current pool gates reconfigure; non-equip attachment effects are
        // tested below and are never classified by their instruction alone.
        assert!(
            Card::from_name("Wizard's Staff")
                .unwrap()
                .activation(0)
                .is_equip()
        );
        g.blank_and_grant_haste(g.object_ref(k), g.object_ref(mauler));
        reset(&mut g, p);
        assert!(!g.first_equip_zero_available(
            p,
            Card::from_name("Goldvein Pick").unwrap().activation(0)
        ));
    }
}
#[test]
fn copied_ability_and_triggered_attachment_do_not_consume_another_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let sting = put(&mut g, p, "Sting, Bilbo's Sword", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        cast(&mut g, sting, None);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        target(&mut g, bearer);
        settle(&mut g);
        assert_eq!(g.object_view(sting).attached_to, Some(g.object_ref(bearer)));
        assert!(!g.state.players[p.index()].equip_activated_this_turn);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        start(&mut g, sting, 0);
        target(&mut g, bearer);
        pay(&mut g);
        let mut copy = g.state.stack.last().unwrap().clone();
        copy.id = g.next_stack_id();
        copy.controller = p.other();
        copy.is_copy = true;
        g.push_stack_item(copy);
        assert!(g.state.players[p.index()].equip_activated_this_turn);
        assert!(!g.state.players[p.other().index()].equip_activated_this_turn);
        contracts::branch_contract(g.clone());
    }
}
#[test]
fn kili_draw_uses_other_controlled_subtypes_once_each_turn_and_survives_departure() {
    for p in [Player::P0, Player::P1] {
        for equipment in [false, true] {
            let mut g = scenario(p);
            let k = put(&mut g, p, "Kíli the Resourceful", Zone::Battlefield);
            // Opponent-controlled qualifying entries and own nonmatching entries do not draw.
            g.state.active = p.other();
            g.state.priority = p.other();
            let enemy = put(&mut g, p.other(), "Axgard Cavalry", Zone::Hand);
            let own_hand = g.hand_count(p);
            cast(&mut g, enemy, None);
            settle(&mut g);
            assert_eq!(g.hand_count(p), own_hand);
            g.state.active = p;
            g.state.priority = p;
            let ordinary = put(&mut g, p, "Bear Cub", Zone::Hand);
            let own_hand = g.hand_count(p);
            cast(&mut g, ordinary, None);
            settle(&mut g);
            assert_eq!(g.hand_count(p), own_hand - 1);
            let c = put(
                &mut g,
                p,
                if equipment {
                    "Goldvein Pick"
                } else {
                    "Axgard Cavalry"
                },
                Zone::Hand,
            );
            let hand = g.hand_count(p);
            cast(&mut g, c, None);
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand);
            let c2 = put(&mut g, p, "Goldvein Pick", Zone::Hand);
            let hand = g.hand_count(p);
            cast(&mut g, c2, None);
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand - 1);
            reset(&mut g, p);
            g.state.players[p.index()].mana = [8; 6];
            let c3 = put(&mut g, p, "Goldvein Pick", Zone::Hand);
            let hand = g.hand_count(p);
            cast(&mut g, c3, None);
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            g.move_object(k, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand);
        }
    }
}

#[test]
fn kili_prior_equip_without_story_consumes_first_after_story_is_achieved() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Kíli the Resourceful", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let boots = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
        g.refresh_enduring_stories();
        assert!(!g.state.players[p.index()].enduring_story);
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: boots,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        equip(&mut g, boots, bear);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        assert!(g.state.players[p.index()].enduring_story);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: pick,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reset(&mut g, p);
        start(&mut g, pick, 0);
        route(&mut g, p, ActivationCostRoute::FirstEquipZero);
        target(&mut g, bear);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(pick).attached_to, Some(g.object_ref(bear)));
    }
}
#[test]
fn kili_printed_trigger_once_cap_survives_bifur_multiplicity_and_resets() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let k = story(&mut g, p);
        put(&mut g, p, "Bifur, Melodic Rider", Zone::Battlefield);
        assert_eq!(g.object_view(k).additional_triggers, 1);
        let equipment = put(&mut g, p, "Goldvein Pick", Zone::Hand);
        let hand = g.hand_count(p);
        cast(&mut g, equipment, None);
        settle(&mut g);
        assert_eq!(
            g.hand_count(p),
            hand,
            "one draw despite an active extra-trigger provider"
        );
        let dwarf = put(&mut g, p, "Axgard Cavalry", Zone::Hand);
        let hand = g.hand_count(p);
        cast(&mut g, dwarf, None);
        settle(&mut g);
        assert_eq!(
            g.hand_count(p),
            hand - 1,
            "later qualifying entry cannot trigger again"
        );
        reset(&mut g, p);
        g.state.players[p.index()].mana = [8; 6];
        let dwarf = put(&mut g, p, "Axgard Cavalry", Zone::Hand);
        let hand = g.hand_count(p);
        cast(&mut g, dwarf, None);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand, "one draw on the next turn");
    }
}

#[test]
fn ordinary_equip_preserves_legacy_payment_major_direct_action_order() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let equipment = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.state.players[p.index()].mana = [1, 1, 0, 0, 0, 0];
        let rows: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter_map(|action| {
                if let Action::Activate {
                    card,
                    target: Some(Target::Card(recipient)),
                    payment,
                    ..
                } = action
                {
                    (card == equipment).then_some((recipient, payment))
                } else {
                    None
                }
            })
            .collect();
        // Existing protocol order traverses each concrete payment, then its
        // targets in object order. Each of two colors can pay this equip1.
        assert_eq!(rows.len(), 4);
        assert_eq!(
            rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            vec![a, b, a, b]
        );
        assert_eq!(rows[0].1, rows[1].1);
        assert_eq!(rows[2].1, rows[3].1);
        assert_ne!(rows[0].1, rows[2].1);
    }
}
