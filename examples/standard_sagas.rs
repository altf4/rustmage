//! Legal Standard self-play: all twelve whole Saga bodies, real paid casts and chapters.
//! The fixture's trigger index matches its numbered chapters; the engine itself uses
//! immutable Chapter metadata and has no fixture-specific dispatch.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectView, Observation, Player,
    SpellView, StackKind, Target, Zone,
};
const NAMES: [&str; 12] = [
    "Summon: Choco/Mog",
    "Summon: Esper Ramuh",
    "Summon: Fat Chocobo",
    "Summon: G.F. Ifrit",
    "Burn, Burn, Tree and Fern",
    "Summon: Anima",
    "Leaves from the Vine",
    "Summon: Primal Garuda",
    "Welcome to Sweettooth",
    "The Huntsman's Redemption",
    "Roar of Endless Song",
    "Summon: Shiva",
];
fn deck(i: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [
        ("Forest", 8),
        ("Plains", 6),
        ("Island", 4),
        ("Mountain", 6),
        ("Swamp", 8),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    for n in [
        NAMES[i],
        "Bear Cub",
        "Three Tree Mascot",
        "Llanowar Elves",
        "Jibbirik Omnivore",
        "Candy Trail",
        "Thrill of Possibility",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
#[derive(Default)]
struct Progress {
    source: Option<u32>,
    done: bool,
    entered: bool,
    attacked: bool,
    thrill: bool,
    chapters: u8,
    pending: Option<(SpellView, Observation)>,
}
fn field(o: &Observation, p: Player) -> impl Iterator<Item = &ObjectView> {
    o.objects
        .iter()
        .filter(move |v| v.owner == p && v.zone == Zone::Battlefield)
}
fn tokens(o: &Observation, p: Player, s: &str) -> usize {
    field(o, p)
        .filter(|v| v.token.is_some_and(|t| t.subtypes == s))
        .count()
}
fn score(o: &Observation, a: &Action, i: usize, s: &[Progress; 2]) -> i32 {
    let p = o.actor.unwrap();
    let f = &s[p.index()];
    let all = s.iter().all(|v| v.done);
    let obj = |id| o.objects.iter().find(|v| v.id == id);
    let friends = field(o, p)
        .filter(|v| v.is_creature() && v.card.name() != NAMES[i])
        .count();
    match a {
        Action::AnnounceCast { card }
        | Action::Cast { card, .. }
        | Action::BeginCast { card, .. } => {
            if !matches!(a, Action::AnnounceCast { .. }) {
                return -1000;
            }
            match obj(*card).unwrap().card.name() {
                n if n == NAMES[i] => {
                    if f.source.is_none()
                        && f.thrill
                        && friends >= 2
                        && field(o, p.other()).any(|v| v.is_creature())
                        && field(o, p.other()).any(|v| v.card.name() == "Candy Trail")
                        && (!matches!(i, 7 | 11)
                            || field(o, p.other()).any(|v| v.card.name() == "Llanowar Elves"))
                        && (i != 7
                            || field(o, p.other())
                                .any(|v| v.card.name() == "Llanowar Elves" && v.tapped))
                    {
                        2100
                    } else {
                        -1000
                    }
                }
                "Thrill of Possibility" => {
                    if !f.thrill
                        && o.objects.iter().any(|v| {
                            v.owner == p
                                && v.zone == Zone::Hand
                                && v.card.is_creature()
                                && v.card.name() != NAMES[i]
                        })
                    {
                        2200
                    } else {
                        -1000
                    }
                }
                "Candy Trail" => {
                    if !field(o, p).any(|v| v.card.name() == "Candy Trail") && !all {
                        1800
                    } else {
                        -1000
                    }
                }
                "Llanowar Elves" => {
                    if !field(o, p).any(|v| v.card.name() == "Llanowar Elves") || all {
                        1850
                    } else {
                        -1000
                    }
                }
                "Bear Cub" | "Three Tree Mascot" | "Jibbirik Omnivore" if friends < 3 || all => {
                    1600
                }
                _ => -1000,
            }
        }
        Action::PlayLand { card } => {
            1700 - 20
                * field(o, p)
                    .filter(|v| v.card == obj(*card).unwrap().card)
                    .count() as i32
        }
        Action::ActivateMana { card, .. } => {
            if o.decision == "pay_costs" {
                2400
            } else if matches!(i, 7 | 11)
                && !all
                && obj(*card).is_some_and(|v| v.card.name() == "Llanowar Elves")
            {
                1900
            } else {
                -1000
            }
        }
        Action::TapMana { card } => {
            if o.decision == "pay_costs" {
                2400
            } else if matches!(i, 7 | 11)
                && !all
                && obj(*card).is_some_and(|v| v.card.name() == "Llanowar Elves")
            {
                1900
            } else {
                -1000
            }
        }
        Action::ChooseAdditionalCost { object } => {
            if obj(object.card).is_some_and(|v| v.card.is_creature() && v.card.name() != NAMES[i]) {
                2400
            } else {
                1000
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = obj(*id).unwrap();
            // Burn/Ramuh/Shiva restrict their own targets. Hunt's unrestricted third chapter
            // deliberately chooses friendly bodies. Garuda's first target is already restricted.
            if v.owner == p && v.card.name() != NAMES[i] {
                2300
            } else if v.owner == p.other() && v.card.name() == "Llanowar Elves" {
                2250
            } else if v.owner == p.other() && v.card.name() == "Jibbirik Omnivore" {
                2000
            } else {
                1500
            }
        }
        Action::ChooseTarget { .. } => 1600,
        Action::Sacrifice { object } => {
            if obj(object.card).is_some_and(|v| v.card.name() != NAMES[i]) {
                2300
            } else {
                -1000
            }
        }
        Action::Discard { card } => {
            let v = obj(*card).unwrap();
            if v.card.name() == NAMES[i] && f.source.is_none() {
                1
            } else if v.card.name() == "Llanowar Elves"
                && !field(o, p).any(|v| v.card.name() == "Llanowar Elves")
            {
                2
            } else if v.card.is_land() {
                2200
            } else {
                1800
            }
        }
        Action::ChooseSearch { .. } => 2200,
        Action::ArrangeLibrary { .. } => 2000,
        Action::Attack { card, .. } => {
            if all || (f.source == Some(*card) && !f.attacked) {
                1500
            } else {
                -1000
            }
        }
        Action::ChooseOptional { accept } => {
            if *accept {
                2400
            } else {
                -1000
            }
        }
        Action::PayCosts { .. } | Action::Keep => 2600,
        Action::PutTrigger { .. } => 2500,
        Action::FinishTargets | Action::FinishSearch => 1800,
        Action::ChooseReplacement { .. } => 2100,
        Action::FinishAttackers | Action::FinishBlockers => 1200,
        Action::Pass => 0,
        _ => -10000,
    }
}
fn verify(i: usize, p: Player, item: &SpellView, b: &Observation, a: &Observation) {
    let StackKind::TriggeredAbility { ability } = item.kind else {
        panic!("not chapter")
    };
    let ch = ability + 1;
    let old = |id| b.objects.iter().find(|v| v.id == id).unwrap();
    let now = |id| a.objects.iter().find(|v| v.id == id).unwrap();
    let creatures = || field(b, p).filter(|v| v.is_creature());
    let target = || {
        let Target::Card(id) = item.targets[0].target else {
            panic!("card target")
        };
        id
    };
    match (i, ch) {
        (0, _) => {
            let others: Vec<_> = creatures()
                .filter(|v| {
                    v.id != item
                        .source()
                        .expect("example expects an object-backed packet")
                        .id
                })
                .collect();
            assert!(!others.is_empty());
            for v in others {
                let n = now(v.id);
                assert_eq!(n.power, v.power + 1);
                assert_eq!(n.toughness, v.toughness);
            }
        }
        (1, 1) => {
            let n = b
                .objects
                .iter()
                .filter(|v| {
                    v.owner == p
                        && v.zone == Zone::Graveyard
                        && !v.card.is_creature()
                        && !v.card.is_land()
                })
                .count();
            assert!(n > 0);
            let v = now(target());
            assert!(v.zone == Zone::Graveyard || v.damage == old(v.id).damage + n as i128);
        }
        (1, _) => {
            let mut witnesses = 0;
            for v in creatures().filter(|v| v.card.has_subtype("Wizard")) {
                if now(v.id).zone == Zone::Battlefield {
                    assert_eq!(now(v.id).power, v.power + 1);
                    witnesses += 1;
                }
            }
            assert!(witnesses > 0 || ch == 3);
        }
        (2, 1) => assert_eq!(tokens(a, p, "Bird"), tokens(b, p, "Bird") + 1),
        (2, _) => assert!(
            field(a, p)
                .filter(|v| v.is_creature())
                .all(|v| v.keywords.contains(Keywords::TRAMPLE))
        ),
        (3, 1 | 2) => {
            assert_eq!(
                a.players[p.index()].cards_drawn_this_turn,
                b.players[p.index()].cards_drawn_this_turn + 1
            );
            assert_eq!(
                a.players[p.index()].cards_discarded_this_turn,
                b.players[p.index()].cards_discarded_this_turn + 1
            );
        }
        (3 | 4, 3 | 4) => assert_eq!(
            a.players[p.index()].mana[3],
            b.players[p.index()].mana[3] + 1
        ),
        (4, 1) => {
            let v = now(target());
            assert!(v.zone == Zone::Graveyard || v.damage == old(v.id).damage + 6);
        }
        (4, 2) => assert_eq!(now(target()).zone, Zone::Graveyard),
        (5, 1..=3) => {
            assert_eq!(a.players[p.index()].life, b.players[p.index()].life - 1);
            assert_eq!(
                a.players[p.index()].cards_drawn_this_turn,
                b.players[p.index()].cards_drawn_this_turn + 1
            );
        }
        (5, 4) => {
            assert_eq!(
                a.players[p.other().index()].life,
                b.players[p.other().index()].life - 3
            );
            assert_eq!(
                field(a, p.other()).filter(|v| v.is_creature()).count() + 1,
                field(b, p.other()).filter(|v| v.is_creature()).count()
            );
        }
        (6, 1) => {
            assert_eq!(
                a.players[p.index()].library_count + 3,
                b.players[p.index()].library_count
            );
            assert_eq!(tokens(a, p, "Food"), tokens(b, p, "Food") + 1);
        }
        (6, 2) => {
            assert_eq!(item.targets.len(), 2);
            for t in &item.targets {
                let Target::Card(id) = t.target else { panic!() };
                assert_eq!(now(id).counters.plus_one, old(id).counters.plus_one + 1);
            }
        }
        (6, 3) => assert_eq!(
            a.players[p.index()].cards_drawn_this_turn,
            b.players[p.index()].cards_drawn_this_turn + 1
        ),
        (7, 1) => {
            assert!(old(target()).tapped);
            assert_eq!(now(target()).zone, Zone::Graveyard);
        }
        (7, _) => {
            let n = now(target());
            assert_eq!(n.power, old(n.id).power + 1);
            assert!(n.keywords.contains(Keywords::FLYING));
        }
        (8, 1) => assert_eq!(tokens(a, p, "Human"), tokens(b, p, "Human") + 1),
        (8, 2) => assert_eq!(tokens(a, p, "Food"), tokens(b, p, "Food") + 1),
        (8, 3) => {
            let id = target();
            assert_eq!(
                now(id).counters.plus_one,
                old(id).counters.plus_one
                    + 1
                    + field(b, p).filter(|v| v.card.has_subtype("Food")).count() as u64
            );
        }
        (9, 1) => assert_eq!(tokens(a, p, "Beast"), tokens(b, p, "Beast") + 1),
        (9, 2) => {
            assert_eq!(
                field(a, p).filter(|v| v.is_creature()).count() + 1,
                creatures().count()
            );
            assert_eq!(
                a.players[p.index()].hand_count,
                b.players[p.index()].hand_count + 1
            );
            assert!(
                a.known_hand
                    .iter()
                    .any(|r| a.objects.iter().any(|v| v.reference == *r
                        && v.owner == p
                        && (v.card.is_creature() || v.card.is_basic_land())))
            );
        }
        (9, 3) => {
            assert_eq!(item.targets.len(), 2);
            for t in &item.targets {
                let Target::Card(id) = t.target else { panic!() };
                assert_eq!(now(id).power, old(id).power + 2);
                assert_eq!(now(id).toughness, old(id).toughness + 2);
                assert!(now(id).keywords.contains(Keywords::TRAMPLE));
            }
        }
        (10, 1 | 2) => assert_eq!(tokens(a, p, "Elephant"), tokens(b, p, "Elephant") + 1),
        (10, 3) => {
            for v in creatures() {
                assert_eq!(now(v.id).power, v.power * 2);
                assert_eq!(now(v.id).toughness, v.toughness * 2);
            }
        }
        (11, 1 | 2) => {
            let n = now(target());
            assert!(n.tapped);
            assert_eq!(n.counters.stun, old(n.id).counters.stun + 1);
        }
        (11, 3) => {
            let n = field(b, p.other())
                .filter(|v| v.is_creature() && v.tapped)
                .count();
            assert!(n > 0);
            assert_eq!(
                a.players[p.index()].cards_drawn_this_turn,
                b.players[p.index()].cards_drawn_this_turn + n as u64
            );
        }
        _ => panic!("missing {} chapter{ch}", NAMES[i]),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut decisions, mut denials, mut closed, mut private, mut initial, mut maximum) =
        (0u64, 0u64, 0u64, 0u64, 0usize, 0usize);
    let (mut casts, mut effects, mut sacrifices) =
        ([[0u64; 2]; 12], [[[0u64; 4]; 2]; 12], [[0u64; 2]; 12]);
    for seed in 0..games {
        let i = seed % 12;
        let mut g = Game::new(Config {
            decks: [deck(i), deck(i)],
            seed: seed as u64,
            starting_player: if seed % 2 == 0 {
                Player::P0
            } else {
                Player::P1
            },
            deck_rules: DeckRules::Standard,
        })?;
        initial = initial.max(g.state_bytes());
        let mut s: [Progress; 2] = Default::default();
        for _ in 0..30000 {
            if g.outcome().is_some() {
                break;
            }
            let p = g.actor().unwrap();
            let o = g.observe(p);
            for q in [Player::P0, Player::P1] {
                let view = g.observe(q);
                let f = &mut s[q.index()];
                if !f.entered
                    && let Some(v) = view
                        .objects
                        .iter()
                        .find(|v| Some(v.id) == f.source && v.zone == Zone::Battlefield)
                {
                    let expected = [
                        (3, 3),
                        (3, 3),
                        (4, 4),
                        (3, 2),
                        (0, 0),
                        (4, 4),
                        (0, 0),
                        (3, 3),
                        (0, 0),
                        (0, 0),
                        (0, 0),
                        (4, 5),
                    ][i];
                    assert_eq!((v.power, v.toughness), expected);
                    assert_eq!(v.counters.lore, 1);
                    if i == 5 {
                        assert!(v.keywords.contains(Keywords::MENACE));
                    }
                    if i == 7 {
                        assert!(v.keywords.contains(Keywords::FLYING));
                    }
                    f.entered = true;
                }
                if let Some((item, b)) = f.pending.take() {
                    if !view.stack.iter().any(|v| v.id == item.id)
                        && !view.resolving.as_ref().is_some_and(|v| v.id == item.id)
                    {
                        verify(i, q, &item, &b, &view);
                        let StackKind::TriggeredAbility { ability } = item.kind else {
                            unreachable!()
                        };
                        f.chapters |= 1 << ability;
                        effects[i][q.index()][ability as usize] += 1;
                    } else {
                        f.pending = Some((item, b));
                    }
                }
                if let Some(item) = view.stack.last().filter(|v| {
                    v.source().map(|source| source.id) == f.source
                        && matches!(v.kind, StackKind::TriggeredAbility { .. })
                }) && view.resolving.is_none()
                    && view.announcing.is_none()
                {
                    f.pending = Some((item.clone(), view.clone()));
                }
                if !f.done
                    && let Some(v) = view.objects.iter().find(|v| Some(v.id) == f.source)
                    && v.zone == Zone::Graveyard
                {
                    let final_ch = v.card.saga_final_chapter().unwrap();
                    assert_eq!(
                        f.chapters,
                        (1 << final_ch) - 1,
                        "seed{seed} {} {:?}",
                        NAMES[i],
                        q
                    );
                    f.done = true;
                    sacrifices[i][q.index()] += 1;
                }
            }
            let actions = g.legal_actions();
            let j = (0..actions.len())
                .max_by_key(|&j| (score(&o, &actions[j], i, &s), -(j as i64)))
                .unwrap();
            let action = actions[j].clone();
            assert!(
                score(&o, &action, i, &s) >= 0,
                "seed{seed} {} {:?} {:?}",
                NAMES[i],
                o.decision,
                actions
            );
            if let (Some(ann), Action::PayCosts { .. }) = (&o.announcing, &action)
                && ann.kind == StackKind::Spell
            {
                if ann
                    .source()
                    .is_some_and(|source| source.card.name() == NAMES[i])
                {
                    s[p.index()].source = Some(
                        ann.source()
                            .expect("example expects an object-backed packet")
                            .id,
                    );
                    casts[i][p.index()] += 1;
                }
                if ann
                    .source()
                    .is_some_and(|source| source.card.name() == "Thrill of Possibility")
                {
                    s[p.index()].thrill = true;
                }
            }
            if let Some(choice) = &o.effect_choice
                && matches!(choice.kind, rustmage::EffectChoiceKind::Discard)
            {
                private += 1;
                assert!(g.observe(p.other()).effect_choice.is_none());
            }
            if let Action::Attack { card, .. } = &action
                && s[p.index()].source == Some(*card)
            {
                s[p.index()].attacked = true;
            }
            let saved = g.snapshot();
            let revision = g.revision();
            assert_eq!(g.apply(p.other(), action.clone()), Err(Error::WrongActor));
            denials += 1;
            assert_eq!(g.snapshot(), saved);
            let mut fork = g.clone();
            fork.apply(p, action.clone())?;
            let parent = g.clone();
            let mut child = parent.clone();
            drop(parent);
            child.apply(p, action.clone())?;
            closed += 1;
            g.apply_index(p, revision, j)?;
            let after = g.snapshot();
            assert_eq!(fork.snapshot(), after);
            assert_eq!(child.snapshot(), after);
            g.restore(&saved)?;
            assert_eq!(g.apply_index(p, revision, j), Err(Error::StaleRevision));
            denials += 1;
            g.apply(p, action)?;
            assert_eq!(g.snapshot(), after);
            decisions += 1;
            maximum = maximum.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "unfinishedseed{seed}");
        assert!(
            s.iter().all(|f| f.done
                && f.entered
                && (f.attacked || !Card::from_name(NAMES[i]).unwrap().is_creature())),
            "seed{seed} {} incomplete {:?}",
            NAMES[i],
            s.iter()
                .map(|f| (f.source, f.chapters, f.done))
                .collect::<Vec<_>>()
        );
        eprintln!("seed{seed} {} complete {:?}", NAMES[i], g.outcome());
    }
    if games >= 12 {
        assert!(casts.iter().flatten().all(|v| *v > 0));
        assert!(sacrifices.iter().flatten().all(|v| *v > 0));
        for i in 0..12 {
            let chapters = Card::from_name(NAMES[i])
                .unwrap()
                .saga_final_chapter()
                .unwrap() as usize;
            for seat in &effects[i] {
                assert!(seat[..chapters].iter().all(|count| *count > 0));
            }
        }
    }
    println!(
        "Completed {games} Saga Standard games; {decisions} decisions/replays; {closed} parent-closed; {private} private frames; {denials} denials; initial{initial}/max{maximum} logical bytes; casts{casts:?}; effects{effects:?}; sacrifices{sacrifices:?}"
    );
    Ok(())
}
