//! Actual TDM combat providers composed without fixture rule overlays.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const LIMIT: usize = 256;

fn quiet(g: &Game) -> bool {
    g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.announcing.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn ready(g: &mut Game, done: impl Fn(&Game) -> bool) {
    for _ in 0..LIMIT {
        if g.state.pending == Pending::Priority {
            g.prepare_triggers();
        }
        if done(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("256-action combat composition limit");
}
fn drain(g: &mut Game) {
    ready(g, quiet);
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn warriors(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id as CardId, "Warrior"))
            .then_some(id as CardId)
        })
        .collect()
}
struct Scene {
    game: Game,
    zurgo: CardId,
    assault: CardId,
    idle: CardId,
    enemy: CardId,
    secret: CardId,
}
fn attack_order_root(p: Player) -> Scene {
    let mut g = scenario(p);
    let zurgo = put(&mut g, p, "Zurgo, Thunder's Decree", Zone::Battlefield);
    let idle = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let secret = put(&mut g, p, "Cancel", Zone::Hand);
    let siege = paid(&mut g, p, "Windcrag Siege", [1, 0, 0, 1, 0, 1]);
    let mardu = Action::ChooseEntryValue {
        value: EntryChoiceValue::NamedMode(0),
    };
    ready(&mut g, |g| g.legal_actions().contains(&mardu));
    act(&mut g, mardu);
    drain(&mut g);
    assert_eq!(
        g.object_view(siege).entry_choice,
        Some(EntryChoiceValue::NamedMode(0))
    );
    let assault = paid(&mut g, p, "All-Out Assault", [1, 0, 1, 1, 0, 2]);
    drain(&mut g);
    assert_eq!(stats(&g, zurgo), (3, 5));
    assert!(g.keywords(zurgo).contains(Keywords::DEATHTOUCH));
    // The isolated All-Out scenarios prove its complete phase ordering. Here
    // its real entry also schedules the same combat/main/natural-combat chain.
    assert_eq!(
        g.observe(p).inserted_phases,
        Some(vec![Step::BeginCombat, Step::Main2, Step::BeginCombat])
    );
    assert_eq!(g.observe(p).turn_callbacks.unwrap().len(), 1);
    g.move_object(assault, Zone::Graveyard);
    assert_eq!(stats(&g, zurgo), (2, 4));
    assert!(!g.keywords(zurgo).contains(Keywords::DEATHTOUCH));
    g.state.objects[idle as usize].tapped = true;
    g.state.objects[enemy as usize].tapped = true;
    ready(&mut g, |g| g.state.pending == Pending::Attackers);
    act(&mut g, Action::Attack { card: zurgo });
    act(&mut g, Action::FinishAttackers);
    if g.state.pending == Pending::Priority {
        g.prepare_triggers();
    }
    assert!(matches!(g.state.pending, Pending::OrderTriggers { .. }));
    assert_eq!(g.state.ordering_triggers.len(), 3);
    assert_eq!(
        g.state
            .ordering_triggers
            .iter()
            .filter(|s| s.source.is_some_and(|v| v.id == zurgo))
            .count(),
        2
    );
    assert_eq!(
        g.state
            .ordering_triggers
            .iter()
            .filter(|s| s.source.is_some_and(|v| v.id == assault))
            .count(),
        1,
        "Siege duplicates the permanent's Mobilize, not the delayed ability"
    );
    assert!(g.observe(p).turn_callbacks.is_none());
    assert!(g.object_view(zurgo).tapped);
    assert!(warriors(&g, p).is_empty());
    Scene {
        game: g,
        zurgo,
        assault,
        idle,
        enemy,
        secret,
    }
}
fn mobilize_first_in_stack_order(g: &mut Game, zurgo: CardId) {
    for _ in 0..3 {
        if !matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            break;
        }
        let id = g
            .state
            .ordering_triggers
            .iter()
            .find(|s| s.source.is_some_and(|v| v.id == zurgo))
            .unwrap()
            .id;
        act(g, Action::PutTrigger { id });
    }
    assert!(!matches!(g.state.pending, Pending::OrderTriggers { .. }));
    drain(g);
}
fn check_four(g: &Game, p: Player, zurgo: CardId, tapped: bool) -> Vec<CardId> {
    let ids = warriors(g, p);
    assert_eq!(
        ids.len(),
        4,
        "two copies of Mobilize 2, no declaration recursion"
    );
    assert!(warriors(g, p.other()).is_empty());
    assert_eq!(g.state.attackers.len(), 5);
    assert!(g.state.attackers.iter().any(|a| a.card == zurgo));
    for &id in &ids {
        assert_eq!(stats(g, id), (1, 1));
        assert_eq!(g.object_view(id).colors, 8);
        assert_eq!(g.object_view(id).tapped, tapped);
        assert!(g.state.attackers.iter().any(|a| a.card == id));
    }
    ids
}
fn next_end(g: &mut Game) {
    // Only this fixed scene's explicit empty next attack/block declaration and
    // trigger order are advanced; there is no general-purpose game-playing bot.
    for _ in 0..LIMIT {
        if g.state.pending == Pending::Priority {
            g.prepare_triggers();
        }
        if g.state.step == Step::End && quiet(g) {
            return;
        }
        match g.state.pending {
            Pending::Attackers => act(g, Action::FinishAttackers),
            Pending::Blockers => act(g, Action::FinishBlockers),
            Pending::Priority => act(g, Action::Pass),
            Pending::OrderTriggers { .. } => choose(g, |a| matches!(a, Action::PutTrigger { .. })),
            _ => panic!("unexpected composed end-step choice {:?}", g.state.pending),
        }
    }
    panic!("256-action composed end-step limit");
}

#[test]
fn f03_siege_doubles_zurgo_mobilize_with_source_independent_untap_and_end_step_sacrifice() {
    for p in [Player::P0, Player::P1] {
        for remove_zurgo in [false, true] {
            let Scene {
                mut game,
                zurgo,
                assault,
                idle,
                enemy,
                ..
            } = attack_order_root(p);
            mobilize_first_in_stack_order(&mut game, zurgo);
            let tokens = check_four(&game, p, zurgo, true);
            assert!(!game.object_view(zurgo).tapped);
            assert!(
                !game.object_view(idle).tapped,
                "departed All-Out's next-attack packet still untaps own creatures"
            );
            assert!(game.object_view(enemy).tapped);
            assert_eq!(game.object_view(assault).zone, Zone::Graveyard);
            if remove_zurgo {
                game.move_object(zurgo, Zone::Graveyard);
            }
            next_end(&mut game);
            assert_eq!(warriors(&game, p).len(), if remove_zurgo { 0 } else { 4 });
            for token in tokens {
                assert_eq!(
                    game.object_view(token).zone,
                    if remove_zurgo {
                        Zone::Outside
                    } else {
                        Zone::Battlefield
                    }
                );
                if !remove_zurgo {
                    assert!(game.keywords(token).contains(Keywords::CANT_BE_SACRIFICED));
                }
            }
        }
    }
}

#[test]
fn f03_actual_attack_trigger_order_is_owned_raw_index_restorable_and_private() {
    for p in [Player::P0, Player::P1] {
        let Scene {
            game,
            zurgo,
            assault,
            idle,
            enemy,
            secret,
        } = attack_order_root(p);
        let mobilize_id = game
            .state
            .ordering_triggers
            .iter()
            .find(|s| s.source.is_some_and(|v| v.id == zurgo))
            .unwrap()
            .id;
        let untap_id = game
            .state
            .ordering_triggers
            .iter()
            .find(|s| s.source.is_some_and(|v| v.id == assault))
            .unwrap()
            .id;
        let action = Action::PutTrigger { id: mobilize_id };
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let saved = game.snapshot();
        let mut raw = game.clone();
        let mut indexed = game.clone();
        let mut sibling = game.clone();
        drop(game);
        assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
        assert_eq!(raw.snapshot(), saved);
        reject(&mut raw, Action::PutTrigger { id: u64::MAX });
        raw.apply(p, action.clone()).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        act(&mut sibling, Action::PutTrigger { id: untap_id });
        for child in [&mut raw, &mut indexed, &mut sibling] {
            mobilize_first_in_stack_order(child, zurgo);
            assert!(!child.object_view(zurgo).tapped && !child.object_view(idle).tapped);
            assert!(child.object_view(enemy).tapped);
            assert!(child.observe(p).objects.iter().any(|o| o.id == secret));
            assert!(
                !child
                    .observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.id == secret)
            );
        }
        check_four(&raw, p, zurgo, true);
        check_four(&sibling, p, zurgo, false);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        mobilize_first_in_stack_order(&mut indexed, zurgo);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
