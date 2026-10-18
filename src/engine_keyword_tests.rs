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
    g.restore(&guard).unwrap();
    pass_pair(&mut g);
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[removal as usize].zone, Zone::Graveyard);
    let result = g.state.clone();
    g.restore(&saved).unwrap();
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
        reserve: None,
        actor: Player::P0,
        required: [0, 0, 0, 0, 1, 0],
        generic: 0,
        source: None
    }));
    assert!(!g.can_fund(FundingGoal {
        reserve: None,
        actor: Player::P0,
        required: [0, 0, 0, 0, 2, 0],
        generic: 0,
        source: None
    }));
    let saved = g.snapshot();
    g.state.objects[second as usize].gained_keywords = Keywords::HASTE;
    assert!(g.can_fund(FundingGoal {
        reserve: None,
        actor: Player::P0,
        required: [0, 0, 0, 0, 2, 0],
        generic: 0,
        source: None
    }));
    g.restore(&saved).unwrap();
    assert!(!g.can_fund(FundingGoal {
        reserve: None,
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
        defender: Some(CombatDefender::Player(Player::P1)),
        card: bear,
        blocked: false,
    });
    g.state.pending = Pending::Blockers;
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::Block{blocker,..} if *blocker==ground))
    );
    g.grant_keyword_effects(&[ground], g.object_ref(ground), Keywords::REACH);
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
        defender: Some(CombatDefender::Player(Player::P1)),
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
        defender: Some(CombatDefender::Player(Player::P1)),
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
    g.restore(&saved).unwrap();
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

#[test]
fn continuous_grants_follow_owner_zone_incarnation_and_snapshot() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let hand = named(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    let samut = named(
        &mut g,
        Player::P0,
        "Samut, Hazoret's Champion",
        Zone::Battlefield,
    );
    for id in [bear, samut] {
        assert!(g.keywords(id).contains(Keywords::HASTE));
    }
    for id in [enemy, hand] {
        assert!(!g.keywords(id).contains(Keywords::HASTE));
    }
    let saved = g.snapshot();
    let reference = g.object_ref(samut);
    g.move_object(samut, Zone::Hand);
    assert!(!g.keywords(bear).contains(Keywords::HASTE));
    assert!(!g.keywords(samut).contains(Keywords::HASTE));
    g.move_object(samut, Zone::Battlefield);
    assert_ne!(g.object_ref(samut), reference);
    assert!(g.keywords(bear).contains(Keywords::HASTE));
    g.restore(&saved).unwrap();
    assert_eq!(g.object_ref(samut), reference);
    assert!(g.keywords(bear).contains(Keywords::HASTE));
}

#[test]
fn own_turn_first_strike_is_derived_and_changeling_stays_intrinsic() {
    let mut g = scenario();
    for name in [
        "Bearer of Glory",
        "Feisty Spikeling",
        "Null Group Biological Assets",
        "Skilled Battlecarver",
    ] {
        let id = named(&mut g, Player::P0, name, Zone::Battlefield);
        let enemy = named(&mut g, Player::P1, name, Zone::Battlefield);
        assert!(g.keywords(id).contains(Keywords::FIRST_STRIKE));
        assert!(!g.keywords(enemy).contains(Keywords::FIRST_STRIKE));
        let saved = g.snapshot();
        g.state.active = Player::P1;
        assert!(!g.keywords(id).contains(Keywords::FIRST_STRIKE));
        assert!(g.keywords(enemy).contains(Keywords::FIRST_STRIKE));
        g.restore(&saved).unwrap();
        assert!(g.keywords(id).contains(Keywords::FIRST_STRIKE));
        g.move_object(id, Zone::Graveyard);
        assert!(!g.keywords(id).contains(Keywords::FIRST_STRIKE));
        assert_eq!(
            g.keywords(id).contains(Keywords::CHANGELING),
            name == "Feisty Spikeling"
        );
    }
}

#[test]
fn tribal_stats_and_lifelink_apply_before_damage_and_capture_departing_batch() {
    let mut g = scenario();
    let lord = named(&mut g, Player::P0, "Regal Caracal", Zone::Hand);
    g.move_object(lord, Zone::Battlefield);
    g.prepare_triggers();
    pass_pair(&mut g);
    let cats: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield && o.card.is_token() && o.card.has_subtype("Cat")
        })
        .map(|(id, _)| id as CardId)
        .collect();
    assert_eq!(cats.len(), 2);
    let changeling = named(&mut g, Player::P0, "Feisty Spikeling", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Feisty Spikeling", Zone::Battlefield);
    assert!(!g.keywords(lord).contains(Keywords::LIFELINK));
    assert!(!g.keywords(enemy).contains(Keywords::LIFELINK));
    for id in cats.iter().copied().chain([changeling]) {
        assert!(g.keywords(id).contains(Keywords::LIFELINK));
        let intrinsic = g.state.objects[id as usize].local_power();
        assert_eq!(g.stats(id).power, intrinsic + 1);
    }
    let power = g.stats(changeling).power;
    g.state.attackers.push(Attacker {
        defender: Some(CombatDefender::Player(Player::P1)),
        card: changeling,
        blocked: false,
    });
    g.state.step = Step::Blockers;
    g.advance_step();
    assert_eq!(g.state.players[0].life, 20 + power);
    assert_eq!(g.state.players[1].life, 20 - power);
    let events = g.move_objects_raw(&[lord, changeling], Zone::Graveyard);
    assert!(events[1].before.keywords.contains(Keywords::LIFELINK));
    assert!(!g.keywords(changeling).contains(Keywords::LIFELINK));
    for id in cats {
        assert_eq!(g.stats(id).power, 1);
        assert!(g.keywords(id).contains(Keywords::LIFELINK));
    }
}

#[test]
fn artifact_keyword_grants_feed_layer_seven_and_indestructible_rules() {
    let mut g = scenario();
    let krang = named(
        &mut g,
        Player::P0,
        "Krang, Utrom Warlord",
        Zone::Battlefield,
    );
    let robot = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Three Tree Mascot", Zone::Battlefield);
    named(&mut g, Player::P0, "Empyrean Eagle", Zone::Battlefield);
    for keyword in [
        Keywords::FLYING,
        Keywords::TRAMPLE,
        Keywords::INDESTRUCTIBLE,
        Keywords::HASTE,
    ] {
        assert!(g.keywords(robot).contains(keyword));
        assert!(!g.keywords(bear).contains(keyword));
        assert!(!g.keywords(enemy).contains(keyword));
    }
    assert_eq!(
        g.stats(robot).power,
        g.state.objects[robot as usize].local_power() + 1
    );
    spell(&mut g, "Murder", Some(Target::Card(robot)));
    assert_eq!(g.object_view(robot).zone, Zone::Battlefield);
    g.state.objects[robot as usize].damage = 100;
    g.state_based_actions();
    assert_eq!(g.object_view(robot).zone, Zone::Battlefield);
    g.move_object(krang, Zone::Hand);
    g.state_based_actions();
    assert_eq!(g.object_view(robot).zone, Zone::Graveyard);
}

#[test]
fn attacking_token_first_strike_recomputes_and_does_not_make_a_second_hit() {
    let mut g = scenario();
    let okoye = named(&mut g, Player::P0, "Okoye, Dora Milaje Leader", Zone::Hand);
    g.move_object(okoye, Zone::Battlefield);
    g.prepare_triggers();
    pass_pair(&mut g);
    let tokens: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.card.is_token())
        .map(|(id, _)| id as CardId)
        .collect();
    assert_eq!(tokens.len(), 2);
    assert!(!g.keywords(tokens[0]).contains(Keywords::FIRST_STRIKE));
    g.state.attackers.push(Attacker {
        defender: Some(CombatDefender::Player(Player::P1)),
        card: tokens[0],
        blocked: false,
    });
    g.state.attackers.push(Attacker {
        defender: Some(CombatDefender::Player(Player::P1)),
        card: okoye,
        blocked: false,
    });
    assert!(g.keywords(tokens[0]).contains(Keywords::FIRST_STRIKE));
    assert!(!g.keywords(tokens[1]).contains(Keywords::FIRST_STRIKE));
    assert!(!g.keywords(okoye).contains(Keywords::FIRST_STRIKE));
    g.state.step = Step::Blockers;
    g.advance_step();
    assert_eq!(g.state.step, Step::FirstStrikeDamage);
    assert_eq!(g.state.players[1].life, 19);
    g.move_object(okoye, Zone::Hand);
    assert!(!g.keywords(tokens[0]).contains(Keywords::FIRST_STRIKE));
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 19);
}

#[test]
fn continuous_tribal_double_strike_gives_changeling_both_damage_steps() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Boldwyr Aggressor", Zone::Battlefield);
    let attacker = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    assert!(g.keywords(attacker).contains(Keywords::DOUBLE_STRIKE));
    g.state.attackers.push(Attacker {
        defender: Some(CombatDefender::Player(Player::P1)),
        card: attacker,
        blocked: false,
    });
    let power = g.stats(attacker).power;
    g.state.step = Step::Blockers;
    g.advance_step();
    assert_eq!(g.state.players[1].life, 20 - power);
    let saved = g.snapshot();
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 20 - 2 * power);
    g.restore(&saved).unwrap();
    g.move_object(source, Zone::Hand);
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 20 - power);
}

#[test]
fn static_grants_preserve_printed_activated_bodies() {
    let mut g = scenario();
    let bearer = named(&mut g, Player::P0, "Bearer of Glory", Zone::Battlefield);
    let carver = named(
        &mut g,
        Player::P0,
        "Skilled Battlecarver",
        Zone::Battlefield,
    );
    for (id, mana) in [(bearer, [1, 0, 0, 0, 0, 4]), (carver, [0, 0, 0, 1, 0, 1])] {
        let before = g.stats(carver);
        g.state.players[0].mana = mana;
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Activate{card,..} if *card==id))
            .unwrap();
        act(&mut g, a);
        pass_pair(&mut g);
        assert_eq!(g.stats(carver).power, before.power + 1);
        assert_eq!(
            g.stats(carver).toughness,
            before.toughness + i128::from(id == bearer)
        );
        assert!(g.keywords(carver).contains(Keywords::FIRST_STRIKE));
    }
}

#[test]
fn expanded_tribal_vocabulary_preserves_temporary_trigger_recipient_sets() {
    for (name, step, keywords) in [
        (
            "Cosmic Spider-Man",
            Step::Main1,
            Keywords::FLYING
                .union(Keywords::FIRST_STRIKE)
                .union(Keywords::TRAMPLE)
                .union(Keywords::LIFELINK)
                .union(Keywords::HASTE),
        ),
        (
            "Vulture, Scheming Scavenger",
            Step::Attackers,
            Keywords::FLYING,
        ),
    ] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let changeling = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        let enemy = named(&mut g, Player::P1, "Three Tree Mascot", Zone::Battlefield);
        let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        g.state.step = step;
        if step == Step::Main1 {
            g.advance_step();
        } else {
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: source });
            act(&mut g, Action::FinishAttackers);
        }
        g.prepare_triggers();
        assert!(!g.state.stack.is_empty() || !g.state.pending_triggers.is_empty());
        pass_pair(&mut g);
        assert!(g.keywords(changeling).contains(keywords));
        assert!(!g.keywords(enemy).contains(Keywords::FLYING));
        assert!(!g.keywords(bear).contains(Keywords::FLYING));
        g.move_object(source, Zone::Hand);
        assert!(g.keywords(changeling).contains(keywords));
        let later = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        assert!(!g.keywords(later).contains(Keywords::FLYING));
    }
}

#[test]
fn every_admitted_battlefield_continuous_source_is_registered_and_removed_on_zone_change() {
    let mut g = scenario();
    let sources: Vec<_> = Card::catalogue()
        // Emblems are command-zone sources and cannot enter the battlefield.
        // Their creation/registration is covered by the emblem scenarios.
        .filter(|c| c.is_supported() && !c.is_emblem() && !c.keyword_grants().is_empty())
        .collect();
    assert!(sources.len() >= 9);
    for card in sources {
        let id = spawn(&mut g, Player::P0, card, Zone::Hand);
        assert!(!g.state.static_sources.contains(&id));
        g.move_object(id, Zone::Battlefield);
        assert_eq!(
            g.state.static_sources.iter().filter(|&&s| s == id).count(),
            1
        );
        let saved = g.snapshot();
        g.move_object(id, Zone::Exile);
        assert!(!g.state.static_sources.contains(&id));
        g.restore(&saved).unwrap();
        assert_eq!(
            g.state.static_sources.iter().filter(|&&s| s == id).count(),
            1
        );
        g.move_object(id, Zone::Hand);
    }
}

#[test]
fn own_turn_static_source_keeps_its_optional_discard_draw_attack_trigger() {
    for accept in [false, true] {
        let mut g = scenario();
        let source = named(
            &mut g,
            Player::P0,
            "Null Group Biological Assets",
            Zone::Battlefield,
        );
        let hand = named(&mut g, Player::P0, "Mountain", Zone::Hand);
        let library = g.state.players[0].library.len();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        pass_pair(&mut g);
        assert_eq!(
            g.observe(Player::P0).effect_choice.unwrap().kind,
            EffectChoiceKind::Discard
        );
        let saved = g.snapshot();
        let action = if accept {
            Action::Discard { card: hand }
        } else {
            Action::DeclineEffectPayment
        };
        act(&mut g, action.clone());
        let expected = g.snapshot();
        g.restore(&saved).unwrap();
        act(&mut g, action);
        assert_eq!(g.snapshot(), expected);
        assert_eq!(
            g.state.players[0].library.len(),
            library - usize::from(accept)
        );
        assert_eq!(
            g.object_view(hand).zone,
            if accept { Zone::Graveyard } else { Zone::Hand }
        );
        assert!(g.keywords(source).contains(Keywords::FIRST_STRIKE));
    }
}
