//! Legal Standard games with immutable live subject checks and owned replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, Observation, Player, StackKind, Target,
    Zone,
};
use std::collections::BTreeSet;
const NAMES: [&str; 9] = [
    "Aunt May",
    "Dreamdew Entrancer",
    "Fear of Immobility",
    "Hazardroot Herbalist",
    "Ingenious Leonin",
    "Little Bear",
    "Thwip!",
    "Wakandan Royal Guard",
    "Yip Yip!",
];
fn deck(i: usize) -> Vec<Card> {
    let mut d = vec![Card::from_name("Plains").unwrap(); 16];
    d.extend([Card::from_name("Forest").unwrap(); 12]);
    d.extend([Card::from_name("Island").unwrap(); 4]);
    for n in [
        NAMES[i],
        "Three Tree Mascot",
        "Bear Cub",
        "Llanowar Elves",
        "Hop to It",
        "Attercop",
        "Savannah Lions",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn score(o: &Observation, a: &Action, index: usize, seed: usize, fallback: bool) -> i32 {
    let wanted = NAMES[index];
    let actor = o.actor.unwrap();
    let object = |id| {
        o.objects.iter().find(|v| v.id == id).unwrap_or_else(|| {
            panic!(
                "missing object {id}, action {a:?}, actor {actor:?}, decision {}",
                o.decision
            )
        })
    };
    let source_up = o
        .objects
        .iter()
        .any(|v| v.card.name() == wanted && v.owner == actor && v.zone == Zone::Battlefield);
    let positive = seed.is_multiple_of(2);
    let target_owner = if index == 1 && !positive || index == 2 && positive {
        actor.other()
    } else {
        actor
    };
    let desired = if index == 5 && positive {
        "Bear Cub"
    } else if positive {
        "Three Tree Mascot"
    } else {
        "Llanowar Elves"
    };
    let ready = o.objects.iter().any(|v| {
        v.card.name() == desired && v.owner == target_owner && v.zone == Zone::Battlefield
    });
    match a {
        Action::Cast { card, .. } | Action::AnnounceCast { card } => {
            let name = object(*card).card.name();
            if name == wanted {
                if (index == 0 || ready) && !(matches!(index, 0 | 4) && source_up) {
                    800
                } else {
                    -100
                }
            } else if name == "Hop to It" {
                if index == 3 && source_up { 750 } else { -100 }
            } else if index == 4 && source_up && ready {
                -100
            } else if name == desired {
                650
            } else {
                550
            }
        }
        Action::Activate { card, .. }
        | Action::BeginActivate { card, .. }
        | Action::AnnounceActivate { card, .. } => {
            if index == 4 && object(*card).card.name() == wanted {
                950
            } else {
                -100
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = object(*id);
            if index == 3 && positive {
                if v.card.is_token() && v.owner == actor {
                    1100
                } else {
                    100
                }
            } else if index == 7 && (seed / 9).is_multiple_of(3) && v.card.name() == wanted {
                1100
            } else if v.card.name() == desired && v.owner == target_owner {
                1000
            } else {
                200
            }
        }
        Action::PlayLand { card } => {
            let name = object(*card).card.name();
            let count = o
                .objects
                .iter()
                .filter(|v| {
                    v.card.name() == name && v.owner == actor && v.zone == Zone::Battlefield
                })
                .count();
            900 - count as i32
        }
        Action::Attack { card } => {
            if ((index == 3 || index == 4) && source_up && object(*card).card.name() != wanted)
                || o.turn > 24
            {
                500
            } else {
                -1000
            }
        }
        Action::Block { .. } | Action::Concede => -1000,
        Action::TapMana { .. } | Action::ActivateMana { .. } | Action::AnnounceMana { .. } => {
            if o.decision == "pay_costs" { 400 } else { -100 }
        }
        Action::PayCosts { .. } => 1100,
        Action::PutTrigger { .. } => 850,
        Action::FinishTargets => 700,
        Action::FinishAttackers | Action::FinishBlockers => 400,
        Action::Keep => 1200,
        Action::Mulligan => -100,
        _ => {
            if fallback {
                100
            } else {
                0
            }
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut decisions, mut replays, mut rejects, mut max_bytes) = (0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut effects, mut yes, mut no, mut seats) =
        ([0u64; 9], [0u64; 9], [0u64; 9], [0u64; 9], [0u64; 2]);
    let mut body_controllers = [[0u64; 2]; 9];
    let mut seen = BTreeSet::new();
    let (
        mut aunt_life,
        mut drawn,
        mut taps,
        mut untaps,
        mut self_one,
        mut other_two,
        mut token_checks,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    for seed in 0..games {
        let index = seed % 9;
        let mut g = Game::new(Config {
            decks: [deck(index), deck(index)],
            seed: seed as u64,
            starting_player: if seed.is_multiple_of(2) {
                Player::P0
            } else {
                Player::P1
            },
            deck_rules: DeckRules::Standard,
        })?;
        for _ in 0..100000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let fallback = rustmage::policy::smoke_index(&g, &before.actions);
            let j = before
                .actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| score(&before, a, index, seed, *j == fallback))
                .unwrap()
                .0;
            let a = before.actions[j].clone();
            for packet in before.stack.iter().chain(before.announcing.iter()) {
                if packet.kind == StackKind::Spell
                    && packet.source().is_some_and(|source| source.card.name() == NAMES[index])
                    && seen.insert((seed, packet.id))
                {
                    casts[index] += 1;
                }
            }
            if before.decision == "targets" {
                let s = g.snapshot();
                let rev = g.revision();
                assert_eq!(
                    g.apply(
                        actor,
                        Action::ChooseTarget {
                            target: Target::Card(u32::MAX)
                        }
                    ),
                    Err(Error::IllegalAction)
                );
                assert_eq!(g.snapshot(), s);
                assert_eq!(g.revision(), rev);
                assert_eq!(g.apply(actor.other(), a.clone()), Err(Error::WrongActor));
                assert_eq!(g.snapshot(), s);
                rejects += 2;
            }
            let packet = before.stack.last().filter(|s| {
                s.source().is_some_and(|source| source.card.name() == NAMES[index])
                    && (matches!(index, 6 | 8) || s.kind != StackKind::Spell)
            });
            let subject = packet
                .and_then(|s| {
                    if index == 0 {
                        s.entering_object
                    } else {
                        s.targets.first().and_then(|t| t.object)
                    }
                })
                .and_then(|r| before.objects.iter().find(|v| v.reference == r));
            let saved = g.snapshot();
            let revision = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, a.clone())?;
            g.apply_index(actor, revision, j)?;
            assert_eq!(g.snapshot(), fork.snapshot());
            let expected = g.snapshot();
            g.restore(&saved)?;
            assert_eq!(g.apply_index(actor, revision, j), Err(Error::StaleRevision));
            g.apply(actor, a)?;
            assert_eq!(g.snapshot(), expected);
            decisions += 1;
            replays += 1;
            max_bytes = max_bytes.max(g.state_bytes());
            let after = g.observe(actor);
            if let Some(packet) = packet
                && !after.stack.iter().any(|s| s.id == packet.id)
                && after.resolving.is_none()
                && let Some(old) = subject
                && let Some(new) = after.objects.iter().find(|v| v.reference == old.reference)
            {
                let own_index = packet.controller.index();
                let plus = new.counters.plus_one - old.counters.plus_one;
                let condition = match index {
                    0 | 6 => old.card.has_subtype("Spider"),
                    1 => old.owner == packet.controller,
                    2 => old.owner != packet.controller,
                    3 => old.card.is_token(),
                    4 => old.card.has_subtype("Cat"),
                    5 => old.card.has_subtype("Bear"),
                    7 => old.card.has_subtype("Hero") && old.reference != packet.object,
                    8 => old.card.has_subtype("Ally"),
                    _ => unreachable!(),
                };
                effects[index] += 1;
                seats[own_index] += 1;
                body_controllers[index][own_index] += 1;
                if condition {
                    yes[index] += 1
                } else {
                    no[index] += 1
                }
                match index {
                    0 => {
                        assert_eq!(
                            after.players[own_index].life,
                            before.players[own_index].life + 1
                        );
                        assert_eq!(plus, u64::from(condition));
                        aunt_life += 1;
                    }
                    1 => {
                        assert!(new.tapped);
                        assert_eq!(new.counters.stun - old.counters.stun, 3);
                        let count = if condition {
                            before.players[own_index].library_count.min(2)
                        } else {
                            0
                        };
                        assert_eq!(
                            after.players[own_index].hand_count
                                - before.players[own_index].hand_count,
                            count
                        );
                        drawn += count as u64;
                        taps += 1;
                    }
                    2 => {
                        assert!(new.tapped);
                        assert_eq!(new.counters.stun - old.counters.stun, u64::from(condition));
                        taps += 1;
                    }
                    3 => {
                        assert_eq!(new.power, old.power + 1);
                        assert_eq!(new.keywords.contains(Keywords::DEATHTOUCH), condition);
                        token_checks += u64::from(condition);
                    }
                    4 => {
                        assert_eq!(plus, 1);
                        assert_eq!(new.keywords.contains(Keywords::FIRST_STRIKE), condition);
                    }
                    5 => {
                        assert!(!new.tapped || old.counters.stun > 0);
                        assert_eq!(plus, u64::from(condition));
                        untaps += u64::from(old.tapped && !new.tapped);
                    }
                    6 => {
                        assert_eq!(
                            (new.power - old.power, new.toughness - old.toughness),
                            (2, 2)
                        );
                        assert!(new.keywords.contains(Keywords::FLYING));
                        assert_eq!(
                            after.players[own_index].life - before.players[own_index].life,
                            if condition { 2 } else { 0 }
                        );
                    }
                    7 => {
                        assert_eq!(plus, if condition { 2 } else { 1 });
                        if old.reference == packet.object {
                            self_one += 1;
                        } else if condition {
                            other_two += 1;
                        }
                    }
                    8 => {
                        assert_eq!(
                            (new.power - old.power, new.toughness - old.toughness),
                            (2, 2)
                        );
                        assert_eq!(
                            new.keywords.contains(Keywords::FLYING),
                            condition || old.keywords.contains(Keywords::FLYING)
                        );
                    }
                    _ => unreachable!(),
                }
            }
        }
        assert!(g.outcome().is_some(), "unfinished seed {seed}");
    }
    if games >= 100 {
        assert!(casts.iter().all(|&n| n > 0));
        assert!(effects.iter().all(|&n| n > 0));
        assert!(yes.iter().all(|&n| n > 0));
        assert!(no.iter().all(|&n| n > 0));
        assert!(seats.iter().all(|&n| n > 0));
        assert!(
            body_controllers
                .iter()
                .all(|pair| pair.iter().all(|&n| n > 0))
        );
        assert!(self_one > 0 && other_two > 0 && token_checks > 0 && drawn > 0 && untaps > 0);
    }
    println!(
        "Completed {games} subject-condition Standard games, {decisions} decisions/{replays} replays, casts {casts:?}, effects {effects:?}, positive {yes:?}, negative {no:?}, controllers {seats:?}, {aunt_life} Aunt life triggers, {drawn} conditional draws, {taps} taps, {untaps} real untaps, {self_one} self-one/{other_two} other-Hero-two counter operations, {token_checks} token checks, {rejects} illegal rejections, {max_bytes} max logical bytes"
    );
    Ok(())
}
