//! Hone outcomes derive from CR122.1j and the pinned HOB release notes.
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};

fn sword(g: &mut Game, p: Player, creature: CardId) -> CardId {
    let id = put(g, p, "Goldvein Pick", Zone::Battlefield);
    assert!(g.attach_equipment(g.object_ref(id), g.object_ref(creature)));
    id
}
fn attack(g: &mut Game, card: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card });
    act(g, Action::FinishAttackers);
    settle(g);
}

#[test]
fn dwalin_paid_hybrid_entry_and_attack_hone_each_own_equipment_including_unattached() {
    for p in [Player::P0, Player::P1] {
        for color in [0, 3] {
            let mut g = scenario(p);
            let bearer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let blade = sword(&mut g, p, bearer);
            let spare = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            let artifact = put(&mut g, p, "Prophetic Prism", Zone::Battlefield);
            let dwalin = put(&mut g, p, "Dwalin, Weaponmaster", Zone::Hand);
            let mut mana = [0; 6];
            mana[color] = 1;
            mana[5] = 1;
            g.state.players[p.index()].mana = mana;
            cast(&mut g, dwalin, None);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(stats(&g, dwalin), (2, 1));
            assert!(g.keywords(dwalin).contains(Keywords::FIRST_STRIKE));
            assert_eq!(g.counter_count(blade, CounterKind::Hone), 1);
            assert_eq!(g.counter_count(spare, CounterKind::Hone), 1);
            assert_eq!(g.counter_count(enemy, CounterKind::Hone), 0);
            assert_eq!(g.counter_count(artifact, CounterKind::Hone), 0);
            assert_eq!(stats(&g, bearer), (4, 3));
            g.state.objects[dwalin as usize].summoning_sick = false;
            attack(&mut g, dwalin);
            assert_eq!(g.counter_count(blade, CounterKind::Hone), 2);
            assert_eq!(g.counter_count(spare, CounterKind::Hone), 2);
            assert_eq!(stats(&g, bearer), (5, 3));
            g.finish_cleanup();
            assert_eq!(stats(&g, bearer), (5, 3));
        }
    }
}

#[test]
fn hone_is_counter_effect_even_when_equipment_and_recipient_lose_abilities() {
    let mut g = scenario(Player::P0);
    let bearer = put(&mut g, Player::P0, "Healer's Hawk", Zone::Battlefield);
    let blade = sword(&mut g, Player::P0, bearer);
    g.add_counter(blade, CounterKind::Hone, 3);
    assert_eq!(stats(&g, bearer), (5, 2));
    // Source-independent resolved removal supplies the generic layer-six root.
    g.blank_and_grant_haste(g.object_ref(blade), g.object_ref(bearer));
    g.blank_and_grant_haste(g.object_ref(bearer), g.object_ref(blade));
    assert!(!g.printed_abilities_available(blade));
    assert!(!g.keywords(bearer).contains(Keywords::FLYING));
    assert_eq!(stats(&g, bearer), (4, 1)); // Printed Goldvein Pick bonus is gone; hone remains.
    assert!(g.state.objects[blade as usize].attached_to.is_some());
}

#[test]
fn hone_tracks_attachment_exact_incarnations_removal_and_creature_equipment_until_sba() {
    let mut g = scenario(Player::P0);
    let a = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let b = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let blade = sword(&mut g, Player::P0, a);
    g.add_counter(blade, CounterKind::Hone, 2);
    assert_eq!(stats(&g, a), (5, 3));
    assert!(g.attach_equipment(g.object_ref(blade), g.object_ref(b)));
    assert_eq!(stats(&g, a), (2, 2));
    assert_eq!(stats(&g, b), (5, 3));
    g.pay_counters(blade, &Counters::of(CounterKind::Hone, 1));
    assert_eq!(stats(&g, b), (4, 3));
    g.state.objects[blade as usize].attached_to = None;
    assert_eq!(stats(&g, b), (2, 2));
    assert!(g.attach_equipment(g.object_ref(blade), g.object_ref(b)));
    let old_recipient = g.object_ref(b);
    g.move_object(b, Zone::Hand);
    g.move_object(b, Zone::Battlefield);
    // Even a stale restored link cannot bind the new recipient incarnation.
    g.state.objects[blade as usize].attached_to = Some(old_recipient);
    assert_eq!(stats(&g, b), (2, 2));
    g.state.objects[blade as usize].attached_to = None;
    assert!(g.attach_equipment(g.object_ref(blade), g.object_ref(a)));
    g.move_object(blade, Zone::Graveyard);
    g.move_object(blade, Zone::Battlefield);
    assert_eq!(g.counter_count(blade, CounterKind::Hone), 0);
    assert_eq!(stats(&g, a), (2, 2));
    let non_equipment = put(&mut g, Player::P0, "Prophetic Prism", Zone::Battlefield);
    g.add_counter(non_equipment, CounterKind::Hone, 7);
    g.state.objects[non_equipment as usize].attached_to = Some(g.object_ref(a));
    assert_eq!(stats(&g, a), (2, 2));
    // Fixture animation creates the pre-SBA boundary; CR122.1j has no noncreature exclusion.
    g.add_counter(blade, CounterKind::Hone, 3);
    assert!(g.attach_equipment(g.object_ref(blade), g.object_ref(a)));
    g.change_object_form(
        g.object_ref(blade),
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((1, 1)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
    assert!(g.is_creature(blade));
    assert_eq!(stats(&g, a), (5, 2)); // Printed attachment fast path stops; counter still supplies +3/+0.
    g.state_based_actions();
    assert_eq!(g.state.objects[blade as usize].attached_to, None);
    assert_eq!(stats(&g, a), (2, 2));
}

#[test]
fn hone_uses_counter_replacements_and_owned_sparse_public_snapshots() {
    let mut g = scenario(Player::P0);
    let bearer = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let blade = sword(&mut g, Player::P0, bearer);
    put(&mut g, Player::P0, "Doubling Season", Zone::Battlefield);
    let dwalin = put(&mut g, Player::P0, "Dwalin, Weaponmaster", Zone::Hand);
    cast(&mut g, dwalin, None);
    // Existing owned contract traverses the pending stack using raw/index/restore.
    contracts::branch_contract(g.clone());
    settle(&mut g);
    assert_eq!(g.counter_count(blade, CounterKind::Hone), 2);
    assert_eq!(stats(&g, bearer), (5, 3));
    let saved = g.snapshot();
    let mut child = {
        let parent = g.clone();
        parent.clone()
    };
    child.pay_counters(blade, &Counters::of(CounterKind::Hone, 1));
    assert_eq!(child.counter_count(blade, CounterKind::Hone), 1);
    assert_eq!(g.counter_count(blade, CounterKind::Hone), 2);
    assert_eq!(stats(&child, bearer), (4, 3));
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            g.observe(viewer)
                .objects
                .iter()
                .find(|o| o.id == blade)
                .unwrap()
                .counters
                .get(CounterKind::Hone),
            2
        );
    }
    let encoded = serde_json::to_value(Counters::of(CounterKind::Hone, 2)).unwrap();
    assert_eq!(encoded["named"], serde_json::json!([["Hone", 2]]));
    let decoded: Counters = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(decoded.get(CounterKind::Hone), 2);
    for named in [
        serde_json::json!([["Hone", 0]]),
        serde_json::json!([["Hone", 1], ["Hone", 2]]),
    ] {
        let mut malformed = encoded.clone();
        malformed["named"] = named;
        assert!(serde_json::from_value::<Counters>(malformed).is_err());
    }
    g.pay_counters(blade, &Counters::of(CounterKind::Hone, 2));
    assert_eq!(stats(&g, bearer), (3, 3));
    g.restore(&saved).unwrap();
    assert_eq!(g.counter_count(blade, CounterKind::Hone), 2);
    assert_eq!(stats(&g, bearer), (5, 3));
}
