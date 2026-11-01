//! Focused printed outcomes; shared engine invariants live in core tests.
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
fn sos_nita_activation_legality_deadline_and_target_reincarnation() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
