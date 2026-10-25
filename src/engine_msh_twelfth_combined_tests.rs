//! Two bounded assurance groups: a paid eight-card fixture and natural Standard MSH-only games.
//! Fixture setup/payment mana is explicit and separate from the unmodified Game::new group.
use super::assurance::big_helpers::{checked, named, setup};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
const EIGHT: [&str; 8] = [
    "Ka-Zar of the Savage Land",
    "Mole Man, Moloid Master",
    "Iron Man, Master of Machines",
    "Avengers Assemble!",
    "Night Nurse, Healer of Heroes",
    "Restorative Technique",
    "Spider-Man, To the Rescue",
    "Speed, Young Avenger",
];
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    let bytes = g.state_bytes();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.state_bytes(), bytes);
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut observation = g.observe(p);
        // Restoring deliberately advances the branch revision.
        observation.revision = 0;
        serde_json::to_value(observation).unwrap()
    })
}
fn orphan(g: Game, action: Action) -> Game {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    let before_views = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    let after = raw.snapshot();
    assert_eq!(indexed.snapshot(), after);
    assert_eq!(views(&raw), views(&indexed));
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn finish_packet(g: &mut Game, packet: u64) {
    for _ in 0..20 {
        if g.state
            .announcing
            .as_ref()
            .is_none_or(|a| a.item.id != packet)
        {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseSpellCost { .. }
                )
            })
            .unwrap_or_else(|| panic!("packet {packet} stalled at {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("packet {packet} did not finish");
}
// Never guess modal choices or choose optional/multi targets. Only later Daredevil
// attacks are declined by default. Every nontrivial packet is stopped explicitly.
fn drive(g: &mut Game, stop: impl Fn(&Game) -> bool) -> usize {
    let mut declined = 0;
    g.prepare_triggers();
    for _ in 0..600 {
        if stop(g) {
            return declined;
        }
        assert!(g.outcome().is_none(), "unexpected terminal state");
        let action = if matches!(g.state.pending, Pending::EffectOptional { .. }) {
            declined += 1;
            Action::ChooseOptional { accept: false }
        } else if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
            Action::DeclineResolutionMana
        } else if g
            .state
            .announcing
            .as_ref()
            .is_some_and(|a| a.item.source_card.name() == "Human Torch, Johnny Storm")
            && g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(g.state.active.other()),
            })
        {
            Action::ChooseTarget {
                target: Target::Player(g.state.active.other()),
            }
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishAttackers
                            | Action::FinishBlockers
                    ) || matches!(a, Action::FinishTargets)
                        && g.state.announcing.as_ref().is_some_and(|a| a.spec.max == 0)
                })
                .unwrap_or_else(|| {
                    panic!(
                        "unhandled combined decision {:?} at {:?}",
                        g.state.pending, g.state.step
                    )
                })
        };
        checked(g, action);
    }
    panic!("combined progression stalled");
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn settle(g: &mut Game) {
    drive(g, quiet);
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn next_own_main(g: &mut Game, p: Player) {
    let turn = g.state.turn;
    drive(g, |g| {
        g.state.turn > turn
            && g.state.active == p
            && g.state.step == Step::Main1
            && quiet(g)
            && g.actor() == Some(p)
    });
}
fn trigger(g: &Game, id: CardId, ability: u16) -> bool {
    g.state.announcing.as_ref().is_some_and(|a| {
        a.item.card == id && a.item.kind == StackKind::TriggeredAbility { ability }
    })
}

fn cast_paid(g: &mut Game, p: Player, id: CardId, targets: &[Target], extra: u32) {
    priority(g, p);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    let (mut mana, generic) = g.object_view(id).card.cost();
    mana[5] += generic + extra;
    g.state.players[p.index()].mana = mana;
    checked(g, Action::AnnounceCast { card: id });
    let packet = g.state.announcing.as_ref().unwrap().item.id;
    for &target in targets {
        while !g.legal_actions().contains(&Action::ChooseTarget { target }) {
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    finish_packet(g, packet);
    assert_eq!(g.state.players[p.index()].mana[5], extra);
}
fn put_paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast_paid(g, p, id, &[], 0);
    settle(g);
    id
}
fn tokens(g: &Game, p: Player, name: &str) -> Vec<CardId> {
    (0..g.state.objects.len() as CardId)
        .filter(|&id| {
            g.on_battlefield(id)
                && g.state.objects[id as usize].controller == p
                && g.object_view(id).name() == name
        })
        .collect()
}
fn target(g: &mut Game, id: CardId) {
    let packet = g.state.announcing.as_ref().unwrap().item.id;
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    finish_packet(g, packet);
}
#[test]
fn msh_twelfth_combined_paid_eight_land_origins_history_recovery_reflexive_and_natural_win() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        for player in &mut g.state.players {
            player.mana = [0; 6];
        }
        let ka = put_paid(&mut g, p, EIGHT[0]);
        let zabu = tokens(&g, p, "Zabu")[0];
        let mole = put_paid(&mut g, p, EIGHT[1]);
        let speed = put_paid(&mut g, p, EIGHT[7]);
        let iron = put_paid(&mut g, p, EIGHT[2]);
        let torch = put_paid(&mut g, p, "Human Torch, Johnny Storm");
        let bear = named(&mut g, q, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].summoning_sick = false;
        let murder = named(&mut g, q, "Murder", Zone::Hand);
        cast_paid(&mut g, q, murder, &[Target::Card(torch)], 0);
        settle(&mut g);
        assert_eq!(g.object_view(torch).zone, Zone::Graveyard);
        let nurse = named(&mut g, p, EIGHT[4], Zone::Hand);
        cast_paid(&mut g, p, nurse, &[], 0);
        drive(&mut g, |g| trigger(g, nurse, 0));
        let packet = g.state.announcing.as_ref().unwrap().item.id;
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(torch),
            },
        );
        finish_packet(&mut g, packet);
        settle(&mut g);
        assert_eq!(g.object_view(torch).zone, Zone::Hand);
        cast_paid(&mut g, p, torch, &[], 0);
        settle(&mut g);
        // Bounded graveyard origin fixture, not a claim of a natural mill producer.
        let forest = named(&mut g, p, "Forest", Zone::Hand);
        g.move_object(forest, Zone::Graveyard);
        priority(&mut g, p);
        g = orphan(g, Action::PlayLand { card: forest });
        settle(&mut g);
        assert_eq!(g.counter_count(zabu, CounterKind::PlusOne), 1);
        assert_eq!(tokens(&g, p, "Moloid").len(), 1);
        let rest = named(&mut g, p, EIGHT[5], Zone::Hand);
        cast_paid(&mut g, p, rest, &[Target::Player(p), Target::Card(zabu)], 1);
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
        g = orphan(
            g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        drive(&mut g, |g| trigger(g, speed, 1));
        target(&mut g, speed);
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        let search = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseSearch { .. }))
            .unwrap();
        let fetched = if let Action::ChooseSearch { card } = search {
            card
        } else {
            unreachable!()
        };
        g = orphan(g, search);
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert!(g.object_view(fetched).tapped);
        assert_eq!(g.object_view(fetched).zone, Zone::Battlefield);
        assert_eq!(tokens(&g, p, "Moloid").len(), 2);
        assert_eq!(g.counter_count(zabu, CounterKind::PlusOne), 3);
        let spider = named(&mut g, p, EIGHT[6], Zone::Hand);
        cast_paid(&mut g, p, spider, &[], 0);
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        g = orphan(g, Action::ChooseOptional { accept: true });
        drive(&mut g, |g| trigger(g, spider, 1));
        target(&mut g, speed);
        settle(&mut g);
        assert!(g.object_view(spider).tapped);
        assert!(g.keywords(speed).contains(Keywords::INDESTRUCTIBLE));
        put_paid(&mut g, p, EIGHT[3]);
        assert_eq!((g.stats(speed).power, g.stats(speed).toughness), (4, 4));
        drive(&mut g, |g| {
            g.state.active == p && g.state.pending == Pending::Attackers
        });
        checked(&mut g, Action::Attack { card: speed });
        checked(&mut g, Action::FinishAttackers);
        drive(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: speed,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        drive(&mut g, |g| g.state.step == Step::Main2 && quiet(g));
        let before = g.state.players[p.index()].library.len();
        drive(&mut g, |g| g.state.step == Step::End && quiet(g));
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        next_own_main(&mut g, p);
        assert!(!g.keywords(speed).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.keywords(speed).contains(Keywords::ONLY_HASTE_BLOCKERS));
        let top = *g.state.players[p.index()].library.last().unwrap();
        assert_eq!(g.object_view(top).card, Card::from_name("Forest").unwrap());
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            g.object_ref(top)
        );
        assert!(g.observe(q).known_library_top[p.index()].is_none());
        g = orphan(g, Action::PlayLand { card: top });
        settle(&mut g);
        assert_eq!(tokens(&g, p, "Moloid").len(), 3);
        assert_eq!(g.counter_count(zabu, CounterKind::PlusOne), 4);
        let old_power = g.stats(iron).power;
        let arc = put_paid(&mut g, p, "Arc Reactor");
        assert!(g.object_view(arc).tapped);
        assert!(!g.object_view(iron).tapped);
        assert_eq!(g.stats(iron).power, old_power + 1);
        let before = g.state.players[p.index()].library.len();
        let life = g.state.players[p.index()].life;
        let first_moloid = tokens(&g, p, "Moloid")[0];
        drive(&mut g, |g| {
            g.state.active == p && g.state.pending == Pending::Attackers
        });
        for card in [iron, nurse, first_moloid] {
            checked(&mut g, Action::Attack { card });
        }
        checked(&mut g, Action::FinishAttackers);
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        drive(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(&mut g, Action::FinishBlockers);
        drive(&mut g, |g| {
            g.outcome().is_some() || g.state.step == Step::Main2 && quiet(g)
        });
        assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        assert_eq!(g.state.players[p.index()].life, life + 4);
        // Let the army win by ordinary combat, with no concession or life injection.
        for _ in 0..5 {
            if g.outcome().is_some() {
                break;
            }
            next_own_main(&mut g, p);
            drive(&mut g, |g| {
                g.state.active == p && g.state.pending == Pending::Attackers
            });
            for card in [ka, mole, speed, iron, torch, nurse, spider, zabu] {
                if g.legal_actions().contains(&Action::Attack { card }) {
                    checked(&mut g, Action::Attack { card });
                }
            }
            checked(&mut g, Action::FinishAttackers);
            drive(&mut g, |g| {
                g.outcome().is_some() || g.state.step == Step::Main2 && quiet(g)
            });
        }
        assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
    }
}
// Neither the policy nor these checks receives an opponent's hidden identities.
// The counterfactual changes only unobserved cards and below-top library order.
fn hidden_information_contract(game: &Game, viewer: Player) {
    let observation = serde_json::to_value(game.observe(viewer)).unwrap();
    let mut alternative = game.clone();
    let hidden: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            let id = i as CardId;
            (o.owner == viewer.other()
                && o.zone == Zone::Hand
                && !game.state.known_hand.contains(&game.object_ref(id)))
            .then_some(i)
        })
        .take(2)
        .collect();
    if hidden.len() == 2 {
        let first = alternative.state.objects[hidden[0]].card;
        alternative.state.objects[hidden[0]].card = alternative.state.objects[hidden[1]].card;
        alternative.state.objects[hidden[1]].card = first;
    }
    let library = &mut alternative.state.players[viewer.other().index()].library;
    let below_top = library.len().saturating_sub(1);
    library[..below_top].reverse();
    assert_eq!(
        observation,
        serde_json::to_value(alternative.observe(viewer)).unwrap()
    );
    assert_eq!(game.legal_actions(), alternative.legal_actions());
}

fn owned_checked_step(mut game: Game, actor: Player, index: usize, action: Action) -> Game {
    let revision = game.revision();
    let before = game.snapshot();
    assert_eq!(
        game.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(game.snapshot(), before);
    assert_eq!(
        game.apply_index(actor, revision, usize::MAX),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    let (mut raw, mut indexed) = (game.clone(), game.clone());
    drop(game);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(raw.observe(viewer)).unwrap(),
            serde_json::to_value(indexed.observe(viewer)).unwrap()
        );
    }
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}

#[test]
fn msh_twelfth_combined_32_natural_standard_mirror_games_full_replay_and_all_eight_paid() {
    let membership: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/msh.json")).unwrap();
    let members: BTreeSet<_> = membership["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    let mut deck = Vec::new();
    for name in ["Plains", "Island", "Mountain", "Forest"] {
        assert!(members.contains(name));
        deck.extend([Card::from_name(name).unwrap(); 6]);
    }
    for name in EIGHT.into_iter().chain(["A.I.M. Scientists"]) {
        assert!(members.contains(name));
        let card = Card::from_name(name).unwrap();
        assert!(card.is_supported() && card.is_standard_legal());
        deck.extend([card; 4]);
    }
    assert_eq!(deck.len(), 60);
    let start = std::time::Instant::now();
    let mut records = Vec::new();
    let mut casts = BTreeSet::new();
    let mut events: BTreeMap<String, usize> = BTreeMap::new();
    for seed in 20261024..20261040 {
        for starting_player in [Player::P0, Player::P1] {
            let config = Config {
                decks: [deck.clone(), deck.clone()],
                seed,
                starting_player,
                deck_rules: DeckRules::Standard,
            };
            let mut g = Game::new(config.clone()).unwrap();
            let mut ledger = Vec::new();
            let mut checkpoints = 0;
            while g.outcome().is_none() {
                let actor = g.actor().unwrap();
                let actions = g.legal_actions();
                let index = crate::policy::smoke_index(&g, &actions);
                let action = actions[index].clone();
                assert!(!matches!(action, Action::Concede));
                let paid_name = if matches!(action, Action::PayCosts { .. }) {
                    g.state
                        .announcing
                        .as_ref()
                        .filter(|a| a.item.kind == StackKind::Spell)
                        .map(|a| a.item.source_card.name())
                } else {
                    None
                };
                if let Action::PlayLand { card } = action {
                    *events
                        .entry(format!("land_from_{:?}", g.object_view(card).zone))
                        .or_default() += 1;
                }
                if matches!(
                    action,
                    Action::ChooseOptional { .. }
                        | Action::PayResolutionMana { .. }
                        | Action::DeclineResolutionMana
                ) {
                    *events.entry(format!("{action:?}")).or_default() += 1;
                }
                ledger.push((actor, action.clone()));
                if ledger.len() % 97 == 1 {
                    hidden_information_contract(&g, actor);
                    g = owned_checked_step(g, actor, index, action);
                    checkpoints += 1;
                } else {
                    g.apply(actor, action).unwrap();
                }
                if let Some(name) = paid_name {
                    casts.insert(name);
                }
                assert!(
                    ledger.len() < 30_000,
                    "natural policy stalled seed{seed} starter{starting_player:?} pending{:?}",
                    g.state.pending
                );
            }
            assert!(matches!(g.outcome(), Some(Outcome::Winner(_))));
            let mut replay = Game::new(config).unwrap();
            for (actor, action) in &ledger {
                replay.apply(*actor, action.clone()).unwrap();
            }
            assert_eq!(g.snapshot(), replay.snapshot());
            assert_eq!(g.outcome(), replay.outcome());
            assert!(checkpoints > 1);
            records.push(serde_json::json!({"seed":seed,"starting_player":starting_player,"decisions":ledger.len(),"turn":g.state.turn,"outcome":g.outcome(),"owned_privacy_checkpoints":checkpoints}));
        }
    }
    assert_eq!(records.len(), 32);
    if let Ok(path) = std::env::var("RUSTMAGE_RECORD_MSH_TWELFTH_GAMES") {
        let path = std::path::PathBuf::from(path);
        assert!(
            path.starts_with(
                std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache")
            )
        );
        std::fs::write(path,serde_json::to_string_pretty(&serde_json::json!({"scope":"32 natural Standard60-card MSH-only four-color mirror games, unchanged observation-only smoke_index, no setup or progress injection, full replay and owned/privacy checks. Bounded evidence, not entire-set acceptance or simulation benchmark.","games":records,"paid_cast_names":casts,"observed_choices_and_land_origins":events,"checked_seconds":start.elapsed().as_secs_f64()})).unwrap()+"\n").unwrap();
    }
    for name in EIGHT {
        assert!(
            casts.contains(name),
            "no actual paid cast observed for {name}"
        );
    }
}
