//! Five direct-authored choice/link cards in ordinary set-only complete games.
use super::{deck, run_matrix_actions_observed_with_stride};
use rustmage::{Card, EntryChoiceValue, Observation, StackKind, Zone};
use std::collections::BTreeSet;
#[test]
fn foundations_linked_choice_cards_natural_games_and_owned_private_choices() {
    let mut blue = deck(
        "Island",
        &[
            "Curator of Destinies",
            "Hoarding Dragon",
            "Sorcerous Spyglass",
            "Hedron Archive",
            "Mazemind Tome",
            "Gleaming Barrier",
            "Think Twice",
            "Abrade",
            "Unsummon",
        ],
    );
    blue[..12].fill(Card::Mountain);
    let mut black = deck(
        "Swamp",
        &[
            "Liliana, Dreadhorde General",
            "Deadly Brew",
            "Llanowar Elves",
            "Gleaming Barrier",
            "Gilded Lotus",
            "Reassembling Skeleton",
            "Infestation Sage",
            "Feed the Swarm",
            "Solemn Simulacrum",
        ],
    );
    black[..12].fill(Card::Forest);
    let expected = [
        "Curator of Destinies",
        "Hoarding Dragon",
        "Sorcerous Spyglass",
        "Liliana, Dreadhorde General",
        "Deadly Brew",
    ];
    let mut mechanics = BTreeSet::new();
    let mut owned_choices = 0;
    let mut owned_links = 0;
    let mut owned_names = 0;
    let mut observe = |before: &Observation, after: &Observation, owned: bool| {
        if owned
            && (before.library_piles.is_some()
                || before.keep_types.is_some()
                || before.recovery_choice.is_some()
                || before.entry_value.is_some())
        {
            owned_choices += 1;
        }
        if owned && !before.linked_exile.is_empty() {
            owned_links += 1;
        }
        if owned
            && before.objects.iter().any(|o| {
                o.zone == Zone::Battlefield
                    && matches!(o.entry_choice, Some(EntryChoiceValue::CardName(_)))
            })
        {
            owned_names += 1;
        }
        if let Some(piles) = &before.library_piles
            && piles.choosing
            && after.library_piles.is_none()
            && after.players[piles.controller.index()].hand_count
                > before.players[piles.controller.index()].hand_count
        {
            mechanics.insert("Curator of Destinies");
        }
        if let Some(item) = before.resolving.as_ref().or_else(|| before.stack.last())
            && (before.resolving.as_ref().is_some_and(|s| s.id == item.id)
                || !after.stack.iter().any(|s| s.id == item.id))
        {
            if item.source_card.name() == "Deadly Brew"
                && before.recovery_choice.is_some()
                && after.recovery_choice.is_none()
                && after.players[item.controller.index()].hand_count
                    > before.players[item.controller.index()].hand_count
            {
                mechanics.insert("Deadly Brew");
            }
            if item.source_card.name() == "Liliana, Dreadhorde General"
                && matches!(item.kind, StackKind::TriggeredAbility { .. })
                && after.players[item.controller.index()].cards_drawn_this_turn
                    > before.players[item.controller.index()].cards_drawn_this_turn
            {
                mechanics.insert("Liliana, Dreadhorde General");
            }
            if item.source_card.name() == "Hoarding Dragon"
                && matches!(item.kind, StackKind::TriggeredAbility { .. })
                && before.linked_exile.iter().any(|link| {
                    link.source == item.object
                        && after.objects.iter().any(|o| {
                            o.id == link.exiled.card
                                && o.reference != link.exiled
                                && o.zone == Zone::Hand
                        })
                })
            {
                mechanics.insert("Hoarding Dragon");
            }
        }
        if after.objects.iter().any(|glass|glass.zone==Zone::Battlefield && glass.name()=="Sorcerous Spyglass"
            && matches!(glass.entry_choice,Some(EntryChoiceValue::CardName(name)) if after.objects.iter().any(|o|o.zone==Zone::Battlefield && o.controller!=glass.controller && o.name()==name.as_str())))
        {mechanics.insert("Sorcerous Spyglass");}
    };
    run_matrix_actions_observed_with_stride(
        &[blue, black],
        &expected,
        &[],
        &["Hedron Archive", "Gilded Lotus"],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
        Some(&mut observe),
        13,
    );
    println!(
        "FDN choice/link witnesses={mechanics:?}; owned choices/links/names={owned_choices}/{owned_links}/{owned_names}"
    );
    for name in expected {
        assert!(
            mechanics.contains(name),
            "missing actual mechanic: {name}; witnessed={mechanics:?}"
        );
    }
    assert!(owned_choices > 0 && owned_links > 0 && owned_names > 0);
}
