//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
use crate::engine::testing::card_cases::big_helpers::spawn;
use crate::engine::testing::card_cases::big_helpers::{cast, checked, next_turn, settle, setup};
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn reject(g: &mut Game, action: Action) {
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
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
fn territory_forge_inherited_sacrifice_draw_and_target_costs_use_actual_source_lki() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let archive = fixture(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
        let id = forge(&mut g, p, archive);
        let count = g.hand_count(p);
        announce(&mut g, id, 1);
        let item = &g.state.announcing.as_ref().unwrap().item;
        assert_eq!(item.source.unwrap().card.name(), "Hedron Archive");
        assert_eq!(item.source.unwrap().facts.card.name(), "Territory Forge");
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn territory_forge_inherited_amulet_uses_actual_soul_counter_lki_after_sacrifice() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
