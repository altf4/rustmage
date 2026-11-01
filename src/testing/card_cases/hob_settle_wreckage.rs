//! Focused printed outcomes; shared engine invariants live in core tests.
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
fn hob_settle_optional_zero_fewer_and_fail_to_find_shuffle_only_on_accept() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn hob_settle_exile_ignores_graveyard_replacement_and_land_entries_trigger_normally() {
    {
        let p = Player::P0;
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
