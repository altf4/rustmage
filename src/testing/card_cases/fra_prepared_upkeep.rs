//! Focused printed outcomes; shared engine invariants live in core tests.
//! Additional prepared-creature printed obligations and incarnation checks.
use super::big_helpers::{checked, settle, setup};
use super::*;

fn candidate(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = put(g, p, "Forest", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    id
}

fn finish(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .expect("funded announcement");
        checked(g, action);
    }
    panic!("announcement did not finish");
}

fn cast_front(g: &mut Game, source: CardId) {
    checked(g, Action::AnnounceCast { card: source });
    finish(g);
    settle(g);
}

fn upkeep(g: &mut Game) {
    g.begin_step(Step::Upkeep);
    g.give_priority();
    g.prepare_triggers();
}

fn copy(g: &Game, source: CardId) -> ObjectRef {
    g.state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|row| row.permanent == g.object_ref(source))
        .unwrap()
        .spell_copy
}

#[test]
fn woodwork_upkeep_prepares_and_casts_its_own_soul_tether() {
    {
        let p = Player::P0;
        for green in [false, true] {
            let mut g = setup(p);
            let source = candidate(&mut g, p, "Woodwork Prodigy // Soul Tether");
            cast_front(&mut g, source);
            assert!(!g.is_prepared(g.object_ref(source)));
            upkeep(&mut g);
            settle(&mut g);
            assert!(g.is_prepared(g.object_ref(source)));
            let spell = copy(&g, source);
            // Resume sorcery timing after the actual upkeep trigger resolves.
            g.state.step = Step::Main1;
            g.state.players[p.index()].mana = if green {
                [0, 0, 0, 0, 1, 2]
            } else {
                [0, 0, 0, 1, 0, 2]
            };
            checked(&mut g, Action::AnnounceExile { card: spell.card });
            finish(&mut g);
            assert!(!g.is_prepared(g.object_ref(source)));
            settle(&mut g);
            assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                && o.controller == p
                && o.card == Card::from_name("@token/heartwood").unwrap()));
        }
    }
}

#[test]
fn woodwork_queued_upkeep_cannot_prepare_new_incarnation_after_blink() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let source = candidate(&mut g, p, "Woodwork Prodigy // Soul Tether");
        cast_front(&mut g, source);
        upkeep(&mut g);
        assert!(!g.state.stack.is_empty());
        let old = g.object_ref(source);
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        let new = g.object_ref(source);
        assert_ne!(old, new);
        assert!(!g.is_prepared(new));
        settle(&mut g);
        assert!(!g.is_prepared(new));
        upkeep(&mut g);
        settle(&mut g);
        assert!(g.is_prepared(new));
    }
}

#[test]
fn emergency_phytomedic_front_accepts_either_hybrid_color() {
    {
        let p = Player::P0;
        for green in [false, true] {
            let mut g = setup(p);
            let source = candidate(&mut g, p, "Emergency Phytomedic // Seed Suture");
            g.state.players[p.index()].mana = if green {
                [0, 0, 0, 0, 1, 0]
            } else {
                [1, 0, 0, 0, 0, 0]
            };
            cast_front(&mut g, source);
            assert!(g.on_battlefield(source));
            assert!(g.is_prepared(g.object_ref(source)));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn heartwood_crafter_tap_mana_respects_summoning_sickness() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let source = candidate(&mut g, p, "Heartwood Crafter // Soul Tether");
        cast_front(&mut g, source);
        assert!(g.state.objects[source as usize].summoning_sick);
        assert!(!g.legal_actions().iter().any(|a| matches!(a,
            Action::ActivateMana { card, .. } if *card == source)));
        g.state.objects[source as usize].summoning_sick = false;
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a,
            Action::ActivateMana { card, production: [0, 0, 0, 0, 0, 1], .. }
                if *card == source)
            })
            .expect("Crafter's colorless mana");
        checked(&mut g, action);
        assert!(g.object_view(source).tapped);
    }
}
