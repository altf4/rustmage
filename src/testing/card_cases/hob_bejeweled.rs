//! Focused printed outcomes; shared engine invariants live in core tests.
//! Full Oracle outcomes for Bejeweled Warg's combat-only modal trigger.
use super::*;

fn until_modes(g: &mut Game, warg: CardId, blocker: Option<CardId>) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: warg });
    act(g, Action::FinishAttackers);
    for _ in 0..8 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(g, Action::Pass);
    }
    assert_eq!(g.state.pending, Pending::Blockers);
    if let Some(blocker) = blocker {
        act(
            g,
            Action::Block {
                blocker,
                attacker: warg,
            },
        );
    }
    act(g, Action::FinishBlockers);
    for _ in 0..20 {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseMode { .. }))
        {
            return;
        }
        if let Some(range) = g.damage_assignment_range() {
            act(
                g,
                Action::AssignDamage {
                    target: range.target,
                    amount: range.minimum,
                },
            );
        } else {
            act(g, Action::Pass);
        }
    }
    panic!("combat modal trigger missing");
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
fn hob_bejeweled_paid_body_combat_counter_mode_and_live_target_legality() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let warg = put(&mut g, p, "Bejeweled Warg", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, warg, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(stats(&g, warg), (3, 2));
        assert!(g.keywords(warg).contains(Keywords::TRAMPLE));
        g.state.objects[warg as usize].summoning_sick = false;
        let enemy = put(&mut g, p.other(), "Ravening Warg", Zone::Battlefield);
        let wrong = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        until_modes(&mut g, warg, None);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        act(&mut g, Action::ChooseMode { mode: 0 });
        for id in [enemy, wrong] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(warg),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        settle(&mut g);
        assert_eq!(g.counter_count(warg, CounterKind::PlusOne), 1);
        assert_eq!(stats(&g, warg), (4, 3));
        assert!(treasures(&g, p).is_empty());
    }
}

#[test]
fn hob_bejeweled_trample_dying_source_still_creates_usable_treasure() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let warg = put(&mut g, p, "Bejeweled Warg", Zone::Battlefield);
        let blocker = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        until_modes(&mut g, warg, Some(blocker));
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert_eq!(g.state.objects[warg as usize].zone, Zone::Graveyard);
        assert!(!g.legal_actions().contains(&Action::ChooseMode { mode: 0 }));
        reject(&mut g, Action::ChooseMode { mode: 0 });
        act(&mut g, Action::ChooseMode { mode: 1 });
        settle(&mut g);
        let tokens = treasures(&g, p);
        assert_eq!(tokens.len(), 1);
        let token = tokens[0];
        assert!(g.state.objects[token as usize].card.is_token());
        assert!(!g.state.objects[token as usize].tapped);
        assert_eq!(g.object_view(token).kinds, 16);
        g.state.priority = p;
        g.state.players[p.index()].mana = [0; 6];
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana { card,production,.. } if *card==token && *production==[0,1,0,0,0,0]),
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
        assert_ne!(g.state.objects[token as usize].zone, Zone::Battlefield);
    }
}

#[test]
fn hob_bejeweled_ignores_noncombat_nonplayer_and_other_sources_damage() {
    let p = Player::P0;
    let mut g = scenario(p);
    let warg = put(&mut g, p, "Bejeweled Warg", Zone::Battlefield);
    let other = put(&mut g, p, "Ravening Warg", Zone::Battlefield);
    let creature = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
    // Inject distinct actual damage events to isolate trigger qualification. The
    // positive tests above use ordinary paid cast and complete combat actions.
    for (source, target, combat) in [
        (warg, Target::Player(p.other()), false),
        (warg, Target::Card(creature), true),
        (other, Target::Player(p.other()), true),
    ] {
        let source = g.damage_source(source);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target,
            amount: 1,
            combat,
            preventable: true,
        }]);
        g.prepare_triggers();
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.state.stack.is_empty());
        assert!(!g.legal_actions().contains(&Action::ChooseMode { mode: 0 }));
    }
}
