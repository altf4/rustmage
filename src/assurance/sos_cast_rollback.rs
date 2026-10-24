//! An opponent keeps its entry choices even if they make a proposal fail.
use super::*;
use big_helpers::{cast as paid, checked, settle as finish};
fn until_choice(g: &mut Game) {
    for _ in 0..40 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing entry choice");
}
fn clause(g: &mut Game, clause: CastCostClause) {
    let n = g
        .observe(g.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == clause && !r.paid)
        .unwrap();
    checked(g, Action::ChooseCastCost { clause: n as u16 });
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, a);
}
fn setup(p: Player, foreign: bool) -> (Game, CardId, CardId, CardId, CardId) {
    let mut g = scenario(p);
    let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let owner = if foreign { p.other() } else { p };
    let aura = put(&mut g, owner, "Unable to Scream", Zone::Battlefield);
    let r = g.object_ref(enemy);
    g.state.objects[aura as usize].attached_to = Some(r);
    if !foreign {
        g.record_control(g.object_ref(aura), p.other(), None);
    }
    let prison = put(&mut g, p, "Banishing Light", Zone::Hand);
    paid(&mut g, prison, None, &[]);
    until_choice(&mut g);
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    finish(&mut g);
    assert_eq!(g.object_view(aura).zone, Zone::Exile);
    (g, silver, enemy, aura, prison)
}
fn propose(g: &mut Game, p: Player, spell: CardId, prison: CardId, enemy: CardId) {
    checked(g, Action::AnnounceCast { card: spell });
    let sacrifice = Some(g.object_ref(prison));
    checked(g, Action::ChooseBargain { sacrifice });
    checked(g, Action::ChooseCasualty { pay: true });
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    assert_eq!(g.actor(), Some(p));
}
#[test]
fn sos_cast_rollback_foreign_aura_can_strand_costs_and_returns_entire_proposal() {
    for p in [Player::P0, Player::P1] {
        for harmful in [false, true] {
            let (mut g, silver, enemy, aura, prison) = setup(p, true);
            let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
            let before = g.snapshot();
            let own_view = serde_json::to_value(g.observe(p)).unwrap();
            let other_view = serde_json::to_value(g.observe(p.other())).unwrap();
            propose(&mut g, p, spell, prison, enemy);
            pay(&mut g);
            clause(&mut g, CastCostClause::Additional);
            assert_eq!(g.actor(), Some(p.other()));
            let object = g.object_ref(if harmful { silver } else { enemy });
            assert!(
                g.legal_actions()
                    .contains(&Action::ChooseAuraAttachment { object })
            );
            checked(&mut g, Action::ChooseAuraAttachment { object });
            if harmful {
                assert_eq!(g.snapshot(), before);
                assert_eq!(g.actor(), Some(p));
                assert_eq!(serde_json::to_value(g.observe(p)).unwrap(), own_view);
                assert_eq!(
                    serde_json::to_value(g.observe(p.other())).unwrap(),
                    other_view
                );
                assert!(g.state.cast_rollback.is_none());
            } else {
                clause(&mut g, CastCostClause::Casualty(0));
                let object = g.object_ref(silver);
                checked(&mut g, Action::PayCasualty { object });
                finish(&mut g);
                assert_eq!(g.object_view(prison).zone, Zone::Graveyard);
                assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                assert!(g.state.cast_rollback.is_none());
            }
        }
    }
}
#[test]
fn sos_cast_rollback_keeps_completed_basic_mana_ability_but_cancels_spell_payments() {
    for p in [Player::P0, Player::P1] {
        let (mut g, silver, enemy, aura, prison) = setup(p, true);
        let forest = put(&mut g, p, "Forest", Zone::Battlefield);
        let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        propose(&mut g, p, spell, prison, enemy);
        checked(&mut g, Action::TapMana { card: forest });
        pay(&mut g);
        clause(&mut g, CastCostClause::Additional);
        let object = g.object_ref(silver);
        checked(&mut g, Action::ChooseAuraAttachment { object });
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.object_view(prison).zone, Zone::Battlefield);
        assert_eq!(g.object_view(aura).zone, Zone::Exile);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        assert!(g.object_view(forest).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 1, 0]);
        assert!(g.state.announcing.is_none());
        assert!(g.state.cast_rollback.is_none());
    }
}
#[test]
fn sos_cast_rollback_own_entry_choices_still_must_complete_declared_costs() {
    for p in [Player::P0, Player::P1] {
        let (mut g, silver, enemy, _, prison) = setup(p, false);
        let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
        propose(&mut g, p, spell, prison, enemy);
        pay(&mut g);
        clause(&mut g, CastCostClause::Additional);
        assert_eq!(g.actor(), Some(p));
        let object = g.object_ref(silver);
        reject(&mut g, Action::ChooseAuraAttachment { object });
        let object = g.object_ref(enemy);
        checked(&mut g, Action::ChooseAuraAttachment { object });
        clause(&mut g, CastCostClause::Casualty(0));
        let object = g.object_ref(silver);
        checked(&mut g, Action::PayCasualty { object });
        finish(&mut g);
    }
}
#[test]
fn sos_cast_rollback_irreversible_self_shuffle_retains_owner_library_and_cleans_rows() {
    for p in [Player::P0, Player::P1] {
        for name in ["Darksteel Colossus", "Progenitus"] {
            for owner in [p, p.other()] {
                let (mut g, _, _, _, _) = setup(p, true);
                let card = put(&mut g, owner, name, Zone::Battlefield);
                let ref_before = g.object_ref(card);
                g.set_counters(card, Counters::of(CounterKind::Page, 3));
                g.state.objects[card as usize].growth.power = 7;
                let victim = put(&mut g, p, "Sneering Shadewriter", Zone::Battlefield);
                let spell = put(&mut g, p, "Fix What's Broken", Zone::Hand);
                g.prepare_cast_rollback(p, &Action::AnnounceCast { card: spell });
                assert!(g.state.cast_rollback.is_some());
                g.sacrifice_batch(&[(p, victim)]);
                g.move_object(card, Zone::Graveyard);
                assert_eq!(g.object_view(card).zone, Zone::Library);
                let actual = g.object_view(card);
                let order = g.state.players[owner.index()].library.clone();
                let rng = g.state.rng;
                g.finish_cast_rollback(false);
                assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
                assert_eq!(g.object_view(card).zone, Zone::Library);
                assert_ne!(g.object_ref(card), ref_before);
                assert_eq!(g.object_ref(card), actual.reference);
                assert_eq!(g.state.players[owner.index()].library, order);
                assert_eq!(g.state.rng, rng);
                assert_eq!(g.counter_count(card, CounterKind::Page), 0);
                assert_eq!(g.object_view(card).growth, actual.growth);
                assert!(!g.state.static_sources.contains(&card));
                assert!(g.state.pending_triggers.is_empty());
            }
        }
    }
}
