//! Oracle-derived outcomes for the initial HOB equipment and Adventure cohort.
use super::*;

fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    act(
        g,
        Action::Cast {
            card: id,
            target: None,
            payment: mana,
        },
    );
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn top(g: &mut Game) {
    act(g, Action::Pass);
    act(g, Action::Pass);
}
fn token(g: &Game, p: Player, name: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card.name() == name
        })
        .map(|(i, _)| i as CardId)
        .collect()
}
fn equip(g: &mut Game, p: Player, e: CardId, c: CardId, payment: Mana) {
    g.state.players[p.index()].mana = payment;
    act(
        g,
        Action::Activate {
            card: e,
            ability: 0,
            target: Some(Target::Card(c)),
            payment,
            counter_payment: Counters::default(),
        },
    );
    settle(g);
}
#[test]
fn blacksmith_creates_a_real_equippable_axe_and_has_double_strike() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let smith = paid(&mut g, p, "Iron Hills Blacksmith", [1, 0, 0, 0, 0, 1]);
        settle(&mut g);
        assert!(g.keywords(smith).contains(Keywords::DOUBLE_STRIKE));
        assert_eq!(stats(&g, smith), (1, 1));
        let axes = token(&g, p, "Axe");
        assert_eq!(axes.len(), 1);
        let axe = axes[0];
        let v = g.object_view(axe);
        assert!(v.has_subtype("Equipment"));
        assert_eq!(v.card.colors(), 0);
        assert_eq!(v.attached_to, None);
        equip(&mut g, p, axe, smith, [0, 0, 0, 0, 0, 2]);
        assert_eq!(stats(&g, smith), (2, 1));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn shortsword_creates_attaches_and_can_move_to_another_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let sword = paid(&mut g, p, "Dwarven Shortsword", [1, 0, 0, 0, 0, 3]);
        settle(&mut g);
        let dwarves = token(&g, p, "Dwarf Token");
        assert_eq!(dwarves.len(), 1);
        let dwarf = dwarves[0];
        assert_eq!(stats(&g, dwarf), (3, 4));
        assert!(g.object_view(dwarf).has_subtype("Dwarf"));
        assert_eq!(g.object_view(dwarf).card.colors(), 8);
        assert_eq!(g.object_view(sword).attached_to, Some(g.object_ref(dwarf)));
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        equip(&mut g, p, sword, bear, [0, 0, 0, 0, 0, 2]);
        assert_eq!(stats(&g, dwarf), (2, 2));
        assert_eq!(stats(&g, bear), (3, 4));
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        reject(
            &mut g,
            Action::Activate {
                card: sword,
                ability: 0,
                target: Some(Target::Card(enemy)),
                payment: [0, 0, 0, 0, 0, 2],
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn shortsword_trigger_creates_token_after_source_departure() {
    let p = Player::P0;
    let mut g = scenario(p);
    let sword = paid(&mut g, p, "Dwarven Shortsword", [1, 0, 0, 0, 0, 3]);
    top(&mut g);
    assert!(!g.state.stack.is_empty());
    g.move_object(sword, Zone::Graveyard);
    g.give_priority();
    settle(&mut g);
    let dwarves = token(&g, p, "Dwarf Token");
    assert_eq!(dwarves.len(), 1);
    assert_eq!(stats(&g, dwarves[0]), (2, 2));
    assert_eq!(g.object_view(sword).zone, Zone::Graveyard);
}
#[test]
fn stalwart_attaches_only_controlled_equipment_to_optional_controlled_recipient() {
    for p in [Player::P0, Player::P1] {
        for attach in [false, true] {
            let mut g = scenario(p);
            let e = put(&mut g, p, "Lavaspur Boots", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Lavaspur Boots", Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let c = paid(&mut g, p, "Iron Hills Stalwart", [0, 0, 0, 1, 0, 4]);
            top(&mut g);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(e),
                },
            );
            if attach {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(bear),
                    },
                );
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            settle(&mut g);
            assert!(
                g.keywords(c)
                    .contains(Keywords::REACH.union(Keywords::TRAMPLE))
            );
            assert_eq!(stats(&g, c), (4, 5));
            assert_eq!(
                g.object_view(e).attached_to,
                if attach {
                    Some(g.object_ref(bear))
                } else {
                    None
                }
            );
            assert_eq!(stats(&g, bear), if attach { (3, 2) } else { (2, 2) });
        }
    }
}
#[test]
fn unexpected_party_adventure_x_and_chosen_type_bonus() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let c = put(&mut g, p, "An Unexpected Party // At the Door", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 4];
        act(&mut g, Action::AnnounceAdventure { card: c });
        act(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        settle(&mut g);
        let dwarves = token(&g, p, "Dwarf Token");
        assert_eq!(dwarves.len(), 2);
        assert_eq!(g.object_view(c).zone, Zone::Exile);
        for &d in &dwarves {
            assert_eq!(stats(&g, d), (2, 2));
        }
        let enemy = put(
            &mut g,
            p.other(),
            "Iron Hills Blacksmith",
            Zone::Battlefield,
        );
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceExile { card: c });
        pay(&mut g);
        top(&mut g);
        let choice=g.legal_actions().into_iter().find(|a|matches!(a,Action::ChooseEntryValue {value:EntryChoiceValue::CreatureType(t)} if t.name()=="Dwarf")).unwrap();
        contracts::branch_contract(g.clone());
        act(&mut g, choice);
        settle(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Battlefield);
        for &d in &dwarves {
            assert_eq!(stats(&g, d), (4, 4));
        }
        assert_eq!(stats(&g, enemy), (1, 1));
        assert_eq!(stats(&g, bear), (2, 2));
        g.move_object(c, Zone::Graveyard);
        for &d in &dwarves {
            assert_eq!(stats(&g, d), (2, 2));
        }
    }
}
#[test]
fn my_precious_adventure_sacrifice_draw_and_paid_equip() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let c = put(&mut g, p, "My Precious // Allure of Power", Zone::Hand);
        let fodder = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let hand = g.hand_count(p);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
        act(&mut g, Action::AnnounceAdventure { card: c });
        let enemy_ref = g.object_ref(enemy);
        reject(&mut g, Action::ChooseAdditionalCost { object: enemy_ref });
        let r = g.object_ref(fodder);
        act(&mut g, Action::ChooseAdditionalCost { object: r });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
        assert_eq!(g.object_view(c).zone, Zone::Exile);
        assert_eq!(g.hand_count(p), hand + 1);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        act(&mut g, Action::AnnounceExile { card: c });
        pay(&mut g);
        settle(&mut g);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        equip(&mut g, p, c, bear, [0, 0, 0, 0, 0, 2]);
        assert_eq!(g.state.players[p.index()].life, life - 2);
        assert!(
            g.keywords(bear)
                .contains(Keywords::HEXPROOF.union(Keywords::CANT_BE_BLOCKED))
        );
        assert_eq!(stats(&g, bear), (2, 2));
        assert!(!g.can_block_pair(enemy, bear));
        let blank = put(&mut g, p, "Witness Protection", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        act(
            &mut g,
            Action::Cast {
                card: blank,
                target: Some(Target::Card(bear)),
                payment: [0, 1, 0, 0, 0, 0],
            },
        );
        settle(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF));
        assert!(g.keywords(bear).contains(Keywords::CANT_BE_BLOCKED));
        assert!(!g.can_block_pair(enemy, bear));
        g.move_object(blank, Zone::Graveyard);
        assert!(g.keywords(bear).contains(Keywords::HEXPROOF));
        g.move_object(c, Zone::Graveyard);
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF));
        assert!(!g.keywords(bear).contains(Keywords::CANT_BE_BLOCKED));
    }
}
