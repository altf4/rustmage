//! Independent public forage payment, event, reflexive, effect and replay witnesses.
use rustmage::{
    Action, Card, Config, DeckRules, Error, ForageRoute, Game, Keywords, ObjectRef, ObjectView,
    Observation, Player, SpellView, StackKind, Step, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 6] = [
    "Bushy Bodyguard",
    "Camellia, the Seedmiser",
    "Corpseberry Cultivator",
    "Curious Forager",
    "Feed the Cycle",
    "Treetop Sentries",
];
fn bf(o: &Observation) -> impl Iterator<Item = &ObjectView> {
    o.objects.iter().filter(|v| v.zone == Zone::Battlefield)
}
fn exact(o: &Observation, r: ObjectRef) -> Option<&ObjectView> {
    o.objects.iter().find(|v| v.reference == r)
}
fn packets(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(o.announcing.iter())
        .chain(o.resolving.iter())
}
fn deck() -> Vec<Card> {
    let mut d = Vec::new();
    for n in ["Forest", "Swamp"] {
        d.extend([Card::from_name(n).unwrap(); 12]);
    }
    for n in NAMES.into_iter().chain([
        "Bumbleflower's Sharepot",
        "Daggerfang Duo",
        "Mineshaft Spider",
    ]) {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn credit(d: &mut BTreeMap<(ObjectRef, u16), u64>, r: ObjectRef, i: u16) {
    *d.entry((r, i)).or_default() += 1;
}
#[derive(Clone)]
struct Selection {
    route: ForageRoute,
    objects: Vec<ObjectRef>,
    cost: bool,
    player: Player,
    name: &'static str,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let (mut decisions, mut replays, mut qualified, mut stat_pairs, mut max_bytes) =
        (0u64, 0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut effects, mut causes) = ([0u64; 6], [0u64; 6], [[0u64; 3]; 6]);
    let (
        mut routes,
        mut decline,
        mut reflexive,
        mut missing_targets,
        mut offspring,
        mut activations,
    ) = ([0u64; 3], 0u64, 0u64, 0u64, 0u64, 0u64);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let (
            mut seen_cast,
            mut seen_trigger,
            mut debt,
            mut selections,
            mut paid,
            mut counters,
            mut activated,
        ) = (
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeMap::new(),
            BTreeMap::<u64, Selection>::new(),
            BTreeSet::new(),
            BTreeMap::<ObjectRef, u64>::new(),
            BTreeSet::new(),
        );
        for _ in 0..100000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let b = g.observe(actor);
            let fallback = rustmage::policy::smoke_index(&g, &b.actions);
            let choice = b
                .actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| match a {
                    Action::PayCosts { .. } => 600,
                    Action::ChooseForageRoute { route } => {
                        let preferred = match seed % 3 {
                            0 => ForageRoute::Food,
                            1 => ForageRoute::ExileThree,
                            _ => ForageRoute::Black,
                        };
                        if *route == preferred {
                            550
                        } else if *route == ForageRoute::Black {
                            350
                        } else {
                            450
                        }
                    }
                    Action::ChooseForage { .. } => 550,
                    Action::DeclineForage => {
                        if seed % 7 == 0 {
                            580
                        } else {
                            0
                        }
                    }
                    Action::ChooseOffspring { pay } => {
                        if *pay == (seed % 2 == 0) {
                            500
                        } else {
                            100
                        }
                    }
                    Action::ChooseOptional { accept } => {
                        if *accept {
                            450
                        } else {
                            0
                        }
                    }
                    Action::TapMana { .. }
                    | Action::ActivateMana { .. }
                    | Action::AnnounceMana { .. } => {
                        if b.decision == "pay_costs" {
                            300
                        } else {
                            -100
                        }
                    }
                    Action::Cast { .. } | Action::AnnounceCast { .. } => 400,
                    Action::AnnounceActivate { card, .. }
                        if b.objects
                            .iter()
                            .any(|v| v.id == *card && v.card.name() == NAMES[1]) =>
                    {
                        if b.active == actor
                            && b.step == Step::Main2
                            && !activated.contains(&(b.turn, *card))
                        {
                            490
                        } else {
                            -100
                        }
                    }
                    Action::Activate { card, .. }
                    | Action::AnnounceActivate { card, .. }
                    | Action::BeginActivate { card, .. }
                        if b.objects.iter().any(|v| {
                            v.id == *card
                                && v.card.token_info().is_some_and(|t| t.subtypes == "Food")
                        }) =>
                    {
                        -100
                    }
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    } => {
                        let v = b.objects.iter().find(|v| v.id == *id).unwrap();
                        if v.zone == Zone::Graveyard || v.owner != actor {
                            500
                        } else if *j == fallback {
                            100
                        } else {
                            0
                        }
                    }
                    Action::ChooseSpellCost { .. } => 450,
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
            let action = b.actions[choice].clone();
            if let Action::AnnounceActivate { card, .. } = action {
                activated.insert((b.turn, card));
            }
            let context = b.announcing.as_ref().or(b.resolving.as_ref());
            if let Action::ChooseForageRoute { route } = action {
                let s = context.unwrap();
                selections.insert(
                    s.id,
                    Selection {
                        route,
                        objects: Vec::new(),
                        cost: b.announcing.is_some(),
                        player: actor,
                        name: s.source_card.name(),
                    },
                );
            }
            if let Action::ChooseForage { object } = action {
                selections
                    .get_mut(&context.unwrap().id)
                    .unwrap()
                    .objects
                    .push(object);
            }
            let commit = if let Some(s) = context {
                selections
                    .get(&s.id)
                    .filter(|f| {
                        if f.cost {
                            matches!(action, Action::PayCosts { .. })
                        } else {
                            matches!(action, Action::ChooseForage { .. })
                                && f.objects.len()
                                    == if f.route == ForageRoute::Food { 1 } else { 3 }
                        }
                    })
                    .map(|f| (s.id, f.clone()))
            } else {
                None
            };
            let resolving = if b.resolving.is_some() {
                b.resolving.clone()
            } else if matches!(action, Action::Pass) {
                b.stack.last().cloned()
            } else {
                None
            };
            let snap = g.snapshot();
            let rev = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, action.clone())?;
            g.apply(actor, action.clone())?;
            let expected = g.snapshot();
            assert_eq!(expected, fork.snapshot());
            g.restore(&snap)?;
            assert_eq!(g.apply_index(actor, rev, choice), Err(Error::StaleRevision));
            g.apply(actor, action.clone())?;
            assert_eq!(g.snapshot(), expected);
            replays += 1;
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
            let a = g.observe(actor);
            if let Some((id, f)) = commit {
                selections.remove(&id);
                let route = match f.route {
                    ForageRoute::ExileThree => 0,
                    ForageRoute::Food => 1,
                    ForageRoute::Black => 2,
                };
                routes[route] += 1;
                if let Some(i) = NAMES.iter().position(|&n| n == f.name) {
                    causes[i][route] += 1;
                }
                if f.route != ForageRoute::Black {
                    assert_eq!(
                        f.objects.len(),
                        if f.route == ForageRoute::Food { 1 } else { 3 }
                    );
                    let mut distinct = BTreeSet::new();
                    for r in &f.objects {
                        assert!(distinct.insert(*r));
                        let old = exact(&b, *r).unwrap();
                        assert_eq!(old.owner, f.player);
                        assert_eq!(
                            old.zone,
                            if f.route == ForageRoute::Food {
                                Zone::Battlefield
                            } else {
                                Zone::Graveyard
                            }
                        );
                        let new = a.objects.iter().find(|v| v.id == r.card);
                        assert_eq!(
                            new.map_or(Zone::Outside, |v| v.zone),
                            if f.route == ForageRoute::Food {
                                Zone::Outside
                            } else {
                                Zone::Exile
                            }
                        );
                    }
                    for source in bf(&b).filter(|v| v.owner == f.player) {
                        if source.card.name() == NAMES[2] {
                            credit(&mut debt, source.reference, 1);
                        }
                        if f.route == ForageRoute::Food && source.card.name() == NAMES[1] {
                            credit(&mut debt, source.reference, 0);
                        }
                    }
                    paid.insert(id);
                    if !f.cost && f.name == NAMES[3] {
                        credit(&mut debt, context.unwrap().object, 1);
                        if !a.objects.iter().any(|v| {
                            v.owner == f.player
                                && v.zone == Zone::Graveyard
                                && v.card
                                    .info()
                                    .type_line
                                    .split('—')
                                    .next()
                                    .unwrap()
                                    .split_whitespace()
                                    .any(|t| {
                                        [
                                            "Creature",
                                            "Land",
                                            "Artifact",
                                            "Enchantment",
                                            "Planeswalker",
                                            "Battle",
                                        ]
                                        .contains(&t)
                                    })
                        }) {
                            missing_targets += 1;
                        }
                    }
                } else {
                    let request = b.payment_request.as_ref().unwrap();
                    assert_eq!(request.required[2], 2);
                    assert_eq!(request.generic, 1);
                }
            }
            if matches!(action, Action::DeclineForage) {
                decline += 1;
                if let Some(s) = context {
                    selections.remove(&s.id);
                }
            }
            for v in bf(&a).filter(|v| !bf(&b).any(|old| old.reference == v.reference)) {
                match v.card.name() {
                    "Bushy Bodyguard" => {
                        credit(&mut debt, v.reference, 1);
                        if !v.card.is_token()
                            && b.stack.iter().any(|s| {
                                s.kind == StackKind::Spell
                                    && s.card == v.id
                                    && s.cast.is_some_and(|c| c.offspring == Some(true))
                            })
                        {
                            credit(&mut debt, v.reference, 0);
                        }
                    }
                    "Curious Forager" | "Treetop Sentries" => credit(&mut debt, v.reference, 0),
                    _ => {}
                }
            }
            if a.step == Step::BeginCombat && (a.turn != b.turn || b.step != Step::BeginCombat) {
                for v in bf(&b).filter(|v| v.owner == a.active && v.card.name() == NAMES[2]) {
                    credit(&mut debt, v.reference, 0);
                }
            }
            for s in &a.stack {
                if s.kind == StackKind::Spell
                    && seen_cast.insert(s.id)
                    && let Some(i) = NAMES.iter().position(|&n| n == s.source_card.name())
                {
                    casts[i] += 1;
                }
            }
            if let Some(s) = resolving
                && a.resolving.as_ref().is_none_or(|r| r.id != s.id)
                && !a.stack.iter().any(|r| r.id == s.id)
            {
                match (s.source_card.name(), s.kind) {
                    ("Bushy Bodyguard", StackKind::TriggeredAbility { ability: 1 })
                        if paid.contains(&s.id) =>
                    {
                        if let Some(v) = exact(&a, s.object).filter(|v| v.zone == Zone::Battlefield)
                        {
                            let before = exact(&b, s.object).unwrap().counters.plus_one;
                            assert_eq!(v.counters.plus_one, before + 2);
                            *counters.entry(s.object).or_default() += 2;
                            effects[0] += 1;
                        }
                    }
                    ("Treetop Sentries", StackKind::TriggeredAbility { ability: 0 })
                        if paid.contains(&s.id) =>
                    {
                        assert_eq!(
                            a.players[s.controller.index()].cards_drawn_this_turn,
                            b.players[s.controller.index()].cards_drawn_this_turn
                                + u64::from(b.players[s.controller.index()].library_count > 0)
                        );
                        effects[5] += 1;
                    }
                    ("Corpseberry Cultivator", StackKind::TriggeredAbility { ability: 1 }) => {
                        if let Some(v) = exact(&a, s.object).filter(|v| v.zone == Zone::Battlefield)
                        {
                            assert_eq!(
                                v.counters.plus_one,
                                exact(&b, s.object).unwrap().counters.plus_one + 1
                            );
                            *counters.entry(s.object).or_default() += 1;
                            effects[2] += 1;
                        }
                    }
                    ("Curious Forager", StackKind::TriggeredAbility { ability: 1 }) => {
                        if let Some(r) = s.targets.first().and_then(|t| t.object)
                            && let Some(v) = exact(&b, r).filter(|v| v.zone == Zone::Graveyard)
                        {
                            assert_eq!(
                                a.objects.iter().find(|n| n.id == v.id).unwrap().zone,
                                Zone::Hand
                            );
                            effects[3] += 1;
                            reflexive += 1;
                        }
                    }
                    ("Camellia, the Seedmiser", StackKind::ActivatedAbility { ability: 0 }) => {
                        for v in
                            bf(&b).filter(|v| {
                                v.owner == s.controller
                                    && v.reference != s.object
                                    && v.card.info().type_line.split('—').nth(1).is_some_and(|x| {
                                        x.split_whitespace().any(|x| x == "Squirrel")
                                    })
                            })
                        {
                            if let Some(now) =
                                exact(&a, v.reference).filter(|v| v.zone == Zone::Battlefield)
                            {
                                assert_eq!(now.counters.plus_one, v.counters.plus_one + 1);
                                *counters.entry(v.reference).or_default() += 1;
                            }
                        }
                        activations += 1;
                        effects[1] += 1;
                    }
                    ("Camellia, the Seedmiser", StackKind::TriggeredAbility { ability: 0 }) => {
                        let new: Vec<_> = bf(&a)
                            .filter(|v| {
                                v.card
                                    .token_info()
                                    .is_some_and(|t| t.name == "Squirrel Token")
                                    && !bf(&b).any(|old| old.reference == v.reference)
                            })
                            .collect();
                        assert_eq!(new.len(), 1);
                        assert_eq!((new[0].power, new[0].toughness), (1, 1));
                    }
                    ("Bushy Bodyguard", StackKind::TriggeredAbility { ability: 0 }) => {
                        let new: Vec<_> = bf(&a)
                            .filter(|v| {
                                v.card.is_token()
                                    && v.card.name() == NAMES[0]
                                    && !bf(&b).any(|old| old.reference == v.reference)
                            })
                            .collect();
                        assert_eq!(new.len(), 1);
                        assert_eq!((new[0].power, new[0].toughness), (1, 1));
                        offspring += 1;
                    }
                    ("Feed the Cycle", StackKind::Spell) => {
                        if let Some(r) = s.targets.first().and_then(|t| t.object)
                            && let Some(v) = exact(&b, r).filter(|v| v.zone == Zone::Battlefield)
                        {
                            assert_eq!(
                                a.objects
                                    .iter()
                                    .find(|o| o.id == v.id)
                                    .map_or(Zone::Outside, |v| v.zone),
                                if v.card.is_token() {
                                    Zone::Outside
                                } else {
                                    Zone::Graveyard
                                }
                            );
                            effects[4] += 1;
                        }
                    }
                    _ => {}
                }
            }
            for s in packets(&a) {
                if let StackKind::TriggeredAbility { ability } = s.kind
                    && s.source().is_some_and(|source| NAMES.contains(&source.card.name()))
                    && seen_trigger.insert(s.id)
                {
                    let n = debt.get_mut(&(s.object, ability)).unwrap_or_else(|| {
                        panic!(
                            "unpredicted {} {:?} seed{seed}",
                            s.source_card.name(),
                            s.kind
                        )
                    });
                    assert!(*n > 0);
                    *n -= 1;
                    qualified += 1;
                }
            }
            for v in bf(&a).filter(|v| NAMES.contains(&v.card.name())) {
                assert_eq!(
                    v.counters.plus_one,
                    counters.get(&v.reference).copied().unwrap_or(0)
                );
                if v.card.name() != NAMES[4] {
                    let (p, t) = if v.card.is_token() {
                        (1, 1)
                    } else {
                        match v.card.name() {
                            "Bushy Bodyguard" => (2, 1),
                            "Camellia, the Seedmiser" => (3, 3),
                            "Corpseberry Cultivator" => (2, 3),
                            "Curious Forager" => (3, 2),
                            "Treetop Sentries" => (2, 4),
                            _ => unreachable!(),
                        }
                    };
                    assert_eq!(
                        (v.power, v.toughness),
                        (
                            p + i128::from(v.counters.plus_one),
                            t + i128::from(v.counters.plus_one)
                        )
                    );
                    if v.card.name() != NAMES[1]
                        && v.card.info().type_line.contains("Squirrel")
                        && bf(&a).any(|cam| cam.owner == v.owner && cam.card.name() == NAMES[1])
                    {
                        assert!(v.keywords.contains(Keywords::MENACE));
                    }
                    stat_pairs += 1;
                }
            }
        }
        assert!(g.outcome().is_some(), "seed{seed} not finished");
    }
    if games >= 100 {
        assert!(casts.iter().all(|&n| n > 0));
        assert!(effects.iter().all(|&n| n > 0));
        assert!(routes.iter().all(|&n| n > 0), "routes {routes:?}");
        assert!(reflexive > 0 && offspring > 0 && activations > 0 && decline > 0);
    }
    println!(
        "Completed {games} Standard forage games, {decisions} decisions, casts{casts:?}, effects{effects:?}, costs/resolutions by source and [exile,Food,black] {causes:?}, routes{routes:?}, {decline} declines, {reflexive} reflexive returns, {missing_targets} paid no-target reflexives, {offspring} offspring, {activations} Camellia activations, {qualified} independently qualified triggers, {stat_pairs} stat/counter/menace pairs, {replays} replay boundaries, {max_bytes} maximum logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
