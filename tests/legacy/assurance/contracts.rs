use super::*;

// Ownership, indices and restore are shared engine contracts, not repeated in
// every card test. Include suspended announcement, payment, private search and
// numeric combat choices in addition to ordinary priority.
pub(super) fn branch_contract(mut root: Game) {
    let actor = root.actor().unwrap();
    let revision = root.revision();
    let actions = root.legal_actions();
    let index = actions
        .iter()
        .position(|a| !matches!(a, Action::Concede))
        .unwrap();
    let action = actions[index].clone();
    let saved = root.snapshot();
    assert_eq!(
        root.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(root.snapshot(), saved);
    assert_eq!(
        root.apply_index(actor, revision, usize::MAX),
        Err(Error::IllegalAction)
    );
    assert_eq!(root.snapshot(), saved);
    assert_eq!(root.revision(), revision);
    let parent = root.clone();
    let mut indexed = parent.clone();
    drop(parent);
    indexed.apply_index(actor, revision, index).unwrap();
    root.apply(actor, action.clone()).unwrap();
    assert_eq!(root.snapshot(), indexed.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(root.observe(viewer)).unwrap(),
            serde_json::to_value(indexed.observe(viewer)).unwrap()
        );
    }
    let expected = root.snapshot();
    root.restore(&saved).unwrap();
    assert_eq!(
        root.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    root.apply(actor, action).unwrap();
    assert_eq!(root.snapshot(), expected);
    // A child remains writable after every ancestor is dropped.
    drop(root);
    let checkpoint = indexed.snapshot();
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.snapshot(), expected);
}

#[test]
fn owned_raw_index_and_restore_at_distinct_decision_roots() {
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    branch_contract(g.clone());
    let shock = put(&mut g, Player::P0, "Shock", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: shock });
    branch_contract(g.clone());
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    branch_contract(g);

    let mut g = scenario(Player::P1);
    let secret = put(&mut g, Player::P1, "Shock", Zone::Library);
    let mage = put(&mut g, Player::P1, "Micromancer", Zone::Hand);
    cast(&mut g, mage, None);
    for _ in 0..8 {
        if g.state.pending != Pending::Priority {
            break;
        }
        act(&mut g, Action::Pass);
    }
    if g.legal_actions()
        .contains(&Action::ChooseOptional { accept: true })
    {
        act(&mut g, Action::ChooseOptional { accept: true });
    }
    assert!(
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: secret })
    );
    branch_contract(g);

    let mut g = scenario(Player::P0);
    let bear = put(&mut g, Player::P0, "Aggressive Mammoth", Zone::Battlefield);
    let blocker = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: bear });
    act(&mut g, Action::FinishAttackers);
    for _ in 0..6 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(&mut g, Action::Pass);
    }
    act(
        &mut g,
        Action::Block {
            blocker,
            attacker: bear,
        },
    );
    act(&mut g, Action::FinishBlockers);
    for _ in 0..6 {
        if g.state.pending == Pending::Damage {
            break;
        }
        act(&mut g, Action::Pass);
    }
    assert_eq!(g.state.pending, Pending::Damage);
    branch_contract(g);
}

#[test]
fn late_resolution_overflow_rolls_back_earlier_counter_writes() {
    let mut g = scenario(Player::P0);
    let zimone = put(
        &mut g,
        Player::P0,
        "Zimone, Paradox Sculptor",
        Zone::Battlefield,
    );
    let artifact = put(&mut g, Player::P0, "Mox Jasper", Zone::Battlefield);
    g.state.objects[artifact as usize].counters.plus_one = 1;
    g.state.objects[artifact as usize].counters.stun = u64::MAX;
    choose(
        &mut g,
        |a| matches!(a,Action::AnnounceActivate{card,..}if *card==zimone),
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    act(&mut g, Action::FinishTargets);
    pay(&mut g);
    act(&mut g, Action::Pass);
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    assert!(matches!(
        g.apply(actor, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.counter_count(artifact, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(artifact, CounterKind::Stun), u64::MAX);
    assert!(g.object_view(zimone).tapped); // already paid; failed resolution stays on stack
    assert_eq!(g.state.stack.len(), 1);
}

#[test]
fn protocol_malformed_and_stale_requests_preserve_the_session() {
    use crate::protocol::{Request, Session};
    let mut session = Session::from_game(scenario(Player::P0));
    let request = r#"{"op":"observe","viewer":"P0"}"#;
    let before = session.json(request);
    for bad in [
        "{",
        r#"{"op":"unknown"}"#,
        r#"{"op":"observe","viewer":"P2"}"#,
        r#"{"op":"observe","viewer":"P0","extra":true}"#,
    ] {
        let result: serde_json::Value = serde_json::from_str(&session.json(bad)).unwrap();
        assert!(result["error"].is_string());
        assert_eq!(session.json(request), before);
    }
    let snapshot = session.request(Request::Snapshot).snapshot.unwrap();
    assert!(
        session
            .request(Request::Restore { snapshot })
            .error
            .is_none()
    );
    assert!(
        session
            .request(Request::Step {
                actor: Player::P0,
                revision: 0,
                index: 0,
                viewer: Player::P0
            })
            .error
            .unwrap()
            .contains("StaleRevision")
    );
    assert!(
        session
            .request(Request::DropSnapshot { snapshot })
            .error
            .is_none()
    );
    assert!(
        session
            .request(Request::Restore { snapshot })
            .error
            .is_some()
    );
}

#[test]
fn one_seeded_game_reaches_a_real_outcome_within_256_actions() {
    let mut deck = vec![Card::Mountain; 4];
    deck.extend([Card::from_name("Brazen Scourge").unwrap(); 3]);
    let mut g = Game::new(Config {
        decks: [deck.clone(), deck],
        seed: 23,
        starting_player: Player::P0,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    for _ in 0..256 {
        if g.outcome().is_some() {
            return;
        }
        let actions = g.legal_actions();
        let index = crate::policy::smoke_index(&g, &actions);
        assert!(!matches!(actions[index], Action::Concede));
        g.apply_index(g.actor().unwrap(), g.revision(), index)
            .unwrap();
    }
    panic!("256-action complete-game budget exhausted");
}
