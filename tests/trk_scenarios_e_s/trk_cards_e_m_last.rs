//! Printed E–M cards using shared Star Trek mechanics.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

const TAP_BEARER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBecomesTapped,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const DILEMMA: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Gain a life.",
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        SpellMode {
            text: "Gain two life.",
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
    ]),
    ..CardRules::EMPTY
};

fn settle(game: &mut Game) {
    for _ in 0..64 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}

#[test]
fn gwyndala_creates_attached_heirloom_then_copies_controlled_equipment_tokens() {
    let player = Player::P0;
    let mut game = scenario(player);
    let gwyndala = put(&mut game, player, "Gwyndala, the Progeny", Zone::Hand);
    cast(&mut game, gwyndala, None);
    settle(&mut game);
    let heirloom = game
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.card == Card::from_catalogue_id(6015))
            .then_some(id as CardId)
        })
        .unwrap();
    assert_eq!(
        game.object_view(heirloom).attached_to,
        Some(game.object_ref(gwyndala))
    );
    assert_eq!(stats(&game, gwyndala), (2, 2));
    let target = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: gwyndala,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(gwyndala),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(heirloom).name, "Swiftfoot Boots");
    assert_eq!(
        game.object_view(heirloom).attached_to,
        Some(game.object_ref(gwyndala))
    );
    assert_eq!(stats(&game, gwyndala), (1, 1));
    assert!(game.keywords(gwyndala).contains(Keywords::HEXPROOF));
    assert!(game.keywords(gwyndala).contains(Keywords::HASTE));
    game.expire_temporary_copies(true);
    assert_eq!(game.object_view(heirloom).name, "Neuroflux Heirloom");
    assert_eq!(stats(&game, gwyndala), (2, 2));
}

#[test]
fn kira_makes_a_controlled_entrant_tapped_attacking_and_boosts_that_entrant() {
    let player = Player::P0;
    let mut game = scenario(player);
    let kira = put(
        &mut game,
        player,
        "Kira Nerys, Firebrand",
        Zone::Battlefield,
    );
    assert!(game.keywords(kira).contains(Keywords::TRAMPLE));
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    let walker = put(
        &mut game,
        player.other(),
        "Kaito, Cunning Infiltrator",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 3);
    let entrant = put(&mut game, player, "Hill Giant", Zone::Hand);
    let movement = game.entry_move(entrant, Zone::Battlefield, false);
    game.stage_entries(vec![movement], None);
    assert!(game.advance_entries());
    assert_eq!(
        game.state.pending,
        Pending::EntryAttackingDefender { player }
    );
    let choose_defender = Action::ChooseEntryAttackingDefender {
        defender: CombatDefender::Player(player.other()),
    };
    let before = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == choose_defender)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, choose_defender);
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert!(game.object_view(entrant).tapped);
    assert!(game.state.attackers.iter().any(|row| row.card == entrant));
    settle(&mut game);
    assert_eq!(stats(&game, entrant), (4, 4));
    game.state.step = Step::Cleanup;
    game.finish_cleanup();
    assert_eq!(stats(&game, entrant), (3, 3));
    indexed.restore(&before).unwrap();
    assert_eq!(indexed.snapshot(), before);
}

#[test]
fn infinite_diversity_uses_non_borg_types_and_exiles_original_and_itself() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.create_token_batch_capture(player, Card(6007), 1, false, None, true);
    let target = put(&mut game, player, "Divination", Zone::Graveyard);
    let spell = put(&mut game, player, "Infinite Diversity", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let copies = game
        .observe(player)
        .resolution_free_casts
        .unwrap()
        .remaining;
    assert_eq!(copies.len(), 2);
    assert_eq!(game.object_view(target).zone, Zone::Exile);
    let original = game.object_ref(target);
    reject(
        &mut game,
        Action::CastExiledDuringResolution {
            object: original,
            face: CastFace::Normal,
        },
    );
    for object in copies {
        act(
            &mut game,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(&mut game);
        }
    }
    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
}

#[test]
fn lwaxana_doubles_one_creatures_trigger_while_tapped_and_may_skip_untap() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &TAP_BEARER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let lwaxana = put(
        &mut game,
        player,
        "Lwaxana Troi, Charming Guest",
        Zone::Battlefield,
    );
    let bearer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: bearer,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::AnnounceActivate {
            card: lwaxana,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(lwaxana),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bearer),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(lwaxana).tapped);

    act(
        &mut game,
        Action::AnnounceActivate {
            card: bearer,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    game.begin_turn();
    assert_eq!(
        game.observe(player).untap_choice,
        Some(game.object_ref(lwaxana))
    );
    act(&mut game, Action::ChooseOptional { accept: false });
    assert!(game.object_view(lwaxana).tapped);
}

#[test]
fn marooned_paid_entry_taps_and_enchanted_creature_cannot_beam_up() {
    let player = Player::P0;
    let mut game = scenario(player);
    let enchanted = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let aura = put(&mut game, player, "Marooned", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(enchanted)));
    for _ in 0..16 {
        if game
            .legal_actions()
            .contains(&Action::DeclineResolutionMana)
        {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert!(!game.object_view(enchanted).tapped);
    choose(&mut game, |a| matches!(a, Action::PayResolutionMana { .. }));
    settle(&mut game);
    assert!(game.object_view(enchanted).tapped);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(enchanted))
    );
    game.begin_turn();
    assert!(game.object_view(enchanted).tapped);
    game.state.step = Step::Main1;
    game.state.pending = Pending::Priority;
    game.state.priority = player;
    game.state.players[player.index()].mana = [8; 6];
    let spell = put(&mut game, player, "Final Frontier", Zone::Graveyard);
    let beam = Action::AnnounceBeamMeUp {
        card: spell,
        face: CastFace::Normal,
        cost: ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 1,
        },
        flash_surcharge: false,
        provider: None,
    };
    reject(&mut game, beam.clone());
    let other = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    assert!(game.legal_actions().contains(&beam));
    act(&mut game, beam);
    act(&mut game, Action::BeginOrderedCosts);
    let mana_clause = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && row.clause == CastCostClause::Mana)
        .unwrap() as u16;
    act(
        &mut game,
        Action::ChooseCastCost {
            clause: mana_clause,
        },
    );
    pay(&mut game);
    let return_clause = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && matches!(row.clause, CastCostClause::BeamMeUpReturn(_, _)))
        .unwrap() as u16;
    act(
        &mut game,
        Action::ChooseCastCost {
            clause: return_clause,
        },
    );
    let blocked = Action::PayBeamMeUpReturn {
        object: game.object_ref(enchanted),
    };
    reject(&mut game, blocked);
    let other_ref = game.object_ref(other);
    act(&mut game, Action::PayBeamMeUpReturn { object: other_ref });
    assert_eq!(game.object_view(enchanted).zone, Zone::Battlefield);
    assert_eq!(game.object_view(other).zone, Zone::Hand);
}

#[test]
fn bounty_tutors_whale_stations_and_extra_turn_requires_no_begun_extra_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let whale_card = put(&mut game, player, "Humpback Whales", Zone::Library);
    let bounty = put(
        &mut game,
        player,
        "H.M.S. Bounty, Unlikely Timeship",
        Zone::Hand,
    );
    cast(&mut game, bounty, None);
    for _ in 0..12 {
        if game
            .legal_actions()
            .contains(&Action::ChooseSearch { card: whale_card })
        {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert!(game.observe(player).search_choice.is_some());
    assert!(game.observe(player.other()).search_choice.is_none());
    act(&mut game, Action::ChooseSearch { card: whale_card });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(whale_card).zone, Zone::Hand);
    assert!(!game.is_creature(bounty));

    let whale = put(&mut game, player, "Humpback Whales", Zone::Battlefield);
    let giant = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    for donor in [whale, giant] {
        act(
            &mut game,
            Action::AnnounceActivate {
                card: bounty,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = game.object_ref(donor);
        act(&mut game, Action::ChooseAdditionalCost { object });
        pay(&mut game);
        settle(&mut game);
    }
    assert_eq!(game.counter_count(bounty, CounterKind::Charge), 8);
    assert!(game.is_creature(bounty));
    assert_eq!(stats(&game, bounty), (3, 4));
    assert!(game.keywords(bounty).contains(Keywords::FLYING));

    let mut walker_branch = game.clone();
    let walker = put(
        &mut walker_branch,
        player.other(),
        "Kaito, Cunning Infiltrator",
        Zone::Battlefield,
    );
    walker_branch.add_counter(walker, CounterKind::Loyalty, 5);
    let walker_source = walker_branch.damage_source(bounty);
    walker_branch.apply_damage_batch(&[DamageEvent {
        source: walker_source,
        target: Target::Card(walker),
        amount: 1,
        combat: true,
        preventable: true,
    }]);
    settle(&mut walker_branch);
    assert_eq!(
        walker_branch.state.players[player.index()].cards_drawn_this_turn,
        0
    );

    let mut without_whale = game.clone();
    without_whale.move_object(whale, Zone::Graveyard);
    let no_whale_hit = DamageEvent {
        source: without_whale.damage_source(bounty),
        target: Target::Player(player.other()),
        amount: 3,
        combat: true,
        preventable: true,
    };
    without_whale.apply_damage_batch(&[no_whale_hit]);
    settle(&mut without_whale);
    assert_eq!(
        without_whale.state.players[player.index()].cards_drawn_this_turn,
        1
    );
    assert!(without_whale.observe(player).turn_schedule.is_none());

    let hit = DamageEvent {
        source: game.damage_source(bounty),
        target: Target::Player(player.other()),
        amount: 3,
        combat: true,
        preventable: true,
    };
    game.apply_damage_batch(&[hit]);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.observe(player).turn_schedule.unwrap().pending.len(), 1);
    game.state.step = Step::End;
    game.give_priority();
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(game.observe(player).players[player.index()].has_begun_extra_turn_this_game);
    let pending = game.observe(player).turn_schedule.unwrap().pending.len();
    game.state.step = Step::Main1;
    game.state.pending = Pending::Priority;
    game.state.priority = player;
    let again_source = game.damage_source(bounty);
    game.apply_damage_batch(&[DamageEvent {
        source: again_source,
        ..hit
    }]);
    settle(&mut game);
    assert_eq!(
        game.observe(player).turn_schedule.unwrap().pending.len(),
        pending
    );
}

#[test]
fn exploratory_mission_exiles_to_nonland_grants_play_and_adds_red_at_five_quests() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mission = put(&mut game, player, "Exploratory Mission", Zone::Battlefield);
    let found = put(&mut game, player, "Shock", Zone::Library);
    let first_land = put(&mut game, player, "Forest", Zone::Library);
    let second_land = put(&mut game, player, "Island", Zone::Library);
    let entered = put(&mut game, player, "Forest", Zone::Hand);
    game.move_object(entered, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.counter_count(mission, CounterKind::Quest), 1);
    for id in [found, first_land, second_land] {
        assert_eq!(game.object_view(id).zone, Zone::Exile);
    }
    act(&mut game, Action::AnnounceExile { card: found });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(found).zone, Zone::Graveyard);

    for quest in 2..=5 {
        let nonland = put(&mut game, player, "Shock", Zone::Library);
        let land = put(&mut game, player, "Forest", Zone::Hand);
        let red_before = game.state.players[player.index()].mana[3];
        game.move_object(land, Zone::Battlefield);
        settle(&mut game);
        assert_eq!(game.counter_count(mission, CounterKind::Quest), quest);
        assert_eq!(game.object_view(nonland).zone, Zone::Exile);
        assert_eq!(
            game.state.players[player.index()].mana[3],
            red_before + if quest == 5 { 2 } else { 0 }
        );
    }
}

#[test]
fn gabriel_lorca_returns_from_graveyard_only_after_a_paid_dilemma() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DILEMMA)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let gabriel = put(
        &mut game,
        player,
        "Gabriel Lorca, Mirror Menace",
        Zone::Graveyard,
    );
    let modal = put(&mut game, player, "Opt", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: modal });
    act(&mut game, Action::ChooseMode { mode: 0 });
    pay(&mut game);
    for _ in 0..16 {
        if game
            .legal_actions()
            .contains(&Action::DeclineResolutionMana)
        {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert_eq!(declined.object_view(gabriel).zone, Zone::Graveyard);
    choose(&mut game, |a| matches!(a, Action::PayResolutionMana { .. }));
    settle(&mut game);
    assert_eq!(game.object_view(gabriel).zone, Zone::Hand);
}

#[test]
fn hive_mind_grants_paid_combat_draw_only_to_controlled_artifact_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hive = put(
        &mut game,
        player,
        "Hive Mind Coprocessor",
        Zone::Battlefield,
    );
    let artifact = put(&mut game, player, "Interstellar Effigy", Zone::Battlefield);
    let ordinary = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Interstellar Effigy",
        Zone::Battlefield,
    );
    let events = [artifact, ordinary, enemy].map(|card| DamageEvent {
        source: game.damage_source(card),
        target: Target::Player(if card == enemy {
            player
        } else {
            player.other()
        }),
        amount: 1,
        combat: true,
        preventable: true,
    });
    game.apply_damage_batch(&events);
    for _ in 0..24 {
        if matches!(game.state.pending, Pending::EffectOptional { .. }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    settle(&mut declined);
    assert_eq!(
        declined.state.players[player.index()].cards_drawn_this_turn,
        0
    );
    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.state.players[player.index()].life, 18);
    game.move_object(hive, Zone::Graveyard);
    let source = game.damage_source(artifact);
    game.apply_damage_batch(&[DamageEvent {
        source,
        target: Target::Player(player.other()),
        amount: 1,
        combat: true,
        preventable: true,
    }]);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn mirror_universe_kicker_switches_from_one_creature_target_to_one_player_target() {
    let player = Player::P0;
    for kicked in [false, true] {
        let mut game = scenario(player);
        let cub = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let spell = put(&mut game, player, "Mirror Universe Intrusion", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseKicker { pay: kicked });
        let correct = if kicked {
            Target::Player(player.other())
        } else {
            Target::Card(cub)
        };
        let wrong = if kicked {
            Target::Card(cub)
        } else {
            Target::Player(player.other())
        };
        reject(&mut game, Action::ChooseTarget { target: wrong });
        act(&mut game, Action::ChooseTarget { target: correct });
        pay(&mut game);
        settle(&mut game);
        let tokens: Vec<_> = game.controlled_token_ids(player).collect();
        assert_eq!(tokens.len(), if kicked { 2 } else { 1 });
        assert!(
            tokens
                .iter()
                .all(|&id| game.keywords(id).contains(Keywords::MENACE))
        );
        assert!(tokens.iter().any(|&id| stats(&game, id) == (2, 2)));
        if kicked {
            assert!(tokens.iter().any(|&id| stats(&game, id) == (3, 3)));
        }
    }
}

#[test]
fn gul_dukat_drains_each_opponent_for_life_already_lost_this_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Gul Dukat, Colonial Prefect",
        Zone::Battlefield,
    );
    game.state.players[player.other().index()].life = 16;
    game.state.players[player.other().index()].life_lost_this_turn = 4;
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 12);
    assert_eq!(game.state.players[player.index()].life, 24);
    assert_eq!(
        game.state.players[player.other().index()].life_lost_this_turn,
        8
    );
}

const ONE_CHARGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::Charge,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn geordi_doubles_charge_placement_only_on_artifacts_you_control() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &ONE_CHARGE)]);
    let player = Player::P0;
    for (target_name, controller, expected) in [
        ("Short Bow", player, 2),
        ("Bear Cub", player, 1),
        ("Short Bow", player.other(), 1),
    ] {
        let mut game = scenario(player);
        put(
            &mut game,
            player,
            "Lt. Cmdr. Geordi La Forge",
            Zone::Battlefield,
        );
        let recipient = put(&mut game, controller, target_name, Zone::Battlefield);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(recipient)));
        settle(&mut game);
        assert_eq!(game.counter_count(recipient, CounterKind::Charge), expected);
    }
}

#[test]
fn mr_spock_replaces_own_draws_but_not_opponents_draws() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spock = put(
        &mut game,
        player,
        "Mr. Spock, Science Officer",
        Zone::Battlefield,
    );
    for _ in 0..4 {
        put(&mut game, player, "Island", Zone::Library);
        put(&mut game, player.other(), "Forest", Zone::Library);
    }
    let checkpoint = game.snapshot();
    game.draw_many(player, 1);
    game.draw_many(player.other(), 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(
        game.state.players[player.other().index()].cards_drawn_this_turn,
        1
    );
    game.restore(&checkpoint).unwrap();
    assert_eq!(game.snapshot(), checkpoint);
    game.move_object(spock, Zone::Graveyard);
    game.draw_many(player, 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn live_long_doubles_life_draws_current_hand_size_and_grants_temporary_no_maximum() {
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].life = 7;
    put(&mut game, player, "Island", Zone::Hand);
    put(&mut game, player, "Forest", Zone::Hand);
    for _ in 0..4 {
        put(&mut game, player, "Mountain", Zone::Library);
    }
    let spell = put(&mut game, player, "Live Long and Prosper", Zone::Hand);
    assert!(
        Card::from_name("Live Long and Prosper")
            .unwrap()
            .cannot_be_countered()
    );
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 14);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.maximum_hand_size(player), None);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn erica_puts_hand_land_and_returns_one_controlled_land_at_next_end_step() {
    let player = Player::P0;
    let mut game = scenario(player);
    let erica = put(
        &mut game,
        player,
        "Erica Ortegas, Plucky Pilot",
        Zone::Battlefield,
    );
    let prior = put(&mut game, player, "Forest", Zone::Battlefield);
    let entering = put(&mut game, player, "Island", Zone::Hand);
    game.tap_objects(&[erica]);
    for _ in 0..12 {
        if game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::PutHandLand { .. }))
        {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let entering_ref = game.object_ref(entering);
    act(
        &mut game,
        Action::PutHandLand {
            object: entering_ref,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(entering).zone, Zone::Battlefield);
    game.begin_step(Step::End);
    game.prepare_triggers();
    for _ in 0..12 {
        let action = Action::ChooseEffectObject {
            object: game.object_ref(prior),
        };
        if game.legal_actions().contains(&action) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let prior_ref = game.object_ref(prior);
    act(&mut game, Action::ChooseEffectObject { object: prior_ref });
    if game.legal_actions().contains(&Action::FinishEffectChoice) {
        act(&mut game, Action::FinishEffectChoice);
    }
    settle(&mut game);
    assert_eq!(game.object_view(prior).zone, Zone::Hand);
}

#[test]
fn doctor_spell_replaces_creature_types_and_grows_target_until_end_of_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let klingon = put(
        &mut game,
        player,
        "General Chang, Cold Warrior",
        Zone::Battlefield,
    );
    game.tap_objects(&[klingon]);
    let spell = put(&mut game, player, "I'm a Doctor, Not a . . .", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(klingon)));
    settle(&mut game);
    assert_eq!(game.counter_count(klingon, CounterKind::PlusOne), 1);
    assert!(!game.object_view(klingon).tapped);
    assert!(game.has_subtype(klingon, "Doctor"));
    assert!(!game.has_subtype(klingon, "Klingon"));
    let grown = stats(&game, klingon);
    game.state.step = Step::Cleanup;
    game.finish_cleanup();
    assert_eq!(stats(&game, klingon), (grown.0 - 1, grown.1 - 1));
    assert!(game.has_subtype(klingon, "Klingon"));
    assert!(!game.has_subtype(klingon, "Doctor"));
}

#[test]
fn engineered_explosives_sunburst_and_sacrifice_use_charge_counter_lki() {
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let cheap = put(
        &mut game,
        player.other(),
        "Llanowar Elves",
        Zone::Battlefield,
    );
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let explosives = put(&mut game, player, "Engineered Explosives", Zone::Hand);
    game.state.players[player.index()].mana = [0, 1, 1, 0, 0, 0];
    act(&mut game, Action::AnnounceCast { card: explosives });
    act(&mut game, Action::ChooseX { value: 2 });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 1, 0, 0, 0],
        },
    );
    settle(&mut game);
    assert_eq!(game.counter_count(explosives, CounterKind::Charge), 2);
    game.state.players[player.index()].mana = [2, 0, 0, 0, 0, 0];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: explosives,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    settle(&mut game);
    for card in [own, opposing, explosives] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    for card in [cheap, land] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }
}

#[test]
fn gray_and_adira_copy_requires_prior_noncreature_cast_and_expires_at_next_end() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Gray and Adira, Forever Joined",
        Zone::Battlefield,
    );
    let target = put(
        &mut game,
        player,
        "Jett Reno, Problem Solver",
        Zone::Battlefield,
    );
    let mut no_spell = game.clone();
    no_spell.begin_step(Step::BeginCombat);
    no_spell.prepare_triggers();
    assert!(no_spell.controlled_token_ids(player).next().is_none());
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(player.other())));
    settle(&mut game);
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    for _ in 0..12 {
        let choice = Action::ChooseTarget {
            target: Target::Card(target),
        };
        if game.legal_actions().contains(&choice) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    let copy = game.controlled_token_ids(player).next().unwrap();
    assert!(!game.object_view(copy).is_legendary());
    assert!(game.keywords(copy).contains(Keywords::HASTE));
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(&mut game);
    assert!(!game.on_battlefield(copy));
    assert_eq!(game.object_view(target).zone, Zone::Battlefield);
}

#[test]
fn jadzia_adds_mana_equal_to_live_counters_and_discards_for_transfer() {
    let player = Player::P0;
    let mut game = scenario(player);
    let jadzia = put(
        &mut game,
        player,
        "Jadzia Dax, Living Legacy",
        Zone::Battlefield,
    );
    game.add_counter(jadzia, CounterKind::PlusOne, 2);
    game.add_counter(jadzia, CounterKind::Stun, 1);
    game.state.players[player.index()].mana[4] = 0;
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.counter_count(jadzia, CounterKind::PlusOne), 3);
    assert_eq!(game.state.players[player.index()].mana[4], 4);

    let mut transfer = scenario(player);
    let recipient = put(&mut transfer, player, "Bear Cub", Zone::Battlefield);
    let from_hand = put(
        &mut transfer,
        player,
        "Jadzia Dax, Living Legacy",
        Zone::Hand,
    );
    transfer.state.players[player.index()].mana = [2, 0, 0, 0, 1, 0];
    act(
        &mut transfer,
        Action::AnnounceActivate {
            card: from_hand,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut transfer,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    if transfer.legal_actions().contains(&Action::FinishTargets) {
        act(&mut transfer, Action::FinishTargets);
    }
    pay(&mut transfer);
    settle(&mut transfer);
    assert_eq!(transfer.object_view(from_hand).zone, Zone::Graveyard);
    assert_eq!(transfer.counter_count(recipient, CounterKind::PlusOne), 2);
}

#[test]
fn khan_can_be_cast_from_own_exile_and_sacrifices_exactly_one_other_permanent() {
    let player = Player::P0;
    let mut game = scenario(player);
    let khan = put(&mut game, player, "Khan, Vengeful Heart", Zone::Exile);
    let material = put(&mut game, player, "Short Bow", Zone::Battlefield);
    for _ in 0..2 {
        put(&mut game, player, "Island", Zone::Library);
    }
    let foreign = put(
        &mut game,
        player.other(),
        "Khan, Vengeful Heart",
        Zone::Exile,
    );
    reject(&mut game, Action::AnnounceExile { card: foreign });
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceExile { card: khan })
    );
    act(&mut game, Action::AnnounceExile { card: khan });
    pay(&mut game);
    for _ in 0..12 {
        let choice = Action::Sacrifice {
            object: game.object_ref(material),
        };
        if game.legal_actions().contains(&choice) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let own = game.object_ref(khan);
    reject(&mut game, Action::Sacrifice { object: own });
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    settle(&mut declined);
    assert_eq!(declined.object_view(material).zone, Zone::Battlefield);
    assert_eq!(
        declined.state.players[player.index()].cards_drawn_this_turn,
        0
    );
    let material_ref = game.object_ref(material);
    act(
        &mut game,
        Action::Sacrifice {
            object: material_ref,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(khan).zone, Zone::Battlefield);
    assert_eq!(game.object_view(material).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn khaaaaaaaaaaaannn_scales_x_damage_and_exiles_if_target_dies_later() {
    let player = Player::P0;
    let mut game = scenario(player);
    let target = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Khaaaaaaaaaaaannn!", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 2, 0, 0];
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseX { value: 1 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Battlefield);
    let source = game.damage_source(target);
    game.apply_damage_batch(&[DamageEvent {
        source,
        target: Target::Card(target),
        amount: 1,
        combat: false,
        preventable: true,
    }]);
    game.state_based_actions();
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Exile);
}
