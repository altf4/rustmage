use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [10; 6];
    }
    g
}
fn replay(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
    let before = g.snapshot();
    let mut child = {
        let parent = g.clone();
        parent.clone()
    };
    child.apply_index(actor, revision, index).unwrap();
    act(g, action.clone());
    let after = g.snapshot();
    assert_eq!(child.snapshot(), after);
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(child.observe(viewer)).unwrap(),
            serde_json::to_value(g.observe(viewer)).unwrap()
        );
    }
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(g, action);
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, target: CardId) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(
            g.actor().unwrap(),
            Action::ChooseTarget {
                target: Target::Card(target)
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn pay(g: &mut Game) {
    for _ in 0..10 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .expect("legal paid continuation");
        replay(g, action);
    }
    panic!("announcement did not finish");
}
fn announce(g: &mut Game, spell: CardId, source: CardId, recipient: CardId) {
    replay(g, Action::AnnounceCast { card: spell });
    replay(
        g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    replay(
        g,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    pay(g);
}

#[test]
fn fdn_power_damage_paid_casts_current_power_counter_order_and_no_fight_back() {
    for p in [Player::P0, Player::P1] {
        for (name, cost, counter) in [("Bite Down", 2, 0), ("Felling Blow", 3, 1)] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let recipient = named(&mut g, p.other(), "Aggressive Mammoth", Zone::Battlefield);
            let spell = named(&mut g, p, name, Zone::Hand);
            let hidden = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
            let mana: u32 = g.state.players[p.index()].mana.iter().sum();
            announce(&mut g, spell, source, recipient);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                mana - cost
            );
            assert!(!g.observe(p).objects.iter().any(|o| o.id == hidden));
            let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
            cast(&mut g, growth, Some(Target::Card(source)));
            pass_pair(&mut g);
            pass_pair(&mut g);
            assert_eq!(g.object_view(recipient).damage, 5 + counter);
            assert_eq!(g.object_view(source).damage, 0);
            assert_eq!(g.object_view(source).counters.plus_one, counter as u64);
            assert_eq!(g.object_view(recipient).counters.plus_one, 0);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn fdn_power_damage_target_restrictions_and_planeswalker_damage() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bite Down", "Felling Blow"] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let own_land = named(&mut g, p, "Forest", Zone::Battlefield);
            let walker = named(
                &mut g,
                p.other(),
                "Ajani, Caller of the Pride",
                Zone::Battlefield,
            );
            g.state.objects[walker as usize].counters.loyalty = 6;
            let spell = named(&mut g, p, name, Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: spell });
            reject(&mut g, foe);
            reject(&mut g, own_land);
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(source),
                },
            );
            reject(&mut g, source);
            reject(&mut g, own_land);
            if name == "Felling Blow" {
                reject(&mut g, walker);
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foe),
                    },
                );
            } else {
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(walker),
                    },
                );
            }
            pay(&mut g);
            pass_pair(&mut g);
            assert_eq!(
                g.object_view(walker).counters.loyalty,
                if name == "Bite Down" { 4 } else { 6 }
            );
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn fdn_power_damage_partial_illegal_targets_do_only_remaining_instructions() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bite Down", "Felling Blow"] {
            for source_leaves in [false, true] {
                let mut g = setup(p);
                let source = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let recipient = named(&mut g, p.other(), "Aggressive Mammoth", Zone::Battlefield);
                let spell = named(&mut g, p, name, Zone::Hand);
                announce(&mut g, spell, source, recipient);
                let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
                cast(
                    &mut g,
                    bounce,
                    Some(Target::Card(if source_leaves { source } else { recipient })),
                );
                pass_pair(&mut g);
                pass_pair(&mut g);
                assert_eq!(g.object_view(recipient).damage, 0);
                assert_eq!(
                    g.object_view(source).counters.plus_one,
                    u64::from(name == "Felling Blow" && !source_leaves)
                );
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            }
        }
    }
}

#[test]
fn fdn_power_damage_rechecks_control_and_uses_creatures_damage_keywords() {
    for p in [Player::P0, Player::P1] {
        for source_changes in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let recipient = named(&mut g, p.other(), "Aggressive Mammoth", Zone::Battlefield);
            let spell = named(&mut g, p, "Felling Blow", Zone::Hand);
            announce(&mut g, spell, source, recipient);
            let changed = if source_changes { source } else { recipient };
            // Fixture models a control-changing response, leaving the incarnation intact.
            g.state.objects[changed as usize].controller =
                if source_changes { p.other() } else { p };
            pass_pair(&mut g);
            assert_eq!(g.object_view(recipient).damage, 0);
            assert_eq!(
                g.object_view(source).counters.plus_one,
                u64::from(!source_changes)
            );
        }
        let mut g = setup(p);
        let source = named(&mut g, p, "Vampire Nighthawk", Zone::Battlefield);
        let recipient = named(&mut g, p.other(), "Aggressive Mammoth", Zone::Battlefield);
        let spell = named(&mut g, p, "Bite Down", Zone::Hand);
        let life = g.state.players[p.index()].life;
        announce(&mut g, spell, source, recipient);
        pass_pair(&mut g);
        assert_eq!(g.object_view(recipient).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, life + 2);
        assert_eq!(g.object_view(source).damage, 0);
    }
}
