use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn cycle_index(g: &Game, id: CardId) -> u16 {
    g.state.objects[id as usize]
        .card
        .activations()
        .iter()
        .position(|a| a.origin == ActivationOrigin::Hand)
        .unwrap() as u16
}
fn cycle(g: &mut Game, id: CardId) {
    let index = cycle_index(g, id);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,
        Action::Activate { card, ability, .. } if *card==id && *ability==index)
        })
        .expect("cycle available");
    replay(g, action);
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let mut branch = g.clone();
    act(&mut branch, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(expected, branch.snapshot());
    g.restore(&saved).unwrap();
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision;
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(saved, g.snapshot());
    assert_eq!(revision, g.revision);
}
fn searches(g: &Game) -> Vec<CardId> {
    g.legal_actions()
        .into_iter()
        .filter_map(|a| match a {
            Action::ChooseSearch { card } => Some(card),
            _ => None,
        })
        .collect()
}

#[test]
fn cycling_discards_as_a_cost_draws_on_resolution_and_does_not_cast_or_enter() {
    let mut g = scenario();
    let mage = named(&mut g, Player::P0, "Elementalist Adept", Zone::Battlefield);
    let card = named(&mut g, Player::P0, "Migrating Ketradon", Zone::Hand);
    let original = g.object_ref(card);
    let top = *g.state.players[0].library.last().unwrap();
    g.state.players[0].mana = [0, 2, 0, 0, 0, 0];
    cycle(&mut g, card);
    assert_eq!(g.state.players[0].life, 20);
    assert_eq!(g.state.objects[mage as usize].growth, Stats::default());
    assert!(g.state.pending_triggers.is_empty());
    assert_eq!(g.object_view(card).zone, Zone::Graveyard);
    assert_ne!(g.object_ref(card), original);
    assert_eq!(g.object_view(top).zone, Zone::Library);
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.stack[0].object, original);
    assert!(matches!(
        g.state.stack[0].kind,
        StackKind::ActivatedAbility { .. }
    ));
    assert!(!g.state.stack[0].source_facts.battlefield);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.object_view(top).zone, Zone::Hand);
    assert_eq!(g.object_view(card).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].life, 20);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
}

#[test]
fn hand_origin_checks_owner_zone_actor_and_exact_allocation() {
    for zone in [
        Zone::Battlefield,
        Zone::Graveyard,
        Zone::Exile,
        Zone::Library,
    ] {
        let mut g = scenario();
        let card = named(&mut g, Player::P0, "Locust Spray", zone);
        g.state.players[0].mana = [10; 6];
        reject(
            &mut g,
            Action::Activate {
                card,
                ability: 0,
                target: None,
                payment: [0, 0, 1, 0, 0, 0],
                counter_payment: Counters::default(),
            },
        );
    }
    let mut g = scenario();
    let card = named(&mut g, Player::P1, "Locust Spray", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    reject(
        &mut g,
        Action::Activate {
            card,
            ability: 0,
            target: None,
            payment: [0, 0, 1, 0, 0, 0],
            counter_payment: Counters::default(),
        },
    );
    let own = named(&mut g, Player::P0, "Locust Spray", Zone::Hand);
    let a = Action::Activate {
        card: own,
        ability: 0,
        target: None,
        payment: [0, 0, 1, 0, 0, 0],
        counter_payment: Counters::default(),
    };
    let saved = g.snapshot();
    assert_eq!(g.apply(Player::P1, a.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), saved);
    reject(
        &mut g,
        Action::Activate {
            payment: [0, 0, 2, 0, 0, 0],
            card: own,
            ability: 0,
            target: None,
            counter_payment: Counters::default(),
        },
    );
    replay(&mut g, a);
}

#[test]
fn announced_cycling_keeps_the_source_in_hand_until_all_costs_commit() {
    let mut g = scenario();
    let card = named(&mut g, Player::P0, "Topiary Panther", Zone::Hand);
    let forest = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let island = named(&mut g, Player::P0, "Island", Zone::Battlefield);
    let original = g.object_ref(card);
    let ability = cycle_index(&g, card);
    replay(
        &mut g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    assert_eq!(g.state.pending, Pending::PayCosts { player: Player::P0 });
    assert_eq!(g.object_ref(card), original);
    assert_eq!(g.object_view(card).zone, Zone::Hand);
    assert!(g.state.stack.is_empty());
    assert_eq!(
        g.observe(Player::P1).announcing.unwrap().source_card,
        Card::from_name("Topiary Panther").unwrap()
    );
    assert!(g.observe(Player::P1).actions.is_empty());
    replay(&mut g, Action::TapMana { card: forest });
    replay(&mut g, Action::TapMana { card: island });
    assert_eq!(g.object_view(card).zone, Zone::Hand);
    replay(
        &mut g,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 1, 0],
        },
    );
    assert_eq!(g.object_view(card).zone, Zone::Graveyard);
    assert_eq!(g.state.stack[0].object, original);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    pass_pair(&mut g);
    assert!(g.observe(Player::P0).search_choice.is_some());
}

#[test]
fn funding_protects_the_exact_hand_incarnation_for_the_unpaid_discard() {
    let mut g = scenario();
    let card = named(&mut g, Player::P0, "Lightshield Parry", Zone::Hand);
    named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    replay(
        &mut g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    // Synthetic interruption: an unpaid discard cannot consume a later incarnation.
    g.move_object(card, Zone::Graveyard);
    g.move_object(card, Zone::Hand);
    g.state.players[0].mana = [2, 0, 0, 0, 0, 0];
    assert!(!g.can_fund(g.funding_goal(g.state.announcing.as_ref().unwrap())));
    reject(
        &mut g,
        Action::PayCosts {
            payment: [2, 0, 0, 0, 0, 0],
        },
    );
}

#[test]
fn colored_cycling_cost_is_independent_of_printed_casting_cost() {
    let mut g = scenario();
    let card = named(&mut g, Player::P0, "Topiary Panther", Zone::Hand);
    g.state.players[0].mana = [0, 2, 0, 0, 0, 0];
    reject(
        &mut g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut g,
        Action::Activate {
            card,
            ability: 0,
            target: None,
            payment: [0, 2, 0, 0, 0, 0],
            counter_payment: Counters::default(),
        },
    );
    g.state.players[0].mana = [0, 1, 0, 0, 1, 0];
    cycle(&mut g, card);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn basic_landcycling_search_is_private_then_reveals_only_the_selected_hand_card() {
    let mut g = scenario();
    let card = named(&mut g, Player::P0, "Kree Sentinel", Zone::Hand);
    let nonbasic = named(&mut g, Player::P0, "Commercial District", Zone::Library);
    g.state.players[0].mana = [2, 0, 0, 0, 0, 0];
    cycle(&mut g, card);
    pass_pair(&mut g);
    assert_eq!(
        g.observe(Player::P0).schema,
        crate::engine::OBSERVATION_SCHEMA
    );
    assert_eq!(
        g.observe(Player::P0).search_choice.unwrap().filter,
        crate::SearchFilter::BasicLand
    );
    let hidden = g.observe(Player::P1);
    assert!(hidden.search_choice.is_none());
    assert!(hidden.actions.is_empty());
    assert!(hidden.objects.iter().all(|o| o.zone != Zone::Library));
    assert!(!searches(&g).contains(&nonbasic));
    let selected = searches(&g)[0];
    let other = searches(&g)[1];
    let rng = g.state.rng;
    replay(&mut g, Action::ChooseSearch { card: selected });
    assert_eq!(g.object_view(selected).zone, Zone::Library);
    assert_eq!(g.state.rng, rng);
    reject(&mut g, Action::ChooseSearch { card: other });
    replay(&mut g, Action::FinishSearch);
    assert_eq!(g.object_view(selected).zone, Zone::Hand);
    assert_ne!(g.state.rng, rng);
    let public = g.observe(Player::P1);
    assert!(
        public
            .objects
            .iter()
            .any(|o| o.id == selected && o.zone == Zone::Hand)
    );
    assert!(public.known_hand.contains(&g.object_ref(selected)));
    assert!(public.objects.iter().all(|o| o.id != other));
}

#[test]
fn forestcycling_finds_nonbasic_forests_but_not_unrelated_lands_or_creatures() {
    let mut g = scenario();
    let card = named(&mut g, Player::P0, "Balamb T-Rexaur", Zone::Hand);
    let dual = named(&mut g, Player::P0, "Commercial District", Zone::Library);
    let island = named(&mut g, Player::P0, "Island", Zone::Library);
    let creature = named(&mut g, Player::P0, "Grizzly Bears", Zone::Library);
    g.state.players[0].mana = [2, 0, 0, 0, 0, 0];
    cycle(&mut g, card);
    pass_pair(&mut g);
    assert_eq!(
        g.observe(Player::P0).search_choice.unwrap().filter,
        crate::SearchFilter::Subtype("Forest")
    );
    assert!(searches(&g).contains(&dual));
    for bad in [island, creature] {
        assert!(!searches(&g).contains(&bad));
        reject(&mut g, Action::ChooseSearch { card: bad });
    }
    replay(&mut g, Action::ChooseSearch { card: dual });
    replay(&mut g, Action::FinishSearch);
    assert_eq!(g.object_view(dual).zone, Zone::Hand);
    assert_eq!(g.state.players[0].life, 20);
}

#[test]
fn every_admitted_land_type_uses_its_own_subtype_filter() {
    for (name, land) in [
        ("Shepherding Spirits", "Plains"),
        ("Daggermaw Megalodon", "Island"),
        ("Canyon Crawler", "Swamp"),
        ("Bedhead Beastie", "Mountain"),
        ("Slavering Branchsnapper", "Forest"),
    ] {
        let mut g = scenario();
        let card = named(&mut g, Player::P0, name, Zone::Hand);
        let right = named(&mut g, Player::P0, land, Zone::Library);
        let wrong = named(
            &mut g,
            Player::P0,
            if land == "Island" { "Swamp" } else { "Island" },
            Zone::Library,
        );
        g.state.players[0].mana = [2, 0, 0, 0, 0, 0];
        cycle(&mut g, card);
        pass_pair(&mut g);
        assert!(searches(&g).contains(&right));
        assert!(!searches(&g).contains(&wrong));
        replay(&mut g, Action::ChooseSearch { card: right });
        replay(&mut g, Action::FinishSearch);
        assert_eq!(g.object_view(right).zone, Zone::Hand);
    }
}

#[test]
fn typecycling_can_fail_to_find_and_still_shuffle_with_exact_rng_replay() {
    for name in ["Hobbit Hole", "Balamb T-Rexaur"] {
        let mut g = scenario();
        let card = named(&mut g, Player::P0, name, Zone::Hand);
        g.state.players[0].mana = [4, 0, 0, 0, 0, 0];
        cycle(&mut g, card);
        pass_pair(&mut g);
        let rng = g.state.rng;
        let before = g.state.players[0].library.len();
        replay(&mut g, Action::FinishSearch);
        assert_eq!(g.state.players[0].library.len(), before);
        assert_ne!(g.state.rng, rng);
        assert!(g.state.resolving.is_none());
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
    }
}

#[test]
fn cycling_can_respond_on_opponents_turn_and_is_not_a_counterspell_target() {
    let mut g = scenario();
    let shock = named(&mut g, Player::P0, "Shock", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 1, 0, 0];
    cast(&mut g, shock, Some(Target::Player(Player::P1)));
    act(&mut g, Action::Pass);
    let card = named(&mut g, Player::P1, "Lightshield Parry", Zone::Hand);
    let counter = named(&mut g, Player::P1, "Counterspell", Zone::Hand);
    g.state.players[1].mana = [0, 4, 0, 0, 0, 0];
    cycle(&mut g, card);
    assert_eq!(g.state.stack.len(), 2);
    let original = g.state.stack[1].object;
    assert!(!g.target_matches(
        TargetRule::Spell,
        Player::P1,
        g.object_ref(counter),
        Target::Spell(card)
    ));
    // Reusing the physical source cannot turn the cycling ability into a spell.
    g.move_object(card, Zone::Hand);
    assert_ne!(g.object_ref(card), original);
    pass_pair(&mut g);
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.players[1].life, 20);
    assert_eq!(g.object_view(card).zone, Zone::Hand);
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 18);
}

#[test]
fn cycling_lands_and_mixed_permanents_keep_separate_hand_and_battlefield_abilities() {
    let mut g = scenario();
    let city = named(&mut g, Player::P0, "Capital City", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    assert!(g.legal_actions().contains(&Action::PlayLand { card: city }));
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::ActivateMana { card,.. } if *card==city))
    );
    replay(&mut g, Action::PlayLand { card: city });
    assert!(
        g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::ActivateMana {card,ability:0,..} if *card==city))
    );
    let ability = cycle_index(&g, city);
    reject(
        &mut g,
        Action::Activate {
            card: city,
            ability,
            target: None,
            payment: [2, 0, 0, 0, 0, 0],
            counter_payment: Counters::default(),
        },
    );
    let witness = named(&mut g, Player::P0, "Undulating Witness", Zone::Battlefield);
    assert_eq!(cycle_index(&g, witness), 1);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate {card,ability:0,..} if *card==witness))
        .unwrap();
    replay(&mut g, action);
    pass_pair(&mut g);
    assert_eq!(
        g.state.objects[witness as usize].growth,
        Stats {
            power: 1,
            toughness: -1
        }
    );
}

#[test]
fn original_spell_token_and_enter_die_programs_remain_executable() {
    let mut g = scenario();
    g.state.players[0].mana = [10; 6];
    let backup = named(&mut g, Player::P0, "Borough Backup", Zone::Hand);
    cast(&mut g, backup, None);
    pass_pair(&mut g);
    let heroes: Vec<_> = g
        .state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
        .collect();
    assert_eq!(heroes.len(), 2);
    assert!(
        heroes
            .iter()
            .all(|o| o.card.keywords().contains(Keywords::VIGILANCE))
    );
    let apex = named(&mut g, Player::P0, "Apex Witchstalker", Zone::Hand);
    cast(&mut g, apex, None);
    pass_pair(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 22);
    g.move_object(apex, Zone::Graveyard);
    g.prepare_triggers();
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 24);
}

#[test]
fn cycling_is_illegal_inside_resolution_mana_and_private_search_windows() {
    let mut g = scenario();
    g.state.players[0].mana = [10; 6];
    let bears = named(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(&mut g, bears, None);
    act(&mut g, Action::Pass);
    let quench = named(&mut g, Player::P1, "It'll Quench Ya!", Zone::Hand);
    g.state.players[1].mana = [10; 6];
    cast(&mut g, quench, Some(Target::Spell(bears)));
    pass_pair(&mut g);
    let cycle_card = named(&mut g, Player::P0, "Lightshield Parry", Zone::Hand);
    reject(
        &mut g,
        Action::Activate {
            card: cycle_card,
            ability: 0,
            target: None,
            payment: [2, 0, 0, 0, 0, 0],
            counter_payment: Counters::default(),
        },
    );
    replay(&mut g, Action::DeclineResolutionMana);
    let search_card = named(&mut g, Player::P0, "Kree Sentinel", Zone::Hand);
    cycle(&mut g, search_card);
    pass_pair(&mut g);
    reject(
        &mut g,
        Action::Activate {
            card: cycle_card,
            ability: 0,
            target: None,
            payment: [2, 0, 0, 0, 0, 0],
            counter_payment: Counters::default(),
        },
    );
}

#[test]
fn cycling_with_empty_library_is_legal_but_loses_only_when_draw_resolves() {
    let mut g = scenario();
    let ids = g.state.players[0].library.clone();
    for id in ids {
        g.move_object(id, Zone::Graveyard);
    }
    let card = named(&mut g, Player::P0, "Lightshield Parry", Zone::Hand);
    g.state.players[0].mana = [2, 0, 0, 0, 0, 0];
    cycle(&mut g, card);
    assert!(g.outcome().is_none());
    pass_pair(&mut g);
    assert!(g.outcome().is_some());
}
