//! Shared Lorwyn program effects without admitting new card definitions.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;
fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn ready(g: &mut Game, test: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if test(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("bounded effect continuation");
}
fn drain(g: &mut Game) {
    ready(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));

#[test]
fn return_then_lose_abilities_preserves_entry_choices_and_frozen_entry_trigger() {
    const BODY: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        keywords: Keywords::FLYING,
        entry: EntryTapped::PayLife(2),
        triggers: &[
            TriggeredAbility {
                condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::SourceCounter {
                    kind: CounterKind::MinusOne,
                    minimum: 1,
                }),
                targets: TargetPlan::NONE,
                program: &[Instruction::ReturnDeathCard(ReturnKind::WithoutAbilities)],
            },
            TriggeredAbility {
                condition: TriggerCondition::SelfEnters,
                targets: TargetPlan::NONE,
                program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            },
        ],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Hill Giant"), &BODY)]);
    for player in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = scenario(player);
            let creature = put(&mut g, player, "Hill Giant", Zone::Battlefield);
            let source = g.object_ref(creature);
            let before = g.state.players[player.index()].library.len();
            g.add_counter(creature, CounterKind::MinusOne, 1);
            g.move_object(creature, Zone::Graveyard);

            let mut movement = g.entry_move(creature, Zone::Battlefield, false);
            movement.return_modification = Some((ReturnKind::WithoutAbilities, source));
            let goal = FundingGoal {
                actor: player,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let world = FundingState::from_game(&g, &goal);
            let mut projected = Vec::new();
            world.visit_entry_moves(vec![movement], true, None, &mut |next| {
                projected.push(
                    characteristic_query::CharacteristicQuery::in_world(next).object_view(creature),
                );
                false
            });

            g.prepare_triggers();
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EntryLife { .. })
            });
            assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
            let before_choice = g.snapshot();
            let action = Action::ChooseEntryLife { pay };
            let mut child = g.clone();
            let index = child
                .legal_actions()
                .iter()
                .position(|a| *a == action)
                .unwrap();
            child.apply_index(player, child.revision(), index).unwrap();
            act(&mut g, action.clone());
            assert_eq!(g.snapshot(), child.snapshot());
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(
                    serde_json::to_value(g.observe(viewer)).unwrap(),
                    serde_json::to_value(child.observe(viewer)).unwrap(),
                );
            }
            let after_choice = g.snapshot();
            g.restore(&before_choice).unwrap();
            act(&mut g, action);
            assert_eq!(g.snapshot(), after_choice);

            let returned = g.object_view(creature);
            assert!(projected.contains(&returned));
            assert_eq!(returned.zone, Zone::Battlefield);
            assert_eq!(returned.tapped, !pay);
            assert!(!returned.keywords.contains(Keywords::FLYING));
            assert!(!g.printed_abilities_available(creature));
            assert_eq!(
                g.state.players[player.index()].life,
                if pay { 18 } else { 20 }
            );
            drain(&mut g);
            assert_eq!(g.state.players[player.index()].library.len(), before - 1);

            g.add_counter(creature, CounterKind::MinusOne, 1);
            g.move_object(creature, Zone::Graveyard);
            g.prepare_triggers();
            drain(&mut g);
            assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
        }
    }
}

const KILL: CardRules = CardRules {
    effect: Effect::Spell(CREATURE, &[Instruction::Destroy]),
    ..CardRules::EMPTY
};
const DRAW_LOCK: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CantDrawCards],
    }],
    ..CardRules::EMPTY
};
const DRAW: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Each, 2),
            Instruction::LoseLife(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
#[test]
fn draw_prohibition_blocks_both_players_and_empty_library_attempts_then_ends_with_source() {
    let _s = fixture_rules::activate(&[(card("Short Bow"), &DRAW_LOCK), (card("Opt"), &DRAW)]);
    let mut g = scenario(Player::P0);
    let lock = put(&mut g, Player::P1, "Short Bow", Zone::Battlefield);
    g.state.players[0].library.clear();
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    drain(&mut g);
    assert!(!g.state.players[0].attempted_empty_draw);
    assert_eq!(g.state.players[0].life, 19);
    assert_eq!(g.hand_count(Player::P1), 0);
    g.move_object(lock, Zone::Graveyard);
    g.draw_many(Player::P1, 1);
    assert_eq!(g.hand_count(Player::P1), 1);
}
const RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseCreatureType,
            Instruction::ReturnChosenCreatureTypeFromGraveyard,
        ],
    ),
    ..CardRules::EMPTY
};
#[test]
fn chosen_type_return_is_simultaneous_and_filters_owner_and_creature() {
    let _s = fixture_rules::activate(&[(card("Opt"), &RETURN)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let b = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let foe = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        cast(&mut g, spell, None);
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::CreatureTypeChoice { .. })
        });
        act(
            &mut g,
            Action::ChooseCreatureType {
                creature_type: CreatureType::from_name("Bear").unwrap(),
            },
        );
        drain(&mut g);
        for id in [a, b] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        }
        for id in [elf, foe] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
    }
}
const DAMAGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        &[Instruction::DamagePlayerCreatures {
            players: PlayerSet::Targeted,
            amount: 2,
        }],
    ),
    ..CardRules::EMPTY
};
#[test]
fn damage_player_creatures_preserves_players_and_other_controllers() {
    let _s = fixture_rules::activate(&[(card("Opt"), &DAMAGE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let foe = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        drain(&mut g);
        assert_eq!(g.object_view(foe).damage, 2);
        assert_eq!(g.object_view(own).damage, 0);
        assert_eq!(g.state.players[0].life, 20);
        assert_eq!(g.state.players[1].life, 20);
    }
}
const ENTER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::EnteringPowerDamagePlayers(PlayerSet::Opponent)],
    }],
    ..CardRules::EMPTY
};
const BODY: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::LIFELINK,
    ..CardRules::EMPTY
};
#[test]
fn entering_power_damage_uses_entrant_lki_and_its_controller_for_lifelink() {
    let _s = fixture_rules::activate(&[
        (card("Short Bow"), &ENTER),
        (card("Bear Cub"), &BODY),
        (card("Shock"), &KILL),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    let entrant = put(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    cast(&mut g, entrant, None);
    ready(&mut g, |g| {
        g.state.stack.len() == 1 && g.object_view(entrant).zone == Zone::Battlefield
    });
    g.state.objects[entrant as usize].growth.power = 4;
    let kill = put(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, kill, Some(Target::Card(entrant)));
    drain(&mut g);
    assert_eq!(g.state.players[1].life, 13);
    assert_eq!(g.state.players[0].life, 27);
}
const DELAY: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[Instruction::DelayTargetEvent {
            event: DelayedTargetEvent::DiesOnce,
            ability: 0,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopChoosePlayScaled {
            amount: AmountValue::DeathEventPower,
            next_turn: true,
        }],
    }],
    ..CardRules::EMPTY
};
#[test]
fn delayed_death_power_is_frozen_before_graveyard_and_only_chosen_exile_card_is_playable() {
    let _s = fixture_rules::activate(&[(card("Opt"), &DELAY), (card("Shock"), &KILL)]);
    let mut g = scenario(Player::P0);
    let victim = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let watch = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, watch, Some(Target::Card(victim)));
    drain(&mut g);
    g.state.objects[victim as usize].growth.power = 3;
    let kill = put(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, kill, Some(Target::Card(victim)));
    ready(&mut g, |g| {
        g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::SelectMilled { .. }))
    });
    assert_eq!(g.state.players[0].library.len(), 3);
    choose(&mut g, |a| matches!(a, Action::SelectMilled { .. }));
    drain(&mut g);
    assert_eq!(g.state.exile_permissions.len(), 1);
    assert_eq!(
        g.state.exile_permissions[0].expiry,
        ExilePlayExpiry::EndOfNextTurn
    );
}
const COUNTER_DEATH: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileDeathCounterTotalUntilNextEnd],
    }],
    ..CardRules::EMPTY
};
#[test]
fn death_counter_total_keeps_full_width_and_bounds_work_by_library_length() {
    let _s = fixture_rules::activate(&[(card("Bear Cub"), &COUNTER_DEATH), (card("Shock"), &KILL)]);
    let mut g = scenario(Player::P0);
    let victim = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.add_counter(victim, CounterKind::PlusOne, 2);
    g.add_counter(victim, CounterKind::Charge, u64::MAX);
    let kill = put(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, kill, Some(Target::Card(victim)));
    drain(&mut g);
    assert!(g.state.players[0].library.is_empty());
    assert_eq!(g.state.exile_permissions.len(), 8);
    assert!(
        g.state
            .exile_permissions
            .iter()
            .all(|p| p.expiry == ExilePlayExpiry::StartOfNextEndStep(Player::P0))
    );
}
const MEEK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PutHandCreatureHasteSacrifice {
            maximum_total_stats: 5,
        }],
    ),
    ..CardRules::EMPTY
};
#[test]
fn hand_deployment_checks_total_stats_stays_private_and_delays_sacrifice() {
    let _s = fixture_rules::activate(&[(card("Opt"), &MEEK)]);
    let mut g = scenario(Player::P0);
    let small = put(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    let large = put(&mut g, Player::P0, "Hill Giant", Zone::Hand);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    assert!(g.effect_choice_view(Player::P1).is_none());
    let object = g.object_ref(large);
    reject(&mut g, Action::PutHandCreature { object });
    let object = g.object_ref(small);
    act(&mut g, Action::PutHandCreature { object });
    drain(&mut g);
    assert_eq!(g.object_view(small).zone, Zone::Battlefield);
    assert!(g.keywords(small).contains(Keywords::HASTE));
    assert_eq!(
        g.state
            .roles
            .as_ref()
            .unwrap()
            .keyword_effects
            .last()
            .unwrap()
            .expires_turn,
        None
    );
    for _ in 0..32 {
        if g.object_view(small).zone == Zone::Graveyard {
            break;
        }
        choose(&mut g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    assert_eq!(g.object_view(small).zone, Zone::Graveyard);
}
const CHOSEN_DOUBLE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(
            0,
            ReplacementSpec::DoubleControlledChosenTypeDamage,
        )],
    }],
    extras: &CardExtras {
        replacement_effects: &[ReplacementSpec::DoubleControlledChosenTypeDamage],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
#[test]
fn chosen_type_damage_doubler_composes_with_wither_before_counter_placement() {
    const WITHER: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        keywords: Keywords::WITHER,
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (card("Short Bow"), &CHOSEN_DOUBLE),
        (card("Bear Cub"), &WITHER),
    ]);
    let mut g = scenario(Player::P0);
    let provider = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    g.set_entry_value(
        g.object_ref(provider),
        EntryChoiceValue::CreatureType(CreatureType::from_name("Bear").unwrap()),
    );
    let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut g, Player::P1, "Gigantosaurus", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Bite Down", Zone::Hand);
    announce(&mut g, spell, &[Target::Card(source), Target::Card(foe)]);
    drain(&mut g);
    assert_eq!(g.counter_count(foe, CounterKind::MinusOne), 6);
    assert_eq!(g.object_view(foe).damage, 0);
}

#[test]
fn required_search_uses_selected_players_private_library_and_cannot_fail_to_find_any_card() {
    const SEARCH: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::SearchFor {
                player: PlayerSet::Opponent,
                required: true,
                spec: SearchSpec {
                    filter: SearchFilter::Any,
                    maximum: 1,
                    destination: SearchDestination::Hand,
                    tapped: false,
                    reveal: false,
                },
            }],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &SEARCH)]);
    let mut g = scenario(Player::P0);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
    assert!(g.observe(Player::P0).search_choice.is_none());
    assert!(g.observe(Player::P1).search_choice.is_some());
    reject(&mut g, Action::FinishSearch);
    choose(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
    if matches!(g.state.pending, Pending::EffectSearch { .. }) {
        act(&mut g, Action::FinishSearch);
    }
    drain(&mut g);
    assert_eq!(g.hand_count(Player::P1), 1);
    assert_eq!(g.hand_count(Player::P0), 0);
}

#[test]
fn hand_exile_keeps_apnap_selections_private_until_the_simultaneous_move() {
    const EXILE: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::ExileFromHand,
                players: PlayerSet::Each,
                filter: SacrificeFilter {
                    kinds: 0,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            }],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &EXILE)]);
    let mut parent = scenario(Player::P0);
    let a = put(&mut parent, Player::P0, "Bear Cub", Zone::Hand);
    let b = put(&mut parent, Player::P1, "Hill Giant", Zone::Hand);
    let spell = put(&mut parent, Player::P0, "Opt", Zone::Hand);
    cast(&mut parent, spell, None);
    ready(&mut parent, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    let object = parent.object_ref(a);
    act(&mut parent, Action::SelectResolutionExile { object });
    assert_eq!(parent.object_view(a).zone, Zone::Hand);
    assert!(parent.effect_choice_view(Player::P1).unwrap().selected[0].is_empty());
    let saved = parent.snapshot();
    let action = Action::SelectResolutionExile {
        object: parent.object_ref(b),
    };
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap();
    let revision = parent.revision();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut restored = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(Player::P0, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    raw.apply(Player::P1, action.clone()).unwrap();
    indexed.apply_index(Player::P1, revision, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    restored.restore(&saved).unwrap();
    act(&mut restored, action);
    drain(&mut restored);
    assert_eq!(raw.snapshot(), restored.snapshot());
    for id in [a, b] {
        assert_eq!(raw.object_view(id).zone, Zone::Exile);
    }
}

#[test]
fn delayed_death_power_trigger_remains_active_through_cleanup_state_based_actions() {
    // The accepted old branch expired these watches before cleanup SBAs. Keep
    // current CR514.3a behavior: the duration ends only when the turn advances.
    const LOPSIDED: CardRules = CardRules {
        effect: Effect::Creature(7, 3),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &DELAY), (card("Bear Cub"), &LOPSIDED)]);
    let mut g = scenario(Player::P0);
    let victim = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    for name in ["Giant Growth", "Blight Rot", "Opt"] {
        let spell = put(&mut g, Player::P0, name, Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(victim)));
        drain(&mut g);
    }
    assert_eq!(stats(&g, victim), (6, 2));
    for _ in 0..32 {
        if g.state.step == Step::End {
            break;
        }
        choose(&mut g, |a| {
            matches!(
                a,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::Cleanup);
    assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    ready(&mut g, |g| {
        g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::SelectMilled { .. }))
    });
    assert_eq!(g.state.players[0].library.len(), 5);
    choose(&mut g, |a| matches!(a, Action::SelectMilled { .. }));
    drain(&mut g);
    assert_eq!(g.state.exile_permissions.len(), 1);
}
