//! Six typed combat/allocation consumers using the unchanged shared game runner.
use super::run_matrix_actions_observed_with_stride;
use rustmage::{Card, CounterKind, Observation, Player, StackKind, Target, Zone};
use std::collections::BTreeSet;

#[test]
fn foundations_combat_allocation_cards_natural_games_and_owned_state() {
    let mut dragons = vec![Card::Mountain; 16];
    dragons.extend([Card::Plains; 16]);
    for name in [
        "Aurelia, the Warleader",
        "Drakuseth, Maw of Flames",
        "Steel Hellkite",
        "Goblin Negotiation",
        "Dragonlord's Servant",
        "Hedron Archive",
        "Gleaming Barrier",
    ] {
        dragons.extend([Card::from_name(name).unwrap(); 4]);
    }
    let mut green = vec![Card::Forest; 32];
    for name in [
        "Biogenic Upgrade",
        "Joraga Invocation",
        "Llanowar Elves",
        "Druid of the Cowl",
        "Rampaging Baloths",
        "Gleaming Barrier",
        "Reclamation Sage",
    ] {
        green.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(dragons.len(), 60);
    assert_eq!(green.len(), 60);
    let expected = [
        "Aurelia, the Warleader",
        "Drakuseth, Maw of Flames",
        "Steel Hellkite",
        "Goblin Negotiation",
        "Biogenic Upgrade",
        "Joraga Invocation",
    ];
    let mut witnessed = BTreeSet::new();
    let mut hellkite_abilities = BTreeSet::new();
    let mut hellkite_destroyed = 0;
    let mut owned = [0; 3];
    let mut observe = |before: &Observation, after: &Observation, replay: bool| {
        if replay {
            owned[0] += usize::from(
                before.target_amount.is_some()
                    || before.stack.iter().any(|s| s.target_allocations.is_some()),
            );
            owned[1] +=
                usize::from(before.additional_combats > 0 || !before.must_be_blocked.is_empty());
            owned[2] += usize::from(!before.combat_damaged_players.is_empty());
        }
        for row in &after.combat_damaged_players {
            if !before.combat_damaged_players.contains(row)
                && let Some(object) = before.objects.iter().find(|o| o.reference == row.source)
                && object.name() == "Steel Hellkite"
            {
                witnessed.insert(("Steel Hellkite", object.controller));
            }
        }
        let source = before.resolving.as_ref().or_else(|| {
            before.stack.last().filter(|s| {
                !after.stack.iter().any(|queued| queued.id == s.id)
                    || after.resolving.as_ref().is_some_and(|r| r.id == s.id)
            })
        });
        let Some(source) = source else {
            return;
        };
        let name = source.source_card.name();
        let player = source.controller;
        match name {
            "Aurelia, the Warleader"
                if source.kind != StackKind::Spell
                    && after.additional_combats > before.additional_combats =>
            {
                witnessed.insert((name, player));
            }
            "Joraga Invocation"
                if source.kind == StackKind::Spell
                    && after.must_be_blocked.len() > before.must_be_blocked.len() =>
            {
                witnessed.insert((name, player));
            }
            "Biogenic Upgrade" if source.kind == StackKind::Spell => {
                if source
                    .targets
                    .iter()
                    .filter_map(|b| b.object)
                    .any(|target| {
                        let old = before.objects.iter().find(|o| o.reference == target);
                        let new = after.objects.iter().find(|o| o.reference == target);
                        old.zip(new).is_some_and(|(old, new)| {
                            new.counters.get(CounterKind::PlusOne)
                                > old.counters.get(CounterKind::PlusOne)
                        })
                    })
                {
                    witnessed.insert((name, player));
                }
            }
            "Goblin Negotiation" if source.kind == StackKind::Spell => {
                if let Some(cast) = source.cast {
                    assert_eq!(
                        cast.payment.into_iter().map(u64::from).sum::<u64>(),
                        u64::from(source.x.unwrap()) + 2
                    );
                }
                if after.objects.iter().any(|o| {
                    o.zone == Zone::Battlefield
                        && o.is_token()
                        && o.name() == "Goblin"
                        && !before.objects.iter().any(|old| {
                            old.reference == o.reference && old.zone == Zone::Battlefield
                        })
                }) {
                    witnessed.insert((name, player));
                }
            }
            "Drakuseth, Maw of Flames" if source.kind != StackKind::Spell => {
                if source.targets.iter().any(|bound| match bound.target {
                    Target::Player(p) => {
                        after.players[p.index()].life < before.players[p.index()].life
                    }
                    Target::Card(id) => before
                        .objects
                        .iter()
                        .find(|o| o.id == id && o.zone == Zone::Battlefield)
                        .zip(after.objects.iter().find(|o| o.id == id))
                        .is_some_and(|(old, new)| {
                            new.damage > old.damage || new.zone == Zone::Graveyard
                        }),
                    _ => false,
                }) {
                    witnessed.insert((name, player));
                }
            }
            "Steel Hellkite" => {
                if let StackKind::ActivatedAbility { ability } = source.kind
                    && !after.stack.iter().any(|s| s.id == source.id)
                    && after.resolving.as_ref().is_none_or(|s| s.id != source.id)
                {
                    hellkite_abilities.insert((player, ability));
                    if ability == 1 {
                        hellkite_destroyed += before
                            .objects
                            .iter()
                            .filter(|old| {
                                old.zone == Zone::Battlefield
                                    && after
                                        .objects
                                        .iter()
                                        .any(|new| new.id == old.id && new.zone == Zone::Graveyard)
                            })
                            .count();
                    }
                }
            }
            _ => (),
        }
    };
    run_matrix_actions_observed_with_stride(
        &[dragons, green],
        &expected,
        &[],
        &["Hedron Archive"],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
        Some(&mut observe),
        13,
    );
    println!(
        "FDN combat/allocation witnesses={witnessed:?}; Hellkite abilities={hellkite_abilities:?}, destroyed={hellkite_destroyed}; owned allocation/combat/history={owned:?}"
    );
    for name in expected {
        for player in [Player::P0, Player::P1] {
            assert!(
                witnessed.contains(&(name, player)),
                "missing actual {name} mechanic for {player:?}; {witnessed:?}"
            );
        }
    }
    for player in [Player::P0, Player::P1] {
        for ability in [0, 1] {
            assert!(
                hellkite_abilities.contains(&(player, ability)),
                "missing Hellkite activation{ability} for{player:?}"
            );
        }
    }
    assert!(owned.into_iter().all(|count| count > 0));
}
