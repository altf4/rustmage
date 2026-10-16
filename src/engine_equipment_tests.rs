use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn permanent(g: &mut Game, name: &str) -> CardId {
    named(g, Player::P0, name, Zone::Battlefield)
}
fn attach(g: &mut Game, equipment: CardId, creature: CardId) {
    assert!(g.attach_equipment(g.object_ref(equipment), g.object_ref(creature)));
}
fn replay(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, action.clone());
    act(g, action.clone());
    assert_eq!(g.snapshot(), fork.snapshot());
    let after = g.snapshot();
    g.restore(&before);
    act(g, action);
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision;
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision, revision);
}
fn equip_action(g: &mut Game, equipment: CardId, creature: CardId) -> Action {
    let index = g.state.objects[equipment as usize]
        .card
        .activations()
        .iter()
        .position(|a| a.program.contains(&Instruction::AttachSource))
        .unwrap();
    let cost = g.state.objects[equipment as usize].card.activations()[index].cost;
    let mut payment = cost.colored;
    payment[5] += cost.generic;
    g.state.players[0].mana = payment;
    Action::Activate {
        card: equipment,
        ability: index as u16,
        target: Some(Target::Card(creature)),
        payment,
        counter_payment: Counters::default(),
    }
}
fn resolve_all(g: &mut Game) {
    g.prepare_triggers();
    while !g.state.stack.is_empty() || !g.state.ordering_triggers.is_empty() {
        if let Some(action) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
        {
            replay(g, action);
        } else {
            replay(g, Action::Pass);
            replay(g, Action::Pass);
        }
    }
}
fn cast_permanent(g: &mut Game, name: &str) -> CardId {
    let card = named(g, Player::P0, name, Zone::Hand);
    let (mut mana, generic) = g.state.objects[card as usize].card.cost();
    mana[5] += generic;
    g.state.players[0].mana = mana;
    cast(g, card, None);
    pass_pair(g);
    card
}

#[test]
fn equip_is_a_paid_targeted_stack_ability_and_attachment_is_public_owned_state() {
    let mut g = scenario();
    let bow = permanent(&mut g, "Short Bow");
    let bear = permanent(&mut g, "Grizzly Bears");
    let action = equip_action(&mut g, bow, bear);
    replay(&mut g, action.clone());
    assert_eq!(g.object_view(bow).attached_to, None);
    assert_eq!(g.stats(bear).power, 2);
    assert!(matches!(
        g.state.stack[0].kind,
        StackKind::ActivatedAbility { .. }
    ));
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.object_view(bow).attached_to, Some(g.object_ref(bear)));
    assert_eq!(
        g.stats(bear),
        Stats {
            power: 3,
            toughness: 3
        }
    );
    assert!(
        g.keywords(bear)
            .contains(Keywords::REACH.union(Keywords::VIGILANCE))
    );
    for player in [Player::P0, Player::P1] {
        let view = g.observe(player);
        assert_eq!(view.schema, 35);
        assert_eq!(
            view.objects
                .iter()
                .find(|o| o.id == bow)
                .unwrap()
                .attached_to,
            Some(g.object_ref(bear))
        );
    }
    // A second activation to the same creature remains legal; the smoke policy
    // merely avoids repeating it indefinitely.
    let action = equip_action(&mut g, bow, bear);
    assert!(g.legal_actions().contains(&action));
    replay(&mut g, action);
    pass_pair(&mut g);
    assert_eq!(g.stats(bear).power, 3);
}

#[test]
fn equip_rejects_enemy_noncreature_and_out_of_sorcery_timing_atomically() {
    let mut g = scenario();
    let sword = permanent(&mut g, "Coral Sword"); // Flash doesn't change equip timing.
    let bear = permanent(&mut g, "Grizzly Bears");
    let enemy = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let forest = permanent(&mut g, "Forest");
    let action = equip_action(&mut g, sword, enemy);
    reject(&mut g, action);
    let action = equip_action(&mut g, sword, forest);
    reject(&mut g, action);
    let action = equip_action(&mut g, sword, bear);
    g.state.step = Step::BeginCombat;
    reject(&mut g, action.clone());
    g.state.step = Step::Main1;
    g.state.active = Player::P1;
    g.state.priority = Player::P0;
    reject(&mut g, action);
    assert!(!g.attach_equipment(g.object_ref(sword), g.object_ref(sword)));
}

#[test]
fn bonuses_stack_move_with_equipment_and_persist_across_cleanup_and_turns() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Grizzly Bears");
    let other = permanent(&mut g, "Hill Giant");
    let sword = permanent(&mut g, "Sword of Vengeance");
    let bow = permanent(&mut g, "Short Bow");
    attach(&mut g, sword, bear);
    attach(&mut g, bow, bear);
    assert_eq!(
        g.stats(bear),
        Stats {
            power: 5,
            toughness: 3
        }
    );
    let action = equip_action(&mut g, sword, other);
    replay(&mut g, action);
    pass_pair(&mut g);
    assert_eq!(
        g.stats(bear),
        Stats {
            power: 3,
            toughness: 3
        }
    );
    assert_eq!(
        g.stats(other),
        Stats {
            power: 5,
            toughness: 3
        }
    );
    assert!(!g.keywords(bear).contains(Keywords::HASTE));
    g.finish_cleanup();
    assert_eq!(g.stats(other).power, 5);
    assert_eq!(g.object_view(sword).attached_to, Some(g.object_ref(other)));
    assert_eq!(g.object_view(sword).used_abilities, 0);
}

#[test]
fn failed_target_does_not_move_an_existing_attachment_or_refund_the_cost() {
    let mut g = scenario();
    let bow = permanent(&mut g, "Short Bow");
    let a = permanent(&mut g, "Grizzly Bears");
    let b = permanent(&mut g, "Hill Giant");
    attach(&mut g, bow, a);
    let action = equip_action(&mut g, bow, b);
    replay(&mut g, action);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    g.move_object(b, Zone::Hand);
    g.move_object(b, Zone::Battlefield);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.object_view(bow).attached_to, Some(g.object_ref(a)));
    assert_eq!(g.stats(b).power, 3);
}

#[test]
fn departed_and_returned_equipment_cannot_attach_from_an_old_activation() {
    let mut g = scenario();
    let bow = permanent(&mut g, "Short Bow");
    let bear = permanent(&mut g, "Grizzly Bears");
    let action = equip_action(&mut g, bow, bear);
    replay(&mut g, action);
    g.move_object(bow, Zone::Hand);
    g.move_object(bow, Zone::Battlefield);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.object_view(bow).attached_to, None);
    assert_eq!(g.stats(bear).power, 2);
}

#[test]
fn lost_creature_detaches_by_sba_and_returning_incarnation_never_inherits_bonuses() {
    let mut g = scenario();
    let bow = permanent(&mut g, "Short Bow");
    let bear = permanent(&mut g, "Grizzly Bears");
    attach(&mut g, bow, bear);
    let old = g.object_ref(bear);
    g.move_object(bear, Zone::Hand);
    g.move_object(bear, Zone::Battlefield);
    assert_eq!(g.object_view(bow).attached_to, Some(old));
    assert_eq!(g.stats(bear).power, 2);
    g.state_based_actions();
    assert_eq!(g.object_view(bow).attached_to, None);
    assert_eq!(g.object_view(bow).zone, Zone::Battlefield);
}

#[test]
fn entry_attachment_targets_then_temporary_grants_survive_source_departure() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Grizzly Bears");
    let armor = cast_permanent(&mut g, "Celestial Armor");
    assert_eq!(g.observe(Player::P0).decision, "targets");
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    g.move_object(armor, Zone::Graveyard);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.stats(bear).power, 2);
    assert!(!g.keywords(bear).contains(Keywords::FLYING));
    assert!(
        g.keywords(bear)
            .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
    );
    g.finish_cleanup();
    assert!(!g.keywords(bear).contains(Keywords::HEXPROOF));
}

#[test]
fn equipment_enters_normally_without_a_creature_and_stays_unattached() {
    let mut g = scenario();
    let sword = cast_permanent(&mut g, "Coral Sword");
    assert_eq!(g.object_view(sword).zone, Zone::Battlefield);
    assert_eq!(g.object_view(sword).attached_to, None);
    assert!(g.state.stack.is_empty() && g.state.announcing.is_none());
    assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate { card, .. } | Action::AnnounceActivate { card, .. } if *card==sword)));
}

#[test]
fn continuous_keywords_drive_hexproof_blocking_and_earlier_layer_lord_filters() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Grizzly Bears");
    let armor = permanent(&mut g, "Celestial Armor");
    let boots = permanent(&mut g, "Swiftfoot Boots");
    let eagle = permanent(&mut g, "Empyrean Eagle");
    let blocker = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    attach(&mut g, armor, bear);
    attach(&mut g, boots, bear);
    assert_eq!(
        g.stats(bear),
        Stats {
            power: 5,
            toughness: 3
        }
    );
    assert!(!g.can_block_pair(blocker, bear));
    assert!(!g.target_matches(
        TargetRule::Creature,
        Player::P1,
        g.object_ref(blocker),
        Target::Card(bear)
    ));
    assert!(g.target_matches(
        TargetRule::Creature,
        Player::P0,
        g.object_ref(boots),
        Target::Card(bear)
    ));
    g.move_object(armor, Zone::Graveyard);
    assert_eq!(
        g.stats(bear),
        Stats {
            power: 2,
            toughness: 2
        }
    );
    assert!(g.can_block_pair(blocker, bear));
    g.move_object(boots, Zone::Graveyard);
    assert!(g.target_matches(
        TargetRule::Creature,
        Player::P1,
        g.object_ref(blocker),
        Target::Card(bear)
    ));
    assert_eq!(g.stats(eagle).power, 2);
}

#[test]
fn haste_controls_attacks_intrinsic_mana_and_activated_mana_funding() {
    for name in ["Llanowar Elves", "Druid of the Cowl"] {
        let mut g = scenario();
        let elf = permanent(&mut g, name);
        g.state.objects[elf as usize].summoning_sick = true;
        let boots = permanent(&mut g, "Swiftfoot Boots");
        let spell = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceCast { card } if *card==spell))
        );
        attach(&mut g, boots, elf);
        assert!(
            g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceCast { card } if *card==spell))
        );
        replay(&mut g, Action::AnnounceCast { card: spell });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(elf),
            },
        );
        let mana = g.legal_actions().into_iter().find(|a|matches!(a,Action::TapMana {card} | Action::ActivateMana {card,..} if *card==elf)).unwrap();
        replay(&mut g, mana);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, pay);
        pass_pair(&mut g);
        g.state.objects[elf as usize].tapped = false;
        g.state.pending = Pending::Attackers;
        assert!(g.legal_actions().contains(&Action::Attack { card: elf }));
        g.move_object(boots, Zone::Graveyard);
        assert!(!g.legal_actions().contains(&Action::Attack { card: elf }));
    }
}

#[test]
fn granted_vigilance_first_strike_trample_deathtouch_and_lifelink_apply_to_combat() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Grizzly Bears");
    let sword = permanent(&mut g, "Sword of Vengeance");
    let collar = permanent(&mut g, "Basilisk Collar");
    let blocker = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    attach(&mut g, sword, bear);
    attach(&mut g, collar, bear);
    g.state.objects[bear as usize].summoning_sick = true;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(&mut g, Action::Attack { card: bear });
    replay(&mut g, Action::FinishAttackers);
    assert!(!g.object_view(bear).tapped);
    // Commit one legal blocker, then assign one deathtouch damage and trample.
    pass_pair(&mut g);
    replay(
        &mut g,
        Action::Block {
            blocker,
            attacker: bear,
        },
    );
    replay(&mut g, Action::FinishBlockers);
    pass_pair(&mut g);
    if matches!(g.state.pending, Pending::Damage) {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::AssignDamage { amount: 1, .. }))
            .unwrap();
        replay(&mut g, action);
    }
    assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].life, 24);
    assert_eq!(g.state.players[1].life, 17);
    assert_eq!(g.object_view(bear).damage, 0);
}

#[test]
fn losing_double_strike_between_damage_steps_prevents_second_assignment() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Grizzly Bears");
    let weapon = permanent(&mut g, "Fireshrieker");
    attach(&mut g, weapon, bear);
    g.state.attackers.push(Attacker {
        card: bear,
        blocked: false,
    });
    g.state.step = Step::Blockers;
    g.give_priority();
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::FirstStrikeDamage);
    assert_eq!(g.state.players[1].life, 18);
    g.move_object(weapon, Zone::Graveyard);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::CombatDamage);
    assert_eq!(g.state.players[1].life, 18);
}

#[test]
fn simultaneous_departures_preserve_pre_batch_keywords_for_existing_and_new_triggers() {
    let mut g = scenario();
    let reaper = permanent(&mut g, "Midnight Reaper");
    let collar = permanent(&mut g, "Basilisk Collar");
    permanent(&mut g, "Marauding Blight-Priest");
    attach(&mut g, collar, reaper);
    g.queue_trigger(&g.object_view(reaper), 0);
    let sources = g.death_trigger_sources();
    let events = g.move_objects_raw(&[collar, reaper], Zone::Graveyard);
    g.finish_zone_events(&events, &sources);
    assert_eq!(g.state.pending_triggers.len(), 2);
    assert!(
        g.state
            .pending_triggers
            .iter()
            .all(|t| t.source_keywords.contains(Keywords::LIFELINK))
    );
    assert!(events[1].before.keywords.contains(Keywords::LIFELINK));
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].life, 20);
    assert_eq!(g.state.players[0].library.len(), 18);
    assert_eq!(g.state.players[1].life, 18);
}

#[test]
fn frozen_detachment_waits_for_legend_choices_and_replays_the_whole_sba_event() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Grizzly Bears");
    let bow = permanent(&mut g, "Short Bow");
    attach(&mut g, bow, bear);
    g.move_object(bear, Zone::Hand);
    let keep = permanent(&mut g, "Tigra, Feline Fury");
    permanent(&mut g, "Tigra, Feline Fury");
    g.state_based_actions();
    assert!(matches!(g.state.pending, Pending::Legend { .. }));
    assert!(g.object_view(bow).attached_to.is_some());
    let object = g.object_ref(keep);
    replay(&mut g, Action::KeepLegend { object });
    assert_eq!(g.object_view(bow).attached_to, None);
}

#[test]
fn equipment_source_index_and_links_reset_on_every_admitted_zone_transition() {
    let mut g = scenario();
    for &card in &Card::ALL {
        let id = spawn(&mut g, Player::P0, card, Zone::Battlefield);
        let equipped = card.attachment_bonus().is_some();
        assert_eq!(
            g.state.attachment_sources.contains(&id),
            equipped,
            "{}",
            card.name()
        );
        let saved = g.snapshot();
        g.move_object(id, Zone::Exile);
        assert!(!g.state.attachment_sources.contains(&id));
        assert_eq!(g.object_view(id).attached_to, None);
        g.restore(&saved);
        assert_eq!(g.state.attachment_sources.contains(&id), equipped);
        g.move_object(id, Zone::Exile);
    }
    assert!(g.state.attachment_sources.is_empty());
}
