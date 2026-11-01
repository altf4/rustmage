//! Restore a genuinely private exact permission after a failed paid hidden cast.
use super::*;
#[test]
fn hob_flameshape_failed_private_paid_cast_restores_hidden_permission_and_actual_mana() {
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let (mut g, silver, enemy, prison) = setup(p);
            let card = put(&mut g, p, "Candy Grapple", Zone::Library);
            let adventure = put(
                &mut g,
                p,
                "Gandalf, Goblins' Bane // Flameshape",
                Zone::Hand,
            );
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
            checked(&mut g, Action::AnnounceAdventure { card: adventure });
            checked(&mut g, Action::ChooseCasualty { pay: false });
            pay(&mut g);
            finish(&mut g);
            let original = g.object_ref(card);
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != card));
            let wizard = put(
                &mut g,
                p,
                "Gandalf, Goblins' Bane // Flameshape",
                Zone::Battlefield,
            );
            g.state.objects[wizard as usize].growth.power = -2;
            let fire = put(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
            let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(fire)));
            g.state.objects[fire as usize].growth.power = -4;
            g.state.players[p.index()].mana = [0; 6];
            propose_exile(&mut g, card, prison, enemy);
            let grantor = g.object_ref(ring);
            checked(
                &mut g,
                Action::TapGrantedMana {
                    card: fire,
                    grantor,
                    production: [0, 0, 3, 0, 0, 0],
                },
            );
            fail(&mut g, silver);
            crate::engine::assurance::contracts::branch_contract(g.clone());
            checked(&mut g, Action::ChooseManaRollback { keep });
            assert_eq!(g.object_ref(card), original);
            assert_eq!(g.object_view(card).zone, Zone::Exile);
            assert!(g.can_play_exiled(p, card));
            assert!(g.observe(p).objects.iter().any(|o| o.id == card));
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != card));
            assert!(
                g.observe(p.other())
                    .exile_permissions
                    .iter()
                    .all(|r| r.object.card != card)
            );
            assert_eq!(
                g.state.players[p.index()].mana,
                if keep { [0, 0, 3, 0, 0, 0] } else { [0; 6] }
            );
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
            assert!(g.state.cast_rollback.is_none());
        }
    }
}
fn propose_exile(g: &mut Game, card: CardId, prison: CardId, enemy: CardId) {
    checked(g, Action::AnnounceExile { card });
    let sacrifice = Some(g.object_ref(prison));
    checked(g, Action::ChooseBargain { sacrifice });
    checked(g, Action::ChooseCasualty { pay: true });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
}
