//! Focused printed outcomes; shared engine invariants live in core tests.
//! Frozen target/sacrifice/death power and controlled scaled amass outcomes.
use super::*;
const FIXTURES: &[&str] = &[
    "Azog, Moria's Ruin",
    "Rhovanion Rampager",
    "Bear Cub",
    "Forest",
    "Goldvein Pick",
    "Giant Growth",
    "Murder",
    "Rest in Peace",
    "Doubling Season",
    "Elvish Archdruid",
    "Adaptive Automaton",
    "Banishing Light",
    "@token/goblin_army",
    "Doc Samson, Super Psychiatrist",
    "Ant-Man, Colony Commander",
    "Goblin Plate Mail",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted fixture {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("azog/rampager decision budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    for _ in 0..96 {
        if quiet(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseArmy { .. }
                    | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("azog/rampager finish budget {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana, target: Option<Target>) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, target);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn source(g: &mut Game, p: Player, name: &str) -> CardId {
    paid(g, p, name, [0, 0, 1, 0, 0, 2], None)
}
fn bind(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn azog(g: &mut Game, p: Player) -> CardId {
    let id = source(g, p, "Azog, Moria's Ruin");
    until(g, |g| g.legal_actions().contains(&Action::FinishTargets));
    id
}

fn types(g: &mut Game, id: CardId, names: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: names,
            additional: false,
            timestamp,
        });
}
fn army(g: &mut Game, p: Player) -> CardId {
    let id = fixture(g, p, "Bear Cub", Zone::Battlefield);
    types(g, id, &["Army"]);
    id
}
fn armies(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && g.is_creature(id as CardId)
                && g.has_subtype(id as CardId, "Army"))
            .then_some(id as CardId)
        })
        .collect()
}
#[test]
fn hob_azog_rampager_azog_paid_optional_zero_and_other_creature_target_legality() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = fixture(&mut g, p, "Forest", Zone::Battlefield);
        let id = azog(&mut g, p);
        assert_eq!(stats(&g, id), (1, 3));
        assert!(g.object_view(id).is_legendary());
        assert!(g.has_subtype(id, "Goblin") && g.has_subtype(id, "Soldier"));
        for target in [
            Target::Card(id),
            Target::Card(land),
            Target::Player(p.other()),
        ] {
            reject(&mut g, Action::ChooseTarget { target });
        }
        act(&mut g, Action::FinishTargets);
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert!(armies(&g, p).is_empty() && armies(&g, p.other()).is_empty());
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn hob_azog_rampager_azog_resolution_controller_owner_failed_destroy_and_negative_zero() {
    {
        let p = Player::P0;
        for mode in 0..9 {
            let mut g = scenario(p);
            let target = fixture(
                &mut g,
                if mode == 1 || mode == 2 || mode == 4 {
                    p.other()
                } else {
                    p
                },
                "Bear Cub",
                Zone::Battlefield,
            );
            if mode == 2 {
                g.record_control(g.object_ref(target), p, None);
            }
            if mode == 5 {
                g.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            if mode == 6 {
                fixture(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            if mode == 7 {
                g.state.objects[target as usize].growth.power = -5;
            }
            if mode == 8 {
                g.state.objects[target as usize].growth.power = -2;
            }
            let existing = (mode == 8).then(|| army(&mut g, p));
            let source = azog(&mut g, p);
            bind(&mut g, target);
            if mode == 3 {
                g.record_control(g.object_ref(target), p.other(), None);
            }
            if mode == 4 {
                g.record_control(g.object_ref(target), p, None);
            }
            let receiver = if mode == 1 || mode == 3 { p.other() } else { p };
            finish(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if mode == 5 {
                    Zone::Battlefield
                } else if mode == 6 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                },
                "mode{mode}"
            );
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(receiver == p),
                "mode{mode}"
            );
            assert_eq!(stats(&g, source), (1, 3));
            assert!(armies(&g, receiver.other()).is_empty());
            if mode == 7 {
                assert!(armies(&g, receiver).is_empty());
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.card == Card::GoblinArmyToken
                            && o.owner == p
                            && o.zone == Zone::Outside)
                        .count(),
                    1,
                    "negative Amass0 still creates then loses 0/0 Army"
                );
            } else {
                let a = armies(&g, receiver);
                assert_eq!(a.len(), 1);
                let id = a[0];
                assert_eq!(
                    g.counter_count(id, CounterKind::PlusOne),
                    if mode == 8 { 0 } else { 2 }
                );
                assert!(g.has_subtype(id, "Goblin"));
                if let Some(existing) = existing {
                    assert_eq!(id, existing);
                    assert_eq!(stats(&g, id), (2, 2));
                } else {
                    assert!(g.object_view(id).is_token());
                    assert_eq!(g.object_view(id).colors, 4);
                }
            }
        }
    }
}

fn insects(g: &Game, player: Player) -> usize {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == player
                && o.is_token()
                && g.has_subtype(*id as CardId, "Insect")
        })
        .count()
}

#[test]
fn hob_azog_rampager_legacy_fixed_amass_keeps_original_placer_and_controller() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let recipient = army(&mut g, p);
        for player in [p, p.other()] {
            fixture(
                &mut g,
                player,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
            fixture(
                &mut g,
                player,
                "Ant-Man, Colony Commander",
                Zone::Battlefield,
            );
        }
        paid(&mut g, p, "Goblin Plate Mail", [0, 0, 1, 0, 0, 1], None);
        finish(&mut g);
        assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 2);
        assert_eq!(insects(&g, p), 1);
        assert_eq!(insects(&g, p.other()), 0);
        let object = g.object_ref(recipient);
        assert!(g.put_counter_on_source_this_turn(object, p));
        assert!(!g.put_counter_on_source_this_turn(object, p.other()));
    }
}
