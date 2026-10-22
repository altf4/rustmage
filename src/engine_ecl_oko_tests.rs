//! Paid loyalty choices on both faces, exact mill groups and independent emblems.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::ability_world::CharacteristicWorld;
use super::*;
const FRONT: &str = "Oko, Lorwyn Liege";
const BACK: &str = "Oko, Shadowmoor Scion";
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, FRONT, Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    cast(g, id, None, &[]);
    settle(g);
    assert_eq!(g.object_view(id).counters.loyalty, 3);
    id
}
fn flip(g: &mut Game, color: usize) {
    g.begin_step(Step::Main1);
    g.prepare_triggers();
    for _ in 0..40 {
        if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
            break;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
                .unwrap()
        };
        checked(g, a);
    }
    let mut payment = [0; 6];
    payment[color] = 1;
    g.state.players[g.state.active.index()].mana = payment;
    checked(g, Action::PayResolutionMana { payment });
    settle(g);
}
fn activate(g: &mut Game, id: CardId, ability: u16, target: Option<Target>) {
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,ability:i,target:t,..}if *card==id&&*i==ability&&*t==target)).unwrap();
    checked(g, a);
}
fn can_activate(g: &Game, id: CardId) -> bool {
    g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,..}|Action::AnnounceActivate{card,..}|Action::BeginActivate{card,..}if *card==id))
}
#[test]
fn ecl_oko_paid_front_permanent_types_and_loyalty_limit_survives_transforms() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        activate(&mut g, id, 0, Some(Target::Card(bear)));
        settle(&mut g);
        assert_eq!(g.object_view(id).counters.loyalty, 5);
        assert!(g.has_subtype(bear, "Goblin") && g.has_subtype(bear, "Elk"));
        assert!(!g.keywords(bear).contains(Keywords::CHANGELING));
        let reference = g.object_ref(id);
        flip(&mut g, 4);
        assert_eq!(g.state.objects[id as usize].card.name(), BACK);
        assert_eq!(g.object_ref(id), reference);
        assert_eq!(g.object_view(id).counters.loyalty, 5);
        assert!(!can_activate(&g, id));
        next_turn(&mut g);
        next_turn(&mut g);
        settle(&mut g);
        assert!(g.has_subtype(bear, "Goblin"));
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        let counters = Counters::of(CounterKind::Loyalty, 7);
        g.set_counters(id, counters);
        activate(&mut g, id, 2, None);
        // Change face after payment to exercise the back-only activation index2.
        g.transform_permanent(id);
        settle(&mut g);
        assert!(!can_activate(&g, id));
    }
}
#[test]
fn ecl_oko_front_growth_survives_cleanup_source_departure_and_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p);
        let bear = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        activate(&mut g, id, 1, Some(Target::Card(bear)));
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 1);
        assert_eq!(g.object_view(id).counters.loyalty, 4);
        g.move_object(id, Zone::Graveyard);
        g.state.objects[bear as usize].controller = p;
        next_turn(&mut g);
        assert_eq!(g.stats(bear).power, 1);
        next_turn(&mut g);
        assert_eq!(g.stats(bear).power, 3);
        assert!(g.observe(p).stat_modifier_effects.is_empty());
    }
}
#[test]
fn ecl_oko_back_mill_optional_permanent_selection_is_exact_to_the_event() {
    for p in [Player::P0, Player::P1] {
        for take in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p);
            flip(&mut g, 4);
            let old = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let land = named(&mut g, p, "Forest", Zone::Library);
            let bear = named(&mut g, p, "Bear Cub", Zone::Library);
            let instant = named(&mut g, p, "Shock", Zone::Library);
            g.state.players[p.index()].library = vec![land, bear, instant].into();
            activate(&mut g, id, 0, None);
            for _ in 0..10 {
                if g.legal_actions().contains(&Action::FinishMilledSelection) {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            assert!(!g.legal_actions().contains(&Action::SelectMilled {
                object: g.object_ref(old)
            }));
            assert!(!g.legal_actions().contains(&Action::SelectMilled {
                object: g.object_ref(instant)
            }));
            if take {
                let object = g.object_ref(land);
                checked(&mut g, Action::SelectMilled { object });
            }
            if g.legal_actions().contains(&Action::FinishMilledSelection) {
                checked(&mut g, Action::FinishMilledSelection);
            }
            settle(&mut g);
            assert_eq!(g.object_view(id).counters.loyalty, 2);
            assert_eq!(
                g.state.objects[land as usize].zone,
                if take { Zone::Hand } else { Zone::Graveyard }
            );
            assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
        }
    }
}
#[test]
fn ecl_oko_back_minus_three_creates_two_real_green_elk_after_oko_dies() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p);
        flip(&mut g, 4);
        activate(&mut g, id, 1, None);
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
            .map(|(i, _)| i as CardId)
            .collect();
        assert_eq!(tokens.len(), 2);
        for token in tokens {
            assert_eq!(
                g.stats(token),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
            assert_eq!(g.colors(token), 16);
            assert!(g.has_subtype(token, "Elk"));
        }
    }
}
#[test]
fn ecl_oko_emblem_chosen_type_is_owned_and_independent_after_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p);
        flip(&mut g, 4);
        g.set_counters(id, Counters::of(CounterKind::Loyalty, 6));
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        activate(&mut g, id, 2, None);
        for _ in 0..10 {
            if matches!(g.state.pending, Pending::ResolutionCreatureType { .. }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        let kind = crate::cards::CreatureType::from_name("Bear").unwrap();
        checked(
            &mut g,
            Action::ChooseCreatureType {
                creature_type: kind,
            },
        );
        settle(&mut g);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        assert_eq!(
            g.stats(enemy),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(
            g.keywords(bear)
                .contains(Keywords::VIGILANCE.union(Keywords::HEXPROOF))
        );
        assert!(!g.keywords(enemy).contains(Keywords::HEXPROOF));
        let later = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(g.stats(later).power, 5);
    }
}
#[test]
fn ecl_oko_optional_zero_targets_and_stale_target_do_not_refund_loyalty() {
    for p in [Player::P0, Player::P1] {
        for optional in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p);
            if optional {
                activate(&mut g, id, 0, None);
                settle(&mut g);
                assert_eq!(g.object_view(id).counters.loyalty, 5);
            } else {
                let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                activate(&mut g, id, 1, Some(Target::Card(bear)));
                g.move_object(bear, Zone::Hand);
                g.move_object(bear, Zone::Battlefield);
                settle(&mut g);
                assert_eq!(g.stats(bear).power, 2);
                assert!(g.observe(p).stat_modifier_effects.is_empty());
                assert_eq!(g.object_view(id).counters.loyalty, 4);
            }
            assert!(!can_activate(&g, id));
        }
    }
}
