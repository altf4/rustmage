//! Supported Standard Aura games, including noncasting linked returns.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, Target, Zone};
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
fn aura(card: Card) -> bool {
    card.info().type_line.ends_with(" — Aura")
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let selesnya = deck(
        &[("Plains", 12), ("Forest", 12)],
        &[
            "Healer's Hawk",
            "Llanowar Elves",
            "Frantic Strength",
            "Unflinching Courage",
            "Feather of Flight",
            "Colossification",
            "Adventuring Gear",
            "Banishing Light",
            "Disenchant",
        ],
    );
    let orzhov = deck(
        &[("Plains", 12), ("Swamp", 12)],
        &[
            "Healer's Hawk",
            "Voracious Vermin",
            "Dead Weight",
            "Twinblade Blessing",
            "Feather of Flight",
            "Untamed Hunger",
            "Banishing Light",
            "Disenchant",
            "Inspiring Overseer",
        ],
    );
    let (mut decisions, mut attachments, mut entries, mut departures, mut replays, mut max_bytes) =
        (0, 0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [selesnya.clone(), orzhov.clone()],
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
            // Exercise relevant legal removal targets using only the acting
            // player's observation, while retaining the shared smoke policy elsewhere.
            let preferred = before.announcing.as_ref().and_then(|source| {
                actions.iter().position(|a| match a {
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    } if source.source().is_some_and(|source| source.card.name() == "Banishing Light") => {
                        before.objects.iter().any(|o| o.id == *id && aura(o.card))
                    }
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    } if source.source().is_some_and(|source| source.card.name() == "Disenchant") => {
                        before.exile_links.iter().any(|l| {
                            l.source.card == *id
                                && before
                                    .objects
                                    .iter()
                                    .any(|o| o.id == l.exiled.card && aura(o.card))
                        })
                    }
                    _ => false,
                })
            });
            let action = actions
                [preferred.unwrap_or_else(|| rustmage::policy::smoke_index(&g, &actions))]
            .clone();
            entries += usize::from(matches!(action, Action::ChooseAuraAttachment { .. }));
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            let attached = after
                .objects
                .iter()
                .filter(|o| {
                    aura(o.card)
                        && o.zone == Zone::Battlefield
                        && o.attached_to.is_some()
                        && before
                            .objects
                            .iter()
                            .find(|old| old.id == o.id)
                            .is_none_or(|old| old.attached_to != o.attached_to)
                })
                .count();
            attachments += attached;
            departures += after
                .objects
                .iter()
                .filter(|o| {
                    aura(o.card)
                        && o.zone == Zone::Graveyard
                        && before
                            .objects
                            .iter()
                            .any(|old| old.id == o.id && old.zone == Zone::Battlefield)
                })
                .count();
            if attached > 0
                || before.decision == "aura_entry"
                || after.decision == "aura_entry"
                || before
                    .announcing
                    .as_ref()
                    .is_some_and(|s| aura(s.source_card))
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
            max_bytes = max_bytes.max(g.state_bytes());
            decisions += 1;
        }
        if g.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    assert!(attachments > 0 && entries > 0 && departures > 0 && replays > 0);
    println!(
        "Completed {games} Aura games, {decisions} decisions, {attachments} Aura attachments, {entries} noncasting attachment choices, {departures} Aura battlefield-to-graveyard changes, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
