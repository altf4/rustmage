//! Full Standard games with independent fixed-effect witnesses and native replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectView, Observation, Player,
    SpellView, StackKind, Target, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 12] = [
    "Callous Inspector",
    "Cosmium Blast",
    "Deadly Riposte",
    "Defibrillating Current",
    "Fear of Lost Teeth",
    "Joust Through",
    "Keep Out",
    "Lightning Helix",
    "Ravenous Giant",
    "Risky Shortcut",
    "Romantic Rendezvous",
    "Viashino Pyromancer",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(f: usize) -> Vec<Card> {
    let spells = match f {
        0 => vec![
            NAMES[0], NAMES[1], NAMES[2], NAMES[3], NAMES[4], NAMES[5], NAMES[6], NAMES[7],
            NAMES[8],
        ],
        1 => vec![
            NAMES[0], NAMES[3], NAMES[4], NAMES[6], NAMES[7], NAMES[8], NAMES[9], NAMES[10],
            NAMES[11],
        ],
        _ => vec![
            NAMES[1],
            NAMES[2],
            NAMES[5],
            NAMES[6],
            NAMES[8],
            NAMES[9],
            NAMES[10],
            NAMES[11],
            "Helpful Hunter",
        ],
    };
    let mut d = Vec::new();
    for n in ["Plains", "Mountain", "Swamp"] {
        d.extend([Card::from_name(n).unwrap(); 8]);
    }
    for n in spells {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn check(o: &Observation) -> u64 {
    let mut n = 0;
    for v in o
        .objects
        .iter()
        .filter(|v| v.zone == Zone::Battlefield && v.card.is_creature())
    {
        let (base, k) = match index(v.card) {
            Some(0) => ((1, 1), Keywords::MENACE),
            Some(4) => ((1, 1), Keywords::NONE),
            Some(8) => ((5, 5), Keywords::NONE),
            Some(11) => ((2, 1), Keywords::NONE),
            _ => {
                assert_eq!(v.card.name(), "Helpful Hunter");
                ((1, 1), Keywords::NONE)
            }
        };
        assert_eq!((v.power, v.toughness), base);
        assert_eq!(v.keywords, k);
        n += 1;
    }
    n
}
fn classify(s: &SpellView) -> Option<usize> {
    if matches!(s.kind, StackKind::ActivatedAbility { .. }) {
        return s
            .source_card
            .token_info()
            .filter(|t| t.subtypes == "Clue")
            .map(|_| 12);
    }
    let i = index(s.source_card)?;
    if matches!(s.kind, StackKind::TriggeredAbility { .. }) {
        [0, 4, 8, 11].contains(&i).then_some(i)
    } else {
        (![0, 4, 8, 11].contains(&i)).then_some(i)
    }
}
fn object_target<'a>(o: &'a Observation, s: &SpellView) -> Option<&'a ObjectView> {
    s.targets.first().and_then(|t| t.object).and_then(|r| {
        o.objects
            .iter()
            .find(|v| v.reference == r && v.zone == Zone::Battlefield)
    })
}
fn clues(o: &Observation, p: Player) -> usize {
    o.objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.token.is_some_and(|t| t.subtypes == "Clue")
        })
        .count()
}
fn verify(i: usize, root: &SpellView, b: &Observation, a: &Observation) {
    let mut life = [b.players[0].life, b.players[1].life];
    let p = root.controller.index();
    match i {
        0 => {
            life[p] -= 1;
            assert_eq!(clues(a, root.controller), clues(b, root.controller) + 1);
        }
        8 => {
            life[p] -= 1;
        }
        9 => {
            life[0] -= 2;
            life[1] -= 2;
            assert_eq!(a.players[p].library_count, b.players[p].library_count - 2);
            assert_eq!(a.players[p].hand_count, b.players[p].hand_count + 2);
        }
        10 => {
            assert_eq!(a.players[p].library_count, b.players[p].library_count - 2);
            assert_eq!(
                a.players[p].hand_count,
                b.players[p].hand_count - usize::from(b.players[p].hand_count > 0) + 2
            );
        }
        12 => {
            assert_eq!(a.players[p].library_count, b.players[p].library_count - 1);
            assert_eq!(a.players[p].hand_count, b.players[p].hand_count + 1);
        }
        6 if root.mode == Some(1) => {
            if let Some(v) = object_target(b, root) {
                assert_eq!(index(v.card), Some(4));
                let after = a.objects.iter().find(|o| o.id == v.id).unwrap();
                assert_eq!(after.zone, Zone::Graveyard);
                assert_ne!(after.reference, v.reference);
            }
        }
        _ => {
            let (damage, gain) = match i {
                1 => (4, 0),
                2 => (3, 2),
                3 => (4, 2),
                4 => (1, 1),
                5 => (3, 1),
                6 => {
                    assert_eq!(root.mode, Some(0));
                    (4, 0)
                }
                7 => (3, 3),
                11 => (2, 0),
                _ => unreachable!(),
            };
            let mut valid = false;
            if let Some(v) = object_target(b, root) {
                let qualifiers = match i {
                    1 | 5 => b.attackers.contains(&v.id) || b.blocking.contains(&v.id),
                    2 | 6 => v.tapped,
                    _ => true,
                };
                if qualifiers {
                    valid = true;
                    let after = a.objects.iter().find(|o| o.id == v.id).unwrap();
                    if v.damage + i128::from(damage) >= v.toughness {
                        assert_eq!(after.zone, Zone::Graveyard);
                        assert_ne!(after.reference, v.reference);
                    } else {
                        assert_eq!(after.reference, v.reference);
                        assert_eq!(after.damage, v.damage + i128::from(damage));
                    }
                }
            } else if let Some(t) = root.targets.first()
                && let Target::Player(player) = t.target
            {
                valid = true;
                life[player.index()] -= i128::from(damage);
            }
            if valid {
                life[p] += i128::from(gain);
            }
        }
    }
    assert_eq!(
        [a.players[0].life, a.players[1].life],
        life,
        "{} effect {i}",
        root.source_card.name()
    );
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let (mut decisions, mut replays, mut pairs, mut private, mut max_bytes) = (0, 0, 0, 0, 0);
    let (mut casts, mut triggers, mut verified, mut modes, mut clue_activations) =
        ([0u64; 12], [0u64; 12], [0u64; 13], [0u64; 2], 0);
    let (mut attack_choices, mut combat_legal) = (0u64, 0u64);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(seed % 3), deck(seed % 3)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut pending = BTreeMap::<u64, (usize, SpellView, Observation)>::new();
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            let enemy_combat = before.objects.iter().any(|v| {
                v.owner != actor
                    && (before.attackers.contains(&v.id) || before.blocking.contains(&v.id))
            });
            if enemy_combat
                && actions.iter().any(|a| match a {
                    Action::Cast { card, .. } | Action::AnnounceCast { card } => before
                        .objects
                        .iter()
                        .find(|v| v.id == *card)
                        .is_some_and(|v| matches!(index(v.card), Some(1 | 5))),
                    _ => false,
                })
            {
                combat_legal += 1;
            }
            let waiting = before.objects.iter().any(|v| {
                v.owner == actor && v.zone == Zone::Hand && matches!(index(v.card), Some(1 | 5))
            });
            let chosen = actions
                .iter()
                .enumerate()
                .max_by_key(|(j, action)| match action {
                    Action::ChooseMode { mode } => {
                        if usize::from(*mode) == (seed + modes.iter().sum::<u64>() as usize) % 2 {
                            500
                        } else {
                            400
                        }
                    }
                    Action::ChooseSpellCost { cost } => {
                        let c = before
                            .cost_choices
                            .iter()
                            .find(|c| c.index == *cost)
                            .unwrap()
                            .cost;
                        400 - (c.generic + c.colored.iter().sum::<u32>()) as i64
                    }
                    Action::AnnounceActivate { card, .. } => {
                        if before
                            .objects
                            .iter()
                            .any(|v| v.id == *card && v.token.is_some_and(|t| t.subtypes == "Clue"))
                        {
                            180
                        } else {
                            -100
                        }
                    }
                    Action::Activate { .. } | Action::BeginActivate { .. } => -100,
                    // Stop mana actions as soon as payment is available; retain
                    // unused lands for combat responses across step boundaries.
                    Action::Concede => -1000,
                    Action::PayCosts { .. } => 500,
                    Action::TapMana { .. } => {
                        if before.decision == "pay_costs" {
                            150
                        } else {
                            -100
                        }
                    }
                    Action::Attack { .. } | Action::AttackPlaneswalker { .. } => 350,
                    Action::Cast { card, target, .. } => {
                        let i = before
                            .objects
                            .iter()
                            .find(|v| v.id == *card)
                            .and_then(|v| index(v.card));
                        if matches!(i, Some(1 | 5)) && !enemy_combat
                            || waiting && !enemy_combat && matches!(i, Some(2 | 3 | 6 | 7))
                        {
                            return -100;
                        }
                        (if matches!(i, Some(1 | 5)) { 380 } else { 250 })
                            + match target {
                                Some(Target::Player(p)) => {
                                    if *p != actor {
                                        10
                                    } else {
                                        -80
                                    }
                                }
                                Some(Target::Card(id)) => {
                                    if before
                                        .objects
                                        .iter()
                                        .any(|v| v.id == *id && v.owner != actor)
                                    {
                                        20
                                    } else {
                                        -60
                                    }
                                }
                                _ => 0,
                            }
                    }
                    Action::AnnounceCast { card } => {
                        let i = before
                            .objects
                            .iter()
                            .find(|v| v.id == *card)
                            .and_then(|v| index(v.card));
                        if matches!(i, Some(1 | 5)) && !enemy_combat
                            || waiting && !enemy_combat && matches!(i, Some(2 | 3 | 6 | 7))
                        {
                            -100
                        } else if matches!(i, Some(1 | 5)) {
                            380
                        } else {
                            245
                        }
                    }
                    Action::ChooseTarget { target } => {
                        300 + match target {
                            Target::Player(p) => {
                                if *p != actor {
                                    10
                                } else {
                                    -80
                                }
                            }
                            Target::Card(id) => {
                                if before
                                    .objects
                                    .iter()
                                    .any(|v| v.id == *id && v.owner != actor)
                                {
                                    20
                                } else {
                                    -60
                                }
                            }
                            _ => 0,
                        }
                    }
                    _ => {
                        if *j == fallback {
                            100
                        } else {
                            0
                        }
                    }
                })
                .unwrap()
                .0;
            let action = actions[chosen].clone();
            attack_choices += u64::from(matches!(action, Action::Attack { .. }));
            if before.decision == "effect_discard" {
                let other = g.observe(actor.other());
                assert!(other.effect_discards.is_empty() && other.actions.is_empty());
                private += 1;
            }
            if let Action::ChooseMode { mode } = action {
                assert_eq!(
                    before.announcing.as_ref().unwrap().source_card.name(),
                    NAMES[6]
                );
                assert!(g.observe(actor.other()).mode_choices.is_empty());
                modes[mode as usize] += 1;
            }
            let boundary = before.announcing.is_some()
                || before.resolving.is_some()
                || before
                    .stack
                    .iter()
                    .any(|s| index(s.source_card).is_some() || classify(s) == Some(12));
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
                if s.kind == StackKind::Spell {
                    if let Some(i) = index(s.source_card) {
                        casts[i] += 1;
                    }
                } else if classify(s) == Some(12) {
                    clue_activations += 1;
                } else if let Some(i) = index(s.source_card) {
                    triggers[i] += 1;
                }
            }
            if matches!(action, Action::Pass)
                && let Some(root) = before.stack.last()
                && let Some(i) = classify(root)
                && !after.stack.iter().any(|s| s.id == root.id)
            {
                pending.insert(root.id, (i, root.clone(), before.clone()));
            }
            let finished: Vec<_> = pending
                .keys()
                .copied()
                .filter(|id| {
                    !after.stack.iter().any(|s| s.id == *id)
                        && !after.resolving.as_ref().is_some_and(|s| s.id == *id)
                })
                .collect();
            for id in finished {
                let (i, root, b) = pending.remove(&id).unwrap();
                if after.outcome.is_none() {
                    verify(i, &root, &b, &after);
                    verified[i] += 1;
                }
            }
            pairs += check(&after);
            check(&g.observe(actor.other()));
            assert_eq!(after.schema, rustmage::OBSERVATION_SCHEMA);
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
        assert!(casts.iter().all(|n| *n > 0), "casts {casts:?}");
        assert!(verified.iter().all(|n| *n > 0), "verified {verified:?}");
        assert!(modes.iter().all(|n| *n > 0));
        assert!(clue_activations > 0 && attack_choices > 0 && combat_legal > 0);
    }
    println!(
        "Completed {games} Standard fixed-effect games, {decisions} decisions, casts {casts:?}, triggers {triggers:?}, verified effects {verified:?}, Keep Out modes {modes:?}, {clue_activations} Clue activations, {attack_choices} attack declarations, {combat_legal} fundable combat windows, {pairs} independently checked public stat/ability pairs, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
