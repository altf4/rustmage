//! Legal Standard self-play with exact target-owner destinations and owned replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, LibraryDestination, ObjectRef, Observation,
    Player, StackKind, Target, Zone,
};
use std::collections::BTreeSet;
const NAMES: [&str; 13] = [
    "Chrome Companion",
    "Hoverstone Pilgrim",
    "Jade-Cast Sentinel",
    "Malevolent Chandelier",
    "The Spot's Portal",
    "Lost in Space",
    "Misleading Motes",
    "Return to the Sewers",
    "Riverwalk Technique",
    "Trip Up",
    "Uncharted Voyage",
    "Unlucky Drop",
    "Vanish from Sight",
];
fn deck(index: usize) -> Vec<Card> {
    let mut d = vec![Card::from_name("Island").unwrap(); 20];
    d.extend([Card::from_name("Swamp").unwrap(); 8]);
    d.extend([Card::from_name("Forest").unwrap(); 4]);
    for n in [
        NAMES[index],
        "Bear Cub",
        "Fell",
        "Three Tree Mascot",
        "Opt",
        "Mockingbird, Ace Agent",
        if index == 1 {
            "Goldvein Pick"
        } else {
            "Hoverstone Pilgrim"
        },
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn score(
    o: &Observation,
    a: &Action,
    wanted: &str,
    seed: usize,
    used: &BTreeSet<(u32, u32, u16)>,
    fallback: bool,
) -> i32 {
    let name = |id| o.objects.iter().find(|v| v.id == id).unwrap().card.name();
    match a {
        Action::ChooseLibraryDestination { destination, .. } => {
            if *destination
                == if seed.is_multiple_of(2) {
                    LibraryDestination::Top
                } else {
                    LibraryDestination::Bottom
                }
            {
                1000
            } else {
                0
            }
        }
        Action::PayCosts { .. } | Action::PayResolutionMana { .. } => 900,
        Action::ChooseMode { mode } => {
            if *mode
                == u16::from(o.stack.iter().any(|s| {
                    s.kind == StackKind::Spell
                        && s.controller != o.actor.unwrap()
                        && !s.source_card.is_creature()
                }))
            {
                850
            } else {
                0
            }
        }
        Action::Cast { card, .. } | Action::AnnounceCast { card } => {
            let n = name(*card);
            if n == wanted {
                700
            } else if n == "Fell" {
                if o.objects
                    .iter()
                    .any(|v| v.zone == Zone::Battlefield && v.card.name() == "Bear Cub")
                {
                    620
                } else {
                    -100
                }
            } else if n == "Opt" {
                300
            } else {
                600
            }
        }
        Action::Activate { card, ability, .. }
        | Action::BeginActivate { card, ability, .. }
        | Action::AnnounceActivate { card, ability, .. } => {
            if used.contains(&(o.turn, *card, *ability)) {
                return -100;
            }
            if name(*card) == wanted && wanted == "Trip Up" {
                if seed.is_multiple_of(2) { 780 } else { -100 }
            } else if name(*card) == wanted
                || o.objects.iter().any(|v| {
                    v.id == *card && v.card.token_info().is_some_and(|t| t.subtypes == "Mutagen")
                })
            {
                750
            } else {
                -100
            }
        }
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = o.objects.iter().find(|v| v.id == *id).unwrap();
            if v.card.name() == "Hoverstone Pilgrim"
                && v.owner != o.actor.unwrap()
                && seed.is_multiple_of(3)
            {
                920
            } else if v.card.name() == "Bear Cub" {
                900 + i32::from(v.owner != o.actor.unwrap())
            } else if v.card.name() == wanted {
                20
            } else {
                800
            }
        }
        Action::ChooseTarget {
            target: Target::Spell(_),
        } => 950,
        Action::TapMana { .. } | Action::ActivateMana { .. } | Action::AnnounceMana { .. } => {
            if o.decision == "pay_costs" || o.resolution_payment.is_some() {
                500
            } else {
                -100
            }
        }
        Action::Attack { .. } => {
            if o.turn > 22 {
                200
            } else {
                -1000
            }
        }
        Action::Block { .. } | Action::Concede => -1000,
        Action::PlayLand { card } => {
            if name(*card) == "Island" && o.players[o.actor.unwrap().index()].mana[1] == 0 {
                820
            } else {
                810
            }
        }
        _ => {
            if fallback {
                100
            } else {
                0
            }
        }
    }
}
fn verify_movement(
    game: &Game,
    before: &Observation,
    after: &Observation,
    target: ObjectRef,
    source: ObjectRef,
    destination: LibraryDestination,
) {
    let old = before
        .objects
        .iter()
        .find(|v| v.reference == target)
        .unwrap();
    let movement = after.last_target_library_move.as_ref().unwrap();
    assert_eq!(movement.source, source);
    assert_eq!(movement.target_before, target);
    assert_eq!(movement.card, old.card);
    assert_eq!(
        movement.target_after,
        ObjectRef {
            card: target.card,
            generation: target.generation + 1
        }
    );
    assert_eq!(movement.owner, old.owner);
    assert_eq!(movement.destination, destination);
    assert_eq!(
        serde_json::to_value(game.observe(Player::P0).last_target_library_move).unwrap(),
        serde_json::to_value(game.observe(Player::P1).last_target_library_move).unwrap()
    );
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut decisions, mut replays, mut invalid, mut divergent, mut deferred, mut max_bytes) =
        (0u64, 0u64, 0u64, 0u64, 0u64, 0usize);
    let (
        mut casts,
        mut moves,
        mut owners,
        mut destinations,
        mut cycles,
        mut counters,
        mut surveils,
        mut mutagens,
    ) = (
        [0u64; 13], [0u64; 13], [0u64; 2], [0u64; 2], 0u64, 0u64, 0u64, 0u64,
    );
    let (mut chrome_life, mut ward_payments, mut mutagen_counters) = (0u64, 0u64, 0u64);
    for seed in 0..games {
        let index = seed % 13;
        let mut game = Game::new(Config {
            decks: [deck(index), deck(index)],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let (mut seen, mut used, mut pending) = (BTreeSet::new(), BTreeSet::new(), Vec::new());
        for _ in 0..100000 {
            if game.outcome().is_some() {
                break;
            }
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            let fallback = rustmage::policy::smoke_index(&game, &before.actions);
            let choice = before
                .actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| score(&before, a, NAMES[index], seed, &used, *j == fallback))
                .unwrap()
                .0;
            let action = before.actions[choice].clone();
            if let Action::Activate { card, ability, .. }
            | Action::BeginActivate { card, ability, .. }
            | Action::AnnounceActivate { card, ability, .. } = action
            {
                used.insert((before.turn, card, ability));
            }
            for packet in before.stack.iter().chain(before.announcing.iter()) {
                if packet.kind == StackKind::Spell
                    && packet.source().is_some_and(|source| source.card.name() == NAMES[index])
                    && seen.insert(packet.id)
                {
                    casts[index] += 1;
                }
            }
            if let Some(view) = &before.target_library {
                let target = before
                    .objects
                    .iter()
                    .find(|v| v.reference == view.target)
                    .unwrap();
                assert_eq!(view.player, actor);
                assert_eq!(target.owner, actor);
                let other = game.observe(if actor == Player::P0 {
                    Player::P1
                } else {
                    Player::P0
                });
                assert!(other.target_library.is_none());
                assert!(other.actions.is_empty());
                assert!(before.library_choice.is_none());
                assert!(matches!(target.zone, Zone::Battlefield | Zone::Graveyard));
                let snapshot = game.snapshot();
                let revision = game.revision();
                for bad in [
                    Action::Pass,
                    Action::ChooseLibraryDestination {
                        object: ObjectRef {
                            card: view.target.card,
                            generation: view.target.generation + 1,
                        },
                        destination: LibraryDestination::Top,
                    },
                ] {
                    assert!(matches!(game.apply(actor, bad), Err(Error::IllegalAction)));
                    assert_eq!(game.snapshot(), snapshot);
                    assert_eq!(game.revision(), revision);
                    invalid += 1;
                }
                let mut alternative = game.clone();
                let alternate = match &action {
                    Action::ChooseLibraryDestination {
                        destination: LibraryDestination::Top,
                        ..
                    } => LibraryDestination::Bottom,
                    _ => LibraryDestination::Top,
                };
                alternative.apply(
                    actor,
                    Action::ChooseLibraryDestination {
                        object: view.target,
                        destination: alternate,
                    },
                )?;
                let mut chosen_fork = game.clone();
                chosen_fork.apply(actor, action.clone())?;
                assert_ne!(alternative.snapshot(), chosen_fork.snapshot());
                assert_eq!(game.snapshot(), snapshot);
                divergent += 1;
                owners[actor.index()] += 1;
                destinations[usize::from(matches!(
                    action,
                    Action::ChooseLibraryDestination {
                        destination: LibraryDestination::Bottom,
                        ..
                    }
                ))] += 1;
                pending.push((
                    view.target,
                    actor,
                    index,
                    before.players[actor.index()].library_count,
                    target.token.is_some(),
                ));
                if matches!(index, 5 | 10 | 12) {
                    deferred += 1;
                }
            }
            if matches!(action, Action::PayResolutionMana { .. })
                && before
                    .resolving
                    .as_ref()
                    .is_some_and(|s| s.source().is_some_and(|source| source.card.name() == "Hoverstone Pilgrim"))
            {
                assert_eq!(before.resolution_payment.as_ref().unwrap().cost.generic, 2);
                ward_payments += 1;
            }
            let resolving = if matches!(action, Action::Pass) {
                before.stack.last()
            } else {
                None
            };
            let resolved = resolving.map(|s| {
                (
                    s.id,
                    s.source_card
                        .token_info()
                        .map_or(s.source_card.name(), |t| t.name),
                    s.targets.clone(),
                    s.kind,
                    s.controller,
                )
            });
            let snapshot = game.snapshot();
            let revision = game.revision();
            let mut fork = game.clone();
            fork.apply(actor, action.clone())?;
            game.apply(actor, action.clone())?;
            assert_eq!(fork.snapshot(), game.snapshot());
            let expected = game.snapshot();
            game.restore(&snapshot)?;
            assert_eq!(
                game.apply_index(actor, revision, choice),
                Err(Error::StaleRevision)
            );
            game.apply(actor, action)?;
            assert_eq!(game.snapshot(), expected);
            decisions += 1;
            replays += 1;
            max_bytes = max_bytes.max(game.state_bytes());
            let after = game.observe(actor);
            for (target, owner, i, count, token) in pending.drain(..) {
                assert!(!after.objects.iter().any(|v| v.reference == target));
                assert_eq!(
                    after.players[owner.index()].library_count,
                    count + usize::from(!token)
                );
                moves[i] += 1;
                let Action::ChooseLibraryDestination { destination, .. } = before.actions[choice]
                else {
                    unreachable!()
                };
                verify_movement(
                    &game,
                    &before,
                    &after,
                    target,
                    before.target_library.as_ref().unwrap().source,
                    destination,
                );
                if matches!(i, 5 | 10 | 12) {
                    let controller = before.resolving.as_ref().unwrap().controller;
                    let view = game.observe(controller);
                    assert_eq!(
                        view.library_choice.is_some(),
                        view.players[controller.index()].library_count > 0
                    );
                    surveils += 1;
                }
                if i == 7 {
                    assert!(after.objects.iter().any(|v| v.zone == Zone::Battlefield
                        && v.card.token_info().is_some_and(|t| t.subtypes == "Mutagen")));
                    mutagens += 1;
                }
            }
            if let Some((id, name, targets, kind, _)) = &resolved
                && *name == "Mutagen Token"
                && matches!(kind, StackKind::ActivatedAbility { .. })
                && !after.stack.iter().any(|s| s.id == *id)
            {
                for t in targets {
                    if let Some(r) = t.object
                        && let Some(old) = before.objects.iter().find(|v| v.reference == r)
                        && let Some(new) = after.objects.iter().find(|v| v.reference == r)
                    {
                        assert_eq!(new.counters.plus_one, old.counters.plus_one + 1);
                        mutagen_counters += 1;
                    }
                }
            }
            if let Some((id, name, targets, kind, controller)) = resolved
                && !after.stack.iter().any(|s| s.id == id)
                && after.resolving.as_ref().is_none_or(|s| s.id != id)
                && let Some(i) = NAMES.iter().position(|&n| n == name)
            {
                if name == "Chrome Companion" && matches!(kind, StackKind::TriggeredAbility { .. })
                {
                    assert_eq!(
                        after.players[controller.index()].life,
                        before.players[controller.index()].life + 1
                    );
                    chrome_life += 1;
                }
                if i < 5 {
                    for target in &targets {
                        if let Some(r) = target.object
                            && let Some(old) = before.objects.iter().find(|v| v.reference == r)
                            && !after.objects.iter().any(|v| v.reference == r)
                            && after.players[old.owner.index()].library_count
                                == before.players[old.owner.index()].library_count + 1
                        {
                            moves[i] += 1;
                            verify_movement(
                                &game,
                                &before,
                                &after,
                                r,
                                resolving.unwrap().object,
                                LibraryDestination::Bottom,
                            );
                            if i == 4 {
                                let villain = after.objects.iter().any(|v| {
                                    v.zone == Zone::Battlefield
                                        && v.owner == controller
                                        && v.card.has_subtype("Villain")
                                });
                                assert_eq!(
                                    after.players[controller.index()].life,
                                    before.players[controller.index()].life
                                        - if villain { 0 } else { 2 }
                                );
                            }
                        }
                    }
                }
                if i == 9 && kind != StackKind::Spell {
                    cycles += 1;
                }
                if i == 8 && targets.iter().any(|t| matches!(t.target, Target::Spell(_))) {
                    counters += 1;
                }
            }
        }
        assert!(game.outcome().is_some(), "unfinished seed {seed}");
    }
    if games >= 100 {
        assert!(casts.iter().all(|&n| n > 0));
        assert!(moves.iter().all(|&n| n > 0));
        assert!(owners.iter().all(|&n| n > 0));
        assert!(destinations.iter().all(|&n| n > 0));
        assert!(cycles > 0 && counters > 0 && surveils > 0 && mutagens > 0);
        assert!(chrome_life > 0 && ward_payments > 0 && mutagen_counters > 0);
    }
    println!(
        "Completed {games} library-tuck Standard games, {decisions} decisions/{replays} replays, casts {casts:?}, moves {moves:?}, owners {owners:?}, destinations {destinations:?}, {cycles} cycling, {counters} counters, {surveils} surveils, {mutagens} Mutagens, {invalid} invalid actions, {divergent} divergent continuations, {deferred} deferred surveils, {chrome_life} Chrome life triggers, {ward_payments} Ward payments, {mutagen_counters} Mutagen counters, {max_bytes} max logical bytes"
    );
    Ok(())
}
