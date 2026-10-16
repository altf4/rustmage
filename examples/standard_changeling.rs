//! Native supported Standard tribal interactions and changeling continuations.
use rustmage::{Action, Card, Config, DeckRules, Game, Keywords, Observation, Player, StackKind};
fn tribal_triggers(view: &Observation) -> Vec<u64> {
    view.stack
        .iter()
        .chain(&view.waiting_triggers)
        .chain(view.announcing.iter())
        .filter(|s| {
            matches!(s.kind, StackKind::TriggeredAbility { .. })
                && [
                    "Mary Jane Watson",
                    "Youthful Valkyrie",
                    "Flying Octobot",
                    "Obyra, Dreaming Duelist",
                ]
                .contains(&s.source_card.name())
        })
        .map(|s| s.id)
        .collect()
}
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for &(name, n) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported(), "{name}");
        cards.extend([c; 4]);
    }
    assert_eq!(cards.len(), 60);
    cards
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let bant = deck(
        &[("Plains", 10), ("Forest", 10), ("Island", 4)],
        &[
            "Chitinous Graspling",
            "Prideful Feastling",
            "Changeling Wayfinder",
            "Chomping Changeling",
            "Flock Impostor",
            "Three Tree Mascot",
            "Mary Jane Watson",
            "Youthful Valkyrie",
            "Imperious Perfect",
        ],
    );
    let dimir = deck(
        &[("Island", 12), ("Swamp", 12)],
        &[
            "Mischievous Sneakling",
            "Graveshifter",
            "Rooftop Percher",
            "Three Tree Mascot",
            "Homicide Investigator",
            "Hostile Investigator",
            "Flying Octobot",
            "Mob Lookout",
            "Obyra, Dreaming Duelist",
        ],
    );
    let (
        mut decisions,
        mut entries,
        mut optional_searches,
        mut subtype_triggers,
        mut replays,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [bant.clone(), dimir.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut actions = Vec::new();
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            g.legal_actions_into(&mut actions);
            let a = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            if before.decision == "optional_effect"
                && before
                    .resolving
                    .as_ref()
                    .is_some_and(|s| s.source_card.name() == "Changeling Wayfinder")
                && matches!(a, Action::ChooseOptional { .. })
            {
                optional_searches += 1;
            }
            let saved = g.snapshot();
            g.apply(actor, a.clone())?;
            let after = g.observe(g.actor().unwrap_or(actor));
            let previous = tribal_triggers(&before);
            subtype_triggers += tribal_triggers(&after)
                .iter()
                .filter(|id| !previous.contains(id))
                .count();
            entries += after
                .objects
                .iter()
                .filter(|o| {
                    o.zone == rustmage::Zone::Battlefield
                        && o.keywords.contains(Keywords::CHANGELING)
                        && !before.objects.iter().any(|b| {
                            b.zone == rustmage::Zone::Battlefield && b.reference == o.reference
                        })
                })
                .count();
            if before.announcing.is_some()
                || before.search_choice.is_some()
                || after.search_choice.is_some()
                || before.decision == "optional_effect"
                || after.decision == "optional_effect"
            {
                let expected = g.snapshot();
                g.restore(&saved).unwrap();
                let mut fork = g.clone();
                g.apply(actor, a.clone())?;
                fork.apply(actor, a)?;
                assert_eq!(g.snapshot(), expected);
                assert_eq!(fork.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game {seed} did not finish");
    }
    if games >= 20 {
        assert!(entries > 0 && optional_searches > 0 && replays > 0);
    }
    println!(
        "Completed {games} supported Standard changeling games, {decisions} decisions, {entries} observed changeling entries, {optional_searches} Wayfinder optional-search choices, {subtype_triggers} new visible subtype triggers, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
