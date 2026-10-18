//! Paid endgame scenarios reuse the shared raw/index owned-branch timeline driver.
use super::ability_timeline_tests::{cast, checked, next_turn, settle, setup};
use super::tests::spawn;
use super::*;
const GATES: [&str; 10] = [
    "Azorius Guildgate",
    "Boros Guildgate",
    "Dimir Guildgate",
    "Golgari Guildgate",
    "Gruul Guildgate",
    "Izzet Guildgate",
    "Orzhov Guildgate",
    "Rakdos Guildgate",
    "Selesnya Guildgate",
    "Simic Guildgate",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    for _ in 0..150 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::FinishTargets
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, action);
    }
    panic!("missing endgame boundary {:?}", g.state.pending);
}
fn empty_library(g: &mut Game, p: Player) {
    let cards = g.state.players[p.index()].library.clone();
    g.move_objects_raw(&cards, Zone::Exile);
}
fn activate_maze(g: &mut Game, maze: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: maze,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    for _ in 0..10 {
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| matches!(a, Action::ChooseAdditionalCost { object } if object.card == maze))
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            })
            .unwrap()
            .clone();
        checked(g, action);
    }
    panic!("unpaid Maze's End");
}
#[test]
fn endgame_herald_flash_prevents_persistent_losses_and_ability_removal_restores_them() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert_eq!(std::mem::size_of::<Pending>(), 40);
    assert_eq!(std::mem::size_of::<ManaProduction>(), 32);
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        checked(&mut g, Action::Pass);
        let herald = paid(&mut g, p, "Herald of Eternal Dawn", &[]);
        settle(&mut g);
        assert_eq!(
            g.stats(herald),
            Stats {
                power: 6,
                toughness: 6
            }
        );
        assert!(
            g.keywords(herald)
                .contains(Keywords::FLASH.union(Keywords::FLYING))
        );
        g.state.players[p.index()].life = -5;
        g.state.players[p.index()].poison_counters = 10;
        checked(&mut g, Action::Pass);
        assert_eq!(g.outcome(), None);
        // The active opponent can suppress the real static ability with a paid Aura.
        if g.actor() != Some(p.other()) {
            checked(&mut g, Action::Pass);
        }
        paid(
            &mut g,
            p.other(),
            "Eaten by Piranhas",
            &[Target::Card(herald)],
        );
        until(&mut g, |g| g.outcome().is_some());
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
    }
}
#[test]
fn endgame_prevented_empty_draw_is_consumed_and_does_not_follow_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let herald = paid(&mut g, p, "Herald of Eternal Dawn", &[]);
        settle(&mut g);
        empty_library(&mut g, p);
        paid(&mut g, p, "Opt", &[]);
        settle(&mut g);
        assert!(!g.state.players[p.index()].attempted_empty_draw);
        assert_eq!(g.outcome(), None);
        paid(&mut g, p, "Unsummon", &[Target::Card(herald)]);
        settle(&mut g);
        assert_eq!(
            g.outcome(),
            None,
            "prevented past draw is not a persistent loss"
        );
        paid(&mut g, p, "Opt", &[]);
        until(&mut g, |g| g.outcome().is_some());
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
    }
}
#[test]
fn endgame_herald_prevents_explicit_pact_loss_once_without_deferring_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let herald = paid(&mut g, p, "Herald of Eternal Dawn", &[]);
        settle(&mut g);
        paid(&mut g, p, "Demonic Pact", &[]);
        settle(&mut g);
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        until(&mut g, |g| matches!(g.state.pending, Pending::Modes { .. }));
        checked(&mut g, Action::ChooseMode { mode: 3 });
        settle(&mut g);
        assert_eq!(g.outcome(), None);
        paid(&mut g, p, "Unsummon", &[Target::Card(herald)]);
        settle(&mut g);
        assert_eq!(g.outcome(), None);
        checked(&mut g, Action::Pass);
        assert_eq!(g.outcome(), None);
    }
}
#[test]
fn endgame_maze_pays_source_return_before_search_then_wins_after_gate_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let maze = fixture(&mut g, p, "Maze's End", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: maze });
        assert!(g.object_view(maze).tapped);
        assert!(!g.legal_actions().iter().any(
            |a| matches!(a, Action::AnnounceActivate { card, ability: 1, .. } if *card == maze)
        ));
        next_turn(&mut g);
        next_turn(&mut g);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        for name in &GATES[..9] {
            fixture(&mut g, p, name, Zone::Battlefield);
        }
        let tenth = fixture(&mut g, p, GATES[9], Zone::Library);
        let source = g.object_ref(maze);
        activate_maze(&mut g, maze);
        assert_eq!(g.object_view(maze).zone, Zone::Hand);
        assert_ne!(g.object_ref(maze), source);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.stack.last().unwrap().object, source);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == tenth));
        checked(&mut g, Action::ChooseSearch { card: tenth });
        checked(&mut g, Action::FinishSearch);
        assert_eq!(g.object_view(tenth).zone, Zone::Battlefield);
        assert!(g.object_view(tenth).tapped);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
    }
}
#[test]
fn endgame_maze_uses_current_distinct_names_and_prevented_win_is_not_deferred() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let maze = fixture(&mut g, p, "Maze's End", Zone::Battlefield);
        let mut gates = Vec::new();
        for name in &GATES[..9] {
            gates.push(fixture(&mut g, p, name, Zone::Battlefield));
        }
        fixture(&mut g, p, GATES[0], Zone::Battlefield);
        assert_eq!(g.distinct_subtype_names(p, "Gate"), 9);
        // A real Moon effect removes Gate subtype despite retaining the physical card.
        paid(
            &mut g,
            p,
            "Imprisoned in the Moon",
            &[Target::Card(gates[0])],
        );
        settle(&mut g);
        assert_eq!(
            g.distinct_subtype_names(p, "Gate"),
            9,
            "the duplicate still supplies the name"
        );
        fixture(&mut g, p, GATES[9], Zone::Battlefield);
        let herald = fixture(
            &mut g,
            p.other(),
            "Herald of Eternal Dawn",
            Zone::Battlefield,
        );
        activate_maze(&mut g, maze);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        let rng = g.state.rng.clone();
        checked(&mut g, Action::FinishSearch);
        assert_ne!(
            g.state.rng, rng,
            "a zero-result qualified search still shuffles"
        );
        assert_eq!(g.outcome(), None);
        paid(&mut g, p, "Unsummon", &[Target::Card(herald)]);
        settle(&mut g);
        assert_eq!(
            g.outcome(),
            None,
            "only the resolving Maze ability checks for a win"
        );
    }
}
#[test]
fn endgame_time_stop_exiles_spells_and_skips_end_step_while_removing_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ball = fixture(&mut g, p, "Ball Lightning", Zone::Battlefield);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: ball });
        checked(&mut g, Action::FinishAttackers);
        let opt = paid(&mut g, p, "Opt", &[]);
        let stop = paid(&mut g, p, "Time Stop", &[]);
        let turn = g.state.turn;
        until(&mut g, |g| g.state.turn > turn || g.outcome().is_some());
        assert_eq!(g.object_view(opt).zone, Zone::Exile);
        assert_eq!(g.object_view(stop).zone, Zone::Exile);
        assert_eq!(
            g.object_view(ball).zone,
            Zone::Battlefield,
            "skipped end-step sacrifice never triggered"
        );
        assert!(
            g.state.attackers.is_empty()
                && g.state.blocking.is_empty()
                && g.state.blocks.is_empty()
        );
        assert!(g.state.stack.is_empty() && g.state.resolving.is_none());
        assert_eq!(g.state.active, p.other());
        assert_eq!(g.state.step, Step::Upkeep);
    }
}
#[test]
fn endgame_time_stop_cleanup_discards_trigger_priority_and_repeat_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mako = fixture(&mut g, p, "Marauding Mako", Zone::Battlefield);
        let bear = fixture(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        paid(&mut g, p, "Giant Growth", &[Target::Card(bear)]);
        settle(&mut g);
        g.state.objects[bear as usize].damage = 2;
        for _ in 0..8 {
            fixture(&mut g, p, "Forest", Zone::Hand);
        }
        let stop = paid(&mut g, p, "Time Stop", &[]);
        let turn = g.state.turn;
        until(&mut g, |g| g.state.pending == Pending::Discard);
        assert_eq!(g.state.step, Step::Cleanup);
        assert_eq!(g.object_view(stop).zone, Zone::Exile);
        assert_eq!(
            g.object_view(bear).damage,
            2,
            "discard precedes damage removal"
        );
        let card = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Discard { .. }))
            .unwrap();
        checked(&mut g, card);
        assert_eq!(g.state.turn, turn);
        assert_eq!(g.state.step, Step::Cleanup);
        assert_eq!(g.object_view(bear).damage, 0);
        assert_eq!(g.stats(bear).power, 2);
        settle(&mut g);
        assert_eq!(g.counter_count(mako, CounterKind::PlusOne), 1);
        assert_eq!(
            g.state.turn, turn,
            "cleanup-trigger priority precedes the repeated cleanup"
        );
        next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
    }
}
#[test]
fn endgame_simultaneous_loss_draw_and_concession_remain_distinct() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[0].life = 0;
        g.state.players[1].life = 0;
        checked(&mut g, Action::Pass);
        assert_eq!(g.outcome(), Some(Outcome::Draw));
        let mut g = setup(p);
        fixture(&mut g, p, "Herald of Eternal Dawn", Zone::Battlefield);
        g.state.players[0].life = 0;
        g.state.players[1].life = 0;
        checked(&mut g, Action::Pass);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
        let mut g = setup(p);
        fixture(&mut g, p, "Herald of Eternal Dawn", Zone::Battlefield);
        g.apply(p, Action::Concede).unwrap();
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
    }
}

#[test]
fn endgame_herald_control_and_cleanup_restoration_change_the_protected_player_immediately() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let herald = fixture(
            &mut g,
            p.other(),
            "Herald of Eternal Dawn",
            Zone::Battlefield,
        );
        g.state.players[p.other().index()].life = 0;
        paid(&mut g, p, "Act of Treason", &[Target::Card(herald)]);
        until(&mut g, |g| g.outcome().is_some());
        assert_eq!(g.object_view(herald).controller, p);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
        let mut g = setup(p);
        let herald = fixture(
            &mut g,
            p.other(),
            "Herald of Eternal Dawn",
            Zone::Battlefield,
        );
        paid(&mut g, p, "Act of Treason", &[Target::Card(herald)]);
        settle(&mut g);
        g.state.players[p.index()].life = 0;
        let turn = g.state.turn;
        paid(&mut g, p, "Time Stop", &[]);
        until(&mut g, |g| g.outcome().is_some());
        assert_eq!(g.object_view(herald).controller, p.other());
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
        assert_eq!(g.state.step, Step::Cleanup);
        assert_eq!(g.state.turn, turn, "cleanup loss precedes the next turn");
    }
}
#[test]
fn endgame_simultaneous_herald_death_uses_pre_event_loss_restrictions() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let herald = fixture(&mut g, p, "Herald of Eternal Dawn", Zone::Battlefield);
        g.state.objects[herald as usize].counters.minus_one = 6;
        g.state.players[0].life = 0;
        g.state.players[1].life = 0;
        checked(&mut g, Action::Pass);
        assert_eq!(g.object_view(herald).zone, Zone::Graveyard);
        assert_eq!(
            g.outcome(),
            Some(Outcome::Winner(p)),
            "only the unprotected opponent lost in the frozen simultaneous event"
        );
    }
}
