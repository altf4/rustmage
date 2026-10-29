//! In-process Standard games with temporary block legality and snapshot replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, Observation, Player, StackKind, Zone,
};
const NAMES: [&str; 6] = [
    "Brambleback Brute",
    "Sandstorm Verge",
    "Sower of Chaos",
    "Summit Intimidator",
    "War Squeak",
    "Zog, Triceraton Castaway",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn choose(g: &Game, o: &Observation, actions: &[Action]) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    let enemies = o
        .objects
        .iter()
        .filter(|b| {
            b.zone == Zone::Battlefield
                && b.owner != o.actor.unwrap()
                && b.card.is_creature()
                && !b.keywords.contains(Keywords::CANT_BLOCK)
        })
        .count();
    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 240,
            Action::Cast { .. } | Action::AnnounceCast { .. } => 220,
            Action::TapMana { .. } | Action::ActivateMana { .. } => {
                if o.decision == "pay_costs" {
                    170
                } else {
                    -100
                }
            }
            Action::AnnounceActivate { card, ability, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if index(b.card).is_some_and(|i| i < 3)
                    && b.zone == Zone::Battlefield
                    && *ability
                        == if b.card.name() == "Sandstorm Verge" {
                            1
                        } else {
                            0
                        }
                    && enemies > 0
                {
                    210
                } else {
                    -100
                }
            }
            Action::Activate { .. }
            | Action::BeginActivate { .. }
            | Action::AnnounceMana { .. } => -100,
            Action::ChooseTarget {
                target: rustmage::Target::Card(id),
            } => {
                let b = o.objects.iter().find(|b| b.id == *id).unwrap();
                let attachment = o.announcing.as_ref().is_some_and(|s| {
                    s.kind == StackKind::Spell && s.source().is_some_and(|source| source.card.name() == "War Squeak")
                });
                if attachment {
                    if b.owner == o.actor.unwrap() {
                        240
                    } else {
                        100
                    }
                } else if b.owner != o.actor.unwrap() {
                    230 - if b.keywords.contains(Keywords::CANT_BLOCK) {
                        100
                    } else {
                        0
                    }
                } else {
                    50
                }
            }
            Action::Attack { .. } if o.turn < 10 => -100,
            _ => {
                if *i == fallback {
                    150
                } else {
                    0
                }
            }
        })
        .unwrap()
        .0
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let mut deck = vec![Card::Mountain; 12];
    deck.extend([Card::Forest; 8]);
    for n in NAMES
        .into_iter()
        .chain(["Bear Cub", "Llanowar Elves", "Dragon Fodder", "Abrade"])
    {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        deck.extend([c; 4]);
    }
    assert_eq!(deck.len(), 60);
    let (mut decisions, mut replays, mut checks, mut rejected, mut expiries, mut max_bytes) =
        (0, 0, 0, 0, 0, 0);
    let (mut casts, mut effects) = ([0; 6], [0; 6]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck.clone(), deck.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let p = g.actor().unwrap();
            let before = g.observe(p);
            let actions = g.legal_actions();
            if before.decision == "blockers" {
                for blocker in before.objects.iter().filter(|b| {
                    b.zone == Zone::Battlefield
                        && b.owner == p
                        && b.card.is_creature()
                        && b.keywords.contains(Keywords::CANT_BLOCK)
                }) {
                    for &attacker in &before.attackers {
                        let illegal = Action::Block {
                            blocker: blocker.id,
                            attacker,
                        };
                        assert!(!actions.contains(&illegal));
                        checks += 1;
                        if decisions % 7 == 0 {
                            let saved = g.snapshot();
                            let r = g.revision();
                            assert_eq!(g.apply(p, illegal), Err(Error::IllegalAction));
                            assert_eq!(g.snapshot(), saved);
                            assert_eq!(g.revision(), r);
                            rejected += 1;
                        }
                    }
                }
            }
            let action = actions[choose(&g, &before, &actions)].clone();
            let boundary = before.announcing.is_some()
                || before.resolving.is_some()
                || before.stack.iter().any(|s| index(s.source_card).is_some());
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(p, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(p, action.clone())?;
            }
            let after = g.observe(p);
            for s in &after.stack {
                if s.kind == StackKind::Spell
                    && !before.stack.iter().any(|old| old.id == s.id)
                    && let Some(i) = index(s.source_card)
                {
                    casts[i] += 1;
                }
            }
            for old in before
                .objects
                .iter()
                .filter(|o| o.card.name() == "Sandstorm Verge" && o.zone == Zone::Hand)
            {
                if after
                    .objects
                    .iter()
                    .any(|o| o.id == old.id && o.zone == Zone::Battlefield)
                {
                    casts[1] += 1;
                }
            }
            for s in before.stack.iter().chain(before.resolving.iter()) {
                if s.kind != StackKind::Spell
                    && let Some(i) = index(s.source_card)
                    && !s.targets.is_empty()
                    && !after.stack.iter().any(|a| a.id == s.id)
                    && !after.resolving.as_ref().is_some_and(|a| a.id == s.id)
                    && s.targets.iter().any(|t| {
                        t.object.is_some_and(|r| {
                            after.objects.iter().any(|b| {
                                b.reference == r
                                    && b.zone == Zone::Battlefield
                                    && b.keywords.contains(Keywords::CANT_BLOCK)
                            })
                        })
                    })
                {
                    effects[i] += 1;
                }
            }
            if after.turn != before.turn {
                for b in before.objects.iter().filter(|b| {
                    b.zone == Zone::Battlefield && b.keywords.contains(Keywords::CANT_BLOCK)
                }) {
                    if let Some(now) = after
                        .objects
                        .iter()
                        .find(|o| o.reference == b.reference && o.zone == Zone::Battlefield)
                    {
                        assert!(!now.keywords.contains(Keywords::CANT_BLOCK));
                        expiries += 1;
                    }
                }
            }
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert_eq!(
                    g.apply_index(p, before.revision, 0),
                    Err(Error::StaleRevision)
                );
                g.apply(p, action)?;
                assert_eq!(g.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game {seed} unfinished");
    }
    if games >= 100 {
        assert!(casts.iter().all(|n| *n > 0), "casts {casts:?}");
        assert!(effects.iter().all(|n| *n > 0), "effects {effects:?}");
        assert!(checks > 0 && rejected > 0 && expiries > 0);
    }
    println!(
        "Completed {games} Standard temporary-blocking games, {decisions} decisions, casts/land plays {casts:?}, resolved effects {effects:?}, {checks} forbidden block pairs, {rejected} rejected illegal actions, {expiries} expiry checks, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
