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
    for p in [Player::P0, Player::P1] {
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
            contracts::branch_contract(g.clone());
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
            contracts::branch_contract(g.clone());
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
fn elven_passage_unpayable_life_or_tapped_land_is_transactional() {
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p, "Elven Passage", Zone::Battlefield);
    let action = Action::AnnounceActivate {
        card: land,
        ability: 0,
        counter_payment: Counters::default(),
    };
    g.state.objects[land as usize].tapped = true;
    reject(&mut g, action.clone());
    g.state.objects[land as usize].tapped = false;
    g.state.players[p.index()].life = 0;
    reject(&mut g, action);
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
#[test]
fn elven_passage_synthetic_entry_choice_preserves_search_result_and_shuffle() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Elven Passage", Zone::Battlefield);
        let courtyard = put(&mut g, p, "Secluded Courtyard", Zone::Library);
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        activate(&mut g, land);
        search(&mut g);
        // A basic land has no intrinsic choice; exercise the generic search result
        // continuation with an Any-filter search and the actual entry-choice machinery.
        g.state
            .resolving
            .as_mut()
            .unwrap()
            .search
            .as_mut()
            .unwrap()
            .spec
            .filter = crate::SearchFilter::Any;
        let rng = g.state.rng;
        act(&mut g, Action::ChooseSearch { card: courtyard });
        act(&mut g, Action::FinishSearch);
        assert!(g.observe(p).entry_value.is_some());
        assert_eq!(g.state.rng, rng);
        contracts::branch_contract(g.clone());
        act(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(
                    crate::cards::CreatureType::from_name("Wizard").unwrap(),
                ),
            },
        );
        assert_ne!(g.state.rng, rng);
        assert_eq!(g.object_view(courtyard).zone, Zone::Battlefield);
        assert!(g.state.objects[courtyard as usize].tapped);
        assert!(matches!(g.state.pending, Pending::HandCardChoice { player } if player == p));
        let er = g.object_ref(elf);
        act(&mut g, Action::ChooseHandCard { object: Some(er) });
        assert!(!g.state.objects[courtyard as usize].tapped);
        settle(&mut g);
    }
}
#[test]
fn elven_passage_behold_uses_current_subtype_and_controller() {
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p, "Elven Passage", Zone::Battlefield);
    let forest = put(&mut g, p, "Forest", Zone::Library);
    let lost = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let gained = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let stolen = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
    let lost_ref = g.object_ref(lost);
    let gained_ref = g.object_ref(gained);
    let stolen_ref = g.object_ref(stolen);
    let timestamp = g.next_role_timestamp();
    let roles = g.state.roles.get_or_insert_with(Default::default);
    roles
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient: lost_ref,
            source: lost_ref,
            types: &["Goblin"],
            additional: false,
            timestamp,
        });
    roles
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient: gained_ref,
            source: gained_ref,
            types: &["Elf"],
            additional: false,
            timestamp,
        });
    g.change_controller(stolen_ref, p);
    assert!(!g.has_subtype(lost, "Elf"));
    assert!(g.has_subtype(gained, "Elf"));
    activate(&mut g, land);
    search(&mut g);
    act(&mut g, Action::ChooseSearch { card: forest });
    act(&mut g, Action::FinishSearch);
    reject(
        &mut g,
        Action::ChooseHandCard {
            object: Some(lost_ref),
        },
    );
    assert!(g.legal_actions().contains(&Action::ChooseHandCard {
        object: Some(stolen_ref)
    }));
    act(
        &mut g,
        Action::ChooseHandCard {
            object: Some(gained_ref),
        },
    );
    assert!(!g.state.objects[forest as usize].tapped);
    settle(&mut g);
}
#[test]
fn elven_passage_synthetic_found_land_reentry_does_not_rebind() {
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p, "Elven Passage", Zone::Battlefield);
    let forest = put(&mut g, p, "Forest", Zone::Library);
    let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    activate(&mut g, land);
    search(&mut g);
    act(&mut g, Action::ChooseSearch { card: forest });
    act(&mut g, Action::FinishSearch);
    let old = g.object_ref(forest);
    // Synthetic incarnation change during the suspended choice tests the exact
    // result contract independently of ordinary no-priority resolution.
    g.move_object(forest, Zone::Hand);
    g.move_object(forest, Zone::Battlefield);
    assert_ne!(old, g.object_ref(forest));
    g.state.objects[forest as usize].tapped = true;
    let er = g.object_ref(elf);
    act(&mut g, Action::ChooseHandCard { object: Some(er) });
    assert!(g.state.objects[forest as usize].tapped);
    settle(&mut g);
}
