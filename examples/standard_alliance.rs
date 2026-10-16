//! Complete Standard games with independently modeled Alliance effects and replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, Observation, Player,
    SpellView, StackKind, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 6] = [
    "EPF Point Squad",
    "East Wind Avatar",
    "Mighty Mutanimals",
    "Mutant Town Musicians",
    "Raphael, Tough Turtle",
    "Slash, Reptile Rampager",
];
const BASE: [(i128, i128); 6] = [(2, 1), (2, 4), (2, 1), (2, 4), (1, 3), (7, 5)];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck() -> Vec<Card> {
    let mut d = Vec::new();
    for (n, count) in [("Plains", 12), ("Mountain", 12)].into_iter().chain(
        NAMES
            .into_iter()
            .chain(["Helpful Hunter", "Inspiring Overseer", "Shock"])
            .map(|n| (n, 4)),
    ) {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal(), "{n}");
        d.extend(vec![c; count]);
    }
    assert_eq!(d.len(), 60);
    d
}
type Ledger = BTreeMap<ObjectRef, (u32, i128)>;
fn check(o: &Observation, ledger: &Ledger) -> u64 {
    let mut checks = 0;
    for v in o
        .objects
        .iter()
        .filter(|v| v.zone == Zone::Battlefield && v.card.is_creature())
    {
        let (base, k) = if let Some(i) = index(v.card) {
            (
                BASE[i],
                match i {
                    1 => Keywords::FLYING.union(Keywords::VIGILANCE),
                    3 => Keywords::TRAMPLE,
                    _ => Keywords::NONE,
                },
            )
        } else {
            match v.card.name() {
                "Helpful Hunter" => ((1, 1), Keywords::NONE),
                "Inspiring Overseer" => ((2, 1), Keywords::FLYING),
                _ => {
                    assert_eq!(v.token.unwrap().subtypes, "Mutant");
                    ((2, 2), Keywords::NONE)
                }
            }
        };
        let growth = ledger
            .get(&v.reference)
            .filter(|v| v.0 == o.turn)
            .map_or(0, |v| v.1);
        let counters = i128::from(v.counters.plus_one) - i128::from(v.counters.minus_one);
        assert_eq!(
            (v.power, v.toughness),
            (base.0 + growth + counters, base.1 + counters),
            "{}",
            v.card.name()
        );
        assert_eq!(v.keywords, k, "{}", v.card.name());
        checks += 1;
    }
    checks
}
fn tokens(o: &Observation, p: Player) -> usize {
    o.objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.token.is_some_and(|t| t.subtypes == "Mutant")
        })
        .count()
}
fn verify(
    before: &Observation,
    after: &Observation,
    root: &SpellView,
    i: usize,
    ledger: &mut Ledger,
) -> bool {
    let StackKind::TriggeredAbility { ability } = root.kind else {
        return false;
    };
    let token = (i == 2 && ability == 0) || (i == 5 && ability == 1);
    if token {
        assert_eq!(
            tokens(after, root.controller),
            tokens(before, root.controller) + 1
        );
        return true;
    }
    let expected_ability = u16::from(i == 2);
    assert_eq!(ability, expected_ability);
    let source = before
        .objects
        .iter()
        .find(|v| v.reference == root.object && v.zone == Zone::Battlefield);
    if i == 0 || i == 2 {
        let target = if i == 0 {
            source
        } else {
            root.targets.first().and_then(|b| {
                before.objects.iter().find(|v| {
                    Some(v.reference) == b.object
                        && v.zone == Zone::Battlefield
                        && v.owner == root.controller
                        && v.card.is_creature()
                })
            })
        };
        if let Some(b) = target {
            let a = after
                .objects
                .iter()
                .find(|v| v.reference == b.reference)
                .unwrap();
            assert_eq!(a.counters.plus_one, b.counters.plus_one + 1);
        }
    } else if i == 1 || i == 3 {
        if let Some(b) = source {
            let item = ledger.entry(b.reference).or_insert((after.turn, 0));
            if item.0 != after.turn {
                *item = (after.turn, 0);
            }
            item.1 += 1;
            let a = after
                .objects
                .iter()
                .find(|v| v.reference == b.reference)
                .unwrap();
            assert_eq!((a.power, a.toughness), (b.power + 1, b.toughness));
        }
    } else {
        let opponent = root.controller.other();
        let loss = if i == 4 {
            assert_eq!(root.targets[0].target, rustmage::Target::Player(opponent));
            1
        } else {
            2
        };
        assert_eq!(
            after.players[opponent.index()].life,
            before.players[opponent.index()].life - loss
        );
        assert_eq!(
            after.players[root.controller.index()].life,
            before.players[root.controller.index()].life
        );
    }
    false
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let (mut decisions, mut replays, mut pairs, mut max_bytes) = (0, 0, 0, 0);
    let (mut casts, mut triggers, mut resolved, mut token_checks) =
        ([0u64; 6], [0u64; 6], [0u64; 6], [0u64; 6]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut ledger = Ledger::new();
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            let boundary = before.announcing.is_some()
                || before.resolving.is_some()
                || before.stack.iter().any(|s| index(s.source_card).is_some());
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(actor, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            let after = g.observe(actor);
            for s in &after.stack {
                if before.stack.iter().any(|b| b.id == s.id)
                    || before.resolving.as_ref().is_some_and(|b| b.id == s.id)
                {
                    continue;
                }
                if let Some(i) = index(s.source_card) {
                    if s.kind == StackKind::Spell {
                        casts[i] += 1;
                    } else if matches!(s.kind, StackKind::TriggeredAbility { .. }) {
                        triggers[i] += 1;
                    }
                }
            }
            if matches!(action, Action::Pass)
                && let Some(root) = before.stack.last()
                && let Some(i) = index(root.source_card)
                && matches!(root.kind, StackKind::TriggeredAbility { .. })
                && !after.stack.iter().any(|s| s.id == root.id)
                && after.outcome.is_none()
            {
                if verify(&before, &after, root, i, &mut ledger) {
                    token_checks[i] += 1;
                } else {
                    resolved[i] += 1;
                }
            }
            pairs += check(&after, &ledger);
            check(&g.observe(actor.other()), &ledger);
            assert_eq!(after.schema, 51);
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert_eq!(
                    g.apply_index(actor, before.revision, 0),
                    Err(Error::StaleRevision)
                );
                g.apply(actor, action)?;
                assert_eq!(g.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game {seed} unfinished");
    }
    if games >= 100 {
        assert!(casts.iter().all(|n| *n > 0));
        assert!(resolved.iter().all(|n| *n > 0));
        assert!(token_checks[2] > 0 && token_checks[5] > 0);
    }
    println!(
        "Completed {games} Standard Alliance games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, verified Alliance effects {resolved:?}, verified token effects {token_checks:?}, {pairs} independently checked public stat/ability pairs, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
