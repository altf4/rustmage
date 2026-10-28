//! Control changes and source departure while casting prepared spell copies.
use super::big_helpers::{checked, settle, setup};
use super::*;

fn candidate(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = put(g, p, "Forest", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    id
}

fn finish(g: &mut Game, mut target: Option<Target>) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let selected = target.map(|target| Action::ChooseTarget { target });
        let action = if selected.as_ref().is_some_and(|a| actions.contains(a)) {
            target = None;
            selected.unwrap()
        } else {
            actions
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
                .expect("prepared cast can be completed")
        };
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
fn controller_of_prepared_permanent_casts_its_copy_and_gets_effect() {
    for owner in [Player::P0, Player::P1] {
        for (name, seed) in [
            ("Emergency Phytomedic // Seed Suture", true),
            ("Blossom-Blessed Angel // Seed Suture", true),
            ("Heartwood Crafter // Soul Tether", false),
            ("Konstrari Improviser // Soul Tether", false),
        ] {
            let mut g = setup(owner);
            let source = candidate(&mut g, owner, name);
            checked(&mut g, Action::AnnounceCast { card: source });
            finish(&mut g, None);
            settle(&mut g);
            let spell = prepared_copy(&g, source);
            let recipient = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            let controller = owner.other();
            let old_life = g.state.players[controller.index()].life;
            let old_owner_life = g.state.players[owner.index()].life;
            g.change_controller(g.object_ref(source), controller);
            g.state.active = controller;
            g.state.priority = controller;
            checked(&mut g, Action::AnnounceExile { card: spell.card });
            finish(&mut g, seed.then_some(Target::Card(recipient)));
            assert!(!g.is_prepared(g.object_ref(source)));
            settle(&mut g);
            if seed {
                assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 1);
                assert_eq!(g.state.players[controller.index()].life, old_life + 1);
                assert_eq!(g.state.players[owner.index()].life, old_owner_life);
            } else {
                assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                    && o.controller == controller
                    && o.card == Card::test_index(5438)));
            }
        }
    }
}

#[test]
fn prepared_source_can_be_sacrificed_for_mana_during_its_copys_cast() {
    for p in [Player::P0, Player::P1] {
        for (name, seed) in [
            ("Emergency Phytomedic // Seed Suture", true),
            ("Heartwood Crafter // Soul Tether", false),
        ] {
            let mut g = setup(p);
            let source = candidate(&mut g, p, name);
            checked(&mut g, Action::AnnounceCast { card: source });
            finish(&mut g, None);
            settle(&mut g);
            let copy = prepared_copy(&g, source);
            let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
            g.state.objects[ring as usize].attached_to = Some(g.object_ref(source));
            g.state.objects[source as usize].summoning_sick = false;
            g.state.players[p.index()].mana = [0; 6];
            checked(&mut g, Action::AnnounceExile { card: copy.card });
            if let Some(action) = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
            {
                checked(&mut g, action);
            }
            if seed {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
            }
            let mana = Action::TapGrantedMana {
                card: source,
                grantor: g.object_ref(ring),
                production: [0, 0, 0, 0, 3, 0],
            };
            checked(&mut g, mana);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.object_view(copy.card).zone, Zone::Stack);
            finish(&mut g, None);
            settle(&mut g);
            if seed {
                assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
            } else {
                assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card == Card::test_index(5438)));
            }
        }
    }
}
