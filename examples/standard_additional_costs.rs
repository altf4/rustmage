//! Native Standard mandatory additional costs and complete payment replay.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, Zone};
fn deck() -> Vec<Card> {
    let mut cards = Vec::new();
    for (name, count) in [
        ("Swamp", 9),
        ("Mountain", 6),
        ("Forest", 5),
        ("Blood Crypt", 4),
        ("Llanowar Elves", 4),
        ("Bear Cub", 4),
        ("Corrupted Conviction", 4),
        ("Final Vengeance", 4),
        ("Thrill of Possibility", 4),
        ("Marauding Mako", 4),
        ("Shock", 4),
        ("Giant Growth", 4),
        ("Tarmogoyf", 4),
    ] {
        let card = Card::from_name(name).unwrap();
        assert!(card.is_supported() && card.is_standard_legal(), "{name}");
        cards.extend(vec![card; count]);
    }
    assert_eq!(cards.len(), 60);
    cards
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let (mut decisions, mut selections, mut paid, mut replays, mut max_bytes) = (0, 0, 0, 0, 0);
    let mut kinds = std::collections::BTreeSet::new();
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            if let Action::ChooseAdditionalCost { object } = action {
                selections += 1;
                kinds.insert(before.announcing.as_ref().unwrap().source().expect("example expects an object-backed packet").card.name());
                assert!(
                    before
                        .additional_cost
                        .as_ref()
                        .unwrap()
                        .candidates
                        .contains(&object)
                );
                assert!(g.observe(actor.other()).additional_cost.is_none());
            }
            let selected = if matches!(action, Action::PayCosts { .. }) {
                before.additional_cost.as_ref().and_then(|c| c.selected)
            } else {
                None
            };
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            if let Some(object) = selected {
                paid += 1;
                let view = g.observe(actor);
                assert!(
                    view.objects
                        .iter()
                        .any(|o| o.id == object.card && o.zone == Zone::Graveyard)
                );
            }
            if before.announcing.is_some() || before.entry_life.is_some() {
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
        assert_eq!(kinds.len(), 3);
        assert!(paid > 0 && replays > 0);
    }
    println!(
        "Completed {games} supported Standard additional-cost games, {decisions} decisions, {selections} selections across {kinds:?}, {paid} payments, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
