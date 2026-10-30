//! Adversarial lifecycle and mana-spending checks for FRA prepared cards.
use super::big_helpers::{checked, settle, setup};
use super::*;

fn candidate(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = put(g, p, "Forest", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    id
}

fn finish(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .expect("completable announcement");
        checked(g, action);
    }
    panic!("announcement did not finish");
}

fn prepared_copy(g: &Game, source: CardId) -> ObjectRef {
    g.state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|row| row.permanent == g.object_ref(source))
        .unwrap()
        .spell_copy
}

#[test]
fn prepared_permanent_departure_and_reentry_use_new_incarnations() {
    for p in [Player::P0, Player::P1] {
        for (name, enters_prepared) in [
            ("Blossom-Blessed Angel // Seed Suture", true),
            ("Emergency Phytomedic // Seed Suture", true),
            ("Heartwood Crafter // Soul Tether", true),
            ("Konstrari Improviser // Soul Tether", true),
            ("Woodwork Prodigy // Soul Tether", false),
        ] {
            let mut g = setup(p);
            let source = candidate(&mut g, p, name);
            checked(&mut g, Action::AnnounceCast { card: source });
            finish(&mut g);
            settle(&mut g);
            if !enters_prepared {
                g.begin_step(Step::Upkeep);
                g.give_priority();
                g.prepare_triggers();
                settle(&mut g);
            }
            let original_source = g.object_ref(source);
            let original_copy = prepared_copy(&g, source);
            g.move_object(source, Zone::Graveyard);
            assert!(!g.is_prepared(original_source), "{name}");
            assert_eq!(
                g.object_view(original_copy.card).zone,
                Zone::Exile,
                "{name}"
            );
            g.state_based_actions();
            assert_eq!(
                g.object_view(original_copy.card).zone,
                Zone::Outside,
                "{name}"
            );
            g.move_object(source, Zone::Battlefield);
            assert_ne!(g.object_ref(source), original_source, "{name}");
            assert_eq!(
                g.is_prepared(g.object_ref(source)),
                enters_prepared,
                "{name}"
            );
            if !enters_prepared {
                g.begin_step(Step::Upkeep);
                g.give_priority();
                g.prepare_triggers();
                settle(&mut g);
            }
            assert_ne!(prepared_copy(&g, source), original_copy, "{name}");
        }
    }
}

#[test]
fn soul_tether_heartwood_mana_can_pay_for_each_hybrid_color() {
    for p in [Player::P0, Player::P1] {
        for green in [false, true] {
            let mut g = setup(p);
            let source = candidate(&mut g, p, "Heartwood Crafter // Soul Tether");
            checked(&mut g, Action::AnnounceCast { card: source });
            finish(&mut g);
            settle(&mut g);
            let copy = prepared_copy(&g, source);
            checked(&mut g, Action::AnnounceExile { card: copy.card });
            finish(&mut g);
            settle(&mut g);
            let token = g
                .state
                .objects
                .iter()
                .enumerate()
                .find(|(_, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card == Card::test_index(5438)
                })
                .map(|(i, _)| i as CardId)
                .unwrap();
            g.state.players[p.index()].mana = [0; 6];
            let production = if green {
                [0, 0, 0, 0, 1, 0]
            } else {
                [0, 0, 0, 1, 0, 0]
            };
            let action = g.legal_actions().into_iter().find(|a| matches!(a,
                Action::ActivateMana { card, production: actual, .. } if *card == token && *actual == production
            )).expect("Heartwood can produce selected color");
            checked(&mut g, action);
            assert!(g.object_view(token).tapped);
            assert_eq!(g.state.players[p.index()].mana, production);
            let spell = candidate(&mut g, p, "Woodwork Prodigy // Soul Tether");
            g.state.players[p.index()].mana[5] = 2;
            checked(&mut g, Action::AnnounceCast { card: spell });
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert!(g.on_battlefield(spell));
        }
    }
}
