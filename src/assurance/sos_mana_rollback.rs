//! Actual paid mana events retained selectively after a foreign choice fails a cast.
use super::*;
use big_helpers::{cast as paid, checked, settle as finish};
fn until_choice(g: &mut Game) {
    for _ in 0..40 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing suspended choice");
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}
fn setup(p: Player) -> (Game, CardId, CardId, CardId) {
    let mut g = scenario(p);
    let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
    g.state.objects[silver as usize].growth.power = -1;
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(enemy));
    let prison = put(&mut g, p, "Banishing Light", Zone::Hand);
    paid(&mut g, prison, None, &[]);
    until_choice(&mut g);
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    finish(&mut g);
    (g, silver, enemy, prison)
}
fn propose(g: &mut Game, spell: CardId, prison: CardId, enemy: CardId) {
    checked(g, Action::AnnounceCast { card: spell });
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
fn fail(g: &mut Game, silver: CardId) {
    pay(g);
    let additional = g
        .observe(g.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == CastCostClause::Additional)
        .unwrap() as u16;
    checked(g, Action::ChooseCastCost { clause: additional });
    let object = g.object_ref(silver);
    checked(g, Action::ChooseAuraAttachment { object });
    assert!(matches!(g.state.pending, Pending::ManaRollback { .. }));
}
#[test]
fn sos_mana_rollback_converter_cannot_keep_mana_without_its_paid_producer() {
    for p in [Player::P0, Player::P1] {
        for (keep_forest, keep_prism) in [(false, false), (true, false), (true, true)] {
            let (mut g, silver, enemy, prison) = setup(p);
            let forest = put(&mut g, p, "Forest", Zone::Battlefield);
            let prism = put(&mut g, p, "Prophetic Prism", Zone::Battlefield);
            let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            propose(&mut g, spell, prison, enemy);
            checked(&mut g, Action::TapMana { card: forest });
            checked(
                &mut g,
                Action::ActivateMana {
                    card: prism,
                    ability: 0,
                    payment: [0, 0, 0, 0, 1, 0],
                    counter_payment: Counters::default(),
                    production: [0, 0, 0, 1, 0, 0],
                },
            );
            fail(&mut g, silver);
            checked(&mut g, Action::ChooseManaRollback { keep: keep_forest });
            if !keep_forest {
                reject(&mut g, Action::ChooseManaRollback { keep: true });
            }
            checked(&mut g, Action::ChooseManaRollback { keep: keep_prism });
            assert_eq!(g.state.pending, Pending::Priority);
            assert_eq!(g.object_view(spell).zone, Zone::Hand);
            assert_eq!(g.object_view(forest).tapped, keep_forest);
            assert_eq!(g.object_view(prism).tapped, keep_prism);
            assert_eq!(
                g.state.players[p.index()].mana,
                [
                    0,
                    0,
                    1,
                    u32::from(keep_prism),
                    u32::from(keep_forest && !keep_prism),
                    0
                ]
            );
        }
    }
}
#[test]
fn sos_mana_rollback_kept_source_power_is_not_recomputed_after_other_reversals() {
    for p in [Player::P0, Player::P1] {
        let (mut g, silver, enemy, prison) = setup(p);
        let topiary = put(&mut g, p, "Topiary Lecturer", Zone::Battlefield);
        let arsenal = put(&mut g, p, "Machinist's Arsenal", Zone::Battlefield);
        let pizzas = [
            put(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield),
            put(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield),
        ];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: arsenal,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(topiary),
            },
        );
        pay(&mut g);
        finish(&mut g);
        g.state.objects[topiary as usize].growth.power = -3;
        assert_eq!(g.stats(topiary).power, 4);
        let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        propose(&mut g, spell, prison, enemy);
        checked(
            &mut g,
            Action::ActivateMana {
                card: pizzas[0],
                ability: 0,
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        assert_eq!(g.stats(topiary).power, 2);
        checked(
            &mut g,
            Action::ActivateMana {
                card: topiary,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 2, 0],
            },
        );
        checked(
            &mut g,
            Action::ActivateMana {
                card: pizzas[1],
                ability: 0,
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        assert_eq!(g.stats(topiary).power, 0);
        fail(&mut g, silver);
        for keep in [false, true, false] {
            checked(&mut g, Action::ChooseManaRollback { keep });
        }
        assert_eq!(g.state.pending, Pending::Priority);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        assert_eq!(g.stats(topiary).power, 4);
        assert!(g.object_view(topiary).tapped);
        assert!(
            pizzas
                .iter()
                .all(|&id| g.object_view(id).zone == Zone::Battlefield)
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 2, 2]);
    }
}
#[test]
fn sos_mana_rollback_irreversible_colossus_sacrifice_keeps_the_entire_mana_event() {
    for p in [Player::P0, Player::P1] {
        let (mut g, silver, enemy, prison) = setup(p);
        let colossus = put(&mut g, p, "Darksteel Colossus", Zone::Battlefield);
        let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: ring,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(colossus),
            },
        );
        pay(&mut g);
        finish(&mut g);
        let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        propose(&mut g, spell, prison, enemy);
        let grantor = g.object_ref(ring);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: colossus,
                grantor,
                production: [0, 0, 3, 0, 0, 0],
            },
        );
        let library = g.state.players[p.index()].library.clone();
        let rng = g.state.rng;
        fail(&mut g, silver);
        let view = g.observe(p).mana_rollback.unwrap();
        assert!(view.irreversible && view.can_keep && !view.can_reverse);
        reject(&mut g, Action::ChooseManaRollback { keep: false });
        checked(&mut g, Action::ChooseManaRollback { keep: true });
        assert_eq!(g.object_view(colossus).zone, Zone::Library);
        assert_eq!(g.state.players[p.index()].library, library);
        assert_eq!(g.state.rng, rng);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 3, 0, 0, 0]);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        assert!(g.state.cast_rollback.is_none());
    }
}

#[path = "hob_trigger_rollback.rs"]
mod hob_extra_triggers;

#[path = "hob_death_return_rollback.rs"]
mod hob_death_return_rollback;

#[path = "hob_crooked_rollback.rs"]
mod hob_crooked_rollback;

#[path = "hob_elrond_rollback.rs"]
mod hob_elrond_rollback;
