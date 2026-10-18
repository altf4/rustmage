use super::tests::{scenario, spawn};
use super::*;

fn setup(player: Player) -> Game {
    let mut game = scenario();
    game.state.active = player;
    game.state.priority = player;
    for player in &mut game.state.players {
        player.mana = [20; 6];
    }
    game
}
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn checked(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let old_revision = g.revision();
    let actor = g.actor().unwrap();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .expect("legal action");
    let mut indexed = g.clone();
    indexed
        .apply_index(actor, indexed.revision(), index)
        .unwrap();
    g.apply(actor, action.clone()).unwrap();
    let after = g.snapshot();
    assert_eq!(indexed.snapshot(), after);
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, old_revision, index),
        Err(Error::StaleRevision)
    );
    g.apply(actor, action).unwrap();
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, action: Action) {
    let state = g.snapshot();
    let rev = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), state);
    assert_eq!(g.revision(), rev);
}
fn settle(g: &mut Game) {
    for _ in 0..30 {
        if g.outcome().is_some()
            || (g.state.stack.is_empty() && g.state.pending == Pending::Priority)
        {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        checked(g, Action::Pass);
    }
    panic!("unsettled Pact scenario");
}
fn cast(g: &mut Game, id: CardId, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    for target in targets {
        checked(g, Action::ChooseTarget { target: *target });
    }
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, pay);
    settle(g);
}
fn upkeep(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.begin_step(Step::Upkeep);
    g.prepare_triggers();
}
fn modes(g: &Game) -> Vec<u16> {
    g.observe(g.actor().unwrap())
        .mode_choices
        .into_iter()
        .map(|m| m.mode)
        .collect()
}
fn choose(g: &mut Game, mode: u16, target: Option<Target>) {
    checked(g, Action::ChooseMode { mode });
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
}

#[test]
fn pact_paid_cast_all_four_printed_modes_use_real_upkeep_damage_private_discards_draw_and_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pact = named(&mut g, p, "Demonic Pact", Zone::Hand);
        let a = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let b = named(&mut g, p.other(), "Llanowar Elves", Zone::Hand);
        cast(&mut g, pact, &[]);
        assert_eq!(g.object_view(pact).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana[2], 18);
        assert_eq!(g.state.players[p.index()].mana.iter().sum::<u32>(), 116);
        upkeep(&mut g, p);
        assert_eq!(modes(&g), vec![0, 1, 2, 3]);
        assert!(g.observe(p.other()).mode_choices.is_empty());
        choose(&mut g, 0, Some(Target::Player(p.other())));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 24);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        upkeep(&mut g, p);
        assert_eq!(modes(&g), vec![1, 2, 3]);
        reject(&mut g, Action::ChooseMode { mode: 0 });
        choose(&mut g, 1, Some(Target::Player(p.other())));
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p.other()));
        assert!(g.observe(p).actions.is_empty());
        assert!(g.observe(p).effect_discards.is_empty());
        checked(&mut g, Action::Discard { card: a });
        checked(&mut g, Action::Discard { card: b });
        settle(&mut g);
        assert_eq!(g.hand_count(p.other()), 0);
        upkeep(&mut g, p);
        let hand = g.hand_count(p);
        choose(&mut g, 2, None);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 2);
        upkeep(&mut g, p);
        assert_eq!(modes(&g), vec![3]);
        choose(&mut g, 3, None);
        settle(&mut g);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
        assert!(g.actor().is_none());
        assert!(g.legal_actions().is_empty());
        assert_eq!(g.state.players[p.index()].life, 24);
    }
}

#[test]
fn pact_all_target_illegal_fizzles_life_gain_but_consumes_mode_even_after_countering() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Demonic Pact", Zone::Battlefield);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        upkeep(&mut g, p);
        choose(&mut g, 0, Some(Target::Card(bear)));
        g.move_object(bear, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        upkeep(&mut g, p);
        assert_eq!(modes(&g), vec![1, 2, 3]);
        choose(&mut g, 3, None);
        let id = g.state.stack.last().unwrap().id;
        assert!(g.counter_stack_item(id, false));
        upkeep(&mut g, p);
        assert_eq!(modes(&g), vec![1, 2]);
        assert!(g.outcome().is_none());
    }
}

#[test]
fn pact_history_follows_permanent_through_paid_control_transfer_and_not_the_original_player() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pact = named(&mut g, p, "Demonic Pact", Zone::Battlefield);
        upkeep(&mut g, p);
        choose(&mut g, 2, None);
        settle(&mut g);
        g.begin_step(Step::Main1);
        let gift = named(&mut g, p, "Harmless Offering", Zone::Hand);
        cast(
            &mut g,
            gift,
            &[Target::Player(p.other()), Target::Card(pact)],
        );
        assert_eq!(g.object_view(pact).controller, p.other());
        upkeep(&mut g, p.other());
        assert_eq!(modes(&g), vec![0, 1, 3]);
        assert_eq!(
            g.observe(p).modal_history,
            g.observe(p.other()).modal_history
        );
        choose(&mut g, 3, None);
        settle(&mut g);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
    }
}

#[test]
fn pact_pending_old_incarnation_retains_history_after_reentry_and_new_copy_starts_fresh() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pact = named(&mut g, p, "Demonic Pact", Zone::Battlefield);
        upkeep(&mut g, p);
        choose(&mut g, 2, None);
        settle(&mut g);
        let old = g.object_ref(pact);
        upkeep(&mut g, p);
        g.move_object(pact, Zone::Exile);
        g.move_object(pact, Zone::Battlefield);
        assert_ne!(g.object_ref(pact), old);
        assert_eq!(modes(&g), vec![0, 1, 3]);
        reject(&mut g, Action::ChooseMode { mode: 2 });
        choose(&mut g, 0, Some(Target::Player(p.other())));
        settle(&mut g);
        assert!(
            g.state.modal_history.is_none(),
            "departed history is reclaimed after final old trigger"
        );
        upkeep(&mut g, p);
        assert_eq!(modes(&g), vec![0, 1, 2, 3]);
        choose(&mut g, 2, None);
        settle(&mut g);
        let source = g.object_view(pact);
        g.stage_copy_creation(
            p,
            CopyRecipe {
                definition: source.card,
            },
            1,
        );
        let copy = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(id, o)| *id != pact as usize && o.copy_token && o.card.name() == "Demonic Pact")
            .map(|(id, _)| id as CardId)
            .unwrap();
        assert!(g.queue_trigger(&g.object_view(copy), 0));
        g.prepare_triggers();
        assert_eq!(modes(&g), vec![0, 1, 2, 3]);
        choose(&mut g, 2, None);
        settle(&mut g);
        assert_eq!(g.observe(p).modal_history.len(), 2);
    }
}

#[test]
fn pact_multiple_waiting_triggers_cannot_reuse_modes_and_exhaustion_removes_trigger_without_deadlock()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pact = named(&mut g, p, "Demonic Pact", Zone::Battlefield);
        for _ in 0..2 {
            assert!(g.queue_trigger(&g.object_view(pact), 0));
        }
        g.prepare_triggers();
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
            .unwrap();
        checked(&mut g, action);
        choose(&mut g, 2, None);
        assert_eq!(modes(&g), vec![0, 1, 3]);
        reject(&mut g, Action::ChooseMode { mode: 2 });
        choose(&mut g, 3, None);
        let ids: Vec<_> = g.state.stack.iter().map(|s| s.id).collect();
        for id in ids {
            assert!(g.counter_stack_item(id, false));
        }
        // Mark the two targeted modes through authentic announcements and counter
        // their stack items, so exhaustion can be tested without ending the game.
        for mode in [0, 1] {
            upkeep(&mut g, p);
            choose(&mut g, mode, Some(Target::Player(p.other())));
            let id = g.state.stack.last().unwrap().id;
            assert!(g.counter_stack_item(id, false));
        }
        assert_eq!(g.observe(p).modal_history[0].modes, vec![0, 1, 2, 3]);
        upkeep(&mut g, p);
        assert!(g.state.stack.is_empty());
        assert!(g.state.announcing.is_none());
        assert_eq!(g.state.pending, Pending::Priority);
        assert!(g.outcome().is_none());
    }
}

#[test]
fn pact_history_is_owned_public_and_resolution_capacity_failure_is_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Demonic Pact", Zone::Battlefield);
        upkeep(&mut g, p);
        choose(&mut g, 2, None);
        settle(&mut g);
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        assert!(
            g.state
                .modal_history
                .as_ref()
                .unwrap()
                .ptr_eq(child.state.modal_history.as_ref().unwrap())
        );
        upkeep(&mut child, p);
        choose(&mut child, 0, Some(Target::Player(p.other())));
        assert_eq!(g.observe(p).modal_history[0].modes, vec![2]);
        assert_eq!(child.observe(p).modal_history[0].modes, vec![0, 2]);
        assert!(
            !g.state
                .modal_history
                .as_ref()
                .unwrap()
                .ptr_eq(child.state.modal_history.as_ref().unwrap())
        );
        child.state.players[p.index()].life = i128::MAX;
        checked(&mut child, Action::Pass);
        let before = child.snapshot();
        let revision = child.revision();
        assert!(matches!(
            child.apply(child.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(child.snapshot(), before);
        assert_eq!(child.revision(), revision);
        assert_eq!(child.state.players[p.other().index()].life, 20);
        assert_eq!(child.observe(p).modal_history[0].modes, vec![0, 2]);
        let public = serde_json::to_value(child.observe(p)).unwrap();
        assert_eq!(
            public["modal_history"][0]["modes"],
            serde_json::json!([0, 2])
        );
        assert_eq!(
            child.observe(p).modal_history,
            child.observe(p.other()).modal_history
        );
        assert_eq!(std::mem::size_of::<Object>(), 144);
    }
}
