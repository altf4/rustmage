//! Supported Standard self-play with resolution payments and simultaneous edicts.
use rustmage::{Action, Card, Config, DeckRules, EffectChoiceKind, Game, Player};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut result = Vec::new();
    for &(name, n) in lands {
        result.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported(), "{name}");
        result.extend([c; 4]);
    }
    result
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let golgari = deck(
        &[("Forest", 12), ("Swamp", 12)],
        &[
            "Abyssal Gorestalker",
            "Beetle-Headed Merchants",
            "Comet Crawler",
            "Springbloom Druid",
            "Llanowar Elves",
            "Elfsworn Giant",
            "Rampaging Baloths",
            "Pick Your Poison",
            "Agents of HYDRA",
        ],
    );
    let izzet = deck(
        &[("Island", 12), ("Mountain", 12)],
        &[
            "Benthic Criminologists",
            "Discerning Peddler",
            "Yuyan Archers",
            "Incinerating Blast",
            "Melded Moxite",
            "Tweeze",
            "Dragon Fodder",
            "Nimble Thopterist",
            "Quick Study",
        ],
    );
    let mut decisions = 0;
    let mut sacrifices = 0;
    let mut discards = 0;
    let mut replays = 0;
    let mut searches = 0;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [golgari.clone(), izzet.clone()],
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
            let action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            sacrifices += usize::from(matches!(action, Action::Sacrifice { .. }));
            discards += usize::from(
                matches!(action, Action::Discard { .. })
                    && before
                        .effect_choice
                        .as_ref()
                        .is_some_and(|c| c.kind == EffectChoiceKind::Discard),
            );
            searches += usize::from(matches!(action, Action::FinishSearch));
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            if before.decision == "effect_choice"
                || after.decision == "effect_choice"
                || matches!(action, Action::ChooseSearch { .. } | Action::FinishSearch)
            {
                let expected = g.snapshot();
                g.restore(&saved).unwrap();
                let mut fork = g.clone();
                fork.apply(actor, action.clone())?;
                g.apply(actor, action.clone())?;
                assert_eq!(g.snapshot(), fork.snapshot());
                assert_eq!(g.snapshot(), expected);
                g.restore(&saved).unwrap();
                g.apply(actor, action)?;
                assert_eq!(g.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
        }
        if g.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    assert!(sacrifices > 0 && discards > 0 && replays > 0 && searches > 0);
    println!(
        "Completed {games} resolution-payment games, {decisions} decisions, {sacrifices} sacrifices, {discards} discard payments, {searches} searches, {replays} cloned/replayed choice boundaries; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
