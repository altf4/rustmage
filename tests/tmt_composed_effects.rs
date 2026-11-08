//! Paid scenarios for the casting, attachment and linked movement integration.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn drain(g: &mut Game, target: Option<CardId>) {
    for _ in 0..40 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.announcing.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let select = target.map(|card| Action::ChooseTarget {
            target: Target::Card(card),
        });
        let action = select
            .filter(|action| actions.contains(action))
            .or_else(|| {
                actions.into_iter().find(|action| {
                    matches!(
                        action,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishTargets
                            | Action::DeclineResolutionMana
                    )
                })
            })
            .unwrap_or_else(|| panic!("unexpected choice {:?}", g.state.pending));
        act(g, action);
    }
    panic!("resolution action budget exhausted");
}

fn activate(g: &mut Game, source: CardId, ability: u16, target: Option<CardId>) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(card) = target {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    pay(g);
}

#[test]
fn stomped_by_the_foot_kicker_consumes_a_legal_sacrifice_and_changes_the_effect() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Stomped by the Foot", Zone::Hand);
        let target = put(
            &mut g,
            p.other(),
            "Ghalta, Primal Hunger",
            Zone::Battlefield,
        );
        let land = put(&mut g, p, "Swamp", Zone::Battlefield);
        let sacrifice = put(&mut g, p, "Healer's Hawk", Zone::Battlefield);
        act(&mut g, Action::AnnounceCast { card: spell });
        let announced = g.snapshot();
        for kicked in [true, false] {
            g.restore(&announced).unwrap();
            act(&mut g, Action::ChooseKicker { pay: kicked });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if kicked {
                let invalid = g.object_ref(land);
                reject(&mut g, Action::ChooseAdditionalCost { object: invalid });
                let object = g.object_ref(sacrifice);
                act(&mut g, Action::ChooseAdditionalCost { object });
            }
            pay(&mut g);
            drain(&mut g, None);
            assert_eq!(stats(&g, target), if kicked { (7, 7) } else { (10, 10) });
            assert_eq!(
                g.state.objects[sacrifice as usize].zone,
                if kicked {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn farm_returns_one_linked_creature_then_the_other_to_hand() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let farm = put(&mut g, p, "Northampton Farm", Zone::Battlefield);
        let first = put(&mut g, p, "Healer's Hawk", Zone::Battlefield);
        let second = put(&mut g, p, "Ghitu Lavarunner", Zone::Battlefield);
        for target in [first, second] {
            g.state.objects[farm as usize].tapped = false;
            activate(&mut g, farm, 1, Some(target));
            drain(&mut g, None);
            assert_eq!(g.state.objects[target as usize].zone, Zone::Exile);
        }
        g.state.objects[farm as usize].tapped = false;
        activate(&mut g, farm, 2, None);
        for _ in 0..6 {
            if matches!(g.state.pending, Pending::LinkedExile { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        let choice = Action::ChooseLinkedExile {
            object: g.object_ref(first),
        };
        let saved = g.snapshot();
        let index = g.legal_actions().iter().position(|a| *a == choice).unwrap();
        let mut child = g.clone();
        child.apply_index(p, g.revision(), index).unwrap();
        act(&mut g, choice.clone());
        assert_eq!(g.snapshot(), child.snapshot());
        g.restore(&saved).unwrap();
        act(&mut g, choice);
        assert_eq!(g.snapshot(), child.snapshot());
        assert_eq!(g.state.objects[first as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[first as usize].controller, p);
        assert_eq!(g.state.objects[second as usize].zone, Zone::Hand);
    }
}

#[test]
fn don_and_leo_blink_both_independent_target_roles() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Don & Leo, Problem Solvers", Zone::Battlefield);
        let artifact = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
        let creature = put(&mut g, p, "Healer's Hawk", Zone::Battlefield);
        let before = [g.object_ref(artifact), g.object_ref(creature)];

        g.begin_step(Step::End);
        g.prepare_triggers();
        for target in [artifact, creature] {
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        }
        drain(&mut g, None);

        for object in before {
            assert_ne!(g.object_ref(object.card), object);
            assert_eq!(
                g.state.objects[object.card as usize].zone,
                Zone::Battlefield
            );
            assert_eq!(g.state.objects[object.card as usize].controller, p);
        }
    }
}

#[test]
fn don_and_leo_blink_one_artifact_creature_chosen_for_both_roles_once() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Don & Leo, Problem Solvers", Zone::Battlefield);
    put(&mut g, p, "Ninja Teen", Zone::Battlefield);
    let mouser = put(&mut g, p, "Mouser Mark III", Zone::Battlefield);
    let generation = g.object_ref(mouser).generation;
    let life = g.state.players[p.other().index()].life;

    g.begin_step(Step::End);
    g.prepare_triggers();
    for _ in 0..2 {
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mouser),
            },
        );
    }
    drain(&mut g, None);

    assert_eq!(g.state.objects[mouser as usize].zone, Zone::Battlefield);
    assert_eq!(g.object_ref(mouser).generation, generation + 2);
    assert_eq!(g.state.players[p.other().index()].life, life - 1);
}

#[test]
fn costed_mana_keeps_the_exact_pending_activation_source_untapped() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
    let other = put(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
    for _ in 0..20 {
        put(&mut g, p, "Forest", Zone::Battlefield);
    }
    g.state.players[p.index()].mana = [0; 6];
    act(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );

    let announce_mana = |card| Action::AnnounceMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        production: [1, 0, 0, 0, 0, 0],
    };
    let actions = g.legal_actions();
    assert!(actions.contains(&announce_mana(other)));
    assert!(!actions.contains(&announce_mana(source)));
    let before = g.snapshot();
    assert_eq!(g.apply(p, announce_mana(source)), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
}

#[test]
fn five_color_payment_does_not_count_the_same_flexible_source_twice() {
    let p = Player::P0;
    let mut g = scenario(p);
    let pizza = put(&mut g, p, "Everything Pizza", Zone::Battlefield);
    let captive = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    g.state.objects[captive as usize].tapped = true;
    let henchbots = put(&mut g, p, "Henchbots", Zone::Hand);
    cast(&mut g, henchbots, None);
    drain(&mut g, Some(captive));
    assert_eq!(g.state.objects[captive as usize].zone, Zone::Exile);

    for _ in 0..24 {
        put(&mut g, p, "Forest", Zone::Battlefield);
    }
    for _ in 0..2 {
        put(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
    }
    put(&mut g, p, "Weather Maker", Zone::Battlefield);
    g.state.players[p.index()].mana = [0; 6];

    let activate = Action::AnnounceActivate {
        card: pizza,
        ability: 0,
        counter_payment: Counters::default(),
    };
    let before = g.snapshot();
    assert!(!g.legal_actions().contains(&activate));
    assert_eq!(g.apply(p, activate.clone()), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);

    put(&mut g, p, "Weather Maker", Zone::Battlefield);
    assert!(g.legal_actions().contains(&activate));
}

#[test]
fn color_bound_leaves_room_for_mana_borrowed_after_sacrifice() {
    let p = Player::P0;
    let mut g = scenario(p);
    let oko = put(&mut g, p, "Oko, Lorwyn Liege", Zone::Battlefield);
    let druid = put(&mut g, p, "Druid of the Cowl", Zone::Battlefield);
    let thranduil = put(&mut g, p, "Thranduil, the Elvenking", Zone::Battlefield);
    let bolg = put(&mut g, p, "Bolg's Company", Zone::Battlefield);
    let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, p, "Tainted Treats", Zone::Hand);
    g.state.objects[oko as usize].counters.loyalty = 4;
    g.state.objects[druid as usize].tapped = true;

    activate(&mut g, oko, 0, Some(druid));
    drain(&mut g, None);
    assert!(g.has_subtype(druid, "Goblin"));
    g.state.players[p.index()].mana = [0; 6];

    act(&mut g, Action::AnnounceCast { card: spell });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    act(
        &mut g,
        Action::AnnounceMana {
            card: bolg,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 1, 1, 0, 0],
        },
    );
    let object = g.object_ref(druid);
    act(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    assert_eq!(g.state.objects[druid as usize].zone, Zone::Graveyard);

    let graveyard_druid = g.object_ref(druid);
    choose(&mut g, |action| {
        matches!(action, Action::ActivateGraveyardBorrowedMana {
        card, graveyard, production: [0, 0, 0, 0, 1, 0], ..
    } if *card == thranduil && *graveyard == graveyard_druid)
    });
    pay(&mut g);
    drain(&mut g, None);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
}

#[test]
fn damage_to_raphael_adds_red_mana_that_survives_the_step() {
    let p = Player::P0;
    let mut g = scenario(p);
    let raphael = put(&mut g, p, "Raphael, Ninja Destroyer", Zone::Battlefield);
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 0];

    cast(&mut g, shock, Some(Target::Card(raphael)));
    drain(&mut g, None);
    assert_eq!(g.state.players[p.index()].mana[3], 3);

    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::BeginCombat);
    assert_eq!(g.state.players[p.index()].mana[3], 2);
}

#[test]
fn damage_to_venus_puts_that_many_counters_on_her() {
    let p = Player::P0;
    let mut g = scenario(p);
    let venus = put(&mut g, p, "Venus, Torn Between Worlds", Zone::Battlefield);
    let shock = put(&mut g, p, "Shock", Zone::Hand);

    cast(&mut g, shock, Some(Target::Card(venus)));
    drain(&mut g, None);
    assert_eq!(g.state.objects[venus as usize].counters.plus_one, 2);
}

#[test]
fn courier_creates_food_only_when_the_search_puts_no_card_in_hand() {
    let p = Player::P0;
    let mut g = scenario(p);
    let courier = put(&mut g, p, "Courier of Comestibles", Zone::Hand);
    let pizza = put(&mut g, p, "Omni-Cheese Pizza", Zone::Library);
    cast(&mut g, courier, None);
    for _ in 0..4 {
        act(&mut g, Action::Pass);
    }
    let optional_search = g.snapshot();

    for search in [None, Some(false), Some(true)] {
        g.restore(&optional_search).unwrap();
        act(
            &mut g,
            Action::ChooseOptional {
                accept: search.is_some(),
            },
        );
        if search == Some(true) {
            act(&mut g, Action::ChooseSearch { card: pizza });
        }
        if search.is_some() {
            act(&mut g, Action::FinishSearch);
        }
        drain(&mut g, None);
        let found = search == Some(true);
        assert_eq!(g.state.objects[pizza as usize].zone == Zone::Hand, found);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|object| object.zone == Zone::Battlefield
                    && object.is_token()
                    && object.card.has_subtype("Food"))
                .count(),
            usize::from(!found)
        );
    }
}

#[test]
fn the_last_ronin_returns_a_creature_only_after_milling_four_cards() {
    let p = Player::P0;
    for cards in [3, 4] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "The Last Ronin", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.state.objects[source as usize].counters.lore = 1;
        while g.state.players[p.index()].library.len() > cards {
            let card = *g.state.players[p.index()].library.last().unwrap();
            g.move_object(card, Zone::Exile);
        }

        g.begin_step(Step::Main1);
        g.prepare_triggers();
        drain(&mut g, Some(creature));

        assert!(g.state.players[p.index()].library.is_empty());
        let expected_zone = if cards == 4 {
            Zone::Hand
        } else {
            Zone::Graveyard
        };
        assert_eq!(g.state.objects[creature as usize].zone, expected_zone);
    }
}

#[test]
fn mikey_and_don_cast_the_known_top_creature_with_its_entry_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Mikey & Don, Party Planners", Zone::Battlefield);
        let creature = put(&mut g, p, "Leonardo, Leader in Blue", Zone::Library);
        assert!(g.can_cast_library_top(p, creature));
        assert!(!g.can_cast_library_top(p.other(), creature));
        cast(&mut g, creature, None);
        drain(&mut g, None);
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[creature as usize].counters.plus_one, 1);
    }
}

#[test]
fn koya_returns_the_exact_exile_when_the_delayed_payment_is_declined() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let koya = put(&mut g, p, "Koya, Death from Above", Zone::Hand);
        let target = put(&mut g, p.other(), "Healer's Hawk", Zone::Battlefield);
        cast(&mut g, koya, None);
        drain(&mut g, Some(target));
        assert_eq!(g.state.objects[target as usize].zone, Zone::Exile);
        let exiled = g.object_ref(target);
        let check_link = |item: &SpellView| {
            assert_eq!(
                item.return_link.as_ref().map(|link| link.object),
                Some(exiled)
            );
        };
        let delay = g.observe(p).delayed_steps.expect("Koya scheduled a return");
        assert_eq!(delay.len(), 1);
        check_link(&delay[0].item);
        g.move_object(koya, Zone::Graveyard);
        g.begin_step(Step::End);
        check_link(
            g.state
                .pending_triggers
                .last()
                .expect("delayed ability queued"),
        );
        g.prepare_triggers();
        check_link(g.state.stack.last().expect("delayed ability stacked"));
        drain(&mut g, None);
        assert_ne!(g.object_ref(target), exiled);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[target as usize].controller, p.other());
    }
}

#[test]
fn library_counter_choice_preserves_the_independent_ordinary_permission() {
    for p in [Player::P0, Player::P1] {
        for with_counter in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Mikey & Don, Party Planners", Zone::Battlefield);
            put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
            let creature = put(&mut g, p, "Leonardo, Leader in Blue", Zone::Library);
            act(&mut g, Action::AnnounceCast { card: creature });
            let choices = g.legal_actions();
            assert!(choices.contains(&Action::ChooseLibraryCastPermission { with_counter: true }));
            assert!(choices.contains(&Action::ChooseLibraryCastPermission {
                with_counter: false
            }));
            act(&mut g, Action::ChooseLibraryCastPermission { with_counter });
            pay(&mut g);
            drain(&mut g, None);
            assert_eq!(
                g.state.objects[creature as usize].counters.plus_one,
                if with_counter { 1 } else { 0 }
            );
        }
    }
}
