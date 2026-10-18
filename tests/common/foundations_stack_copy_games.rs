//! Whole stack-copy consumers in legal FDN-only games, alongside focused rules proofs.
use super::{deck, run_matrix_actions_observed};
use rustmage::{Observation, StackKind};
use std::collections::BTreeSet;

#[test]
fn foundations_stack_copy_cards_natural_games_and_owned_continuations() {
    let mut red_blue = deck(
        "Mountain",
        &[
            "Teach by Example",
            "Thousand-Year Storm",
            "Pyromancer's Goggles",
            "Bolt Bend",
            "Burst Lightning",
            "Fanatical Firebrand",
            "Krenko, Mob Boss",
            "Shivan Dragon",
            "Think Twice",
        ],
    );
    for land in &mut red_blue[..8] {
        *land = rustmage::Card::Island;
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
    let mut bend_changed_target = false;
    let mut owned_copy_replays = 0;
    let mut observe = |before: &Observation, after: &Observation, owned: bool| {
        let copies = |o: &Observation| {
            o.stack
                .iter()
                .filter(|s| s.kind == StackKind::Spell && s.is_copy)
                .count()
        };
        if owned
            && (before.retarget.is_some()
                || copies(before) > 0
                || !before.delayed_spell_copies.is_empty())
        {
            owned_copy_replays += 1;
        }
        if copies(after) > copies(before)
            && let Some(source) = before.resolving.as_ref().or_else(|| before.stack.last())
            && source.copy_event.is_some()
        {
            copied_by.insert(source.source_card.name());
        }
        if let Some(retarget) = &before.retarget
            && retarget.victim.is_some()
            && after.retarget.is_none()
            && let Some(victim) = after.stack.iter().find(|s| Some(s.id) == retarget.victim)
        {
            bend_changed_target |= victim.targets != retarget.original;
        }
    };
    run_matrix_actions_observed(
        &[red_blue, white],
        &[
            "Teach by Example",
            "Thousand-Year Storm",
            "Pyromancer's Goggles",
            "Bolt Bend",
        ],
        &[],
        &["Pyromancer's Goggles"],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
        Some(&mut observe),
    );
    for name in [
        "Teach by Example",
        "Thousand-Year Storm",
        "Pyromancer's Goggles",
    ] {
        assert!(
            copied_by.contains(name),
            "no naturally created copy from {name}: {copied_by:?}"
        );
    }
    assert!(
        bend_changed_target,
        "Bolt Bend did not change an existing spell or ability target"
    );
    assert!(
        owned_copy_replays > 0,
        "no owned replay of a live copy continuation"
    );
    println!(
        "FDN stack-copy acceptance: copy sources={copied_by:?}; Bolt Bend changed a target; {owned_copy_replays} owned copy/delayed/retarget replays"
    );
}
