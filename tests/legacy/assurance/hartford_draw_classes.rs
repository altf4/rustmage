//! Class-level discard rewards and captured-X/search-driven drawing.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

fn reach(g: &mut Game, wanted: impl Fn(&Game) -> bool) {
    for _ in 0..40 {
        if wanted(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("missing decision {:?}", g.state.pending);
}
fn activate(g: &mut Game, card: CardId, ability: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if g.state.announcing.is_some() {
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(g, pay);
    }
}

#[test]
fn cool_class_loots_on_attack_then_unlocks_discard_damage_and_random_discard_tutor() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let class = put(&mut g, p, "Cool but Rude", Zone::Hand);
        cast(&mut g, class, None, &[]);
        settle(&mut g);
        let card = put(&mut g, p, "Mountain", Zone::Hand);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[attacker as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: attacker });
        checked(&mut g, Action::FinishAttackers);
        reach(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let life = g.state.players[p.other().index()].life;
        checked(&mut g, Action::Discard { card });
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
        assert_eq!(g.state.players[p.other().index()].life, life);
        g.state.step = Step::Main2;
        activate(&mut g, class, 0);
        settle(&mut g);
        assert_eq!(g.object_view(class).class_level, Some(2));
        let cards = (0..2)
            .map(|_| put(&mut g, p, "Mountain", Zone::Hand))
            .collect::<Vec<_>>();
        g.discard_batch(&cards, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 4);
        let hand = g.hand_count(p);
        let library = g.state.players[p.index()].library.len();
        activate(&mut g, class, 1);
        reach(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        reject(&mut g, Action::FinishSearch);
        let found = *g.state.players[p.index()].library.last().unwrap();
        checked(&mut g, Action::ChooseSearch { card: found });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(class).class_level, Some(3));
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
        assert_eq!(g.hand_count(p), hand);
        assert_eq!(g.state.players[p.other().index()].life, life - 6);
    }
}

#[test]
fn cool_level_change_does_not_rebind_a_departed_and_returned_class() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let class = put(&mut g, p, "Cool but Rude", Zone::Battlefield);
        activate(&mut g, class, 0);
        settle(&mut g);
        activate(&mut g, class, 1);
        g.move_object(class, Zone::Hand);
        g.move_object(class, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(class).class_level, Some(1));
        assert!(g.state.resolving.is_none());
        assert!(g.hand_count(p) == 0);
    }
}

#[test]
fn wan_flash_cast_captures_x_for_counters_and_rounded_down_draws() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = put(&mut g, p, "Wan Shi Tong, Librarian", Zone::Hand);
        g.state.active = p.other();
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 5];
        checked(&mut g, Action::AnnounceCast { card: source });
        checked(&mut g, Action::ChooseX { value: 5 });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 5);
        assert_eq!(g.hand_count(p), 2);
        assert!(
            g.keywords(source)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        assert_eq!(g.hand_count(p), 2);
    }
}

#[test]
fn wan_observes_opponent_search_even_on_failure_but_not_own_search() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = put(&mut g, p, "Wan Shi Tong, Librarian", Zone::Battlefield);
        for searching in [p.other(), p] {
            g.state.priority = searching;
            let land = put(&mut g, searching, "Evolving Wilds", Zone::Battlefield);
            activate(&mut g, land, 0);
            reach(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
            assert_eq!(g.hand_count(p), 1);
        }
    }
}
