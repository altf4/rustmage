//! Focused printed outcomes; shared engine invariants live in core tests.
//! Prepared inset spells and their associated creatures.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

fn prepared(g: &mut Game, p: Player, name: &str) -> (CardId, ObjectRef) {
    let creature = put(g, p, name, Zone::Hand);
    cast(g, creature, None, &[]);
    settle(g);
    let copy = g
        .state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.permanent == g.object_ref(creature))
        .unwrap()
        .spell_copy;
    (creature, copy)
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("missing prepared choice {:?}", g.state.pending);
}
fn cast_prepared(g: &mut Game, copy: ObjectRef) {
    checked(g, Action::AnnounceExile { card: copy.card });
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, pay);
}
#[test]
fn cultivator_enters_prepared_and_enroot_searches_to_graveyard() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let (creature, copy) = prepared(&mut g, p, "Carnivorous Cultivator // Enroot");
        assert!(g.keywords(creature).contains(Keywords::DEATHTOUCH));
        let land = put(&mut g, p, "Island", Zone::Library);
        let nonland = put(&mut g, p, "Shock", Zone::Library);
        cast_prepared(&mut g, copy);
        until(&mut g, |g| g.observe(p).search_choice.is_some());
        assert!(g.observe(p.other()).search_choice.is_none());
        reject(&mut g, Action::ChooseSearch { card: nonland });
        checked(&mut g, Action::ChooseSearch { card: land });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        assert!(!g.is_prepared(g.object_ref(creature)));
        assert!(!g.on_battlefield(copy.card));
    }
}
#[test]
fn battlemage_prepared_spell_discards_at_most_one_and_draws_only_if_discarded() {
    {
        let p = Player::P0;
        for discard in [false, true] {
            let mut g = setup(p);
            let (creature, copy) = prepared(&mut g, p, "Pompous Battlemage // Improvised Act");
            let hand = put(&mut g, p, "Island", Zone::Hand);
            let drawn = put(&mut g, p, "Mountain", Zone::Library);
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            cast_prepared(&mut g, copy);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            assert!(g.observe(p.other()).effect_choice.is_none());
            if discard {
                checked(&mut g, Action::Discard { card: hand });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn - before,
                u64::from(discard)
            );
            assert_eq!(
                g.object_view(hand).zone,
                if discard { Zone::Graveyard } else { Zone::Hand }
            );
            assert_eq!(
                g.object_view(drawn).zone,
                if discard { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(
                g.stats(creature).power,
                2,
                "casting prepared noncreature triggers prowess"
            );
            assert!(!g.is_prepared(g.object_ref(creature)));
        }
    }
}
