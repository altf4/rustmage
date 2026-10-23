//! Paid land, exile, flash-return and shuffle scenarios with owned raw/index rollback.
use super::ability_timeline_tests::{checked, named, next_turn, settle, setup};
use super::tmt_blink_permissions_tests::cast;
use super::*;
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn land(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::PlayLand { card: id });
    settle(g);
    id
}
fn activate(g: &mut Game, id: CardId, ability: u16, target: Option<CardId>) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(id) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    for _ in 0..16 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .unwrap();
        checked(g, action);
    }
    panic!("activation stalled");
}
fn until(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..50 {
        if predicate(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishTargets | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("pending {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("pending {:?}", g.state.pending);
}
fn mature(g: &mut Game) {
    next_turn(g);
    next_turn(g);
    while g.state.step != Step::Main1 {
        checked(g, Action::Pass);
    }
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
}
#[test]
fn tmt_exile_escape_paid_search_is_private_basic_tapped_and_shuffle_can_fail_to_find() {
    for p in [Player::P0, Player::P1] {
        for choose in [false, true] {
            let mut g = setup(p);
            let tunnel = land(&mut g, p, "Escape Tunnel");
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a,Action::ActivateMana {card,..}if *card==tunnel))
            );
            let basic = *g.state.players[p.index()].library.last().unwrap();
            activate(&mut g, tunnel, 0, None);
            assert_eq!(g.state.objects[tunnel as usize].zone, Zone::Graveyard);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: tunnel });
            if choose {
                checked(&mut g, Action::ChooseSearch { card: basic });
            }
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(
                g.state.objects[basic as usize].zone,
                if choose {
                    Zone::Battlefield
                } else {
                    Zone::Library
                }
            );
            if choose {
                assert!(g.state.objects[basic as usize].tapped);
            }
        }
    }
}
#[test]
fn tmt_exile_escape_revalidates_power_and_duration() {
    for grow_before in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let tunnel = land(&mut g, p, "Escape Tunnel");
        let target = paid(&mut g, p, "Llanowar Elves");
        let big = paid(&mut g, p, "Serra Angel");
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: tunnel,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(big),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        if grow_before {
            g.state.objects[target as usize].counters.plus_one = 3;
        }
        settle(&mut g);
        assert_eq!(
            g.keywords(target).contains(Keywords::CANT_BE_BLOCKED),
            !grow_before
        );
        if !grow_before {
            g.state.objects[target as usize].counters.plus_one = 3;
            assert!(g.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
        }
        next_turn(&mut g);
        assert!(!g.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
    }
}
#[test]
fn tmt_exile_farm_paid_owns_not_controls_and_returns_chosen_under_activator() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let farm = land(&mut g, p, "Northampton Farm");
        checked(
            &mut g,
            Action::ActivateMana {
                card: farm,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        mature(&mut g);
        let first = paid(&mut g, p, "Llanowar Elves");
        let other = paid(&mut g, p, "Serra Angel");
        let foreign = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        g.record_control(g.object_ref(first), p.other(), None);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: farm,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.state.objects[first as usize].zone, Zone::Exile);
        g.state.objects[farm as usize].tapped = false;
        activate(&mut g, farm, 1, Some(other));
        settle(&mut g);
        g.state.objects[farm as usize].tapped = false;
        activate(&mut g, farm, 2, None);
        assert_eq!(g.state.objects[farm as usize].zone, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        assert_eq!(
            g.observe(p).effect_choice.as_ref().unwrap().kind,
            EffectChoiceKind::ReturnLinkedCreature
        );
        let foreign_ref = g.object_ref(foreign);
        let first_ref = g.object_ref(first);
        reject(
            &mut g,
            Action::ChooseCreature {
                object: foreign_ref,
            },
        );
        checked(&mut g, Action::ChooseCreature { object: first_ref });
        settle(&mut g);
        assert_eq!(g.state.objects[first as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[first as usize].controller, p);
        assert_eq!(g.state.objects[other as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[other as usize].owner, p);
    }
}
#[test]
fn tmt_exile_farm_empty_and_noncreature_links_go_to_owner_hand() {
    let p = Player::P0;
    let mut g = setup(p);
    let farm = land(&mut g, p, "Northampton Farm");
    let artifact = paid(&mut g, p, "Mazemind Tome");
    let item = g.activation_item(p, artifact, 0);
    g.animate_source(
        &item,
        crate::cards::AnimationSpec {
            artifact: true,
            creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: crate::cards::AnimationDuration::EndOfTurn,
        },
    );
    activate(&mut g, farm, 1, Some(artifact));
    settle(&mut g);
    assert_eq!(g.state.objects[artifact as usize].zone, Zone::Exile);
    g.state.objects[farm as usize].tapped = false;
    activate(&mut g, farm, 2, None);
    settle(&mut g);
    assert_eq!(g.state.objects[artifact as usize].zone, Zone::Hand);
}
#[test]
fn tmt_exile_farm_departure_reentry_does_not_unlock_old_links() {
    let p = Player::P0;
    let mut g = setup(p);
    let farm = land(&mut g, p, "Northampton Farm");
    let creature = paid(&mut g, p, "Llanowar Elves");
    activate(&mut g, farm, 1, Some(creature));
    settle(&mut g);
    let old = g.object_ref(farm);
    g.move_object(farm, Zone::Exile);
    g.move_object(farm, Zone::Battlefield);
    assert_ne!(g.object_ref(farm), old);
    g.state.objects[farm as usize].tapped = false;
    activate(&mut g, farm, 2, None);
    settle(&mut g);
    assert_eq!(g.state.objects[creature as usize].zone, Zone::Exile);
}
#[test]
fn tmt_exile_renet_flash_returns_all_recent_nonlands_to_owners() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = paid(&mut g, p, "Llanowar Elves");
        mature(&mut g);
        let fresh = paid(&mut g, p, "Serra Angel");
        let artifact = paid(&mut g, p, "Mazemind Tome");
        let land = land(&mut g, p, "Forest");
        g.record_control(g.object_ref(fresh), p.other(), None);
        let renet = named(&mut g, p, "Renet, Temporal Apprentice", Zone::Hand);
        g.state.step = Step::End;
        cast(&mut g, renet, None, &[]);
        settle(&mut g);
        for id in [fresh, artifact] {
            assert_eq!(g.state.objects[id as usize].zone, Zone::Hand);
            assert_eq!(g.state.objects[id as usize].owner, p);
        }
        for id in [old, land, renet] {
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        }
        assert_eq!(g.stats(renet).power, 4);
        assert_eq!(g.stats(renet).toughness, 3);
        assert_eq!(
            g.observe(p).permanent_entry_history,
            g.observe(p.other()).permanent_entry_history
        );
    }
}
#[test]
fn tmt_exile_renet_uses_entry_not_control_change_or_old_incarnation() {
    let p = Player::P0;
    let mut g = setup(p);
    let old = paid(&mut g, p, "Serra Angel");
    mature(&mut g);
    g.record_control(g.object_ref(old), p.other(), None);
    let fresh = paid(&mut g, p, "Llanowar Elves");
    let before = g.object_ref(fresh);
    g.move_object(fresh, Zone::Exile);
    g.move_object(fresh, Zone::Battlefield);
    assert_ne!(before, g.object_ref(fresh));
    let renet = paid(&mut g, p, "Renet, Temporal Apprentice");
    assert_eq!(g.state.objects[old as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[fresh as usize].zone, Zone::Hand);
    assert_eq!(g.state.objects[renet as usize].zone, Zone::Battlefield);
}
#[test]
fn tmt_exile_dimensional_paid_basic_land_target_and_until_leave() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let basic = land(&mut g, p, "Forest");
        let foreign = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let nonbasic = named(&mut g, p, "Escape Tunnel", Zone::Battlefield);
        let creature = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let aura = named(&mut g, p, "Dimensional Exile", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: aura });
        for id in [foreign, nonbasic] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(basic),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        super::ability_timeline_tests::settle_on(&mut g, Some(Target::Card(creature)));
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Exile);
        assert_eq!(
            g.state.objects[aura as usize].attached_to,
            Some(g.object_ref(basic))
        );
        g.move_object(basic, Zone::Graveyard);
        checked(&mut g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.state.objects[aura as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn tmt_exile_dimensional_control_change_detaches_and_departed_source_does_not_exile() {
    for depart_before in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let basic = land(&mut g, p, "Forest");
        let creature = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let aura = named(&mut g, p, "Dimensional Exile", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(basic)]);
        until(&mut g, |g| g.on_battlefield(aura));
        if depart_before {
            g.move_object(aura, Zone::Graveyard);
        }
        super::ability_timeline_tests::settle_on(&mut g, Some(Target::Card(creature)));
        if !depart_before {
            assert_eq!(g.state.objects[creature as usize].zone, Zone::Exile);
            g.record_control(g.object_ref(basic), p.other(), None);
            checked(&mut g, Action::Pass);
            settle(&mut g);
        }
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[aura as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn tmt_exile_time_optional_apnap_shuffle_privacy_and_self_exile() {
    for accept in [[false, false], [true, false], [false, true], [true, true]] {
        let p = Player::P0;
        let mut g = setup(p);
        let own = paid(&mut g, p, "Llanowar Elves");
        let theirs = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let grave = named(&mut g, p, "Island", Zone::Graveyard);
        let hidden = named(&mut g, p.other(), "Plains", Zone::Hand);
        let spell = named(&mut g, p, "Turtles in Time", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        until(&mut g, |g| matches!(g.state.pending, Pending::Wheel { .. }));
        assert_eq!(g.state.objects[own as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[theirs as usize].zone, Zone::Hand);
        for player in [p, p.other()] {
            assert_eq!(g.actor(), Some(player));
            checked(
                &mut g,
                Action::ChooseOptional {
                    accept: accept[player.index()],
                },
            );
        }
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Exile);
        for player in [p, p.other()] {
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.owner == player && o.zone == Zone::Hand)
                    .count(),
                if accept[player.index()] {
                    7
                } else if player == p {
                    1
                } else {
                    2
                }
            );
        }
        if !accept[0] {
            assert_eq!(g.state.objects[grave as usize].zone, Zone::Graveyard);
        }
        if !accept[1] {
            assert_eq!(g.state.objects[hidden as usize].zone, Zone::Hand);
        }
        for state in &g.state.players {
            assert_eq!(state.cards_discarded_this_turn, 0);
        }
        assert!(g.observe(p).last_target_library_move.is_none());
        assert!(g.observe(p).known_library_top.iter().all(Option::is_none));
        if accept[1] {
            assert!(
                !g.observe(p)
                    .objects
                    .iter()
                    .any(|o| o.reference.card == hidden)
            );
        }
    }
}

#[test]
fn tmt_exile_escape_actually_prevents_blocks_after_power_increases() {
    let p = Player::P0;
    let mut g = setup(p);
    let attacker = paid(&mut g, p, "Llanowar Elves");
    mature(&mut g);
    let tunnel = land(&mut g, p, "Escape Tunnel");
    let blocker = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
    activate(&mut g, tunnel, 1, Some(attacker));
    settle(&mut g);
    g.state.objects[attacker as usize].counters.plus_one = 3;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(&mut g, Action::Attack { card: attacker });
    checked(&mut g, Action::FinishAttackers);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert_eq!(g.state.pending, Pending::Blockers);
    reject(&mut g, Action::Block { blocker, attacker });
    checked(&mut g, Action::FinishBlockers);
}
#[test]
fn tmt_exile_farm_new_copied_ability_family_cannot_take_old_links() {
    let p = Player::P0;
    let mut g = setup(p);
    let farm = land(&mut g, p, "Northampton Farm");
    let creature = paid(&mut g, p, "Llanowar Elves");
    activate(&mut g, farm, 1, Some(creature));
    settle(&mut g);
    let object = g.object_ref(farm);
    let recipe = CopyRecipe::new(g.state.objects[farm as usize].card);
    g.copy_permanents_exact(&[object], recipe);
    assert_ne!(g.copy_ability_origin(object), 0);
    g.state.objects[farm as usize].tapped = false;
    activate(&mut g, farm, 2, None);
    settle(&mut g);
    assert_eq!(g.state.objects[creature as usize].zone, Zone::Exile);
}
#[test]
fn tmt_exile_farm_stale_exile_reentry_is_not_linked() {
    let p = Player::P0;
    let mut g = setup(p);
    let farm = land(&mut g, p, "Northampton Farm");
    let creature = paid(&mut g, p, "Llanowar Elves");
    activate(&mut g, farm, 1, Some(creature));
    settle(&mut g);
    let old = g.object_ref(creature);
    g.move_object(creature, Zone::Graveyard);
    g.move_object(creature, Zone::Exile);
    assert_ne!(g.object_ref(creature), old);
    g.state.objects[farm as usize].tapped = false;
    activate(&mut g, farm, 2, None);
    settle(&mut g);
    assert_eq!(g.state.objects[creature as usize].zone, Zone::Exile);
}
#[test]
fn tmt_exile_renet_uses_current_land_status_and_remembers_land_entries() {
    let p = Player::P0;
    let mut g = setup(p);
    let land = land(&mut g, p, "Forest");
    let artifact = paid(&mut g, p, "Mazemind Tome");
    let land_ref = g.object_ref(land);
    let artifact_ref = g.object_ref(artifact);
    g.copy_permanents_exact(
        &[land_ref],
        CopyRecipe::new(Card::from_name("Mazemind Tome").unwrap()),
    );
    g.copy_permanents_exact(&[artifact_ref], CopyRecipe::new(Card::Forest));
    paid(&mut g, p, "Renet, Temporal Apprentice");
    assert_eq!(g.state.objects[land as usize].zone, Zone::Hand);
    assert_eq!(g.state.objects[artifact as usize].zone, Zone::Battlefield);
}
#[test]
fn tmt_exile_renet_new_incarnation_is_other_than_old_trigger_source() {
    let p = Player::P0;
    let mut g = setup(p);
    let renet = named(&mut g, p, "Renet, Temporal Apprentice", Zone::Hand);
    cast(&mut g, renet, None, &[]);
    until(&mut g, |g| g.on_battlefield(renet));
    let old = g.object_ref(renet);
    g.move_object(renet, Zone::Exile);
    g.move_object(renet, Zone::Battlefield);
    assert_ne!(g.object_ref(renet), old);
    settle(&mut g);
    assert_eq!(g.state.objects[renet as usize].zone, Zone::Hand);
}
#[test]
fn tmt_exile_dimensional_basic_status_is_live_attachment_legality() {
    let p = Player::P0;
    let mut g = setup(p);
    let basic = land(&mut g, p, "Forest");
    let enemy = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
    let aura = named(&mut g, p, "Dimensional Exile", Zone::Hand);
    cast(&mut g, aura, None, &[Target::Card(basic)]);
    super::ability_timeline_tests::settle_on(&mut g, Some(Target::Card(enemy)));
    let basic_ref = g.object_ref(basic);
    g.copy_permanents_exact(
        &[basic_ref],
        CopyRecipe::new(Card::from_name("Escape Tunnel").unwrap()),
    );
    checked(&mut g, Action::Pass);
    settle(&mut g);
    assert_eq!(g.state.objects[aura as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[enemy as usize].zone, Zone::Battlefield);
}
#[test]
fn tmt_exile_time_countered_spell_neither_bounces_nor_exiles_itself() {
    let p = Player::P0;
    let mut g = setup(p);
    let creature = paid(&mut g, p, "Llanowar Elves");
    let spell = named(&mut g, p, "Turtles in Time", Zone::Hand);
    let counter = named(&mut g, p.other(), "Negate", Zone::Hand);
    cast(&mut g, spell, None, &[]);
    checked(&mut g, Action::Pass);
    cast(&mut g, counter, None, &[Target::Spell(spell)]);
    settle(&mut g);
    assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[creature as usize].zone, Zone::Battlefield);
}
#[test]
fn tmt_exile_entry_history_footprint_and_query_measurement() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert_eq!(std::mem::size_of::<CastInfo>(), 40);
    assert_eq!(std::mem::size_of::<Pending>(), 40);
    let p = Player::P0;
    let mut g = setup(p);
    let before = g.state_bytes();
    paid(&mut g, p, "Renet, Temporal Apprentice");
    let bytes = g.state_bytes();
    let start = std::time::Instant::now();
    for _ in 0..100 {
        std::hint::black_box(g.legal_actions());
    }
    eprintln!(
        "TMT Renet100legalqueries {:?}; initialized bytes {}; entry rows {}; initial delta {}",
        start.elapsed(),
        bytes,
        g.state.permanent_entry_history.as_ref().unwrap().len(),
        bytes - before
    );
    assert_eq!(g.state_bytes(), bytes);
    next_turn(&mut g);
    assert!(g.state.permanent_entry_history.is_none());
}

#[test]
fn tmt_exile_farm_exiled_tokens_cease_and_cannot_be_returned() {
    let p = Player::P0;
    let mut g = setup(p);
    let farm = land(&mut g, p, "Northampton Farm");
    let token = g.state.objects.len() as CardId;
    g.create_tokens(
        p,
        Card::from_name("@token/283fd83a9aec83a51b736926").unwrap(),
        1,
        false,
    );
    let object = g.object_ref(token);
    g.copy_permanents_exact(
        &[object],
        CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
    );
    activate(&mut g, farm, 1, Some(token));
    settle(&mut g);
    assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
    g.state.objects[farm as usize].tapped = false;
    activate(&mut g, farm, 2, None);
    settle(&mut g);
    assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
}
#[test]
fn tmt_exile_time_accepting_empty_hand_and_graveyard_still_shuffles_and_draws() {
    let p = Player::P0;
    let mut g = setup(p);
    let spell = named(&mut g, p, "Turtles in Time", Zone::Hand);
    cast(&mut g, spell, None, &[]);
    until(&mut g, |g| matches!(g.state.pending, Pending::Wheel { .. }));
    let before = g.state.rng;
    checked(&mut g, Action::ChooseOptional { accept: true });
    checked(&mut g, Action::ChooseOptional { accept: false });
    settle(&mut g);
    assert_ne!(g.state.rng, before);
    assert_eq!(g.observe(p).players[p.index()].hand_count, 7);
    assert_eq!(g.state.objects[spell as usize].zone, Zone::Exile);
}
