//! Simultaneous non-Dragon damage and Dragon-only multi-color mana.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Desolation decision budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn spell(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Desolation of Smaug", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 2];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn mana(g: &mut Game, amount: Mana) {
    until(g, |g| matches!(g.state.pending, Pending::EffectMana { .. }));
    act(g, Action::ChooseMana { production: amount });
    finish(g);
}
#[test]
fn desolation_paid_deals_three_to_all_non_dragons_then_restricted_combination() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let token = put(&mut g, p, "@token/hob-dragon", Zone::Battlefield);
        let id = spell(&mut g, p);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectMana { .. })
        });
        assert_eq!(g.state.objects[a as usize].damage, 3);
        assert_eq!(g.state.objects[b as usize].damage, 3);
        assert_eq!(g.state.objects[dragon as usize].damage, 0);
        assert_eq!(g.state.objects[token as usize].damage, 0);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        let actions = g
            .legal_actions()
            .into_iter()
            .filter(|a| matches!(a, Action::ChooseMana { .. }))
            .count();
        assert_eq!(actions, 70);
        reject(
            &mut g,
            Action::ChooseMana {
                production: [0, 0, 0, 4, 1, 0],
            },
        );
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut a1 = g.clone();
        let mut a2 = g.clone();
        drop(g);
        act(
            &mut a1,
            Action::ChooseMana {
                production: [0, 0, 0, 2, 2, 0],
            },
        );
        finish(&mut a1);
        act(
            &mut a2,
            Action::ChooseMana {
                production: [0, 1, 1, 1, 1, 0],
            },
        );
        finish(&mut a2);
        assert_eq!(a1.object_view(id).zone, Zone::Graveyard);
        assert_eq!(a1.object_view(a).zone, Zone::Graveyard);
        assert_eq!(a1.object_view(b).zone, Zone::Graveyard);
        assert_eq!(a1.object_view(dragon).zone, Zone::Battlefield);
        assert_eq!(a1.object_view(token).zone, Zone::Battlefield);
        assert_eq!(
            a1.mana_lots(p)[0].quality.restriction,
            ManaRestrictionView::SpellSubtype("Dragon")
        );
        assert_eq!(a1.mana_lots(p)[0].amounts, [0, 0, 0, 2, 2, 0]);
        a2.restore(&saved).unwrap();
        act(
            &mut a2,
            Action::ChooseMana {
                production: [0, 0, 0, 2, 2, 0],
            },
        );
        finish(&mut a2);
        assert_eq!(a1.snapshot(), a2.snapshot());
    }
}
#[test]
fn desolation_mana_cannot_pay_non_dragon_spell_or_ability_but_can_cast_dragon() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let dragon = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        spell(&mut g, p);
        mana(&mut g, [0, 0, 0, 2, 2, 0]);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        reject(
            &mut g,
            Action::Cast {
                card: bear,
                target: None,
                payment: [0, 0, 0, 0, 2, 0],
            },
        );
        reject(
            &mut g,
            Action::Activate {
                card: pick,
                ability: 0,
                target: Some(Target::Card(dragon)),
                payment: [0, 0, 0, 0, 1, 0],
                counter_payment: Counters::default(),
            },
        );
        g.add_mana(p, [0, 0, 0, 2, 0, 0]);
        let cast_dragon = put(&mut g, p, "Shivan Dragon", Zone::Hand);
        cast(&mut g, cast_dragon, None);
        finish(&mut g);
        assert_eq!(g.object_view(cast_dragon).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn desolation_shield_and_doubling_pause_before_mana_with_owned_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.add_counter(bear, CounterKind::Shield, 1);
        put(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
        spell(&mut g, p);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        assert_eq!(g.actor(), Some(p.other()));
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut shield = g.clone();
        let mut doubled = g.clone();
        drop(g);
        choose(&mut shield, |a| {
            matches!(a, Action::ChooseDamagePrevention { .. })
        });
        mana(&mut shield, [0, 0, 0, 4, 0, 0]);
        choose(&mut doubled, |a| {
            matches!(a, Action::ChooseReplacement { .. })
        });
        mana(&mut doubled, [0, 0, 0, 4, 0, 0]);
        for child in [&shield, &doubled] {
            assert_eq!(child.object_view(bear).zone, Zone::Battlefield);
            assert_eq!(child.state.objects[bear as usize].damage, 0);
            assert_eq!(child.counter_count(bear, CounterKind::Shield), 0);
            assert_eq!(child.state.players[p.index()].mana, [0, 0, 0, 4, 0, 0]);
        }
        doubled.restore(&saved).unwrap();
        choose(&mut doubled, |a| {
            matches!(a, Action::ChooseDamagePrevention { .. })
        });
        mana(&mut doubled, [0, 0, 0, 4, 0, 0]);
        assert_eq!(shield.snapshot(), doubled.snapshot());
    }
}
#[test]
fn desolation_countered_produces_neither_damage_nor_mana_and_mana_expires_next_step() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let id = spell(&mut g, p);
    if g.actor() == Some(p) {
        act(&mut g, Action::Pass);
    }
    let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
    g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 1];
    cast(&mut g, negate, Some(Target::Spell(id)));
    finish(&mut g);
    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    assert_eq!(g.state.objects[bear as usize].damage, 0);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    let mut g = scenario(p);
    spell(&mut g, p);
    mana(&mut g, [1, 1, 1, 1, 0, 0]);
    until(&mut g, |g| g.state.step == Step::BeginCombat);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert!(g.mana_lots(p).is_empty());
}
