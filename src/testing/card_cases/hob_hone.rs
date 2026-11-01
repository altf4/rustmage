//! Focused printed outcomes; shared engine invariants live in core tests.
//! Hone outcomes derive from CR122.1j and the pinned HOB release notes.
use super::*;

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
    {
        let p = Player::P0;
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
