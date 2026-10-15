//! Supported Standard connive continuations with deterministic boundary replay.
use rustmage::{Action, Card, Config, DeckRules, Game, Player};
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
    let dimir = deck(
        &[("Island", 12), ("Swamp", 12)],
        &[
            "A.I.M. Scientists",
            "Red Room Recruit",
            "Doc Ock's Henchmen",
            "Kang, Temporal Tyrant",
            "Madame Masque",
            "Mechanical Mobster",
            "Prowler, Clawed Thief",
            "Unstable Experiment",
            "Scrawling Crawler",
        ],
    );
    let sultai = deck(
        &[("Swamp", 10), ("Forest", 8), ("Island", 6)],
        &[
            "Red Room Recruit",
            "Yellowjacket, Heartless Marauder",
            "Titanbones, Towering Heart",
            "Ambush Wolf",
            "Heritage Reclamation",
            "Wreck Remover",
            "Mechanical Mobster",
            "Prowler, Clawed Thief",
            "Scrounging Skyray",
        ],
    );
    let (
        mut decisions,
        mut requests,
        mut discards,
        mut nonlands,
        mut counters,
        mut replays,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [dimir.clone(), sultai.clone()],
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
            if before.connive_choice.is_some()&&seed%2==0&&let Some(a)=actions.iter().find(|a|matches!(a,Action::Discard {card} if before.objects.iter().any(|o|o.id==*card&&!o.card.is_land()))){action=a.clone();}
            let current = before.connive_choice.as_ref().and_then(|c| c.current);
            if current.is_some()
                && let Action::Discard { card } = action
            {
                discards += 1;
                nonlands += usize::from(
                    before
                        .objects
                        .iter()
                        .find(|o| o.id == card)
                        .is_some_and(|o| !o.card.is_land()),
                );
            }
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(g.actor().unwrap_or(actor));
            requests +=
                usize::from(before.connive_choice.is_none() && after.connive_choice.is_some());
            if let Some(current) = current
                && matches!(action, Action::Discard { .. })
            {
                let count = |objects: &[rustmage::ObjectView]| {
                    objects
                        .iter()
                        .find(|o| o.reference == current.object)
                        .map_or(0, |o| o.counters.plus_one)
                };
                counters += u64::from(count(&after.objects).saturating_sub(count(&before.objects)));
            }
            if before.connive_choice.is_some()
                || after.connive_choice.is_some()
                || before.announcing.is_some()
            {
                let expected = g.snapshot();
                g.restore(&saved);
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
        assert!(requests > 0 && discards > nonlands && nonlands > 0 && counters > 0 && replays > 0);
    }
    println!(
        "Completed {games} Standard connive games, {decisions} decisions, {requests} visible connive requests, {discards} connive discard choices, {nonlands} nonland discards, {counters} immediate connive counters, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
