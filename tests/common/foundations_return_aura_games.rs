//! Whole return/Aura consumers in ordinary Foundations-only games.
use super::{deck, run_matrix_actions_observed};
use rustmage::{Card, CounterKind, Observation, StackKind, Zone};
use std::collections::BTreeSet;

#[test]
fn foundations_seven_return_aura_cards_natural_games_and_owned_links() {
    let mut black = deck(
        "Plains",
        &[
            "Charming Prince",
            "Infernal Vessel",
            "Nine-Lives Familiar",
            "Valkyrie's Call",
            "Giada, Font of Hope",
            "Inspiring Overseer",
            "Serra Angel",
            "Hero's Downfall",
            "Feed the Swarm",
        ],
    );
    black[..12].fill(Card::Swamp);
    let mut blue = deck(
        "Plains",
        &[
            "Kykar, Zephyr Awakener",
            "Confiscate",
            "Angelic Destiny",
            "Charming Prince",
            "Hedron Archive",
            "Think Twice",
            "Unsummon",
            "Inspiring Overseer",
            "Resolute Reinforcements",
        ],
    );
    blue[..12].fill(Card::Island);
    let expected = [
        "Charming Prince",
        "Infernal Vessel",
        "Nine-Lives Familiar",
        "Valkyrie's Call",
        "Kykar, Zephyr Awakener",
        "Confiscate",
        "Angelic Destiny",
    ];
    let mut witnessed = BTreeSet::new();
    let mut owned_links = 0;
    let mut owned_auras = 0;
    let mut observe = |before: &Observation, after: &Observation, replay: bool| {
        if replay {
            if !before.delayed_returns.is_empty()
                || before.stack.iter().any(|item| item.return_link.is_some())
            {
                owned_links += 1;
            }
            if before.objects.iter().any(|o| {
                o.zone == Zone::Battlefield && o.name() == "Confiscate" && o.attached_to.is_some()
            }) {
                owned_auras += 1;
            }
        }
        if let Some(item) = before.resolving.as_ref().or_else(|| before.stack.last())
            && (before
                .resolving
                .as_ref()
                .is_some_and(|current| current.id == item.id)
                || !after.stack.iter().any(|queued| queued.id == item.id))
        {
            if let Some(link) = &item.return_link
                && before
                    .objects
                    .iter()
                    .any(|o| o.reference == link.object && o.zone == link.zone)
                && let Some(returned) = after.objects.iter().find(|o| {
                    o.id == link.object.card
                        && o.zone == Zone::Battlefield
                        && o.reference != link.object
                })
            {
                match item.source_card.name() {
                    "Infernal Vessel" => {
                        assert!(returned.has_subtype("Demon"));
                        assert!(returned.counters.get(CounterKind::PlusOne) >= 2);
                    }
                    "Nine-Lives Familiar" => {
                        assert_eq!(returned.controller, item.controller);
                        assert!(returned.counters.get(CounterKind::Revival) < 8);
                    }
                    "Valkyrie's Call" => {
                        assert!(returned.has_subtype("Angel"));
                        assert!(returned.keywords.contains(rustmage::Keywords::FLYING));
                    }
                    "Charming Prince" => assert_eq!(returned.controller, item.controller),
                    _ => {}
                }
                witnessed.insert(item.source_card.name());
            }
            if item.source_card.name() == "Angelic Destiny" && item.kind != StackKind::Spell {
                let old = item
                    .source_move
                    .as_ref()
                    .map(|movement| movement.after)
                    .unwrap_or(item.object);
                if let Some(aura) = before
                    .objects
                    .iter()
                    .find(|o| o.reference == old && matches!(o.zone, Zone::Graveyard | Zone::Exile))
                    && after.players[aura.owner.index()].hand_count
                        > before.players[aura.owner.index()].hand_count
                {
                    witnessed.insert("Angelic Destiny");
                }
            }
            if item.source_card.name() == "Kykar, Zephyr Awakener"
                && item.kind != StackKind::Spell
                && after.objects.iter().any(|o| {
                    o.zone == Zone::Battlefield
                        && o.is_token()
                        && o.has_subtype("Spirit")
                        && o.keywords.contains(rustmage::Keywords::FLYING)
                        && !before
                            .objects
                            .iter()
                            .any(|old| old.reference == o.reference)
                })
            {
                witnessed.insert("Kykar, Zephyr Awakener");
            }
        }
        for aura in after
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.name() == "Confiscate")
        {
            if let Some(target) = aura.attached_to
                && let Some(current) = after.objects.iter().find(|o| o.reference == target)
                && before
                    .objects
                    .iter()
                    .any(|old| old.reference == target && old.controller != current.controller)
            {
                assert_eq!(current.controller, aura.controller);
                witnessed.insert("Confiscate");
            }
        }
    };
    run_matrix_actions_observed(
        &[black, blue],
        &expected,
        &[],
        &["Hedron Archive"],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
        Some(&mut observe),
    );
    for name in expected {
        assert!(
            witnessed.contains(name),
            "missing actual return/Aura mechanic: {name}; witnessed={witnessed:?}"
        );
    }
    assert!(owned_links > 0, "no owned replay with live return links");
    assert!(owned_auras > 0, "no owned replay with active Aura control");
    println!(
        "FDN return/Aura witnesses={witnessed:?}; owned links={owned_links}, owned control={owned_auras}"
    );
}
