//! Paid inherited abilities keep the Forge's source identity and exact linked exile.
use super::assurance::big_helpers::spawn;
use super::assurance::big_helpers::{cast, checked, next_turn, settle, setup};
use super::*;
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
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
pub(super) fn finish(g: &mut Game) {
    for _ in 0..30 {
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
    panic!("unfinished announcement");
}
fn settle_target(g: &mut Game, target: CardId) {
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let choose = Action::ChooseTarget {
            target: Target::Card(target),
        };
        let action = if actions.contains(&choose) {
            choose
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
pub(super) fn forge(g: &mut Game, p: Player, target: CardId) -> CardId {
    let id = fixture(g, p, "Territory Forge", Zone::Hand);
    cast(g, id, None, &[]);
    settle_target(g, target);
    assert_eq!(g.object_view(target).zone, Zone::Exile);
    id
}
fn borrowed(g: &Game, id: CardId, index: u16) -> BorrowedAbilityView {
    g.borrowed_ability_views()
        .into_iter()
        .find(|a| a.source.card == id && a.ability == index)
        .unwrap()
}
fn announce(g: &mut Game, id: CardId, index: u16) {
    let b = borrowed(g, id, index);
    checked(
        g,
        Action::AnnounceBorrowedAbility {
            card: id,
            exiled: b.exiled,
            ability: index,
            counter_payment: Counters::default(),
        },
    );
}
#[test]
fn territory_forge_paid_intrinsic_land_mana_funding_exact_departure_and_abolisher() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = fixture(&mut g, p.other(), "Forest", Zone::Battlefield);
        let id = forge(&mut g, p, land);
        g.state.players[p.index()].mana = [0; 6];
        let b = borrowed(&g, id, u16::MAX - 5);
        assert_eq!(
            g.observe(p).borrowed_abilities,
            g.observe(p.other()).borrowed_abilities
        );
        assert!(g.can_fund(FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None
        }));
        let action = Action::AnnounceBorrowedMana {
            card: id,
            exiled: b.exiled,
            ability: u16::MAX - 5,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        };
        checked(&mut g, action.clone());
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        assert!(g.object_view(id).tapped);
        g.state.objects[id as usize].tapped = false;
        g.state.active = p.other();
        g.state.priority = p;
        let lock = fixture(&mut g, p.other(), "Grand Abolisher", Zone::Battlefield);
        reject(&mut g, action.clone());
        g.move_object(lock, Zone::Exile);
        g.move_object(land, Zone::Hand);
        reject(&mut g, action);
        assert!(g.borrowed_ability_views().is_empty());
    }
}
#[test]
fn territory_forge_inherited_sacrifice_draw_and_target_costs_use_actual_source_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let archive = fixture(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
        let id = forge(&mut g, p, archive);
        let count = g.hand_count(p);
        announce(&mut g, id, 1);
        let item = &g.state.announcing.as_ref().unwrap().item;
        assert_eq!(item.source_card.name(), "Hedron Archive");
        assert_eq!(item.source_facts.card.name(), "Territory Forge");
        assert_eq!(
            item.source_copy_recipe.as_ref().unwrap().definition.name(),
            "Territory Forge"
        );
        finish(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.hand_count(p), count + 2);
        assert_eq!(g.object_view(archive).zone, Zone::Exile);
        let bomb = fixture(&mut g, p.other(), "Goblin Firebomb", Zone::Battlefield);
        let second = forge(&mut g, p, bomb);
        let victim = fixture(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
        announce(&mut g, second, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(second).zone, Zone::Graveyard);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(g.object_view(bomb).zone, Zone::Exile);
    }
}
#[test]
fn territory_forge_acquired_counter_costs_do_not_inherit_static_or_triggered_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tome = fixture(&mut g, p.other(), "Mazemind Tome", Zone::Battlefield);
        let id = forge(&mut g, p, tome);
        let life = g.state.players[p.index()].life;
        for n in 1..=4 {
            g.state.objects[id as usize].tapped = false;
            announce(&mut g, id, 0);
            finish(&mut g);
            settle(&mut g);
            assert_eq!(g.counter_count(id, CounterKind::Page), n);
        }
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, life);
        assert_eq!(g.counter_count(tome, CounterKind::Page), 0);
    }
}
#[test]
fn territory_forge_once_per_turn_exhaust_and_new_incarnations_have_sparse_usage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let camera = fixture(&mut g, p.other(), "Camera Launcher", Zone::Battlefield);
        let id = forge(&mut g, p, camera);
        announce(&mut g, id, 0);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        let b = borrowed(&g, id, 0);
        let action = Action::AnnounceBorrowedAbility {
            card: id,
            exiled: b.exiled,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, action.clone());
        next_turn(&mut g);
        next_turn(&mut g);
        for _ in 0..10 {
            if g.state.step == Step::Main1 {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        reject(&mut g, action);
        g.state.players[p.index()].mana = [30; 6];
        g.move_object(id, Zone::Hand);
        g.move_object(camera, Zone::Battlefield);
        cast(&mut g, id, None, &[]);
        settle_target(&mut g, camera);
        announce(&mut g, id, 0);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        let mascot = fixture(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
        let second = forge(&mut g, p, mascot);
        let descriptor = borrowed(&g, second, 0);
        let mana = Action::AnnounceBorrowedMana {
            card: second,
            exiled: descriptor.exiled,
            ability: 0,
            counter_payment: Counters::default(),
            production: [1, 0, 0, 0, 0, 0],
        };
        checked(&mut g, mana.clone());
        finish(&mut g);
        reject(&mut g, mana);
    }
}
#[test]
fn territory_forge_no_cast_entry_and_source_departure_do_not_rebind_links() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let archive = fixture(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
        let id = fixture(&mut g, p, "Territory Forge", Zone::Hand);
        g.move_object(id, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(archive).zone, Zone::Battlefield);
        assert!(g.borrowed_ability_views().is_empty());
        g.move_object(id, Zone::Hand);
        cast(&mut g, id, None, &[]);
        for _ in 0..50 {
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(archive),
            }) {
                break;
            }
            let a = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                g.legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap()
            };
            checked(&mut g, a);
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(archive),
            },
        );
        g.move_object(id, Zone::Hand);
        settle_target(&mut g, archive);
        assert_eq!(g.object_view(archive).zone, Zone::Exile);
        assert!(g.borrowed_ability_views().is_empty());
    }
}
#[test]
fn territory_forge_more_than_sixty_four_acquired_abilities_never_alias_usage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = fixture(&mut g, p, "Territory Forge", Zone::Battlefield);
        for slot in 0..70 {
            let exiled = fixture(&mut g, p.other(), "Three Tree Mascot", Zone::Exile);
            let source = g.object_ref(id);
            let exiled = g.object_ref(exiled);
            g.state
                .borrowed_links
                .get_or_insert_default()
                .push(borrowed_activations::Link {
                    source,
                    origin: None,
                    exiled,
                    first_slot: slot * borrowed_activations::DONOR_SLOT_WIDTH,
                    width: borrowed_activations::DONOR_SLOT_WIDTH,
                });
        }
        let descriptors = g.borrowed_ability_views();
        assert_eq!(descriptors.len(), 70);
        assert_eq!(
            descriptors.iter().map(|a| a.slot).collect::<Vec<_>>(),
            (0..70).collect::<Vec<_>>()
        );
        let last = descriptors[69];
        checked(
            &mut g,
            Action::AnnounceBorrowedMana {
                card: id,
                exiled: last.exiled,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        finish(&mut g);
        let current = g.borrowed_ability_views();
        assert!(!current[0].used);
        assert!(current[69].used);
        assert_eq!(g.state.objects[id as usize].used_abilities, 0);
    }
}

#[test]
fn territory_forge_inherited_amulet_uses_actual_soul_counter_lki_after_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let amulet = fixture(&mut g, p.other(), "Ravenous Amulet", Zone::Battlefield);
        let id = forge(&mut g, p, amulet);
        let offering = fixture(&mut g, p, "Savannah Lions", Zone::Battlefield);
        announce(&mut g, id, 0);
        let offering = g.object_ref(offering);
        checked(&mut g, Action::ChooseAdditionalCost { object: offering });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Soul), 1);
        g.state.objects[id as usize].tapped = false;
        let life = g.state.players[p.other().index()].life;
        announce(&mut g, id, 1);
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .source_counter_lki
                .as_ref()
                .unwrap()[0]
                .count,
            1
        );
        finish(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 1);
    }
}

#[test]
fn territory_forge_goggles_mana_preserves_borrowed_rider_definition_and_actual_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let goggles = fixture(&mut g, p.other(), "Pyromancer's Goggles", Zone::Battlefield);
        let id = forge(&mut g, p, goggles);
        g.state.players[p.index()].mana = [0; 6];
        let b = borrowed(&g, id, 0);
        checked(
            &mut g,
            Action::AnnounceBorrowedMana {
                card: id,
                exiled: b.exiled,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        finish(&mut g);
        let source = g.mana_lots(p)[0].quality.copy_source.unwrap();
        assert_eq!(source.card.name(), "Territory Forge");
        assert_eq!(source.definition.name(), "Pyromancer's Goggles");
        let shock = fixture(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        let life = g.state.players[p.other().index()].life;
        for _ in 0..100 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            let actions = g.legal_actions();
            let action = if actions.contains(&Action::FinishStackTargets) {
                Action::FinishStackTargets
            } else if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                actions
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap()
            };
            checked(&mut g, action);
        }
        assert_eq!(g.state.players[p.other().index()].life, life - 4);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
}
#[test]
fn territory_forge_courtyard_mana_has_no_inherited_creature_type_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let court = fixture(&mut g, p.other(), "Secluded Courtyard", Zone::Battlefield);
        g.set_entry_value(
            g.object_ref(court),
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Elf").unwrap()),
        );
        let id = forge(&mut g, p, court);
        assert!(g.object_view(id).entry_choice.is_none());
        g.state.players[p.index()].mana = [0; 6];
        let b = borrowed(&g, id, 1);
        checked(
            &mut g,
            Action::AnnounceBorrowedMana {
                card: id,
                exiled: b.exiled,
                ability: 1,
                counter_payment: Counters::default(),
                production: [1, 0, 0, 0, 0, 0],
            },
        );
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
        assert_eq!(
            g.mana_lots(p)[0].quality.restriction,
            ManaRestrictionView::Unavailable
        );
        let lions = fixture(&mut g, p, "Savannah Lions", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: lions });
        assert!(!g.can_fund(FundingGoal {
            actor: p,
            required: [1, 0, 0, 0, 0, 0],
            generic: 0,
            reserve: None,
            source: None
        }));
    }
}

#[test]
fn territory_forge_inherited_saddle_does_not_inherit_mount_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mount = fixture(&mut g, p.other(), "Stubborn Burrowfiend", Zone::Battlefield);
        let moon = fixture(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        cast(&mut g, moon, None, &[Target::Card(mount)]);
        settle(&mut g);
        assert_eq!(g.current_kinds(mount), 2);
        let id = forge(&mut g, p, mount);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let saddler = g.object_ref(bear);
        announce(&mut g, id, 0);
        checked(&mut g, Action::ChooseSaddler { object: saddler });
        checked(&mut g, Action::FinishSaddle);
        settle(&mut g);
        assert!(g.is_saddled(g.object_ref(id)));
        assert!(g.on_battlefield(id));
        assert!(g.state.pending_triggers.is_empty());
        assert_eq!(g.state.objects[moon as usize].zone, Zone::Graveyard);
    }
}

#[test]
fn forge_borrows_lonely_mountain_printed_and_intrinsic_mana_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = fixture(&mut g, p.other(), "The Lonely Mountain", Zone::Battlefield);
        let source = forge(&mut g, p, land);
        let abilities = g.borrowed_ability_views();
        assert!(
            abilities
                .iter()
                .any(|a| a.source.card == source && a.ability == 0)
        );
        assert!(
            abilities
                .iter()
                .any(|a| a.source.card == source && a.ability == u16::MAX - 4)
        );
    }
}

#[test]
fn forge_borrows_both_intrinsic_mana_abilities_of_dual_land() {
    for p in [Player::P0, Player::P1] {
        for color in [1u16, 4] {
            let mut g = setup(p);
            let land = fixture(&mut g, p.other(), "Hedge Maze", Zone::Battlefield);
            let source = forge(&mut g, p, land);
            let abilities = g.borrowed_ability_views();
            assert!(
                abilities
                    .iter()
                    .any(|a| a.source.card == source && a.ability == u16::MAX - 2)
            );
            assert!(
                abilities
                    .iter()
                    .any(|a| a.source.card == source && a.ability == u16::MAX - 5)
            );
            assert!(
                !abilities
                    .iter()
                    .any(|a| a.source.card == source && a.ability == 0)
            );
            let descriptor = borrowed(&g, source, u16::MAX - 1 - color);
            g.state.players[p.index()].mana = [0; 6];
            let mut production = [0; 6];
            production[usize::from(color)] = 1;
            checked(
                &mut g,
                Action::AnnounceBorrowedMana {
                    card: source,
                    exiled: descriptor.exiled,
                    ability: u16::MAX - 1 - color,
                    counter_payment: Counters::default(),
                    production,
                },
            );
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, production);
        }
    }
}

#[test]
fn borrowed_land_subtype_abilities_follow_donor_copy_exception_with_stable_slots() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = fixture(&mut g, p.other(), "Hedge Maze", Zone::Battlefield);
        let source = forge(&mut g, p, land);
        let before = borrowed(&g, source, u16::MAX - 2);
        let green = borrowed(&g, source, u16::MAX - 5);
        let reference = g.object_ref(land);
        let mut recipe = CopyRecipe::new(Card::from_name("Hedge Maze").unwrap());
        recipe.overrides = Some(CopyOverrides {
            name: None,
            legendary: None,
            kinds: None,
            types: Some(CopyTypes {
                replace: true,
                ..CopyTypes::default()
            }),
            keywords: Keywords::NONE,
        });
        g.set_copy_recipe(reference, recipe);
        assert!(
            !g.borrowed_ability_views()
                .iter()
                .any(|ability| ability.source.card == source
                    && (ability.ability == u16::MAX - 2 || ability.ability == u16::MAX - 5))
        );
        g.clear_copy_recipe(reference);
        assert_eq!(borrowed(&g, source, u16::MAX - 2).slot, before.slot);
        assert_eq!(borrowed(&g, source, u16::MAX - 5).slot, green.slot);
    }
}

#[test]
fn borrowed_links_reserve_stable_slots_for_multiple_and_initially_ability_free_donors() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = fixture(&mut g, p, "Territory Forge", Zone::Battlefield);
        let first = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let second = fixture(&mut g, p.other(), "Forest", Zone::Battlefield);
        let source_ref = g.object_ref(source);
        let mut item = g.cast_preview(p, source, Zone::Battlefield, source_ref, 0, [0; 6]);
        for card in [first, second] {
            item.targets.push(BoundTarget {
                group: 0,
                target: Target::Card(card),
                object: Some(g.object_ref(card)),
            });
        }
        g.exile_and_borrow_activated_abilities(&item);
        let links = g.state.borrowed_links.as_ref().unwrap();
        assert_eq!(links.len(), 2);
        assert!(links[0].first_slot + links[0].width <= links[1].first_slot);
        assert_eq!(links[0].width, 1);
        assert_eq!(links[1].width, 1);
        let green = borrowed(&g, source, u16::MAX - 5);
        g.state.objects[first as usize].card = Card::from_name("Island").unwrap();
        let blue = borrowed(&g, source, u16::MAX - 2);
        assert_ne!(blue.slot, green.slot);
        assert_eq!(g.object_ref(first), blue.exiled);
        let blue_slot = blue.slot;
        g.state.borrowed_slots = borrowed_activations::refreshed_slots(&g);
        g.state.objects[first as usize].card = Card::from_name("Hedron Archive").unwrap();
        let added = g
            .borrowed_ability_views()
            .into_iter()
            .filter(|ability| ability.exiled.card == first)
            .collect::<Vec<_>>();
        assert!(added.iter().any(|ability| ability.ability == 0));
        assert!(added.iter().any(|ability| ability.ability == 1));
        assert!(
            added
                .iter()
                .all(|ability| ability.slot != green.slot && ability.slot != blue_slot)
        );
        let archive_slot = added
            .iter()
            .find(|ability| ability.ability == 0)
            .unwrap()
            .slot;
        g.state.borrowed_slots = borrowed_activations::refreshed_slots(&g);
        g.state.objects[first as usize].card = Card::from_name("Goblin Firebomb").unwrap();
        let changed = g
            .borrowed_ability_views()
            .into_iter()
            .find(|ability| ability.exiled.card == first && ability.ability == 0)
            .unwrap();
        assert_ne!(changed.slot, archive_slot);
        g.state.objects[first as usize].card = Card::from_name("Island").unwrap();
        assert_eq!(borrowed(&g, source, u16::MAX - 2).slot, blue_slot);
    }
}

#[test]
fn borrowed_donor_definition_changes_replay_and_restore_slots() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = fixture(&mut g, p, "Territory Forge", Zone::Battlefield);
        let donor = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let other = fixture(&mut g, p.other(), "Forest", Zone::Battlefield);
        let reference = g.object_ref(source);
        let mut item = g.cast_preview(p, source, Zone::Battlefield, reference, 0, [0; 6]);
        for card in [donor, other] {
            item.targets.push(BoundTarget {
                group: 0,
                target: Target::Card(card),
                object: Some(g.object_ref(card)),
            });
        }
        g.exile_and_borrow_activated_abilities(&item);
        let checkpoint = g.snapshot();
        let mut replay = g.clone();
        let exercise = |game: &mut Game| {
            let green = borrowed(game, source, u16::MAX - 5).slot;
            game.state.objects[donor as usize].card = Card::from_name("Hedron Archive").unwrap();
            let archive = game
                .borrowed_ability_views()
                .into_iter()
                .find(|a| a.exiled.card == donor && a.ability == 0)
                .unwrap()
                .slot;
            game.state.borrowed_slots = borrowed_activations::refreshed_slots(game);
            game.state.objects[donor as usize].card = Card::from_name("Goblin Firebomb").unwrap();
            let firebomb = game
                .borrowed_ability_views()
                .into_iter()
                .find(|a| a.exiled.card == donor && a.ability == 0)
                .unwrap()
                .slot;
            assert_ne!(archive, firebomb);
            assert_ne!(green, archive);
            assert_ne!(green, firebomb);
            game.state.borrowed_slots = borrowed_activations::refreshed_slots(game);
            game.state.objects[donor as usize].card = Card::from_name("Hedron Archive").unwrap();
            assert_eq!(
                game.borrowed_ability_views()
                    .into_iter()
                    .find(|a| a.exiled.card == donor && a.ability == 0)
                    .unwrap()
                    .slot,
                archive
            );
            (green, archive, firebomb)
        };
        let expected = exercise(&mut g);
        assert_eq!(exercise(&mut replay), expected);
        assert_eq!(g.snapshot(), replay.snapshot());
        g.restore(&checkpoint).unwrap();
        assert_eq!(g.snapshot(), checkpoint);
        assert_eq!(exercise(&mut g), expected);
        assert_eq!(g.snapshot(), replay.snapshot());
    }
}

#[test]
fn graveyard_donor_reacquisition_replays_without_aliasing_old_slot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = fixture(&mut g, p, "Thranduil, the Elvenking", Zone::Battlefield);
        let donor = fixture(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        g.refresh_graveyard_borrowing();
        g.state.borrowed_slots = borrowed_activations::refreshed_slots(&g);
        let original = g
            .graveyard_borrowed_ability_views()
            .unwrap()
            .into_iter()
            .find(|a| a.source.card == source && a.graveyard.card == donor)
            .unwrap()
            .slot;
        let checkpoint = g.snapshot();
        let mut replay = g.clone();
        let exercise = |game: &mut Game| {
            game.state.objects[donor as usize].card = Card::from_name("Bear Cub").unwrap();
            game.refresh_graveyard_borrowing();
            game.state.borrowed_slots = borrowed_activations::refreshed_slots(game);
            assert!(
                !game
                    .graveyard_borrowed_ability_views()
                    .unwrap_or_default()
                    .iter()
                    .any(|a| a.source.card == source && a.graveyard.card == donor)
            );
            game.state.objects[donor as usize].card = Card::from_name("Llanowar Elves").unwrap();
            game.refresh_graveyard_borrowing();
            game.state.borrowed_slots = borrowed_activations::refreshed_slots(game);
            let new_slot = game
                .graveyard_borrowed_ability_views()
                .unwrap()
                .into_iter()
                .find(|a| a.source.card == source && a.graveyard.card == donor)
                .unwrap()
                .slot;
            assert_ne!(new_slot, original);
            new_slot
        };
        let expected = exercise(&mut g);
        assert_eq!(exercise(&mut replay), expected);
        assert_eq!(g.snapshot(), replay.snapshot());
        g.restore(&checkpoint).unwrap();
        assert_eq!(g.snapshot(), checkpoint);
        assert_eq!(exercise(&mut g), expected);
        assert_eq!(g.snapshot(), replay.snapshot());
    }
}

#[test]
fn dual_land_mana_actions_pay_actual_blue_and_green_spells() {
    for player in [Player::P0, Player::P1] {
        for color in [1usize, 4] {
            let mut game = setup(player);
            let land = fixture(&mut game, player, "Hedge Maze", Zone::Battlefield);
            let creature = fixture(&mut game, player, "Bear Cub", Zone::Battlefield);
            let name = if color == 1 { "Opt" } else { "Giant Growth" };
            let spell = fixture(&mut game, player, name, Zone::Hand);
            game.state.players[player.index()].mana = [0; 6];
            let mut production = [0; 6];
            production[color] = 1;
            let action = if color == 1 {
                Action::TapMana { card: land }
            } else {
                Action::TapGrantedMana {
                    card: land,
                    grantor: game.object_ref(land),
                    production,
                }
            };
            checked(&mut game, action);
            assert_eq!(game.state.players[player.index()].mana, production);
            let targets = if color == 1 {
                vec![]
            } else {
                vec![Target::Card(creature)]
            };
            cast(&mut game, spell, None, &targets);
            assert_eq!(game.state.players[player.index()].mana, [0; 6]);
            assert_eq!(game.object_view(spell).zone, Zone::Stack);
        }
    }
}

#[test]
fn copied_dual_donor_removes_and_restores_borrowed_mana_for_actual_spending() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let donor = fixture(&mut game, player.other(), "Hedge Maze", Zone::Battlefield);
        let source = forge(&mut game, player, donor);
        let creature = fixture(&mut game, player, "Bear Cub", Zone::Battlefield);
        let spell = fixture(&mut game, player, "Giant Growth", Zone::Hand);
        let green = borrowed(&game, source, u16::MAX - 5);
        let action = Action::AnnounceBorrowedMana {
            card: source,
            exiled: green.exiled,
            ability: green.ability,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        };
        let reference = game.object_ref(donor);
        let mut recipe = CopyRecipe::new(Card::from_name("Hedge Maze").unwrap());
        recipe.overrides = Some(CopyOverrides {
            name: None,
            legendary: None,
            kinds: None,
            types: Some(CopyTypes {
                replace: true,
                ..CopyTypes::default()
            }),
            keywords: Keywords::NONE,
        });
        game.set_copy_recipe(reference, recipe);
        reject(&mut game, action.clone());
        game.clear_copy_recipe(reference);
        assert_eq!(borrowed(&game, source, u16::MAX - 5).slot, green.slot);
        game.state.players[player.index()].mana = [0; 6];
        checked(&mut game, action);
        finish(&mut game);
        assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 1, 0]);
        cast(&mut game, spell, None, &[Target::Card(creature)]);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        assert_eq!(game.object_view(spell).zone, Zone::Stack);
    }
}
