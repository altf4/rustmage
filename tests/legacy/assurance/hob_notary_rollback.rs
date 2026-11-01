//! Reuse the real foreign-Aura failure; keep or reverse an actual two-C Notary tap.
use super::*;
#[test]
fn hob_notary_retained_or_reversed_mana_restores_exact_tap_amount_and_owned_failure_root() {
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let (mut g, silver, enemy, prison) = setup(p);
            let source = put(&mut g, p, "The Notary Hobbits", Zone::Battlefield);
            let friend = put(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
            // Casualty1 must still become unpayable after the foreign Aura returns.
            // Power is irrelevant to this two-Halfling mana production.
            g.state.objects[source as usize].growth.power = -1;
            g.state.objects[friend as usize].growth.power = -1;
            let recipient = g.object_ref(friend);
            let timestamp = g.next_role_timestamp();
            g.state
                .roles
                .get_or_insert_with(Default::default)
                .creature_types
                .push(subtypes::CreatureTypesEffectView {
                    recipient,
                    source: recipient,
                    types: &["Halfling"],
                    additional: false,
                    timestamp,
                });
            let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            propose(&mut g, spell, prison, enemy);
            checked(
                &mut g,
                Action::ActivateMana {
                    card: source,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [0, 0, 0, 0, 0, 2],
                },
            );
            fail(&mut g, silver);
            crate::engine::assurance::contracts::branch_contract(g.clone());
            checked(&mut g, Action::ChooseManaRollback { keep });
            finish(&mut g);
            assert_eq!(g.object_view(source).tapped, keep);
            assert_eq!(
                g.state.players[p.index()].mana,
                [0, 0, 1, 0, 0, if keep { 2 } else { 0 }]
            );
            assert_eq!(g.object_view(spell).zone, Zone::Hand);
            assert!(g.state.cast_rollback.is_none());
        }
    }
}
