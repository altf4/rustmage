//! Public Standard workload for resolution-time damage and life quantities.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Keywords, Observation, Player, SpellView, StackKind,
    Target, Zone,
};
use std::collections::{BTreeSet, HashSet};
fn deck(which: usize) -> Vec<Card> {
    let (lands, spells): (&[(&str, usize)], &[&str]) = match which {
        0 => (
            &[
                ("Plains", 12),
                ("Azorius Guildgate", 4),
                ("Boros Guildgate", 4),
                ("Selesnya Guildgate", 4),
            ],
            &[
                "Archway Angel",
                "Coordinated Maneuver",
                "Slash of Light",
                "Jet, Freedom Fighter",
                "Goldvein Pick",
                "Hinterland Sanctifier",
                "Brightblade Stoat",
                "Ajani's Pridemate",
                "Clachan Festival",
            ],
        ),
        1 => (
            &[("Mountain", 24)],
            &[
                "Smaug the Magnificent",
                "Volley Veteran",
                "Rumbling Rockslide",
                "Krenko, Mob Boss",
                "Dragon Fodder",
                "Elder Auntie",
                "Redcap Thief",
                "Raging Redcap",
                "Fire Elemental",
            ],
        ),
        _ => (
            &[("Swamp", 24)],
            &[
                "Cat-Gator",
                "Vampire Nighthawk",
                "Vampire Interloper",
                "Highborn Vampire",
                "Yargle, Glutton of Urborg",
                "Infestation Sage",
                "Mintstrosity",
                "Vault Plunderer",
                "Graveshifter",
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
fn count(obs: &Observation, p: Player, filter: impl Fn(&rustmage::ObjectView) -> bool) -> i128 {
    obs.objects
        .iter()
        .filter(|o| o.owner == p && o.zone == Zone::Battlefield && filter(o))
        .count() as i128
}
fn check(before: &Observation, after: &Observation, item: &SpellView) -> usize {
    let p = item.controller;
    let amount = match (item.source().expect("example expects an object-backed packet").card.name(), item.kind) {
        ("Archway Angel", StackKind::TriggeredAbility { ability: 0 }) => {
            let amount = 2 * count(before, p, |o| o.card.has_subtype("Gate"));
            assert_eq!(
                after.players[p.index()].life,
                before.players[p.index()].life + amount
            );
            assert_eq!(
                after.players[p.index()].life_gained_this_turn,
                before.players[p.index()].life_gained_this_turn + amount as u128
            );
            assert_eq!(
                after.players[p.index()].life_gain_events_this_turn,
                before.players[p.index()].life_gain_events_this_turn + u64::from(amount > 0)
            );
            return 3;
        }
        ("Cat-Gator", StackKind::TriggeredAbility { ability: 0 }) => {
            count(before, p, |o| o.card.has_subtype("Swamp"))
        }
        ("Volley Veteran", StackKind::TriggeredAbility { ability: 0 }) => {
            count(before, p, |o| o.card.has_subtype("Goblin"))
        }
        ("Jet, Freedom Fighter", StackKind::TriggeredAbility { ability: 0 }) => {
            count(before, p, |o| o.card.is_creature())
        }
        ("Smaug the Magnificent", StackKind::TriggeredAbility { ability: 0 }) => {
            count(before, p, |o| o.card.has_subtype("Treasure"))
        }
        ("Rumbling Rockslide", StackKind::Spell) => count(before, p, |o| o.card.is_land()),
        ("Slash of Light", StackKind::Spell) => {
            count(before, p, |o| o.card.is_creature())
                + count(before, p, |o| o.card.has_subtype("Equipment"))
        }
        ("Coordinated Maneuver", StackKind::Spell) if item.mode == Some(0) => {
            count(before, p, |o| o.card.is_creature())
        }
        _ => return 0,
    };
    let Some(bound) = item.targets.first() else {
        return 0;
    };
    let live = match bound.target {
        Target::Card(id) => before.objects.iter().any(|o| {
            o.id == id && o.zone == Zone::Battlefield && Some(o.reference) == bound.object
        }),
        Target::Player(_) => true,
        Target::Spell(_) | Target::Stack(_) => false,
    };
    if !live {
        return 0;
    }
    let mut life = [before.players[0].life, before.players[1].life];
    let mut checks = 0;
    match bound.target {
        Target::Player(player) => life[player.index()] -= amount,
        Target::Card(id) => {
            let old = before.objects.iter().find(|o| o.id == id).unwrap();
            if let Some(new) = after.objects.iter().find(|o| o.id == id) {
                if old.card.is_planeswalker() {
                    assert_eq!(
                        new.counters.loyalty,
                        old.counters
                            .loyalty
                            .saturating_sub(u64::try_from(amount).unwrap())
                    );
                } else if new.zone == Zone::Battlefield {
                    assert_eq!(new.damage, old.damage + amount);
                } else {
                    assert_eq!(new.zone, Zone::Graveyard);
                    assert!(old.damage + amount >= old.toughness);
                }
            } else {
                assert!(old.card.is_token());
                assert!(old.damage + amount >= old.toughness);
            }
            checks += 1;
        }
        _ => unreachable!(),
    }
    let keywords = before
        .objects
        .iter()
        .find(|o| o.reference == item.object)
        .map_or(item.source_keywords, |o| o.keywords);
    if keywords.contains(Keywords::LIFELINK) {
        life[p.index()] += amount;
    }
    for (i, expected) in life.into_iter().enumerate() {
        assert_eq!(after.players[i].life, expected);
        checks += 1;
    }
    checks
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
            let creatures = count(&before, actor, |o| {
                o.zone == Zone::Battlefield && o.card.is_creature()
            });
            actions.retain(|a| {
                if before.turn < 16
                    && matches!(a, Action::Attack { .. } | Action::AttackPlaneswalker { .. })
                {
                    return false;
                }
                if matches!(a, Action::Block { .. }) {
                    return false;
                }
                if let Action::Activate { card, ability, .. }
                | Action::BeginActivate { card, ability, .. }
                | Action::AnnounceActivate { card, ability, .. } = a
                {
                    let o = before.objects.iter().find(|o| o.id == *card).unwrap();
                    if ["Krenko, Mob Boss", "Clachan Festival"].contains(&o.card.name())
                        && (creatures >= 16 || o.used_abilities & (1 << ability) != 0)
                    {
                        return false;
                    }
                }
                true
            });
            let index = rustmage::policy::smoke_index(&game, &actions);
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
            if matches!(action, Action::Pass)
                && let Some(item) = before.stack.last()
                && after.stack.iter().all(|i| i.id != item.id)
                && checked.insert(item.id)
            {
                let n = check(&before, &after, item);
                if n > 0 {
                    checks += n;
                    batches += 1;
                    seen.insert((item.source().expect("example expects an object-backed packet").card.name(), format!("{:?}", item.kind)));
                }
            }
            decisions += 1;
            max_bytes = max_bytes.max(game.state_bytes());
        }
        assert!(game.outcome().is_some());
    }
    assert!(batches > 0);
    println!(
        "Completed {games} supported Standard scaled-effect games, {decisions} decisions, {checks} independently checked public facts, {batches} scaled effects, {branches} cloned/replayed boundaries, {max_bytes} max initialized state bytes; resolved {seen:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
}
