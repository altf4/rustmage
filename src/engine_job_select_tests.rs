use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn resolve(g: &mut Game) {
    g.prepare_triggers();
    while !g.state.stack.is_empty() || !g.state.ordering_triggers.is_empty() {
        if let Some(a) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
        {
            act(g, a);
        } else {
            pass_pair(g);
        }
    }
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    g.state.active = p;
    g.state.priority = p;
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, n) = g.state.objects[id as usize].card.cost();
    mana[5] += n;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    pass_pair(g);
    resolve(g);
    id
}
#[test]
fn job_select_complete_basic_bodies_create_actual_heroes_with_live_types() {
    for p in [Player::P0, Player::P1] {
        for (name, subtype, power, toughness, keywords) in [
            ("Bard's Bow", "Bard", 3, 3, Keywords::REACH),
            ("Dark Knight's Greatsword", "Knight", 4, 1, Keywords::NONE),
            ("Dragoon's Lance", "Knight", 2, 1, Keywords::FLYING),
            ("Machinist's Arsenal", "Artificer", 3, 3, Keywords::NONE),
            ("Monk's Fist", "Monk", 2, 1, Keywords::NONE),
            (
                "Samurai's Katana",
                "Samurai",
                3,
                3,
                Keywords::TRAMPLE.union(Keywords::HASTE),
            ),
            ("Warrior's Sword", "Warrior", 4, 3, Keywords::NONE),
        ] {
            let mut g = scenario();
            let equipment = paid(&mut g, p, name);
            let hero = g.state.objects[equipment as usize].attached_to.unwrap();
            assert!(g.state.objects[hero.card as usize].card.is_token());
            assert!(g.state.objects[hero.card as usize].card.has_subtype("Hero"));
            assert!(
                !g.state.objects[hero.card as usize]
                    .card
                    .has_subtype(subtype)
            );
            assert!(g.object_view(hero.card).has_subtype(subtype));
            assert_eq!(g.stats(hero.card), Stats { power, toughness });
            assert!(g.keywords(hero.card).contains(keywords));
            let before = g.snapshot();
            let mut fork = g.clone();
            for viewer in [Player::P0, Player::P1] {
                let observed = g.observe(viewer);
                assert!(
                    observed
                        .objects
                        .iter()
                        .find(|o| o.id == hero.card)
                        .unwrap()
                        .has_subtype(subtype)
                );
            }
            g.move_object(equipment, Zone::Hand);
            assert!(!g.has_subtype(hero.card, subtype));
            assert_eq!(
                g.stats(hero.card),
                Stats {
                    power: 1,
                    toughness: 1
                }
            );
            g.restore(&before).unwrap();
            assert_eq!(g.snapshot(), fork.snapshot());
            fork.state.active = p.other();
            if name == "Dragoon's Lance" {
                assert!(!fork.keywords(hero.card).contains(Keywords::FLYING));
            }
        }
    }
}
#[test]
fn job_select_exact_source_departure_still_creates_but_does_not_reattach() {
    for reenter in [false, true] {
        let mut g = scenario();
        let equipment = named(&mut g, Player::P0, "Monk's Fist", Zone::Hand);
        g.state.players[0].mana[5] = 2;
        cast(&mut g, equipment, None);
        pass_pair(&mut g);
        let old = g.object_ref(equipment);
        g.move_object(equipment, Zone::Hand);
        if reenter {
            g.move_object(equipment, Zone::Battlefield);
        }
        resolve(&mut g);
        let heroes: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.zone == Zone::Battlefield && o.card.is_token() && o.card.has_subtype("Hero")
            })
            .collect();
        assert_eq!(heroes.len(), if reenter { 2 } else { 1 });
        let attached = g.object_view(equipment).attached_to;
        assert_eq!(
            heroes
                .iter()
                .filter(|(id, _)| Some(g.object_ref(*id as CardId)) != attached)
                .count(),
            1
        );
        assert_ne!(g.object_ref(equipment), old);
        if !reenter {
            assert_eq!(attached, None);
        }
        // Reentry has its own genuine Job Select trigger; the old trigger's Hero stays unattached.
        for (id, _) in heroes {
            if Some(g.object_ref(id as CardId)) != attached {
                assert_eq!(
                    g.stats(id as CardId),
                    Stats {
                        power: 1,
                        toughness: 1
                    }
                );
            }
        }
    }
}
#[test]
fn job_select_union_exact_links_and_captured_lki() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let sword = named(&mut g, Player::P0, "Warrior's Sword", Zone::Battlefield);
    let fist = named(&mut g, Player::P1, "Monk's Fist", Zone::Battlefield);
    assert!(g.attach_equipment(g.object_ref(sword), g.object_ref(bear)));
    assert!(g.attach_equipment(g.object_ref(fist), g.object_ref(bear)));
    let before = g.object_view(bear);
    assert!(
        before.has_subtype("Warrior") && before.has_subtype("Monk") && before.has_subtype("Bear")
    );
    let damage = DamageFacts::from_view(&before);
    assert!(damage.has_subtype("Monk"));
    g.move_object(bear, Zone::Hand);
    assert!(!g.object_view(bear).has_subtype("Warrior"));
    assert!(before.has_subtype("Warrior") && damage.has_subtype("Monk"));
    g.move_object(bear, Zone::Battlefield);
    assert!(!g.has_subtype(bear, "Monk"));
    assert!(!crate::SearchFilter::Subtype("Warrior").matches(g.state.objects[bear as usize].card));
}

#[test]
fn job_select_life_equip_is_targeted_once_each_turn_and_scaling_tracks_controller_artifacts() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let sword = paid(&mut g, p, "Dark Knight's Greatsword");
        let old = g.object_view(sword).attached_to.unwrap();
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..} if *card==sword&&*t==bear)).unwrap();
        let before = g.snapshot();
        let mut fork = g.clone();
        let life = g.state.players[p.index()].life;
        act(&mut g, action.clone());
        act(&mut fork, action.clone());
        assert_eq!(g.snapshot(), fork.snapshot());
        assert_eq!(g.state.players[p.index()].life, life - 3);
        assert_eq!(g.object_view(sword).attached_to, Some(old));
        resolve(&mut g);
        assert_eq!(g.object_view(sword).attached_to, Some(g.object_ref(bear)));
        assert!(g.has_subtype(bear, "Knight"));
        assert!(!g.has_subtype(old.card, "Knight"));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::Activate{card,..} if *card==sword))
        );
        g.restore(&before).unwrap();
        act(&mut g, action);
        resolve(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 3);
        let arsenal = paid(&mut g, p, "Machinist's Arsenal");
        let hero = g.object_view(arsenal).attached_to.unwrap();
        assert_eq!(
            g.stats(hero.card),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        let foe = named(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        assert_eq!(
            g.stats(hero.card),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        g.move_object(sword, Zone::Graveyard);
        assert_eq!(
            g.stats(hero.card),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        g.move_object(foe, Zone::Graveyard);
        assert_eq!(
            g.stats(hero.card),
            Stats {
                power: 3,
                toughness: 3
            }
        );
    }
}
