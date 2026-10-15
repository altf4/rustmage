use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn spell(g: &mut Game, name: &str, target: Option<Target>) -> CardId {
    let actor = g.actor().unwrap();
    let card = named(g, actor, name, Zone::Hand);
    let (mut mana, generic) = g.state.objects[card as usize].card.cost();
    mana[5] += generic;
    g.state.players[actor.index()].mana = mana;
    cast(g, card, target);
    pass_pair(g);
    card
}

#[test]
fn hexproof_revalidates_opposing_spell_and_keeps_own_targets_legal() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let removal = named(&mut g, Player::P1, "Murder", Zone::Hand);
    let protect = named(&mut g, Player::P0, "Shore Up", Zone::Hand);
    g.state.priority = Player::P1;
    g.state.players[1].mana = [0, 0, 3, 0, 0, 0];
    cast(&mut g, removal, Some(Target::Card(bear)));
    act(&mut g, Action::Pass);
    g.state.players[0].mana[1] = 1;
    cast(&mut g, protect, Some(Target::Card(bear)));
    let saved = g.snapshot();
    pass_pair(&mut g);
    assert!(g.object_view(bear).keywords.contains(Keywords::HEXPROOF));
    assert!(!g.target_matches(
        TargetRule::Creature,
        Player::P1,
        g.object_ref(removal),
        Target::Card(bear)
    ));
    assert!(g.target_matches(
        TargetRule::Creature,
        Player::P0,
        g.object_ref(protect),
        Target::Card(bear)
    ));
    let guard = g.snapshot();
    let priority = g.state.priority;
    g.state.priority = Player::P1;
    let illegal = named(&mut g, Player::P1, "Murder", Zone::Hand);
    g.state.players[1].mana = [0, 0, 3, 0, 0, 0];
    let state = g.state.clone();
    let revision = g.revision();
    assert_eq!(
        g.apply(
            Player::P1,
            Action::Cast {
                card: illegal,
                target: Some(Target::Card(bear)),
                payment: [0, 0, 3, 0, 0, 0]
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.state, state);
    assert_eq!(g.revision(), revision);
    // Continue the original pending removal with the fixture restored.
    g.state.priority = priority;
    g.restore(&guard);
    pass_pair(&mut g);
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[removal as usize].zone, Zone::Graveyard);
    let result = g.state.clone();
    g.restore(&saved);
    pass_pair(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.state, result);
    g.finish_cleanup();
    assert!(!g.object_view(bear).keywords.contains(Keywords::HEXPROOF));
    assert!(g.target_matches(
        TargetRule::Creature,
        Player::P1,
        g.object_ref(removal),
        Target::Card(bear)
    ));
}

#[test]
fn indestructible_prevents_destroy_and_damage_but_not_zero_toughness_or_sacrifice() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    spell(&mut g, "Horrid Vigor", Some(Target::Card(bear)));
    assert!(
        g.object_view(bear)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );
    spell(&mut g, "Murder", Some(Target::Card(bear)));
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
    g.state.objects[bear as usize].damage = 8;
    g.state.objects[bear as usize].deathtouch_since_sba = true;
    g.state_based_actions();
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
    assert!(!g.state.objects[bear as usize].deathtouch_since_sba);
    spell(&mut g, "Last Gasp", Some(Target::Card(bear)));
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
    let firebrand = named(&mut g, Player::P0, "Fanatical Firebrand", Zone::Battlefield);
    g.state.objects[firebrand as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..} if *card==firebrand))
        .unwrap();
    act(&mut g, action);
    assert_eq!(g.state.objects[firebrand as usize].zone, Zone::Graveyard);
}

#[test]
fn cleanup_expires_indestructible_and_removes_damage_simultaneously() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[bear as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    g.state.objects[bear as usize].damage = 10;
    g.state_based_actions();
    g.finish_cleanup();
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[bear as usize].damage, 0);
    assert!(
        !g.object_view(bear)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );
}

#[test]
fn granted_haste_enables_sick_mana_source_and_exact_payment_search() {
    let mut g = scenario();
    let elf = named(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    g.state.objects[elf as usize].summoning_sick = true;
    let growth = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceCast{card} if *card==growth))
    );
    spell(&mut g, "Impolite Entrance", Some(Target::Card(elf)));
    g.state.players[0].mana = [0; 6];
    assert!(
        g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::TapMana{card} if *card==elf))
    );
    assert!(
        g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceCast{card} if *card==growth))
    );
    act(&mut g, Action::AnnounceCast { card: growth });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(elf),
        },
    );
    act(&mut g, Action::TapMana { card: elf });
    act(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    pass_pair(&mut g);
    assert_eq!(g.object_view(elf).power, 4);
}

#[test]
fn funding_canonicalization_distinguishes_same_card_with_different_gains() {
    let mut g = scenario();
    let first = named(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let second = named(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    for id in [first, second] {
        g.state.objects[id as usize].summoning_sick = true;
    }
    g.state.objects[first as usize].gained_keywords = Keywords::HASTE;
    assert!(g.can_fund(FundingGoal {
        actor: Player::P0,
        required: [0, 0, 0, 0, 1, 0],
        generic: 0,
        source: None
    }));
    assert!(!g.can_fund(FundingGoal {
        actor: Player::P0,
        required: [0, 0, 0, 0, 2, 0],
        generic: 0,
        source: None
    }));
    let saved = g.snapshot();
    g.state.objects[second as usize].gained_keywords = Keywords::HASTE;
    assert!(g.can_fund(FundingGoal {
        actor: Player::P0,
        required: [0, 0, 0, 0, 2, 0],
        generic: 0,
        source: None
    }));
    g.restore(&saved);
    assert!(!g.can_fund(FundingGoal {
        actor: Player::P0,
        required: [0, 0, 0, 0, 2, 0],
        generic: 0,
        source: None
    }));
}

#[test]
fn granted_flying_changes_blocking_and_layer7_flying_anthem_immediately() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    named(&mut g, Player::P0, "Empyrean Eagle", Zone::Battlefield);
    let ground = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    spell(&mut g, "Acrobatic Leap", Some(Target::Card(bear)));
    assert_eq!(
        (g.object_view(bear).power, g.object_view(bear).toughness),
        (4, 6)
    );
    g.state.attackers.push(Attacker {
        card: bear,
        blocked: false,
    });
    g.state.pending = Pending::Blockers;
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::Block{blocker,..} if *blocker==ground))
    );
    g.state.objects[ground as usize].gained_keywords = Keywords::REACH;
    assert!(
        g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::Block{blocker,..} if *blocker==ground))
    );
}

#[test]
fn mass_grants_lock_new_tokens_and_existing_set_then_reset_on_zone_change() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    spell(&mut g, "Heroic Reinforcements", None);
    let soldiers: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.card.is_token() && o.zone == Zone::Battlefield)
        .map(|(id, _)| id as CardId)
        .collect();
    assert_eq!(soldiers.len(), 2);
    assert!(
        soldiers
            .iter()
            .all(|&id| g.object_view(id).keywords.contains(Keywords::HASTE))
    );
    assert!(g.object_view(bear).keywords.contains(Keywords::HASTE));
    let later = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    assert!(!g.object_view(later).keywords.contains(Keywords::HASTE));
    g.move_object(bear, Zone::Hand);
    g.move_object(bear, Zone::Battlefield);
    assert!(!g.object_view(bear).keywords.contains(Keywords::HASTE));
}

#[test]
fn granted_lifelink_and_deathtouch_use_current_damage_source_characteristics() {
    let mut g = scenario();
    let attacker = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let blocker = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    spell(&mut g, "Toxin Analysis", Some(Target::Card(attacker)));
    g.state.attackers.push(Attacker {
        card: attacker,
        blocked: true,
    });
    g.state.blocking.push(blocker);
    g.state.blocks.push((blocker, attacker));
    g.state.step = Step::CombatDamage;
    g.prepare_damage();
    g.state_based_actions();
    assert_eq!(g.state.objects[blocker as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].life, 22);
}

#[test]
fn granted_double_strike_gets_both_strike_steps() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let source = named(&mut g, Player::P0, "Terror of Mount Velus", Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    g.prepare_triggers();
    pass_pair(&mut g);
    assert!(
        g.object_view(bear)
            .keywords
            .contains(Keywords::DOUBLE_STRIKE)
    );
    g.state.attackers.push(Attacker {
        card: bear,
        blocked: false,
    });
    g.state.step = Step::Blockers;
    g.advance_step();
    assert_eq!(g.state.step, Step::FirstStrikeDamage);
    assert_eq!(g.state.players[1].life, 18);
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::CombatDamage);
    assert_eq!(g.state.players[1].life, 16);
}

#[test]
fn ability_uses_keywords_gained_after_announcement_before_source_departure() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Deadeye Duelist", Zone::Battlefield);
    g.state.players[0].mana[5] = 1;
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..} if *card==source))
        .unwrap();
    act(&mut g, action);
    spell(&mut g, "Give In to Violence", Some(Target::Card(source)));
    assert!(g.object_view(source).keywords.contains(Keywords::LIFELINK));
    spell(&mut g, "Unsummon", Some(Target::Card(source)));
    assert_eq!(g.state.objects[source as usize].zone, Zone::Hand);
    assert!(
        g.state.stack[0]
            .source_keywords
            .contains(Keywords::LIFELINK)
    );
    let saved = g.snapshot();
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 21);
    assert_eq!(g.state.players[1].life, 19);
    let result = g.state.clone();
    g.restore(&saved);
    pass_pair(&mut g);
    assert_eq!(g.state, result);
}

#[test]
fn granted_vigilance_preserves_untapped_attack_and_source_grants_reset() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    spell(&mut g, "Preposterous Proportions", None);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: bear });
    act(&mut g, Action::FinishAttackers);
    assert!(!g.state.objects[bear as usize].tapped);
    g.state.attackers.clear();
    g.state.pending = Pending::Priority;
    g.state.step = Step::Main2;
    let frog = named(&mut g, Player::P0, "Frog Butler", Zone::Battlefield);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..} if *card==frog))
        .unwrap();
    act(&mut g, action);
    pass_pair(&mut g);
    assert!(g.object_view(frog).keywords.contains(Keywords::REACH));
    g.move_object(frog, Zone::Hand);
    g.move_object(frog, Zone::Battlefield);
    assert!(!g.object_view(frog).keywords.contains(Keywords::REACH));
}
