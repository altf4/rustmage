//! Paid evidence transactions and complete bodies in legal Standard games.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, Observation, Player,
    StackKind, Target, Zone,
};
const NAMES: [&str; 10] = [
    "Analyze the Pollen",
    "Axebane Ferox",
    "Bite Down on Crime",
    "Crimestopper Sprite",
    "Evidence Examiner",
    "Extract a Confession",
    "Forensic Researcher",
    "Sample Collector",
    "Surveillance Monitor",
    "Vitu-Ghazi Inspector",
];
const BASICS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
fn card(n: &str) -> Card {
    Card::from_name(n).unwrap()
}
fn deck(i: usize, focal: bool) -> Vec<Card> {
    let mut d = Vec::new();
    let lands = if focal {
        [
            ("Plains", 2),
            ("Island", 8),
            ("Swamp", 4),
            ("Mountain", 8),
            ("Forest", 10),
            ("Evolving Wilds", 4),
        ]
    } else {
        [
            ("Plains", 0),
            ("Island", 0),
            ("Swamp", 0),
            ("Mountain", 16),
            ("Forest", 20),
            ("Evolving Wilds", 4),
        ]
    };
    for (n, k) in lands {
        d.extend(std::iter::repeat_n(card(n), k));
    }
    let support = if focal {
        vec![
            NAMES[i],
            "Hamlet Glutton",
            "Thrill of Possibility",
            "Bear Cub",
            "Candy Trail",
            "Corrupted Conviction",
        ]
    } else {
        vec![
            "Hamlet Glutton",
            "Thrill of Possibility",
            "Bear Cub",
            "Candy Trail",
            "Shock",
        ]
    };
    for n in support {
        d.extend([card(n); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn mana(c: Card, pay: bool) -> ([u32; 6], u32) {
    let (mut r, mut g) = ([0; 6], 0);
    for s in c
        .info()
        .mana_cost
        .split('{')
        .skip(1)
        .map(|v| v.trim_end_matches('}'))
    {
        if let Ok(n) = s.parse::<u32>() {
            g += n
        } else {
            r["WUBRGC".find(s).unwrap()] += 1;
        }
    }
    if pay && c.name() == "Bite Down on Crime" {
        g -= 2;
    }
    (r, g)
}
fn value(c: Card) -> u128 {
    let (r, g) = mana(c, false);
    u128::from(g) + r.into_iter().map(u128::from).sum::<u128>()
}
#[derive(Default)]
struct Pilot {
    paid: bool,
    source: Option<ObjectRef>,
    done: bool,
    choice_done: bool,
    ward_seen: bool,
    untap_paid: bool,
    untap_done: bool,
    tap_paid: bool,
    untap_target: Option<ObjectRef>,
    sacrificed: Option<(ObjectRef, Card)>,
    search_success: bool,
}
fn score(o: &Observation, a: &Action, i: usize, p: Player, pay: bool, s: &Pilot) -> i32 {
    let actor = o.actor.unwrap();
    let other = p.other();
    let object = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let has = |who, n: &str| {
        o.objects
            .iter()
            .any(|v| v.owner == who && v.zone == Zone::Battlefield && v.card.name() == n)
    };
    let grave = |who| {
        o.objects
            .iter()
            .filter(|v| v.owner == who && v.zone == Zone::Graveyard)
            .map(|v| value(v.card))
            .sum::<u128>()
    };
    let n = if i == 0 {
        8
    } else if [2, 3, 5, 9].contains(&i) {
        6
    } else if [1, 4, 8].contains(&i) {
        4
    } else {
        3
    };
    let required = mana(card(NAMES[i]), pay);
    let mut potential = o.players[p.index()].mana;
    for v in &o.objects {
        if v.owner == p
            && v.zone == Zone::Battlefield
            && !v.tapped
            && let Some(k) = BASICS.iter().position(|n| *n == v.card.name())
        {
            potential[k] += 1;
        }
    }
    let fundable = potential.iter().zip(required.0).all(|(a, b)| *a >= b)
        && potential.iter().sum::<u32>() >= required.0.iter().sum::<u32>() + required.1;
    let ready = fundable
        && (i == 1 || !(pay || [4, 6, 7, 8].contains(&i)) || grave(p) >= n)
        && (![2, 3, 7, 9].contains(&i) || has(p, "Bear Cub"))
        && (![2, 5, 6].contains(&i) || has(other, "Bear Cub"))
        && (i != 5 || has(other, "Hamlet Glutton"));
    let tapped_land = || {
        o.objects
            .iter()
            .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_land() && v.tapped)
    };
    match a {
        Action::AnnounceCast { card: id } | Action::Cast { card: id, .. } => {
            let name = object(*id).card.name();
            if actor == other {
                match name {
                    "Bear Cub" => {
                        if !has(other, "Bear Cub") || s.done {
                            900
                        } else {
                            -1000
                        }
                    }
                    "Hamlet Glutton" => {
                        if i == 5 && !has(other, "Hamlet Glutton") || s.done {
                            950
                        } else {
                            -1000
                        }
                    }
                    "Thrill of Possibility" => {
                        if i == 1
                            && !s.done
                            && grave(other) < 4
                            && o.objects.iter().any(|v| {
                                v.owner == other
                                    && v.zone == Zone::Hand
                                    && v.card.name() == "Hamlet Glutton"
                            })
                        {
                            1100
                        } else {
                            -1000
                        }
                    }
                    "Shock" => {
                        if i == 1 && has(p, NAMES[i]) && !s.ward_seen && grave(other) >= 4 {
                            1300
                        } else {
                            -1000
                        }
                    }
                    "Candy Trail" if s.done => 700,
                    _ => -1000,
                }
            } else if name == NAMES[i] {
                if ready && !s.paid && matches!(a, Action::AnnounceCast { .. }) {
                    1300
                } else {
                    -1000
                }
            } else {
                match name {
                    "Bear Cub" => {
                        if [2, 3, 7, 9].contains(&i) && !has(p, "Bear Cub") || s.done {
                            850
                        } else {
                            -1000
                        }
                    }
                    "Thrill of Possibility" => {
                        if (pay || [4, 6, 7, 8].contains(&i))
                            && grave(p) < n
                            && o.objects.iter().any(|v| {
                                v.owner == p
                                    && v.zone == Zone::Hand
                                    && v.card.name() == "Hamlet Glutton"
                            })
                        {
                            1100
                        } else {
                            -1000
                        }
                    }
                    "Candy Trail" if s.done => 700,
                    _ => -1000,
                }
            }
        }
        Action::ChooseCollectEvidence { collect } => {
            if *collect == pay {
                1800
            } else {
                -1000
            }
        }
        Action::ChooseEvidence { object: r } => {
            let e = o.evidence.as_ref().unwrap();
            let zero = value(object(r.card).card) == 0;
            let has_zero = e.selected.iter().any(|r| value(object(r.card).card) == 0);
            if pay || i == 6 {
                if e.total_mana >= u128::from(e.threshold) && zero && !has_zero {
                    2100
                } else {
                    1600 + value(object(r.card).card).min(100) as i32
                }
            } else {
                -1000
            }
        }
        Action::FinishEvidence => 1900,
        Action::DeclineEvidence => {
            if !pay && i != 6 {
                2000
            } else {
                -1000
            }
        }
        Action::ChooseBargain { sacrifice } => {
            if sacrifice.is_none() {
                1800
            } else {
                1000
            }
        }
        Action::ChooseAdditionalCost { object: r } => {
            if object(r.card).card.name() == "Hamlet Glutton" {
                1800
            } else {
                1000
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = object(*id);
            let ann = o.announcing.as_ref().unwrap();
            if ann.source().is_some_and(|source| source.card.name() == "Shock") {
                if v.owner == p && v.card.name() == NAMES[i] {
                    1800
                } else {
                    -1000
                }
            } else if ann.source().is_some_and(|source| source.card.name() == "Forensic Researcher") {
                if ann.kind == (StackKind::ActivatedAbility { ability: 0 })
                    && v.owner == p
                    && v.card.is_land()
                    && v.tapped
                    || ann.kind == (StackKind::ActivatedAbility { ability: 1 })
                        && v.owner == other
                        && v.card.name() == "Bear Cub"
                {
                    1800
                } else {
                    -1000
                }
            } else if v.owner == p && v.card.name() == "Bear Cub" {
                1800
            } else if v.owner == other && v.card.name() == "Bear Cub" {
                1600
            } else {
                100
            }
        }
        Action::ChooseTarget {
            target: Target::Player(who),
        } => {
            if *who == other {
                1700
            } else {
                0
            }
        }
        Action::AnnounceActivate {
            card: id, ability, ..
        }
        | Action::BeginActivate {
            card: id, ability, ..
        }
        | Action::Activate {
            card: id, ability, ..
        } => {
            if object(*id).card.name() == "Evolving Wilds" {
                1500
            } else if i == 6
                && actor == p
                && object(*id).card.name() == NAMES[i]
                && *ability == u16::from(s.untap_done)
                && (s.untap_done || tapped_land())
            {
                1550
            } else {
                -1000
            }
        }
        Action::TapMana { .. } => {
            if o.decision == "pay_costs" {
                1100
            } else if i == 6 && actor == p && has(p, NAMES[i]) && !s.untap_done && !tapped_land() {
                1300
            } else {
                -1000
            }
        }
        Action::ChooseSearch { card: id } => {
            let v = o
                .search_choice
                .as_ref()
                .unwrap()
                .cards
                .iter()
                .find(|v| v.id == *id)
                .unwrap();
            if o.resolving
                .as_ref()
                .is_some_and(|r| r.source().is_some_and(|source| source.card.name() == NAMES[0]))
                && pay
                && v.card.is_creature()
            {
                1700
            } else {
                let k = BASICS.iter().position(|n| *n == v.card.name());
                let have = o
                    .objects
                    .iter()
                    .filter(|b| b.zone == Zone::Battlefield && b.owner == actor && b.card == v.card)
                    .count() as i32;
                1600 - have * 10
                    + k.map_or(0, |k| if required.0[k] > have as u32 { 100 } else { 0 })
            }
        }
        Action::FinishSearch => 1500,
        Action::Sacrifice { object: r } => {
            if object(r.card).card.name() == if pay { "Hamlet Glutton" } else { "Bear Cub" } {
                1800
            } else {
                1000
            }
        }
        Action::PlayLand { card: id } => {
            let c = object(*id).card;
            let have = o
                .objects
                .iter()
                .filter(|v| v.owner == actor && v.zone == Zone::Battlefield && v.card == c)
                .count();
            if c.name() == "Evolving Wilds" {
                1550
            } else {
                1400 - have as i32 * 10
            }
        }
        Action::Discard { card: id } => {
            if object(*id).card.name() == "Hamlet Glutton" {
                1700
            } else if object(*id).card.is_land() {
                1500
            } else if object(*id).card.name() == NAMES[i] {
                100
            } else {
                1000
            }
        }
        Action::Attack { card: id } => {
            if s.done
                || i == 7
                    && object(*id).owner == p
                    && object(*id).card.name() == NAMES[i]
                    && !s.choice_done
            {
                1400
            } else {
                -1000
            }
        }
        Action::ArrangeLibrary { .. } => 1000,
        Action::Keep => 2000,
        Action::PayCosts { .. } => 1800,
        Action::PutTrigger { .. } => 1700,
        Action::FinishTargets => 1400,
        Action::FinishAttackers | Action::FinishBlockers => 700,
        Action::Pass => 0,
        Action::Concede | Action::Mulligan | Action::Block { .. } => -10000,
        _ => -1000,
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut casts, mut effects) = ([[[0u64; 2]; 2]; 10], [[[0u64; 2]; 2]; 10]);
    let (
        mut decisions,
        mut replays,
        mut frames,
        mut rejected,
        mut collected,
        mut exiled,
        mut extras,
        mut wards,
        mut declines,
        mut searches,
        mut untaps,
        mut taps,
        mut reflexives,
        mut clues,
        mut thopters,
    ) = (
        0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64,
    );
    let (mut initial, mut maximum) = (0, 0);
    for seed in 0..games {
        let i = seed % 10;
        let round = seed / 10;
        let p = if round.is_multiple_of(2) {
            Player::P0
        } else {
            Player::P1
        };
        let pay = (round / 2) % 2 == 1;
        let a = deck(i, true);
        let b = deck(i, false);
        let mut g = Game::new(Config {
            decks: if p == Player::P0 { [a, b] } else { [b, a] },
            seed: seed as u64,
            starting_player: p,
            deck_rules: DeckRules::Constructed,
        })?;
        if seed == 0 {
            initial = g.state_bytes();
        }
        let mut s = Pilot::default();
        let mut sequence = 0;
        for _ in 0..20000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let o = g.observe(actor);
            let aa = g.legal_actions();
            let object = |id| o.objects.iter().find(|v| v.id == id).unwrap();
            if let Some(e) = &o.evidence {
                frames += 1;
                assert!(g.observe(actor.other()).evidence.is_none());
                assert!(g.observe(actor.other()).actions.is_empty());
                for r in &e.selected {
                    let v = object(r.card);
                    assert_eq!(v.reference, *r);
                    assert_eq!(v.zone, Zone::Graveyard);
                }
                if i == 1
                    && o.resolving
                        .as_ref()
                        .is_some_and(|r| r.source().is_some_and(|source| source.card.name() == NAMES[i]))
                {
                    s.ward_seen = true;
                    assert_eq!(actor, p.other());
                    wards += 1;
                }
                let before = g.snapshot();
                let rev = g.revision();
                assert_eq!(
                    g.apply(
                        actor,
                        Action::ChooseEvidence {
                            object: ObjectRef {
                                card: u32::MAX,
                                generation: 0
                            }
                        }
                    ),
                    Err(Error::IllegalAction)
                );
                assert_eq!(g.snapshot(), before);
                assert_eq!(g.revision(), rev);
                let valid = aa
                    .iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap()
                    .clone();
                assert_eq!(g.apply(actor.other(), valid), Err(Error::WrongActor));
                assert_eq!(g.snapshot(), before);
                rejected += 2;
            }
            let j = (0..aa.len())
                .max_by_key(|j| (score(&o, &aa[*j], i, p, pay, &s), std::cmp::Reverse(*j)))
                .unwrap();
            let action = aa[j].clone();
            if let Action::PayCosts { .. } = action
                && let Some(ann) = &o.announcing
                && ann.source().is_some_and(|source| source.card.name() == NAMES[i])
            {
                if ann.kind == StackKind::Spell {
                    s.paid = true;
                    s.source = Some(ann.source().expect("example expects an object-backed packet").object);
                    casts[i][usize::from(pay)][p.index()] += 1;
                    assert_eq!(
                        ann.cast.unwrap().evidence,
                        if [0, 2, 3, 5, 9].contains(&i) {
                            Some(pay)
                        } else {
                            None
                        }
                    );
                } else if i == 6 {
                    if ann.kind == (StackKind::ActivatedAbility { ability: 0 }) {
                        s.untap_paid = true;
                        s.untap_target = ann.targets[0].object;
                    } else if ann.kind == (StackKind::ActivatedAbility { ability: 1 }) {
                        s.tap_paid = true;
                    }
                }
            }
            if matches!(action, Action::FinishEvidence | Action::DeclineEvidence)
                && o.resolving
                    .as_ref()
                    .is_some_and(|r| r.source().is_some_and(|source| source.card.name() == NAMES[i]))
            {
                s.choice_done = true;
                if action == Action::DeclineEvidence {
                    declines += 1;
                }
            }
            if let Action::Sacrifice { object: r } = action
                && o.resolving
                    .as_ref()
                    .is_some_and(|v| v.source().is_some_and(|source| source.card.name() == NAMES[i]))
            {
                s.sacrificed = Some((r, object(r.card).card));
            }
            let search = if action == Action::FinishSearch
                && o.resolving
                    .as_ref()
                    .is_some_and(|r| r.source().is_some_and(|source| source.card.name() == NAMES[0]))
            {
                o.search_choice
                    .as_ref()
                    .and_then(|f| {
                        f.selected
                            .first()
                            .and_then(|id| f.cards.iter().find(|v| v.id == *id))
                    })
                    .map(|v| v.reference)
            } else {
                None
            };
            let saved = g.snapshot();
            let revision = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, action.clone())?;
            g.apply(actor, action.clone())?;
            let end = g.snapshot();
            assert_eq!(fork.snapshot(), end);
            g.restore(&saved)?;
            assert_eq!(g.apply_index(actor, revision, j), Err(Error::StaleRevision));
            g.apply(actor, action)?;
            assert_eq!(g.snapshot(), end);
            replays += 1;
            decisions += 1;
            maximum = maximum.max(g.state_bytes());
            let v = g.observe(p);
            assert_eq!(v.schema, rustmage::OBSERVATION_SCHEMA);
            assert_eq!(v.last_evidence, g.observe(p.other()).last_evidence);
            if let Some(e) = &v.last_evidence
                && e.sequence > sequence
            {
                sequence = e.sequence;
                collected += 1;
                exiled += e.exiled.len() as u64;
                extras += e
                    .exiled
                    .iter()
                    .filter(|r| {
                        v.objects
                            .iter()
                            .find(|v| v.reference == **r)
                            .is_some_and(|v| value(v.card) == 0)
                    })
                    .count() as u64;
                assert!(e.total_mana >= u128::from(e.threshold));
                for r in &e.exiled {
                    assert_eq!(
                        v.objects.iter().find(|v| v.reference == *r).unwrap().zone,
                        Zone::Exile
                    );
                }
            }
            if let Some(r) = search {
                let moved = v.objects.iter().find(|v| v.id == r.card).unwrap();
                assert_eq!(moved.zone, Zone::Hand);
                assert_eq!(moved.reference.generation, r.generation + 1);
                assert!(v.known_hand.contains(&moved.reference));
                s.search_success = true;
                searches += 1;
            }
            if s.untap_paid
                && !s.untap_done
                && s.untap_target.is_some_and(|r| {
                    v.objects
                        .iter()
                        .find(|v| v.reference == r)
                        .is_some_and(|v| !v.tapped)
                })
            {
                s.untap_done = true;
                untaps += 1;
            }
            let idle = v.stack.is_empty()
                && v.announcing.is_none()
                && v.resolving.is_none()
                && v.waiting_triggers.is_empty();
            let body = v
                .objects
                .iter()
                .find(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == NAMES[i]);
            let friend = v.objects.iter().find(|v| {
                v.owner == p && v.zone == Zone::Battlefield && v.card.name() == "Bear Cub"
            });
            let foe = v.objects.iter().find(|v| {
                v.owner == p.other() && v.zone == Zone::Battlefield && v.card.name() == "Bear Cub"
            });
            if !s.done && s.paid && idle {
                let done = match i {
                    0 => s.search_success,
                    1 => {
                        s.ward_seen
                            && body.is_some_and(|b| {
                                assert_eq!((b.power, b.toughness), (4, 4));
                                assert!(b.keywords.contains(Keywords::HASTE));
                                assert!(b.keywords.contains(Keywords::DEATHTOUCH));
                                b.damage == if pay { 2 } else { 0 }
                            })
                    }
                    2 => {
                        s.source.is_some_and(|r| {
                            v.objects
                                .iter()
                                .find(|v| v.id == r.card)
                                .is_some_and(|v| v.zone == Zone::Graveyard)
                        }) && friend.is_some_and(|v| v.power == 4)
                            && v.objects.iter().any(|v| {
                                v.owner == p.other()
                                    && v.zone == Zone::Graveyard
                                    && v.card.name() == "Bear Cub"
                            })
                    }
                    3 => {
                        body.is_some()
                            && friend.is_some_and(|v| v.tapped && v.counters.stun == u64::from(pay))
                    }
                    4 | 8 => {
                        let count = v
                            .objects
                            .iter()
                            .filter(|v| {
                                v.owner == p
                                    && v.zone == Zone::Battlefield
                                    && v.token.is_some()
                                    && v.card.has_subtype(if i == 4 { "Clue" } else { "Thopter" })
                            })
                            .count();
                        if s.choice_done && body.is_some() && count == usize::from(pay) {
                            if i == 4 {
                                clues += count as u64
                            } else {
                                thopters += count as u64;
                            }
                            true
                        } else {
                            false
                        }
                    }
                    5 => s.sacrificed.is_some_and(|(r, c)| {
                        assert_eq!(c.name(), if pay { "Hamlet Glutton" } else { "Bear Cub" });
                        v.objects
                            .iter()
                            .find(|v| v.id == r.card)
                            .is_some_and(|v| v.zone == Zone::Graveyard)
                    }),
                    6 => {
                        s.tap_paid
                            && s.untap_done
                            && foe.is_some_and(|v| v.tapped)
                            && body.is_some_and(|v| v.tapped)
                    }
                    7 => {
                        s.choice_done
                            && friend.is_some_and(|v| v.counters.plus_one == u64::from(pay))
                    }
                    9 => {
                        body.is_some()
                            && friend.is_some_and(|v| v.counters.plus_one == u64::from(pay))
                            && v.players[p.index()].life == 20 + if pay { 2 } else { 0 }
                    }
                    _ => false,
                };
                if done {
                    s.done = true;
                    effects[i][usize::from(pay)][p.index()] += 1;
                    if i == 6 {
                        taps += 1;
                    }
                    if i == 7 && pay {
                        reflexives += 1;
                    }
                }
            }
        }
        assert!(g.outcome().is_some(), "unfinished {seed}");
    }
    if games >= 100 {
        assert!(
            casts.iter().flatten().flatten().all(|v| *v > 0),
            "casts {casts:?}"
        );
        assert!(
            effects.iter().flatten().flatten().all(|v| *v > 0),
            "effects {effects:?}"
        );
        assert!(
            wards > 0
                && declines > 0
                && searches > 0
                && untaps > 0
                && taps > 0
                && reflexives > 0
                && clues > 0
                && thopters > 0
                && extras > 0
        );
    }
    println!(
        "Completed {games} evidence Standard games: {decisions} decisions/{replays} replay boundaries; casts[card][profile][seat] {casts:?}; full effects {effects:?}; {collected} actual collections/{exiled} exiled cards/{extras} zero-MV extras, {frames} private frames/{rejected} invalid denials, {wards} ward frames/{declines} optional declines, {searches} revealed search puts/{untaps} actual untaps/{taps} evidence taps/{reflexives} reflexives/{clues} Clues/{thopters} Thopters; initial {initial}/max {maximum} logical bytes"
    );
    Ok(())
}
