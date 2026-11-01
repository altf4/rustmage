//! Focused printed outcomes; shared engine invariants live in core tests.
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
fn desolation_mana_cannot_pay_non_dragon_spell_or_ability_but_can_cast_dragon() {
    {
        let p = Player::P0;
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
