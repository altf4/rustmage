//! Graveyard exile triggers look back to the same world as battlefield departures.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const GLOBAL: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            event: &TriggerCondition::CardsExiledFromBattlefieldOrGraveyard,
            guard: StepGuard::YourTurn,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 7)],
    }],
    ..CardRules::EMPTY
};
const CONTROLLED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 7)],
    }],
    ..CardRules::EMPTY
};
fn commit(g: &mut Game, moves: Vec<entries::EntryMove>) {
    g.stage_entries(moves, None);
    g.advance_entries();
}
fn finish(g: &mut Game) {
    if !g.state.pending_triggers.is_empty() {
        act(g, Action::Pass);
    }
    settle(g);
}

#[test]
fn simultaneous_departing_observer_sees_a_graveyard_exile_for_both_seats() {
    for rules in [&GLOBAL, &CONTROLLED] {
        let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, rules)]);
        for player in [Player::P0, Player::P1] {
            let mut g = scenario(player);
            let observer = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
            let victim = put(&mut g, player, "Savannah Lions", Zone::Graveyard);
            let moves = vec![
                g.entry_move(observer, Zone::Hand, false),
                g.entry_move(victim, Zone::Exile, false),
            ];
            commit(&mut g, moves);
            assert_eq!(g.state.pending_triggers.len(), 1);
            if matches!(
                rules.triggers[0].condition,
                TriggerCondition::ControlledCreaturesOrGraveyardCardsExiled
            ) {
                assert_eq!(
                    g.state.pending_triggers[0].exiled_trigger_cards.as_deref(),
                    Some(&vec![g.object_ref(victim)])
                );
            }
            finish(&mut g);
            assert_eq!(g.state.players[player.index()].life, 27);
            assert_eq!(g.state.objects[observer as usize].zone, Zone::Hand);
        }
    }
}

#[test]
fn simultaneous_new_observer_does_not_see_the_graveyard_exile() {
    for rules in [&GLOBAL, &CONTROLLED] {
        let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, rules)]);
        let mut g = scenario(Player::P0);
        let observer = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
        let victim = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
        let moves = vec![
            g.entry_move(observer, Zone::Battlefield, false),
            g.entry_move(victim, Zone::Exile, false),
        ];
        commit(&mut g, moves);
        assert!(g.state.pending_triggers.is_empty());
        assert_eq!(g.state.objects[observer as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.players[0].life, 20);
    }
}

#[test]
fn removal_of_ability_suppression_in_the_same_event_does_not_create_a_trigger() {
    for rules in [&GLOBAL, &CONTROLLED] {
        let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, rules)]);
        let mut g = scenario(Player::P0);
        let observer = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let aura = put(&mut g, Player::P1, "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(observer));
        assert!(!g.object_view(observer).printed_abilities_available());
        let victim = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
        let moves = vec![
            g.entry_move(aura, Zone::Hand, false),
            g.entry_move(victim, Zone::Exile, false),
        ];
        commit(&mut g, moves);
        assert!(g.object_view(observer).printed_abilities_available());
        assert!(g.state.pending_triggers.is_empty());
        assert_eq!(g.state.players[0].life, 20);
    }
}

#[test]
fn a_new_ability_suppression_does_not_erase_the_pre_event_trigger() {
    for rules in [&GLOBAL, &CONTROLLED] {
        let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, rules)]);
        let mut g = scenario(Player::P0);
        let observer = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let aura = put(&mut g, Player::P1, "Witness Protection", Zone::Exile);
        let victim = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
        let mut aura_move = g.entry_move(aura, Zone::Battlefield, false);
        aura_move.attachment = Some(g.object_ref(observer));
        let moves = vec![aura_move, g.entry_move(victim, Zone::Exile, false)];
        commit(&mut g, moves);
        assert!(!g.object_view(observer).printed_abilities_available());
        assert_eq!(g.state.pending_triggers.len(), 1);
        finish(&mut g);
        assert_eq!(g.state.players[0].life, 27);
    }
}

#[test]
fn departing_stolen_observer_keeps_its_pre_event_controller() {
    for rules in [&GLOBAL, &CONTROLLED] {
        let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, rules)]);
        let mut g = scenario(Player::P1);
        let observer = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[observer as usize].controller = Player::P1;
        let victim = put(&mut g, Player::P1, "Savannah Lions", Zone::Graveyard);
        let moves = vec![
            g.entry_move(observer, Zone::Hand, false),
            g.entry_move(victim, Zone::Exile, false),
        ];
        commit(&mut g, moves);
        assert_eq!(g.state.pending_triggers.len(), 1);
        assert_eq!(g.state.pending_triggers[0].controller, Player::P1);
        finish(&mut g);
        assert_eq!(g.state.players[1].life, 27);
        assert_eq!(g.state.players[0].life, 20);
    }
}

#[test]
fn global_noncreature_exiles_and_controlled_creature_filters_stay_distinct() {
    for (rules, expected) in [(&GLOBAL, 27), (&CONTROLLED, 20)] {
        let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, rules)]);
        let mut g = scenario(Player::P0);
        put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let victim = put(&mut g, Player::P0, "Forest", Zone::Graveyard);
        g.move_object(victim, Zone::Exile);
        finish(&mut g);
        assert_eq!(g.state.players[0].life, expected);
    }
}

#[test]
fn pending_foreign_attachment_preserves_lookback_across_raw_index_and_restore() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &GLOBAL)]);
    let mut g = scenario(Player::P0);
    let observer = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let aura = put(&mut g, Player::P1, "Witness Protection", Zone::Exile);
    let victim = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    let moves = vec![
        g.entry_move(aura, Zone::Battlefield, false),
        g.entry_move(victim, Zone::Exile, false),
    ];
    commit(&mut g, moves);
    assert!(matches!(
        g.state.pending,
        Pending::AuraEntry { player: Player::P1 }
    ));
    let action = Action::ChooseAuraAttachment {
        object: g.object_ref(observer),
    };
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let saved = g.snapshot();
    let revision = g.revision();
    let mut owned = {
        let parent = g.clone();
        parent.clone()
    };
    act(&mut g, action.clone());
    owned.apply_index(Player::P1, revision, index).unwrap();
    assert_eq!(g.snapshot(), owned.snapshot());
    g.restore(&saved).unwrap();
    assert_eq!(
        g.apply_index(Player::P1, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut g, action);
    assert_eq!(g.snapshot(), owned.snapshot());
    assert_eq!(g.state.pending_triggers.len(), 1);
    finish(&mut g);
    finish(&mut owned);
    assert_eq!(g.snapshot(), owned.snapshot());
    assert_eq!(g.state.players[0].life, 27);
}
