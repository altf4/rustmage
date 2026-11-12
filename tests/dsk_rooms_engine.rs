//! Rules fixtures for permanent split spells and exact Room door state.
mod support;

use rustmage::{PermanentType, RoomDoor};
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const ROOM: Card = Card::from_catalogue_id(176); // Anthem of Champions
const LEFT: Card = Card::from_catalogue_id(1024); // Deeproot Pilgrimage, {1}{U}
const RIGHT: Card = Card::from_catalogue_id(2145); // Impact Tremors, {1}{R}

const LEFT_ONLY: RoomDoors = RoomDoors {
    left: true,
    right: false,
};
const RIGHT_ONLY: RoomDoors = RoomDoors {
    left: false,
    right: true,
};
const OPEN: RoomDoors = RoomDoors {
    left: true,
    right: true,
};

const BLUE_COST: ManaCost = ManaCost {
    colored: [0, 1, 0, 0, 0, 0],
    generic: 1,
};
const RED_COST: ManaCost = ManaCost {
    colored: [0, 0, 0, 1, 0, 0],
    generic: 1,
};
const RED_ROOM_MANA: Mana = [0, 0, 0, 2, 0, 0];

const ROOM_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Left),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Right),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 2)],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LifeGainAdditional(1)],
    }],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [LEFT, RIGHT],
            triggers: &[(0, RoomDoor::Left), (1, RoomDoor::Right)],
            static_groups: &[(1, RoomDoor::Left)],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const ROOM_HALF: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room_parent: Some(ROOM),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const EERIE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Eerie,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const TOGGLE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Subtype("Room"),
        })),
        &[Instruction::ChooseRoomDoor {
            selection: RoomSelection::Target,
            operation: RoomOperation::Either,
        }],
    ),
    ..CardRules::EMPTY
};

const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::YourSpell)),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

const ROOM_MANA: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed(RED_ROOM_MANA),
            quality: &ManaQuality {
                restriction: ManaRestriction::RoomSpellOrUnlockDoor,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        targets: TargetPlan::NONE,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

const REDUCTION: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::RoomUnlockReduction(1)],
    }],
    ..CardRules::EMPTY
};

const OFF_TURN_ROOM: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::UntapControlledCreaturesDuringOtherUntapSteps],
    }],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [LEFT, RIGHT],
            triggers: &[],
            static_groups: &[(0, RoomDoor::Right)],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const CANNOT_UNTAP: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SelfUntap(UntapRestriction::All)],
    }],
    ..CardRules::EMPTY
};

const MULTIPLIER_ROOM: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDoorUnlocked(RoomDoor::Left),
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::CreateCopyTokenWithExceptions {
            subject: Subject::Targets,
            amount: 1,
            exceptions: CopyExceptions {
                reflection: true,
                ..CopyExceptions::NONE
            },
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalTriggers {
            scope: AdditionalTriggerScope::ControlledPermanent,
            condition: StatValue::Fixed(1),
        }],
    }],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [LEFT, RIGHT],
            triggers: &[(0, RoomDoor::Left)],
            static_groups: &[(1, RoomDoor::Right)],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const DEPARTURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const EACH_END: TriggerCondition = TriggerCondition::AtStep {
    step: TriggerStep::End,
    relation: CastRelation::Any,
    guard: StepGuard::Always,
};

const LIMITED_TRIGGERS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&EACH_END),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GainLife(PlayerSet::Controller, 1),
                Instruction::Reflexive { ability: 2 },
                Instruction::ScheduleStep {
                    step: TriggerStep::Upkeep,
                    relation: CastRelation::Any,
                    ability: 3,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::OncePerIncarnation(&EACH_END),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 8)],
        },
    ],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn room_cast(card: CardId, door: RoomDoor) -> Action {
    Action::AnnounceRoom {
        card,
        door,
        flash_surcharge: false,
    }
}

fn begin_unlock(object: ObjectRef, door: RoomDoor, cost: ManaCost) -> Action {
    Action::BeginRoomUnlock { object, door, cost }
}

fn unlock_door(game: &mut Game, room: CardId, door: RoomDoor, cost: ManaCost) {
    let object = game.object_ref(room);
    act(game, begin_unlock(object, door, cost));
    choose(game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
}

fn settle_rooms(game: &mut Game) {
    for _ in 0..48 {
        match game.state.pending {
            Pending::Priority if game.state.stack.is_empty() && game.state.resolving.is_none() => {
                return;
            }
            Pending::Priority => act(game, Action::Pass),
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }))
            }
            ref pending => panic!("unexpected Room resolution choice {pending:?}"),
        }
    }
    panic!("Room resolution did not settle");
}

#[test]
fn each_half_casts_with_its_cost_and_only_the_second_door_completes_the_room() {
    let _rules = fixture_rules::activate(&[
        (ROOM, &ROOM_RULES),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
        (Card::GrizzlyBears, &EERIE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let room = spawn(&mut game, player, ROOM, Zone::Hand);
        game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
        assert_eq!(game.object_mana_value(room), 4);
        reject(&mut game, room_cast(room, RoomDoor::Right));
        reject(&mut game, Action::AnnounceCast { card: room });
        act(&mut game, room_cast(room, RoomDoor::Left));
        pay(&mut game);
        let spell = game.state.stack.last().unwrap();
        assert_eq!(spell.mana_value(), 2);
        assert_eq!(spell.cast.unwrap().face, CastFace::RoomLeft);
        assert_eq!(game.object_view(room).name, LEFT.name());
        assert_eq!(game.object_view(room).colors, LEFT.colors());
        settle_rooms(&mut game);

        let object = game.object_ref(room);
        assert_eq!(game.object_view(room).card, ROOM);
        assert_eq!(game.object_view(room).room_doors, Some(LEFT_ONLY));
        assert_eq!(game.object_view(room).name, LEFT.name());
        assert_eq!(game.object_mana_value(room), 2);
        assert_eq!(game.state.players[player.index()].life, 22);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
        assert_eq!(
            game.state.players[player.other().index()].cards_drawn_this_turn,
            0
        );
        game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 1];
        reject(&mut game, begin_unlock(object, RoomDoor::Left, BLUE_COST));
        act(&mut game, begin_unlock(object, RoomDoor::Right, RED_COST));
        assert!(game.observe(player.other()).room_unlock_payment.is_some());
        choose(&mut game, |action| {
            matches!(action, Action::PayRoomUnlock { .. })
        });
        assert_eq!(game.object_ref(room), object);
        assert_eq!(game.object_view(room).room_doors, Some(OPEN));
        assert_eq!(
            game.object_view(room).colors,
            LEFT.colors() | RIGHT.colors()
        );
        assert_eq!(game.object_mana_value(room), 4);
        settle_rooms(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
        assert_eq!(
            game.state.players[player.other().index()].cards_drawn_this_turn,
            0
        );
    }
}

#[test]
fn graveyard_permanent_type_permission_allows_either_room_half_once() {
    let _rules =
        fixture_rules::activate(&[(ROOM, &ROOM_RULES), (LEFT, &ROOM_HALF), (RIGHT, &ROOM_HALF)]);
    let player = Player::P0;
    for (chosen, other, face, mana, doors) in [
        (
            RoomDoor::Left,
            RoomDoor::Right,
            CastFace::RoomLeft,
            [0, 1, 0, 0, 0, 1],
            LEFT_ONLY,
        ),
        (
            RoomDoor::Right,
            RoomDoor::Left,
            CastFace::RoomRight,
            [0, 0, 0, 1, 0, 1],
            RIGHT_ONLY,
        ),
    ] {
        let mut game = scenario(player);
        let muldrotha = put(
            &mut game,
            player,
            "Muldrotha, the Gravetide",
            Zone::Battlefield,
        );
        let room = spawn(&mut game, player, ROOM, Zone::Graveyard);
        let second = spawn(&mut game, player, ROOM, Zone::Graveyard);
        let opposing_room = spawn(&mut game, player.other(), ROOM, Zone::Graveyard);
        let permission = game.object_ref(muldrotha);
        let announce = |card, door, permanent_type| Action::AnnounceGraveyardRoom {
            card,
            door,
            flash_surcharge: false,
            permission,
            permanent_type,
        };
        let enchantment = PermanentType::Enchantment;
        game.state.players[player.index()].mana = mana;
        assert!(
            game.legal_actions()
                .contains(&announce(room, chosen, enchantment))
        );
        reject(&mut game, announce(room, other, enchantment));
        reject(&mut game, announce(room, chosen, PermanentType::Creature));
        reject(&mut game, announce(opposing_room, chosen, enchantment));
        reject(&mut game, room_cast(room, chosen));
        act(&mut game, announce(room, chosen, enchantment));
        pay(&mut game);
        let cast = game.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.face, face);
        assert_eq!(cast.origin, Zone::Graveyard);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        settle_rooms(&mut game);
        assert_eq!(game.object_view(room).room_doors, Some(doors));

        game.state.players[player.index()].mana = [8; 6];
        reject(&mut game, announce(second, chosen, enchantment));
        reject(&mut game, announce(second, other, enchantment));
        let creature = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
        assert!(
            game.legal_actions()
                .contains(&Action::PlayGraveyardPermanent {
                    card: creature,
                    permission,
                    permanent_type: PermanentType::Creature,
                })
        );
    }
}

#[test]
fn unlocking_an_off_turn_untap_door_adds_each_live_creature_to_one_untap_attempt() {
    let _rules = fixture_rules::activate(&[
        (ROOM, &OFF_TURN_ROOM),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
        (Card::GrizzlyBears, &CANNOT_UNTAP),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let rooms = [ROOM; 2].map(|card| spawn(&mut game, player, card, Zone::Battlefield));
        let ordinary = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        let own_step_only = put(&mut game, player, "Slumbering Cerberus", Zone::Battlefield);
        let never = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
        let infantry = put(
            &mut game,
            player,
            "Thousand Moons Infantry",
            Zone::Battlefield,
        );
        let animated_off_turn = put(&mut game, player, "Midnight Mangler", Zone::Battlefield);
        let land = put(&mut game, player, "Forest", Zone::Battlefield);
        let opposing_creature = put(&mut game, opponent, "Healer's Hawk", Zone::Battlefield);
        for id in [
            ordinary,
            own_step_only,
            never,
            infantry,
            animated_off_turn,
            land,
            opposing_creature,
        ] {
            game.state.objects[id as usize].tapped = true;
        }
        assert!(!game.object_view(animated_off_turn).is_creature());
        game.add_counter(infantry, CounterKind::Stun, 2);
        game.add_counter(never, CounterKind::Stun, 1);
        let locked = game.clone();
        let refs = rooms.map(|id| game.object_ref(id));
        for object in refs {
            act(&mut game, begin_unlock(object, RoomDoor::Right, RED_COST));
            choose(&mut game, |action| {
                matches!(action, Action::PayRoomUnlock { .. })
            });
            assert_eq!(game.object_view(object.card).room_doors, Some(RIGHT_ONLY));
        }
        let mut blanked = game.clone();
        blanked.remove_abilities_exact(&refs, refs[0], player, AnimationDuration::Permanent);

        for (mut branch, active_doors) in [(locked, false), (blanked, false), (game, true)] {
            branch.state.step = Step::End;
            act(&mut branch, Action::Pass);
            act(&mut branch, Action::Pass);
            assert_eq!(branch.state.active, opponent);
            assert_eq!(branch.state.step, Step::Upkeep);
            assert!(branch.object_view(animated_off_turn).is_creature());
            for id in [ordinary, own_step_only, animated_off_turn] {
                assert_eq!(branch.object_view(id).tapped, !active_doors);
            }
            assert!(!branch.object_view(opposing_creature).tapped);
            for id in [land, never, infantry] {
                assert!(branch.object_view(id).tapped);
                if id != land {
                    assert_eq!(branch.counter_count(id, CounterKind::Stun), 1);
                }
            }
        }
    }
}

#[test]
fn each_unlocked_multiplier_adds_one_trigger_with_an_independent_target() {
    let _rules = fixture_rules::activate(&[
        (ROOM, &MULTIPLIER_ROOM),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
    ]);
    let player = Player::P0;
    for multiplier_count in 0..=2 {
        let mut game = scenario(player);
        let room = spawn(&mut game, player, ROOM, Zone::Battlefield);
        let other_room = spawn(&mut game, player, ROOM, Zone::Battlefield);
        let foreign = spawn(&mut game, player.other(), ROOM, Zone::Battlefield);
        let foreign = game.object_ref(foreign);
        game.open_room_on_entry(foreign, RoomDoor::Right);
        let chosen: Vec<_> = ["Grizzly Bears", "Healer's Hawk", "Hill Giant"]
            .into_iter()
            .take(multiplier_count + 1)
            .map(|name| put(&mut game, player, name, Zone::Battlefield))
            .collect();
        for provider in [room, other_room].into_iter().take(multiplier_count) {
            unlock_door(&mut game, provider, RoomDoor::Right, RED_COST);
        }
        unlock_door(&mut game, room, RoomDoor::Left, BLUE_COST);
        for &target in &chosen {
            if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
                choose(&mut game, |action| {
                    matches!(action, Action::PutTrigger { .. })
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
        }
        let stack = &game.state.stack;
        assert_eq!(stack.len(), chosen.len());
        for (item, &target) in stack.iter().zip(&chosen) {
            assert!(!item.is_copy);
            assert_eq!(item.targets[0].target, Target::Card(target));
            assert_eq!(item.source.unwrap().id, room);
        }
        assert!(stack.windows(2).all(|pair| pair[0].id != pair[1].id));
        let definitions: Vec<_> = chosen
            .iter()
            .map(|&target| game.object_view(target).card)
            .collect();
        settle_rooms(&mut game);
        let copies: Vec<_> = game
            .observe(player)
            .objects
            .into_iter()
            .filter(|object| object.zone == Zone::Battlefield && object.is_token())
            .collect();
        assert_eq!(copies.len(), definitions.len());
        for definition in definitions {
            assert!(copies.iter().any(|copy| copy.card == definition
                && copy.controller == player
                && copy.has_subtype("Reflection")));
        }
    }
}

#[test]
fn a_departing_multiplier_uses_the_same_pre_event_world_as_the_death_trigger() {
    let _rules = fixture_rules::activate(&[
        (ROOM, &MULTIPLIER_ROOM),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
        (Card::GrizzlyBears, &DEPARTURE),
    ]);
    let player = Player::P0;
    for multiplier_open in [false, true] {
        let mut game = scenario(player);
        let room = spawn(&mut game, player, ROOM, Zone::Battlefield);
        let yours = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
        let theirs = spawn(
            &mut game,
            player.other(),
            Card::GrizzlyBears,
            Zone::Battlefield,
        );
        if multiplier_open {
            unlock_door(&mut game, room, RoomDoor::Right, RED_COST);
        }
        let observers = game.death_trigger_sources();
        let events = game.move_objects_raw(&[room, yours, theirs], Zone::Graveyard);
        assert!(
            events
                .iter()
                .all(|event| event.moved && event.destination == Zone::Graveyard)
        );
        game.finish_zone_events(&events, &observers);
        game.prepare_triggers();
        settle_rooms(&mut game);
        assert_eq!(
            game.state.players[player.index()].life,
            21 + i128::from(multiplier_open)
        );
        assert_eq!(game.state.players[player.other().index()].life, 21);
    }
}

#[test]
fn the_multiplier_applies_to_mana_triggers_in_payment_planning_and_actual_taps() {
    let _rules = fixture_rules::activate(&[
        (ROOM, &MULTIPLIER_ROOM),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let room = spawn(&mut game, player, ROOM, Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    put(
        &mut game,
        player,
        "Groundchuck & Dirtbag",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Outcaster Trailblazer", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    let mut locked = game.clone();
    locked.state.players[player.index()].mana = [0; 6];
    reject(&mut locked, cast.clone());

    unlock_door(&mut game, room, RoomDoor::Right, RED_COST);
    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, cast);
    act(&mut game, Action::TapMana { card: land });
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 3, 0]);
    assert!(game.state.stack.is_empty());
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.state.stack.len(), 1);
}

#[test]
fn limited_triggers_and_created_delayed_or_reflexive_abilities_resolve_once() {
    let _rules = fixture_rules::activate(&[
        (ROOM, &MULTIPLIER_ROOM),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
        (Card::GrizzlyBears, &LIMITED_TRIGGERS),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let room = spawn(&mut game, player, ROOM, Zone::Battlefield);
    spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    unlock_door(&mut game, room, RoomDoor::Right, RED_COST);

    game.state.step = Step::Main2;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    settle_rooms(&mut game);
    assert_eq!(game.state.step, Step::End);
    assert_eq!(game.state.players[player.index()].life, 27);

    game.begin_step(Step::End);
    game.give_priority();
    game.prepare_triggers();
    settle_rooms(&mut game);
    assert_eq!(game.state.players[player.index()].life, 27);

    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    settle_rooms(&mut game);
    assert_eq!(game.state.step, Step::Upkeep);
    assert_eq!(game.state.players[player.index()].life, 35);

    game.state.step = Step::Main2;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    settle_rooms(&mut game);
    assert_eq!(game.state.players[player.index()].life, 40);
}

#[test]
fn stack_copy_enters_with_same_half_while_a_battlefield_copy_enters_locked() {
    let _rules = fixture_rules::activate(&[
        (ROOM, &ROOM_RULES),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
        (card("Opt"), &COPY),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let room = spawn(&mut game, player, ROOM, Zone::Hand);
    let copier = put(&mut game, player, "Opt", Zone::Hand);
    act(&mut game, room_cast(room, RoomDoor::Right));
    pay(&mut game);
    cast(&mut game, copier, Some(Target::Spell(room)));
    settle_rooms(&mut game);
    let rooms: Vec<_> = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|object| object.zone == Zone::Battlefield && object.card == ROOM)
        .collect();
    assert_eq!(rooms.len(), 2);
    assert!(
        rooms
            .iter()
            .all(|object| object.room_doors == Some(RIGHT_ONLY))
    );
    assert!(rooms.iter().any(|object| object.is_token()));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);

    game.stage_copy_creation(player, CopyRecipe::new(ROOM), 1);
    game.advance_entries();
    game.give_priority();
    let rooms: Vec<_> = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|object| object.zone == Zone::Battlefield && object.card == ROOM)
        .collect();
    assert_eq!(rooms.len(), 3);
    let locked = rooms
        .iter()
        .find(|object| object.room_doors == Some(RoomDoors::default()))
        .unwrap();
    assert!(locked.is_token());
    assert!(locked.name.is_empty());
    assert_eq!(locked.colors, 0);
    assert_eq!(game.object_mana_value(locked.id), 0);
    assert!(locked.has_subtype("Room"));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
}

#[test]
fn locking_and_reopening_use_the_live_door_designation_and_keep_the_object() {
    let _rules = fixture_rules::activate(&[
        (ROOM, &ROOM_RULES),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
        (card("Unsummon"), &TOGGLE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let room = spawn(&mut game, player, ROOM, Zone::Hand);
    let toggle = put(&mut game, player, "Unsummon", Zone::Hand);
    act(&mut game, room_cast(room, RoomDoor::Left));
    pay(&mut game);
    settle_rooms(&mut game);
    let object = game.object_ref(room);

    cast(&mut game, toggle, Some(Target::Card(room)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(game.observe(player).room_choice.is_some());
    reject(
        &mut game,
        Action::ChooseRoomDoor {
            object,
            door: RoomDoor::Left,
            change: RoomChange::Unlock,
        },
    );
    act(
        &mut game,
        Action::ChooseRoomDoor {
            object,
            door: RoomDoor::Left,
            change: RoomChange::Lock,
        },
    );
    settle_rooms(&mut game);
    assert_eq!(
        game.object_view(room).room_doors,
        Some(RoomDoors::default())
    );
    assert_eq!(game.object_view(room).name, "");
    assert_eq!(game.object_ref(room), object);
    assert_eq!(game.state.players[player.index()].life, 22);

    act(&mut game, begin_unlock(object, RoomDoor::Left, BLUE_COST));
    choose(&mut game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
    settle_rooms(&mut game);
    assert_eq!(game.object_view(room).room_doors, Some(LEFT_ONLY));
    assert_eq!(game.state.players[player.index()].life, 24);
    assert_eq!(game.object_ref(room), object);
}

#[test]
fn reduced_unlock_funding_owns_restricted_mana_and_branches_transactionally() {
    let _rules = fixture_rules::activate(&[
        (ROOM, &ROOM_RULES),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
        (card("Short Bow"), &ROOM_MANA),
        (card("High Noon"), &REDUCTION),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let room = spawn(&mut game, player, ROOM, Zone::Battlefield);
    let next_room = spawn(&mut game, player, ROOM, Zone::Hand);
    let mana_source = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let enchantment = put(&mut game, player, "Artist's Talent", Zone::Hand);
    put(&mut game, player, "High Noon", Zone::Battlefield);
    game.state.players[player.index()].mana = [0; 6];
    let object = game.object_ref(room);
    let unlock = begin_unlock(
        object,
        RoomDoor::Right,
        ManaCost {
            generic: 0,
            ..RED_COST
        },
    );
    assert!(game.legal_actions().contains(&unlock));
    assert!(
        game.legal_actions()
            .contains(&room_cast(next_room, RoomDoor::Right))
    );
    reject(&mut game, begin_unlock(object, RoomDoor::Right, RED_COST));
    reject(&mut game, Action::AnnounceCast { card: shock });
    reject(&mut game, Action::AnnounceCast { card: enchantment });

    let checkpoint = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == unlock)
        .unwrap();
    let revision = game.revision();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    drop(game);
    act(&mut raw, unlock.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());

    let make_mana = Action::ActivateMana {
        card: mana_source,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: RED_ROOM_MANA,
    };
    act(&mut raw, make_mana.clone());
    act(&mut indexed, make_mana);
    act(&mut raw, Action::AbortRoomUnlock);
    act(&mut raw, Action::ChooseManaRollback { keep: false });
    assert_eq!(raw.snapshot(), checkpoint);
    assert!(!raw.object_view(mana_source).tapped);
    assert_eq!(raw.state.players[player.index()].mana, [0; 6]);
    assert_eq!(raw.object_view(room).room_doors, Some(RoomDoors::default()));
    choose(&mut indexed, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
    settle_rooms(&mut indexed);
    assert!(indexed.object_view(mana_source).tapped);
    assert_eq!(
        indexed.state.players[player.index()].mana,
        [0, 0, 0, 1, 0, 0]
    );
    assert_eq!(indexed.object_view(room).room_doors, Some(RIGHT_ONLY));
    assert!(indexed.state.stack.is_empty());
    assert_eq!(
        indexed.state.players[player.index()].spells_cast_this_turn,
        0
    );
    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.legal_actions().contains(&unlock));
    assert_eq!(
        indexed.object_view(room).room_doors,
        Some(RoomDoors::default())
    );
}
