//! Shared Lorwyn characteristic rules, tested before admitting any new cards.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const HISTORY_SCALER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ControlledCreatureScaling {
            subtype: None,
            value: StatValue::CreatureEntriesThisTurn { other: false },
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    ..CardRules::EMPTY
};
const ATTACHMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::AttachedAllCreatureTypes],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttachmentCombatToughnessIfGreater],
        },
    ],
    ..CardRules::EMPTY
};
const DIFFERENCE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(0, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksOrBlocks,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowEventPowerToughnessDifference],
    }],
    ..CardRules::EMPTY
};
const ENTER_TAP: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrBecomesTapped,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn entry_history_freezes_controller_and_incarnation_and_scales_current_groups() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &HISTORY_SCALER)]);
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let provider = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let entrant = put(&mut g, player, "Hill Giant", Zone::Hand);
        g.move_object(entrant, Zone::Battlefield);
        assert_eq!(stats(&g, provider), (2, 2));
        g.state.objects[entrant as usize].controller = player.other();
        g.move_object(entrant, Zone::Graveyard);
        assert_eq!(
            g.creature_entries_this_turn(player, g.object_ref(provider), true),
            1
        );
        g.move_object(entrant, Zone::Battlefield);
        assert_eq!(
            g.creature_entries_this_turn(player, g.object_ref(entrant), true),
            1
        );
        assert_eq!(
            g.creature_entries_this_turn(player, g.object_ref(provider), false),
            2
        );
        assert_eq!(stats(&g, provider), (3, 3));
        let checkpoint = g.snapshot();
        let mut branch = g.clone();
        branch.move_object(provider, Zone::Graveyard);
        assert_eq!(stats(&branch, entrant), (3, 3));
        g.restore(&checkpoint).unwrap();
        assert_eq!(stats(&g, entrant), (5, 5));
        g.begin_turn();
        assert_eq!(
            g.creature_entries_this_turn(player, g.object_ref(provider), false),
            0
        );
    }
}

#[test]
fn attack_history_keeps_subtypes_before_observer_entry_and_counts_each_incarnation_once() {
    let mut g = scenario(Player::P0);
    let id = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let declaration = [AttackDeclaration {
        card: id,
        defender: Some(CombatDefender::Player(Player::P1)),
        blocked: false,
    }];
    g.record_declared_attack_history(Player::P0, &declaration);
    g.record_declared_attack_history(Player::P0, &declaration);
    assert_eq!(g.declared_subtype_attackers(Player::P0, "Bear"), 1);
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Named(&["Merfolk"]),
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    g.record_declared_attack_history(Player::P0, &declaration);
    assert_eq!(g.declared_subtype_attackers(Player::P0, "Bear"), 1);
    assert_eq!(g.declared_subtype_attackers(Player::P0, "Merfolk"), 1);
    g.move_object(id, Zone::Graveyard);
    g.move_object(id, Zone::Battlefield);
    assert_eq!(
        g.declared_subtype_attackers(Player::P0, "Bear"),
        1,
        "attacking entries are not declarations"
    );
    g.record_declared_attack_history(Player::P0, &declaration);
    assert_eq!(g.declared_subtype_attackers(Player::P0, "Bear"), 2);
    g.begin_step(Step::EndCombat);
    g.give_priority();
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::Main2);
    assert!(g.observe(Player::P0).declared_attack_history.is_none());
    let observer = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    assert_eq!(
        g.stat_value(
            StatValue::DeclaredSubtypeAttackersThisTurn("Bear"),
            g.object_ref(observer),
            Player::P0,
        ),
        2,
    );
    g.begin_turn();
    assert_eq!(g.declared_subtype_attackers(Player::P0, "Bear"), 0);
}

#[test]
fn attachment_type_grant_and_greater_toughness_follow_live_attachment_and_layer_order() {
    const BODY: CardRules = CardRules {
        effect: Effect::Creature(2, 5),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &ATTACHMENT),
        (Card::HillGiant, &BODY),
    ]);
    let mut g = scenario(Player::P0);
    let creature = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let equipment = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    g.state.objects[equipment as usize].attached_to = Some(g.object_ref(creature));
    assert!(g.has_subtype(creature, "Merfolk"));
    assert_eq!(g.combat_assignment_amount(creature), 5);
    g.change_object_form(
        g.object_ref(creature),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Named(&[]),
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    assert!(!g.has_subtype(creature, "Merfolk"));
    g.expire_animations();
    assert!(g.has_subtype(creature, "Merfolk"));
    g.state.objects[creature as usize].growth.power = 7;
    assert_eq!(g.combat_assignment_amount(creature), 9);
    let mut world = FundingState::from_game(&g, &goal(Player::P0));
    world
        .resources
        .iter_mut()
        .find(|row| row.id == equipment)
        .unwrap()
        .object
        .zone = Zone::Graveyard;
    assert!(!world.subtypes(creature).has(Card::HillGiant, "Merfolk"));
    g.move_object(equipment, Zone::Graveyard);
    assert!(!g.has_subtype(creature, "Merfolk"));
}

#[test]
fn attack_and_block_growth_uses_exact_event_object_and_current_signed_difference() {
    const BODY: CardRules = CardRules {
        effect: Effect::Creature(-1, 4),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &DIFFERENCE_OBSERVER),
        (Card::HillGiant, &BODY),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let creature = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: creature });
        checked(&mut g, Action::FinishAttackers);
        assert_eq!(g.state.stack.len(), 1);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(stats(&g, creature), (4, 9));
        let source = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        g.self_block_triggers(&[creature, creature], 0);
        assert_eq!(
            g.state.pending_triggers.len(),
            1,
            "one event per blocking creature"
        );
        g.move_object(creature, Zone::Graveyard);
        g.move_object(creature, Zone::Battlefield);
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(stats(&g, creature), (-1, 4));
        assert_eq!(stats(&g, source), (0, 5));
    }
}

#[test]
fn combined_entry_and_tap_trigger_has_one_identity_and_requires_an_actual_tap() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &ENTER_TAP)]);
    let mut g = scenario(Player::P0);
    let id = put(&mut g, Player::P0, "Hill Giant", Zone::Hand);
    let life = g.state.players[0].life;
    cast(&mut g, id, None);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, life + 1);
    g.state.objects[id as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(&mut g, Action::Attack { card: id });
    checked(&mut g, Action::FinishAttackers);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, life + 2);
}

const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const PROTECTED: CardRules = CardRules {
    effect: Effect::Creature(10, 10),
    keywords: Keywords::PROTECTION_OPPONENTS,
    ..CardRules::EMPTY
};
const OWN_COLORS: CardRules = CardRules {
    effect: Effect::Creature(10, 10),
    keywords: Keywords::HEXPROOF_OWN_COLORS,
    ..CardRules::EMPTY
};
const DAMAGE: CardRules = CardRules {
    effect: Effect::Spell(CREATURE, &[Instruction::Damage(2)]),
    ..CardRules::EMPTY
};
const TAPPED_PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledTappedKeywords {
            other: true,
            keywords: Keywords::INDESTRUCTIBLE,
        }],
    }],
    ..CardRules::EMPTY
};
const UNTAPPED: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
            condition: StatValue::SourceUntapped,
            keywords: Keywords::HEXPROOF,
        }],
    }],
    ..CardRules::EMPTY
};

fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 1,
        reserve: None,
        source: None,
    }
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

fn checked(g: &mut Game, action: Action) {
    let snapshot = g.snapshot();
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    branch.apply_index(actor, revision, index).unwrap();
    act(g, action.clone());
    assert_eq!(g.snapshot(), branch.snapshot());
    for player in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(player)).unwrap(),
            serde_json::to_value(branch.observe(player)).unwrap()
        );
    }
    g.restore(&snapshot).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(g, action);
    assert_eq!(g.snapshot(), branch.snapshot());
}

#[test]
fn opponent_protection_rejects_opposing_targets_before_payment_but_allows_own_spells() {
    let _scope =
        fixture_rules::activate(&[(Card::GrizzlyBears, &PROTECTED), (Card::Shock, &DAMAGE)]);
    for caster in [Player::P0, Player::P1] {
        for owner in [caster, caster.other()] {
            let mut g = scenario(caster);
            let target = put(&mut g, owner, "Grizzly Bears", Zone::Battlefield);
            let spell = put(&mut g, caster, "Shock", Zone::Hand);
            if owner != caster {
                reject(&mut g, Action::AnnounceCast { card: spell });
                assert_eq!(g.state.objects[spell as usize].zone, Zone::Hand);
            } else {
                cast(&mut g, spell, Some(Target::Card(target)));
                settle(&mut g);
                assert_eq!(g.state.objects[target as usize].damage, 2);
            }
        }
    }
}

#[test]
fn opponent_protection_uses_damage_source_controller_lki_and_preserves_unpreventable_damage() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &PROTECTED)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let source = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        let mut hit = DamageEvent {
            source: g.damage_source(source),
            target: Target::Card(target),
            amount: 2,
            preventable: true,
            combat: true,
        };
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Battlefield);
        g.state.objects[source as usize].controller = p;
        g.apply_damage_batch(&[hit]);
        g.advance_replacements(true);
        assert_eq!(
            g.state.objects[target as usize].damage, 0,
            "new source controller cannot rewrite the old damage event"
        );
        hit.preventable = false;
        g.apply_damage_batch(&[hit]);
        g.advance_replacements(true);
        assert_eq!(g.state.objects[target as usize].damage, 2);
        hit.source = g.damage_source(source);
        hit.preventable = true;
        g.apply_damage_batch(&[hit]);
        g.advance_replacements(true);
        assert_eq!(
            g.state.objects[target as usize].damage, 4,
            "friendly sources remain unprotected"
        );
    }
}

#[test]
fn opponent_protection_is_an_owned_prevention_choice_without_consuming_a_shield() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &PROTECTED)]);
    let mut g = scenario(Player::P0);
    let target = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let source = put(&mut g, Player::P1, "Goblin Piker", Zone::Battlefield);
    g.add_counter(target, CounterKind::Shield, 1);
    g.apply_damage_batch(&[DamageEvent {
        source: g.damage_source(source),
        target: Target::Card(target),
        amount: 2,
        preventable: true,
        combat: false,
    }]);
    g.advance_replacements(true);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(
                a,
                Action::ChooseDamagePrevention {
                    prevention: DamagePreventionKind::ProtectionOpponents,
                    ..
                }
            )
        })
        .unwrap();
    checked(&mut g, action);
    assert_eq!(g.counter_count(target, CounterKind::Shield), 1);
    assert_eq!(g.state.objects[target as usize].damage, 0);
}

#[test]
fn opponent_protection_excludes_blockers_and_foreign_attachments_and_rechecks_control() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &PROTECTED)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let recipient = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let blocker = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.step = Step::Blockers;
        g.state.pending = Pending::Blockers;
        g.state.attackers.push(AttackDeclaration {
            card: recipient,
            defender: Some(CombatDefender::Player(p.other())),
            blocked: false,
        });
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: recipient,
            },
        );
        let aura = Card::from_name("Pacifism").unwrap();
        assert!(g.aura_can_enchant(aura, p, g.object_ref(recipient)));
        assert!(!g.aura_can_enchant(aura, p.other(), g.object_ref(recipient)));
        let equipment = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
        assert!(equipment::can_attach(
            &g,
            g.object_ref(equipment),
            g.object_ref(recipient)
        ));
        g.state.objects[equipment as usize].attached_to = Some(g.object_ref(recipient));
        g.state.objects[equipment as usize].controller = p.other();
        assert!(!equipment::can_attach(
            &g,
            g.object_ref(equipment),
            g.object_ref(recipient)
        ));
        g.state_based_actions();
        assert_eq!(g.state.objects[equipment as usize].attached_to, None);
    }
}

#[test]
fn own_color_hexproof_uses_live_color_intersection_including_colorless_and_friendly_sources() {
    let _scope = fixture_rules::activate(&[
        (Card::HillGiant, &OWN_COLORS),
        (Card::Shock, &DAMAGE),
        (Card::GiantGrowth, &DAMAGE),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let recipient = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let green = put(&mut g, p, "Giant Growth", Zone::Hand);
        let red = put(&mut g, p, "Shock", Zone::Hand);
        for (colors, green_allowed, red_allowed) in
            [(1 | 16, false, true), (8, true, false), (0, true, true)]
        {
            paint(&mut g, recipient, colors);
            for (spell, allowed) in [(green, green_allowed), (red, red_allowed)] {
                assert_eq!(
                    g.cast_target_matches(
                        TargetRule::Creature,
                        p,
                        g.object_ref(spell),
                        g.state.objects[spell as usize].card,
                        Target::Card(recipient)
                    ),
                    allowed
                );
            }
        }
        paint(&mut g, recipient, 16);
        g.state.objects[recipient as usize].controller = p;
        cast(&mut g, green, Some(Target::Card(recipient)));
        settle(&mut g);
        assert_eq!(g.state.objects[recipient as usize].damage, 2);
    }
}

#[test]
fn tapped_grants_and_untapped_conditions_use_actual_and_projected_objects() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &TAPPED_PROVIDER),
        (Card::HillGiant, &UNTAPPED),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let recipient = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        assert!(g.keywords(recipient).contains(Keywords::HEXPROOF));
        assert!(!g.keywords(recipient).contains(Keywords::INDESTRUCTIBLE));
        let mut world = FundingState::from_game(&g, &goal(p));
        for row in &mut world.resources {
            row.object.tapped = true;
        }
        assert!(
            world
                .activation_keywords(recipient)
                .contains(Keywords::INDESTRUCTIBLE)
        );
        assert!(
            !world
                .activation_keywords(recipient)
                .contains(Keywords::HEXPROOF)
        );
        assert!(
            !world
                .activation_keywords(provider)
                .contains(Keywords::INDESTRUCTIBLE)
        );
        assert!(
            !world
                .activation_keywords(foreign)
                .contains(Keywords::INDESTRUCTIBLE)
        );
        assert!(
            g.keywords(recipient).contains(Keywords::HEXPROOF),
            "projection is owned"
        );
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: recipient });
        checked(&mut g, Action::FinishAttackers);
        assert!(g.keywords(recipient).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.keywords(recipient).contains(Keywords::HEXPROOF));
        g.move_object(provider, Zone::Graveyard);
        assert!(!g.keywords(recipient).contains(Keywords::INDESTRUCTIBLE));
    }
}

#[test]
fn attacking_you_quantity_excludes_permanent_defenders_and_departed_attackers() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let one = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let two = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let walker = put(
        &mut g,
        Player::P1,
        "Ajani, Outland Chaperone",
        Zone::Battlefield,
    );
    g.state.attackers.push(AttackDeclaration {
        card: one,
        defender: Some(CombatDefender::Player(Player::P1)),
        blocked: false,
    });
    g.state.attackers.push(AttackDeclaration {
        card: two,
        defender: Some(CombatDefender::Planeswalker(g.object_ref(walker))),
        blocked: false,
    });
    assert_eq!(
        g.stat_value(
            StatValue::CreaturesAttackingYou,
            g.object_ref(source),
            Player::P1
        ),
        1
    );
    g.move_object(one, Zone::Graveyard);
    assert_eq!(
        g.stat_value(
            StatValue::CreaturesAttackingYou,
            g.object_ref(source),
            Player::P1
        ),
        0
    );
}
