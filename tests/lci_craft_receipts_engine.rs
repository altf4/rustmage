//! Craft payment triggers, linked material characteristics, and combat entries.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const BLUE_RED_GREEN: Mana = [0, 1, 0, 1, 1, 0];
const ACTIVATION: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};
const EXILE_MANA_AND_CRAFT_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            exile_self: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Choice {
            mask: 1 | (1 << 3),
            amount: 1,
        })],
        ..ACTIVATION
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfExiledWhileCrafting,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GainLife(PlayerSet::Controller, 1),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
const CREATURE_CRAFT_TWO: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    ..CRAFT_TWO
};

const CRAFT_TWO: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            exile_self: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileCraftMaterials(CraftMaterials::Count {
                    minimum: 2,
                    maximum: Some(2),
                }),
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: true,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        program: &[Instruction::ReturnCraftedSourceTransformed],
        sorcery: true,
        ..ACTIVATION
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MATERIAL_READERS: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::CraftMaterialColors)],
        ..ACTIVATION
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::CraftMaterialManaValue,
        )],
    }],
    ..CardRules::EMPTY
};

fn announce(source: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card: source,
        ability,
        counter_payment: Counters::default(),
    }
}

fn choose_mana(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn select_materials(game: &mut Game, cards: &[CardId]) {
    act(game, Action::BeginOrderedCosts);
    let clause = game
        .observe(game.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause });
    for &card in cards {
        let object = game.object_ref(card);
        act(game, Action::ChooseAdditionalCost { object });
    }
}

fn finish_craft(game: &mut Game) {
    for _ in 0..8 {
        if game.state.announcing.is_none() {
            return;
        }
        match game.state.pending {
            Pending::CastCosts { .. } => choose(game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::PayCosts { .. } => pay(game),
            _ => panic!("unexpected Craft cost state {:?}", game.state.pending),
        }
    }
    panic!("Craft costs did not finish");
}

#[test]
fn nested_and_direct_battlefield_exile_each_queue_one_craft_trigger() {
    let observer = Card::from_name("Cartographer's Companion").unwrap();
    let _rules = fixture_rules::activate(&[(observer, &EXILE_MANA_AND_CRAFT_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = BLUE;
    let source = put(&mut game, player, "Braided Net", Zone::Battlefield);
    let mana = spawn(&mut game, player, observer, Zone::Battlefield);
    let material = spawn(&mut game, player, observer, Zone::Battlefield);
    let life = game.state.players[player.index()].life;
    let hand = game.hand_count(player);
    let before_craft = game.snapshot();

    let mut outside = game.clone();
    act(&mut outside, choose_mana(mana, [0; 6]));
    act(&mut outside, Action::ChooseMana { production: WHITE });
    settle(&mut outside);
    assert_eq!(outside.object_view(mana).zone, Zone::Exile);
    assert_eq!(outside.state.players[player.index()].life, life);
    assert_eq!(outside.hand_count(player), hand);

    act(&mut game, announce(source, 1));
    let start_mana = choose_mana(mana, [0; 6]);
    act(&mut game, start_mana);
    assert_eq!(game.object_view(mana).zone, Zone::Exile);
    assert_eq!(game.state.pending_triggers.len(), 1);
    assert_eq!(game.observe(player).suspended_mana_announcements.len(), 1);
    let snapshot = game.snapshot();
    let select_white = Action::ChooseMana { production: WHITE };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == select_white)
        .unwrap();
    let mut indexed = game.clone();
    let mut restored = outside;
    restored.restore(&before_craft).unwrap();
    assert_eq!(restored.object_view(mana).zone, Zone::Battlefield);
    assert!(restored.state.pending_triggers.is_empty());
    restored.restore(&snapshot).unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, select_white.clone());
    act(&mut restored, select_white);
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(restored.snapshot(), game.snapshot());

    select_materials(&mut game, &[material]);
    assert_eq!(game.object_view(material).zone, Zone::Exile);
    assert_eq!(game.state.pending_triggers.len(), 2);
    finish_craft(&mut game);
    for _ in 0..2 {
        if game.state.pending == Pending::Priority {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, life + 2);
    assert_eq!(game.hand_count(player), hand + 2);
    assert_eq!(
        game.crafted_exiled_cards(game.object_ref(source)),
        vec![game.object_ref(material)]
    );
}

#[test]
fn exact_materials_supply_live_mana_value_and_one_mana_per_color() {
    let front = Card::from_name("Braided Net").unwrap();
    let back = Card::from_name("Braided Quipu").unwrap();
    let _rules = fixture_rules::activate(&[(front, &CRAFT_TWO), (back, &MATERIAL_READERS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, front, Zone::Battlefield);
    let dragon = put(&mut game, player, "Niv-Mizzet, Visionary", Zone::Graveyard);
    let elf = put(&mut game, player, "Llanowar Elves", Zone::Graveyard);
    let life = game.state.players[player.index()].life;
    act(&mut game, announce(source, 0));
    select_materials(&mut game, &[dragon, elf]);
    finish_craft(&mut game);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(source).card, back);
    let crafted = game.object_ref(source);
    assert_eq!(
        game.stat_value(StatValue::CraftMaterialColorCount, crafted, player),
        3
    );

    let mut stale = game.clone();
    let exiled_dragon = stale.object_ref(dragon);
    stale.move_object(dragon, Zone::Graveyard);
    stale.move_object(dragon, Zone::Exile);
    assert_ne!(stale.object_ref(dragon), exiled_dragon);
    assert_eq!(
        stale.stat_value(StatValue::CraftMaterialColorCount, crafted, player),
        1
    );
    let mut departed = game.clone();
    departed.move_object(source, Zone::Graveyard);
    assert_ne!(departed.object_ref(source), crafted);
    settle(&mut game);
    settle(&mut stale);
    settle(&mut departed);
    assert_eq!(game.state.players[player.index()].life, life + 7);
    assert_eq!(stale.state.players[player.index()].life, life + 1);
    assert_eq!(departed.state.players[player.index()].life, life + 7);
    for (mut branch, expected) in [(game, BLUE_RED_GREEN), (stale, GREEN)] {
        branch.state.players[player.index()].mana = [0; 6];
        reject(&mut branch, choose_mana(source, [0, 0, 0, 0, 0, 1]));
        act(&mut branch, choose_mana(source, expected));
        assert_eq!(branch.state.players[player.index()].mana, expected);
    }
}

#[test]
fn crafted_creature_returns_tapped_against_a_separately_chosen_defender() {
    const NONLAND: SacrificeFilter = SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 0,
        nonland: true,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    };
    const CAPTAIN: CardRules = CardRules {
        effect: Effect::Creature(7, 7),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Opponent,
                    filter: NONLAND,
                    amount: 1,
                    optional_skip: None,
                },
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::ReturnCraftedCreatureAttacking,
                    players: PlayerSet::Controller,
                    filter: NONLAND,
                    amount: 1,
                    optional_skip: Some(0),
                },
            ],
        }],
        ..CardRules::EMPTY
    };
    const DONOR: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        }],
        ..CardRules::EMPTY
    };
    let front = Card::from_name("Cecil, Dark Knight").unwrap();
    let back = front.back_face().unwrap();
    let _rules = fixture_rules::activate(&[
        (front, &CREATURE_CRAFT_TWO),
        (back, &CAPTAIN),
        (Card::GrizzlyBears, &DONOR),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, front, Zone::Battlefield);
    let creature = spawn(&mut game, opponent, Card::GrizzlyBears, Zone::Battlefield);
    game.state.objects[creature as usize].controller = player;
    let noncreature = put(&mut game, player, "Opt", Zone::Graveyard);
    let unrelated = put(&mut game, player, "Llanowar Elves", Zone::Exile);
    let victim = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let walker = put(
        &mut game,
        opponent,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 4);
    let life = game.state.players[player.index()].life;
    act(&mut game, announce(source, 0));
    select_materials(&mut game, &[creature, noncreature]);
    finish_craft(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(source).card, back);
    let creature_ref = game.object_ref(creature);
    let victim_ref = game.object_ref(victim);
    let walker_ref = game.object_ref(walker);

    game.state.objects[source as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    assert!(game.state.stack.last().unwrap().targets.is_empty());

    let mut stale = game.clone();
    stale.move_object(creature, Zone::Graveyard);
    stale.move_object(creature, Zone::Exile);
    let mut departed = game.clone();
    departed.move_object(source, Zone::Graveyard);
    for branch in [&mut game, &mut stale, &mut departed] {
        act(branch, Action::Pass);
        act(branch, Action::Pass);
        assert_eq!(branch.actor(), Some(opponent));
        act(branch, Action::Sacrifice { object: victim_ref });
        assert_eq!(branch.object_view(victim).zone, Zone::Graveyard);
    }
    assert_eq!(stale.state.pending, Pending::Priority);
    assert_eq!(stale.object_view(creature).zone, Zone::Exile);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(creature).zone, Zone::Exile);
    assert_eq!(game.actor(), Some(player));
    for id in [noncreature, unrelated] {
        let object = game.object_ref(id);
        reject(&mut game, Action::ChooseEffectObject { object });
    }
    for branch in [&mut game, &mut departed] {
        act(
            branch,
            Action::ChooseEffectObject {
                object: creature_ref,
            },
        );
        assert_eq!(
            branch.state.pending,
            Pending::EntryAttackingDefender { player }
        );
        let defender = CombatDefender::Planeswalker(walker_ref);
        let choose_defender = Action::ChooseEntryAttackingDefender { defender };
        let index = branch
            .legal_actions()
            .iter()
            .position(|action| *action == choose_defender)
            .unwrap();
        let mut indexed = branch.clone();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(branch, choose_defender);
        assert_eq!(indexed.snapshot(), branch.snapshot());
        let entrant = branch.object_view(creature);
        assert_eq!(entrant.controller, player);
        assert_eq!(entrant.owner, opponent);
        assert!(entrant.tapped);
        assert_eq!(
            branch
                .state
                .attackers
                .iter()
                .find(|row| row.card == creature)
                .unwrap()
                .defender,
            Some(defender)
        );
        assert!(branch.state.stack.is_empty());
        assert_eq!(branch.state.players[player.index()].life, life);
    }
}

#[test]
fn crafted_power_uses_exile_cdas_and_signed_physical_card_power() {
    const SOULS_STATS: DynamicStats = DynamicStats {
        power: Some(StatExpr {
            value: StatValue::Count(CountFilter {
                zone: CountZone::Graveyard,
                relation: 0,
                kind: 1 | 2 | 16 | 32 | 64 | 256,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: true,
            }),
            multiplier: 1,
            offset: 0,
        }),
        toughness: None,
    };
    const RAPTOR_STATS: DynamicStats = DynamicStats {
        power: Some(StatExpr {
            value: StatValue::CraftMaterialPower,
            multiplier: 1,
            offset: 0,
        }),
        toughness: None,
    };
    const SOULS_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, SOULS_STATS)],
    }];
    const RAPTOR_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, RAPTOR_STATS)],
    }];
    const SOULS: CardRules = CardRules {
        effect: Effect::Creature(0, 1),
        static_groups: SOULS_GROUPS,
        extras: &CardExtras {
            static_ability_groups: SOULS_GROUPS,
            dynamic_stats: Some(SOULS_STATS),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const RAPTOR: CardRules = CardRules {
        effect: Effect::Creature(0, 4),
        static_groups: RAPTOR_GROUPS,
        extras: &CardExtras {
            static_ability_groups: RAPTOR_GROUPS,
            dynamic_stats: Some(RAPTOR_STATS),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const NEGATIVE: CardRules = CardRules {
        effect: Effect::Creature(-3, 4),
        ..CardRules::EMPTY
    };
    let front = Card::from_name("Garland, Knight of Cornelia").unwrap();
    let back = front.back_face().unwrap();
    let souls_card = Card::from_name("Souls of the Lost").unwrap();
    let dinosaur = Card::from_name("Colossadactyl").unwrap();
    let _rules = fixture_rules::activate(&[
        (front, &CREATURE_CRAFT_TWO),
        (back, &RAPTOR),
        (souls_card, &SOULS),
        (dinosaur, &NEGATIVE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, front, Zone::Battlefield);
    let souls = spawn(&mut game, player.other(), souls_card, Zone::Battlefield);
    let witness = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    let negative = spawn(&mut game, player, dinosaur, Zone::Graveyard);
    let vehicle = put(&mut game, player, "Detention Chariot", Zone::Graveyard);
    let vehicle_power = game.object_view(vehicle).card.vehicle_stats().unwrap().0;
    let souls_ref = game.object_ref(souls);
    game.change_controller(souls_ref, player);
    game.apply_temporary_copy(
        souls_ref,
        CopyRecipe::new(dinosaur),
        CopyDeadline::EndOfTurn,
    );
    game.state.objects[souls as usize].growth.power = 9;
    assert_eq!(stats(&game, souls).0, 6);
    act(&mut game, announce(source, 0));
    let mut printed_vehicle = game.clone();
    select_materials(&mut printed_vehicle, &[souls, vehicle]);
    finish_craft(&mut printed_vehicle);
    settle(&mut printed_vehicle);
    assert_eq!(stats(&printed_vehicle, source), (1 + vehicle_power, 4));

    select_materials(&mut game, &[souls, negative]);
    finish_craft(&mut game);
    assert_eq!(game.object_view(souls).card, souls_card);
    let mut projection = game.entry_move(source, Zone::Battlefield, false);
    projection.project_printed_face(back);
    projection.craft_materials = Some(Box::new(vec![
        game.object_ref(souls),
        game.object_ref(negative),
    ]));
    let preview = game.ghost_entry_preview(&projection).unwrap();
    assert_eq!((preview.power, preview.toughness), (-2, 4));
    settle(&mut game);
    assert_eq!(game.object_view(source).card, back);
    assert_eq!(stats(&game, source), (-2, 4));
    game.move_object(witness, Zone::Exile);
    assert_eq!(stats(&game, source), (-3, 4));
    game.move_object(negative, Zone::Graveyard);
    game.move_object(negative, Zone::Exile);
    assert_eq!(stats(&game, source), (0, 4));
}
