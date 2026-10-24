//! Paid foreign-card permission, exact replacement and ownership-trigger outcomes.
use super::big_helpers::{cast as paid, checked, next_turn, settle as finish};
use super::*;
fn activate(g: &mut Game, source: CardId, target: CardId, fodder: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    let object = g.object_ref(source);
    reject(g, Action::ChooseAdditionalCost { object });
    let object = g.object_ref(fodder);
    checked(g, Action::ChooseAdditionalCost { object });
    pay(g);
    finish(g);
    assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
    assert_eq!(g.object_view(target).zone, Zone::Exile);
}
#[test]
fn sos_nita_paid_permission_uses_any_mana_is_independent_and_cast_trigger_freezes_ownership() {
    for p in [Player::P0, Player::P1] {
        for leave in 0..6 {
            let early = leave >= 3;
            let leave = leave % 3;
            let mut g = scenario(p);
            let source = put(&mut g, p, "Nita, Forum Conciliator", Zone::Hand);
            paid(&mut g, source, None, &[]);
            finish(&mut g);
            assert_eq!(stats(&g, source), (2, 3));
            let fodder = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let survivor = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let early_own = early.then(|| put(&mut g, p, "Shock", Zone::Hand));
            let spell = put(&mut g, p.other(), "Shock", Zone::Graveyard);
            activate(&mut g, source, spell, fodder);
            let permission = g
                .observe(p)
                .exile_permissions
                .into_iter()
                .find(|row| row.object == g.object_ref(spell))
                .unwrap();
            assert_eq!(permission.player, p);
            assert!(permission.spend_any_mana && permission.exile_on_graveyard);
            let own = early_own.unwrap_or_else(|| put(&mut g, p, "Shock", Zone::Hand));
            if leave == 1 {
                g.move_object(source, Zone::Exile);
            }
            // Normal costs still apply, with a genuine off-color payment permission.
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
            reject(&mut g, Action::AnnounceCast { card: own });
            checked(&mut g, Action::AnnounceExile { card: spell });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
            let cast = g
                .observe(p)
                .stack
                .iter()
                .find(|s| s.card == spell)
                .unwrap()
                .cast
                .unwrap();
            assert_eq!(cast.payment, [0, 0, 0, 0, 1, 0]);
            assert!(cast.spend_any_mana && cast.exile_on_graveyard);
            if leave == 2 {
                g.move_object(source, Zone::Exile);
            }
            finish(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
            assert!(!g.can_play_exiled(p, spell));
            assert_eq!(
                g.counter_count(survivor, CounterKind::PlusOne),
                if leave == 1 { 0 } else { 1 }
            );
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
            if leave == 0 {
                assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
            }
            g.state.players[p.index()].mana[3] = 1;
            paid(&mut g, own, None, &[Target::Player(p.other())]);
            finish(&mut g);
            assert_eq!(
                g.counter_count(survivor, CounterKind::PlusOne),
                if leave == 1 { 0 } else { 1 },
                "owned spells do not satisfy the trigger"
            );
        }
    }
}
#[test]
fn sos_nita_activation_legality_deadline_and_target_reincarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Nita, Forum Conciliator", Zone::Battlefield);
        let spell = put(&mut g, p.other(), "Shock", Zone::Graveyard);
        let announce = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, announce.clone());
        let fodder = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::BeginCombat;
        reject(&mut g, announce.clone());
        g.state.step = Step::Main1;
        let own = put(&mut g, p, "Shock", Zone::Graveyard);
        let creature = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let land = put(&mut g, p.other(), "Island", Zone::Graveyard);
        checked(&mut g, announce.clone());
        for bad in [own, creature, land] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(spell),
            },
        );
        let object = g.object_ref(fodder);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        let old = g.object_ref(spell);
        g.move_object(spell, Zone::Hand);
        g.move_object(spell, Zone::Graveyard);
        assert_ne!(old, g.object_ref(spell));
        finish(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert!(!g.can_play_exiled(p, spell));
        let fodder = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        activate(&mut g, source, spell, fodder);
        next_turn(&mut g);
        assert!(!g.can_play_exiled(p, spell));
    }
}
#[test]
fn sos_nita_replacement_covers_countering_but_never_other_destinations_or_later_incarnations() {
    for p in [Player::P0, Player::P1] {
        for counter in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Nita, Forum Conciliator", Zone::Battlefield);
            let fodder = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p.other(), "Shock", Zone::Graveyard);
            activate(&mut g, source, spell, fodder);
            checked(&mut g, Action::AnnounceExile { card: spell });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
            // Resolve Nita's trigger before responding to the physical foreign spell.
            while g.state.stack.len() > 1 {
                checked(&mut g, Action::Pass);
            }
            if counter {
                if g.actor() != Some(p.other()) {
                    checked(&mut g, Action::Pass);
                }
                let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
                paid(&mut g, negate, None, &[Target::Spell(spell)]);
                finish(&mut g);
                assert_eq!(g.object_view(spell).zone, Zone::Exile);
                assert_eq!(g.state.players[p.other().index()].life, 20);
            } else {
                g.move_object(spell, Zone::Hand);
                assert_eq!(g.object_view(spell).zone, Zone::Hand);
                g.move_object(spell, Zone::Graveyard);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                finish(&mut g);
            }
        }
    }
}
