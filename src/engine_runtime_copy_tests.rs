//! Gated runtime recipes exercised through production effects and owned branches.
use super::ability_timeline_tests::{cast, checked, settle, setup};
use super::tests::spawn;
use super::*;

fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn copies(g: &Game, name: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.copy_token && o.card.name() == name)
        .map(|(i, _)| i as CardId)
        .collect()
}
fn reflect(g: &mut Game, p: Player, target: CardId) {
    let id = fixture(g, p, "Self-Reflection", Zone::Hand);
    cast(g, id, None, &[Target::Card(target)]);
    settle(g);
}
#[test]
fn runtime_copy_paid_values_are_copiable_and_token_identity_survives_owned_restore() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        assert!(g.state.copy_recipes.is_none());
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].growth = Stats {
            power: 7,
            toughness: 7,
        };
        g.state.objects[bear as usize].counters.plus_one = 3;
        g.state.objects[bear as usize].gained_keywords = Keywords::FLYING;
        reflect(&mut g, p, bear);
        let copy = copies(&g, "Bear Cub")[0];
        let view = g.object_view(copy);
        assert_eq!(view.name(), "Bear Cub");
        assert!(view.is_token());
        assert_eq!(
            view.copy_recipe,
            Some(CopyRecipe {
                definition: Card::from_name("Bear Cub").unwrap()
            })
        );
        assert_eq!((g.stats(copy).power, g.stats(copy).toughness), (2, 2));
        assert!(!g.keywords(copy).contains(Keywords::FLYING));
        assert_eq!(g.object_view(bear).copy_recipe, None);
        reflect(&mut g, p, copy);
        assert_eq!(copies(&g, "Bear Cub").len(), 2);
        let before = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.sacrifice_batch(&[(p, copy)]);
        child.give_priority();
        settle(&mut child);
        assert_eq!(child.object_view(copy).zone, Zone::Outside);
        assert!(child.copy_recipe(child.object_ref(copy)).is_none());
        assert_eq!(g.snapshot(), before);
        child.restore(&before).unwrap();
        assert_eq!(child.snapshot(), before);
        assert!(child.object_view(copy).is_token());
        checked(&mut child, Action::Pass);
    }
}
#[test]
fn runtime_copy_enters_with_fresh_printed_counters_not_source_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = fixture(&mut g, p, "Myojin of Night's Reach", Zone::Battlefield);
        // Creating a token is not casting the copied definition from hand.
        g.stage_copy_creation(
            p,
            CopyRecipe {
                definition: g.state.objects[source as usize].card,
            },
            1,
        );
        g.give_priority();
        settle(&mut g);
        let copy = copies(&g, "Myojin of Night's Reach");
        // Legend choices may retain either incarnation; no copied divinity counter exists.
        for id in copy {
            assert_eq!(g.counter_count(id, CounterKind::Divinity), 0);
        }
        assert!(g.state.objects.iter().all(|o| o.zone != Zone::Battlefield
            || o.card.name() != "Myojin of Night's Reach"
            || o.counters.plus_one == 0));
    }
}
