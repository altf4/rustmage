//! Paid interactions across the separately authored mechanics in this cohort.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::msh_thirteenth_combined_tests::owned_checked_step;
use super::*;
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, generic) = Card::from_name(name).unwrap().cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = packet(g, p, name, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..300 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let index = g.legal_actions().iter().position(|v| *v == a).unwrap();
    owned_checked_step(g, actor, index, a)
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseActivationCost { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished announcement");
}
fn activate(g: &mut Game, p: Player, source: CardId, ability: u16, mana: Mana) {
    priority(g, p);
    g.state.players[p.index()].mana = mana;
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn copies(g: &Game, p: Player, name: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_copy_token()
                && g.object_view(i as CardId).name() == name)
                .then_some(i as CardId)
        })
        .collect()
}
#[test]
fn msh_fourteenth_paid_incursion_registers_loki_justice_doc_and_ant_before_real_entry_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let doc = paid(&mut g, p, "Doc Samson, Super Psychiatrist", &[]);
        let ant = paid(&mut g, p, "The Astonishing Ant-Man", &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let loki = paid(&mut g, p, "Loki, God of Mischief", &[]);
        let justice = paid(&mut g, p, "Justice, Vance Astrovik", &[]);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        let before = g.hand_count(p);
        let inc = packet(&mut g, p, "Multiversal Incursion", &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
                && g.state.announcing.as_ref().is_some_and(|a| {
                    g.state.objects[a.item.card as usize].is_copy_token()
                        && a.item.source_card.name() == "Justice, Vance Astrovik"
                })
        });
        for name in [
            "Doc Samson, Super Psychiatrist",
            "The Astonishing Ant-Man",
            "Bear Cub",
            "Loki, God of Mischief",
            "Justice, Vance Astrovik",
        ] {
            assert_eq!(copies(&g, p, name).len(), 1, "{name}");
        }
        let ant_copy = copies(&g, p, "The Astonishing Ant-Man")[0];
        let justice_copy = copies(&g, p, "Justice, Vance Astrovik")[0];
        let loki_copy = copies(&g, p, "Loki, God of Mischief")[0];
        assert!(!g.object_view(loki_copy).is_legendary());
        assert_eq!(g.counter_count(ant_copy, CounterKind::PlusOne), 0);
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        until(&mut g, |g| g.hand_count(p) == before + 2);
        assert_eq!(
            g.object_view(bear).zone,
            Zone::Battlefield,
            "both independent Loki draws precede the causing return"
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 3);
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
        assert_eq!(g.object_view(inc).zone, Zone::Graveyard);
        for id in [ant, ant_copy] {
            assert_eq!(
                g.counter_count(id, CounterKind::PlusOne),
                6,
                "two draws each put one plus two Doc additions"
            );
        }
        for id in [justice, justice_copy] {
            assert_eq!(
                g.counter_count(id, CounterKind::PlusOne),
                3,
                "one other return plus two Doc additions"
            );
        }
        assert_eq!(g.counter_count(doc, CounterKind::PlusOne), 0);
        let draw_count = g.state.players[p.index()].library.len();
        let bear_copy = copies(&g, p, "Bear Cub")[0];
        activate(&mut g, p, pole, 0, [0, 0, 0, 0, 0, 2]);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear_copy),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].library.len(),
            draw_count,
            "both original and new Loki consumed their own once-per-turn histories"
        );
        assert_eq!(g.object_view(loki).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_fourteenth_paid_power_up_counters_feed_hyde_origin_entry_and_strat_connive() {
    use super::assurance::big_helpers::next_turn;
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let thanos = paid(&mut g, p, "Thanos, the Mad Titan", &[]);
        activate(&mut g, p, thanos, 0, [0, 1, 0, 0, 1, 1]);
        finish(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ManaValueParity { .. })
        });
        g = orphan(
            g,
            Action::ChooseManaValueParity {
                parity: ManaValueParity::Even,
            },
        );
        settle(&mut g);
        assert_eq!(g.counter_count(thanos, CounterKind::PlusOne), 2);
        let hyde = paid(&mut g, p, "Mister Hyde, Monster Within", &[]);
        let saga = paid(&mut g, p, "Origin of the Avengers", &[]);
        let nick = named(&mut g, p, "Nick Fury, Agent of S.H.I.E.L.D.", Zone::Hand);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| matches!(g.state.pending, Pending::Modes { .. }));
        g = orphan(g, Action::ChooseMode { mode: 1 });
        until(&mut g, |g| g.observe(p).effect_choice.is_some());
        let hand = g.hand_count(p);
        let object = g.object_ref(thanos);
        g = orphan(
            g,
            Action::ChooseCreatureCounterCost {
                object,
                counter: CounterKind::PlusOne,
            },
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        assert_eq!(g.counter_count(thanos, CounterKind::PlusOne), 1);
        until(&mut g, |g| g.observe(p).effect_choice.is_some());
        assert_eq!(g.counter_count(saga, CounterKind::Lore), 2);
        let object = g.object_ref(nick);
        g = orphan(g, Action::PutHandPermanent { object });
        settle(&mut g);
        assert_eq!(g.object_view(nick).zone, Zone::Battlefield);
        let hero = named(&mut g, p, "Jennifer Walters", Zone::Library);
        activate(&mut g, p, nick, 0, [0, 1, 1, 1, 1, 0]);
        finish(&mut g);
        until(&mut g, |g| g.observe(p).library_selection.is_some());
        g = orphan(g, Action::SelectLibrary { card: hero });
        checked(&mut g, Action::FinishLibrarySelection);
        g = orphan(g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.object_view(hero).name(), "The Sensational She-Hulk");
        assert_eq!(g.counter_count(nick, CounterKind::PlusOne), 2);
        let enemy = named(&mut g, p.other(), "The Serpent Society", Zone::Battlefield);
        let discard = named(&mut g, p, "Shock", Zone::Hand);
        let spell = packet(
            &mut g,
            p,
            "Trickster's Stratagem",
            &[Target::Card(enemy), Target::Card(hyde)],
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        g = orphan(g, Action::ChooseOptional { accept: true });
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::TargetLibrary { .. })
        });
        let object = g.object_ref(enemy);
        g = orphan(
            g,
            Action::ChooseLibraryDestination {
                object,
                destination: LibraryDestination::SecondFromTop,
            },
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        g = orphan(g, Action::Discard { card: discard });
        settle(&mut g);
        assert_eq!(g.counter_count(hyde, CounterKind::PlusOne), 1);
        assert_eq!(g.object_view(enemy).zone, Zone::Library);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].poison_counters, 5);
        assert_eq!(g.counter_count(thanos, CounterKind::PlusOne), 1);
    }
}
