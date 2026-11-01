//! Existing foreign-entry failure proves the free origin survives a failed cast.
use super::*;
#[test]
fn hob_decree_failed_free_cast_restores_exact_permission_and_retained_or_reversed_mana() {
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let (mut g, silver, enemy, prison) = setup(p);
            let spell = put(&mut g, p.other(), "Candy Grapple", Zone::Exile);
            let original = g.object_ref(spell);
            g.grant_exile_free_permission(p, original);
            let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
            let fire = put(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(fire)));
            g.state.objects[fire as usize].growth.power = -4;
            g.state.players[p.index()].mana = [0; 6];
            checked(
                &mut g,
                Action::AnnounceExileFree {
                    card: spell,
                    face: CastFace::Normal,
                    flash_surcharge: false,
                },
            );
            let sacrifice = Some(g.object_ref(prison));
            checked(&mut g, Action::ChooseBargain { sacrifice });
            checked(&mut g, Action::ChooseCasualty { pay: true });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
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
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
            assert_eq!(g.object_ref(spell), original);
            assert!(g.has_exile_free_permission(p, spell));
            assert!(!g.has_exile_free_permission(p.other(), spell));
            assert_eq!(
                g.object_view(fire).zone,
                if keep {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.state.players[p.index()].mana,
                if keep { [0, 0, 3, 0, 0, 0] } else { [0; 6] }
            );
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
            assert!(g.state.cast_rollback.is_none());
        }
    }
}
