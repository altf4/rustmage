//! Actual foreign-entry cast failure retains or reverses Treasure provenance.
use super::*;
use crate::engine::assurance::big_helpers::{cast as paid, checked, settle};
#[test]
fn wicked_smaug_failed_cast_retains_or_reverses_treasure_mana() {
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let mut g = scenario(p);
            let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
            g.state.objects[silver as usize].growth.power = -1;
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(enemy));
            let prison = put(&mut g, p, "Banishing Light", Zone::Hand);
            paid(&mut g, prison, None, &[]);
            until(&mut g, |g| g.state.pending != Pending::Priority);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(aura),
                },
            );
            settle(&mut g);
            let smaug = put(&mut g, p, "Smaug, Wicked Worm", Zone::Battlefield);
            g.state.objects[smaug as usize].growth.power = -5;
            put(&mut g, p, "Forest", Zone::Library);
            let token = put(
                &mut g,
                p,
                "@token/b60f6cd3b44b22bd289fe4e2",
                Zone::Battlefield,
            );
            let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            checked(&mut g, Action::AnnounceCast { card: spell });
            let sacrifice = Some(g.object_ref(prison));
            checked(&mut g, Action::ChooseBargain { sacrifice });
            checked(&mut g, Action::ChooseCasualty { pay: true });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana {card,production:[0,0,0,1,0,0],..} if *card==token)).unwrap();
            checked(&mut g, action);
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, action);
            let additional = g
                .observe(g.actor().unwrap())
                .cast_costs
                .unwrap()
                .clauses
                .iter()
                .position(|r| r.clause == CastCostClause::Additional)
                .unwrap() as u16;
            checked(&mut g, Action::ChooseCastCost { clause: additional });
            let object = g.object_ref(silver);
            checked(&mut g, Action::ChooseAuraAttachment { object });
            assert!(matches!(g.state.pending, Pending::ManaRollback { .. }));
            contracts::branch_contract(g.clone());
            checked(&mut g, Action::ChooseManaRollback { keep });
            settle(&mut g);
            assert_eq!(
                g.object_view(token).zone,
                if keep {
                    Zone::Outside
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.observe(p)
                    .mana_lots
                    .iter()
                    .any(|lot| lot.player == p && lot.treasure_source),
                keep
            );
            if keep {
                let shock = put(&mut g, p, "Shock", Zone::Hand);
                act(&mut g, Action::AnnounceCast { card: shock });
                act(&mut g, Action::ChooseCasualty { pay: false });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                pay(&mut g);
                finish(&mut g);
                assert_eq!(g.state.players[p.index()].life, 19);
            }
        }
    }
}
