//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn until_target(g: &mut Game) {
    for _ in 0..32 {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseTarget { .. }))
        {
            return;
        }
        choose(g, |a| matches!(a, Action::PutTrigger { .. } | Action::Pass));
    }
    panic!("target announcement budget");
}
fn target(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn bard(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Bard the Bowman", Zone::Hand);
    g.state.players[p.index()].mana = [1, 1, 0, 0, 0, 1];
    cast(g, id, None);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, id), (1, 3));

    id
}
fn second_draw(g: &mut Game, p: Player) {
    // A paid instant draws two individual cards during the opponent's turn.
    g.state.active = p.other();
    g.state.priority = p;
    let instant = put(g, p, "Quick Study", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    let before = g.hand_count(p);
    cast(g, instant, None);
    until_target(g);
    assert_eq!(g.hand_count(p), before + 1);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
#[test]
fn hob_inherited_bard_paid_second_draw_any_creature_counter_lifelink_and_cleanup() {
    {
        let p = Player::P0;
        for owner in [p, p.other()] {
            let mut g = scenario(p);
            let source = bard(&mut g, p);
            let recipient = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            // Opponent's first two draws never trigger our Bard.
            g.draw(p.other());
            g.draw(p.other());
            g.prepare_triggers();
            assert!(g.state.pending_triggers.is_empty());
            assert!(g.state.stack.is_empty());
            second_draw(&mut g, p);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            target(&mut g, Target::Card(recipient));
            // The ability is independent of its departed source.
            g.move_object(source, Zone::Graveyard);
            g.give_priority();
            settle(&mut g);
            assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 1);
            assert_eq!(stats(&g, recipient), (3, 3));
            assert!(g.keywords(recipient).contains(Keywords::LIFELINK));
            let before = g.state.players[owner.index()].life;
            let damage_source = g.damage_source(recipient);
            g.apply_damage_batch(&[DamageEvent {
                source: damage_source,
                target: Target::Player(owner.other()),
                amount: 3,
                combat: false,
                preventable: true,
            }]);
            assert_eq!(g.state.players[owner.index()].life, before + 3);
            g.finish_cleanup();
            assert_eq!(stats(&g, recipient), (3, 3));
            assert!(!g.keywords(recipient).contains(Keywords::LIFELINK));
        }
    }
}

fn smaug(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Smaug the Magnificent", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 2];
    cast(g, id, None);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, id), (4, 3));
    assert!(
        g.keywords(id)
            .contains(Keywords::FLYING.union(Keywords::HASTE))
    );
    assert!(g.object_view(id).summoning_sick);
    id
}

fn treasures(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype("Treasure"))
                .then_some(i as CardId)
        })
        .collect()
}

#[test]
fn hob_inherited_smaug_only_own_upkeep_creates_one_usable_treasure() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = smaug(&mut g, p);
        g.state.active = p.other();
        g.begin_step(Step::Upkeep);
        g.give_priority();
        g.prepare_triggers();
        settle(&mut g);
        assert!(treasures(&g, p).is_empty());
        g.state.active = p;
        g.begin_step(Step::Upkeep);
        g.give_priority();
        g.prepare_triggers();
        g.move_object(source, Zone::Graveyard);
        g.give_priority();
        settle(&mut g);
        let tokens = treasures(&g, p);
        assert_eq!(tokens.len(), 1);
        let id = tokens[0];
        assert!(g.state.objects[id as usize].card.is_token());
        assert_eq!(g.object_view(id).kinds, 16);
        assert!(!g.object_view(id).tapped);
        g.state.priority = p;
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,production,..} if *card == id && *production == [0,0,0,1,0,0]),
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
        assert_ne!(g.object_view(id).zone, Zone::Battlefield);
    }
}
