//! A player-only combat answer, public exiles and private optional land search.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers
            )
        });
    }
    panic!(
        "Settle decision budget {:?} {:?}",
        g.state.pending, g.state.step
    );
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn attack(g: &mut Game, ids: &[CardId]) {
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    until(g, |g| g.state.pending == Pending::Attackers);
    for &card in ids {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
}
fn spell(g: &mut Game, p: Player, target: Player) -> CardId {
    g.state.priority = p;
    let s = put(g, p, "Settle the Wreckage", Zone::Hand);
    g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 2];
    cast(g, s, Some(Target::Player(target)));
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    s
}
fn offer(g: &mut Game) {
    until(g, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
}
fn accept(g: &mut Game) {
    offer(g);
    act(g, Action::ChooseOptional { accept: true });
    assert!(matches!(g.state.pending, Pending::EffectSearch { .. }));
}
fn choose_land(g: &mut Game, id: CardId) {
    act(g, Action::ChooseSearch { card: id });
}
#[test]
fn hob_settle_paid_instant_exiles_only_actual_target_player_attackers_and_private_search() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let idle = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[a as usize].gained_keywords =
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE);
        let nonbasic = put(&mut g, p, "Raucous Theater", Zone::Library);
        let secret = put(&mut g, p, "Shock", Zone::Library);
        let plains = put(&mut g, p, "Plains", Zone::Library);
        let forest = put(&mut g, p, "Forest", Zone::Library);
        let extra = put(&mut g, p, "Island", Zone::Library);
        attack(&mut g, &[a, b]);
        spell(&mut g, p.other(), p);
        accept(&mut g);
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.object_view(a).zone, Zone::Exile);
        assert_eq!(g.object_view(b).zone, Zone::Exile);
        assert_eq!(g.object_view(idle).zone, Zone::Battlefield);
        assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
        let view = g.observe(p).search_choice.unwrap();
        assert_eq!((view.minimum, view.maximum), (0, 2));
        assert!(g.observe(p.other()).search_choice.is_none());
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == secret));
        reject(&mut g, Action::ChooseSearch { card: nonbasic });
        reject(&mut g, Action::ChooseSearch { card: secret });
        contracts::branch_contract(g.clone());
        choose_land(&mut g, plains);
        reject(&mut g, Action::ChooseSearch { card: plains });
        choose_land(&mut g, forest);
        reject(&mut g, Action::ChooseSearch { card: extra });
        assert_eq!(g.object_view(plains).zone, Zone::Library);
        assert!(g.observe(p.other()).search_choice.is_none());
        let rng = g.state.rng;
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        assert_ne!(g.state.rng, rng);
        for c in [plains, forest] {
            assert_eq!(g.object_view(c).zone, Zone::Battlefield);
            assert!(g.state.objects[c as usize].tapped);
            assert_eq!(g.state.objects[c as usize].controller, p);
        }
        assert_eq!(g.object_view(extra).zone, Zone::Library);
    }
}
#[test]
fn hob_settle_optional_zero_fewer_and_fail_to_find_shuffle_only_on_accept() {
    for p in [Player::P0, Player::P1] {
        for (count, accept_search, find) in [
            (0, false, 0),
            (0, true, 0),
            (2, false, 0),
            (2, true, 0),
            (2, true, 1),
        ] {
            let mut g = scenario(p);
            let attackers: Vec<_> = (0..count)
                .map(|_| put(&mut g, p, "Bear Cub", Zone::Battlefield))
                .collect();
            if count > 0 {
                attack(&mut g, &attackers);
            }
            let l = put(&mut g, p, "Plains", Zone::Library);
            spell(&mut g, p.other(), p);
            offer(&mut g);
            let rng = g.state.rng;
            let library = g.state.players[p.index()].library.to_vec();
            act(
                &mut g,
                Action::ChooseOptional {
                    accept: accept_search,
                },
            );
            if accept_search {
                assert_eq!(g.observe(p).search_choice.unwrap().maximum, count as u64);
                if find == 1 {
                    choose_land(&mut g, l);
                }
                act(&mut g, Action::FinishSearch);
            }
            finish(&mut g);
            assert_eq!(g.state.rng == rng, !accept_search);
            if !accept_search {
                assert_eq!(g.state.players[p.index()].library.to_vec(), library);
            }
            assert_eq!(
                g.object_view(l).zone,
                if find == 1 {
                    Zone::Battlefield
                } else {
                    Zone::Library
                }
            );
        }
    }
}
#[test]
fn hob_settle_self_target_tokens_and_animated_noncreature_cards_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        g.change_object_form(
            g.object_ref(land),
            crate::cards::AnimationSpec {
                artifact: false,
                creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        g.create_tokens(p, Card::from_name("@token/hob-bear").unwrap(), 1, false);
        let token = (g.state.objects.len() - 1) as CardId;
        g.state.objects[token as usize].summoning_sick = false;
        attack(&mut g, &[land, token]);
        spell(&mut g, p, p);
        accept(&mut g);
        assert_eq!(g.observe(p).search_choice.unwrap().maximum, 2);
        assert_eq!(g.object_view(land).zone, Zone::Exile);
        assert_ne!(g.object_view(token).zone, Zone::Battlefield);
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
    }
}
#[test]
fn hob_settle_resolution_uses_current_attackers_and_includes_put_attacking_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let c = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, &[a, b, c]);
        spell(&mut g, p.other(), p);
        g.move_object(a, Zone::Hand);
        g.move_object(a, Zone::Battlefield);
        g.change_controller(g.object_ref(b), p.other());
        let added = g.create_token_batch_capture(
            p,
            Card::from_name("@token/hob-bear").unwrap(),
            1,
            true,
            Some(&[CombatDefender::Player(p.other())]),
            true,
        )[0]
        .card;
        offer(&mut g);
        assert_eq!(g.state.resolving.as_ref().unwrap().moved_count, Some(2));
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        assert_eq!(g.object_view(b).zone, Zone::Battlefield);
        assert_eq!(g.object_view(c).zone, Zone::Exile);
        assert_ne!(g.object_view(added).zone, Zone::Battlefield);
        act(&mut g, Action::ChooseOptional { accept: false });
        finish(&mut g);
    }
}
#[test]
fn hob_settle_player_hexproof_and_unaffordable_cast_reject_or_fizzle_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = put(&mut g, p, "Settle the Wreckage", Zone::Hand);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Cast{card,target:Some(Target::Player(t)),..}if *card==s&&*t==p.other())).unwrap();
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, action.clone());
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 2];
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        reject(&mut g, action);
        g.state.players[p.other().index()].hexproof_until_end_turn = false;
        g.state.active = p.other();
        let a = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attack(&mut g, &[a]);
        spell(&mut g, p, p.other());
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        finish(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
    }
}
#[test]
fn hob_settle_exile_ignores_graveyard_replacement_and_land_entries_trigger_normally() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let hospitality = put(&mut g, p, "Beorn's Hospitality", Zone::Battlefield);
        let recipient = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p, "Plains", Zone::Library);
        attack(&mut g, &[a]);
        spell(&mut g, p.other(), p);
        accept(&mut g);
        assert_eq!(g.observe(p).search_choice.unwrap().maximum, 1);
        choose_land(&mut g, land);
        act(&mut g, Action::FinishSearch);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(recipient),
            })
        });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(recipient),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        finish(&mut g);
        assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 1);
        assert_eq!(g.object_view(hospitality).zone, Zone::Battlefield);
        assert!(g.state.objects[land as usize].tapped);
    }
}
#[test]
fn hob_settle_scaled_search_maximum_preserves_full_u64_domain() {
    for value in [65535, 65536, u64::MAX] {
        let p = Player::P0;
        let mut g = scenario(p);
        spell(&mut g, p, p.other());
        offer(&mut g);
        g.state.resolving.as_mut().unwrap().moved_count = Some(value);
        act(&mut g, Action::ChooseOptional { accept: true });
        let view = g.observe(p.other()).search_choice.unwrap();
        assert_eq!(view.maximum, value);
        assert_eq!(
            serde_json::to_value(view).unwrap()["maximum"],
            serde_json::json!(value)
        );
        contracts::branch_contract(g.clone());
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
    }
}
#[test]
fn hob_settle_synthetic_redirected_moves_count_but_suppressed_moves_do_not() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let mut events = g.move_objects_raw(&[a, b], Zone::Exile);
    assert_eq!(attacking_exile::moved_creature_count(&events), 2);
    // Explicit direct event-boundary contract: no admitted effect redirects or
    // suppresses a BF-to-exile event. A redirected completed event still counts.
    events[0].destination = Zone::Hand;
    assert_eq!(attacking_exile::moved_creature_count(&events), 2);
    events[1].moved = false;
    assert_eq!(attacking_exile::moved_creature_count(&events), 1);
}
