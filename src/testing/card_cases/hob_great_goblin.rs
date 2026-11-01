//! Focused printed outcomes; shared engine invariants live in core tests.
//! The Great Goblin: actual placement and frozen-death observations.
use super::*;
use crate::engine::replacements::ResumeMode;

fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn finish(g: &mut Game) {
    for _ in 0..96 {
        if quiet(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseTarget {
                        target: Target::Player(_)
                    }
                    | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("Great Goblin decision budget {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "The Great Goblin", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 1];
    cast(g, id, None);
    finish(g);
    assert_eq!(stats(g, id), (3, 2));
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn types(g: &mut Game, id: CardId, names: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: names,
            additional: false,
            timestamp,
        });
}
#[test]
fn great_goblin_mixed_placement_per_recipient_both_seats_and_placer() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = paid(&mut g, p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        types(&mut g, a, &["Goblin", "Orc", "Army"]);
        types(&mut g, b, &["Army"]);
        types(&mut g, enemy, &["Goblin"]);
        let ar = g.object_ref(a);
        let br = g.object_ref(b);
        g.effect_counters(
            vec![
                (
                    ar,
                    Counters::of(CounterKind::PlusOne, 2).with(CounterKind::Stun, 1),
                ),
                (br, Counters::of(CounterKind::Stun, 1)),
            ],
            ResumeMode::Pending,
        );
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        g.effect_counter_by(a, CounterKind::Stun, 0, p, ResumeMode::Pending);
        g.effect_counter_by(a, CounterKind::Stun, 1, p.other(), ResumeMode::Pending);
        g.effect_counter_by(enemy, CounterKind::Stun, 1, p, ResumeMode::Pending);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        g.effect_counter_by(source, CounterKind::Stun, 1, p, ResumeMode::Pending);
        g.move_object(source, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}
#[test]
fn great_goblin_other_deaths_use_lki_and_public_exact_exile() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = paid(&mut g, p);
        let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        types(&mut g, victim, &["Goblin", "Orc"]);
        let top = put(&mut g, p, "Shock", Zone::Library);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == top));
        g.move_object(victim, Zone::Graveyard);
        g.move_object(source, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Exile);
        assert!(g.observe(p.other()).objects.iter().any(|o| o.id == top));
        assert_eq!(g.state.exile_permissions.len(), 1);
    }
}

#[test]
fn great_goblin_actual_opponent_amass_uses_placing_player_and_entry() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "The Great Goblin", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let azog = put(&mut g, p.other(), "Azog, Moria's Ruin", Zone::Hand);
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [0, 0, 1, 0, 0, 2];
        cast(&mut g, azog, None);
        for _ in 0..64 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert!(
            g.state
                .objects
                .iter()
                .enumerate()
                .any(|(id, o)| o.zone == Zone::Battlefield
                    && o.controller == p
                    && g.has_subtype(id as CardId, "Army")
                    && g.counter_count(id as CardId, CounterKind::PlusOne) == 2)
        );
    }
}
