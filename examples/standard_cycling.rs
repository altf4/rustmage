//! Supported Standard cycling and typecycling games with owned-boundary replay.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, StackKind, Zone};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for &(name, n) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let card = Card::from_name(name).unwrap();
        assert!(card.is_supported(), "{name}");
        cards.extend([card; 4]);
    }
    assert_eq!(cards.len(), 60);
    cards
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let selesnya = deck(
        &[("Forest", 10), ("Plains", 10), ("Lush Portico", 4)],
        &[
            "Topiary Panther",
            "Soaring Sandwing",
            "Balamb T-Rexaur",
            "Kree Sentinel",
            "Vinelasher Adept",
            "Slavering Branchsnapper",
            "Lightshield Parry",
            "Migrating Ketradon",
            "Healer's Hawk",
        ],
    );
    let dimir = deck(
        &[("Island", 10), ("Swamp", 10), ("Capital City", 4)],
        &[
            "Locust Spray",
            "Shefet Archfiend",
            "Daggermaw Megalodon",
            "Marauding Brinefang",
            "Canyon Crawler",
            "Stockman, Mad Fly-entist",
            "Undulating Witness",
            "Divert Disaster",
            "Fear of Failed Tests",
        ],
    );
    let (
        mut decisions,
        mut cycles,
        mut searches,
        mut found,
        mut missed,
        mut replays,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [selesnya.clone(), dimir.clone()],
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
            let mut action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            if before.search_choice.is_some() && seed % 4 == 0 {
                action = Action::FinishSearch;
            }
            if matches!(action, Action::FinishSearch) {
                if before.search_choice.as_ref().unwrap().selected.is_empty() {
                    missed += 1;
                } else {
                    found += 1;
                }
            }
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(g.actor().unwrap_or(actor));
            cycles += after
                .stack
                .iter()
                .filter(|s| {
                    matches!(s.kind, StackKind::ActivatedAbility { .. })
                        && !before.stack.iter().any(|old| old.id == s.id)
                        && before
                            .objects
                            .iter()
                            .any(|o| o.id == s.card && o.zone == Zone::Hand)
                        && after
                            .objects
                            .iter()
                            .any(|o| o.id == s.card && o.zone == Zone::Graveyard)
                })
                .count();
            searches +=
                usize::from(before.search_choice.is_none() && after.search_choice.is_some());
            if before.announcing.is_some()
                || before.search_choice.is_some()
                || after.search_choice.is_some()
                || before
                    .stack
                    .iter()
                    .chain(&after.stack)
                    .any(|s| matches!(s.kind, StackKind::ActivatedAbility { .. }))
            {
                let expected = g.snapshot();
                g.restore(&saved).unwrap();
                let mut fork = g.clone();
                g.apply(actor, action.clone())?;
                fork.apply(actor, action)?;
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
        assert!(cycles > 0 && searches > 0 && found > 0 && missed > 0 && replays > 0);
    }
    println!(
        "Completed {games} Standard cycling games, {decisions} decisions, {cycles} cycling activations, {searches} searches, {found} successful searches, {missed} failed-to-find searches, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
