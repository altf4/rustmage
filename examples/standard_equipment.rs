//! Supported Standard Equipment self-play with branch replay at attachment boundaries.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, StackKind};
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
    let boros = deck(
        &[("Mountain", 12), ("Plains", 12)],
        &[
            "Short Bow",
            "Basilisk Collar",
            "Sword of Vengeance",
            "Fireshrieker",
            "Coral Sword",
            "Swiftfoot Boots",
            "Brightblade Stoat",
            "Healer's Hawk",
            "Raging Redcap",
        ],
    );
    let green = deck(
        &[("Forest", 24)],
        &[
            "Short Bow",
            "Swiftfoot Boots",
            "Vibranium Energy Daggers",
            "Gold Pan",
            "Glimmerlight",
            "Llanowar Elves",
            "Druid of the Cowl",
            "Elfsworn Giant",
            "Rampaging Baloths",
        ],
    );
    let (mut decisions, mut attachments, mut activations, mut replays) = (0, 0, 0, 0);
    let mut maximum_bytes = 0;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [boros.clone(), green.clone()],
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
            let equipment_activation = match action {
                Action::Activate { card, .. } => before
                    .objects
                    .iter()
                    .any(|o| o.id == card && o.card.info().type_line.ends_with(" — Equipment")),
                Action::PayCosts { .. } => before.announcing.as_ref().is_some_and(|s| {
                    matches!(s.kind, StackKind::ActivatedAbility { .. })
                        && s.source_card.info().type_line.ends_with(" — Equipment")
                }),
                _ => false,
            };
            activations += usize::from(equipment_activation);
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            let attached = after
                .objects
                .iter()
                .filter(|o| {
                    o.attached_to.is_some()
                        && before
                            .objects
                            .iter()
                            .find(|old| old.id == o.id)
                            .is_none_or(|old| old.attached_to != o.attached_to)
                })
                .count();
            attachments += attached;
            if attached > 0
                || equipment_activation
                || matches!(action, Action::ChooseTarget { .. })
                    && before
                        .announcing
                        .as_ref()
                        .is_some_and(|s| s.source_card.info().type_line.ends_with(" — Equipment"))
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
            maximum_bytes = maximum_bytes.max(g.state_bytes());
            decisions += 1;
        }
        if g.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    assert!(attachments > 0 && activations > 0 && replays > 0);
    println!(
        "Completed {games} Equipment games, {decisions} decisions, {attachments} attachment changes, {activations} paid Equipment activations, {replays} cloned/replayed boundaries, {maximum_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
