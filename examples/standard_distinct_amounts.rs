//! Public supported Standard checks for characteristic unions and scaled effects.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, Observation, Player, SpellView, StackKind,
    Target, Zone,
};
use std::collections::{BTreeSet, HashSet};
fn deck(which: usize) -> Vec<Card> {
    let (lands, spells): (&[(&str, usize)], &[&str]) = match which {
        0 => (
            &[("Forest", 12), ("Mountain", 12)],
            &[
                "Explosive Prodigy",
                "Squawkroaster",
                "Peer Past the Veil",
                "Luminollusk",
                "Glister Bairn",
                "Bear Cub",
                "Three Tree Mascot",
                "Dragon Fodder",
                "Llanowar Elves",
            ],
        ),
        1 => (
            &[("Island", 12), ("Swamp", 12)],
            &[
                "Shinestriker",
                "Shimmercreep",
                "Clinquant Skymage",
                "Erudite Wizard",
                "Vampire Nighthawk",
                "Desperate Bloodseeker",
                "Burglar Rat",
                "Three Tree Mascot",
                "Vampire Spawn",
            ],
        ),
        _ => (
            &[("Plains", 12), ("Mountain", 12)],
            &[
                "Frontline Rush",
                "Explosive Prodigy",
                "Squawkroaster",
                "Goldvein Pick",
                "Dragon Fodder",
                "Hinterland Sanctifier",
                "Ajani's Pridemate",
                "Jet, Freedom Fighter",
                "Archway Angel",
            ],
        ),
    };
    let mut result = Vec::new();
    for &(name, n) in lands {
        result.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{name}");
        result.extend([c; 4]);
    }
    assert_eq!(result.len(), 60);
    result
}
fn colors(obs: &Observation, p: Player) -> i128 {
    let mut seen = BTreeSet::new();
    for o in obs
        .objects
        .iter()
        .filter(|o| o.owner == p && o.zone == Zone::Battlefield)
    {
        for color in 0..5 {
            if o.card.colors() & (1 << color) != 0 {
                seen.insert(color);
            }
        }
    }
    seen.len() as i128
}
fn check(before: &Observation, after: &Observation, item: &SpellView) -> usize {
    let p = item.controller;
    let i = p.index();
    let amount = colors(before, p);
    match (item.source_card.name(), item.kind) {
        ("Squawkroaster", StackKind::Spell) => {
            let o = after
                .objects
                .iter()
                .find(|o| o.id == item.card && o.zone == Zone::Battlefield)
                .unwrap();
            assert_eq!(o.power, colors(after, p));
            assert!(o.keywords.contains(Keywords::DOUBLE_STRIKE));
            2
        }
        ("Luminollusk", StackKind::TriggeredAbility { ability: 0 })
        | ("Shimmercreep", StackKind::TriggeredAbility { ability: 0 }) => {
            assert_eq!(after.players[i].life, before.players[i].life + amount);
            assert_eq!(
                after.players[i].life_gained_this_turn,
                before.players[i].life_gained_this_turn + amount as u128
            );
            assert_eq!(
                after.players[i].life_gain_events_this_turn,
                before.players[i].life_gain_events_this_turn + u64::from(amount > 0)
            );
            if item.source_card.name() == "Shimmercreep" {
                assert_eq!(
                    after.players[p.other().index()].life,
                    before.players[p.other().index()].life - amount
                );
                4
            } else {
                3
            }
        }
        ("Shinestriker", StackKind::TriggeredAbility { ability: 0 }) => {
            let actual = usize::try_from(amount)
                .unwrap()
                .min(before.players[i].library_count);
            assert_eq!(
                after.players[i].library_count,
                before.players[i].library_count - actual
            );
            assert_eq!(
                after.players[i].hand_count,
                before.players[i].hand_count + actual
            );
            assert_eq!(
                after.players[i].cards_drawn_this_turn,
                before.players[i].cards_drawn_this_turn + actual as u64
            );
            3
        }
        ("Peer Past the Veil", StackKind::Spell) => {
            let mut types = BTreeSet::new();
            for o in after.objects.iter().filter(|o| {
                o.owner == p && o.zone == Zone::Graveyard && !o.card.is_token() && o.id != item.card
            }) {
                for word in o
                    .card
                    .info()
                    .type_line
                    .split(" — ")
                    .next()
                    .unwrap()
                    .split_whitespace()
                {
                    if [
                        "Artifact",
                        "Battle",
                        "Creature",
                        "Enchantment",
                        "Instant",
                        "Kindred",
                        "Land",
                        "Planeswalker",
                        "Sorcery",
                    ]
                    .contains(&word)
                    {
                        types.insert(word);
                    }
                }
            }
            let actual = types.len().min(before.players[i].library_count);
            assert_eq!(
                after.players[i].library_count,
                before.players[i].library_count - actual
            );
            assert_eq!(after.players[i].hand_count, actual);
            assert_eq!(
                after.players[i].cards_drawn_this_turn,
                before.players[i].cards_drawn_this_turn + actual as u64
            );
            3
        }
        ("Glister Bairn", StackKind::TriggeredAbility { ability: 0 })
        | ("Frontline Rush", StackKind::Spell)
            if item.source_card.name() == "Glister Bairn" || item.mode == Some(1) =>
        {
            let Some(bound) = item.targets.first() else {
                return 0;
            };
            let Some(old) = before
                .objects
                .iter()
                .find(|o| Some(o.reference) == bound.object && o.zone == Zone::Battlefield)
            else {
                return 0;
            };
            let new = after
                .objects
                .iter()
                .find(|o| o.reference == old.reference)
                .unwrap();
            let amount = if item.source_card.name() == "Frontline Rush" {
                before
                    .objects
                    .iter()
                    .filter(|o| o.owner == p && o.zone == Zone::Battlefield && o.card.is_creature())
                    .count() as i128
            } else {
                amount
            };
            assert_eq!(new.power, old.power + amount);
            assert_eq!(new.toughness, old.toughness + amount);
            2
        }
        ("Explosive Prodigy", StackKind::TriggeredAbility { ability: 0 }) => {
            let Some(bound) = item.targets.first() else {
                return 0;
            };
            let Target::Card(id) = bound.target else {
                unreachable!();
            };
            let Some(old) = before.objects.iter().find(|o| {
                o.id == id && o.zone == Zone::Battlefield && Some(o.reference) == bound.object
            }) else {
                return 0;
            };
            if let Some(new) = after.objects.iter().find(|o| o.id == id) {
                if new.zone == Zone::Battlefield {
                    assert_eq!(new.damage, old.damage + amount);
                } else {
                    assert_eq!(new.zone, Zone::Graveyard);
                    assert!(old.damage + amount >= old.toughness);
                }
            } else {
                assert!(old.card.is_token());
                assert!(old.damage + amount >= old.toughness);
            }
            1
        }
        _ => 0,
    }
}
fn main() {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()
        .unwrap();
    assert!(games > 0);
    let mut decisions = 0;
    let mut checks = 0;
    let mut branches = 0;
    let mut max_bytes = 0;
    let mut batches = 0;
    let mut seen = BTreeSet::new();
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [deck(seed % 3), deck((seed + 1) % 3)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut checked = HashSet::new();
        for step in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "did not end naturally");
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            let mut actions = game.legal_actions();
            // Policy choices allow expensive entries to execute before combat ends games.
            actions.retain(|a| {
                !(before.turn < 18
                    && matches!(a, Action::Attack { .. } | Action::AttackPlaneswalker { .. }))
                    && !matches!(a, Action::Block { .. })
            });
            let index = if before
                .announcing
                .as_ref()
                .is_some_and(|a| a.source_card.name() == "Frontline Rush")
                && seed % 2 == 0
            {
                actions
                    .iter()
                    .position(|a| matches!(a, Action::ChooseMode { mode: 1 }))
                    .unwrap_or_else(|| rustmage::policy::smoke_index(&game, &actions))
            } else {
                rustmage::policy::smoke_index(&game, &actions)
            };
            let action = actions[index].clone();
            let saved = game.snapshot();
            let expected = if step % 4 == 0 {
                let mut fork = game.clone();
                fork.apply(actor, action.clone()).unwrap();
                branches += 1;
                Some(fork.snapshot())
            } else {
                None
            };
            game.apply(actor, action.clone()).unwrap();
            if let Some(expected) = expected {
                assert_eq!(game.snapshot(), expected);
                game.restore(&saved).unwrap();
                assert_eq!(
                    game.apply_index(actor, before.revision, 0),
                    Err(rustmage::Error::StaleRevision)
                );
                game.apply(actor, action.clone()).unwrap();
                assert_eq!(game.snapshot(), expected);
            }
            let after = game.observe(actor);
            let completed = before
                .resolving
                .as_ref()
                .filter(|item| after.resolving.as_ref().is_none_or(|a| a.id != item.id))
                .or_else(|| {
                    before.stack.last().filter(|item| {
                        after.stack.iter().all(|a| a.id != item.id)
                            && after.resolving.as_ref().is_none_or(|a| a.id != item.id)
                    })
                });
            if let Some(item) = completed
                && !checked.contains(&item.id)
            {
                let n = check(&before, &after, item);
                if n > 0 {
                    checks += n;
                    batches += 1;
                    checked.insert(item.id);
                    seen.insert((item.source_card.name(), format!("{:?}", item.kind)));
                }
            }
            decisions += 1;
            max_bytes = max_bytes.max(game.state_bytes());
        }
        assert!(game.outcome().is_some());
    }
    assert!(batches > 0);
    println!(
        "Completed {games} supported Standard distinct-amount games, {decisions} decisions, {checks} independently checked public facts, {batches} quantity effects, {branches} cloned/replayed boundaries, {max_bytes} max initialized state bytes; resolved {seen:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
}
