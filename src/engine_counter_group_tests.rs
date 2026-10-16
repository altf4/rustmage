use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn funded(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    let (mut pool, generic) = g.state.objects[id as usize].card.cost();
    pool[5] += generic;
    g.state.players[0].mana = pool;
    id
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(g, a.clone());
    act(&mut fork, a.clone());
    let after = g.snapshot();
    assert_eq!(after, fork.snapshot());
    g.restore(&saved);
    assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(saved, g.snapshot());
    assert_eq!(revision, g.revision());
}
fn choose(g: &mut Game, id: CardId) {
    replay(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
}
fn enter(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    g.state_based_actions();
    g.prepare_triggers();
    id
}
fn activate(g: &mut Game, id: CardId) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::AnnounceActivate{card,..} if *card==id))
        .unwrap();
    replay(g, a);
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
#[test]
fn darkness_is_nontargeted_hits_both_players_and_zero_toughness_ignores_indestructible() {
    let mut g = scenario();
    let own = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let big = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    g.state.objects[enemy as usize].gained_keywords =
        Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE);
    g.state.objects[big as usize].counters.plus_one = 3;
    let spell = funded(&mut g, "Darkness Descends");
    replay(&mut g, Action::AnnounceCast { card: spell });
    assert_eq!(g.observe(Player::P0).decision, "pay_costs");
    pay(&mut g);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    for id in [own, enemy] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    assert_eq!(g.object_view(big).counters.plus_one, 1);
    assert_eq!(g.object_view(big).counters.minus_one, 0);
    assert_eq!(g.object_view(big).power, 4);
}
#[test]
fn earth_soldier_targets_zero_one_or_two_distinct_owned_creatures_and_only_current_incarnations() {
    for number in 0..=2 {
        let mut g = scenario();
        let a = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let soldier = enter(&mut g, "Earth Kingdom Soldier");
        assert_eq!(g.observe(Player::P0).decision, "targets");
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        if number > 0 {
            choose(&mut g, a);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
        }
        if number == 2 {
            choose(&mut g, b);
        } else {
            replay(&mut g, Action::FinishTargets);
        }
        // A different incarnation does not receive counters; other legal targets still do.
        if number == 2 {
            g.move_object(a, Zone::Hand);
            g.move_object(a, Zone::Battlefield);
        }
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert_eq!(
            g.object_view(a).counters.plus_one,
            if number == 1 { 1 } else { 0 }
        );
        assert_eq!(
            g.object_view(b).counters.plus_one,
            if number == 2 { 1 } else { 0 }
        );
        assert_eq!(g.object_view(soldier).counters.plus_one, 0);
    }
}
#[test]
fn beginning_combat_counters_choose_both_players_but_respect_opposing_hexproof() {
    let mut g = scenario();
    let bishop = named(
        &mut g,
        Player::P0,
        "Agent Bishop, Man in Black",
        Zone::Battlefield,
    );
    let own = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.objects[enemy as usize].gained_keywords = Keywords::HEXPROOF;
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::BeginCombat);
    assert_eq!(g.observe(Player::P0).decision, "targets");
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    choose(&mut g, bishop);
    choose(&mut g, own);
    pass_pair(&mut g);
    assert_eq!(g.object_view(bishop).counters.plus_one, 1);
    assert_eq!(g.object_view(own).counters.plus_one, 1);
    assert_eq!(g.object_view(enemy).counters.plus_one, 0);
    g.state.active = Player::P1;
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    g.prepare_triggers();
    assert!(g.state.stack.is_empty());
}
#[test]
fn hero_counters_use_resolution_time_owned_type_and_other_incarnation() {
    let mut g = scenario();
    let coulson = named(&mut g, Player::P0, "Agent Phil Coulson", Zone::Battlefield);
    let hero = named(&mut g, Player::P0, "Web-Warriors", Zone::Battlefield);
    let change = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Web-Warriors", Zone::Battlefield);
    g.state.objects[coulson as usize].summoning_sick = false;
    activate(&mut g, coulson);
    pay(&mut g);
    assert!(g.object_view(coulson).tapped);
    assert!(!g.object_view(hero).modified);
    // The non-targeted group is evaluated at resolution, including entrants since activation.
    let fresh = named(&mut g, Player::P0, "Web-Warriors", Zone::Battlefield);
    g.move_object(coulson, Zone::Hand);
    g.move_object(coulson, Zone::Battlefield);
    pass_pair(&mut g);
    for id in [coulson, hero, change, fresh] {
        assert_eq!(g.object_view(id).counters.plus_one, 1);
    }
    assert_eq!(g.object_view(enemy).counters.plus_one, 0);
}
#[test]
fn gain_life_triggers_counter_only_matching_own_creature_types_including_changelings() {
    for (name, other) in [
        ("Minwu, White Mage", "Lyra, Archangel of Dawn"),
        ("Lyra, Archangel of Dawn", "Minwu, White Mage"),
    ] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let excluded = named(&mut g, Player::P0, other, Zone::Battlefield);
        let changeling = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        let enemy = named(&mut g, Player::P1, name, Zone::Battlefield);
        g.gain_life(Player::P0, 2);
        g.prepare_triggers();
        assert_eq!(g.state.ordering_triggers.len(), 2);
        // Each controller's Angel and Cleric observer sees the same life event, then its own type set.
        while !g.state.stack.is_empty() || !g.state.ordering_triggers.is_empty() {
            if let Some(a) = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
            {
                replay(&mut g, a);
            } else {
                pass_pair(&mut g);
            }
        }
        assert_eq!(g.object_view(source).counters.plus_one, 1);
        assert_eq!(g.object_view(excluded).counters.plus_one, 1);
        assert_eq!(g.object_view(changeling).counters.plus_one, 2);
        assert_eq!(g.object_view(enemy).counters.plus_one, 0);
    }
}
#[test]
fn zimone_doubles_every_existing_counter_kind_on_creatures_or_artifacts_once_per_incarnation() {
    let mut g = scenario();
    let z = named(
        &mut g,
        Player::P0,
        "Zimone, Paradox Sculptor",
        Zone::Battlefield,
    );
    g.state.objects[z as usize].summoning_sick = false;
    let creature = named(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let artifact = named(&mut g, Player::P0, "Mox Jasper", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Mox Jasper", Zone::Battlefield);
    let land = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    g.state.objects[creature as usize].counters = Counters {
        plus_one: 2,
        minus_one: 1,
        stun: 2,
    };
    g.state.objects[artifact as usize].counters = Counters {
        plus_one: 3,
        minus_one: 0,
        stun: 4,
    };
    g.state.players[0].mana = [0, 1, 0, 0, 1, 0];
    activate(&mut g, z);
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    choose(&mut g, creature);
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    choose(&mut g, artifact);
    pay(&mut g);
    g.move_object(z, Zone::Hand);
    pass_pair(&mut g);
    // Existing pairs canceled by SBAs before resolution; the surviving counts double.
    assert_eq!(
        g.object_view(creature).counters,
        Counters {
            plus_one: 2,
            minus_one: 0,
            stun: 4
        }
    );
    assert_eq!(
        g.object_view(artifact).counters,
        Counters {
            plus_one: 6,
            minus_one: 0,
            stun: 8
        }
    );
    g.state.objects[artifact as usize].tapped = true;
    g.state.objects[artifact as usize].untap();
    assert_eq!(g.object_view(artifact).counters.stun, 7);
    assert!(g.object_view(artifact).tapped);
}
#[test]
fn crystal_creates_four_heroes_then_counters_all_owned_creatures() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = funded(&mut g, "The Crystal's Chosen");
    cast(&mut g, spell, None);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    let heroes: Vec<_> = g
        .state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
        .collect();
    assert_eq!(heroes.len(), 4);
    assert!(
        heroes.iter().all(|o| o.owner == Player::P0
            && o.card.has_subtype("Hero")
            && o.counters.plus_one == 1)
    );
    assert_eq!(g.object_view(bear).counters.plus_one, 1);
    assert_eq!(g.object_view(enemy).counters.plus_one, 0);
}
#[test]
fn duty_sacrifice_is_paid_before_team_protection_and_counter_placement() {
    let mut g = scenario();
    let sacrificed = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let survivor = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = funded(&mut g, "Duty Beyond Death");
    replay(&mut g, Action::AnnounceCast { card: spell });
    let object = g.object_ref(sacrificed);
    replay(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    assert_eq!(g.object_view(sacrificed).zone, Zone::Graveyard);
    assert!(!g.keywords(survivor).contains(Keywords::INDESTRUCTIBLE));
    pass_pair(&mut g);
    assert!(g.keywords(survivor).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(g.object_view(survivor).counters.plus_one, 1);
    assert!(!g.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(g.object_view(enemy).counters.plus_one, 0);
    g.finish_cleanup();
    assert!(!g.keywords(survivor).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(g.object_view(survivor).counters.plus_one, 1);
}
#[test]
fn air_temple_entry_condition_and_nonmana_tap_cost_are_enforced() {
    let mut g = scenario();
    let land = named(&mut g, Player::P0, "Abandoned Air Temple", Zone::Hand);
    replay(&mut g, Action::PlayLand { card: land });
    assert!(g.object_view(land).tapped);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.begin_turn();
    g.state.players[0].mana = [1, 0, 0, 0, 0, 3];
    activate(&mut g, land);
    pay(&mut g);
    assert!(g.object_view(land).tapped);
    pass_pair(&mut g);
    assert_eq!(g.object_view(bear).counters.plus_one, 1);
    g.move_object(land, Zone::Hand);
    named(&mut g, Player::P0, "Plains", Zone::Battlefield);
    g.state.land_played = false;
    g.state.step = Step::Main1;
    replay(&mut g, Action::PlayLand { card: land });
    assert!(!g.object_view(land).tapped);
}
#[test]
fn web_warriors_excludes_original_source_and_lyra_dawnbringer_grants_other_angels() {
    let mut g = scenario();
    let own = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let source = enter(&mut g, "Web-Warriors");
    pass_pair(&mut g);
    assert_eq!(g.object_view(source).counters.plus_one, 0);
    assert_eq!(g.object_view(own).counters.plus_one, 1);
    let dawn = named(&mut g, Player::P0, "Lyra Dawnbringer", Zone::Battlefield);
    let angel = named(
        &mut g,
        Player::P0,
        "Lyra, Archangel of Dawn",
        Zone::Battlefield,
    );
    assert_eq!(g.object_view(dawn).power, 5);
    assert_eq!(g.object_view(angel).power, 4);
    assert!(g.keywords(angel).contains(Keywords::LIFELINK));
    g.move_object(dawn, Zone::Hand);
    assert_eq!(g.object_view(angel).power, 3);
    assert!(!g.keywords(angel).contains(Keywords::LIFELINK));
}
#[test]
fn group_counter_instructions_finish_before_state_based_deaths_and_pair_cancellation() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Shock", Zone::Hand);
    let item = g.cast_item(Player::P0, spell, [0; 6]);
    // Synthetic IR: no otherwise unsupported printed card is admitted by this fixture.
    const FILTER: CreatureFilter = CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    };
    g.state.resolving = Some(Resolution {
        item,
        program: &[
            Instruction::PutCreatureCounters {
                filter: FILTER,
                kind: CounterKind::MinusOne,
                amount: 2,
            },
            Instruction::PutCreatureCounters {
                filter: FILTER,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
        pc: 0,
        target_group: 0,
        connive: None,
        mana_payment: None,
        discards: [0; 2],
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        search: None,
        choice: None,
    });
    let before = g.snapshot();
    let mut fork = g.clone();
    g.run_program();
    g.state_based_actions();
    fork.run_program();
    fork.state_based_actions();
    assert_eq!(g.snapshot(), fork.snapshot());
    let expected = g.snapshot();
    g.restore(&before);
    g.run_program();
    g.state_based_actions();
    assert_eq!(g.snapshot(), expected);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert_eq!(g.object_view(bear).toughness, 2);
    assert_eq!(g.object_view(bear).counters, Counters::default());
}
#[test]
fn darkness_deaths_are_simultaneous_and_dying_reaper_sees_the_whole_batch() {
    let mut g = scenario();
    let reaper = named(&mut g, Player::P0, "Midnight Reaper", Zone::Battlefield);
    let a = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let b = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = funded(&mut g, "Darkness Descends");
    cast(&mut g, spell, None);
    pass_pair(&mut g);
    for id in [reaper, a, b] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    assert_eq!(g.state.ordering_triggers.len(), 3);
    assert!(g.state.ordering_triggers.iter().all(|t| t.card == reaper));
    while !g.state.stack.is_empty() || !g.state.ordering_triggers.is_empty() {
        if let Some(a) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
        {
            replay(&mut g, a);
        } else {
            pass_pair(&mut g);
        }
    }
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
    assert_eq!(g.state.players[0].life, 17);
}
