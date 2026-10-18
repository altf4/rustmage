//! Six growth/cost candidates in ordinary Foundations-only Standard games.
use super::{deck, run_matrix_actions_observed};
use rustmage::{Card, CounterKind, Keywords, Observation, Player, StackKind, Zone};
use std::collections::BTreeSet;

#[test]
fn foundations_six_growth_cards_natural_games_and_owned_state() {
    let mut elves = deck(
        "Forest",
        &[
            "Llanowar Elves",
            "Dwynen's Elite",
            "Elvish Archdruid",
            "Lathril, Blade of the Elves",
            "Wildborn Preserver",
            "Quilled Greatwurm",
            "Fishing Pole",
            "Macabre Waltz",
            "Feed the Swarm",
        ],
    );
    elves[..10].fill(Card::Swamp);
    let mut spells = deck(
        "Mountain",
        &[
            "Kellan, Planar Trailblazer",
            "Ovika, Enigma Goliath",
            "Fishing Pole",
            "Hedron Archive",
            "Think Twice",
            "Unsummon",
            "Burst Lightning",
            "Gleaming Barrier",
            "Seize the Spoils",
        ],
    );
    spells[..12].fill(Card::Island);
    let white = deck(
        "Plains",
        &[
            "Healer's Hawk",
            "Inspiring Overseer",
            "Serra Angel",
            "Leonin Vanguard",
            "Angel of Vitality",
            "Resolute Reinforcements",
            "Pacifism",
            "Day of Judgment",
            "Banishing Light",
        ],
    );
    let expected = [
        "Fishing Pole",
        "Kellan, Planar Trailblazer",
        "Lathril, Blade of the Elves",
        "Ovika, Enigma Goliath",
        "Quilled Greatwurm",
        "Wildborn Preserver",
    ];
    let mut witnessed = BTreeSet::new();
    let mut owned_growth = 0;
    let mut evolved_double_strike = BTreeSet::new();
    let mut natural_counter_cast = BTreeSet::new();
    let mut natural_fish = BTreeSet::new();
    let mut observe =
        |before: &Observation, after: &Observation, owned: bool| {
            if owned
                && (!before.temporary_abilities.is_empty()
                    || before.additional_cost.is_some()
                    || before.x_choice.is_some()
                    || before.objects.iter().any(|o| {
                        o.zone == Zone::Battlefield && o.counters.get(CounterKind::Bait) > 0
                    }))
            {
                owned_growth += 1;
            }
            for item in &after.stack {
                if item.source_card.name() == "Quilled Greatwurm"
                    && item
                        .cast
                        .is_some_and(|c| c.method == rustmage::CastMethod::GraveyardCounterCost)
                {
                    natural_counter_cast.insert(item.controller);
                }
            }
            let source = before.resolving.as_ref().or_else(|| {
                before.stack.last().filter(|item| {
                    !after.stack.iter().any(|s| s.id == item.id)
                        || after.resolving.as_ref().is_some_and(|s| s.id == item.id)
                })
            });
            let Some(source) = source else {
                return;
            };
            if source.kind == StackKind::Spell {
                return;
            }
            let fresh: Vec<_> = after
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && !before.objects.iter().any(|old| {
                            old.reference == o.reference && old.zone == Zone::Battlefield
                        })
                })
                .collect();
            let controller = source.controller;
            match source.source_card.name() {
                "Fishing Pole" => {
                    if fresh.iter().any(|o| {
                        o.controller == controller && o.is_token() && o.has_subtype("Fish")
                    }) {
                        witnessed.insert(("Fishing Pole", controller));
                        natural_fish.insert(controller);
                    }
                }
                "Kellan, Planar Trailblazer" => {
                    if let Some(k) = after
                        .objects
                        .iter()
                        .find(|o| o.reference == source.object && o.zone == Zone::Battlefield)
                    {
                        if k.has_subtype("Detective") && !k.has_subtype("Scout") {
                            witnessed.insert(("Kellan, Planar Trailblazer", controller));
                        }
                        if k.has_subtype("Rogue") && k.keywords.contains(Keywords::DOUBLE_STRIKE) {
                            evolved_double_strike.insert(controller);
                        }
                    }
                }
                "Lathril, Blade of the Elves" => {
                    if source
                        .damage_event
                        .is_some_and(|e| e.combat && e.amount > 0)
                        && fresh.iter().any(|o| {
                            o.controller == controller
                                && o.is_token()
                                && o.has_subtype("Elf")
                                && o.has_subtype("Warrior")
                        })
                    {
                        witnessed.insert(("Lathril, Blade of the Elves", controller));
                    }
                }
                "Ovika, Enigma Goliath" => {
                    if source.cast_event.as_ref().is_some_and(|e| e.mana_value > 0)
                        && fresh.iter().any(|o| {
                            o.controller == controller
                                && o.is_token()
                                && o.has_subtype("Phyrexian")
                                && o.has_subtype("Goblin")
                                && o.keywords.contains(Keywords::HASTE)
                        })
                    {
                        witnessed.insert(("Ovika, Enigma Goliath", controller));
                    }
                }
                "Quilled Greatwurm" => {
                    if let Some(e) = &source.creature_combat_event
                        && before
                            .objects
                            .iter()
                            .any(|o| o.reference == e.dealer && o.zone == Zone::Battlefield)
                        && after.objects.iter().any(|o| {
                            o.reference == e.dealer
                                && o.zone == Zone::Battlefield
                                && o.counters.get(CounterKind::PlusOne)
                                    > before
                                        .objects
                                        .iter()
                                        .find(|old| old.reference == e.dealer)
                                        .unwrap()
                                        .counters
                                        .get(CounterKind::PlusOne)
                        })
                    {
                        witnessed.insert(("Quilled Greatwurm", controller));
                    }
                }
                "Wildborn Preserver"
                    if matches!(source.kind, StackKind::TriggeredAbility { ability: 1 })
                        && source.x.is_some_and(|x| x > 0)
                        && after.objects.iter().any(|o| {
                            o.reference == source.object
                                && o.zone == Zone::Battlefield
                                && before
                                    .objects
                                    .iter()
                                    .find(|old| old.reference == o.reference)
                                    .is_some_and(|old| {
                                        o.counters.get(CounterKind::PlusOne)
                                            > old.counters.get(CounterKind::PlusOne)
                                    })
                        }) =>
                {
                    witnessed.insert(("Wildborn Preserver", controller));
                }
                _ => {}
            }
        };
    run_matrix_actions_observed(
        &[elves, spells, white],
        &expected,
        &[],
        &[],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
        Some(&mut observe),
    );
    println!(
        "Growth witnesses={witnessed:?}; double strike={evolved_double_strike:?}; counter casts={natural_counter_cast:?}; fish={natural_fish:?}; owned growth={owned_growth}"
    );
    for name in expected {
        for player in [Player::P0, Player::P1] {
            assert!(
                witnessed.contains(&(name, player)),
                "no actual {name} mechanic in seat {player:?}: {witnessed:?}"
            );
        }
    }
    assert_eq!(
        evolved_double_strike,
        BTreeSet::from([Player::P0, Player::P1])
    );
    assert!(owned_growth > 0);
}
