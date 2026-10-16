use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, player, Card::from_name(name).unwrap(), zone)
}
fn permanent(g: &mut Game, name: &str) -> CardId {
    named(g, Player::P0, name, Zone::Battlefield)
}
fn funded(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    let (mut mana, generic) = g.state.objects[id as usize].card.cost();
    mana[5] += generic;
    g.state.players[0].mana = mana;
    id
}
fn order(g: &mut Game) {
    while let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        act(g, a);
    }
}
fn resolve_all(g: &mut Game) {
    order(g);
    while !g.state.stack.is_empty() {
        pass_pair(g);
        order(g);
    }
}
fn choose(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(g, a);
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

#[test]
fn direct_cast_triggers_above_spell_only_after_cast_and_keeps_caster_priority() {
    let mut g = scenario();
    let archer = permanent(&mut g, "Firebrand Archer");
    let bolt = funded(&mut g, "Lightning Bolt");
    cast(&mut g, bolt, Some(Target::Player(Player::P1)));
    assert_eq!(g.actor(), Some(Player::P0));
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert_eq!(g.state.stack.len(), 2);
    assert_eq!(g.state.stack[0].card, bolt);
    assert_eq!(g.state.stack[1].card, archer);
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 19);
    assert_eq!(g.object_view(bolt).zone, Zone::Stack);
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 16);
    assert_eq!(g.observe(Player::P1).players[0].spells_cast_this_turn, 1);
}

#[test]
fn staged_target_and_payment_boundaries_never_count_proposals_and_replay() {
    let mut g = scenario();
    permanent(&mut g, "Firebrand Archer");
    let bolt = named(&mut g, Player::P0, "Lightning Bolt", Zone::Hand);
    let mountain = spawn(&mut g, Player::P0, Card::Mountain, Zone::Battlefield);
    replay(&mut g, Action::AnnounceCast { card: bolt });
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    assert!(g.state.pending_triggers.is_empty() && g.state.stack.is_empty());
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    replay(&mut g, Action::TapMana { card: mountain });
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    let before = g.snapshot();
    let rev = g.revision;
    assert_eq!(
        g.apply(Player::P0, Action::PayCosts { payment: [0; 6] }),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision, rev);
    replay(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 0, 0],
        },
    );
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert_eq!(g.state.stack.len(), 2);
    resolve_all(&mut g);
    assert_eq!(g.state.players[1].life, 16);
}

#[test]
fn prepaid_multitarget_and_optional_target_casts_complete_exactly_once() {
    let mut g = scenario();
    let body = permanent(&mut g, "Firebrand Archer");
    let spell = funded(&mut g, "Fancy Footwork");
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::BeginCast {card,..} if *card==spell))
        .unwrap();
    replay(&mut g, action);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    choose(&mut g, Target::Card(body));
    replay(&mut g, Action::FinishTargets);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    resolve_all(&mut g);
    assert_eq!(g.state.players[1].life, 19);
    let spell = funded(&mut g, "Plunge into Winter");
    cast(&mut g, spell, None);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 2);
    assert_eq!(g.state.stack.len(), 2);
}

#[test]
fn modal_selection_is_not_cast_until_targets_and_costs_complete() {
    let mut g = scenario();
    permanent(&mut g, "Firebrand Archer");
    let spell = funded(&mut g, "Bushwhack");
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(&mut g, Action::ChooseMode { mode: 0 });
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    assert!(g.state.stack.is_empty());
    pay(&mut g);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert_eq!(g.state.stack.len(), 2);
}

#[test]
fn repeated_prowess_instances_trigger_independently_and_are_not_static_keywords() {
    let mut g = scenario();
    let thor = permanent(&mut g, "Thor Odinson");
    let base = g.stats(thor);
    let bolt = funded(&mut g, "Lightning Bolt");
    cast(&mut g, bolt, Some(Target::Player(Player::P1)));
    assert_eq!(g.state.ordering_triggers.len(), 2);
    assert_eq!(g.stats(thor), base);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
        .unwrap();
    replay(&mut g, action);
    pass_pair(&mut g);
    assert_eq!(
        g.stats(thor),
        Stats {
            power: base.power + 1,
            toughness: base.toughness + 1
        }
    );
    pass_pair(&mut g);
    assert_eq!(
        g.stats(thor),
        Stats {
            power: base.power + 2,
            toughness: base.toughness + 2
        }
    );
    pass_pair(&mut g);
    g.finish_cleanup();
    assert_eq!(g.stats(thor), base);
}

#[test]
fn spell_type_filters_exclude_artifact_creatures_from_noncreature_triggers() {
    use crate::cards::CastFilter;
    let mut g = scenario();
    let prowess = permanent(&mut g, "Agent of Atlas");
    let base = g.stats(prowess);
    let robot = permanent(&mut g, "Ravenous Robots");
    let archer = permanent(&mut g, "Firebrand Archer");
    let artifact = funded(&mut g, "Meteor Golem");
    let card = g.state.objects[artifact as usize].card;
    assert!(CastFilter::Creature.matches(card) && CastFilter::Artifact.matches(card));
    assert!(!CastFilter::Noncreature.matches(card));
    cast(&mut g, artifact, None);
    assert_eq!(g.state.stack.len(), 2);
    assert_eq!(g.state.stack[1].card, robot);
    assert!(
        !g.state
            .stack
            .iter()
            .any(|item| item.card == archer || item.card == prowess)
    );
    assert_eq!(g.stats(prowess), base);
    // Creature spells do not see their own battlefield trigger while still on the stack.
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
}

#[test]
fn second_spell_counts_earlier_casts_before_source_exists_and_resets_both_players() {
    let mut g = scenario();
    let first = funded(&mut g, "Devoted Duelist");
    cast(&mut g, first, None);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    let second = funded(&mut g, "Boltwave");
    cast(&mut g, second, None);
    assert_eq!(g.state.stack.len(), 2); // The duelist counts itself as the first cast.
    resolve_all(&mut g);
    assert_eq!(g.state.players[1].life, 16);
    let third = funded(&mut g, "Boltwave");
    cast(&mut g, third, None);
    assert_eq!(g.state.stack.len(), 1);
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 3);
    g.state.players[1].spells_cast_this_turn = 7;
    g.state.active = Player::P1;
    g.begin_turn();
    assert_eq!(
        g.state.players.each_ref().map(|p| p.spells_cast_this_turn),
        [0, 0]
    );
    let saved = g.snapshot();
    g.state.active = Player::P0;
    g.state.priority = Player::P0;
    g.state.step = Step::Main1;
    let first = funded(&mut g, "Boltwave");
    cast(&mut g, first, None);
    assert_eq!(g.state.stack.len(), 1);
    resolve_all(&mut g);
    let second = funded(&mut g, "Boltwave");
    cast(&mut g, second, None);
    assert_eq!(g.state.stack.len(), 2);
    g.restore(&saved);
    assert_eq!(
        g.state.players.each_ref().map(|p| p.spells_cast_this_turn),
        [0, 0]
    );
}

#[test]
fn second_spell_on_opponents_turn_has_own_count_and_count_survives_countering() {
    let mut g = scenario();
    let duelist = permanent(&mut g, "Devoted Duelist");
    let hostile = named(&mut g, Player::P1, "Firebrand Archer", Zone::Battlefield);
    g.state.active = Player::P1;
    g.state.priority = Player::P0;
    let spell = funded(&mut g, "Lightning Bolt");
    cast(&mut g, spell, Some(Target::Player(Player::P1)));
    assert_eq!(g.state.stack.len(), 1); // Opponent's archer does not trigger.
    let counter = funded(&mut g, "Counterspell");
    cast(&mut g, counter, Some(Target::Spell(spell)));
    assert_eq!(g.state.players[0].spells_cast_this_turn, 2);
    assert_eq!(g.state.stack.last().unwrap().card, duelist);
    assert!(!g.state.stack.iter().any(|item| item.card == hostile));
    resolve_all(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 2);
    assert_eq!(g.state.players[1].spells_cast_this_turn, 0);
    assert_eq!(g.state.players[1].life, 19);
}

#[test]
fn pending_prowess_does_not_follow_a_departed_or_returned_source() {
    let mut g = scenario();
    let source = permanent(&mut g, "Agent of Atlas");
    let spell = funded(&mut g, "Lightning Bolt");
    cast(&mut g, spell, Some(Target::Player(Player::P1)));
    let old = g.object_ref(source);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    assert_ne!(g.object_ref(source), old);
    let base = g.stats(source);
    pass_pair(&mut g);
    assert_eq!(g.stats(source), base);
}

#[test]
fn source_damage_uses_current_or_departing_keywords_and_untap_obeys_stun() {
    let mut g = scenario();
    let archer = permanent(&mut g, "Firebrand Archer");
    g.state.objects[archer as usize].gained_keywords = Keywords::LIFELINK;
    let spell = funded(&mut g, "Boltwave");
    cast(&mut g, spell, None);
    g.move_object(archer, Zone::Graveyard);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 21);
    assert_eq!(g.state.players[1].life, 19);
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 16);
    let catapult = permanent(&mut g, "Unruly Catapult");
    g.state.objects[catapult as usize].tapped = true;
    g.state.objects[catapult as usize].counters.stun = 1;
    let spell = funded(&mut g, "Lightning Bolt");
    cast(&mut g, spell, Some(Target::Player(Player::P1)));
    pass_pair(&mut g);
    assert!(g.object_view(catapult).tapped);
    assert_eq!(g.object_view(catapult).counters.stun, 0);
    pass_pair(&mut g);
    let spell = funded(&mut g, "Lightning Bolt");
    cast(&mut g, spell, Some(Target::Player(Player::P1)));
    pass_pair(&mut g);
    assert!(!g.object_view(catapult).tapped);
}

#[test]
fn land_play_activated_abilities_and_direct_entry_do_not_count_as_casts() {
    let mut g = scenario();
    permanent(&mut g, "Firebrand Archer");
    let land = named(&mut g, Player::P0, "Mountain", Zone::Hand);
    act(&mut g, Action::PlayLand { card: land });
    let catapult = permanent(&mut g, "Unruly Catapult");
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate {card,..} if *card==catapult))
        .unwrap();
    act(&mut g, action);
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    pass_pair(&mut g);
    let creature = named(&mut g, Player::P0, "Agent of Atlas", Zone::Hand);
    g.move_object(creature, Zone::Battlefield);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    assert!(g.state.pending_triggers.is_empty());
}

#[test]
fn source_index_agrees_with_full_scan_after_every_admitted_card_enters_and_leaves() {
    let mut g = scenario();
    for card in Card::ALL.into_iter().filter(|c| !c.is_token()) {
        let id = spawn(&mut g, Player::P0, card, Zone::Hand);
        if matches!(card.entry_tapped(), EntryTapped::PayLife(_)) {
            g.stage_entries(vec![g.entry_move(id, Zone::Battlefield, false)], None);
            g.advance_entries();
            g.choose_entry_life(false);
            g.advance_entries();
        } else {
            g.move_object(id, Zone::Battlefield);
        }
        let expected = card
            .triggers()
            .iter()
            .any(|a| matches!(a.condition.event(), TriggerCondition::SpellCast { .. }));
        assert_eq!(
            g.state.cast_sources.contains(&id),
            expected,
            "{}",
            card.name()
        );
        let before = g.snapshot();
        g.move_object(id, Zone::Exile);
        assert!(!g.state.cast_sources.contains(&id));
        g.restore(&before);
        assert_eq!(g.state.cast_sources.contains(&id), expected);
        g.move_object(id, Zone::Exile);
    }
    assert!(g.state.cast_sources.is_empty());
}

#[test]
fn targeted_second_spell_damage_and_private_scry_keep_source_and_cast_history() {
    let mut g = scenario();
    let source = permanent(&mut g, "Iron-Fist Pulverizer");
    let first = funded(&mut g, "Boltwave");
    cast(&mut g, first, None);
    resolve_all(&mut g);
    let second = funded(&mut g, "Lightning Bolt");
    cast(&mut g, second, Some(Target::Player(Player::P1)));
    assert_eq!(g.observe(Player::P0).decision, "targets");
    assert_eq!(g.state.players[0].spells_cast_this_turn, 2);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    g.state.objects[source as usize].gained_keywords = Keywords::LIFELINK;
    g.move_object(source, Zone::Graveyard);
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 15);
    assert_eq!(g.state.players[0].life, 22);
    assert_eq!(g.observe(Player::P0).decision, "library_choice");
    assert!(g.observe(Player::P1).library_choice.is_none());
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ArrangeLibrary { .. }))
        .unwrap();
    replay(&mut g, action);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 2);
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 12);
}

#[test]
fn explicit_source_damage_on_another_creatures_entry_is_distinct_from_that_creature() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Shocking Sharpshooter", Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    assert!(g.state.pending_triggers.is_empty());
    let opponent = named(&mut g, Player::P1, "Grizzly Bears", Zone::Hand);
    g.move_object(opponent, Zone::Battlefield);
    assert!(g.state.pending_triggers.is_empty());
    let own = named(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    g.move_object(own, Zone::Battlefield);
    g.prepare_triggers();
    choose(&mut g, Target::Player(Player::P1));
    assert_eq!(g.state.stack[0].card, source);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].life, 19);
}
