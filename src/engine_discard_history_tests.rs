use super::tests::{act, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(f.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn announce(g: &mut Game, card: CardId) {
    replay(
        g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
fn choose(g: &mut Game, card: CardId) {
    replay(
        g,
        Action::ChooseAdditionalCost {
            object: g.object_ref(card),
        },
    );
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
fn discard(g: &mut Game, card: CardId) {
    replay(g, Action::Discard { card });
}
fn prior(g: &mut Game, p: Player, n: usize) {
    let ids: Vec<_> = (0..n).map(|_| named(g, p, "Forest", Zone::Hand)).collect();
    g.discard_batch(&ids, None);
}
#[test]
fn committed_batches_count_both_players_before_observers_and_ordinary_moves_do_not() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let b = named(&mut g, Player::P1, "Mountain", Zone::Hand);
    let c = named(&mut g, Player::P0, "Swamp", Zone::Hand);
    g.discard_batch(&[a, b, c], None);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 2);
    assert_eq!(g.state.players[1].cards_discarded_this_turn, 1);
    for p in [Player::P0, Player::P1] {
        let o = g.observe(p);
        assert_eq!(o.players[0].cards_discarded_this_turn, 2);
        assert_eq!(o.players[1].cards_discarded_this_turn, 1);
    }
    let d = named(&mut g, Player::P0, "Forest", Zone::Hand);
    g.move_object(d, Zone::Graveyard);
    let e = named(&mut g, Player::P0, "Forest", Zone::Hand);
    g.move_object(e, Zone::Exile);
    let f = g.state.players[0].library.last().copied().unwrap();
    g.move_object(f, Zone::Graveyard);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 2);
    g.move_object(a, Zone::Hand);
    g.discard_batch(&[a], None);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 3);
    g.discard_batch(&[], None);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 3);
}
#[test]
fn misty_counts_its_committed_cost_and_uses_global_history_before_its_own_entry() {
    for n in [0, 2, 5] {
        let mut g = scenario();
        prior(&mut g, Player::P0, n);
        prior(&mut g, Player::P1, 3);
        let misty = named(
            &mut g,
            Player::P0,
            "Misty Knight, Hero for Hire",
            Zone::Battlefield,
        );
        let card = named(&mut g, Player::P0, "Forest", Zone::Hand);
        g.state.players[0].mana = [0, 0, 0, 2, 0, 0];
        let s = g.snapshot();
        let _ = g.legal_actions();
        assert_eq!(g.snapshot(), s);
        announce(&mut g, misty);
        choose(&mut g, card);
        assert_eq!(g.state.players[0].cards_discarded_this_turn, n as u64);
        pay(&mut g);
        assert_eq!(g.state.players[0].cards_discarded_this_turn, n as u64 + 1);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
        assert!(g.object_view(misty).tapped);
        passes(&mut g);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, n as u64 + 1);
        assert_eq!(g.state.players[1].cards_discarded_this_turn, 3);
    }
}
#[test]
fn misty_resolution_observes_later_costs_and_survives_source_departure() {
    let mut g = scenario();
    let misty = named(
        &mut g,
        Player::P0,
        "Misty Knight, Hero for Hire",
        Zone::Battlefield,
    );
    let knight = named(
        &mut g,
        Player::P0,
        "Charging Strifeknight",
        Zone::Battlefield,
    );
    let a = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let b = named(&mut g, Player::P0, "Swamp", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 2, 0, 0];
    announce(&mut g, misty);
    choose(&mut g, a);
    pay(&mut g);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 1);
    announce(&mut g, knight);
    choose(&mut g, b);
    pay(&mut g);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 2);
    passes(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    g.move_object(misty, Zone::Graveyard);
    passes(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
}
fn attack_goblin(g: &mut Game, source: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Attack{card,..} if *card==source))
        .unwrap();
    replay(g, a);
    replay(g, Action::FinishAttackers);
}
#[test]
fn goblin_counts_resolution_discard_then_draws_after_owned_private_choice() {
    for n in [0, 2] {
        let mut g = scenario();
        prior(&mut g, Player::P0, n);
        let goblin = named(
            &mut g,
            Player::P0,
            "Green Goblin, Revenant",
            Zone::Battlefield,
        );
        let card = named(&mut g, Player::P0, "Forest", Zone::Hand);
        attack_goblin(&mut g, goblin);
        assert_eq!(g.state.players[0].cards_discarded_this_turn, n as u64);
        passes(&mut g);
        assert!(matches!(g.state.pending, Pending::EffectDiscard { .. }));
        assert!(g.observe(Player::P1).actions.is_empty());
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
        discard(&mut g, card);
        assert_eq!(g.state.players[0].cards_discarded_this_turn, n as u64 + 1);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, n as u64 + 1);
        assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    }
}
#[test]
fn goblin_empty_hand_discards_nothing_but_draws_prior_history_even_without_source() {
    for n in [0, 3] {
        let mut g = scenario();
        prior(&mut g, Player::P0, n);
        let goblin = named(
            &mut g,
            Player::P0,
            "Green Goblin, Revenant",
            Zone::Battlefield,
        );
        attack_goblin(&mut g, goblin);
        g.move_object(goblin, Zone::Graveyard);
        passes(&mut g);
        assert_eq!(g.state.players[0].cards_discarded_this_turn, n as u64);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, n as u64);
        assert!(g.state.resolving.is_none());
    }
}
#[test]
fn cycling_and_mandatory_spell_costs_use_the_same_history_without_counting_draws() {
    let mut g = scenario();
    let cycle = named(&mut g, Player::P0, "Migrating Ketradon", Zone::Hand);
    g.state.players[0].mana = [0, 2, 0, 0, 0, 0];
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..} if *card==cycle))
        .unwrap();
    replay(&mut g, a);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 1);
    passes(&mut g);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 1);
    let spell = named(&mut g, Player::P0, "Thrill of Possibility", Zone::Hand);
    let card = named(&mut g, Player::P0, "Forest", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 2, 0, 0];
    replay(&mut g, Action::AnnounceCast { card: spell });
    choose(&mut g, card);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 1);
    pay(&mut g);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 2);
    passes(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 2);
}
#[test]
fn cleanup_commits_history_as_one_batch_and_turn_start_resets_both_players() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    prior(&mut g, Player::P1, 2);
    let ids: Vec<_> = (0..9)
        .map(|_| named(&mut g, Player::P0, "Forest", Zone::Hand))
        .collect();
    g.state.step = Step::End;
    passes(&mut g);
    replay(&mut g, Action::Discard { card: ids[0] });
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 0);
    replay(&mut g, Action::Discard { card: ids[1] });
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 2);
    assert_eq!(g.state.players[1].cards_discarded_this_turn, 2);
    let saved = g.snapshot();
    g.begin_turn();
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 0);
    assert_eq!(g.state.players[1].cards_discarded_this_turn, 0);
    g.restore(&saved).unwrap();
    assert_eq!(g.state.players[0].cards_discarded_this_turn, 2);
    assert_eq!(g.state.players[1].cards_discarded_this_turn, 2);
}
#[test]
fn exact_wide_history_and_overflow_restore_all_activation_payment_changes() {
    let mut g = scenario();
    g.state.players[0].cards_discarded_this_turn = (1u64 << 60) + 7;
    let v = serde_json::to_value(g.observe(Player::P1)).unwrap();
    assert_eq!(
        v["players"][0]["cards_discarded_this_turn"].as_u64(),
        Some((1u64 << 60) + 7)
    );
    g.state.players[0].cards_discarded_this_turn = u64::MAX;
    let misty = named(
        &mut g,
        Player::P0,
        "Misty Knight, Hero for Hire",
        Zone::Battlefield,
    );
    let card = named(&mut g, Player::P0, "Forest", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 2, 0, 0];
    announce(&mut g, misty);
    choose(&mut g, card);
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(
        g.apply(Player::P0, a),
        Err(Error::CapacityExceeded(crate::CapacityExceeded {
            resource: "cards discarded this turn representation exhausted"
        }))
    );
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
    assert_eq!(g.object_view(card).zone, Zone::Hand);
    assert!(!g.object_view(misty).tapped);
    assert_eq!(g.state.players[0].mana, [0, 0, 0, 2, 0, 0]);
}
