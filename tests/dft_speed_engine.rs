//! Independent outcomes for shared DFT quantities, speed rules and mana quality.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const MAX_SPEED: StatValue = StatValue::AtLeast {
    value: &StatValue::ControllerSpeed,
    minimum: 4,
};
const SPEED_RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
                condition: MAX_SPEED,
                keywords: Keywords::HASTE,
            }],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Source,
                    value: MAX_SPEED,
                    stats: Stats {
                        power: 1,
                        toughness: 2,
                    },
                },
            )],
        },
    ],
    extras: &CardExtras {
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Source,
            value: MAX_SPEED,
            stats: Stats {
                power: 1,
                toughness: 2,
            },
        }],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::Expression {
                value: StatValue::ControllerSpeed,
                minimum: Some(4),
                maximum: None,
            },
            event: &TriggerCondition::SelfAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn speed_queries_and_conditional_abilities_follow_live_controller() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SPEED_RULES)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        for speed in [0, 3, 4] {
            g.state.players[p.index()].speed = speed;
            assert_eq!(
                g.stat_value(StatValue::ControllerSpeed, g.object_ref(source), p),
                i128::from(speed)
            );
            assert_eq!(
                g.stats(source),
                if speed == 4 {
                    Stats {
                        power: 3,
                        toughness: 4,
                    }
                } else {
                    Stats {
                        power: 2,
                        toughness: 2,
                    }
                }
            );
            assert_eq!(g.keywords(source).contains(Keywords::HASTE), speed == 4);
        }
        g.state.objects[source as usize].controller = p.other();
        g.state.players[p.other().index()].speed = 2;
        assert_eq!(stats(&g, source), (2, 2));
        assert!(!g.keywords(source).contains(Keywords::HASTE));
        g.move_object(source, Zone::Graveyard);
        assert_eq!(
            g.observe(p).players[p.index()].speed,
            4,
            "designation outlives permanent"
        );
    }
}

#[test]
fn max_speed_event_qualification_is_not_an_intervening_resolution_check() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SPEED_RULES)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.players[p.index()].speed = 3;
        assert!(!g.queue_trigger(&g.object_view(source), 0));
        g.state.players[p.index()].speed = 4;
        assert!(g.queue_trigger(&g.object_view(source), 0));
        g.prepare_triggers();
        g.state.players[p.index()].speed = 3;
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
    }
}

const DOUBLE_RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ConditionalDoubleDraw(MAX_SPEED)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn conditional_doubling_includes_first_draw_and_multiple_sources_compose() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &DOUBLE_RULES)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        // Other static sources must not accidentally double a draw.
        put(&mut g, p, "Perilous Snare", Zone::Battlefield);
        g.state.players[p.index()].speed = 3;
        g.draw_many(p, 1);
        assert_eq!(g.hand_count(p), 1);
        g.state.players[p.index()].speed = 4;
        g.state.step = Step::Draw;
        g.state.players[p.index()].drew_in_own_draw_step = false;
        g.draw_many(p, 1);
        assert_eq!(g.hand_count(p), 3, "normal draw-step draw doubles too");
        put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.draw_many(p, 1);
        assert_eq!(g.hand_count(p), 7, "two independent doublers multiply");
        assert_eq!(g.hand_count(p.other()), 0);
    }
}

const DAMAGE_RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::Replacement(
            0,
            ReplacementSpec::ConditionalControlledDamageToOpponentPlus {
                condition: MAX_SPEED,
                amount: 1,
            },
        )],
    }],
    extras: &CardExtras {
        replacement_effects: &[ReplacementSpec::ConditionalControlledDamageToOpponentPlus {
            condition: MAX_SPEED,
            amount: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn conditional_damage_addition_covers_combat_and_permanents_but_only_opponents() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &DAMAGE_RULES)]);
    for p in [Player::P0, Player::P1] {
        for combat in [false, true] {
            for speed in [3, 4] {
                for hostile_source in [false, true] {
                    let mut g = scenario(p);
                    put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                    let dealer = put(
                        &mut g,
                        if hostile_source { p.other() } else { p },
                        "Hill Giant",
                        Zone::Battlefield,
                    );
                    let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                    g.state.players[p.index()].speed = speed;
                    let amount = if speed == 4 && !hostile_source { 2 } else { 1 };
                    let source = g.damage_source(dealer);
                    g.apply_damage_batch(&[DamageEvent {
                        source,
                        target: Target::Card(target),
                        amount: 1,
                        combat,
                        preventable: true,
                    }]);
                    assert_eq!(g.state.objects[target as usize].damage, amount);
                    g.apply_damage_batch(&[DamageEvent {
                        source,
                        target: Target::Player(p.other()),
                        amount: 1,
                        combat,
                        preventable: true,
                    }]);
                    assert_eq!(g.state.players[p.other().index()].life, 20 - amount);
                    g.apply_damage_batch(&[DamageEvent {
                        source,
                        target: Target::Player(p),
                        amount: 1,
                        combat,
                        preventable: true,
                    }]);
                    assert_eq!(
                        g.state.players[p.index()].life,
                        19,
                        "own recipient is never increased"
                    );
                }
            }
        }
    }
}

const DISCARD_RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Discarded {
            relation: CastRelation::You,
            one_or_more: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensScaled {
            token: Card::DrakeToken,
            amount: AmountValue::DiscardEventAmount,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn captured_discard_quantity_uses_one_batch_and_survives_source_departure() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &DISCARD_RULES)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let cards = [
            put(&mut g, p, "Forest", Zone::Hand),
            put(&mut g, p, "Forest", Zone::Hand),
        ];
        g.discard_batch(&cards, None);
        assert_eq!(g.state.pending_triggers.len(), 1);
        let packet = g.state.pending_triggers[0].clone();
        assert_eq!(g.item_amount(AmountValue::DiscardEventAmount, &packet), 2);
        g.state.players[p.index()].cards_discarded_this_turn = 9;
        g.move_object(observer, Zone::Graveyard);
        assert_eq!(g.item_amount(AmountValue::DiscardEventAmount, &packet), 2);
        g.prepare_triggers();
        let saved = g.snapshot();
        let mut branch = g.clone();
        settle(&mut branch);
        assert_eq!(
            branch
                .state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.card == Card::DrakeToken)
                .count(),
            2
        );
        g.restore(&saved).unwrap();
        settle(&mut g);
        assert_eq!(g.snapshot(), branch.snapshot());
    }
}

const MANA_RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
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
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed([0, 3, 0, 0, 0, 0]),
            quality: &ManaQuality {
                restriction: ManaRestriction::ArtifactSpellOrActivation,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn artifact_or_activation_mana_rejects_other_payments_and_keeps_quality_after_departure() {
    let _scope = fixture_rules::activate(&[(Card::LlanowarElves, &MANA_RULES)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.players[p.index()].mana = [0; 6];
        let producer = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let artifact = put(&mut g, p, "Short Bow", Zone::Hand);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        let camera = put(&mut g, p, "Camera Launcher", Zone::Battlefield);
        choose(
            &mut g,
            |a| matches!(a, Action::ActivateMana { card, production, .. } if *card == producer && *production == [0,3,0,0,0,0]),
        );
        g.move_object(producer, Zone::Graveyard);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: artifact })
        );
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        assert!(g.legal_actions().iter().any(
            |a| matches!(a, Action::AnnounceActivate {card, ability:0, ..} if *card == camera)
        ));
        let quality = g.state.mana_lots.as_ref().unwrap().pools[p.index()][0].quality;
        assert!(
            !quality.eligible(mana_lots::Purpose::Other),
            "ward and counter-unless-pay cannot spend the restricted lot"
        );
        let purpose = mana_lots::Purpose::Activation(mana_lots::Source::in_world(&g, camera));
        assert!(quality.eligible(purpose));
        let before = g.snapshot();
        reject(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn own_spell_target_accepts_battle_spells_and_rejects_opponents_and_abilities() {
    let mut g = scenario(Player::P0);
    let id = put(&mut g, Player::P0, "Divination", Zone::Hand);
    cast(&mut g, id, None);
    let mut packet = g.state.stack.last().unwrap().clone();
    // Existing synthetic Siege has only Battle type (bit256), outside u8 masks.
    let siege = Card::from_name("@fixture/siege-front").unwrap();
    packet.source.as_mut().unwrap().card = siege;
    assert_eq!(siege.kind_mask(), 256);
    assert!(g.stack_target_rule_matches(TargetRule::YourSpell, &packet, Player::P0));
    assert!(!g.stack_target_rule_matches(TargetRule::YourSpell, &packet, Player::P1));
    packet.kind = StackKind::ActivatedAbility { ability: 0 };
    assert!(!g.stack_target_rule_matches(TargetRule::YourSpell, &packet, Player::P0));
}

#[test]
fn more_lands_condition_is_strict_and_uses_each_players_current_lands() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let reference = g.object_ref(source);
        assert_eq!(
            g.stat_value(StatValue::OpponentControlsMoreLands, reference, p),
            0
        );
        let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        assert_eq!(
            g.stat_value(StatValue::OpponentControlsMoreLands, reference, p),
            1
        );
        put(&mut g, p, "Forest", Zone::Battlefield);
        assert_eq!(
            g.stat_value(StatValue::OpponentControlsMoreLands, reference, p),
            0
        );
        g.move_object(land, Zone::Graveyard);
        assert_eq!(
            g.stat_value(StatValue::OpponentControlsMoreLands, reference, p.other()),
            1
        );
    }
}

const SPEED_DAMAGE_RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DamagePlayers(PlayerSet::SpeedBelow(4), 2)],
    ),
    ..CardRules::EMPTY
};

#[test]
fn speed_filtered_damage_checks_each_player_independently_before_the_batch() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &SPEED_DAMAGE_RULES)]);
    for p in [Player::P0, Player::P1] {
        for (ours, theirs) in [(0, 0), (3, 4), (4, 3), (3, 3), (4, 4)] {
            let mut g = scenario(p);
            g.state.players[p.index()].speed = ours;
            g.state.players[p.other().index()].speed = theirs;
            let spell = put(&mut g, p, "Divination", Zone::Hand);
            cast(&mut g, spell, None);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if ours < 4 { 18 } else { 20 }
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                if theirs < 4 { 18 } else { 20 }
            );
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
    }
}

const BOUNCE_SPEED_RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::ReturnToHand,
            Instruction::DecreaseLeadingSpeed {
                players: PlayerSet::TargetController,
                minimum: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn speed_decrease_uses_target_pre_bounce_controller_and_preserves_ties_and_floor() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &BOUNCE_SPEED_RULES)]);
    for p in [Player::P0, Player::P1] {
        for (ours, theirs, expected) in [(0, 4, 3), (3, 3, 3), (0, 1, 1)] {
            let mut g = scenario(p);
            g.state.players[p.index()].speed = ours;
            g.state.players[p.other().index()].speed = theirs;
            // The card is ours but the pre-bounce controller is the other seat.
            let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[target as usize].controller = p.other();
            let spell = put(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Hand);
            assert_eq!(g.state.players[p.index()].speed, ours);
            assert_eq!(g.state.players[p.other().index()].speed, expected);
        }
    }
}
