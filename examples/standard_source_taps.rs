//! Independent public tap-transition, trigger and complete-effect ledger.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, LibraryDestination, ObjectRef,
    ObjectView, Observation, Player, SpellView, StackKind, Step, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 13] = [
    "Attentive Sunscribe",
    "Mechan Navigator",
    "Pestered Wellguard",
    "Rescue Leopard",
    "Seedship Agrarian",
    "Silvergill Peddler",
    "Spider-Gwen, Free Spirit",
    "Starfighter Pilot",
    "Swarm Culler",
    "Volatile Wanderglyph",
    "Wanderbrine Preacher",
    "Wanderwine Distracter",
    "Wylie Duke, Atiin Hero",
];
fn index(card: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == card.name())
}
fn deck(seed: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (name, n) in [
        ("Plains", 6),
        ("Island", 6),
        ("Swamp", 4),
        ("Mountain", 4),
        ("Forest", 4),
    ] {
        d.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for name in [
        NAMES[seed % 13],
        NAMES[(seed + 1) % 13],
        NAMES[(seed + 2) % 13],
        "Frostbridge Guard",
        "Mental Modulation",
        "Glamermite",
        "Helpful Hunter",
        "Three Tree Mascot",
        "Bear Cub",
    ] {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn battlefield(o: &Observation) -> impl Iterator<Item = &ObjectView> {
    o.objects.iter().filter(|v| v.zone == Zone::Battlefield)
}
fn items(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(&o.announcing)
        .chain(&o.resolving)
}
fn check(o: &Observation, debuff: &BTreeMap<ObjectRef, i128>) -> u64 {
    let mut n = 0;
    for v in battlefield(o).filter(|v| v.card.is_creature()) {
        let (p, t, k) = match v.card.name() {
            "Attentive Sunscribe"
            | "Starfighter Pilot"
            | "Volatile Wanderglyph"
            | "Wanderbrine Preacher"
            | "Frostbridge Guard"
            | "Bear Cub" => (2, 2, Keywords::NONE),
            "Mechan Navigator" => (2, 1, Keywords::NONE),
            "Pestered Wellguard" => (3, 2, Keywords::NONE),
            "Rescue Leopard" => (4, 2, Keywords::NONE),
            "Seedship Agrarian" => (3, 3, Keywords::NONE),
            "Silvergill Peddler" => (2, 3, Keywords::NONE),
            "Spider-Gwen, Free Spirit" => (2, 3, Keywords::REACH),
            "Swarm Culler" => (2, 4, Keywords::FLYING),
            "Wanderwine Distracter" => (4, 3, Keywords::NONE),
            "Wylie Duke, Atiin Hero" => (4, 2, Keywords::VIGILANCE),
            "Glamermite" => (2, 2, Keywords::FLYING.union(Keywords::FLASH)),
            "Helpful Hunter" => (1, 1, Keywords::NONE),
            "Three Tree Mascot" => (2, 1, Keywords::CHANGELING),
            _ => {
                assert_eq!(v.card.token_info().unwrap().subtypes, "Faerie");
                (1, 1, Keywords::FLYING)
            }
        };
        let counter = i128::from(v.counters.plus_one) - i128::from(v.counters.minus_one);
        assert_eq!(
            (v.power, v.toughness, v.keywords),
            (
                p + counter + debuff.get(&v.reference).copied().unwrap_or(0),
                t + counter,
                k
            ),
            "{}",
            v.card.name()
        );
        n += 1;
    }
    n
}
fn verify(
    s: &SpellView,
    b: &Observation,
    a: &Observation,
    debuff: &mut BTreeMap<ObjectRef, i128>,
) -> bool {
    let p = s.controller.index();
    let draw = a.players[p].cards_drawn_this_turn - b.players[p].cards_drawn_this_turn;
    let discard = a.players[p].cards_discarded_this_turn - b.players[p].cards_discarded_this_turn;
    let Some(i) = index(s.source_card) else {
        return false;
    };
    assert!(s.entering_object.is_none() || i == 4);
    if i == 4 && matches!(s.kind, StackKind::TriggeredAbility { ability: 1 }) {
        if let Some(v) = battlefield(a).find(|v| v.reference == s.object) {
            let old = battlefield(b).find(|v| v.reference == s.object).unwrap();
            assert_eq!(v.counters.plus_one, old.counters.plus_one + 1);
        }
        return false;
    }
    match i {
        0 => {
            assert_eq!(a.players[p].library_count, b.players[p].library_count);
            assert_eq!(draw, 0);
        }
        1 | 5 => {
            assert_eq!((draw, discard), (1, 1));
        }
        2 | 4 => {
            let fresh: Vec<_> = battlefield(a)
                .filter(|v| {
                    v.owner == s.controller
                        && v.card.is_token()
                        && !battlefield(b).any(|old| old.reference == v.reference)
                })
                .collect();
            assert_eq!(fresh.len(), 1);
            assert_eq!(
                fresh[0].card.token_info().unwrap().subtypes,
                if i == 2 { "Faerie" } else { "Lander" }
            );
            assert!(!fresh[0].tapped);
        }
        3 | 6 | 9 => {
            assert!(draw <= 1);
            assert_eq!(draw, discard);
            return draw == 1;
        }
        7 => {
            assert_eq!(draw, 0);
            assert!(b.players[p].library_count - a.players[p].library_count <= 1);
        }
        8 => {
            assert!(draw <= 1);
            if draw == 1 {
                let removed: Vec<_> = battlefield(b)
                    .filter(|v| {
                        v.owner == s.controller
                            && !battlefield(a).any(|now| now.reference == v.reference)
                    })
                    .collect();
                assert_eq!(removed.len(), 1);
                assert_ne!(removed[0].reference, s.object);
                assert!(removed[0].card.is_artifact() || removed[0].card.is_creature());
            }
            return draw == 1;
        }
        10 => assert_eq!(a.players[p].life, b.players[p].life + 2),
        11 => {
            assert_eq!(s.targets.len(), 1);
            let reference = s.targets[0].object.unwrap();
            if let Some(v) = battlefield(a).find(|v| v.reference == reference) {
                let old = battlefield(b).find(|v| v.reference == reference).unwrap();
                assert_ne!(old.owner, s.controller);
                assert_eq!(v.power, old.power - 3);
                assert_eq!(v.toughness, old.toughness);
                *debuff.entry(reference).or_default() -= 3;
            }
        }
        12 => {
            assert_eq!(a.players[p].life, b.players[p].life + 1);
            assert_eq!(draw, 1);
        }
        _ => unreachable!(),
    }
    true
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let (
        mut decisions,
        mut replays,
        mut pairs,
        mut private,
        mut tapped,
        mut entry_untapped,
        mut vigilance,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    let (mut casts, mut triggers, mut effects) = ([0u64; 13], [0u64; 13], [0u64; 13]);
    let (mut attack_taps, mut effect_taps, mut unannouncable, mut buried) =
        (0u64, 0u64, 0u64, 0u64);
    let mut max_bytes = 0;
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(seed), deck(seed + 6)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let (mut seen, mut debt, mut pending, mut debuff) = (
            BTreeSet::new(),
            BTreeMap::<(ObjectRef, u16, Option<ObjectRef>), u64>::new(),
            BTreeMap::<u64, (SpellView, Observation)>::new(),
            BTreeMap::new(),
        );
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let b = g.observe(actor);
            let actions = g.legal_actions();
            let fallback = rustmage::policy::smoke_index(&g, &actions);
            let tap_target = |target: Target| {
                if let Target::Card(id) = target {
                    battlefield(&b)
                        .find(|v| v.id == id)
                        .is_some_and(|v| !v.tapped && index(v.card).is_some())
                } else {
                    false
                }
            };
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
                        if index(v.card).is_some() {
                            280
                        } else if v.card.name() == "Mental Modulation"
                            && !battlefield(&b).any(|v| index(v.card).is_some() && !v.tapped)
                        {
                            -100
                        } else {
                            250
                        }
                    }
                    Action::Activate { card, .. }
                    | Action::BeginActivate { card, .. }
                    | Action::AnnounceActivate { card, .. } => {
                        if b.objects
                            .iter()
                            .any(|v| v.id == *card && v.card.name() == "Frostbridge Guard")
                            && battlefield(&b).any(|v| index(v.card).is_some() && !v.tapped)
                        {
                            240
                        } else {
                            -100
                        }
                    }
                    Action::ChooseMode { mode } => 300 - i32::from(*mode != 0),
                    Action::ChooseTarget { target } => {
                        if tap_target(*target) {
                            220
                        } else if *j == fallback {
                            100
                        } else {
                            0
                        }
                    }
                    Action::ChooseOptional { accept } => 300 + i32::from(*accept),
                    Action::ArrangeLibrary { destination, .. } => {
                        300 + i32::from(
                            *destination
                                == if seed % 2 == 0 {
                                    LibraryDestination::Top
                                } else if b.library_choice.as_ref().unwrap().surveil {
                                    LibraryDestination::Graveyard
                                } else {
                                    LibraryDestination::Bottom
                                },
                        )
                    }
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
            if let Some(choice) = &b.library_choice {
                let other = g.observe(actor.other());
                assert!(other.library_choice.is_none());
                assert_eq!(choice.remaining.len(), 1);
                private += 1;
            }
            let boundary = b.announcing.is_some()
                || b.resolving.is_some()
                || !b.stack.is_empty()
                || battlefield(&b).any(|v| index(v.card).is_some());
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(actor, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(actor, action.clone())?;
            }
            let a = g.observe(actor);
            if let Action::ArrangeLibrary { card, destination } = action {
                let choice = b.library_choice.as_ref().unwrap();
                assert!(choice.remaining.iter().any(|v| v.id == card));
                let grave = destination == LibraryDestination::Graveyard;
                assert!(!grave || choice.surveil);
                assert_eq!(
                    a.players[actor.index()].library_count + usize::from(grave),
                    b.players[actor.index()].library_count
                );
                if grave {
                    assert!(
                        a.objects
                            .iter()
                            .any(|v| v.id == card && v.zone == Zone::Graveyard)
                    );
                    buried += 1;
                } else if a.library_choice.is_none() {
                    assert!(!a.objects.iter().any(|v| v.id == card));
                }
            }
            if b.turn != a.turn || b.step == Step::Cleanup && a.step != Step::Cleanup {
                debuff.clear();
            }
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
                if a.outcome.is_none() && verify(&s, &before, &a, &mut debuff) {
                    effects[index(s.source_card).unwrap()] += 1;
                }
            }
            for old in battlefield(&b) {
                if let Some(now) = battlefield(&a).find(|v| v.reference == old.reference)
                    && !old.tapped
                    && now.tapped
                    && index(old.card).is_some()
                {
                    *debt.entry((old.reference, 0, None)).or_default() += 1;
                    tapped += 1;
                    if matches!(action, Action::FinishAttackers) {
                        attack_taps += 1;
                    } else {
                        effect_taps += 1;
                    }
                }
            }
            for v in
                battlefield(&a).filter(|v| !battlefield(&b).any(|old| old.reference == v.reference))
            {
                if index(v.card).is_some() {
                    assert!(!v.tapped);
                    entry_untapped += 1;
                }
                if matches!(v.card.name(), "Glamermite" | "Helpful Hunter") {
                    *debt.entry((v.reference, 0, Some(v.reference))).or_default() += 1;
                }
                if v.card.is_land() {
                    for source in
                        battlefield(&a).filter(|s| s.owner == v.owner && s.card.name() == NAMES[4])
                    {
                        *debt
                            .entry((source.reference, 1, Some(v.reference)))
                            .or_default() += 1;
                    }
                }
            }
            if matches!(action, Action::FinishAttackers) {
                for v in battlefield(&a).filter(|v| v.attacking && v.card.name() == NAMES[12]) {
                    assert!(!v.tapped);
                    vigilance += 1;
                }
            }
            for s in items(&a) {
                if !seen.insert(s.id) {
                    continue;
                }
                match s.kind {
                    StackKind::DelayedReturn
                    | StackKind::DelayedSacrifice
                    | StackKind::DelayedWarpExile => {
                        panic!("no delayed producer in this fixture")
                    }
                    StackKind::Spell => {
                        assert!(s.entering_object.is_none());
                        if let Some(i) = index(s.source_card) {
                            casts[i] += 1;
                        }
                    }
                    StackKind::TriggeredAbility { ability } => {
                        let key = (s.object, ability, s.entering_object);
                        let n = debt.get_mut(&key).unwrap_or_else(|| {
                            panic!("unexpected {} {key:?} seed{seed}", s.source_card.name())
                        });
                        assert!(*n > 0);
                        *n -= 1;
                        if ability == 0
                            && let Some(i) = index(s.source_card)
                        {
                            assert!(s.entering_object.is_none());
                            triggers[i] += 1;
                        }
                    }
                    StackKind::ActivatedAbility { .. } => {
                        assert!(s.entering_object.is_none());
                        assert!(matches!(
                            s.source_card.name(),
                            "Frostbridge Guard" | "Three Tree Mascot"
                        ));
                    }
                }
            }
            if a.outcome.is_none() && a.decision == "priority" {
                // A mandatory targeted trigger with no legal opponent creature
                // is removed during announcement without a public stack item.
                for ((source, ability, entering), amount) in &mut debt {
                    if *amount > 0
                        && *ability == 0
                        && entering.is_none()
                        && let Some(v) = a.objects.iter().find(|v| v.id == source.card)
                        && v.card.name() == NAMES[11]
                        && !battlefield(&a)
                            .any(|target| target.owner != v.owner && target.card.is_creature())
                    {
                        unannouncable += *amount;
                        *amount = 0;
                    }
                }
                assert!(
                    debt.values().all(|n| *n == 0),
                    "missing triggers seed{seed}: {debt:?}"
                );
            }
            pairs += check(&a, &debuff);
            check(&g.observe(actor.other()), &debuff);
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
        assert!(casts.iter().all(|n| *n > 0), "casts {casts:?}");
        assert!(triggers.iter().all(|n| *n > 0), "triggers {triggers:?}");
        assert!(effects.iter().all(|n| *n > 0), "effects {effects:?}");
        assert!(
            private > 0
                && vigilance > 0
                && replays > 0
                && attack_taps > 0
                && effect_taps > 0
                && buried > 0
        );
    }
    println!(
        "Completed {games} Standard source-tap games, {decisions} decisions, casts {casts:?}, tap triggers {triggers:?}, complete effects {effects:?}, {tapped} tap transitions ({attack_taps} attack, {effect_taps} effect), {unannouncable} unannouncable mandatory triggers, {buried} surveilled graveyard cards, {entry_untapped} untapped source entries, {vigilance} vigilance declarations, {private} private library choices, {pairs} public stat/keyword pairs, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
