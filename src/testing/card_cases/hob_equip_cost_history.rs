//! Focused printed outcomes; shared engine invariants live in core tests.
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

#[test]
fn mauler_paid_body_and_target_specific_direct_and_announced_generic_reductions() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn kili_paid_entry_storied_is_persistent_and_self_entry_does_not_draw() {
    {
        let p = Player::P0;
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
    const { assert!(OBSERVATION_SCHEMA >= 156) };
}

#[test]
fn restricted_equip_variants_still_require_legal_targets_and_non_equip_is_excluded() {
    {
        let p = Player::P0;
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
