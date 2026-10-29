//! Complete Standard decks exercising self-spell protection and owned counter decisions.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, StackKind, Target, Zone};
const NAMES: [&str; 4] = ["Eject", "Gigantic Big Bear", "Out Cold", "Long Goodbye"];
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let mut deck = Vec::new();
    for (name, count) in [("Island", 12), ("Swamp", 6), ("Forest", 4), ("Plains", 2)] {
        deck.extend(vec![Card::from_name(name).unwrap(); count]);
    }
    for name in NAMES.into_iter().chain([
        "Negate",
        "Spell Pierce",
        "Refute",
        "Spectral Snatcher",
        "Sleep-Cursed Faerie",
    ]) {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported());
        deck.extend([c; 4]);
    }
    assert_eq!(deck.len(), 60);
    let (mut decisions, mut replays, mut protected_checks, mut declined, mut max_bytes) =
        (0, 0, 0, 0, 0);
    let (mut resolutions, mut counters) = ([0; 4], [0; 3]);
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
            let mut actions = g.legal_actions();
            // Preserve Snatcher as a Ward permanent and allow seven-land games.
            actions.retain(|a|!matches!(a,Action::Activate{card,..}|Action::AnnounceActivate{card,..}|Action::BeginActivate{card,..}if before.objects.iter().any(|o|o.id==*card&&o.card.name()=="Spectral Snatcher")));
            if before.turn < 12 {
                actions.retain(|a| !matches!(a, Action::Attack { .. }));
            }
            let mut action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            if let Some(a)=actions.iter().find(|a|matches!(a,Action::Cast{target:Some(Target::Spell(id)),..}if before.stack.iter().any(|s|s.kind==StackKind::Spell&&s.card==*id&&s.controller!=p&&s.source_card.cannot_be_countered()))) {action=a.clone();}
            if seed % 2 == 0
                && let Some(a) = actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::DeclineResolutionMana | Action::DeclineEffectPayment
                    )
                })
            {
                action = a.clone();
                declined += 1;
            }
            // Capture exact stack associations before the counter finishes. Ward
            // abilities can have an owned resolution continuation instead of a stack row.
            let mut attempts = Vec::new();
            for s in before.stack.iter().chain(before.resolving.iter()) {
                let (victim, kind) = if let Some(link) = s.related_stack {
                    (Some(link.id), 2)
                } else {
                    (
                        s.targets.iter().find_map(|t| {
                            if let Target::Spell(id) = t.target {
                                before
                                    .stack
                                    .iter()
                                    .find(|v| v.card == id && v.kind == StackKind::Spell)
                                    .map(|v| v.id)
                            } else {
                                None
                            }
                        }),
                        usize::from(s.source().is_some_and(|source| source.card.name() == "Spell Pierce")),
                    )
                };
                if let Some(victim) = victim
                    && before
                        .stack
                        .iter()
                        .any(|v| v.id == victim && v.source_card.cannot_be_countered())
                {
                    attempts.push((s.id, victim, kind));
                }
            }
            let boundary =
                !attempts.is_empty() || before.announcing.is_some() || before.resolving.is_some();
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(p, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(p, action.clone())?;
            }
            let after = g.observe(p);
            for (counter, victim, kind) in attempts {
                let present = after.stack.iter().any(|s| s.id == counter)
                    || after.resolving.as_ref().is_some_and(|s| s.id == counter);
                if !present {
                    assert!(after.stack.iter().any(|s| s.id == victim));
                    counters[kind] += 1;
                    protected_checks += 1;
                }
            }
            for old in &before.objects {
                if old.zone != Zone::Stack {
                    continue;
                }
                if let Some(i) = NAMES.iter().position(|n| *n == old.card.name())
                    && let Some(new) = after.objects.iter().find(|o| o.id == old.id)
                    && new.zone != Zone::Stack
                {
                    assert_eq!(
                        new.zone,
                        if i == 1 {
                            Zone::Battlefield
                        } else {
                            Zone::Graveyard
                        }
                    );
                    resolutions[i] += 1;
                }
            }
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert!(g.apply_index(p, before.revision, 0).is_err());
                g.apply(p, action)?;
                assert_eq!(g.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game {seed} did not finish");
    }
    if games >= 100 {
        assert!(resolutions.iter().all(|n| *n > 0));
        assert!(
            counters.iter().all(|n| *n > 0),
            "counter paths {counters:?}"
        );
        assert!(declined > 0);
    }
    println!(
        "Completed {games} Standard uncounterable games, {decisions} decisions, resolutions {resolutions:?}, failed counter paths [direct,mana,ward] {counters:?}, {protected_checks} exact stack protection checks, {declined} declined payments, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
