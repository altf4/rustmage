//! Complete characteristic/modal batch in real FDN-only Standard games.
use super::{deck, run_matrix_actions_observed};
use rustmage::{Card, Observation, Zone};
use std::collections::BTreeSet;

#[test]
fn foundations_six_characteristic_modal_cards_full_games_and_owned_effect_state() {
    let mut layered = deck(
        "Island",
        &[
            "Eaten by Piranhas",
            "Imprisoned in the Moon",
            "Witness Protection",
            "Demonic Pact",
            "Bear Cub",
            "Llanowar Elves",
            "Think Twice",
            "Hedron Archive",
            "Burglar Rat",
        ],
    );
    let lands = [
        "Crawling Barrens",
        "Soulstone Sanctuary",
        "Dimir Guildgate",
        "Swamp",
        "Forest",
        "Island",
    ];
    for (group, name) in layered[..24].chunks_mut(4).zip(lands) {
        group.fill(Card::from_name(name).unwrap());
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
    let mut attached = BTreeSet::new();
    let mut animated = BTreeSet::new();
    let mut modal_history = false;
    let mut owned_effect_replays = 0usize;
    let mut observe =
        |before: &Observation, after: &Observation, owned_replay: bool| {
            for aura in &after.objects {
                if aura.zone == Zone::Battlefield
                    && aura.attached_to.is_some()
                    && matches!(
                        aura.card.name(),
                        "Eaten by Piranhas" | "Imprisoned in the Moon" | "Witness Protection"
                    )
                {
                    attached.insert(aura.card.name());
                }
            }
            for animation in &after.animations {
                if before.animations.iter().any(|old| {
                    old.object == animation.object && old.timestamp == animation.timestamp
                }) {
                    continue;
                }
                let object = after
                    .objects
                    .iter()
                    .find(|o| o.reference == animation.object)
                    .unwrap();
                assert_eq!(object.zone, Zone::Battlefield);
                assert!(object.kinds & 1 != 0 && object.kinds & 2 != 0);
                if object.card.name() == "Crawling Barrens" {
                    assert!(animation.expires_turn.is_some());
                    animated.insert("Crawling Barrens");
                } else if object.card.name() == "Soulstone Sanctuary" {
                    assert!(animation.expires_turn.is_none());
                    animated.insert("Soulstone Sanctuary");
                }
            }
            modal_history |= !after.modal_history.is_empty();
            if owned_replay && (!before.animations.is_empty() || !before.modal_history.is_empty()) {
                owned_effect_replays += 1;
            }
        };
    run_matrix_actions_observed(
        &[layered, white],
        &[
            "Eaten by Piranhas",
            "Imprisoned in the Moon",
            "Witness Protection",
            "Demonic Pact",
        ],
        &["Crawling Barrens", "Soulstone Sanctuary"],
        &["Crawling Barrens", "Soulstone Sanctuary"],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
        Some(&mut observe),
    );
    for name in [
        "Eaten by Piranhas",
        "Imprisoned in the Moon",
        "Witness Protection",
    ] {
        assert!(
            attached.contains(name),
            "no natural Aura attachment: {name}"
        );
    }
    for name in ["Crawling Barrens", "Soulstone Sanctuary"] {
        assert!(animated.contains(name), "no natural land animation: {name}");
    }
    assert!(modal_history, "no natural Pact mode history");
    assert!(
        owned_effect_replays > 0,
        "no owned replay with animation or modal history"
    );
    println!(
        "FDN six-card acceptance: attached={attached:?}, animated={animated:?}, modal history={modal_history}, owned effect-state replays={owned_effect_replays}"
    );
}
