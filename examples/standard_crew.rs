//! Legal Standard self-play with paid complete Vehicles and owned private Crew.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, Observation, Player, StackKind, Target,
    Zone,
};
const NAMES: [&str; 24] = [
    "Adventurer's Airship",
    "Air Response Unit",
    "Boosted Sloop",
    "Broadcast Rambler",
    "Burner Rocket",
    "Careening Mine Cart",
    "Clamorous Ironclad",
    "Cultivator's Caravan",
    "Debris Beetle",
    "Dependable Quinjet",
    "Fire Nation Warship",
    "Haunted Hellride",
    "Hulldrifter",
    "Lumbering Worldwagon",
    "Magitek Armor",
    "Ripclaw Wrangler",
    "S.H.I.E.L.D. Helicarrier",
    "Skybox Ferry",
    "Spotcycle Scouter",
    "Strixhaven Skycoach",
    "Thopter Fabricator",
    "Tundra Tank",
    "Turtle Blimp",
    "Veloheart Bike",
];
fn deck(i: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [
        ("Plains", 6),
        ("Island", 6),
        ("Swamp", 4),
        ("Mountain", 6),
        ("Forest", 10),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    for n in [
        NAMES[i],
        "Bear Cub",
        "Three Tree Mascot",
        "Chrome Companion",
        "Jibbirik Omnivore",
        "Thrill of Possibility",
        "Disenchant",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
#[derive(Default)]
struct Progress {
    source: Option<u32>,
    entry: Option<Observation>,
    entry_done: bool,
    crew: bool,
    attack: Option<Observation>,
    attack_done: bool,
    cycle: Option<(u64, u64)>,
    cycle_done: bool,
    mana_done: bool,
    draw: Option<(u64, usize)>,
    draw_done: bool,
    death: bool,
    death_done: bool,
    scry: usize,
}
fn tokens(o: &Observation, p: Player, sub: &str) -> usize {
    o.objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.card.token_info().is_some_and(|t| t.subtypes == sub)
        })
        .count()
}
fn lands(o: &Observation, p: Player) -> usize {
    o.objects
        .iter()
        .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_land())
        .count()
}
fn quiet(o: &Observation) -> bool {
    o.decision == "priority"
        && o.stack.is_empty()
        && o.announcing.is_none()
        && o.resolving.is_none()
}
fn complete(i: usize, p: &Progress) -> bool {
    p.entry_done
        && p.crew
        && p.attack_done
        && (!matches!(i, 6 | 17) || p.cycle_done)
        && (!matches!(i, 7 | 9 | 23) || p.mana_done)
        && (i != 20 || p.draw_done)
        && (i != 10 || p.death_done)
}
fn score(o: &Observation, a: &Action, i: usize, s: &[Progress; 2]) -> i32 {
    let p = o.actor.unwrap();
    let f = &s[p.index()];
    let obj = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let done = complete(i, f);
    let all_done = s.iter().all(|v| complete(i, v));
    let friends = o
        .objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.is_creature()
                && v.card.name() != NAMES[i]
        })
        .count();
    match a {
        Action::AnnounceCast { card }
        | Action::Cast { card, .. }
        | Action::BeginCast { card, .. } => {
            let v = obj(*card);
            match v.card.name() {
                n if n == NAMES[i] => {
                    if f.source.is_none()
                        && friends >= 3
                        && matches!(a, Action::AnnounceCast { .. })
                    {
                        1800
                    } else {
                        -1000
                    }
                }
                "Thrill of Possibility" => {
                    if i == 20 && f.entry_done && f.draw.is_none() && !f.draw_done {
                        2200
                    } else {
                        -1000
                    }
                }
                "Disenchant" => {
                    if i == 10 && f.attack_done && !f.death {
                        2300
                    } else {
                        -1000
                    }
                }
                "Bear Cub" | "Three Tree Mascot" | "Chrome Companion" | "Jibbirik Omnivore" => {
                    if friends < 4 || all_done {
                        1600
                    } else {
                        -1000
                    }
                }
                _ => -1000,
            }
        }
        Action::AnnounceActivate { card, ability, .. } => {
            let v = obj(*card);
            if v.card.name() != NAMES[i] {
                -1000
            } else if v.zone == Zone::Hand {
                if matches!(i, 6 | 17) && f.entry_done && f.cycle.is_none() && !f.cycle_done {
                    2200
                } else {
                    -1000
                }
            } else if !done
                && f.entry_done
                && f.attack.is_none()
                && !v.summoning_sick
                && !v.is_creature()
                && o.active == p
                && o.step == rustmage::Step::Main1
                && (!matches!(i, 7 | 9 | 23) || f.mana_done)
                && (i != 20 || f.draw_done)
                && (!matches!(i, 6 | 17) || f.cycle_done)
                && *ability == if matches!(i, 7 | 9 | 23) { 1 } else { 0 }
            {
                2000
            } else {
                -1000
            }
        }
        Action::ActivateMana { card, .. } => {
            if o.decision == "pay_costs" {
                return 2400;
            }
            if matches!(i, 7 | 9 | 23) && f.entry_done && !f.mana_done && Some(*card) == f.source {
                2400
            } else {
                -1000
            }
        }
        Action::ChooseCrewer { object } => 1700 + obj(object.card).power.clamp(-100, 100) as i32,
        Action::FinishCrew => 2600,
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            if i == 10 && f.attack_done && Some(*id) == f.source {
                2400
            } else if obj(*id).owner == p {
                1800
            } else {
                100
            }
        }
        Action::ChooseTarget {
            target: Target::Player(q),
        } => {
            if *q == p.other() {
                1800
            } else {
                100
            }
        }
        Action::Attack { card } => {
            if all_done {
                1400
            } else if f.crew
                && f.attack.is_none()
                && Some(*card) == f.source
                && s[p.other().index()].entry_done
            {
                2300
            } else if done && s[p.other().index()].attack_done {
                1400
            } else {
                -1000
            }
        }
        Action::ChooseAdditionalCost { object } => {
            if obj(object.card).card.name() == "Bear Cub" {
                2400
            } else {
                1800
            }
        }
        Action::ChooseOptional { accept } => {
            if *accept {
                2400
            } else {
                -1000
            }
        }
        Action::ChooseSearch { .. } => 2200,
        Action::FinishSearch => 2000,
        Action::PlayLand { card } => {
            1800 - 20
                * o.objects
                    .iter()
                    .filter(|v| {
                        v.owner == p && v.zone == Zone::Battlefield && v.card == obj(*card).card
                    })
                    .count() as i32
        }
        Action::TapMana { .. } => {
            if o.decision == "pay_costs" {
                2400
            } else {
                -1000
            }
        }
        Action::Discard { card } => {
            if obj(*card).card.is_land() {
                2000
            } else {
                1800
            }
        }
        Action::ArrangeLibrary { .. } => 2200,
        Action::PayCosts { .. } | Action::Keep => 2600,
        Action::PutTrigger { .. } => 2500,
        Action::FinishTargets => 2200,
        Action::FinishAttackers | Action::FinishBlockers => 1200,
        Action::Pass => 0,
        _ => -10000,
    }
}
fn verify_entry(i: usize, p: Player, b: &Observation, a: &Observation, f: &Progress) {
    let source = a
        .objects
        .iter()
        .find(|v| Some(v.id) == f.source && v.zone == Zone::Battlefield)
        .unwrap();
    assert!(!source.is_creature());
    assert_eq!((source.power, source.toughness), (0, 0));
    match i {
        3 | 14 | 16 | 22 => assert_eq!(
            tokens(
                a,
                p,
                match i {
                    3 => "Thopter",
                    14 => "Hero",
                    16 => "Soldier",
                    _ => "Mutant",
                }
            ),
            tokens(
                b,
                p,
                match i {
                    3 => "Thopter",
                    14 => "Hero",
                    16 => "Soldier",
                    _ => "Mutant",
                }
            ) + if i == 16 { 2 } else { 1 }
        ),
        4 => assert!(a.objects.iter().any(|v| v.owner == p
            && v.zone == Zone::Battlefield
            && v.is_creature()
            && v.keywords.contains(Keywords::TRAMPLE)
            && b.objects.iter().any(|old| old.reference == v.reference
                && v.power == old.power + 2
                && v.toughness == old.toughness))),
        8 => {
            assert_eq!(a.players[p.index()].life, b.players[p.index()].life + 3);
            assert_eq!(
                a.players[p.other().index()].life,
                b.players[p.other().index()].life - 3
            );
        }
        12 => assert_eq!(
            a.players[p.index()].cards_drawn_this_turn,
            b.players[p.index()].cards_drawn_this_turn + 2
        ),
        13 => assert_eq!(lands(a, p), lands(b, p) + 1),
        15 => assert_eq!(
            a.players[p.other().index()].cards_discarded_this_turn,
            b.players[p.other().index()].cards_discarded_this_turn + 1
        ),
        18 => assert_eq!(f.scry, 2),
        19 => assert!(a.known_hand.iter().any(|r| {
            a.objects
                .iter()
                .any(|v| v.reference == *r && v.owner == p && v.card.is_land())
        })),
        21 => assert!(a.objects.iter().any(|v| v.owner == p
            && v.zone == Zone::Battlefield
            && v.is_creature()
            && v.keywords.contains(Keywords::INDESTRUCTIBLE))),
        23 => assert_eq!(a.players[p.index()].life, b.players[p.index()].life + 2),
        _ => {}
    }
    let expected = match i {
        0 | 1 | 9 | 12 | 16 | 17 | 19 | 20 | 22 => Some(Keywords::FLYING),
        2 | 6 => Some(Keywords::MENACE),
        8 => Some(Keywords::TRAMPLE),
        10 => Some(Keywords::REACH),
        _ => None,
    };
    if let Some(k) = expected {
        assert!(source.keywords.contains(k), "{} keyword {:?}", NAMES[i], k);
    }
    if i == 1 {
        assert!(source.keywords.contains(Keywords::VIGILANCE));
    }
    if i == 4 {
        assert!(source.keywords.contains(Keywords::FLASH));
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut decisions, mut frames, mut denials, mut closed, mut initial, mut maximum) =
        (0u64, 0u64, 0u64, 0u64, 0usize, 0usize);
    let (
        mut casts,
        mut crews,
        mut entries,
        mut attacks,
        mut cycles,
        mut mana,
        mut draws,
        mut deaths,
    ) = (
        [[0u64; 2]; 24],
        [[0u64; 2]; 24],
        [[0u64; 2]; 24],
        [[0u64; 2]; 24],
        [[0u64; 2]; 24],
        [[0u64; 2]; 24],
        [[0u64; 2]; 24],
        [[0u64; 2]; 24],
    );
    for seed in 0..games {
        let i = seed % 24;
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
                let a = g.observe(q);
                let f = &mut s[q.index()];
                if quiet(&a) {
                    if !f.entry_done
                        && let Some(b) = &f.entry
                        && a.objects
                            .iter()
                            .any(|v| Some(v.id) == f.source && v.zone == Zone::Battlefield)
                    {
                        verify_entry(i, q, b, &a, f);
                        f.entry_done = true;
                        entries[i][q.index()] += 1;
                    }
                    if let Some((b, d)) = f.cycle.take() {
                        assert_eq!(a.players[q.index()].cards_drawn_this_turn, b + 1);
                        assert_eq!(a.players[q.index()].cards_discarded_this_turn, d + 1);
                        f.cycle_done = true;
                        cycles[i][q.index()] += 1;
                    }
                    if let Some((b, n)) = f.draw.take() {
                        assert_eq!(a.players[q.index()].cards_drawn_this_turn, b + 2);
                        assert_eq!(tokens(&a, q, "Thopter"), n + 1);
                        f.draw_done = true;
                        draws[i][q.index()] += 1;
                    }
                    if let Some(b) = &f.attack
                        && !f.attack_done
                    {
                        match i {
                            0 | 2 => {
                                assert_eq!(
                                    a.players[q.index()].cards_drawn_this_turn,
                                    b.players[q.index()].cards_drawn_this_turn + 1
                                );
                                assert_eq!(
                                    a.players[q.index()].cards_discarded_this_turn,
                                    b.players[q.index()].cards_discarded_this_turn + 1
                                );
                            }
                            5 => {
                                assert_eq!(tokens(&a, q, "Treasure"), tokens(b, q, "Treasure") + 1)
                            }
                            11 => assert!(a.objects.iter().any(|v| v.owner == q
                                && v.zone == Zone::Battlefield
                                && v.keywords.contains(Keywords::DEATHTOUCH)
                                && !v.tapped)),
                            13 => assert_eq!(lands(&a, q), lands(b, q) + 1),
                            21 => assert!(a.players[q.index()].mana[3] >= 1),
                            _ => {}
                        }
                        f.attack_done = true;
                        attacks[i][q.index()] += 1;
                    }
                    if f.death && !f.death_done {
                        assert_eq!(tokens(&a, q, "Clue"), 1);
                        assert!(a.objects.iter().any(|v| Some(v.id) == f.source
                            && v.zone == Zone::Graveyard
                            && !v.is_creature()));
                        f.death_done = true;
                        deaths[i][q.index()] += 1;
                    }
                }
            }
            let actions = g.legal_actions();
            let j = (0..actions.len())
                .max_by_key(|&j| (score(&o, &actions[j], i, &s), -(j as i64)))
                .unwrap();
            let action = actions[j].clone();
            assert!(
                score(&o, &action, i, &s) >= 0,
                "seed{seed} {:?} {:?}",
                o.decision,
                actions
            );
            if let Some(frame) = &o.crew_choice {
                frames += 1;
                assert!(g.observe(p.other()).crew_choice.is_none());
                assert!(o.payment_request.is_none());
                let before = g.snapshot();
                assert_eq!(
                    g.apply(p.other(), Action::FinishCrew),
                    Err(Error::WrongActor)
                );
                denials += 1;
                assert_eq!(
                    g.apply(
                        p,
                        Action::ChooseCrewer {
                            object: frame.source
                        }
                    ),
                    Err(Error::IllegalAction)
                );
                denials += 1;
                assert_eq!(g.snapshot(), before);
            }
            if let (Some(ann), Action::PayCosts { .. }) = (&o.announcing, &action) {
                if ann.source_card.name() == NAMES[i] {
                    if ann.kind == StackKind::Spell {
                        assert!(!ann.source_card.is_creature());
                        s[p.index()].source = Some(ann.card);
                        s[p.index()].entry = Some(o.clone());
                        casts[i][p.index()] += 1;
                    } else if o
                        .objects
                        .iter()
                        .any(|v| v.id == ann.card && v.zone == Zone::Hand)
                    {
                        s[p.index()].cycle = Some((
                            o.players[p.index()].cards_drawn_this_turn,
                            o.players[p.index()].cards_discarded_this_turn,
                        ));
                    }
                } else if ann.source_card.name() == "Thrill of Possibility" {
                    s[p.index()].draw = Some((
                        o.players[p.index()].cards_drawn_this_turn,
                        tokens(&o, p, "Thopter"),
                    ));
                } else if ann.source_card.name() == "Disenchant" {
                    s[p.index()].death = true;
                }
            }
            if let Action::ArrangeLibrary { .. } = action
                && s[p.index()].entry.is_some()
                && !s[p.index()].entry_done
            {
                s[p.index()].scry += 1;
            }
            if action == Action::FinishCrew {
                s[p.index()].crew = true;
                crews[i][p.index()] += 1;
            }
            if let Action::Attack { card } = action
                && Some(card) == s[p.index()].source
                && s[p.index()].attack.is_none()
            {
                assert!(
                    o.objects
                        .iter()
                        .find(|v| v.id == card)
                        .unwrap()
                        .is_creature()
                );
                s[p.index()].attack = Some(o.clone());
            }
            let saved = g.snapshot();
            let revision = g.revision();
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
            g.apply(p, action.clone())?;
            assert_eq!(g.snapshot(), after);
            decisions += 1;
            maximum = maximum.max(g.state_bytes());
            if let Action::ActivateMana {
                card, production, ..
            } = action
                && Some(card) == s[p.index()].source
                && !s[p.index()].mana_done
            {
                let a = g.observe(p);
                assert_eq!(
                    a.players[p.index()].mana,
                    std::array::from_fn(|k| o.players[p.index()].mana[k] + production[k])
                );
                assert!(a.objects.iter().find(|v| v.id == card).unwrap().tapped);
                s[p.index()].mana_done = true;
                mana[i][p.index()] += 1;
            }
        }
        assert!(g.outcome().is_some(), "unfinishedseed{seed}");
        assert!(
            s.iter().all(|f| complete(i, f)),
            "incomplete seed{seed} entries{:?} attacks{:?}",
            s.iter().map(|f| f.entry_done).collect::<Vec<_>>(),
            s.iter().map(|f| f.attack_done).collect::<Vec<_>>()
        );
    }
    if games >= 100 {
        for m in [&casts, &crews, &entries, &attacks] {
            assert!(m.iter().flatten().all(|v| *v > 0));
        }
        for i in [6, 17] {
            assert!(cycles[i].iter().all(|v| *v > 0));
        }
        for i in [7, 9, 23] {
            assert!(mana[i].iter().all(|v| *v > 0));
        }
        assert!(draws[20].iter().all(|v| *v > 0));
        assert!(deaths[10].iter().all(|v| *v > 0));
    }
    println!(
        "Completed{games}Crew Standard games;{decisions}decisions/replays;{closed}parent-closed;{frames}privateframes;{denials}denials;initial{initial}/max{maximum}logicalbytes; casts{casts:?}; Crew{crews:?}; entry{entries:?}; attacks{attacks:?}; Cycling{cycles:?}; mana{mana:?}; draw{draws:?}; deaths{deaths:?}"
    );
    Ok(())
}
