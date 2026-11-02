//! Remaining TDM mana, copy, cost, graveyard and damage world foundations.
//! Complete typed fixtures establish rules without ordinary card admissions.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const fn ability(program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const CREATURE_MANA: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        ..ability(&[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::any_color(1),
            quality: &ManaQuality {
                restriction: ManaRestriction::CreatureSpells,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })])
    }],
    ..CardRules::EMPTY
};
const DRAGON_OMEN_MANA: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        ..ability(&[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::any_color(1),
            quality: &ManaQuality {
                restriction: ManaRestriction::SpellSubtypeAny(&["Dragon", "Omen"]),
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })])
    }],
    ..CREATURE_MANA
};
const PLAIN: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    ..CardRules::EMPTY
};
const MASS: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[
            Instruction::PutCountersMixed {
                subject: Subject::Targets,
                amounts: &[(CounterKind::PlusOne, 1)],
            },
            Instruction::CopyTargetToOtherControlledCreatures,
        ],
    ),
    ..CardRules::EMPTY
};
const fn grave(kinds: u8, relation: i8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds,
        relation,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}
const OWN_GRAVE: TargetRule = grave(0, 0);
const ANY_GRAVE: TargetRule = grave(0, -1);
const PERMANENTS: TargetRule = TargetRule::Filtered {
    base: &OWN_GRAVE,
    predicate: &TargetPredicate::Permanent,
};
const PLAYER_GRAVE: TargetRule = TargetRule::Grouped {
    base: &ANY_GRAVE,
    constraint: TargetGroupConstraint::GraveyardOwnerOfPlayerGroup(1),
};
const RITE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: PERMANENTS,
                min: 0,
                max: 2,
            },
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
            TargetSpec {
                rule: PLAYER_GRAVE,
                min: 0,
                max: 4,
            },
        ]),
        &[
            Instruction::SelectGroup(0),
            Instruction::ReturnToHand,
            Instruction::ShuffleGraveyardTargetsIntoPlayerLibrary {
                player_group: 1,
                cards_group: 2,
            },
            Instruction::ExileResolvingSpell,
        ],
    ),
    ..CardRules::EMPTY
};
const DOUBLE: ReplacementSpec = ReplacementSpec::DoubleEnteredCreatureDamage;
const NERIV: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const KISHLA: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::ExileGraveyard,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 1,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(grave(4 | 8, 0))),
            program: &[Instruction::ReturnToHand],
        },
    ],
    ..CardRules::EMPTY
};
const NO_EXILE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::GraveyardCardsCannotBeExiled],
    }],
    ..CardRules::EMPTY
};
const NARSET: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 2 },
            Instruction::DiscardHand(PlayerSet::Controller),
            Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::Stat(StatValue::SpellsCastThisTurn),
            ),
        ],
    }],
    ..CardRules::EMPTY
};
fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn drain(g: &mut Game) {
    for _ in 0..128 {
        if idle(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::ChooseReplacement { .. }
                    | Action::FinishStackTargets
            )
        });
    }
    panic!("remaining world action bound: {:?}", g.state.pending);
}
fn wait(g: &mut Game, pred: impl Fn(&Game) -> bool) {
    for _ in 0..128 {
        if pred(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("remaining world pending bound: {:?}", g.state.pending);
}
fn mana(g: &mut Game, source: CardId, color: usize) {
    let mut production = [0; 6];
    production[color] = 1;
    act(
        g,
        Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production,
        },
    );
    pay(g);
}
fn damage(g: &mut Game, source: CardId, player: Player, amount: i128) {
    let source = g.damage_source(source);
    g.apply_damage_batch(&[DamageEvent {
        preventable: true,
        source,
        target: Target::Player(player),
        amount,
        combat: false,
    }]);
    g.advance_replacements(true);
    g.give_priority();
    drain(g);
}

#[test]
fn creature_restricted_mana_pays_a_creature_after_its_producer_leaves_but_not_a_spell_or_ability() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &CREATURE_MANA)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        mana(&mut g, source, 4);
        g.move_object(source, Zone::Hand);
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Hand);
        cast(&mut g, elf, None);
        drain(&mut g);
        assert_eq!(g.object_view(elf).zone, Zone::Battlefield);
        let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
        mana(&mut g, source, 3);
        let bolt = put(&mut g, p, "Lightning Bolt", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: bolt });
        assert_eq!(g.state.players[p.index()].mana[3], 1);
    }
}
#[test]
fn exact_entry_history_damage_multiplier_ignores_old_creatures_and_tracks_control_and_turn_reset() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &NERIV)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let old = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.begin_turn();
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        let fresh = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        damage(&mut g, old, p.other(), 2);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        damage(&mut g, fresh, p.other(), 2);
        assert_eq!(g.state.players[p.other().index()].life, 14);
        g.record_control(g.object_ref(fresh), p.other(), None);
        damage(&mut g, fresh, p, 2);
        assert_eq!(g.state.players[p.index()].life, 18);
        g.record_control(g.object_ref(fresh), p, None);
        damage(&mut g, fresh, p.other(), 2);
        assert_eq!(g.state.players[p.other().index()].life, 10);
        g.begin_turn();
        damage(&mut g, fresh, p.other(), 2);
        assert_eq!(g.state.players[p.other().index()].life, 8);
    }
}
#[test]
fn mass_copy_freezes_one_blueprint_and_preserves_counters_tap_damage_and_ownership_without_entries()
{
    let _scope = fixture_rules::activate(&[(card("Opt"), &MASS), (card("Bear Cub"), &PLAIN)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let blueprint = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        g.state.objects[a as usize].tapped = true;
        g.state.objects[a as usize].damage = 1;
        g.add_counter(a, CounterKind::Soul, 3);
        let old = g.object_ref(a);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(blueprint)));
        drain(&mut g);
        assert_eq!(g.object_view(a).card, card("Bear Cub"));
        assert_eq!(stats(&g, a), (2, 2));
        assert_eq!(stats(&g, blueprint), (3, 3));
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(a, CounterKind::Soul), 3);
        assert!(g.state.objects[a as usize].tapped);
        assert_eq!(g.state.objects[a as usize].damage, 1);
        assert_eq!(g.object_ref(a), old);
        assert_eq!(g.object_view(enemy).card, card("Shivan Dragon"));
        g.expire_temporary_copies(true);
        assert_eq!(g.object_view(a).card, Card::HillGiant);
        assert_eq!(g.counter_count(a, CounterKind::Soul), 3);
    }
}

fn finish_group(g: &mut Game, group: u16) {
    if g.state
        .announcing
        .as_ref()
        .is_some_and(|a| a.group == group)
        && g.legal_actions().contains(&Action::FinishTargets)
    {
        act(g, Action::FinishTargets);
    }
}
fn rite(
    g: &mut Game,
    p: Player,
    returned: &[CardId],
    owner: Player,
    shuffled: &[CardId],
) -> CardId {
    let spell = put(g, p, "Opt", Zone::Hand);
    act(g, Action::AnnounceCast { card: spell });
    for &id in returned {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    finish_group(g, 0);
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(owner),
        },
    );
    for &id in shuffled {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    finish_group(g, 2);
    pay(g);
    spell
}
#[test]
fn graveyard_owner_constraint_rejects_the_other_players_card_transactionally() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &RITE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Forest", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
        let c = put(&mut g, p, "Opt", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: c });
        finish_group(&mut g, 0);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        finish_group(&mut g, 2);
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Library);
        assert_eq!(g.object_view(c).zone, Zone::Exile);
    }
}
#[test]
fn target_player_shuffles_even_with_zero_cards_and_returned_earlier_target_is_not_shuffled() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &RITE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let x = put(&mut g, p, "Forest", Zone::Graveyard);
        // The same card is legal once in each independent target occurrence.
        let c = rite(&mut g, p, &[x], p, &[x]);
        drain(&mut g);
        assert_eq!(g.object_view(x).zone, Zone::Hand);
        assert_eq!(g.object_view(c).zone, Zone::Exile);
        let before_rng = g.state.rng;
        let mut expected = g.clone();
        expected.shuffle_library_cards(p.other());
        let c = rite(&mut g, p, &[], p.other(), &[]);
        drain(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Exile);
        assert_eq!(
            g.state.rng, expected.state.rng,
            "exactly the required empty shuffle"
        );
        assert_ne!(g.state.rng, before_rng);
    }
}
#[test]
fn rite_group_owner_and_partial_target_results_are_owned_across_raw_index_restore() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &RITE)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let x = put(&mut parent, p.other(), "Hill Giant", Zone::Graveyard);
    let c = put(&mut parent, p, "Opt", Zone::Hand);
    act(&mut parent, Action::AnnounceCast { card: c });
    finish_group(&mut parent, 0);
    act(
        &mut parent,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    let action = Action::ChooseTarget {
        target: Target::Card(x),
    };
    let saved = parent.snapshot();
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    for g in [&mut raw, &mut indexed] {
        finish_group(g, 2);
        pay(g);
        drain(g);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(x).zone, Zone::Library);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    finish_group(&mut indexed, 2);
    pay(&mut indexed);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn mass_copy_ignores_a_departed_target_and_never_rebinds_its_returned_incarnation() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &MASS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let other = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let c = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, c, Some(Target::Card(target)));
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    drain(&mut g);
    assert_eq!(g.object_view(other).card, Card::HillGiant);
    assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
}
#[test]
fn mass_copy_copies_the_frozen_copiable_recipe_without_reentering_or_reacquiring_entry_counters() {
    const BLUEPRINT: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        extras: &CardExtras {
            fixed_entry_counters: &[(CounterKind::PlusOne, 3)],
            ..CardExtras::EMPTY
        },
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 5)],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &MASS), (card("Bear Cub"), &BLUEPRINT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    g.apply_temporary_copy(
        g.object_ref(target),
        CopyRecipe::new(card("Bear Cub")),
        CopyDeadline::EndOfTurn,
    );
    let recipient = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let c = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, c, Some(Target::Card(target)));
    drain(&mut g);
    assert_eq!(g.object_view(recipient).card, card("Bear Cub"));
    assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 0);
    assert_eq!(g.state.players[p.index()].life, 20);
    assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
}
#[test]
fn entered_creature_damage_uses_source_lki_after_a_real_sacrifice_cost() {
    const SHOOT: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            })),
            ..ability(&[Instruction::Damage(2)])
        }],
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(card("Short Bow"), &NERIV), (Card::LlanowarElves, &SHOOT)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        let source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        drain(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
#[test]
fn entered_damage_qualification_excludes_an_old_creature_that_only_changed_control_and_an_artifact()
{
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &NERIV)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let old = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.begin_turn();
    put(&mut g, p, "Short Bow", Zone::Battlefield);
    g.record_control(g.object_ref(old), p, None);
    damage(&mut g, old, p.other(), 2);
    assert_eq!(g.state.players[1].life, 18);
    let artifact = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
    damage(&mut g, artifact, p.other(), 2);
    assert_eq!(g.state.players[1].life, 16);
    g.move_object(old, Zone::Hand);
    g.move_object(old, Zone::Battlefield);
    assert_eq!(g.object_view(old).controller, p.other());
    g.record_control(g.object_ref(old), p, None);
    damage(&mut g, old, p.other(), 2);
    assert_eq!(g.state.players[1].life, 12);
}
#[test]
fn optional_resolution_exile_cost_rejects_true_prohibition_but_can_be_declined_or_paid() {
    let _scope = fixture_rules::activate(&[
        (card("Grizzly Bears"), &KISHLA),
        (card("Short Bow"), &NO_EXILE),
    ]);
    for mode in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Hill Giant", Zone::Graveyard);
        let instant = put(&mut g, p, "Lightning Bolt", Zone::Graveyard);
        if mode == 0 {
            put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        }
        let source = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        cast(&mut g, source, None);
        if mode == 0 {
            drain(&mut g);
            assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
            assert_eq!(g.object_view(instant).zone, Zone::Graveyard);
            continue;
        }
        wait(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        if mode == 1 {
            act(&mut g, Action::DeclineEffectPayment);
            drain(&mut g);
            assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
        } else {
            choose(
                &mut g,
                |a| matches!(a,Action::Sacrifice { object } if object.card==creature),
            );
            wait(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(instant),
                },
            );
            drain(&mut g);
            assert_eq!(g.object_view(creature).zone, Zone::Exile);
            assert_eq!(g.object_view(instant).zone, Zone::Hand);
        }
    }
}
#[test]
fn narset_may_pay_an_empty_hand_and_draws_only_actual_casts_when_the_trigger_resolves() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &NARSET)]);
    for accept in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, spell, None);
        drain(&mut g);
        let hand = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| (o.zone == Zone::Hand && o.owner == p).then_some(id as CardId))
            .collect::<Vec<_>>();
        for id in hand {
            g.move_object(id, Zone::Graveyard);
        }
        g.begin_step(Step::End);
        g.give_priority();
        wait(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        act(&mut g, Action::ChooseOptional { accept });
        drain(&mut g);
        assert_eq!(
            g.state.players[p.index()].cards_drawn_this_turn,
            if accept { 3 } else { 2 }
        );
    }
}

#[test]
fn resolution_exile_cost_stays_paid_when_a_legal_payment_is_replaced_or_prevented() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
            zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|v| v.set(None));
        }
    }
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &KISHLA)]);
    let _reset = Reset;
    for prevent in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Hill Giant", Zone::Graveyard);
        let instant = put(&mut g, p, "Lightning Bolt", Zone::Graveyard);
        if prevent {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(Some(g.object_ref(creature))));
        } else {
            zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD
                .with(|v| v.set(Some(g.object_ref(creature))));
        }
        let source = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        cast(&mut g, source, None);
        wait(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let object = g.object_ref(creature);
        act(&mut g, Action::Sacrifice { object });
        wait(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(instant),
            },
        );
        drain(&mut g);
        assert_eq!(
            g.object_view(creature).zone,
            if prevent {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.object_view(instant).zone,
            Zone::Hand,
            "CR118.11 payment remains paid"
        );
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|v| v.set(None));
    }
}
#[test]
fn optional_resolution_exile_payment_is_owned_and_return_target_is_chosen_only_after_payment() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &KISHLA)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let creature = put(&mut parent, p, "Hill Giant", Zone::Graveyard);
    let instant = put(&mut parent, p, "Lightning Bolt", Zone::Graveyard);
    let source = put(&mut parent, p, "Grizzly Bears", Zone::Hand);
    cast(&mut parent, source, None);
    wait(&mut parent, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    let action = Action::Sacrifice {
        object: parent.object_ref(creature),
    };
    let saved = parent.snapshot();
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    for g in [&mut raw, &mut indexed] {
        wait(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
        assert_eq!(g.object_view(creature).zone, Zone::Exile);
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(instant),
            },
        );
        drain(g);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    act(&mut sibling, Action::DeclineEffectPayment);
    drain(&mut sibling);
    assert_eq!(sibling.object_view(creature).zone, Zone::Graveyard);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    wait(&mut indexed, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    act(
        &mut indexed,
        Action::ChooseTarget {
            target: Target::Card(instant),
        },
    );
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn creature_and_dragon_or_omen_mana_filters_use_each_selected_face_and_never_fund_other_payments() {
    use mana_lots::{Purpose, Quality, Source};
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &CREATURE_MANA)]);
    let mut g = scenario(Player::P0);
    let producer = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    for (restriction, expected) in [
        (ManaRestriction::CreatureSpells, [true, true, false, false]),
        (
            ManaRestriction::SpellSubtypeAny(&["Dragon", "Omen"]),
            [false, true, true, false],
        ),
    ] {
        let quality = Quality::capture(
            &g,
            producer,
            ManaQuality {
                restriction,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        );
        for (name, yes) in [
            "Llanowar Elves",
            "Shivan Dragon",
            "Roost Seek",
            "Lightning Bolt",
        ]
        .into_iter()
        .zip(expected)
        {
            let source = Source::printed(card(name));
            assert_eq!(
                quality.eligible(Purpose::Spell(source)),
                yes,
                "selected spell face {name}"
            );
            assert!(
                !quality.eligible(Purpose::Activation(source)),
                "does not fund {name} activation"
            );
        }
        assert!(
            !quality.eligible(Purpose::Other),
            "ward/other payments stay ineligible"
        );
    }
}
#[test]
fn dragon_union_mana_is_actually_spent_on_a_dragon_and_remains_owned_after_branch_restore() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &DRAGON_OMEN_MANA)]);
    let p = Player::P0;
    let mut parent = scenario(p);
    let source = put(&mut parent, p, "Short Bow", Zone::Battlefield);
    parent.state.players[p.index()].mana = [0; 6];
    mana(&mut parent, source, 3);
    parent.state.players[p.index()].mana[3] += 1;
    parent.state.players[p.index()].mana[5] = 4;
    let dragon = put(&mut parent, p, "Shivan Dragon", Zone::Hand);
    let action = Action::AnnounceCast { card: dragon };
    let saved = parent.snapshot();
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    for g in [&mut raw, &mut indexed] {
        pay(g);
        drain(g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(dragon).zone, Zone::Battlefield);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    pay(&mut indexed);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn departed_entry_history_is_exact_owned_and_expires_at_the_next_turn() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &NERIV)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let departed = g.damage_source(source);
        g.move_object(source, Zone::Graveyard);
        // The old source's LKI is still the entered battlefield creature.
        let saved = g.snapshot();
        for next_turn in [false, true] {
            let mut branch = g.clone();
            if next_turn {
                branch.begin_turn();
            }
            // A different live incarnation cannot overwrite the old history.
            branch.move_object(source, Zone::Battlefield);
            assert_ne!(branch.object_ref(source), departed.reference);
            branch.apply_damage_batch(&[DamageEvent {
                preventable: true,
                source: departed.clone(),
                target: Target::Player(p.other()),
                amount: 2,
                combat: false,
            }]);
            branch.advance_replacements(true);
            branch.give_priority();
            drain(&mut branch);
            assert_eq!(
                branch.state.players[p.other().index()].life,
                if next_turn { 18 } else { 16 }
            );
            assert_eq!(g.snapshot(), saved);
            g.restore(&saved).unwrap();
        }
    }
}
