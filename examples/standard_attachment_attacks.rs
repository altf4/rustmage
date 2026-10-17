//! Actual Standard paid Equipment casts, attachment attacks and owned search replay.
//!
//! Run `cargo run --release --example standard_attachment_attacks -- 100`.
//! The arrays use NAMES order; each card must witness casts, paid equips and
//! resolved attacks under both controllers. No state injection or private API.
//! A round changes the starter, preferred recipient, optional target policy,
//! Pirate equip mode and complete two-card surveil arrangement independently.

use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, LibraryDestination, ObjectRef, Observation,
    Player, SpellView, StackKind, Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NAMES: [&str; 10] = [
    "Baseball Bat",
    "Captain America's Shield",
    "Stormbeacon Blade",
    "Thunder Lasso",
    "Ultima Weapon",
    "Candlestick",
    "Diamond Pick-Axe",
    "Medic's Kitesail",
    "Pirate Hat",
    "Web-Shooters",
];
const BONUS: [(i128, i128); 10] = [
    (1, 1),
    (0, 8),
    (3, 0),
    (1, 1),
    (7, 7),
    (1, 1),
    (1, 1),
    (1, 0),
    (1, 1),
    (1, 1),
];
const EQUIP: [u32; 10] = [3, 2, 2, 2, 7, 2, 2, 2, 2, 2];
fn index(card: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == card.name())
}
// Each fixed 60-card deck obeys Standard legality and the four-copy limit.
fn deck(i: usize) -> Vec<Card> {
    let mut d = vec![Card::from_name("Plains").unwrap(); 10];
    d.extend([Card::from_name("Island").unwrap(); 6]);
    d.extend([Card::from_name("Forest").unwrap(); 6]);
    d.extend([Card::from_name("Mountain").unwrap(); 5]);
    d.extend([Card::from_name("Swamp").unwrap(); 5]);
    for name in [
        NAMES[i],
        "Bear Cub",
        "Three Tree Mascot",
        "Pulse Tracker",
        if i == 7 {
            "Diamond Pick-Axe"
        } else {
            "Medic's Kitesail"
        },
        if i == 5 {
            "Web-Shooters"
        } else {
            "Candlestick"
        },
        "Hop to It",
    ] {
        d.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn packets(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(o.announcing.iter())
        .chain(o.resolving.iter())
}
fn cost(card: Card) -> ([u32; 6], u32) {
    let (mut colored, mut generic) = ([0; 6], 0);
    for symbol in card
        .info()
        .mana_cost
        .split('{')
        .skip(1)
        .map(|s| s.trim_end_matches('}'))
    {
        if let Ok(n) = symbol.parse::<u32>() {
            generic += n;
        } else {
            colored["WUBRGC".find(symbol).unwrap()] += 1;
        }
    }
    (colored, generic)
}
fn equip_ability(i: usize, ability: u16) -> bool {
    if i == 5 {
        ability == 1
    } else if i == 8 {
        ability <= 1
    } else {
        ability == 0
    }
}
fn score(
    o: &Observation,
    a: &Action,
    i: usize,
    round: usize,
    paid: &BTreeSet<ObjectRef>,
    seen_grantors: &BTreeSet<ObjectRef>,
    fallback: bool,
) -> i32 {
    let actor = o.actor.unwrap();
    let object = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let friends = o
        .objects
        .iter()
        .filter(|v| v.owner == actor && v.zone == Zone::Battlefield && v.card.is_creature())
        .collect::<Vec<_>>();
    let equipment = o
        .objects
        .iter()
        .filter(|v| v.owner == actor && v.zone == Zone::Battlefield && v.card.name() == NAMES[i])
        .collect::<Vec<_>>();
    let attached = equipment
        .iter()
        .any(|v| paid.contains(&v.reference) && v.attached_to.is_some());
    let need = if i == 2 {
        3
    } else if i == 0 || i == 3 {
        2
    } else {
        1
    };
    let preferred = if round.is_multiple_of(2) {
        "Three Tree Mascot"
    } else {
        "Pulse Tracker"
    };
    match a {
        Action::AnnounceCast { card } | Action::Cast { card, .. } => {
            let name = object(*card).card.name();
            if name == NAMES[i] {
                if friends.len() >= need && matches!(a, Action::AnnounceCast { .. }) {
                    800
                } else {
                    -1000
                }
            } else if name == "Hop to It" {
                if friends.len() < need { 720 } else { -1000 }
            } else if matches!(name, "Bear Cub" | "Three Tree Mascot" | "Pulse Tracker") {
                if friends.len() < need || !attached {
                    700 + i32::from(name == preferred) * 20
                } else {
                    -1000
                }
            } else {
                -1000
            }
        }
        Action::AnnounceActivate { card, ability, .. }
        | Action::BeginActivate { card, ability, .. }
        | Action::Activate { card, ability, .. } => {
            let source = object(*card);
            if source.card.name() != NAMES[i] || !matches!(a, Action::AnnounceActivate { .. }) {
                -1000
            } else if i == 5 && *ability == 0 {
                if seen_grantors.contains(&source.reference) {
                    1300
                } else {
                    -1000
                }
            } else if equip_ability(i, *ability) && !paid.contains(&source.reference) {
                if i == 8 {
                    1000 + if *ability == u16::from(!round.is_multiple_of(2)) {
                        80
                    } else {
                        0
                    }
                } else {
                    1100
                }
            } else {
                -1000
            }
        }
        Action::ActivateMana { card, .. } | Action::AnnounceMana { card, .. } => {
            let v = object(*card);
            if o.decision == "priority"
                && v.card
                    .token_info()
                    .is_some_and(|t| t.subtypes == "Treasure")
            {
                1400
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = object(*id);
            if let Some(ann) = &o.announcing {
                if let Some(event) = &ann.attachment_trigger {
                    if i == 0 && round.is_multiple_of(4) {
                        -1000
                    } else if Some(v.owner) == event.defender {
                        1300
                    } else {
                        100
                    }
                } else if matches!(ann.kind, StackKind::ActivatedAbility { .. }) {
                    let old = object(ann.card).attached_to;
                    if v.owner == actor && v.card.is_creature() && Some(v.reference) != old {
                        1200 + i32::from(v.card.name() == preferred) * 10
                    } else {
                        -1000
                    }
                } else if v.owner == actor && v.card.is_creature() {
                    1000 + i32::from(v.card.name() == preferred) * 10
                } else {
                    -1000
                }
            } else {
                0
            }
        }
        Action::PlayLand { card } => {
            let name = object(*card).card.name();
            let have = o
                .objects
                .iter()
                .filter(|v| {
                    v.owner == actor && v.zone == Zone::Battlefield && v.card.name() == name
                })
                .count();
            1000 - have as i32
        }
        Action::Attack { card } => {
            let goal = if i == 2 {
                if round.is_multiple_of(2) { 2 } else { 3 }
            } else {
                1
            };
            if !attached || friends.len() < need || o.attackers.len() >= goal {
                -1000
            } else if equipment.iter().any(|e| {
                paid.contains(&e.reference) && e.attached_to.is_some_and(|r| r.card == *card)
            }) {
                1050
            } else {
                900
            }
        }
        Action::ArrangeLibrary { destination, .. } => {
            let frame = o.library_choice.as_ref().unwrap();
            let total = frame.remaining.len() + frame.top.len() + frame.graveyard.len();
            let first = frame.remaining.len() == total;
            let grave = match round % 4 {
                0 => false,
                1 => true,
                2 => first,
                _ => !first,
            };
            if *destination
                == if grave {
                    LibraryDestination::Graveyard
                } else {
                    LibraryDestination::Top
                }
            {
                1400
            } else {
                -1000
            }
        }
        Action::TapMana { .. } => {
            if o.decision == "pay_costs" {
                800
            } else {
                -1000
            }
        }
        Action::PayCosts { .. } => 1500,
        Action::ChooseMana { .. } => 1400,
        Action::Discard { card } => {
            if object(*card).card.is_land() {
                1300
            } else {
                1200
            }
        }
        Action::PutTrigger { .. } => 1400,
        Action::FinishTargets => 1100,
        Action::FinishAttackers | Action::FinishBlockers => 500,
        Action::Block { .. } | Action::Concede => -10000,
        Action::Keep => 1600,
        Action::Mulligan => -1000,
        _ => {
            if fallback {
                100
            } else {
                0
            }
        }
    }
}
struct Look {
    cards: Vec<(ObjectRef, Card)>,
    library: usize,
    grave: usize,
    buried: BTreeSet<u32>,
    pattern: usize,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut decisions, mut replays, mut invalid, mut stat_checks, mut max_bytes) =
        (0u64, 0u64, 0u64, 0u64, 0usize);
    let (mut casts, mut equips, mut effects, mut controllers) =
        ([0u64; 10], [0u64; 10], [0u64; 10], [[0u64; 2]; 10]);
    let (mut cast_controllers, mut equip_controllers) = ([[0u64; 2]; 10], [[0u64; 2]; 10]);
    let (
        mut looks,
        mut partials,
        mut loot,
        mut treasure_created,
        mut treasure_used,
        mut sacrifice_draws,
        mut storm_yes,
        mut storm_no,
        mut optional_zero,
        mut printed_coexists,
        mut multiple,
    ) = (
        0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64,
    );
    let (mut pirate_costs, mut patterns) = ([0u64; 2], [0u64; 4]);
    for seed in 0..games {
        let i = seed % 10;
        let round = seed / 10;
        let mut g = Game::new(Config {
            decks: [deck(i), deck(i)],
            seed: seed as u64,
            starting_player: if round.is_multiple_of(2) {
                Player::P0
            } else {
                Player::P1
            },
            deck_rules: DeckRules::Standard,
        })?;
        let mut paid = BTreeSet::new();
        let mut seen_grantors = BTreeSet::new();
        let mut seen_packets = BTreeSet::new();
        let mut active: BTreeMap<u64, (SpellView, Observation)> = BTreeMap::new();
        let mut look: BTreeMap<u64, Look> = BTreeMap::new();
        for _ in 0..100000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            for packet in packets(&before) {
                if let Some(event) = &packet.attachment_trigger {
                    let k = index(event.granting_card).unwrap();
                    assert_eq!(event.granted, k >= 5);
                    assert_eq!(event.defender, Some(packet.controller.other()));
                    assert_eq!(
                        packet.object,
                        if event.granted {
                            event.recipient
                        } else {
                            event.grantor
                        }
                    );
                    assert_eq!(packet.card, packet.object.card);
                    assert!(packet.cast.is_none());
                    if !event.granted {
                        assert_eq!(packet.source_card, event.granting_card);
                    } else if let Some(v) = before
                        .objects
                        .iter()
                        .find(|v| v.reference == event.recipient)
                    {
                        assert_eq!(packet.source_card, v.card);
                    }
                    seen_grantors.insert(event.grantor);
                    if seen_packets.insert(packet.id) {
                        let other = g.observe(actor.other());
                        let public = packets(&other).find(|s| s.id == packet.id).unwrap();
                        assert_eq!(public.attachment_trigger, packet.attachment_trigger);
                        assert_eq!(public.object, packet.object);
                    }
                }
            }
            if before.decision == "order_triggers" {
                let granted = before
                    .waiting_triggers
                    .iter()
                    .filter(|s| s.attachment_trigger.as_ref().is_some_and(|e| e.granted))
                    .count();
                if granted >= 2 {
                    multiple += 1;
                }
                if granted > 0
                    && before.waiting_triggers.iter().any(|s| {
                        s.attachment_trigger.is_none() && s.source_card.name() == "Pulse Tracker"
                    })
                {
                    printed_coexists += 1;
                }
            }
            if let Some(frame) = &before.library_choice {
                assert!(frame.surveil);
                let root = before.resolving.as_ref().unwrap();
                assert_eq!(
                    root.attachment_trigger
                        .as_ref()
                        .unwrap()
                        .granting_card
                        .name(),
                    "Candlestick"
                );
                let other = g.observe(actor.other());
                assert!(other.library_choice.is_none() && other.actions.is_empty());
                look.entry(root.id).or_insert_with(|| {
                    looks += 1;
                    patterns[round % 4] += 1;
                    Look {
                        cards: frame
                            .remaining
                            .iter()
                            .map(|v| (v.reference, v.card))
                            .collect(),
                        library: before.players[actor.index()].library_count,
                        grave: before
                            .objects
                            .iter()
                            .filter(|v| v.owner == actor && v.zone == Zone::Graveyard)
                            .count(),
                        buried: BTreeSet::new(),
                        pattern: round % 4,
                    }
                });
            }
            if before.decision == "effect_discard"
                && before.resolving.as_ref().is_some_and(|s| {
                    s.attachment_trigger
                        .as_ref()
                        .is_some_and(|e| e.granting_card.name() == "Pirate Hat")
                })
            {
                let other = g.observe(actor.other());
                assert!(other.actions.is_empty());
                assert!(
                    !other
                        .objects
                        .iter()
                        .any(|v| v.owner == actor && v.zone == Zone::Hand)
                );
            }
            let fallback = rustmage::policy::smoke_index(&g, &before.actions);
            let j = before
                .actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| {
                    score(&before, a, i, round, &paid, &seen_grantors, *j == fallback)
                })
                .unwrap()
                .0;
            let a = before.actions[j].clone();
            if before.decision == "targets" {
                let snap = g.snapshot();
                let revision = g.revision();
                assert_eq!(
                    g.apply(
                        actor,
                        Action::ChooseTarget {
                            target: Target::Card(u32::MAX)
                        }
                    ),
                    Err(Error::IllegalAction)
                );
                assert_eq!(g.snapshot(), snap);
                assert_eq!(g.revision(), revision);
                assert_eq!(g.apply(actor.other(), a.clone()), Err(Error::WrongActor));
                assert_eq!(g.snapshot(), snap);
                invalid += 2;
            }
            if let Action::PayCosts { payment } = a
                && let Some(ann) = &before.announcing
                && let Some(k) = index(ann.source_card)
            {
                let requested = before.payment_request.as_ref().unwrap();
                let expected = if ann.kind == StackKind::Spell {
                    cost(ann.source_card)
                } else if let StackKind::ActivatedAbility { ability } = ann.kind {
                    (
                        [0; 6],
                        if k == 8 && ability == 0 {
                            1
                        } else if k == 5 && ability == 0 {
                            2
                        } else {
                            EQUIP[k]
                        },
                    )
                } else {
                    unreachable!()
                };
                assert_eq!((requested.required, requested.generic), expected);
                assert_eq!(
                    payment.iter().sum::<u32>(),
                    expected.0.iter().sum::<u32>() + expected.1
                );
                if ann.kind == StackKind::Spell {
                    casts[k] += 1;
                    cast_controllers[k][actor.index()] += 1;
                    assert_eq!(ann.cast.unwrap().origin, Zone::Hand);
                } else if let StackKind::ActivatedAbility { ability } = ann.kind
                    && equip_ability(k, ability)
                {
                    equips[k] += 1;
                    equip_controllers[k][actor.index()] += 1;
                    paid.insert(ann.object);
                    if k == 8 {
                        pirate_costs[usize::from(ability != 0)] += 1;
                    }
                }
            }
            if matches!(a, Action::Pass)
                && let Some(packet) = before.stack.last()
                && (packet.attachment_trigger.is_some()
                    || packet.source_card.name() == "Candlestick"
                        && matches!(packet.kind, StackKind::ActivatedAbility { ability: 0 }))
            {
                active.insert(packet.id, (packet.clone(), g.observe(packet.controller)));
            }
            if let Action::ArrangeLibrary { card, destination } = a
                && destination == LibraryDestination::Graveyard
            {
                look.get_mut(&before.resolving.as_ref().unwrap().id)
                    .unwrap()
                    .buried
                    .insert(card);
            }
            let checkpoint = g.snapshot();
            let revision = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, a.clone())?;
            g.apply_index(actor, revision, j)?;
            let expected = g.snapshot();
            assert_eq!(fork.snapshot(), expected);
            g.restore(&checkpoint)?;
            assert_eq!(g.apply_index(actor, revision, j), Err(Error::StaleRevision));
            g.apply(actor, a.clone())?;
            assert_eq!(g.snapshot(), expected);
            let after = g.observe(actor);
            assert_eq!(after.schema, rustmage::OBSERVATION_SCHEMA);
            decisions += 1;
            replays += 1;
            max_bytes = max_bytes.max(g.state_bytes());
            if matches!(a, Action::Attack { .. } | Action::AttackPlaneswalker { .. }) {
                assert!(!packets(&after).any(|s| s.attachment_trigger.is_some()));
            }
            // Every changed live attachment gives/removes exactly its printed bonus.
            for source in before
                .objects
                .iter()
                .filter(|v| v.zone == Zone::Battlefield && index(v.card).is_some())
            {
                let old = source.attached_to;
                let new = after
                    .objects
                    .iter()
                    .find(|v| v.reference == source.reference && v.zone == Zone::Battlefield)
                    .and_then(|v| v.attached_to);
                if old != new {
                    let bonus = BONUS[index(source.card).unwrap()];
                    for (reference, sign) in [(old, -1), (new, 1)] {
                        if let Some(r) = reference
                            && let Some(b) = before.objects.iter().find(|v| v.reference == r)
                            && let Some(n) = after.objects.iter().find(|v| v.reference == r)
                        {
                            assert_eq!(
                                (n.power - b.power, n.toughness - b.toughness),
                                (sign * bonus.0, sign * bonus.1)
                            );
                            stat_checks += 1;
                        }
                    }
                }
            }
            if let Some(frame) = &before.library_choice {
                let id = before.resolving.as_ref().unwrap().id;
                let held = look.get(&id).unwrap();
                if after.library_choice.is_some() {
                    assert_eq!(after.players[actor.index()].library_count, held.library);
                    assert_eq!(
                        after
                            .objects
                            .iter()
                            .filter(|v| v.owner == actor && v.zone == Zone::Graveyard)
                            .count(),
                        held.grave
                    );
                    assert!(
                        !after
                            .objects
                            .iter()
                            .any(|v| held.cards.iter().any(|(r, _)| v.reference == *r))
                    );
                    partials += 1;
                } else {
                    assert_eq!(
                        after.players[actor.index()].library_count,
                        held.library - held.buried.len()
                    );
                    for (r, card) in &held.cards {
                        if held.buried.contains(&r.card) {
                            let v = after.objects.iter().find(|v| v.id == r.card).unwrap();
                            assert_eq!(v.zone, Zone::Graveyard);
                            assert_eq!(v.card, *card);
                            assert_eq!(v.reference.generation, r.generation + 1);
                        } else {
                            assert!(!after.objects.iter().any(|v| v.reference == *r));
                        }
                    }
                    assert!(held.pattern < 4);
                    look.remove(&id);
                }
                let _ = frame;
            }
            let consumed = before
                .objects
                .iter()
                .filter(|v| {
                    v.zone == Zone::Battlefield
                        && v.owner == actor
                        && v.card
                            .token_info()
                            .is_some_and(|t| t.subtypes == "Treasure")
                        && !after.objects.iter().any(|n| n.reference == v.reference)
                })
                .count();
            if consumed > 0 {
                assert_eq!(consumed, 1);
                assert_eq!(
                    after.players[actor.index()].mana.iter().sum::<u32>(),
                    before.players[actor.index()].mana.iter().sum::<u32>() + 1
                );
                treasure_used += 1;
            }
            let complete = active
                .keys()
                .copied()
                .filter(|id| !packets(&after).any(|s| s.id == *id))
                .collect::<Vec<_>>();
            for id in complete {
                let (packet, base) = active.remove(&id).unwrap();
                let view = g.observe(packet.controller);
                let p = packet.controller.index();
                // A draw from an empty library ends the game before its counter increments.
                // Such packets prove decking, but are not successful effect witnesses.
                if base.players[p].library_count == 0
                    && view.players[p].cards_drawn_this_turn
                        == base.players[p].cards_drawn_this_turn
                    && g.outcome() == Some(rustmage::Outcome::Winner(packet.controller.other()))
                {
                    continue;
                }
                if let Some(event) = &packet.attachment_trigger {
                    let k = index(event.granting_card).unwrap();
                    effects[k] += 1;
                    controllers[k][p] += 1;
                    match k {
                        0 | 1 | 3 | 9 => {
                            if packet.targets.is_empty() {
                                assert_eq!(k, 0);
                                optional_zero += 1;
                            } else {
                                for t in &packet.targets {
                                    let v = view
                                        .objects
                                        .iter()
                                        .find(|v| Some(v.reference) == t.object)
                                        .unwrap();
                                    assert!(v.tapped);
                                }
                            }
                        }
                        2 => {
                            let expected = u64::from(view.attackers.len() >= 3);
                            assert_eq!(
                                view.players[p].cards_drawn_this_turn
                                    - base.players[p].cards_drawn_this_turn,
                                expected
                            );
                            if expected == 1 {
                                storm_yes += 1
                            } else {
                                storm_no += 1
                            }
                        }
                        4 => {
                            for t in &packet.targets {
                                let target = t.object.unwrap();
                                let original =
                                    base.objects.iter().find(|v| v.reference == target).unwrap();
                                if original.card.token_info().is_some() {
                                    assert!(!view.objects.iter().any(|v| v.id == target.card));
                                } else {
                                    let dead =
                                        view.objects.iter().find(|v| v.id == target.card).unwrap();
                                    assert_eq!(dead.zone, Zone::Graveyard);
                                    assert_eq!(dead.reference.generation, target.generation + 1);
                                }
                            }
                        }
                        5 => assert!(!look.contains_key(&id)),
                        6 => {
                            let n = view
                                .objects
                                .iter()
                                .filter(|v| {
                                    v.owner == packet.controller
                                        && v.zone == Zone::Battlefield
                                        && v.card
                                            .token_info()
                                            .is_some_and(|t| t.subtypes == "Treasure")
                                })
                                .count();
                            let old = base
                                .objects
                                .iter()
                                .filter(|v| {
                                    v.owner == packet.controller
                                        && v.zone == Zone::Battlefield
                                        && v.card
                                            .token_info()
                                            .is_some_and(|t| t.subtypes == "Treasure")
                                })
                                .count();
                            assert_eq!(n, old + 1);
                            treasure_created += 1;
                        }
                        7 => assert_eq!(view.players[p].life, base.players[p].life + 1),
                        8 => {
                            assert_eq!(
                                view.players[p].cards_drawn_this_turn,
                                base.players[p].cards_drawn_this_turn + 1
                            );
                            assert_eq!(
                                view.players[p].cards_discarded_this_turn,
                                base.players[p].cards_discarded_this_turn + 1
                            );
                            assert_eq!(view.players[p].hand_count, base.players[p].hand_count);
                            loot += 1;
                        }
                        _ => unreachable!(),
                    }
                } else {
                    assert_eq!(
                        view.players[p].cards_drawn_this_turn,
                        base.players[p].cards_drawn_this_turn + 1,
                        "Candlestick seed {seed} outcome {:?} packet {packet:?} before {:?} after {:?}",
                        g.outcome(),
                        base.players[p],
                        view.players[p]
                    );
                    assert_eq!(
                        view.objects
                            .iter()
                            .find(|v| v.id == packet.card)
                            .unwrap()
                            .zone,
                        Zone::Graveyard
                    );
                    sacrifice_draws += 1;
                }
            }
        }
        assert!(g.outcome().is_some(), "unfinished seed {seed}");
    }
    if games >= 100 {
        assert!(casts.iter().all(|&n| n > 0));
        assert!(equips.iter().all(|&n| n > 0));
        assert!(effects.iter().all(|&n| n > 0));
        assert!(controllers.iter().all(|v| v.iter().all(|&n| n > 0)));
        assert!(cast_controllers.iter().all(|v| v.iter().all(|&n| n > 0)));
        assert!(equip_controllers.iter().all(|v| v.iter().all(|&n| n > 0)));
        assert!(patterns.iter().all(|&n| n > 0));
        assert!(pirate_costs.iter().all(|&n| n > 0));
        assert!(
            partials > 0
                && loot > 0
                && treasure_used > 0
                && sacrifice_draws > 0
                && storm_yes > 0
                && storm_no > 0
                && optional_zero > 0
                && multiple > 0
                && printed_coexists > 0
        );
    }
    println!(
        "Completed {games} attachment-attack Standard games, {decisions} decisions/{replays} replays, casts {casts:?}, paid equips {equips:?}, effects {effects:?}, controllers {controllers:?}, cast controllers {cast_controllers:?}, equip controllers {equip_controllers:?}, {stat_checks} exact static changes, {looks} surveils/{partials} deferred partials/patterns{patterns:?}, {loot} private loots, {treasure_created} Treasures/{treasure_used} actual mana uses, {sacrifice_draws} Candlestick sacrifice draws, Storm true/false {storm_yes}/{storm_no}, Pirate cheap/ordinary {pirate_costs:?}, {optional_zero} optional-zero attacks, {multiple} multiple grants/{printed_coexists} printed coexistence windows, {invalid} invalid actions, {max_bytes} max logical bytes"
    );
    Ok(())
}
