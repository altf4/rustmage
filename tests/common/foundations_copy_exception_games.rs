//! Paid copy exceptions, graveyard history and loyalty in ordinary FDN games.
use super::{deck, run_matrix_actions_observed};
use rustmage::{Action, Card, CastMethod, Observation, StackKind, Zone};
use std::collections::BTreeSet;

#[test]
fn foundations_copy_exception_cards_natural_games_and_owned_state() {
    let mut black_red = deck(
        "Mountain",
        &[
            "Burglar Rat",
            "Infestation Sage",
            "Gleaming Barrier",
            "Vampire Nighthawk",
            "Burst Lightning",
            "Macabre Waltz",
            "Abyssal Harvester",
            "Electroduplicate",
            "Chandra, Flameshaper",
        ],
    );
    for land in &mut black_red[..12] {
        *land = Card::Swamp;
    }
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
    let mut copied_by = BTreeSet::new();
    let mut flashback = false;
    let mut loyalty = BTreeSet::new();
    let mut permission_used = false;
    let mut owned_copies = 0;
    let mut plus_one_legal = 0;
    let mut observe = |before: &Observation, after: &Observation, owned: bool| {
        plus_one_legal += usize::from(before.actions.iter().any(|action| {
            matches!(action, Action::AnnounceActivate { card, ability: 1, .. }
                if before.objects.iter().any(|o| o.id == *card && o.card.name() == "Chandra, Flameshaper"))
        }));
        if owned
            && (before.objects.iter().any(|o| o.copy_recipe.is_some())
                || before.target_amount.is_some()
                || !before.graveyard_entries_this_turn.is_empty())
        {
            owned_copies += 1;
        }
        for item in &after.stack {
            if item.source_card.name() == "Electroduplicate"
                && item.cast.is_some_and(|c| c.method == CastMethod::Flashback)
            {
                flashback = true;
            }
        }
        for permission in &before.exile_permissions {
            permission_used |= after.objects.iter().any(|o| {
                o.id == permission.object.card
                    && matches!(o.zone, Zone::Stack | Zone::Battlefield)
                    && before
                        .objects
                        .iter()
                        .any(|b| b.reference == permission.object && b.zone == Zone::Exile)
            });
        }
        let source = before.resolving.as_ref().or_else(|| {
            before.stack.last().filter(|item| {
                !after.stack.iter().any(|s| s.id == item.id)
                    || after.resolving.as_ref().is_some_and(|s| s.id == item.id)
            })
        });
        if let Some(source) = source {
            for object in &after.objects {
                if object.zone == Zone::Battlefield
                    && !before
                        .objects
                        .iter()
                        .any(|o| o.reference == object.reference && o.zone == Zone::Battlefield)
                    && let Some(recipe) = object.copy_recipe
                {
                    match source.source_card.name() {
                        "Abyssal Harvester" if recipe.exceptions.nightmare => {
                            copied_by.insert("Abyssal Harvester");
                        }
                        "Electroduplicate" | "Chandra, Flameshaper"
                            if recipe.exceptions.sacrifice_abilities != 0 =>
                        {
                            copied_by.insert(source.source_card.name());
                        }
                        _ => {}
                    }
                }
            }
            if source.source_card.name() == "Chandra, Flameshaper"
                && let StackKind::ActivatedAbility { ability } = source.kind
                && !after.stack.iter().any(|s| s.id == source.id)
                && after.resolving.as_ref().is_none_or(|s| s.id != source.id)
            {
                loyalty.insert(ability);
            }
        }
    };
    run_matrix_actions_observed(
        &[black_red, white],
        &[
            "Abyssal Harvester",
            "Electroduplicate",
            "Chandra, Flameshaper",
        ],
        &[],
        &[],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
        Some(&mut observe),
    );
    println!(
        "Natural witnesses: copied_by={copied_by:?}, flashback={flashback}, loyalty={loyalty:?}, permission_used={permission_used}, owned={owned_copies}, plus_one_legal={plus_one_legal}"
    );
    for name in [
        "Abyssal Harvester",
        "Electroduplicate",
        "Chandra, Flameshaper",
    ] {
        assert!(
            copied_by.contains(name),
            "no natural copy from {name}: {copied_by:?}"
        );
    }
    assert!(flashback, "no natural Electroduplicate flashback");
    assert_eq!(loyalty, BTreeSet::from([0, 1, 2]));
    assert!(permission_used, "no natural Chandra exile permission used");
    assert!(owned_copies > 0);
    println!(
        "FDN copy-exception acceptance: copy sources={copied_by:?}; loyalty abilities={loyalty:?}; flashback and exile permission used; {owned_copies} owned family-state replays"
    );
}
