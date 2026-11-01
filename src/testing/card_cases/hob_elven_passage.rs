//! Focused printed outcomes; shared engine invariants live in core tests.
//! Search, behold and the exact tapped land are separate sequential events.
use super::*;

fn activate(g: &mut Game, land: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: land,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
}
fn search(g: &mut Game) {
    for _ in 0..16 {
        if matches!(g.state.pending, Pending::EffectSearch { .. }) {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("search did not start");
}
#[test]
fn elven_passage_actual_cost_search_and_optional_behold() {
    {
        let p = Player::P0;
        for mode in 0..3 {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Elven Passage", Zone::Battlefield);
            let forest = put(&mut g, p, "Forest", Zone::Library);
            let elf = put(
                &mut g,
                p,
                "Llanowar Elves",
                if mode == 1 {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                },
            );
            let other = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
            let life = g.state.players[p.index()].life;
            activate(&mut g, land);
            assert_eq!(g.state.players[p.index()].life, life - 1);
            assert_eq!(g.object_view(land).zone, Zone::Graveyard);
            search(&mut g);
            assert!(g.observe(p.other()).search_choice.is_none());
            let rng = g.state.rng;
            reject(&mut g, Action::ChooseSearch { card: elf });

            act(&mut g, Action::ChooseSearch { card: forest });
            act(&mut g, Action::FinishSearch);
            assert_ne!(g.state.rng, rng);
            assert_eq!(g.object_view(forest).zone, Zone::Battlefield);
            assert!(
                g.state.objects[forest as usize].tapped,
                "land must enter tapped before behold"
            );
            let er = g.object_ref(elf);
            let other_ref = g.object_ref(other);
            assert!(matches!(g.state.pending, Pending::HandCardChoice { player } if player == p));
            assert!(g.observe(p.other()).hand_choice.is_none());
            reject(
                &mut g,
                Action::ChooseHandCard {
                    object: Some(other_ref),
                },
            );

            if mode == 0 {
                act(&mut g, Action::ChooseHandCard { object: None });
                assert!(g.state.objects[forest as usize].tapped);
            } else {
                act(&mut g, Action::ChooseHandCard { object: Some(er) });
                assert!(!g.state.objects[forest as usize].tapped);
                if mode == 1 {
                    assert!(g.observe(p.other()).known_hand.contains(&er));
                }
            }
            settle(&mut g);
        }
    }
}
#[test]
fn elven_passage_can_fail_to_find_and_still_behold_or_have_no_elf() {
    for has_elf in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let land = put(&mut g, p, "Elven Passage", Zone::Battlefield);
        let elf = has_elf.then(|| put(&mut g, p, "Llanowar Elves", Zone::Hand));
        activate(&mut g, land);
        search(&mut g);
        let rng = g.state.rng;
        act(&mut g, Action::FinishSearch);
        assert_ne!(g.state.rng, rng);
        if let Some(elf) = elf {
            let er = g.object_ref(elf);
            assert!(matches!(g.state.pending, Pending::HandCardChoice { .. }));
            act(&mut g, Action::ChooseHandCard { object: Some(er) });
            assert!(g.observe(p.other()).known_hand.contains(&er));
        }
        settle(&mut g);
    }
}

#[test]
fn elven_passage_landfall_waits_for_behold_to_finish() {
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p, "Elven Passage", Zone::Battlefield);
    let forest = put(&mut g, p, "Forest", Zone::Library);
    let spider = put(&mut g, p, "Attercop", Zone::Battlefield);
    let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    activate(&mut g, land);
    search(&mut g);
    act(&mut g, Action::ChooseSearch { card: forest });
    act(&mut g, Action::FinishSearch);
    assert_eq!(stats(&g, spider), (2, 1));
    assert!(g.state.objects[forest as usize].tapped);
    assert!(!g.state.pending_triggers.is_empty());
    let er = g.object_ref(elf);
    act(&mut g, Action::ChooseHandCard { object: Some(er) });
    assert!(!g.state.objects[forest as usize].tapped);
    settle(&mut g);
    assert_eq!(stats(&g, spider), (3, 2));
}
