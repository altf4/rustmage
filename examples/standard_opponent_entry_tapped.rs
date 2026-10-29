//! Public event ledger for opponent entry replacements and Authority's independent trigger.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, ObjectView, Observation,
    Player, SpellView, StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 2] = ["Authority of the Consuls", "Spider-Woman, Stunning Savior"];
fn deck() -> Vec<Card> {
    let mut d = vec![Card::from_name("Plains").unwrap(); 8];
    for (n, count) in [
        ("Forest", 4),
        ("Island", 4),
        ("Swamp", 4),
        ("Tranquil Cove", 4),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); count]);
    }
    for n in [
        NAMES[0],
        NAMES[1],
        "Bear Cub",
        "Three Tree Mascot",
        "Helpful Hunter",
        "Hopeful Vigil",
        "Invasion Reinforcements",
        "Torpor Orb",
        "Diregraf Ghoul",
    ] {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn items(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(&o.announcing)
        .chain(&o.resolving)
}
fn battlefield(o: &Observation) -> impl Iterator<Item = &ObjectView> {
    o.objects.iter().filter(|v| v.zone == Zone::Battlefield)
}
fn entry_ability(s: &ObjectView, v: &ObjectView) -> bool {
    match s.card.name() {
        "Authority of the Consuls" => s.owner != v.owner && v.card.is_creature(),
        "Helpful Hunter" | "Hopeful Vigil" | "Invasion Reinforcements" | "Tranquil Cove" => {
            s.reference == v.reference
        }
        _ => false,
    }
}
fn check(o: &Observation) -> u64 {
    let mut count = 0;
    for v in battlefield(o).filter(|v| v.card.is_creature()) {
        let (p, t, k) = match v.card.name() {
            "Spider-Woman, Stunning Savior" => (2, 2, Keywords::FLYING),
            "Bear Cub" | "Diregraf Ghoul" => (2, 2, Keywords::NONE),
            "Three Tree Mascot" => (2, 1, Keywords::CHANGELING),
            "Helpful Hunter" => (1, 1, Keywords::NONE),
            "Invasion Reinforcements" => (1, 1, Keywords::FLASH),
            _ => match v.card.token_info().expect("known token").subtypes {
                "Knight" => (2, 2, Keywords::VIGILANCE),
                "Ally" => (1, 1, Keywords::NONE),
                x => panic!("unknown token {x}"),
            },
        };
        assert_eq!(
            (v.power, v.toughness, v.keywords),
            (p, t, k),
            "{}",
            v.card.name()
        );
        count += 1;
    }
    count
}
fn verify(s: &SpellView, b: &Observation, a: &Observation) -> usize {
    assert!(s.targets.is_empty());
    match s.source().expect("example expects an object-backed packet").card.name() {
        "Authority of the Consuls" | "Tranquil Cove" => {
            assert_eq!(
                a.players[s.controller.index()].life,
                b.players[s.controller.index()].life + 1
            );
            usize::from(s.source().is_some_and(|source| source.card.name() == NAMES[0]))
        }
        "Helpful Hunter" => {
            assert_eq!(
                a.players[s.controller.index()].library_count + 1,
                b.players[s.controller.index()].library_count
            );
            0
        }
        "Hopeful Vigil" if matches!(s.kind, StackKind::TriggeredAbility { ability: 1 }) => {
            assert_eq!(
                a.players[s.controller.index()].life,
                b.players[s.controller.index()].life + 2
            );
            0
        }
        "Hopeful Vigil" | "Invasion Reinforcements" => {
            assert_eq!(
                battlefield(a)
                    .filter(|v| v.card.is_token()
                        && !b.objects.iter().any(|x| x.reference == v.reference))
                    .count(),
                1
            );
            0
        }
        x => panic!("unknown trigger {x}"),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let (mut decisions, mut replays, mut pairs, mut max_bytes) = (0u64, 0u64, 0u64, 0usize);
    let (
        mut casts,
        mut replacements,
        mut effects,
        mut contexts,
        mut suppressed,
        mut token_entries,
        mut artifact_entries,
        mut untaps,
        mut established,
    ) = (
        [0u64; 2], [0u64; 2], 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64,
    );
    let mut hybrid = [0u64; 2];
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let (mut seen, mut debt, mut pending) = (
            BTreeSet::new(),
            BTreeMap::<(ObjectRef, Option<ObjectRef>, u16), u64>::new(),
            BTreeMap::<u64, (SpellView, Observation)>::new(),
        );
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let b = g.observe(actor);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            let selected = actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| match a {
                    Action::PayCosts { .. } => 500,
                    Action::TapMana { .. }
                    | Action::AnnounceMana { .. }
                    | Action::ActivateMana { .. } => {
                        if b.decision == "pay_costs" {
                            150
                        } else {
                            -100
                        }
                    }
                    Action::Cast { card, .. } | Action::AnnounceCast { card } => {
                        let v = b.objects.iter().find(|v| v.id == *card).unwrap();
                        if v.card.name() == "Torpor Orb" && b.turn < 15 {
                            -100
                        } else if NAMES.contains(&v.card.name()) {
                            280
                        } else {
                            250
                        }
                    }
                    Action::ChooseSpellCost { cost } => 300 - i32::from(*cost != (seed % 2) as u16),
                    Action::Activate { .. }
                    | Action::BeginActivate { .. }
                    | Action::AnnounceActivate { .. } => -100,
                    Action::Concede => -1000,
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
            let action = actions[selected].clone();
            if let Action::ChooseSpellCost { cost } = &action
                && b.announcing
                    .as_ref()
                    .is_some_and(|s| s.source().is_some_and(|source| source.card.name() == NAMES[1]))
            {
                hybrid[usize::from(*cost)] += 1;
            }
            let boundary = b.announcing.is_some()
                || !b.stack.is_empty()
                || battlefield(&b).any(|v| NAMES.contains(&v.card.name()));
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(actor, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            let a = g.observe(actor);
            if matches!(action, Action::Pass)
                && let Some(s) = b.stack.last()
                && matches!(s.kind, StackKind::TriggeredAbility { .. })
                && !a.stack.iter().any(|x| x.id == s.id)
            {
                pending.insert(s.id, (s.clone(), b.clone()));
            }
            let done: Vec<_> = pending
                .keys()
                .copied()
                .filter(|id| {
                    !a.stack.iter().any(|s| s.id == *id)
                        && !a.resolving.as_ref().is_some_and(|s| s.id == *id)
                })
                .collect();
            for id in done {
                let (s, before) = pending.remove(&id).unwrap();
                if a.outcome.is_none() {
                    effects += verify(&s, &before, &a) as u64;
                }
            }
            let orb = battlefield(&a).any(|v| v.card.name() == "Torpor Orb");
            for v in
                battlefield(&a).filter(|v| !battlefield(&b).any(|old| old.reference == v.reference))
            {
                let sources: Vec<_> = battlefield(&b)
                    .filter(|s| {
                        s.owner != v.owner
                            && ((s.card.name() == NAMES[0] && v.card.is_creature())
                                || (s.card.name() == NAMES[1]
                                    && (v.card.is_creature() || v.card.is_artifact())))
                    })
                    .collect();
                let inherent = matches!(v.card.name(), "Diregraf Ghoul" | "Tranquil Cove");
                assert_eq!(
                    v.tapped,
                    inherent || !sources.is_empty(),
                    "{} entry seed {seed}",
                    v.card.name()
                );
                for s in sources {
                    replacements[usize::from(s.card.name() == NAMES[1])] += 1;
                }
                token_entries += u64::from(v.card.is_token());
                artifact_entries += u64::from(v.card.is_artifact() && !v.card.is_creature());
                if NAMES.contains(&v.card.name()) {
                    for old in battlefield(&b).filter(|old| old.card.is_creature()) {
                        if let Some(now) = battlefield(&a).find(|x| x.reference == old.reference) {
                            assert_eq!(old.tapped, now.tapped);
                            established += 1;
                        }
                    }
                }
                for s in battlefield(&a).filter(|s| entry_ability(s, v)) {
                    if orb && v.card.is_creature() {
                        suppressed += 1;
                    } else {
                        *debt.entry((s.reference, Some(v.reference), 0)).or_default() += 1;
                    }
                }
            }
            for old in battlefield(&b) {
                if let Some(now) = battlefield(&a).find(|v| v.reference == old.reference) {
                    untaps += u64::from(old.tapped && !now.tapped);
                } else if old.card.name() == "Hopeful Vigil"
                    && a.objects
                        .iter()
                        .any(|v| v.id == old.id && v.zone == Zone::Graveyard)
                {
                    *debt.entry((old.reference, None, 1)).or_default() += 1;
                }
            }
            for s in items(&a) {
                if !seen.insert(s.id) {
                    continue;
                }
                match s.kind {
                    StackKind::CasualtyCopy
                    | StackKind::CopyEndStepSacrifice
                    | StackKind::CopyEndStepConditionalSacrifice
                    | StackKind::DelayedReturn
                    | StackKind::DelayedTokenCreation { .. }
                    | StackKind::DelayedSacrifice
                    | StackKind::DelayedWarpExile
                    | StackKind::DelayedManifestDread
                    | StackKind::SiegeDefeat | StackKind::InherentRule { .. } => {
                        panic!("no delayed producer in this fixture")
                    }
                    StackKind::Spell => {
                        assert!(s.entering_object.is_none());
                        if let Some(i) = NAMES.iter().position(|n| *n == s.source().expect("example expects an object-backed packet").card.name()) {
                            casts[i] += 1;
                        }
                    }
                    StackKind::TriggeredAbility { ability } => {
                        let key = (s.source().expect("example expects an object-backed packet").object, s.entering_object, ability);
                        let n = debt.get_mut(&key).unwrap_or_else(|| {
                            panic!("unexpected {} {key:?} seed{seed}", s.source().expect("example expects an object-backed packet").card.name())
                        });
                        assert!(*n > 0);
                        *n -= 1;
                        contexts += u64::from(s.entering_object.is_some());
                    }
                    StackKind::ActivatedAbility { .. } => {
                        assert!(matches!(
                            s.source().expect("example expects an object-backed packet").card.name(),
                            "Tranquil Cove" | "Three Tree Mascot"
                        ));
                        assert!(s.entering_object.is_none());
                    }
                }
            }
            if a.outcome.is_none() && a.decision == "priority" {
                assert!(
                    debt.values().all(|n| *n == 0),
                    "missing trigger seed{seed}: {debt:?}"
                );
            }
            pairs += check(&a);
            check(&g.observe(actor.other()));
            assert_eq!(a.schema, rustmage::OBSERVATION_SCHEMA);
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert_eq!(
                    g.apply_index(actor, b.revision, 0),
                    Err(Error::StaleRevision)
                );
                g.apply(actor, action)?;
                assert_eq!(g.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game{seed} unfinished");
    }
    if games >= 100 {
        assert!(casts.iter().all(|n| *n > 0));
        assert!(replacements.iter().all(|n| *n > 0));
        assert!(hybrid.iter().all(|n| *n > 0));
        assert!(
            effects > 0
                && suppressed > 0
                && token_entries > 0
                && artifact_entries > 0
                && untaps > 0
                && established > 0
        );
    }
    println!(
        "Completed {games} Standard opponent-entry-tapped games, {decisions} decisions, casts {casts:?}, replacement witnesses {replacements:?}, hybrid choices {hybrid:?}, {effects} Authority life effects, {contexts} exact entry contexts, {suppressed} suppressed entry triggers, {token_entries} token entries, {artifact_entries} noncreature artifact entries, {untaps} ordinary untaps, {established} established creature checks, {pairs} public stat/ability pairs, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
