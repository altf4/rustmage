//! Full BIG candidate games; basic lands are the only permitted non-BIG cards.
use super::*;
use std::time::Instant;
fn deck(cards: &[&str], lands: &[(&str, usize)]) -> Vec<Card> {
    let mut d = Vec::new();
    for &(n, count) in lands {
        d.extend(std::iter::repeat_n(Card::from_name(n).unwrap(), count));
    }
    for n in cards {
        d.extend(std::iter::repeat_n(Card::from_name(n).unwrap(), 4));
    }
    assert_eq!(d.len(), 60);
    d
}
#[test]
fn big_complete_games_replay_owned_search_all_colors() {
    let decks = [
        deck(
            &[
                "Oltec Matterweaver",
                "Collector's Cage",
                "Simulacrum Synthesizer",
                "Worldwalker Helm",
                "Esoteric Duplicator",
                "Fomori Vault",
                "Sandstorm Salvager",
                "Transmutation Font",
                "Sword of Wealth and Power",
            ],
            &[("Plains", 10), ("Island", 10), ("Forest", 4)],
        ),
        deck(
            &[
                "Generous Plunderer",
                "Harvester of Misery",
                "Hostile Investigator",
                "Greed's Gambit",
                "Legion Extruder",
                "Molten Duplication",
                "Lotus Ring",
                "Lost Jitte",
                "Memory Vessel",
            ],
            &[("Swamp", 12), ("Mountain", 12)],
        ),
        deck(
            &[
                "Bristlebud Farmer",
                "Vaultborn Tyrant",
                "Loot, the Key to Everything",
                "Nexus of Becoming",
                "Omenpath Journey",
                "Ancient Cornucopia",
                "Tarnation Vista",
                "Lost Jitte",
                "Grand Abolisher",
            ],
            &[("Forest", 8), ("Island", 6), ("Mountain", 5), ("Plains", 5)],
        ),
        deck(
            &[
                "Rest in Peace",
                "Pest Control",
                "Torpor Orb",
                "Sandstorm Salvager",
                "Oltec Matterweaver",
                "Generous Plunderer",
                "Harvester of Misery",
                "Ancient Cornucopia",
                "Vaultborn Tyrant",
            ],
            &[("Forest", 6), ("Plains", 6), ("Swamp", 6), ("Mountain", 6)],
        ),
    ];
    let start = Instant::now();
    let mut decisions = 0usize;
    let mut actions = Vec::new();
    let mut played = std::collections::BTreeSet::new();
    let mut activated = std::collections::BTreeSet::new();
    let mut colors = 0u8;
    let mut outcomes = Vec::new();
    for (i, deck) in decks.iter().enumerate() {
        for seed in 0..3 {
            let config = Config {
                decks: [deck.clone(), decks[(i + 1) % decks.len()].clone()],
                seed: 56710 + seed,
                starting_player: if seed % 2 == 0 {
                    Player::P0
                } else {
                    Player::P1
                },
                deck_rules: DeckRules::Standard,
            };
            let mut g = Game::new(config.clone()).unwrap();
            let mut trace = Vec::new();
            let mut count = 0usize;
            while g.outcome().is_none() {
                g.legal_actions_into(&mut actions);
                let index = crate::policy::smoke_index(&g, &actions);
                let actor = g.actor().unwrap();
                let a = actions[index].clone();
                trace.push((actor, a.clone()));
                match &a {
                    Action::AnnounceCast { card }
                    | Action::AnnounceExile { card }
                    | Action::AnnounceFree { card, .. }
                    | Action::Cast { card, .. }
                    | Action::BeginCast { card, .. } => {
                        played.insert(g.state.objects[*card as usize].card.name());
                        colors |= g.state.objects[*card as usize].card.colors();
                    }
                    Action::AnnounceActivate { card, .. } | Action::Activate { card, .. } => {
                        activated.insert(g.state.objects[*card as usize].card.name());
                    }
                    _ => {}
                }
                if count % 23 == 0 {
                    let saved = g.snapshot();
                    let rev = g.revision();
                    let mut owned = {
                        let parent = g.clone();
                        parent.clone()
                    };
                    owned.apply_index(actor, rev, index).unwrap();
                    g.apply(actor, a.clone()).unwrap();
                    assert_eq!(g.snapshot(), owned.snapshot());
                    for p in [Player::P0, Player::P1] {
                        assert_eq!(
                            serde_json::to_value(g.observe(p)).unwrap(),
                            serde_json::to_value(owned.observe(p)).unwrap()
                        );
                    }
                    g.restore(&saved).unwrap();
                    g.apply(actor, a).unwrap();
                    assert_eq!(g.snapshot(), owned.snapshot());
                } else {
                    g.apply(actor, a).unwrap();
                }
                count += 1;
                assert!(count < 30000, "BIG game failed to terminate {i}/{seed}");
            }
            let mut replay = Game::new(config).unwrap();
            for (actor, action) in trace {
                replay.apply(actor, action).unwrap();
            }
            assert_eq!(g.snapshot(), replay.snapshot());
            outcomes.push(serde_json::json!({"deck":i,"seed":56710+seed,"decisions":count,"outcome":g.outcome()}));
            decisions += count;
        }
    }
    let seconds = start.elapsed().as_secs_f64();
    println!(
        "BIG_GAME_EVIDENCE {}",
        serde_json::json!({"games":12,"decisions":decisions,"cards_cast":played,"cards_activated":activated,"spell_colors_mask":colors,"outcomes":outcomes,"seconds":seconds,"games_per_second":12.0/seconds,"decisions_per_second":decisions as f64/seconds,"includes":"Standard BIG decks with basic lands, deterministic smoke policy, natural outcomes, complete raw replay, sampled owned/raw/index/restore and observation equality","excludes":"model inference, exhaustive choice coverage"})
    );
}
