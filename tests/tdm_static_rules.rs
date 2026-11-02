//! Shared rules for TDM static restrictions, attack multiplicity and subtype grants.
//! Engine fixtures only: these are not card admissions.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const MANA: &[Instruction] = &[Instruction::AddMana(ManaProduction::Fixed([
    0, 0, 0, 0, 0, 1,
]))];
const LIFE: &[Instruction] = &[Instruction::GainLife(PlayerSet::Controller, 3)];
const fn ability(
    cost: ActivationCost,
    origin: ActivationOrigin,
    guard: ActivationGuard,
    program: &'static [Instruction],
) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard,
        origin,
        cycling: false,
        cost,
        targets: TargetPlan::NONE,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const ABILITIES: &[ActivatedAbility] = &[
    ability(
        ActivationCost::ZERO,
        ActivationOrigin::Battlefield,
        ActivationGuard::Always,
        LIFE,
    ),
    ability(
        ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        ActivationOrigin::Battlefield,
        ActivationGuard::Always,
        MANA,
    ),
    ability(
        ActivationCost::ZERO,
        ActivationOrigin::Hand,
        ActivationGuard::Always,
        LIFE,
    ),
    ability(
        ActivationCost::ZERO,
        ActivationOrigin::Graveyard,
        ActivationGuard::Always,
        LIFE,
    ),
];
const fn abilities(effect: Effect) -> CardRules {
    CardRules {
        effect,
        activated: ABILITIES,
        ..CardRules::EMPTY
    }
}
const UNIT: CardRules = abilities(Effect::Creature(4, 4));
const PERMANENT: CardRules = abilities(Effect::Permanent);
const LOCK: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ActivatedPermanentKindsProhibited(
            1 | 16 | 64,
        )],
    }],
    ..CardRules::EMPTY
};
const LOCK_FUEL: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: LOCK.static_groups,
    activated: &[ability(
        ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        ActivationOrigin::Battlefield,
        ActivationGuard::Always,
        MANA,
    )],
    ..CardRules::EMPTY
};
const ANCHOR: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ability(
        ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        ActivationOrigin::Battlefield,
        ActivationGuard::OwnTurnAfterGraveyardDeparture,
        LIFE,
    )],
    ..CardRules::EMPTY
};
const EQUIPMENT_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::AttachmentKeywords(
        Keywords::HEXPROOF_MONOCOLORED,
    )],
}];
const EQUIPMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: EQUIPMENT_GROUPS,
    extras: &CardExtras {
        static_ability_groups: EQUIPMENT_GROUPS,
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::HEXPROOF_MONOCOLORED,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const BOLT: CardRules = CardRules {
    effect: Effect::Spell(CREATURE, &[Instruction::Damage(1)]),
    ..CardRules::EMPTY
};
const BLANK: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
    ),
    ..CardRules::EMPTY
};
const GRANT: KeywordGrant = KeywordGrant {
    condition: Some(StatValue::ControllerEndStep),
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: Some("Warrior"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: Some(true),
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::CANT_BE_SACRIFICED,
};
const ZURGO: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, GRANT)],
    }],
    extras: &CardExtras {
        keyword_grants: &[GRANT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ZURGO_FUEL: CardRules = CardRules {
    activated: &[ability(
        ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        ActivationOrigin::Battlefield,
        ActivationGuard::Always,
        MANA,
    )],
    ..ZURGO
};
const TOKEN: Card = Card::test_index(5255);
const SAC_TOKEN: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ability(
        ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        ActivationOrigin::Battlefield,
        ActivationGuard::Always,
        MANA,
    )],
    ..CardRules::EMPTY
};
const MODES: &[&str] = &["Jeskai", "Mardu"];
const MULTIPLIER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::EntryMode(0),
            StaticAbilityFragment::AdditionalTriggers {
                scope: AdditionalTriggerScope::ControlledPermanentAttack,
                condition: StatValue::Fixed(1),
            },
        ],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NamedModes(MODES)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ATTACKER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::DECAYED,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: LIFE,
    }],
    ..CardRules::EMPTY
};
const WATCHER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ControlledCreatureAttacks,
            targets: TargetPlan::NONE,
            program: LIFE,
        },
        TriggeredAbility {
            condition: TriggerCondition::CreaturesAttackYou,
            targets: TargetPlan::NONE,
            program: LIFE,
        },
    ],
    ..CardRules::EMPTY
};
const SUBTYPE: AnimationSpec = AnimationSpec {
    artifact: false,
    creature_types: AnimationCreatureTypes::Additional(&["Dragon"]),
    base: None,
    keywords: Keywords::NONE,
    duration: AnimationDuration::EndOfTurn,
};
const ASCEND: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ability(
        ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        ActivationOrigin::Battlefield,
        ActivationGuard::Always,
        &[Instruction::ChangeSourceCreatureForm(SUBTYPE)],
    )],
    ..CardRules::EMPTY
};
fn goal(p: Player) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0; 6],
        generic: 2,
        reserve: None,
        source: None,
    }
}
fn announce(id: CardId, index: u16) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: index,
        counter_payment: Counters::default(),
    }
}
fn activate(g: &mut Game, id: CardId, index: u16) {
    act(g, announce(id, index));
    pay(g);
}
fn mana(g: &mut Game, id: CardId, index: u16) {
    act(
        g,
        Action::AnnounceMana {
            card: id,
            ability: index,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    pay(g);
}
fn suppress(g: &mut Game, id: CardId) {
    let source = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .resolved_ability_modifications
        .push(ResolvedAbilityModificationView {
            recipient: source,
            source,
            keywords: Keywords::NONE,
            timestamp,
        });
}
fn paint(g: &mut Game, id: CardId, colors: u8) {
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    g.state
        .roles
        .as_mut()
        .unwrap()
        .animations
        .last_mut()
        .unwrap()
        .colors = Some(colors);
}
fn multiplier(g: &mut Game, p: Player, mode: u16) -> CardId {
    let id = put(g, p, "Anthem of Champions", Zone::Battlefield);
    g.set_entry_value(g.object_ref(id), EntryChoiceValue::NamedMode(mode));
    assert_eq!(g.object_view(id).active_entry_mode, Some(mode));
    id
}
fn declare(g: &mut Game, id: CardId) {
    g.state.pending = Pending::Attackers;
    g.state.step = Step::Attackers;
    act(g, Action::Attack { card: id });
    act(g, Action::FinishAttackers);
}
fn packets(g: &Game, kind: impl Fn(&SpellView) -> bool) -> usize {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .filter(|i| kind(i))
        .count()
}
fn drain(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..128 {
        if matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(g, |a| matches!(a, Action::PutTrigger { .. }));
        } else if g.state.stack.is_empty() && g.state.resolving.is_none() {
            return;
        } else {
            act(g, Action::Pass);
        }
    }
    panic!("trigger draining did not finish");
}

#[test]
fn global_lock_covers_both_players_mana_and_nonmana_but_only_matching_battlefield_kinds() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &LOCK),
        (Card::HillGiant, &UNIT),
        (Card::from_name("Short Bow").unwrap(), &PERMANENT),
        (
            Card::from_name("Elspeth, Storm Slayer").unwrap(),
            &PERMANENT,
        ),
        (Card::from_name("Anthem of Champions").unwrap(), &PERMANENT),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let lock = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        for owner in [p, p.other()] {
            for name in ["Hill Giant", "Short Bow", "Elspeth, Storm Slayer"] {
                let id = put(&mut g, owner, name, Zone::Battlefield);
                if name == "Elspeth, Storm Slayer" {
                    g.add_counter(id, CounterKind::Loyalty, 5);
                }
                assert!(CharacteristicWorld::activation_prohibited(&g, owner, id));
                let w = FundingState::from_game(&g, &goal(owner));
                assert!(w.activation_prohibited(owner, id));
                assert!(
                    !w.moves(&goal(owner))
                        .iter()
                        .any(|m| w.resources[m.resource].id == id)
                );
                if owner == p {
                    reject(&mut g, announce(id, 0));
                }
            }
        }
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let enchant = put(&mut g, p, "Anthem of Champions", Zone::Battlefield);
        assert!(!CharacteristicWorld::activation_prohibited(&g, p, land));
        assert!(!CharacteristicWorld::activation_prohibited(&g, p, enchant));
        activate(&mut g, enchant, 0);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 23);
        for (zone, index) in [(Zone::Hand, 2), (Zone::Graveyard, 3)] {
            let id = put(&mut g, p, "Hill Giant", zone);
            activate(&mut g, id, index);
            settle(&mut g);
        }
        assert_eq!(g.state.players[p.index()].life, 29);
        suppress(&mut g, lock);
        let id = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        activate(&mut g, id, 0);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 32);
    }
}

#[test]
fn projected_sacrifice_of_global_lock_unlocks_later_creature_mana_without_mutating_parent() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Anthem of Champions").unwrap(), &LOCK_FUEL),
        (Card::HillGiant, &UNIT),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let lock = put(&mut g, p, "Anthem of Champions", Zone::Battlefield);
        let unit = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let saved = g.snapshot();
        let request = goal(p);
        let world = FundingState::from_game(&g, &request);
        assert!(world.activation_prohibited(p, unit));
        let step = world
            .moves(&request)
            .into_iter()
            .find(|m| world.resources[m.resource].id == lock)
            .unwrap();
        let mut visited = false;
        world.visit_applied(&step, &mut |next| {
            assert_eq!(next.object(lock).zone, Zone::Graveyard);
            assert!(!next.activation_prohibited(p, unit));
            assert!(
                next.moves(&request)
                    .iter()
                    .any(|m| next.resources[m.resource].id == unit)
            );
            visited = true;
            false
        });
        assert!(visited);
        assert_eq!(g.snapshot(), saved);
        mana(&mut g, lock, 0);
        mana(&mut g, unit, 1);
        assert_eq!(g.state.players[p.index()].mana[5], 2);
    }
}

#[test]
fn activation_lock_tracks_effective_creature_types_and_ends_with_provider_departure() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &LOCK),
        (Card::from_name("Anthem of Champions").unwrap(), &PERMANENT),
    ]);
    let mut g = scenario(Player::P0);
    let lock = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let id = put(&mut g, Player::P0, "Anthem of Champions", Zone::Battlefield);
    assert!(!CharacteristicWorld::activation_prohibited(
        &g,
        Player::P0,
        id
    ));
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            base: Some((3, 3)),
            ..SUBTYPE
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
    reject(&mut g, announce(id, 0));
    g.move_object(lock, Zone::Hand);
    activate(&mut g, id, 0);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 23);
}

#[test]
fn anchor_requires_both_own_turn_and_actual_own_graveyard_departure_before_announcement() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &ANCHOR)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = put(&mut g, p, "Short Bow", Zone::Battlefield);
        reject(&mut g, announce(id, 0));
        let enemy = put(&mut g, p.other(), "Forest", Zone::Graveyard);
        g.move_object(enemy, Zone::Exile);
        reject(&mut g, announce(id, 0));
        let own = put(&mut g, p, "Forest", Zone::Graveyard);
        g.move_object(own, Zone::Hand);
        g.state.active = p.other();
        reject(&mut g, announce(id, 0));
        g.state.active = p;
        assert!(g.activation_guard_available(
            ActivationGuard::OwnTurnAfterGraveyardDeparture,
            id,
            p
        ));
        let world = FundingState::from_game(&g, &goal(p));
        assert!(world.card_left_graveyard(p));
        activate(&mut g, id, 0);
        g.state.active = p.other();
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].life,
            23,
            "starting guard is not a resolution condition"
        );
        g.state.active = p;
        g.begin_turn();
        assert!(!g.state.players[p.index()].card_left_graveyard_this_turn);
    }
}

#[test]
fn equipped_hexproof_blocks_only_opposing_exactly_one_color_sources_with_live_colors() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &EQUIPMENT),
        (Card::LightningBolt, &BOLT),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let equip = put(&mut g, p, "Short Bow", Zone::Battlefield);
        g.state.objects[equip as usize].attached_to = Some(g.object_ref(target));
        for owner in [p, p.other()] {
            let source = put(&mut g, owner, "Lightning Bolt", Zone::Battlefield);
            for colors in [0, 1, 2, 4, 8, 16, 3, 31] {
                paint(&mut g, source, colors);
                assert_eq!(
                    g.cast_target_matches(
                        TargetRule::Creature,
                        owner,
                        g.object_ref(source),
                        Target::Card(target)
                    ),
                    owner == p || colors.count_ones() != 1
                );
            }
        }
        assert!(g.keywords(target).contains(Keywords::HEXPROOF_MONOCOLORED));
        suppress(&mut g, target);
        assert!(!g.keywords(target).contains(Keywords::HEXPROOF_MONOCOLORED));
    }
}

#[test]
fn hexproof_checks_selected_omen_face_and_resolved_ability_source_lki() {
    const OMEN: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        extras: &CardExtras {
            omen_spell: Some(Card::LightningBolt),
            adventure_identity: Some("@test/mono-hexproof-omen"),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const SHOOT: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            targets: CREATURE,
            program: &[Instruction::Damage(1)],
            ..ability(
                ActivationCost::ZERO,
                ActivationOrigin::Battlefield,
                ActivationGuard::Always,
                LIFE,
            )
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &EQUIPMENT),
        (Card::from_name("Progenitus").unwrap(), &OMEN),
        (Card::LightningBolt, &BOLT),
        (Card::LlanowarElves, &SHOOT),
    ]);
    let mut g = scenario(Player::P0);
    let target = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let equip = put(&mut g, Player::P1, "Short Bow", Zone::Battlefield);
    g.state.objects[equip as usize].attached_to = Some(g.object_ref(target));
    let source = put(&mut g, Player::P0, "Progenitus", Zone::Hand);
    assert!(!g.cast_definition_targets_available(
        CREATURE,
        Player::P0,
        g.object_ref(source),
        Card::LightningBolt
    ));
    let shooter = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    paint(&mut g, shooter, 3);
    act(&mut g, announce(shooter, 0));
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(&mut g);
    paint(&mut g, shooter, 4);
    g.move_object(shooter, Zone::Hand);
    settle(&mut g);
    assert_eq!(
        g.state.objects[target as usize].damage, 0,
        "mono last-known source loses its only legal target"
    );
}

#[test]
fn conditional_sacrifice_grant_filters_token_subtype_controller_and_step_in_both_worlds() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &ZURGO), (TOKEN, &SAC_TOKEN)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.create_tokens(p, TOKEN, 1, false);
        g.create_tokens(p.other(), TOKEN, 1, false);
        let own = g
            .state
            .objects
            .iter()
            .position(|o| o.card == TOKEN && o.controller == p)
            .unwrap() as CardId;
        let enemy = g
            .state
            .objects
            .iter()
            .position(|o| o.card == TOKEN && o.controller != p)
            .unwrap() as CardId;
        assert!(!CharacteristicWorld::sacrifice_prohibited(&g, own));
        g.state.step = Step::End;
        assert!(CharacteristicWorld::sacrifice_prohibited(&g, own));
        assert!(!CharacteristicWorld::sacrifice_prohibited(&g, enemy));
        let world = FundingState::from_game(&g, &goal(p));
        assert!(world.sacrifice_prohibited(own));
        assert!(
            !world
                .moves(&goal(p))
                .iter()
                .any(|m| world.resources[m.resource].id == own)
        );
        reject(
            &mut g,
            Action::AnnounceMana {
                card: own,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        suppress(&mut g, provider);
        assert!(!CharacteristicWorld::sacrifice_prohibited(&g, own));
        mana(&mut g, own, 0);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Outside);
    }
}

#[test]
fn removing_recipient_abilities_ends_conditional_prohibition_until_a_later_grant() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &ZURGO),
        (TOKEN, &SAC_TOKEN),
        (Card::from_name("Opt").unwrap(), &BLANK),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.create_tokens(Player::P0, TOKEN, 1, false);
    let id = g
        .state
        .objects
        .iter()
        .position(|o| o.card == TOKEN)
        .unwrap() as CardId;
    g.state.step = Step::End;
    assert!(CharacteristicWorld::sacrifice_prohibited(&g, id));
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(id)));
    settle(&mut g);
    assert!(!CharacteristicWorld::sacrifice_prohibited(&g, id));
    put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    assert!(
        CharacteristicWorld::sacrifice_prohibited(&g, id),
        "later continuous grant can restore the granted ability"
    );
}

#[test]
fn sacrificing_provider_in_projected_payment_unlocks_warrior_sacrifice_for_later_mana() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &ZURGO_FUEL), (TOKEN, &SAC_TOKEN)]);
    let mut g = scenario(Player::P0);
    let provider = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.create_tokens(Player::P0, TOKEN, 1, false);
    let token = g
        .state
        .objects
        .iter()
        .position(|o| o.card == TOKEN)
        .unwrap() as CardId;
    g.state.step = Step::End;
    g.state.players[0].mana = [0; 6];
    let request = goal(Player::P0);
    let saved = g.snapshot();
    let world = FundingState::from_game(&g, &request);
    let step = world
        .moves(&request)
        .into_iter()
        .find(|m| world.resources[m.resource].id == provider)
        .unwrap();
    let mut visited = false;
    world.visit_applied(&step, &mut |next| {
        assert!(!next.sacrifice_prohibited(token));
        assert!(
            next.moves(&request)
                .iter()
                .any(|m| next.resources[m.resource].id == token)
        );
        visited = true;
        false
    });
    assert!(visited);
    assert_eq!(g.snapshot(), saved);
    mana(&mut g, provider, 0);
    mana(&mut g, token, 0);
    assert_eq!(g.state.players[0].mana[5], 2);
}

#[test]
fn named_attack_multiplier_duplicates_attack_half_and_each_decayed_instance_without_etb_leakage() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Anthem of Champions").unwrap(), &MULTIPLIER),
        (Card::HillGiant, &ATTACKER),
    ]);
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = scenario(p);
            multiplier(&mut g, p, mode);
            let source = put(&mut g, p, "Hill Giant", Zone::Hand);
            g.move_object(source, Zone::Battlefield);
            drain(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                23,
                "ETB half gets no attack multiplier"
            );
            g.state.objects[source as usize].summoning_sick = false;
            g.add_counter(source, CounterKind::Decayed, 1);
            declare(&mut g, source);
            assert_eq!(
                packets(&g, |i| matches!(i.kind, StackKind::DecayedAttack)),
                if mode == 0 { 4 } else { 2 }
            );
            assert_eq!(
                packets(&g, |i| matches!(i.kind, StackKind::TriggeredAbility { .. })),
                if mode == 0 { 2 } else { 1 }
            );
            drain(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if mode == 0 { 29 } else { 26 }
            );
        }
    }
}

#[test]
fn attack_multiplier_applies_to_controlled_watchers_of_either_players_attack_but_not_enemy_watchers()
 {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Anthem of Champions").unwrap(), &MULTIPLIER),
        (Card::from_name("Short Bow").unwrap(), &WATCHER),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        multiplier(&mut g, p, 0);
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        let attacker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.active = p.other();
        g.state.priority = p.other();
        declare(&mut g, attacker);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 26);
        assert_eq!(g.state.players[p.other().index()].life, 23);
    }
}

#[test]
fn attack_multiplier_is_frozen_on_packets_survives_source_loss_and_raw_index_restore() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Anthem of Champions").unwrap(), &MULTIPLIER),
        (Card::HillGiant, &ATTACKER),
    ]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let siege = multiplier(&mut parent, p, 0);
    let source = put(&mut parent, p, "Hill Giant", Zone::Battlefield);
    parent.state.pending = Pending::Attackers;
    parent.state.step = Step::Attackers;
    act(&mut parent, Action::Attack { card: source });
    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == Action::FinishAttackers)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    raw.apply(p, Action::FinishAttackers).unwrap();
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    raw.move_object(siege, Zone::Hand);
    indexed.move_object(siege, Zone::Hand);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.players[p.index()].life, 26);
    indexed.restore(&saved).unwrap();
    indexed.apply(p, Action::FinishAttackers).unwrap();
    indexed.move_object(siege, Zone::Hand);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn additive_dragon_type_preserves_original_types_base_and_copy_recipe_until_cleanup() {
    let _scope = fixture_rules::activate(&[(Card::LlanowarElves, &ASCEND)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let recipe = g.copy_recipe(g.object_ref(source));
        activate(&mut g, source, 0);
        settle(&mut g);
        assert!(
            g.has_subtype(source, "Elf")
                && g.has_subtype(source, "Druid")
                && g.has_subtype(source, "Dragon")
        );
        assert_eq!(stats(&g, source), (2, 2));
        assert_eq!(g.copy_recipe(g.object_ref(source)), recipe);
        let world = FundingState::from_game(&g, &goal(p));
        assert!(world.subtypes(source).has(Card::LlanowarElves, "Dragon"));
        assert!(
            g.observe(p)
                .animations
                .iter()
                .any(|a| a.object == g.object_ref(source) && a.additional_creature_types)
        );
        suppress(&mut g, source);
        assert!(
            g.has_subtype(source, "Dragon"),
            "resolved type effect survives ability loss"
        );
        g.state.turn += 1;
        g.expire_animations();
        assert!(!g.has_subtype(source, "Dragon"));
        assert!(g.has_subtype(source, "Druid"));
    }
}

#[test]
fn additive_subtype_obeys_later_replacement_timestamp_and_exact_object_incarnation() {
    let _scope = fixture_rules::activate(&[(Card::LlanowarElves, &ASCEND)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    activate(&mut g, source, 0);
    settle(&mut g);
    g.change_object_form(
        g.object_ref(source),
        AnimationSpec {
            creature_types: AnimationCreatureTypes::Named(&["Frog"]),
            ..SUBTYPE
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    assert!(g.has_subtype(source, "Frog"));
    assert!(!g.has_subtype(source, "Dragon"));
    assert!(!g.has_subtype(source, "Druid"));
    activate(&mut g, source, 0);
    settle(&mut g);
    assert!(g.has_subtype(source, "Frog") && g.has_subtype(source, "Dragon"));
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    assert!(g.has_subtype(source, "Druid"));
    assert!(!g.has_subtype(source, "Dragon"));
}

#[test]
fn pending_additive_subtype_cannot_affect_a_returned_source_incarnation() {
    let _scope = fixture_rules::activate(&[(Card::LlanowarElves, &ASCEND)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    activate(&mut g, source, 0);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    settle(&mut g);
    assert!(!g.has_subtype(source, "Dragon"));
    assert!(g.has_subtype(source, "Druid"));
}

#[test]
fn attack_extra_respects_printed_once_per_turn_even_with_two_providers() {
    const LIMITED: CardRules = CardRules {
        effect: Effect::Creature(4, 4),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::SelfAttacks),
            targets: TargetPlan::NONE,
            program: LIFE,
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Anthem of Champions").unwrap(), &MULTIPLIER),
        (Card::HillGiant, &LIMITED),
    ]);
    let mut g = scenario(Player::P0);
    multiplier(&mut g, Player::P0, 0);
    multiplier(&mut g, Player::P0, 0);
    let attacker = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    declare(&mut g, attacker);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 23);
    g.state.attackers.clear();
    g.state.objects[attacker as usize].tapped = false;
    declare(&mut g, attacker);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 23);
}

#[test]
fn multiple_attack_providers_add_instances_and_suppression_or_control_loss_removes_only_its_own() {
    const SIMPLE: CardRules = CardRules {
        effect: Effect::Creature(4, 4),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: LIFE,
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Anthem of Champions").unwrap(), &MULTIPLIER),
        (Card::HillGiant, &SIMPLE),
    ]);
    for expected in [1, 2, 3] {
        let mut g = scenario(Player::P0);
        let first = multiplier(&mut g, Player::P0, 0);
        let second = multiplier(&mut g, Player::P0, 0);
        if expected < 3 {
            suppress(&mut g, first);
        }
        if expected < 2 {
            g.record_control(g.object_ref(second), Player::P1, None);
        }
        let attacker = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        declare(&mut g, attacker);
        drain(&mut g);
        assert_eq!(g.state.players[0].life, 20 + 3 * expected);
    }
}

#[test]
fn attack_only_multiplier_does_not_duplicate_tap_or_combat_damage() {
    const EVENTS: CardRules = CardRules {
        effect: Effect::Creature(4, 4),
        triggers: &[
            TriggeredAbility {
                condition: TriggerCondition::SelfBecomesTapped,
                targets: TargetPlan::NONE,
                program: LIFE,
            },
            TriggeredAbility {
                condition: TriggerCondition::PlayerDamaged {
                    subject: DamageSubject::Source,
                    combat_only: true,
                    opponent_only: true,
                    one_or_more: false,
                },
                targets: TargetPlan::NONE,
                program: LIFE,
            },
        ],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Anthem of Champions").unwrap(), &MULTIPLIER),
        (Card::HillGiant, &EVENTS),
    ]);
    let mut g = scenario(Player::P0);
    multiplier(&mut g, Player::P0, 0);
    let source = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    declare(&mut g, source);
    drain(&mut g);
    assert_eq!(
        g.state.players[0].life, 23,
        "attack tap is a separate event"
    );
    g.apply_damage_batch(&[DamageEvent {
        source: g.damage_source(source),
        target: Target::Player(Player::P1),
        amount: 1,
        preventable: true,
        combat: true,
    }]);
    drain(&mut g);
    assert_eq!(
        g.state.players[0].life, 26,
        "combat damage is a separate event"
    );
}

#[test]
fn conditional_sacrifice_grant_tracks_opponent_end_step_control_and_current_warrior_type() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &ZURGO), (TOKEN, &SAC_TOKEN)]);
    let mut g = scenario(Player::P0);
    let provider = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.create_tokens(Player::P0, TOKEN, 1, false);
    let token = g
        .state
        .objects
        .iter()
        .position(|o| o.card == TOKEN)
        .unwrap() as CardId;
    g.state.step = Step::End;
    assert!(CharacteristicWorld::sacrifice_prohibited(&g, token));
    g.state.active = Player::P1;
    assert!(!CharacteristicWorld::sacrifice_prohibited(&g, token));
    g.record_control(g.object_ref(provider), Player::P1, None);
    assert!(!CharacteristicWorld::sacrifice_prohibited(&g, token));
    g.record_control(g.object_ref(token), Player::P1, None);
    assert!(CharacteristicWorld::sacrifice_prohibited(&g, token));
    g.change_object_form(
        g.object_ref(token),
        AnimationSpec {
            creature_types: AnimationCreatureTypes::Named(&["Frog"]),
            ..SUBTYPE
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    assert!(!CharacteristicWorld::sacrifice_prohibited(&g, token));
}

#[test]
fn hexproof_grant_ends_when_equipment_provider_is_blanked_or_detached() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &EQUIPMENT),
        (Card::LightningBolt, &BOLT),
    ]);
    for blank in [false, true] {
        let mut g = scenario(Player::P0);
        let target = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        let equip = put(&mut g, Player::P1, "Short Bow", Zone::Battlefield);
        g.state.objects[equip as usize].attached_to = Some(g.object_ref(target));
        let bolt = put(&mut g, Player::P0, "Lightning Bolt", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: bolt });
        if blank {
            suppress(&mut g, equip);
        } else {
            g.state.objects[equip as usize].attached_to = None;
        }
        cast(&mut g, bolt, Some(Target::Card(target)));
        settle(&mut g);
        assert_eq!(g.state.objects[target as usize].damage, 1);
    }
}

#[test]
fn windcrag_duplicates_a_real_root_manipulation_gained_attack_ability() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Anthem of Champions").unwrap(), &MULTIPLIER)]);
    for p in [Player::P0, Player::P1] {
        for mode in [1, 0] {
            let mut g = scenario(p);
            multiplier(&mut g, p, mode);
            let attacker = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let spell = put(&mut g, p, "Root Manipulation", Zone::Hand);
            cast(&mut g, spell, None);
            settle(&mut g);
            assert!(
                g.state
                    .temporary_abilities
                    .as_ref()
                    .unwrap()
                    .iter()
                    .any(|a| a.recipient == g.object_ref(attacker))
            );
            assert_eq!(
                g.state.objects[spell as usize].zone,
                Zone::Graveyard,
                "the grant survives its spell source"
            );
            declare(&mut g, attacker);
            drain(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if mode == 0 { 22 } else { 21 }
            );
        }
    }
}

#[test]
fn windcrag_gained_attacks_follow_recipient_control_and_layer_six_availability() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Anthem of Champions").unwrap(), &MULTIPLIER)]);
    for case in 0..4 {
        let p = Player::P0;
        let mut g = scenario(p);
        let siege = multiplier(&mut g, p, 0);
        let attacker = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, p, "Root Manipulation", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        let actor = if case == 3 { Player::P1 } else { p };
        match case {
            0 => suppress(&mut g, attacker),
            1 => suppress(&mut g, siege),
            2 => g.record_control(g.object_ref(siege), p.other(), None),
            3 => g.record_control(g.object_ref(attacker), p.other(), None),
            _ => unreachable!(),
        }
        g.state.active = actor;
        g.state.priority = actor;
        g.state.objects[attacker as usize].summoning_sick = false;
        declare(&mut g, attacker);
        drain(&mut g);
        assert_eq!(
            g.state.players[actor.index()].life,
            if case == 0 { 20 } else { 21 }
        );
    }
}

#[test]
fn windcrag_does_not_duplicate_independent_attack_callback_after_its_source_leaves() {
    const CALLBACK: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ability(
            ActivationCost {
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            ActivationOrigin::Battlefield,
            ActivationGuard::Always,
            &[Instruction::ScheduleTurnTrigger {
                event: TurnCallbackEvent::Attack,
                ability: 0,
                once: true,
            }],
        )],
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: LIFE,
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Anthem of Champions").unwrap(), &MULTIPLIER),
        (Card::from_name("Short Bow").unwrap(), &CALLBACK),
    ]);
    for leave in [false, true] {
        let mut g = scenario(Player::P0);
        multiplier(&mut g, Player::P0, 0);
        let installer = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
        activate(&mut g, installer, 0);
        settle(&mut g);
        if leave {
            g.move_object(installer, Zone::Graveyard);
        }
        let attacker = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        declare(&mut g, attacker);
        drain(&mut g);
        assert_eq!(
            g.state.players[0].life, 23,
            "the scheduled ability is independent whether its installer stays or leaves"
        );
    }
}

#[test]
fn windcrag_temporary_grant_still_obeys_a_printed_once_per_turn_trigger_limit() {
    const FILTER: CreatureFilter = CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    };
    const LIMITED: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::GrantCreatureAbility {
                filter: FILTER,
                ability: 0,
            }],
        ),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::OncePerTurn(
                &TriggerCondition::SelfAttacks,
            )),
            targets: TargetPlan::NONE,
            program: LIFE,
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Anthem of Champions").unwrap(), &MULTIPLIER),
        (Card::from_name("Opt").unwrap(), &LIMITED),
    ]);
    let mut g = scenario(Player::P0);
    multiplier(&mut g, Player::P0, 0);
    let attacker = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let grant = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, grant, None);
    settle(&mut g);
    declare(&mut g, attacker);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 23);
    g.state.attackers.clear();
    g.state.objects[attacker as usize].tapped = false;
    declare(&mut g, attacker);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 23);
}

#[test]
fn independent_native_once_per_turn_grants_keep_separate_acquisitions_and_owned_trigger_history() {
    const FILTER: CreatureFilter = CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    };
    const LIMITED: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::GrantCreatureAbility {
                filter: FILTER,
                ability: 0,
            }],
        ),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::OncePerTurn(
                &TriggerCondition::SelfAttacks,
            )),
            targets: TargetPlan::NONE,
            program: LIFE,
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Anthem of Champions").unwrap(), &MULTIPLIER),
        (Card::from_name("Opt").unwrap(), &LIMITED),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut parent = scenario(p);
        multiplier(&mut parent, p, 0);
        let attacker = put(&mut parent, p, "Hill Giant", Zone::Battlefield);
        for _ in 0..2 {
            let spell = put(&mut parent, p, "Opt", Zone::Hand);
            cast(&mut parent, spell, None);
            settle(&mut parent);
        }
        assert_eq!(parent.state.temporary_abilities.as_ref().unwrap().len(), 2);
        parent.state.pending = Pending::Attackers;
        parent.state.step = Step::Attackers;
        act(&mut parent, Action::Attack { card: attacker });
        let saved = parent.snapshot();
        let revision = parent.revision();
        let index = parent
            .legal_actions()
            .iter()
            .position(|a| *a == Action::FinishAttackers)
            .unwrap();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        drop(parent);
        raw.apply(p, Action::FinishAttackers).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        drain(&mut raw);
        drain(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(
            raw.state.players[p.index()].life,
            26,
            "two independently acquired limited abilities each trigger once"
        );
        indexed.restore(&saved).unwrap();
        indexed.apply(p, Action::FinishAttackers).unwrap();
        drain(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        raw.state.attackers.clear();
        raw.state.objects[attacker as usize].tapped = false;
        declare(&mut raw, attacker);
        drain(&mut raw);
        assert_eq!(
            raw.state.players[p.index()].life,
            26,
            "both independent instances have used their own limit"
        );
    }
}

#[test]
fn independent_native_permanent_trigger_grants_each_keep_their_own_once_limit() {
    const SELF_GRANT: CardRules = CardRules {
        effect: Effect::Creature(4, 4),
        activated: &[ability(
            ActivationCost::ZERO,
            ActivationOrigin::Battlefield,
            ActivationGuard::Always,
            &[Instruction::GrantSourcePermanentTrigger(0)],
        )],
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::OncePerTurn(
                &TriggerCondition::SelfAttacks,
            )),
            targets: TargetPlan::NONE,
            program: LIFE,
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &SELF_GRANT)]);
    let mut g = scenario(Player::P0);
    let attacker = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    for _ in 0..2 {
        activate(&mut g, attacker, 0);
        settle(&mut g);
    }
    declare(&mut g, attacker);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 26);
}

#[test]
fn independent_native_evolved_trigger_grants_keep_prior_acquired_abilities() {
    const EVOLVE: CardRules = CardRules {
        effect: Effect::Creature(4, 4),
        activated: &[ability(
            ActivationCost::ZERO,
            ActivationOrigin::Battlefield,
            ActivationGuard::Always,
            &[Instruction::EvolveSource {
                required: "Elf",
                types: &["Elf", "Druid"],
                stats: None,
                keywords: Keywords::NONE,
                ability: Some(0),
            }],
        )],
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::OncePerTurn(
                &TriggerCondition::SelfAttacks,
            )),
            targets: TargetPlan::NONE,
            program: LIFE,
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::LlanowarElves, &EVOLVE)]);
    let mut g = scenario(Player::P0);
    let attacker = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    for _ in 0..2 {
        activate(&mut g, attacker, 0);
        settle(&mut g);
    }
    declare(&mut g, attacker);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 26);
}

#[test]
fn independent_native_granted_and_printed_abilities_at_the_same_index_do_not_share_history() {
    const FILTER: CreatureFilter = CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    };
    const GRANT: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::GrantCreatureAbility {
                filter: FILTER,
                ability: 0,
            }],
        ),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::OncePerTurn(
                &TriggerCondition::SelfAttacks,
            )),
            targets: TargetPlan::NONE,
            program: LIFE,
        }],
        ..CardRules::EMPTY
    };
    const PRINTED: CardRules = CardRules {
        effect: Effect::Creature(4, 4),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::SelfAttacks),
            targets: TargetPlan::NONE,
            program: LIFE,
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::HillGiant, &PRINTED),
        (Card::from_name("Opt").unwrap(), &GRANT),
    ]);
    let mut g = scenario(Player::P0);
    let attacker = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    settle(&mut g);
    declare(&mut g, attacker);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 26);
}
