//! Public Standard checks for guarded effects, trigger timing and focused tails.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, Observation, Player, SpellView, StackKind,
    Zone,
};
use std::collections::{BTreeSet, HashMap, HashSet};
fn deck(which: usize) -> Vec<Card> {
    let (lands, spells): (&[(&str, usize)], &[&str]) = match which {
        0 => (
            &[("Forest", 12), ("Plains", 12)],
            &[
                "Dwynen's Elite",
                "Nasty Little Rabbit",
                "Nessian Hornbeetle",
                "Snakeskin Veil",
                "Hero in Training",
                "Rabbit Response",
                "You're Not Alone",
                "Rosa, Resolute White Mage",
                "Warg Tactics",
            ],
        ),
        1 => (
            &[("Island", 10), ("Plains", 10), ("Creosote Heath", 4)],
            &[
                "Donatello, Turtle Techie",
                "Failed Fording",
                "Moment of Valor",
                "Spectacular Tactics",
                "Three Tree Mascot",
                "Goldvein Pick",
                "Clinquant Skymage",
                "Erudite Wizard",
                "Cloudblazer",
            ],
        ),
        2 => (
            &[("Swamp", 12), ("Mountain", 12)],
            &[
                "Crypt Feaster",
                "Stinging Cave Crawler",
                "Taken by Nightmares",
                "Violent Urge",
                "Take Out the Trash",
                "Appendage Amalgam",
                "Three Tree Mascot",
                "Raging Redcap",
                "Murder",
            ],
        ),
        _ => (
            &[("Forest", 12), ("Swamp", 12)],
            &[
                "Malamet Veteran",
                "Stinging Cave Crawler",
                "Crypt Feaster",
                "Three Tree Mascot",
                "Bear Cub",
                "Murder",
                "Grasping Longneck",
                "Llanowar Elves",
                "Corrupted Conviction",
            ],
        ),
    };
    let mut result = Vec::new();
    for &(name, n) in lands {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{name}");
        result.extend(vec![c; n]);
    }
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{name}");
        result.extend([c; 4]);
    }
    assert_eq!(result.len(), 60);
    result
}
fn bf(obs: &Observation, p: Player) -> impl Iterator<Item = &rustmage::ObjectView> {
    obs.objects
        .iter()
        .filter(move |o| o.owner == p && o.zone == Zone::Battlefield)
}
fn grave(obs: &Observation, p: Player, permanent: bool) -> usize {
    obs.objects
        .iter()
        .filter(|o| {
            o.owner == p
                && o.zone == Zone::Graveyard
                && !o.card.is_token()
                && (!permanent
                    || o.card
                        .info()
                        .type_line
                        .split(" — ")
                        .next()
                        .unwrap()
                        .split_whitespace()
                        .any(|w| {
                            [
                                "Artifact",
                                "Battle",
                                "Creature",
                                "Enchantment",
                                "Land",
                                "Planeswalker",
                            ]
                            .contains(&w)
                        }))
        })
        .count()
}
fn delirium(obs: &Observation, p: Player) -> bool {
    let mut types = BTreeSet::new();
    for o in obs
        .objects
        .iter()
        .filter(|o| o.owner == p && o.zone == Zone::Graveyard && !o.card.is_token())
    {
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
    types.len() >= 4
}
fn check(before: &Observation, after: &Observation, item: &SpellView) -> (usize, bool) {
    let p = item.controller;
    let i = p.index();
    let n = item.source_card.name();
    let trigger = matches!(item.kind, StackKind::TriggeredAbility { .. });
    let own_source = before
        .objects
        .iter()
        .find(|o| o.reference == item.object && o.zone == Zone::Battlefield);
    let bound = item.targets.first();
    let target = bound.and_then(|b| {
        before.objects.iter().find(|o| {
            Some(o.reference) == b.object
                && o.zone == Zone::Battlefield
                && !(o.owner != p && o.keywords.contains(Keywords::HEXPROOF))
        })
    });
    if bound.is_some() && target.is_none() {
        return (0, false);
    }
    let new_target = target.and_then(|old| after.objects.iter().find(|o| o.id == old.id));
    let draws = |expected: usize| {
        let actual = expected.min(before.players[i].library_count);
        assert_eq!(
            after.players[i].library_count,
            before.players[i].library_count - actual
        );
        assert_eq!(
            after.players[i].cards_drawn_this_turn,
            before.players[i].cards_drawn_this_turn + actual as u64
        );
    };
    let counter = |amount: u64| {
        let old = target.or(own_source).unwrap();
        let new = after
            .objects
            .iter()
            .find(|o| o.reference == old.reference)
            .unwrap();
        assert_eq!(new.counters.plus_one, old.counters.plus_one + amount);
    };
    match (n, trigger) {
        ("Donatello, Turtle Techie", true) => {
            let yes = bf(before, p).any(|o| o.card.is_artifact());
            draws(usize::from(yes));
            (2, yes)
        }
        ("Dwynen's Elite", true) => {
            let yes =
                bf(before, p).any(|o| o.reference != item.object && o.card.has_subtype("Elf"));
            let count = |obs: &Observation| {
                bf(obs, p)
                    .filter(|o| o.card.is_token() && o.card.has_subtype("Elf"))
                    .count()
            };
            assert_eq!(count(after), count(before) + usize::from(yes));
            (1, yes)
        }
        ("Hero in Training", true) => {
            draws(1);
            let yes =
                bf(before, p).any(|o| o.reference != item.object && o.card.has_subtype("Hero"));
            assert_eq!(
                after.players[i].life,
                before.players[i].life + if yes { 2 } else { 0 }
            );
            (3, yes)
        }
        ("Crypt Feaster", true) => {
            let yes = grave(before, p, false) >= 7;
            if let Some(old) = own_source {
                let new = after
                    .objects
                    .iter()
                    .find(|o| o.reference == old.reference)
                    .unwrap();
                assert_eq!(new.power, old.power + if yes { 2 } else { 0 });
                (1, yes)
            } else {
                (0, yes)
            }
        }
        ("Stinging Cave Crawler", true) => {
            let yes = grave(before, p, true) >= 4;
            draws(usize::from(yes));
            assert_eq!(
                after.players[i].life,
                before.players[i].life - i128::from(yes)
            );
            (3, yes)
        }
        ("Malamet Veteran", true) => {
            let yes = grave(before, p, true) >= 4;
            counter(u64::from(yes));
            (1, yes)
        }
        ("Nasty Little Rabbit", true) | ("Nessian Hornbeetle", true) => {
            if own_source.is_none() {
                return (0, false);
            }
            let yes = bf(before, p).any(|o| {
                o.card.is_creature()
                    && (n == "Nasty Little Rabbit" || o.reference != item.object)
                    && o.power >= 4
            });
            counter(u64::from(yes));
            (1, yes)
        }
        ("Rosa, Resolute White Mage", true) => {
            counter(1);
            assert!(new_target.unwrap().keywords.contains(Keywords::LIFELINK));
            (2, true)
        }
        ("Rabbit Response", false) => {
            let mut facts = 0;
            for old in bf(before, p).filter(|o| o.card.is_creature()) {
                let new = after
                    .objects
                    .iter()
                    .find(|o| o.reference == old.reference)
                    .unwrap();
                assert_eq!(new.power, old.power + 2);
                assert_eq!(new.toughness, old.toughness + 1);
                facts += 2;
            }
            (facts, bf(before, p).any(|o| o.card.has_subtype("Rabbit")))
        }
        ("Failed Fording", false) => {
            let old = target.unwrap();
            assert!(
                !after
                    .objects
                    .iter()
                    .any(|o| o.reference == old.reference && o.zone == Zone::Battlefield)
            );
            assert_eq!(
                after.players[old.owner.index()].hand_count,
                before.players[old.owner.index()].hand_count + 1
            );
            (2, bf(before, p).any(|o| o.card.has_subtype("Desert")))
        }
        ("Taken by Nightmares", false) => {
            let old = target.unwrap();
            assert_eq!(new_target.unwrap().zone, Zone::Exile);
            (
                1,
                bf(before, p).any(|o| {
                    o.card
                        .info()
                        .type_line
                        .split(" — ")
                        .next()
                        .unwrap()
                        .split_whitespace()
                        .any(|w| w == "Enchantment")
                        && o.reference != old.reference
                }),
            )
        }
        ("Violent Urge", false) => {
            let old = target.unwrap();
            let new = new_target.unwrap();
            let yes = delirium(before, p);
            assert_eq!(new.power, old.power + 1);
            assert!(new.keywords.contains(Keywords::FIRST_STRIKE));
            assert_eq!(
                new.keywords.contains(Keywords::DOUBLE_STRIKE),
                old.keywords.contains(Keywords::DOUBLE_STRIKE) || yes
            );
            (3, yes)
        }
        ("You're Not Alone", false) => {
            let old = target.unwrap();
            let new = new_target.unwrap();
            let yes = bf(before, p).filter(|o| o.card.is_creature()).count() >= 3;
            let bonus = if yes { 4 } else { 2 };
            assert_eq!(new.power, old.power + bonus);
            assert_eq!(new.toughness, old.toughness + bonus);
            (2, yes)
        }
        ("Take Out the Trash", false) => {
            let old = target.unwrap();
            let yes = bf(before, p).any(|o| o.card.has_subtype("Raccoon"));
            if let Some(new) = new_target {
                if old.card.is_planeswalker() {
                    assert_eq!(new.counters.loyalty, old.counters.loyalty.saturating_sub(3));
                } else if new.zone == Zone::Battlefield {
                    assert_eq!(new.damage, old.damage + 3);
                } else {
                    assert_eq!(new.zone, Zone::Graveyard);
                    assert!(old.damage + 3 >= old.toughness);
                }
            } else {
                assert!(old.card.is_token());
                assert!(old.damage + 3 >= old.toughness);
            }
            let drawn = before.players[i].library_count - after.players[i].library_count;
            assert!(drawn <= usize::from(yes));
            assert_eq!(
                after.players[i].cards_drawn_this_turn,
                before.players[i].cards_drawn_this_turn + drawn as u64
            );
            (3, yes)
        }
        ("Snakeskin Veil", false)
        | ("Moment of Valor", false)
        | ("Spectacular Tactics", false)
        | ("Warg Tactics", false) => {
            let positive =
                n == "Snakeskin Veil" || item.mode == Some(if n == "Warg Tactics" { 1 } else { 0 });
            let old = target.unwrap();
            if positive {
                let new = new_target.unwrap();
                if n == "Moment of Valor" {
                    assert_eq!(new.power, old.power + 1);
                    assert!(new.keywords.contains(Keywords::INDESTRUCTIBLE));
                    assert!(!new.tapped);
                } else {
                    counter(1);
                    assert!(new.keywords.contains(Keywords::HEXPROOF));
                    if n == "Warg Tactics" {
                        assert!(new.keywords.contains(Keywords::TRAMPLE));
                    }
                }
            } else if let Some(new) = new_target {
                assert_eq!(
                    new.zone,
                    if old.keywords.contains(Keywords::INDESTRUCTIBLE) {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
            } else {
                assert!(old.card.is_token());
                assert!(!old.keywords.contains(Keywords::INDESTRUCTIBLE));
            }
            (if positive { 2 } else { 1 }, true)
        }
        _ => (0, false),
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
            decks: [deck(seed % 4), deck((seed + 1) % 4)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut checked = HashSet::new();
        let mut origins = HashMap::new();
        for step in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "did not end naturally");
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            let mut actions = game.legal_actions();
            // Delay combat until expensive conditional bodies can enter; blocks remain legal policy choices.
            actions.retain(|a| {
                !(before.turn < 12
                    && matches!(a, Action::Attack { .. } | Action::AttackPlaneswalker { .. }))
            });
            let index = actions
                .iter()
                .position(|a| matches!(a,Action::ChooseMode{mode}if usize::from(*mode)==seed%2))
                .unwrap_or_else(|| rustmage::policy::smoke_index(&game, &actions));
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
            if let Some(item) = before.stack.last()
                && after.stack.iter().all(|a| a.id != item.id)
            {
                origins.entry(item.id).or_insert_with(|| before.clone());
            }
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
                let origin = origins.remove(&item.id).unwrap_or_else(|| before.clone());
                let (n, active) = check(&origin, &after, item);
                if n > 0 {
                    checks += n;
                    batches += 1;
                    checked.insert(item.id);
                    seen.insert((
                        item.source_card.name(),
                        format!("{:?}/{:?}/{active}", item.kind, item.mode),
                    ));
                }
            }
            decisions += 1;
            max_bytes = max_bytes.max(game.state_bytes());
        }
        assert!(game.outcome().is_some());
    }
    assert!(batches > 0);
    println!(
        "Completed {games} supported Standard conditional-rule games, {decisions} decisions, {checks} independently checked public facts, {batches} rule resolutions, {branches} cloned/replayed boundaries, {max_bytes} max initialized state bytes; resolved {seen:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
}
