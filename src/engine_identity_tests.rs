use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

#[test]
fn zone_change_preserves_lki_and_resets_new_object() {
    let mut game = scenario();
    let card = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let obj = &mut game.state.objects[card as usize];
    obj.tapped = true;
    obj.damage = 4;
    obj.growth = (3).into();
    obj.deathtouch_since_sba = true;
    let before = game.object_ref(card);
    let event = game.move_object(card, Zone::Hand);
    assert_eq!(event.before.reference, before);
    assert_eq!(event.before.zone, Zone::Battlefield);
    assert_eq!(
        (
            event.before.power,
            event.before.toughness,
            event.before.damage
        ),
        (5, 5, 4)
    );
    assert!(event.before.tapped);
    assert_eq!(event.after.generation, before.generation + 1);
    assert_eq!(event.destination, Zone::Hand);
    let returned = game.move_object(card, Zone::Battlefield);
    assert_ne!(returned.after, before);
    let new = game.object_view(card);
    assert_eq!((new.power, new.toughness, new.damage), (2, 2, 0));
    assert!(!new.tapped);
    assert!(new.summoning_sick);
    assert!(!game.state.objects[card as usize].deathtouch_since_sba);
    assert_eq!(event.before.reference, before); // Owned LKI survives subsequent moves.
}

#[test]
fn targeted_spell_cannot_find_returned_permanent_and_replay_is_identical() {
    let mut game = scenario();
    let bears = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let growth = spawn(&mut game, Player::P0, Card::GiantGrowth, Zone::Hand);
    game.state.players[0].mana[4] = 1;
    cast(&mut game, growth, Some(Target::Card(bears)));
    assert_eq!(
        game.state.stack[0].targets[0].object,
        Some(game.object_ref(bears))
    );
    let pending = game.snapshot();
    let original = game.object_ref(bears);
    game.move_object(bears, Zone::Hand);
    game.move_object(bears, Zone::Battlefield);
    pass_pair(&mut game);
    assert_eq!(game.object_view(bears).power, 2);
    assert_eq!(game.state.objects[growth as usize].zone, Zone::Graveyard);
    let stale_result = game.snapshot();
    game.restore(&pending);
    assert_eq!(game.object_ref(bears), original);
    game.move_object(bears, Zone::Hand);
    game.move_object(bears, Zone::Battlefield);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), stale_result);
    game.restore(&pending);
    pass_pair(&mut game);
    assert_eq!(game.object_view(bears).power, 5);
}

#[test]
fn counter_cannot_find_recast_spell_in_same_physical_slot() {
    let mut game = scenario();
    let bolt = spawn(&mut game, Player::P0, Card::LightningBolt, Zone::Hand);
    let counter = spawn(&mut game, Player::P1, Card::Counterspell, Zone::Hand);
    game.state.players[0].mana[3] = 2;
    game.state.players[1].mana[1] = 2;
    cast(&mut game, bolt, Some(Target::Player(Player::P1)));
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(bolt)));
    let old_target = game.state.stack.last().unwrap().targets[0].object.unwrap();
    game.move_object(bolt, Zone::Hand);
    game.state.priority = Player::P0;
    cast(&mut game, bolt, Some(Target::Player(Player::P1)));
    assert_ne!(old_target, game.state.stack.last().unwrap().object);
    // Isolate the stale counter's resolution while the new spell is still on stack.
    game.state.stack.swap(0, 1);
    pass_pair(&mut game);
    assert_eq!(game.state.objects[bolt as usize].zone, Zone::Stack);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 17);
}

#[test]
fn leaving_combat_removes_membership_but_surviving_attacker_stays_blocked() {
    let mut game = scenario();
    let attacker = spawn(&mut game, Player::P0, Card::HillGiant, Zone::Battlefield);
    let blocker = spawn(&mut game, Player::P1, Card::GoblinPiker, Zone::Battlefield);
    game.state.attackers.push(Attacker {
        card: attacker,
        blocked: true,
    });
    game.state.blocks.push((blocker, attacker));
    game.state.first_strikers.push(blocker);
    game.move_object(blocker, Zone::Hand);
    game.move_object(blocker, Zone::Battlefield);
    assert!(game.state.blocks.is_empty());
    assert!(game.state.first_strikers.is_empty());
    assert!(game.state.attackers[0].blocked);
    game.state.step = Step::Blockers;
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 20);
    assert_eq!(game.object_view(attacker).damage, 0);
    game.move_object(attacker, Zone::Hand);
    game.move_object(attacker, Zone::Battlefield);
    assert!(game.state.attackers.is_empty());
}

#[test]
fn all_zone_routes_maintain_incarnations_and_library_membership() {
    let mut game = scenario();
    let card = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Hand);
    let mut generation = game.object_ref(card).generation;
    for zone in [
        Zone::Library,
        Zone::Hand,
        Zone::Stack,
        Zone::Battlefield,
        Zone::Graveyard,
        Zone::Hand,
        Zone::Library,
        Zone::Graveyard,
    ] {
        let event = game.move_object(card, zone);
        generation += 1;
        assert_eq!(event.after.generation, generation);
        assert_eq!(event.before.reference.generation + 1, generation);
        let count = game.state.players[0]
            .library
            .iter()
            .filter(|&&id| id == card)
            .count();
        assert_eq!(count, usize::from(zone == Zone::Library));
        assert!(!game.state.players[1].library.contains(&card));
    }
}

#[test]
fn damage_source_ref_retains_departed_incarnation() {
    let mut game = scenario();
    let card = spawn(
        &mut game,
        Player::P0,
        Card::from_name("Vampire Nighthawk").unwrap(),
        Zone::Battlefield,
    );
    let source = game.damage_source(card);
    game.move_object(card, Zone::Graveyard);
    game.move_object(card, Zone::Battlefield);
    assert_ne!(source.reference, game.object_ref(card));
    assert!(source.keywords.contains(Keywords::LIFELINK));
    let gains = game.apply_damage_batch(&[DamageEvent {
        source,
        target: Target::Player(Player::P1),
        amount: 1,
        combat: false,
    }]);
    assert_eq!(gains[0].source.reference, source.reference);
    assert_eq!(game.state.players[0].life, 21);
}

#[test]
fn unsummon_in_response_invalidates_growth_and_can_replay_through_restore() {
    let mut game = scenario();
    let bears = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let growth = spawn(&mut game, Player::P0, Card::GiantGrowth, Zone::Hand);
    let bounce = spawn(
        &mut game,
        Player::P1,
        Card::from_name("Unsummon").unwrap(),
        Zone::Hand,
    );
    game.state.players[0].mana[4] = 1;
    game.state.players[1].mana[1] = 1;
    cast(&mut game, growth, Some(Target::Card(bears)));
    let targeted = game.object_ref(bears);
    act(&mut game, Action::Pass);
    cast(&mut game, bounce, Some(Target::Card(bears)));
    let pending = game.snapshot();
    for replay in 0..2 {
        if replay == 1 {
            game.restore(&pending);
        }
        pass_pair(&mut game);
        assert_eq!(game.state.objects[bears as usize].zone, Zone::Hand);
        assert_eq!(game.state.objects[bears as usize].owner, Player::P0);
        assert_ne!(game.object_ref(bears), targeted);
        assert_eq!(game.state.stack.len(), 1);
        pass_pair(&mut game);
        assert_eq!(game.object_view(bears).power, 2);
        assert_eq!(game.state.objects[growth as usize].zone, Zone::Graveyard);
        assert!(game.state.stack.is_empty());
    }
}

#[test]
fn removal_templates_enforce_targets_timing_and_payments() {
    for name in ["Bilbo's Deadly Slice", "Fell", "Murder", "Unsummon"] {
        let mut game = scenario();
        let removal = Card::from_name(name).unwrap();
        assert!(removal.is_supported() && removal.is_standard_legal());
        let spell = spawn(&mut game, Player::P0, removal, Zone::Hand);
        let bears = spawn(&mut game, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
        let land = spawn(&mut game, Player::P1, Card::Forest, Zone::Battlefield);
        assert!(
            !game
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::Cast {card, ..} if *card == spell))
        );
        game.state.players[0].mana = [0, 1, 3, 0, 0, 0];
        assert!(!game.legal_actions().iter().any(|a| matches!(a, Action::Cast {card, target: Some(Target::Card(id)), ..} if *card == spell && *id == land)));
        game.state.step = Step::Upkeep;
        let available = game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Cast {card, ..} if *card == spell));
        assert_eq!(available, removal.is_instant());
        game.state.step = Step::Main1;
        let old = game.object_ref(bears);
        cast(&mut game, spell, Some(Target::Card(bears)));
        pass_pair(&mut game);
        let expected = if name == "Unsummon" {
            Zone::Hand
        } else {
            Zone::Graveyard
        };
        assert_eq!(game.state.objects[bears as usize].zone, expected);
        assert_ne!(game.object_ref(bears), old);
        assert_eq!(game.state.objects[spell as usize].zone, Zone::Graveyard);
    }
}

#[test]
fn removing_last_declared_attacker_preserves_remaining_combat_steps() {
    let mut game = scenario();
    let attacker = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let bounce = spawn(
        &mut game,
        Player::P1,
        Card::from_name("Unsummon").unwrap(),
        Zone::Hand,
    );
    game.state.players[1].mana[1] = 1;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    act(&mut game, Action::Pass);
    cast(&mut game, bounce, Some(Target::Card(attacker)));
    pass_pair(&mut game);
    assert!(game.state.attackers.is_empty());
    let removed = game.snapshot();
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::Blockers);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::CombatDamage);
    let normal = game.snapshot();
    game.restore(&removed);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), normal);

    // Only declaring no attackers skips the blockers and damage steps.
    let mut game = scenario();
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::EndCombat);
}

#[test]
fn surviving_blocker_still_creates_first_strike_step_after_attacker_leaves() {
    let mut game = scenario();
    let attacker = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let blocker = spawn(
        &mut game,
        Player::P1,
        Card::from_name("Brightblade Stoat").unwrap(),
        Zone::Battlefield,
    );
    let bounce = spawn(
        &mut game,
        Player::P0,
        Card::from_name("Unsummon").unwrap(),
        Zone::Hand,
    );
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    act(&mut game, Action::Block { blocker, attacker });
    act(&mut game, Action::FinishBlockers);
    game.state.players[0].mana[1] = 1;
    cast(&mut game, bounce, Some(Target::Card(attacker)));
    pass_pair(&mut game);
    assert!(game.state.attackers.is_empty());
    assert!(game.state.blocks.is_empty());
    assert_eq!(game.observe(Player::P0).blocking, vec![blocker]);
    let orphan = game.snapshot();
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.state.first_strikers, vec![blocker]);
    assert_eq!(game.state.players[1].life, 20); // No opponent to damage, no lifelink.
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::CombatDamage);
    let normal = game.snapshot();
    game.restore(&orphan);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), normal);
    pass_pair(&mut game); // End combat step.
    pass_pair(&mut game); // Main2, all combat memberships cleared.
    assert_eq!(game.state.step, Step::Main2);
    assert!(game.observe(Player::P0).blocking.is_empty());
}
