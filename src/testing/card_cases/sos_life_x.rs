//! Focused printed outcomes; shared engine invariants live in core tests.
//! Additional life X keeps printed mana fixed and commits only during cost payment.
use super::big_helpers::{checked, settle as finish};
use super::*;
fn choose_life(g: &mut Game, spell: CardId, x: u32, free: Option<Action>) {
    checked(g, free.unwrap_or(Action::AnnounceCast { card: spell }));
    let life = g.state.players[g.actor().unwrap().index()].life;
    checked(g, Action::ChooseX { value: x });
    assert_eq!(g.state.players[g.actor().unwrap().index()].life, life);
}
fn pay_life(g: &mut Game, spell: CardId, x: u32) {
    let p = g.actor().unwrap();
    let life = g.state.players[p.index()].life;
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, pay);
    assert_eq!(g.state.players[p.index()].life, life);
    assert!(
        !g.state
            .stack
            .iter()
            .any(|s| s.source.is_some_and(|source| source.id == spell))
    );
    assert!(g.observe(p.other()).cast_costs.is_none());
    let index = g
        .observe(p)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == CastCostClause::Life(x))
        .unwrap() as u16;
    checked(g, Action::ChooseCastCost { clause: index });
    assert_eq!(g.state.players[p.index()].life, life - i128::from(x));
    if g.outcome().is_none() {
        let item = g
            .state
            .stack
            .iter()
            .find(|s| s.source.is_some_and(|source| source.id == spell))
            .unwrap();
        assert_eq!(item.x, Some(x));
        assert_eq!(item.mana_value(), 4);
        assert_eq!(item.source.unwrap().card.x_symbols(), 0);
    }
}
#[test]
fn sos_life_x_fix_returns_each_matching_artifact_or_creature_and_respects_exact_x() {
    {
        let p = Player::P0;
        for x in [0, 2, 3] {
            let mut g = scenario(p);
            let zero = put(&mut g, p, "Mox Jasper", Zone::Graveyard);
            let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let pick = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
            let a = put(&mut g, p, "Adaptive Automaton", Zone::Graveyard);
            let b = put(&mut g, p, "Adaptive Automaton", Zone::Graveyard);
            let enchantment = put(&mut g, p, "Pacifism", Zone::Graveyard);
            let land = put(&mut g, p, "Forest", Zone::Graveyard);
            let other = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let spell = put(&mut g, p, "Fix What's Broken", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            assert_eq!(g.x_choice().unwrap().intervals[0].maximum, 20);
            reject(&mut g, Action::ChooseX { value: 21 });
            checked(&mut g, Action::ChooseX { value: x });
            pay_life(&mut g, spell, x);
            finish(&mut g);
            for (id, mana) in [(zero, 0), (bear, 2), (pick, 2), (a, 3), (b, 3)] {
                assert_eq!(
                    g.object_view(id).zone,
                    if x == mana {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
                if x == mana {
                    assert_eq!(g.object_view(id).controller, p);
                }
            }
            for id in [enchantment, land, other] {
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn sos_life_x_vicious_destroys_all_both_sides_up_to_x_without_targeting_or_indestructible() {
    {
        let p = Player::P0;
        for x in [0, 2, 11] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let artifact = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            let token = put(&mut g, p.other(), "@token/sos-pest", Zone::Battlefield);
            let indestructible = put(&mut g, p.other(), "Darksteel Colossus", Zone::Battlefield);
            let enchantment = put(&mut g, p.other(), "Omniscience", Zone::Battlefield);
            let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let spell = put(&mut g, p, "Vicious Rivalry", Zone::Hand);
            choose_life(&mut g, spell, x, None);
            pay_life(&mut g, spell, x);
            finish(&mut g);
            for id in [own, other, artifact] {
                assert_eq!(
                    g.object_view(id).zone,
                    if x >= 2 {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
            assert_eq!(g.object_view(token).zone, Zone::Outside);
            for id in [indestructible, enchantment, land] {
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            }
        }
    }
}
#[test]
fn sos_life_x_free_mana_cost_still_allows_positive_additional_life_x() {
    {
        let p = Player::P0;
        for name in ["Fix What's Broken", "Vicious Rivalry"] {
            for zaffai in [false, true] {
                let mut g = scenario(p);
                let source = put(
                    &mut g,
                    p,
                    if zaffai {
                        "Zaffai and the Tempests"
                    } else {
                        "Omniscience"
                    },
                    Zone::Battlefield,
                );
                let spell = put(&mut g, p, name, Zone::Hand);
                g.state.players[p.index()].mana = [0; 6];
                let a = if zaffai {
                    Action::AnnounceFreeFromPermission {
                        card: spell,
                        face: CastFace::Normal,
                        flash_surcharge: false,
                        permission: g.object_ref(source),
                    }
                } else {
                    Action::AnnounceFree {
                        card: spell,
                        face: CastFace::Normal,
                        flash_surcharge: false,
                    }
                };
                choose_life(&mut g, spell, 3, Some(a));
                pay_life(&mut g, spell, 3);
                assert_eq!(
                    g.state
                        .stack
                        .iter()
                        .find(|s| s.source.is_some_and(|source| source.id == spell))
                        .unwrap()
                        .cast
                        .unwrap()
                        .payment,
                    [0; 6]
                );
                finish(&mut g);
                assert_eq!(g.state.players[p.index()].life, 17);
            }
        }
    }
}
#[test]
fn sos_life_x_domains_handle_u32_max_and_zero_at_negative_life_without_scanning_x() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        g.state.players[p.index()].life = i128::from(u32::MAX) + 3;
        let spell = put(&mut g, p, "Fix What's Broken", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(
            g.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: u32::MAX
            }]
        );
        checked(&mut g, Action::ChooseX { value: u32::MAX });
        pay_life(&mut g, spell, u32::MAX);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 3);
        let mut g = scenario(p);
        put(&mut g, p, "Herald of Eternal Dawn", Zone::Battlefield);
        g.state.players[p.index()].life = -3;
        let spell = put(&mut g, p, "Fix What's Broken", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(
            g.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: 0
            }]
        );
        reject(&mut g, Action::ChooseX { value: 1 });
        checked(&mut g, Action::ChooseX { value: 0 });
        pay_life(&mut g, spell, 0);
        finish(&mut g);
        assert!(g.outcome().is_none());
        assert_eq!(g.state.players[p.index()].life, -3);
    }
}
