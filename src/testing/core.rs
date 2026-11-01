//! Engine contracts are checked here, once, independently of card coverage.
use super::*;

#[test]
fn owned_branches_raw_actions_indices_and_restore() {
    let mut parent = game();
    let card = put(&mut parent, Player::P0, "Shock", Zone::Hand);
    let action = Action::AnnounceCast { card };
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let original = parent.snapshot();
    assert_eq!(
        parent.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(parent.snapshot(), original);
    assert_eq!(
        parent.apply_index(Player::P0, revision, usize::MAX),
        Err(Error::IllegalAction)
    );
    assert_eq!(parent.snapshot(), original);
    let mut child = parent.clone();
    child.apply_index(Player::P0, revision, index).unwrap();
    act(&mut parent, action.clone());
    assert_eq!(parent.snapshot(), child.snapshot());
    parent.restore(&original).unwrap();
    assert_eq!(parent.snapshot(), original);
    assert_eq!(
        parent.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut parent, action);
    assert_eq!(parent.snapshot(), child.snapshot());
    drop(parent);
    act(
        &mut child,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    pay(&mut child);
    resolve(&mut child);
    assert_eq!(child.state.players[1].life, 18);
}

#[test]
fn hidden_hands_and_draws_are_visible_only_to_the_owner() {
    let mut g = game();
    let secret = put(&mut g, Player::P0, "Shock", Zone::Library);
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    assert!(
        !g.observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == secret || o.id == spell)
    );
    cast(&mut g, spell, &[]);
    resolve(&mut g);
    assert!(
        g.observe(Player::P0)
            .objects
            .iter()
            .any(|o| o.id == secret && o.zone == Zone::Hand)
    );
    assert!(!g.observe(Player::P1).objects.iter().any(|o| o.id == secret));
}

#[test]
fn failed_resolution_restores_earlier_writes_and_leaves_the_stack_intact() {
    let mut g = game();
    let source = put(
        &mut g,
        Player::P0,
        "Zimone, Paradox Sculptor",
        Zone::Battlefield,
    );
    let target = put(&mut g, Player::P0, "Mox Jasper", Zone::Battlefield);
    g.state.objects[target as usize].counters.plus_one = 1;
    g.state.objects[target as usize].counters.stun = u64::MAX;
    choose(
        &mut g,
        |a| matches!(a, Action::AnnounceActivate {card,..} if *card==source),
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    act(&mut g, Action::FinishTargets);
    pay(&mut g);
    act(&mut g, Action::Pass);
    let before = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    assert!(matches!(
        g.apply(actor, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
    assert_eq!(g.state.stack.len(), 1);
}

#[test]
fn complete_game_has_a_real_outcome_within_256_actions() {
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
    panic!("complete game exceeded 256 actions");
}
