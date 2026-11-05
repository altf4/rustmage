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
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::creature_you_control(),
            predicate: &TargetPredicate::Nonlegendary,
        })),
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
        let legendary = put(&mut g, p, "Gonti, Night Minister", Zone::Battlefield);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(legendary),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(blueprint),
            },
        );
        pay(&mut g);
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
        // The card target remains legal if the targeted player gains hexproof.
        // The player performs no shuffle, but the spell still resolves/exiles.
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
        let c = rite(&mut g, p, &[], p.other(), &[enemy]);
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        let before_rng = g.state.rng;
        drain(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(g.object_view(c).zone, Zone::Exile);
        assert_eq!(g.state.rng, before_rng);
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
                source: departed,
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

const ANY_FILTER: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const REVERB: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            colored: [0, 0, 0, 1, 0, 0],
            mana_component: true,
            sacrifice: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::DiscardHand,
                filter: ANY_FILTER,
            }),
            ..ActivationCost::ZERO
        },
        ..ability(&[Instruction::Draw(PlayerSet::Controller, 2)])
    }],
    ..CardRules::EMPTY
};
const NWF: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PreventNextChosenSourceDamage { ability: 0 }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DamagePlayersScaled(
                PlayerSet::EventPlayer,
                AmountValue::DamageEventAmount,
            ),
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::DamageEventAmount),
        ],
    }],
    ..CardRules::EMPTY
};
fn announce_reverb(g: &mut Game, source: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    assert!(matches!(g.state.pending, Pending::PayCosts { .. }));
}
fn source_choice(g: &mut Game) -> CardId {
    let player = g.state.active;
    let spell = put(g, player, "Opt", Zone::Hand);
    cast(g, spell, None);
    wait(g, |g| {
        matches!(g.state.pending, Pending::DamageSource { .. })
    });
    spell
}
fn shield(g: &mut Game, chosen: ObjectRef) -> CardId {
    let spell = source_choice(g);
    act(g, Action::ChooseDamageSource { source: chosen });
    drain(g);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    spell
}
fn event(g: &Game, source: CardId, player: Player, amount: i128, preventable: bool) -> DamageEvent {
    DamageEvent {
        preventable,
        source: g.damage_source(source),
        target: Target::Player(player),
        amount,
        combat: false,
    }
}
fn finish_hits(g: &mut Game, events: &[DamageEvent]) {
    g.apply_damage_batch(events);
    g.advance_replacements(true);
    g.give_priority();
    drain(g);
}
#[test]
fn whole_hand_activation_is_paid_with_zero_or_multiple_cards_before_source_sacrifice() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &REVERB)]);
    for p in [Player::P0, Player::P1] {
        for count in [0, 3] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
            let hand = (0..count)
                .map(|_| put(&mut g, p, "Hill Giant", Zone::Hand))
                .collect::<Vec<_>>();
            announce_reverb(&mut g, source);
            pay(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert!(
                hand.iter()
                    .all(|&id| g.object_view(id).zone == Zone::Graveyard)
            );
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                0,
                "cost is paid before ability resolves"
            );
            drain(&mut g);
            assert_eq!(g.hand_count(p), 2);
        }
    }
}
#[test]
fn whole_hand_payment_uses_the_hand_after_a_real_mana_child_and_rejects_unfundable_activation() {
    const HAND_MANA: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            origin: ActivationOrigin::Hand,
            cost: ActivationCost {
                discard: true,
                ..ActivationCost::ZERO
            },
            ..ability(&[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 1, 0, 1,
            ]))])
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (card("Short Bow"), &REVERB),
        (card("Hill Giant"), &HAND_MANA),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let fuel = put(&mut g, p, "Hill Giant", Zone::Hand);
        let remaining = put(&mut g, p, "Forest", Zone::Hand);
        announce_reverb(&mut g, source);
        act(
            &mut g,
            Action::ActivateMana {
                card: fuel,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 1],
            },
        );
        assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
        pay(&mut g);
        assert_eq!(g.object_view(remaining).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        drain(&mut g);
        assert_eq!(g.hand_count(p), 2);
    }
}
#[test]
fn whole_hand_payment_is_private_atomic_owned_and_replacement_modified_payment_still_counts() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        }
    }
    let _reset = Reset;
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &REVERB)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let source = put(&mut parent, p, "Short Bow", Zone::Battlefield);
    let hand = put(&mut parent, p, "Hill Giant", Zone::Hand);
    announce_reverb(&mut parent, source);
    reject(&mut parent, Action::PayCosts { payment: [99; 6] });
    assert_eq!(parent.object_view(hand).zone, Zone::Hand);
    assert_eq!(parent.object_view(source).zone, Zone::Battlefield);
    assert!(parent.observe(p.other()).additional_cost.is_none());
    let action = parent
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    let saved = parent.snapshot();
    let revision = parent.revision();
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
    indexed.apply_index(p, revision, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(Some(indexed.object_ref(hand))));
    act(&mut indexed, action);
    drain(&mut indexed);
    assert_eq!(indexed.object_view(hand).zone, Zone::Hand);
    assert_eq!(indexed.object_view(source).zone, Zone::Graveyard);
    assert_eq!(indexed.state.players[p.index()].cards_drawn_this_turn, 2);
}
#[test]
fn chosen_source_is_nontargeted_private_and_excludes_unreferenced_hidden_graveyard_and_exile_cards()
{
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let permanent = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.state.objects[permanent as usize].gained_keywords = Keywords::HEXPROOF;
    let grave = put(&mut g, p.other(), "Lightning Bolt", Zone::Graveyard);
    let exile = put(&mut g, p, "Divination", Zone::Exile);
    let hand = put(&mut g, p.other(), "Shivan Dragon", Zone::Hand);
    let command = put(&mut g, p, "Forest", Zone::Command);
    let manifested = put(&mut g, p.other(), "Shivan Dragon", Zone::Library);
    let entry = g
        .manifest_entry(g.object_ref(manifested), p.other())
        .unwrap();
    g.stage_entries(vec![entry], None);
    assert!(!g.advance_entries());
    g.advance_replacements(true);
    g.give_priority();
    drain(&mut g);
    assert!(
        g.observe(p)
            .face_down_permanents
            .iter()
            .any(|row| row.object == g.object_ref(manifested) && row.underlying.is_none()),
        "manifest views {:?}; object {:?}; state {:?}",
        g.observe(p).face_down_permanents,
        g.object_view(manifested),
        g.state.manifest
    );
    let own_spell = source_choice(&mut g);
    let candidates = &g.observe(p).damage_source_choice.unwrap().candidates;
    assert!(candidates.contains(&g.object_ref(permanent)));
    assert!(candidates.contains(&g.object_ref(command)));
    assert!(
        candidates.contains(&g.object_ref(manifested)),
        "face-down permanents are sources without revealing their face"
    );
    assert!(candidates.contains(&g.object_ref(own_spell)));
    for id in [grave, exile, hand] {
        assert!(!candidates.contains(&g.object_ref(id)));
    }
    assert!(g.observe(p.other()).damage_source_choice.is_none());
    let invalid = g.object_ref(grave);
    let chosen = g.object_ref(permanent);
    reject(&mut g, Action::ChooseDamageSource { source: invalid });
    act(&mut g, Action::ChooseDamageSource { source: chosen });
    drain(&mut g);
    assert_eq!(
        g.observe(p).chosen_source_shields.unwrap()[0].chosen,
        DamageSourceRef::Object(g.object_ref(permanent))
    );
}
#[test]
fn chosen_source_prevents_positive_damage_once_and_reflexively_damages_its_controller_and_draws_same()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let chosen = g.object_ref(source);
        shield(&mut g, chosen);
        let hit = event(&g, source, p, 3, true);
        finish_hits(&mut g, &[hit]);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        assert!(g.observe(p).chosen_source_shields.is_none());
        let hit = event(&g, source, p, 2, true);
        finish_hits(&mut g, &[hit]);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
    }
}
#[test]
fn zero_and_unpreventable_damage_do_not_consume_the_source_shield_or_create_a_callback() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let chosen = g.object_ref(source);
    shield(&mut g, chosen);
    let hit = event(&g, source, p, 0, true);
    finish_hits(&mut g, &[hit]);
    let hit = event(&g, source, p, 2, false);
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 18);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    assert_eq!(g.observe(p).chosen_source_shields.unwrap().len(), 1);
    let hit = event(&g, source, p, 1, true);
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 18);
    assert_eq!(g.state.players[p.other().index()].life, 19);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
}
#[test]
fn source_shield_ignores_other_recipients_and_sources_and_tracks_current_damage_controller() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let chosen = g.object_ref(source);
    shield(&mut g, chosen);
    let hits = [
        event(&g, source, p.other(), 1, true),
        event(&g, other, p, 2, true),
    ];
    finish_hits(&mut g, &hits);
    assert_eq!(g.state.players[p.index()].life, 18);
    assert!(g.observe(p).chosen_source_shields.is_some());
    g.record_control(g.object_ref(source), p, None);
    let hit = event(&g, source, p, 3, true);
    finish_hits(&mut g, &[hit]);
    assert_eq!(
        g.state.players[p.index()].life,
        15,
        "reflexive damage is to the source's current controller, even yourself"
    );
    assert_eq!(g.state.players[p.other().index()].life, 19);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
}
#[test]
fn simultaneous_source_fragments_are_one_prevented_sum_and_one_reflexive_with_other_hits_independent()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let chosen = g.object_ref(source);
    shield(&mut g, chosen);
    let hits = [
        event(&g, source, p, 2, true),
        event(&g, other, p, 1, true),
        event(&g, source, p.other(), 1, true),
        event(&g, source, p, 3, true),
    ];
    g.apply_damage_batch(&hits);
    g.advance_replacements(true);
    let callbacks = g
        .state
        .pending_triggers
        .iter()
        .filter(|item| item.damage_event.is_some())
        .collect::<Vec<_>>();
    assert_eq!(callbacks.len(), 1);
    assert_eq!(callbacks[0].damage_event.unwrap().amount, 5);
    g.give_priority();
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].life, 19);
    assert_eq!(g.state.players[p.other().index()].life, 14);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 5);
}
#[test]
fn source_choice_and_prevention_roots_are_owned_raw_index_restorable_and_reject_wrong_actor() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let source = put(&mut parent, p.other(), "Hill Giant", Zone::Battlefield);
    source_choice(&mut parent);
    let action = Action::ChooseDamageSource {
        source: parent.object_ref(source),
    };
    let saved = parent.snapshot();
    let revision = parent.revision();
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
    indexed.apply_index(p, revision, index).unwrap();
    for g in [&mut raw, &mut indexed] {
        drain(g);
        let hit = event(g, source, p, 2, true);
        finish_hits(g, &[hit]);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    drain(&mut indexed);
    let hit = event(&indexed, source, p, 2, true);
    finish_hits(&mut indexed, &[hit]);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn chosen_permanent_spell_follows_only_its_exact_stack_to_battlefield_incarnation() {
    const FLASH_BEAR: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        keywords: Keywords::FLASH,
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &FLASH_BEAR)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        // Give the opponent an ordinary cast, then respond with the prevention spell.
        g.state.priority = p.other();
        cast(&mut g, creature, None);
        g.state.priority = p;
        let chosen = g.object_ref(creature);
        source_choice(&mut g);
        act(&mut g, Action::ChooseDamageSource { source: chosen });
        drain(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
        let current = g.object_ref(creature);
        assert_ne!(chosen, current);
        assert_eq!(
            g.observe(p).chosen_source_shields.unwrap()[0].permanent,
            Some(current)
        );
        let hit = event(&g, creature, p, 2, true);
        finish_hits(&mut g, &[hit]);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}
#[test]
fn a_chosen_permanent_that_leaves_and_returns_is_distinct_while_its_departed_lki_still_matches() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let old = g.damage_source(source);
    shield(&mut g, old.reference);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    let hit = event(&g, source, p, 1, true);
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 19);
    assert!(g.observe(p).chosen_source_shields.is_some());
    finish_hits(
        &mut g,
        &[DamageEvent {
            preventable: true,
            source: old,
            target: Target::Player(p),
            amount: 2,
            combat: false,
        }],
    );
    assert_eq!(g.state.players[p.index()].life, 19);
    assert_eq!(g.state.players[p.other().index()].life, 18);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
}
#[test]
fn an_ability_source_that_already_departed_can_be_chosen_and_uses_its_lki_controller() {
    const SAC_SHOOT: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            })),
            ..ability(&[Instruction::Damage(3)])
        }],
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(card("Opt"), &NWF), (card("Llanowar Elves"), &SAC_SHOOT)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let old = g.object_ref(source);
        g.state.priority = p.other();
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
                target: Target::Player(p),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        g.state.priority = p;
        source_choice(&mut g);
        assert!(
            g.observe(p)
                .damage_source_choice
                .unwrap()
                .candidates
                .contains(&old)
        );
        assert!(
            !g.observe(p)
                .damage_source_choice
                .unwrap()
                .candidates
                .contains(&g.object_ref(source))
        );
        act(&mut g, Action::ChooseDamageSource { source: old });
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
    }
}
#[test]
fn chosen_spell_copy_and_countered_spell_reentry_do_not_inherit_the_original_shield() {
    const FLASH_BEAR: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        keywords: Keywords::FLASH,
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &FLASH_BEAR)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let creature = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
    g.state.priority = p.other();
    cast(&mut g, creature, None);
    let original = g.state.stack.last().unwrap().clone();
    g.state.priority = p;
    source_choice(&mut g);
    let chosen = g.object_ref(creature);
    act(&mut g, Action::ChooseDamageSource { source: chosen });
    // Stop before either permanent spell resolves; create a separate spell copy.
    let copy = g.allocate_spell_copy(&original, p.other());
    let copied = copy.source.unwrap().id;
    g.state.stack.push(copy);
    drain(&mut g);
    assert_eq!(g.object_view(copied).zone, Zone::Battlefield);
    let hit = event(&g, copied, p, 1, true);
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 19);
    assert!(g.observe(p).chosen_source_shields.is_some());
    let hit = event(&g, creature, p, 2, true);
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 19);
    // A countered spell's later graveyard-to-battlefield entry cannot continue it.
    let mut g = scenario(p);
    let creature = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
    g.state.priority = p.other();
    cast(&mut g, creature, None);
    let stack_id = g.state.stack.last().unwrap().id;
    g.state.priority = p;
    source_choice(&mut g);
    let chosen = g.object_ref(creature);
    act(&mut g, Action::ChooseDamageSource { source: chosen });
    assert!(g.counter_stack_item(stack_id, false));
    drain(&mut g);
    g.move_object(creature, Zone::Battlefield);
    assert!(
        g.observe(p).chosen_source_shields.unwrap()[0]
            .permanent
            .is_none()
    );
    let hit = event(&g, creature, p, 2, true);
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 18);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
}
#[test]
fn source_prevention_replacement_order_locks_the_actual_prevented_amount_and_is_owned() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Short Bow"), &NERIV)]);
    for double_first in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let doubler = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let chosen = g.object_ref(source);
        shield(&mut g, chosen);
        let hit = event(&g, source, p, 2, true);
        g.apply_damage_batch(&[hit]);
        g.advance_replacements(true);
        assert!(matches!(g.state.pending,Pending::Replacement{player} if player==p));
        let saved = g.snapshot();
        let action = if double_first {
            Action::ChooseReplacement {
                source: g.object_ref(doubler),
                ability: 0,
            }
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::ChooseDamagePrevention {
                            prevention: DamagePreventionKind::ChosenSource(_),
                            ..
                        }
                    )
                })
                .unwrap()
        };
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let revision = g.revision();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        raw.apply(p, action).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        for branch in [&mut raw, &mut indexed] {
            branch.give_priority();
            drain(branch);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        let amount = if double_first { 4 } else { 2 };
        assert_eq!(raw.state.players[p.index()].life, 20);
        assert_eq!(raw.state.players[p.other().index()].life, 20 - amount);
        assert_eq!(
            raw.state.players[p.index()].cards_drawn_this_turn,
            amount as u64
        );
        indexed.restore(&saved).unwrap();
        assert_eq!(indexed.snapshot(), saved);
    }
}
#[test]
fn two_independent_source_shields_allow_the_affected_player_to_choose_one_and_keep_the_other() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let chosen = g.object_ref(source);
    shield(&mut g, chosen);
    shield(&mut g, chosen);
    let ids = g
        .observe(p)
        .chosen_source_shields
        .unwrap()
        .iter()
        .map(|r| r.id)
        .collect::<Vec<_>>();
    assert_ne!(ids[0], ids[1]);
    let hit = event(&g, source, p, 2, true);
    g.apply_damage_batch(&[hit]);
    g.advance_replacements(true);
    let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ChooseDamagePrevention{prevention:DamagePreventionKind::ChosenSource(id),..} if *id==ids[1])).unwrap();
    act(&mut g, action);
    g.give_priority();
    drain(&mut g);
    let left = g.observe(p).chosen_source_shields.unwrap();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].id, ids[0]);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    let hit = event(&g, source, p, 3, true);
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 20);
    assert_eq!(g.state.players[p.other().index()].life, 15);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 5);
}
#[test]
fn source_prevention_reflexive_is_counterable_and_shield_expires_at_cleanup_without_a_callback() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let chosen = g.object_ref(source);
    shield(&mut g, chosen);
    let hit = event(&g, source, p, 2, true);
    g.apply_damage_batch(&[hit]);
    g.advance_replacements(true);
    g.give_priority();
    wait(&mut g, |g| {
        g.state
            .stack
            .iter()
            .any(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
    });
    let packet = g.state.stack.last().unwrap().clone();
    assert!(packet.targets.is_empty());
    assert!(g.counter_stack_item(packet.id, false));
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].life, 20);
    assert_eq!(g.state.players[p.other().index()].life, 20);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    let chosen = g.object_ref(source);
    g.state.priority = p;
    shield(&mut g, chosen);
    g.finish_cleanup();
    assert!(g.observe(p).chosen_source_shields.is_none());
    let hit = event(&g, source, p, 1, true);
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 19);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
}
#[test]
fn global_damage_prevention_prohibition_preserves_the_unconsumed_source_shield() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let chosen = g.object_ref(source);
    shield(&mut g, chosen);
    g.state.damage_prevention_prohibited = true;
    let hit = event(&g, source, p, 1, true);
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 19);
    assert!(g.observe(p).chosen_source_shields.is_some());
    g.state.damage_prevention_prohibited = false;
    let hit = event(&g, source, p, 2, true);
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 19);
    assert_eq!(g.state.players[p.other().index()].life, 18);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
}

#[test]
fn a_chosen_instant_spell_prevents_its_real_stack_damage_and_uses_the_chosen_spell_controller() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bolt = put(&mut g, p.other(), "Lightning Bolt", Zone::Hand);
        g.state.priority = p.other();
        cast(&mut g, bolt, Some(Target::Player(p)));
        let chosen = g.object_ref(bolt);
        g.state.priority = p;
        source_choice(&mut g);
        act(&mut g, Action::ChooseDamageSource { source: chosen });
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        assert_eq!(g.object_view(bolt).zone, Zone::Graveyard);
    }
}
#[test]
fn another_prevention_effect_can_prevent_the_damage_without_consuming_the_chosen_source_shield() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.state.objects[source as usize].gained_keywords = Keywords::PREVENT_COMBAT_BY;
    let chosen = g.object_ref(source);
    shield(&mut g, chosen);
    let mut hit = event(&g, source, p, 3, true);
    hit.combat = true;
    g.apply_damage_batch(&[hit]);
    g.advance_replacements(true);
    act(
        &mut g,
        Action::ChooseDamagePrevention {
            source: chosen,
            prevention: DamagePreventionKind::CombatBySource,
        },
    );
    g.give_priority();
    drain(&mut g);
    assert!(g.observe(p).chosen_source_shields.is_some());
    assert_eq!(g.state.players[p.index()].life, 20);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    g.state.objects[source as usize].gained_keywords = Keywords::NONE;
    let mut hit = event(&g, source, p, 2, true);
    hit.combat = true;
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 20);
    assert_eq!(g.state.players[p.other().index()].life, 18);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
}
#[test]
fn waiting_delayed_trigger_references_are_legal_source_choices_after_their_source_departure() {
    const INSTALLER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ability(&[Instruction::ScheduleTurnTrigger {
            event: TurnCallbackEvent::Attack,
            ability: 0,
            once: true,
        }])],
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        }],
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(card("Opt"), &NWF), (card("Llanowar Elves"), &INSTALLER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let old = g.object_ref(source);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut g);
    drain(&mut g);
    g.move_object(source, Zone::Graveyard);
    let unrelated = put(&mut g, p, "Hill Giant", Zone::Graveyard);
    source_choice(&mut g);
    let choices = g.observe(p).damage_source_choice.unwrap().candidates;
    assert!(choices.contains(&old));
    assert!(!choices.contains(&g.object_ref(source)));
    assert!(!choices.contains(&g.object_ref(unrelated)));
    act(&mut g, Action::ChooseDamageSource { source: old });
    drain(&mut g);
    assert_eq!(
        g.observe(p).chosen_source_shields.unwrap()[0].chosen,
        DamageSourceRef::Object(old)
    );
}
#[test]
fn prevention_callback_keeps_damage_time_controller_after_control_changes_before_the_trigger_resolves()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let chosen = g.object_ref(source);
    shield(&mut g, chosen);
    let hit = event(&g, source, p, 2, true);
    g.apply_damage_batch(&[hit]);
    g.advance_replacements(true);
    g.record_control(g.object_ref(source), p, None);
    g.give_priority();
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].life, 20);
    assert_eq!(g.state.players[p.other().index()].life, 18);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
}

#[test]
fn source_prevention_layout_matches_the_integrated_shared_state_budget() {
    assert_eq!(std::mem::size_of::<Object>(), 160);
    // Measured assembled budget: wide Endure, frozen division and color-sequence
    // packet fields are all present here. Isolated mechanic bases are smaller.
    // Shield/choice collections stay under the existing cold RoleState pointer.
    assert_eq!(std::mem::size_of::<State>(), 4992);
    if let Ok(path) = std::env::var("RUSTMAGE_TDM_WORLD_LAYOUT") {
        let layout = serde_json::json!({
            "Object":std::mem::size_of::<Object>(),"ObjectView":std::mem::size_of::<ObjectView>(),
            "State":std::mem::size_of::<State>(),"DamageSource":std::mem::size_of::<DamageSource>(),
            "Pending":std::mem::size_of::<Pending>(),"SpellView":std::mem::size_of::<SpellView>(),
            "RoleState":std::mem::size_of::<roles::RoleState>(),
            "SourcePreventionState":std::mem::size_of::<source_prevention::SourcePreventionState>(),
            "DamagePreventionKind":std::mem::size_of::<DamagePreventionKind>(),
            "DamagePreventionView":std::mem::size_of::<DamagePreventionView>(),
            "SpellView":std::mem::size_of::<SpellView>(), "EventObjectView":std::mem::size_of::<EventObjectView>(),
            "DamageSourceRef":std::mem::size_of::<DamageSourceRef>(),
            "ReflexiveSubjectView":std::mem::size_of::<ReflexiveSubjectView>(),
            "ExileDeathReplacementView":std::mem::size_of::<ExileDeathReplacementView>(),
            "ManaState":std::mem::size_of::<mana_lots::ManaState>()
        });
        std::fs::write(path, serde_json::to_string_pretty(&layout).unwrap()).unwrap();
    }
}

#[test]
fn unpreventable_damage_applies_the_chosen_prevention_once_without_using_up_the_next_instance_effect()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Short Bow"), &NERIV)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let doubler = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
    let source = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let chosen = g.object_ref(source);
    shield(&mut g, chosen);
    let hit = event(&g, source, p, 2, false);
    g.apply_damage_batch(&[hit]);
    g.advance_replacements(true);
    // Applying the shield is a legal event-local choice even though it cannot
    // prevent any damage; the doubling choice remains independent.
    let prevention = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(
                a,
                Action::ChooseDamagePrevention {
                    prevention: DamagePreventionKind::ChosenSource(_),
                    ..
                }
            )
        })
        .unwrap();
    act(&mut g, prevention);
    g.give_priority();
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].life, 16);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    assert!(g.observe(p).chosen_source_shields.is_some());
    g.move_object(doubler, Zone::Hand);
    let hit = event(&g, source, p, 1, true);
    finish_hits(&mut g, &[hit]);
    assert_eq!(g.state.players[p.index()].life, 16);
    assert_eq!(g.state.players[p.other().index()].life, 19);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn a_waiting_prevention_effect_exposes_its_exact_departed_spell_source_for_another_shield() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let attacker = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let chosen = g.object_ref(attacker);
    let first = shield(&mut g, chosen);
    let old_spell = g.observe(p).chosen_source_shields.unwrap()[0].source;
    assert_ne!(old_spell, g.object_ref(first));
    let second = put(&mut g, p.other(), "Opt", Zone::Hand);
    g.state.priority = p.other();
    cast(&mut g, second, None);
    wait(&mut g, |g| {
        matches!(g.state.pending, Pending::DamageSource { .. })
    });
    let choices = g
        .observe(p.other())
        .damage_source_choice
        .unwrap()
        .candidates;
    assert!(choices.contains(&old_spell));
    assert!(!choices.contains(&g.object_ref(first)));
    act(&mut g, Action::ChooseDamageSource { source: old_spell });
    drain(&mut g);
    let hit = event(&g, attacker, p, 2, true);
    finish_hits(&mut g, &[hit]);
    // First callback's damage is prevented by the second shield. Both reflexive
    // abilities still draw their own actual prevented amounts independently.
    assert_eq!(g.state.players[p.index()].life, 18);
    assert_eq!(g.state.players[p.other().index()].life, 20);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 2);
}
#[test]
fn a_waiting_replacement_effect_exposes_its_historical_source_without_exposing_the_new_graveyard_card()
 {
    const MARKING: CardRules = CardRules {
        effect: Effect::Spell(
            CREATURE,
            &[
                Instruction::Damage(1),
                Instruction::ExileDamagedIfDiesThisTurn,
            ],
        ),
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &MARKING)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let creature = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(creature)));
    let old = g.object_ref(spell);
    drain(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert!(
        g.state
            .death_exile_replacements
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|row| row.source == old))
    );
    source_choice(&mut g);
    let choices = g.observe(p).damage_source_choice.unwrap().candidates;
    assert!(choices.contains(&old));
    assert!(!choices.contains(&g.object_ref(spell)));
    act(&mut g, Action::ChooseDamageSource { source: old });
    drain(&mut g);
    assert_eq!(
        g.observe(p).chosen_source_shields.unwrap()[0].chosen,
        DamageSourceRef::Object(old)
    );
}

#[test]
fn one_mass_copy_effect_has_shared_provider_chronology_with_independent_acquisitions_and_expiry() {
    const BLUEPRINT: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::MaximumHandSize(3)],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &MASS), (card("Bear Cub"), &BLUEPRINT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let b = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let spell = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(target)));
    drain(&mut g);
    let a_copy = g.object_view(a).temporary_copy.unwrap();
    let b_copy = g.object_view(b).temporary_copy.unwrap();
    let a_layer = a_copy.layers.last().unwrap();
    let b_layer = b_copy.layers.last().unwrap();
    assert_ne!(
        a_layer.origin, b_layer.origin,
        "independent ability acquisitions"
    );
    assert_eq!(
        a_layer.provider_timestamp, b_layer.provider_timestamp,
        "one resolved continuous effect"
    );
    assert!(a_layer.hand_timestamp.is_some());
    assert_eq!(a_layer.hand_timestamp, b_layer.hand_timestamp);
    g.grant_no_maximum_hand_size(p);
    assert_eq!(g.maximum_hand_size(p), None);
    // A later independent copy overrides just its own prior layer, not its sibling.
    g.apply_temporary_copy(
        g.object_ref(a),
        CopyRecipe::new(card("Shivan Dragon")),
        CopyDeadline::EndOfTurn,
    );
    assert_eq!(g.object_view(a).card, card("Shivan Dragon"));
    assert_eq!(g.object_view(b).card, card("Bear Cub"));
    g.finish_cleanup();
    assert_eq!(g.object_view(a).card, card("Hill Giant"));
    assert_eq!(g.object_view(b).card, card("Llanowar Elves"));
}

#[test]
fn source_world_completion_mass_copy_never_awards_a_story_from_a_transient_recipient_prefix() {
    const ARTIFACT_CREATURE: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        keywords: Keywords::HASTE,
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (card("Opt"), &MASS),
        (card("Gingerbrute"), &ARTIFACT_CREATURE),
    ]);
    for preexisting in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let bears = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        // Gingerbrute's ordinary definition is gated in this base. Construct its
        // real physical metadata with a complete typed artifact-creature fixture;
        // no production admission or catalogue entry is altered.
        let mut artifact = g.state.objects[bears as usize].clone();
        artifact.card = card("Gingerbrute");
        let blueprint = g.state.objects.len() as CardId;
        g.state.objects.push(artifact);
        g.register_battlefield_sources(blueprint);
        let balin = put(&mut g, p, "Balin, Loremaster", Zone::Battlefield);
        if preexisting {
            put(&mut g, p, "Short Bow", Zone::Battlefield);
        }
        g.refresh_enduring_stories();
        assert_eq!(g.state.players[p.index()].enduring_story, preexisting);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(blueprint)));
        let saved = g.snapshot();
        let revision = g.revision();
        let index = g
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        raw.apply(p, Action::Pass).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        drain(&mut raw);
        drain(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, Action::Pass);
        drain(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        let g = raw;
        assert_eq!(g.object_view(bears).card, card("Gingerbrute"));
        assert_eq!(g.object_view(balin).card, card("Gingerbrute"));
        assert_eq!(
            g.state.players[p.index()].enduring_story,
            preexisting,
            "only a genuinely acquired prior Story survives; no intermediate recipient world is observed"
        );
    }
}
#[test]
fn source_world_completion_incidental_equip_bearer_is_excluded_but_departed_target_is_referred() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let bow = put(&mut g, p, "Short Bow", Zone::Battlefield);
    let old_a = g.object_ref(a);
    let old_b = g.object_ref(b);
    assert!(g.attach_equipment(g.object_ref(bow), old_a));
    act(
        &mut g,
        Action::AnnounceActivate {
            card: bow,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    pay(&mut g);
    g.move_object(a, Zone::Graveyard);
    g.move_object(b, Zone::Graveyard);
    g.state.priority = p;
    source_choice(&mut g);
    let choices = g.observe(p).damage_source_choice.unwrap().candidates;
    assert!(
        !choices.contains(&old_a),
        "the old Equip bearer is only incidental announcement metadata"
    );
    assert!(
        choices.contains(&old_b),
        "the actual Equip target remains a semantic reference"
    );
    assert!(!choices.contains(&g.object_ref(a)));
    assert!(!choices.contains(&g.object_ref(b)));
    reject(&mut g, Action::ChooseDamageSource { source: old_a });
    act(&mut g, Action::ChooseDamageSource { source: old_b });
    drain(&mut g);
}
#[test]
fn source_world_completion_dying_event_identity_is_referred_even_when_effect_uses_no_event_facts() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let bloodwitch = put(&mut g, p, "Vengeful Bloodwitch", Zone::Battlefield);
    let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let before = g.object_ref(creature);
    g.move_object(creature, Zone::Graveyard);
    let after = g.object_ref(creature);
    g.give_priority();
    for _ in 0..64 {
        if g.state
            .stack
            .iter()
            .any(|item| item.source.is_some_and(|source| source.id == bloodwitch))
        {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
                || matches!(a,Action::ChooseTarget{target:Target::Player(player)} if *player==p.other())
        });
    }
    assert!(
        g.state
            .stack
            .iter()
            .any(|item| item.source.is_some_and(|source| source.id == bloodwitch))
    );
    g.state.priority = p;
    source_choice(&mut g);
    let choices = g.observe(p).damage_source_choice.unwrap().candidates;
    assert!(
        choices.contains(&before),
        "the dying creature is referred to by the actual trigger condition"
    );
    assert!(
        choices.contains(&after),
        "the zone-change trigger can find its public resulting incarnation under400.7e"
    );
    act(&mut g, Action::ChooseDamageSource { source: before });
    drain(&mut g);
}
#[test]
fn source_world_completion_departed_physical_stack_spell_keeps_its_target_reference_without_rebinding()
 {
    const COUNTER: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
            &[Instruction::Counter],
        ),
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &COUNTER)]);
    for copied in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let card_id = put(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, card_id, None);
        let original = g.state.stack.last().unwrap().clone();
        let (spell, old, stack_id, target) = if copied {
            let copy = g.allocate_spell_copy(&original, p);
            let source = copy.source.unwrap();
            let id = copy.id;
            g.state.stack.push(copy);
            (source.id, source.object, id, Target::Stack(id))
        } else {
            (
                card_id,
                g.object_ref(card_id),
                original.id,
                Target::Spell(card_id),
            )
        };
        let counter = put(&mut g, p, "Lightning Bolt", Zone::Hand);
        cast(&mut g, counter, Some(target));
        assert!(g.counter_stack_item(stack_id, false));
        assert_ne!(g.object_ref(spell), old);
        g.state.priority = p;
        source_choice(&mut g);
        let choices = g.observe(p).damage_source_choice.unwrap().candidates;
        assert!(
            choices.contains(&old),
            "the waiting counterspell still refers to its exact old spell object; copied={copied}"
        );
        assert!(
            !choices.contains(&g.object_ref(spell)),
            "a later incarnation is a different object"
        );
        act(&mut g, Action::ChooseDamageSource { source: old });
        drain(&mut g);
    }
}

const fn event_trigger(condition: TriggerCondition) -> TriggeredAbility {
    TriggeredAbility {
        condition,
        targets: TargetPlan::NONE,
        program: &[],
    }
}
const fn event_observer(triggers: &'static [TriggeredAbility]) -> CardRules {
    CardRules { triggers, ..PLAIN }
}
fn stack_event_observers(g: &mut Game) {
    // Preserve an already active simultaneous-trigger ordering choice.
    if g.state.pending == Pending::Priority {
        g.give_priority();
    }
    for _ in 0..128 {
        if g.state.pending == Pending::Priority
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            )
        });
    }
    panic!("event observer staging bound: {:?}", g.state.pending);
}
fn source_candidates(g: &mut Game) -> Vec<ObjectRef> {
    stack_event_observers(g);
    g.state.priority = g.state.active;
    source_choice(g);
    assert!(
        g.observe(g.state.active.other())
            .damage_source_choice
            .is_none()
    );
    g.observe(g.state.active)
        .damage_source_choice
        .unwrap()
        .candidates
}
#[test]
fn source_world_completion_referred_ability_is_a_distinct_legal_no_damage_source_owned_in_both_realms()
 {
    const COUNTER_ABILITY: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
            &[Instruction::Counter],
        ),
        ..CardRules::EMPTY
    };
    const ACTIVATOR: CardRules = CardRules {
        activated: &[ability(&[])],
        ..PLAIN
    };
    let _scope = fixture_rules::activate(&[
        (card("Opt"), &NWF),
        (card("Lightning Bolt"), &COUNTER_ABILITY),
        (card("Bear Cub"), &ACTIVATOR),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut parent = scenario(p);
        let source = put(&mut parent, p, "Bear Cub", Zone::Battlefield);
        let old = parent.object_ref(source);
        let damage = parent.damage_source(source);
        act(
            &mut parent,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut parent);
        let id = parent.state.stack.last().unwrap().id;
        let counter = put(&mut parent, p, "Lightning Bolt", Zone::Hand);
        cast(&mut parent, counter, Some(Target::Stack(id)));
        assert!(
            parent.state.stack.last().unwrap().targets[0]
                .object
                .is_none(),
            "an ability's source permanent is not its target object"
        );
        assert!(parent.counter_stack_item(id, false));
        parent.move_object(source, Zone::Graveyard);
        parent.state.priority = p;
        source_choice(&mut parent);
        let choice = parent.observe(p).damage_source_choice.unwrap();
        assert!(choice.stack_objects.contains(&id));
        assert!(
            !choice.candidates.contains(&old),
            "countering an ability does not name its source permanent"
        );
        assert!(parent.observe(p.other()).damage_source_choice.is_none());
        let action = Action::ChooseStackDamageSource { id };
        reject(
            &mut parent,
            Action::ChooseStackDamageSource { id: u64::MAX },
        );
        reject(&mut parent, Action::ChooseDamageSource { source: old });
        let saved = parent.snapshot();
        let revision = parent.revision();
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
        indexed.apply_index(p, revision, index).unwrap();
        for g in [&mut raw, &mut indexed] {
            drain(g);
            assert_eq!(
                g.observe(p).chosen_source_shields.unwrap()[0].chosen,
                DamageSourceRef::StackObject { stack_object: id }
            );
            finish_hits(
                g,
                &[DamageEvent {
                    source: damage,
                    target: Target::Player(p),
                    amount: 2,
                    combat: false,
                    preventable: true,
                }],
            );
            assert_eq!(
                g.state.players[p.index()].life,
                18,
                "the old ability object is not the damage actor"
            );
            assert!(g.observe(p).chosen_source_shields.is_some());
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, action);
        drain(&mut indexed);
        finish_hits(
            &mut indexed,
            &[DamageEvent {
                source: damage,
                target: Target::Player(p),
                amount: 2,
                combat: false,
                preventable: true,
            }],
        );
        assert_eq!(raw.snapshot(), indexed.snapshot());
        let json = serde_json::to_value(raw.observe(p)).unwrap();
        assert_eq!(
            json["chosen_source_shields"][0]["chosen"]["stack_object"],
            id
        );
    }
}
#[test]
fn source_world_completion_ordinary_cast_condition_keeps_spell_and_only_qualifying_nested_targets()
{
    const OBS: CardRules = event_observer(&[event_trigger(TriggerCondition::SpellCast {
        relation: CastRelation::You,
        filter: CastFilter::AnyOf(&[
            CastFilter::Targeted {
                filter: &CastFilter::Any,
                target: CastTarget::Creature(CastRelation::You),
            },
            CastFilter::Creature,
        ]),
        ordinal: None,
    })]);
    const TARGETED: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 2,
                max: 2,
            }),
            &[],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (card("Opt"), &NWF),
        (card("Bear Cub"), &OBS),
        (card("Lightning Bolt"), &TARGETED),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let own = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let own_ref = g.object_ref(own);
    let other_ref = g.object_ref(other);
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(other),
        },
    );
    pay(&mut g);
    let old_spell = g.object_ref(spell);
    let id = g
        .state
        .stack
        .iter()
        .find(|item| {
            item.kind == StackKind::Spell && item.source.is_some_and(|source| source.id == spell)
        })
        .unwrap()
        .id;
    stack_event_observers(&mut g);
    assert!(g.counter_stack_item(id, false));
    g.move_objects_raw(&[own, other], Zone::Graveyard);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old_spell));
    assert!(choices.contains(&own_ref));
    assert!(!choices.contains(&other_ref));
    assert!(!choices.contains(&g.object_ref(spell)));
    assert!(!choices.contains(&g.object_ref(own)));
}
#[test]
fn source_world_completion_damage_aggregate_retains_named_dealers_but_player_only_clause_excludes_them()
 {
    for named in [false, true] {
        const NAMED: CardRules =
            event_observer(&[event_trigger(TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Creatures {
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    qualification: DamageQualification::Any,
                },
                combat_only: false,
                opponent_only: true,
                one_or_more: true,
            })]);
        const PLAYERS: CardRules =
            event_observer(&[event_trigger(TriggerCondition::PlayerDamaged {
                subject: DamageSubject::AnySources,
                combat_only: false,
                opponent_only: true,
                one_or_more: false,
            })]);
        let _scope = fixture_rules::activate(&[
            (card("Opt"), &NWF),
            (card("Bear Cub"), if named { &NAMED } else { &PLAYERS }),
        ]);
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let old = [g.object_ref(a), g.object_ref(b)];
        let hits = [
            event(&g, a, p.other(), 1, true),
            event(&g, b, p.other(), 2, true),
        ];
        g.player_damage_triggers(&hits);
        stack_event_observers(&mut g);
        g.move_objects_raw(&[a, b], Zone::Graveyard);
        let choices = source_candidates(&mut g);
        for object in old {
            assert_eq!(choices.contains(&object), named);
        }
        assert!(!choices.contains(&g.object_ref(a)));
        assert!(!choices.contains(&g.object_ref(b)));
    }
}
#[test]
fn source_world_completion_controlled_tap_batch_keeps_every_matching_old_creature_and_excludes_opponents_and_lands()
 {
    const OBS: CardRules = event_observer(&[event_trigger(
        TriggerCondition::ControlledCreaturesBecomeTapped,
    )]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let c = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
    let land = put(&mut g, p, "Forest", Zone::Battlefield);
    let old = [
        g.object_ref(a),
        g.object_ref(b),
        g.object_ref(c),
        g.object_ref(land),
    ];
    g.tap_objects(&[a, b, c, land]);
    stack_event_observers(&mut g);
    g.move_objects_raw(&[a, b, c, land], Zone::Graveyard);
    let choices = source_candidates(&mut g);
    for (i, object) in old.into_iter().enumerate() {
        assert_eq!(choices.contains(&object), i < 2);
    }
}
#[test]
fn source_world_completion_counter_batch_and_single_recipient_conditions_preserve_exact_matching_sets()
 {
    const OBS: CardRules = event_observer(&[
        event_trigger(TriggerCondition::CounterBatchPlacedByYou {
            kind: CounterKind::PlusOne,
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 1,
                exclude: 0,
                subtype: None,
                basic: false,
                other: true,
                cards_only: false,
            },
        }),
        event_trigger(TriggerCondition::OtherCreatureCounterPlacedByYou {
            kind: CounterKind::PlusOne,
        }),
    ]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let c = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
    let land = put(&mut g, p, "Forest", Zone::Battlefield);
    let old = [
        g.object_ref(a),
        g.object_ref(b),
        g.object_ref(c),
        g.object_ref(land),
    ];
    let plus = Counters {
        plus_one: 1,
        ..Counters::default()
    };
    let rows = [
        (old[0], &plus, p),
        (old[1], &plus, p),
        (old[2], &plus, p.other()),
        (old[3], &plus, p),
    ];
    g.counter_batch_added(rows.iter().copied());
    g.self_counters_added(old[0], CounterKind::PlusOne, 1, p);
    stack_event_observers(&mut g);
    g.move_objects_raw(&[a, b, c, land], Zone::Graveyard);
    let choices = source_candidates(&mut g);
    for (i, object) in old.into_iter().enumerate() {
        assert_eq!(choices.contains(&object), i < 2);
    }
}
#[test]
fn source_world_completion_creature_declaration_and_player_attack_are_explicitly_distinct_after_departure()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    for explicit in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let observer = put(
            &mut g,
            p,
            if explicit {
                "Ancestor Dragon"
            } else {
                "Battlesong Berserker"
            },
            Zone::Battlefield,
        );
        let attacker = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let old = g.object_ref(attacker);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        g.state.objects[attacker as usize].summoning_sick = false;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        if !explicit {
            // Battlesong's actual mandatory target is independent of the
            // creature that attacked. Complete it legally without rebinding
            // the attacked creature as a target.
            wait(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(observer),
                },
            );
        }
        stack_event_observers(&mut g);
        g.move_object(attacker, Zone::Graveyard);
        let choices = source_candidates(&mut g);
        assert_eq!(choices.contains(&old), explicit);
        assert!(!choices.contains(&g.object_ref(attacker)));
    }
}
#[test]
fn source_world_completion_successful_draw_refers_only_to_the_actual_card_and_not_earlier_draws_or_private_faces()
 {
    const OBS: CardRules = event_observer(&[event_trigger(TriggerCondition::CardDrawn {
        relation: CastRelation::Opponent,
        ordinal: Some(2),
    })]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let first = g.top_library_card(p.other()).unwrap();
    let old_first = g.object_ref(first);
    assert!(g.draw_one_unreplaced(p.other()));
    let new_first = g.object_ref(first);
    assert!(g.state.pending_triggers.is_empty());
    let second = g.top_library_card(p.other()).unwrap();
    let old_second = g.object_ref(second);
    assert!(g.draw_one_unreplaced(p.other()));
    let drawn = g.object_ref(second);
    stack_event_observers(&mut g);
    g.move_object(second, Zone::Exile);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&drawn));
    assert!(!choices.contains(&old_second));
    assert!(!choices.contains(&old_first));
    assert!(!choices.contains(&new_first));
    assert!(!choices.contains(&g.object_ref(second)));
    let json = serde_json::to_value(g.observe(p).damage_source_choice.unwrap()).unwrap();
    assert!(json["candidates"].as_array().unwrap().iter().all(|row| {
        row.as_object()
            .unwrap()
            .keys()
            .all(|k| k == "card" || k == "generation")
    }));
    reject(&mut g, Action::ChooseDamageSource { source: old_second });
    act(&mut g, Action::ChooseDamageSource { source: drawn });
    drain(&mut g);
}
#[test]
fn source_world_completion_prevented_and_empty_draws_create_neither_event_nor_hidden_source_reference()
 {
    const OBS: CardRules = event_observer(&[event_trigger(TriggerCondition::CardDrawn {
        relation: CastRelation::You,
        ordinal: None,
    })]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let top = g.top_library_card(p).unwrap();
    let old = g.object_ref(top);
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        }
    }
    let _reset = Reset;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(Some(old)));
    assert!(!g.draw_one_unreplaced(p));
    assert!(g.state.pending_triggers.is_empty());
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
    g.state.players[p.index()].library.clear();
    assert!(!g.draw_one_unreplaced(p));
    assert!(g.state.pending_triggers.is_empty());
}
#[test]
fn source_world_completion_discard_batches_keep_each_explicit_old_card_and_public_result_without_unrelated_hands()
 {
    const OBS: CardRules = event_observer(&[event_trigger(TriggerCondition::Discarded {
        relation: CastRelation::You,
        one_or_more: true,
    })]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let a = put(&mut g, p, "Hill Giant", Zone::Hand);
    let b = put(&mut g, p, "Grizzly Bears", Zone::Hand);
    let untouched = put(&mut g, p.other(), "Shivan Dragon", Zone::Hand);
    let old = [g.object_ref(a), g.object_ref(b)];
    g.discard_batch(&[a, b], None);
    let public = [g.object_ref(a), g.object_ref(b)];
    stack_event_observers(&mut g);
    g.move_objects_raw(&[a, b], Zone::Exile);
    let choices = source_candidates(&mut g);
    for object in old.into_iter().chain(public) {
        assert!(choices.contains(&object));
    }
    assert!(!choices.contains(&g.object_ref(untouched)));
    assert!(!choices.contains(&g.object_ref(a)));
    assert!(!choices.contains(&g.object_ref(b)));
}
#[test]
fn source_world_completion_simultaneous_deaths_refer_to_all_matching_old_and_public_result_objects()
{
    const OBS: CardRules = event_observer(&[event_trigger(TriggerCondition::CreatureDiesBatch {
        filter: CreatureFilter {
            relation: 0,
            other: true,
            subtype: None,
            keyword: Keywords::NONE,
            attacking: false,
            tokens: None,
            artifact: false,
        },
    })]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let c = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
    let old = [g.object_ref(a), g.object_ref(b), g.object_ref(c)];
    let before = g.death_trigger_sources();
    let events = g.move_objects_raw(&[a, b, c], Zone::Graveyard);
    g.finish_zone_events(&events, &before);
    let public = [g.object_ref(a), g.object_ref(b), g.object_ref(c)];
    let choices = source_candidates(&mut g);
    for (i, object) in old.into_iter().enumerate() {
        assert_eq!(choices.contains(&object), i < 2);
    }
    for (i, object) in public.into_iter().enumerate() {
        assert_eq!(choices.contains(&object), i < 2);
    }
}
#[test]
fn source_world_completion_graveyard_departure_batches_retain_only_matching_owner_and_kind() {
    const OBS: CardRules =
        event_observer(&[event_trigger(TriggerCondition::CardsLeaveGraveyardKinds(1))]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let a = put(&mut g, p, "Hill Giant", Zone::Graveyard);
    let b = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let c = put(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
    let d = put(&mut g, p, "Divination", Zone::Graveyard);
    let old = [
        g.object_ref(a),
        g.object_ref(b),
        g.object_ref(c),
        g.object_ref(d),
    ];
    let before = g.death_trigger_sources();
    let events = g.move_objects_raw(&[a, b, c, d], Zone::Exile);
    g.finish_zone_events(&events, &before);
    let public = [
        g.object_ref(a),
        g.object_ref(b),
        g.object_ref(c),
        g.object_ref(d),
    ];
    stack_event_observers(&mut g);
    g.move_objects_raw(&[a, b, c, d], Zone::Hand);
    let choices = source_candidates(&mut g);
    for (i, object) in old.into_iter().enumerate() {
        assert_eq!(choices.contains(&object), i < 2);
    }
    for (i, object) in public.into_iter().enumerate() {
        assert_eq!(choices.contains(&object), i < 2);
    }
}
#[test]
fn source_world_completion_from_anywhere_graveyard_entry_references_new_cards_not_old_hidden_library_objects()
 {
    const OBS: CardRules = event_observer(&[event_trigger(
        TriggerCondition::PermanentCardsEnterYourGraveyard,
    )]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let a = put(&mut g, p, "Hill Giant", Zone::Library);
    let b = put(&mut g, p, "Forest", Zone::Library);
    let c = put(&mut g, p, "Divination", Zone::Library);
    let old = [g.object_ref(a), g.object_ref(b), g.object_ref(c)];
    let before = g.death_trigger_sources();
    let events = g.move_objects_raw(&[a, b, c], Zone::Graveyard);
    g.finish_zone_events(&events, &before);
    let public = [g.object_ref(a), g.object_ref(b), g.object_ref(c)];
    stack_event_observers(&mut g);
    g.move_objects_raw(&[a, b, c], Zone::Exile);
    let choices = source_candidates(&mut g);
    for object in old {
        assert!(!choices.contains(&object));
    }
    for (i, object) in public.into_iter().enumerate() {
        assert_eq!(choices.contains(&object), i < 2);
    }
}
#[test]
fn source_world_completion_food_sacrifice_uses_precommit_food_facts_and_actual_public_results() {
    const OBS: CardRules = event_observer(&[event_trigger(TriggerCondition::FoodsSacrificed)]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let food = g.state.objects.len() as CardId;
    g.create_tokens(p, Card::test_index(5246), 1, false);
    g.advance_replacements(true);
    assert!(g.on_battlefield(food));

    let other = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let old = [g.object_ref(food), g.object_ref(other)];
    g.sacrifice_batch(&[(p, food), (p, other)]);
    let public = [g.object_ref(food), g.object_ref(other)];
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old[0]));
    assert!(choices.contains(&public[0]));
    assert!(!choices.contains(&old[1]));
    assert!(!choices.contains(&public[1]));
}
#[test]
fn source_world_completion_token_entry_batch_keeps_every_matching_entrant_without_later_graveyard_incarnations()
 {
    const OBS: CardRules = event_observer(&[event_trigger(TriggerCondition::TokensEnterYou)]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let start = g.state.objects.len();
    g.create_tokens(p, Card::test_index(5246), 2, false);
    g.advance_replacements(true);
    let own: Vec<_> = (start..g.state.objects.len())
        .filter(|&i| g.state.objects[i].zone == Zone::Battlefield)
        .map(|i| g.object_ref(i as CardId))
        .collect();
    assert_eq!(own.len(), 2);
    let unrelated = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let unrelated_ref = g.object_ref(unrelated);
    stack_event_observers(&mut g);
    let ids: Vec<_> = own.iter().map(|r| r.card).chain([unrelated]).collect();
    g.move_objects_raw(&ids, Zone::Graveyard);
    let choices = source_candidates(&mut g);
    for object in own {
        assert!(choices.contains(&object));
        assert!(!choices.contains(&g.object_ref(object.card)));
    }
    assert!(!choices.contains(&unrelated_ref));
}
#[test]
fn source_world_completion_exact_delayed_dies_subject_survives_dispatch_and_public_move() {
    const WATCH: CardRules = CardRules {
        effect: Effect::Spell(
            CREATURE,
            &[Instruction::DelayTargetEvent {
                event: DelayedTargetEvent::DiesOnce,
                ability: 0,
            }],
        ),
        triggers: &[event_trigger(TriggerCondition::DelayedTargetEvent)],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &WATCH)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let old = g.object_ref(creature);
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(creature)));
    drain(&mut g);
    g.move_object(creature, Zone::Graveyard);
    let after = g.object_ref(creature);
    assert!(g.state.delayed_target_events.is_none());
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old));
    assert!(choices.contains(&after));
}
#[test]
fn source_world_completion_exact_delayed_combat_dealer_is_retained_after_watcher_and_subject_departure()
 {
    const WATCH: CardRules = CardRules {
        effect: Effect::Spell(
            CREATURE,
            &[Instruction::DelayTargetEvent {
                event: DelayedTargetEvent::CombatDamageToPlayer,
                ability: 0,
            }],
        ),
        triggers: &[event_trigger(TriggerCondition::DelayedTargetEvent)],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &WATCH)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let old = g.object_ref(creature);
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(creature)));
    drain(&mut g);
    let mut hit = event(&g, creature, p.other(), 1, true);
    hit.combat = true;
    g.delayed_target_combat_damage(&[hit]);
    stack_event_observers(&mut g);
    g.move_object(creature, Zone::Graveyard);
    assert!(
        g.state
            .delayed_target_events
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|row| row.subject == old)),
        "the turn-duration ability still refers to its old subject until cleanup"
    );
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old));
    assert!(!choices.contains(&g.object_ref(creature)));
}
#[test]
fn source_world_completion_earthbend_keeps_only_the_still_watched_land_and_its_actual_trigger_move()
{
    const EARTH: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 2,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 2,
                max: 2,
            }),
            &[
                Instruction::AnimateEarthbendTargets,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::RegisterEarthbendReturn,
            ],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &EARTH)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Forest", Zone::Battlefield);
    let b = put(&mut g, p, "Island", Zone::Battlefield);
    let old = [g.object_ref(a), g.object_ref(b)];
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    pay(&mut g);
    drain(&mut g);
    // A's delayed ability genuinely triggers and finishes. Its old incarnation
    // must not leak through B's cloned initiating target array.
    g.move_object(a, Zone::Graveyard);
    g.give_priority();
    drain(&mut g);
    assert_ne!(g.object_ref(a), old[0]);
    g.move_object(b, Zone::Graveyard);
    let after = g.object_ref(b);
    let choices = source_candidates(&mut g);
    assert!(!choices.contains(&old[0]));
    assert!(
        choices.contains(&g.object_ref(a)),
        "the returned current land is a permanent"
    );
    assert!(choices.contains(&old[1]));
    assert!(choices.contains(&after));
}
#[test]
fn source_world_completion_next_noncreature_callback_keeps_actual_spell_and_discards_installation_targets()
 {
    const INSTALL: CardRules = CardRules {
        activated: &[ActivatedAbility {
            targets: CREATURE,
            ..ability(&[Instruction::ScheduleTurnTrigger {
                event: TurnCallbackEvent::NoncreatureCast,
                ability: 0,
                once: true,
            }])
        }],
        triggers: &[event_trigger(TriggerCondition::DelayedTargetEvent)],
        ..PLAIN
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &INSTALL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let installer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let incidental = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let old_incidental = g.object_ref(incidental);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: installer,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(incidental),
        },
    );
    pay(&mut g);
    drain(&mut g);
    g.move_object(incidental, Zone::Graveyard);
    let spell = put(&mut g, p, "Divination", Zone::Hand);
    cast(&mut g, spell, None);
    let old_spell = g.object_ref(spell);
    let id = g
        .state
        .stack
        .iter()
        .find(|item| {
            item.kind == StackKind::Spell && item.source.is_some_and(|source| source.id == spell)
        })
        .unwrap()
        .id;
    stack_event_observers(&mut g);
    assert!(g.counter_stack_item(id, false));
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old_spell));
    assert!(!choices.contains(&old_incidental));
    assert!(!choices.contains(&g.object_ref(spell)));
}
#[test]
fn source_world_completion_next_upkeep_tokens_refer_only_to_successfully_returned_owned_creatures()
{
    const RETURN: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 3,
                max: 3,
            }),
            &[Instruction::ReturnTargetsCreateNextUpkeep {
                token: Card::test_index(5255),
            }],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &RETURN)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let c = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
    let old = [g.object_ref(a), g.object_ref(b), g.object_ref(c)];
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    for id in [a, b, c] {
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    pay(&mut g);
    drain(&mut g);
    let choices = source_candidates(&mut g);
    for (i, object) in old.into_iter().enumerate() {
        assert_eq!(choices.contains(&object), i < 2);
    }
    assert!(!choices.contains(&g.object_ref(a)));
    assert!(!choices.contains(&g.object_ref(b)));
    assert!(!choices.contains(&g.object_ref(c)));
}
#[test]
fn source_world_completion_creature_activation_keeps_nonmana_ability_and_old_creature_but_mana_has_no_stack_object()
 {
    const OBS: CardRules =
        event_observer(&[event_trigger(TriggerCondition::CreatureAbilityActivated)]);
    const NONMANA: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            ..ability(&[])
        }],
        ..PLAIN
    };
    const MANA: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            ..ability(&[Instruction::AddMana(ManaProduction::Fixed([
                1, 0, 0, 0, 0, 0,
            ]))])
        }],
        ..PLAIN
    };
    for mana in [false, true] {
        let _scope = fixture_rules::activate(&[
            (card("Opt"), &NWF),
            (card("Bear Cub"), &OBS),
            (card("Llanowar Elves"), if mana { &MANA } else { &NONMANA }),
        ]);
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let old = g.object_ref(source);
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                if mana {
                    matches!(a,Action::AnnounceMana{card,..}if*card==source)
                } else {
                    matches!(a,Action::AnnounceActivate{card,..}if*card==source)
                }
            })
            .unwrap();
        act(&mut g, action);
        pay(&mut g);
        let activated = g
            .state
            .stack
            .iter()
            .find(|s| matches!(s.kind, StackKind::ActivatedAbility { .. }))
            .map(|s| s.id);
        stack_event_observers(&mut g);
        if let Some(id) = activated {
            assert!(g.counter_stack_item(id, false));
        }
        let choices = source_candidates(&mut g);
        assert!(choices.contains(&old));
        let stack = g.observe(p).damage_source_choice.unwrap().stack_objects;
        if let Some(id) = activated {
            assert!(stack.contains(&id));
        } else {
            assert!(stack.is_empty(), "mana abilities have no stack objects");
        }
    }
}
#[test]
fn source_world_completion_explore_condition_retains_explorer_and_only_explicitly_named_revealed_card()
 {
    const BARE: CardRules = event_observer(&[event_trigger(TriggerCondition::CreatureExploresYou)]);
    const CARD: CardRules =
        event_observer(&[event_trigger(TriggerCondition::CreatureExploresCardYou {
            land: true,
        })]);
    for named_card in [false, true] {
        let _scope = fixture_rules::activate(&[
            (card("Opt"), &NWF),
            (card("Bear Cub"), if named_card { &CARD } else { &BARE }),
        ]);
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let old = g.object_ref(creature);
        let land = put(&mut g, p, "Forest", Zone::Library);
        let revealed = g.object_ref(land);
        let moved = g.move_object(land, Zone::Hand);
        g.explore_completed(
            ExplorerView {
                kinds: 1,
                object: old,
                card: card("Hill Giant"),
                controller: p,
            },
            Some(RevealedLibraryCard {
                object: revealed,
                card: card("Forest"),
            }),
            Some(Zone::Hand),
            Some(moved.after),
            false,
        );
        stack_event_observers(&mut g);
        g.move_object(creature, Zone::Graveyard);
        let choices = source_candidates(&mut g);
        assert!(choices.contains(&old));
        assert_eq!(choices.contains(&revealed), named_card);
        assert!(
            !choices.contains(&g.object_ref(land)),
            "explore completion is not a zone-move trigger following the card into a hidden zone"
        );
        assert!(!choices.contains(&g.object_ref(creature)));
    }
}
#[test]
fn source_world_completion_bare_manifest_action_excludes_incidental_graveyard_payload_references() {
    const OBS: CardRules = event_observer(&[event_trigger(TriggerCondition::YouManifestDread)]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let incidental = put(&mut g, p, "Divination", Zone::Graveyard);
    let old = g.object_ref(incidental);
    g.queue_manifest_dread_triggers(ManifestDreadEvent {
        player: p,
        manifested: None,
        graveyard_cards: vec![old],
    });
    let choices = source_candidates(&mut g);
    assert!(!choices.contains(&old));
    reject(&mut g, Action::ChooseDamageSource { source: old });
}
#[test]
fn source_world_completion_prevention_reflexive_explicitly_refers_to_the_prevented_dealer_after_it_departs()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let dealer = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let old = g.object_ref(dealer);
    shield(&mut g, old);
    let hit = event(&g, dealer, p, 2, true);
    g.apply_damage_batch(&[hit]);
    g.advance_replacements(true);
    assert!(g.observe(p).chosen_source_shields.is_none());
    stack_event_observers(&mut g);
    g.move_object(dealer, Zone::Graveyard);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old));
    assert!(!choices.contains(&g.object_ref(dealer)));
}
#[test]
fn source_world_completion_reflexive_x_payment_does_not_keep_the_parent_entrants_semantic_identity()
{
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let preserver = put(&mut g, p, "Wildborn Preserver", Zone::Battlefield);
    let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
    cast(&mut g, creature, None);
    wait(&mut g, |g| {
        g.state.stack.iter().any(|item| {
            item.source.is_some_and(|s| s.id == preserver)
                && item.kind == StackKind::TriggeredAbility { ability: 0 }
        })
    });
    assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
    let old = g.object_ref(creature);
    g.move_object(creature, Zone::Graveyard);
    for _ in 0..64 {
        if g.state.stack.iter().any(|item| {
            item.source.is_some_and(|s| s.id == preserver)
                && item.kind == StackKind::TriggeredAbility { ability: 1 }
        }) {
            break;
        }
        choose(&mut g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseOptional { accept: true }
                    | Action::ChooseX { value: 1 }
                    | Action::PayResolutionMana { .. }
            )
        });
    }
    assert!(
        g.state
            .stack
            .iter()
            .any(|item| item.source.is_some_and(|s| s.id == preserver)
                && item.kind == StackKind::TriggeredAbility { ability: 1 })
    );
    let choices = source_candidates(&mut g);
    assert!(
        !choices.contains(&old),
        "the reflexive clause names only Wildborn Preserver and X; the parent trigger has finished"
    );
    assert!(!choices.contains(&g.object_ref(creature)));
    reject(&mut g, Action::ChooseDamageSource { source: old });
    let chosen = g.object_ref(preserver);
    act(&mut g, Action::ChooseDamageSource { source: chosen });
    drain(&mut g);
    assert_eq!(
        g.counter_amounts(preserver).get(CounterKind::PlusOne),
        1,
        "successful mandatory payment remains meaningful"
    );
}
#[test]
fn source_world_completion_final_bolg_rebinds_the_sacrificed_creature_and_its_public_result() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let sacrifice = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let old = g.object_ref(sacrifice);
    let target = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    let bolg = put(&mut g, p, "Bolg of the North", Zone::Hand);
    cast(&mut g, bolg, None);
    let old_spell = g.object_ref(bolg);
    for _ in 0..96 {
        if g.state.stack.iter().any(|item| {
            item.source.is_some_and(|s| s.id == bolg)
                && item.kind == StackKind::TriggeredAbility { ability: 1 }
        }) {
            break;
        }
        choose(&mut g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            ) || matches!(a,Action::Sacrifice{object}if*object==old)
                || matches!(a,Action::ChooseTarget{target:Target::Card(id)}if*id==target)
        });
    }
    assert!(
        g.state
            .stack
            .iter()
            .any(|item| item.source.is_some_and(|s| s.id == bolg)
                && item.kind == StackKind::TriggeredAbility { ability: 1 })
    );
    assert_eq!(g.object_view(sacrifice).zone, Zone::Graveyard);
    let after = g.object_ref(sacrifice);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old));
    assert!(choices.contains(&after));
    assert!(
        !choices.contains(&old_spell),
        "the new clause does not retain Bolg's old entering spell incarnation"
    );
    act(&mut g, Action::ChooseDamageSource { source: old });
    drain(&mut g);
    assert_eq!(g.object_view(target).damage, 2);
}
#[test]
fn source_world_completion_final_attached_upkeep_guard_keeps_exact_trigger_time_creature_after_both_depart()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let old = g.object_ref(creature);
    let aura = put(&mut g, p, "Super Intelligence", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(old);
    g.begin_step(Step::Upkeep);
    stack_event_observers(&mut g);
    assert!(
        g.state
            .stack
            .iter()
            .any(|item| item.source.is_some_and(|s| s.id == aura))
    );
    g.move_object(creature, Zone::Graveyard);
    g.move_object(aura, Zone::Graveyard);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old));
    assert!(!choices.contains(&g.object_ref(creature)));
    assert!(!choices.contains(&g.object_ref(aura)));
}
#[test]
fn source_world_completion_semantic_event_group_survives_owned_raw_index_restore_after_parent_drop()
{
    const OBS: CardRules =
        event_observer(&[event_trigger(TriggerCondition::CreatureAbilityActivated)]);
    const ACT: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            ..ability(&[])
        }],
        ..PLAIN
    };
    let _scope = fixture_rules::activate(&[
        (card("Opt"), &NWF),
        (card("Bear Cub"), &OBS),
        (card("Llanowar Elves"), &ACT),
    ]);
    let p = Player::P1;
    let mut parent = scenario(p);
    put(&mut parent, p, "Bear Cub", Zone::Battlefield);
    let source = put(&mut parent, p, "Llanowar Elves", Zone::Battlefield);
    let old = parent.object_ref(source);
    act(
        &mut parent,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut parent);
    let id = parent
        .state
        .stack
        .iter()
        .find(|item| matches!(item.kind, StackKind::ActivatedAbility { .. }))
        .unwrap()
        .id;
    stack_event_observers(&mut parent);
    assert!(parent.counter_stack_item(id, false));
    source_choice(&mut parent);
    assert!(
        parent
            .observe(p)
            .damage_source_choice
            .unwrap()
            .stack_objects
            .contains(&id)
    );
    let action = Action::ChooseDamageSource { source: old };
    let saved = parent.snapshot();
    let revision = parent.revision();
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
    indexed.apply_index(p, revision, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::ChooseStackDamageSource { id });
    drain(&mut indexed);
    assert_eq!(
        indexed.observe(p).chosen_source_shields.unwrap()[0].chosen,
        DamageSourceRef::StackObject { stack_object: id }
    );
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn source_world_completion_opponent_draw_reference_reveals_no_face_in_paired_hidden_worlds() {
    const OBS: CardRules = event_observer(&[event_trigger(TriggerCondition::CardDrawn {
        relation: CastRelation::Opponent,
        ordinal: None,
    })]);
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &OBS)]);
    let p = Player::P0;
    let mut observations = Vec::new();
    for secret in ["Shivan Dragon", "Divination"] {
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let top = g.top_library_card(p.other()).unwrap();
        g.state.objects[top as usize].card = card(secret);
        assert!(g.draw_one_unreplaced(p.other()));
        let drawn = g.object_ref(top);
        let choices = source_candidates(&mut g);
        assert!(choices.contains(&drawn));
        observations.push(serde_json::to_value(g.observe(p)).unwrap());
    }
    assert_eq!(
        observations[0], observations[1],
        "the event names an opaque exact card, never its private face"
    );
}
#[test]
fn source_world_completion_cast_during_resolution_finishes_parent_before_child_source_choice() {
    const PARENT: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            })),
            &[Instruction::ExileTargetLibraryTopCastFree {
                amount: 2,
                maximum_casts: 1,
            }],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &PARENT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p, "Forest", Zone::Library);
    let child = put(&mut g, p, "Opt", Zone::Library);
    let parent = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, parent, Some(Target::Player(p)));
    wait(&mut g, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
    let exiled_land = g.object_ref(land);
    let parent_id = g.state.resolving.as_ref().unwrap().item.id;
    let child_ref = g.object_ref(child);
    act(
        &mut g,
        Action::CastExiledDuringResolution {
            object: child_ref,
            face: CastFace::Normal,
        },
    );
    pay(&mut g);
    assert_eq!(
        g.state.resolving.as_ref().unwrap().item.id,
        parent_id,
        "casting the child does not resolve it"
    );
    act(&mut g, Action::FinishResolutionCasts);
    assert_eq!(g.object_view(parent).zone, Zone::Graveyard);
    assert!(g.state.resolving.is_none());
    wait(&mut g, |g| {
        matches!(g.state.pending, Pending::DamageSource { .. })
    });
    assert_eq!(
        g.state.resolving.as_ref().unwrap().item.source.unwrap().id,
        child
    );
    let choices = g.observe(p).damage_source_choice.unwrap().candidates;
    assert!(
        !choices.contains(&exiled_land),
        "CR608.2g: the finished parent does not expose its old exile roster"
    );
    reject(
        &mut g,
        Action::ChooseDamageSource {
            source: exiled_land,
        },
    );
}
#[test]
fn source_world_completion_reflexive_discard_rebinds_paid_card_without_incidental_old_parent_refs()
{
    const DISCARD: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Discard,
                    players: PlayerSet::Controller,
                    filter: ANY_FILTER,
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::Reflexive { ability: 0 },
            ],
        ),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        }],
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &DISCARD)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let discarded = put(&mut g, p, "Hill Giant", Zone::Hand);
    let old = g.object_ref(discarded);
    let unrelated = put(&mut g, p, "Grizzly Bears", Zone::Hand);
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, spell, None);
    for _ in 0..64 {
        if g.state.stack.iter().any(|item| {
            item.source.is_some_and(|s| s.id == spell)
                && matches!(item.kind, StackKind::TriggeredAbility { .. })
        }) {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
                || matches!(a,Action::Discard{card}if*card==discarded)
        });
    }
    let after = g.object_ref(discarded);
    assert_ne!(old, after);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old));
    assert!(choices.contains(&after));
    assert!(!choices.contains(&g.object_ref(unrelated)));
    act(&mut g, Action::ChooseDamageSource { source: old });
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].life, 21);
}
#[test]
fn source_world_completion_reflexive_creation_rebinds_every_actual_created_token_after_departure() {
    const CREATE: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::CreateTokensReflexive {
                token: Card::test_index(5246),
                amount: 2,
                ability: 0,
            }],
        ),
        triggers: &[event_trigger(TriggerCondition::Reflexive)],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &CREATE)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, spell, None);
    wait(&mut g, |g| {
        g.state.stack.iter().any(|item| {
            item.source.is_some_and(|s| s.id == spell)
                && matches!(item.kind, StackKind::TriggeredAbility { .. })
        })
    });
    let tokens: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
        .map(|(i, _)| g.object_ref(i as CardId))
        .collect();
    assert_eq!(tokens.len(), 2);
    let ids: Vec<_> = tokens.iter().map(|r| r.card).collect();
    g.move_objects_raw(&ids, Zone::Graveyard);
    let choices = source_candidates(&mut g);
    for token in tokens {
        assert!(choices.contains(&token));
        assert!(!choices.contains(&g.object_ref(token.card)));
    }
}
#[test]
fn source_world_completion_thorin_reflexive_names_only_the_creature_and_equipment_that_actually_attached()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let recipient = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let old_recipient = g.object_ref(recipient);
    let a = put(&mut g, p, "Short Bow", Zone::Battlefield);
    let b = put(&mut g, p, "Short Bow", Zone::Battlefield);
    let old_a = g.object_ref(a);
    let old_b = g.object_ref(b);
    let target = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    let thorin = put(&mut g, p, "Thorin, Mountain-king", Zone::Hand);
    cast(&mut g, thorin, None);
    wait(&mut g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
            && g.state.announcing.as_ref().is_some_and(|a| {
                a.item.source.is_some_and(|s| s.id == thorin)
                    && a.item.kind == StackKind::TriggeredAbility { ability: 0 }
            })
    });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    act(&mut g, Action::FinishTargets);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    for _ in 0..64 {
        if matches!(g.state.pending, Pending::Targets { .. })
            && g.state.announcing.as_ref().is_some_and(|a| {
                a.item.source.is_some_and(|s| s.id == thorin)
                    && a.item.kind == StackKind::TriggeredAbility { ability: 1 }
            })
        {
            break;
        }
        choose(&mut g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            )
        });
    }
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    assert_eq!(g.object_view(b).attached_to, Some(old_recipient));
    g.move_objects_raw(&[recipient, a, b], Zone::Graveyard);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old_recipient));
    assert!(choices.contains(&old_b));
    assert!(!choices.contains(&old_a));
    for id in [recipient, a, b] {
        assert!(!choices.contains(&g.object_ref(id)));
    }
    act(&mut g, Action::ChooseDamageSource { source: old_b });
    drain(&mut g);
    assert_eq!(
        g.object_view(target).damage,
        3,
        "the actual creature's predeparture equipped power is retained"
    );
}
#[test]
fn source_world_completion_reflexive_mill_uses_only_actual_public_results_and_never_hidden_parent_library_refs()
 {
    const MILL: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::MillThenReflexive {
                amount: 4,
                ability: 0,
            }],
        ),
        triggers: &[event_trigger(TriggerCondition::Reflexive)],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &MILL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let ids: Vec<_> = g.state.players[p.index()]
        .library
        .iter()
        .rev()
        .take(4)
        .copied()
        .collect();
    let old: Vec<_> = ids.iter().map(|&id| g.object_ref(id)).collect();
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, spell, None);
    wait(&mut g, |g| {
        g.state.stack.iter().any(|item| {
            item.source.is_some_and(|s| s.id == spell)
                && matches!(item.kind, StackKind::TriggeredAbility { .. })
        })
    });
    let public: Vec<_> = ids.iter().map(|&id| g.object_ref(id)).collect();
    g.move_objects_raw(&ids, Zone::Exile);
    let choices = source_candidates(&mut g);
    for object in old {
        assert!(!choices.contains(&object));
    }
    for object in public {
        assert!(choices.contains(&object));
        assert!(!choices.contains(&g.object_ref(object.card)));
    }
}
#[test]
fn waiting_source_boundary_paradigm_retains_the_actual_exiled_object_after_later_departure() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    let seminar = put(&mut g, p, "Restoration Seminar", Zone::Hand);
    cast(&mut g, seminar, Some(Target::Card(bear)));
    drain(&mut g);
    assert_eq!(g.object_view(seminar).zone, Zone::Exile);
    assert_eq!(g.observe(p).paradigms.len(), 1);
    let exiled = g.object_ref(seminar);
    g.move_object(seminar, Zone::Graveyard);
    g.move_object(bear, Zone::Hand);
    let choices = source_candidates(&mut g);
    assert!(
        choices.contains(&exiled),
        "the waiting Paradigm clause refers to the actual exiled incarnation"
    );
    assert!(!choices.contains(&g.object_ref(seminar)));
    assert!(!choices.contains(&g.object_ref(bear)));
}
#[test]
fn waiting_source_boundary_unspent_goggles_mana_retains_its_exact_delayed_trigger_source() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let goggles = put(&mut g, p, "Pyromancer's Goggles", Zone::Battlefield);
    let old = g.object_ref(goggles);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::AnnounceMana{card,..}if*card==goggles))
        .unwrap();
    act(&mut g, action);
    pay(&mut g);
    assert!(
        g.mana_lot_views()
            .iter()
            .any(|lot| lot.copy_source.is_some_and(|s| s.object == old)
                && lot.amounts.iter().any(|&n| n > 0))
    );
    g.move_object(goggles, Zone::Graveyard);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old));
    assert!(!choices.contains(&g.object_ref(goggles)));
}
#[test]
fn waiting_source_boundary_generous_plunderer_refers_to_the_actual_created_treasure_after_mana_sacrifice()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let plunderer = put(&mut g, p, "Generous Plunderer", Zone::Battlefield);
    g.begin_step(Step::Upkeep);
    wait(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    act(&mut g, Action::ChooseOptional { accept: true });
    for _ in 0..64 {
        if g.state.stack.iter().any(|item| {
            item.source.is_some_and(|s| s.id == plunderer)
                && item.kind == StackKind::TriggeredAbility { ability: 1 }
        }) {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
                || matches!(a,Action::ChooseTarget{target:Target::Player(player)}if*player==p.other())
        });
    }
    let treasure = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
        .map(|(i, _)| i as CardId)
        .unwrap();
    let old = g.object_ref(treasure);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::AnnounceMana{card,..}if*card==treasure))
        .unwrap();
    act(&mut g, action);
    pay(&mut g);
    let choices = source_candidates(&mut g);
    assert!(
        choices.contains(&old),
        "the new reflexive condition is the actual creation action"
    );
    assert!(!choices.contains(&g.object_ref(treasure)));
}
#[test]
fn waiting_source_boundary_until_cleanup_combat_prevention_retains_departed_recipient() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let old = g.object_ref(bear);
    let flight = put(&mut g, p, "Fleeting Flight", Zone::Hand);
    cast(&mut g, flight, Some(Target::Card(bear)));
    drain(&mut g);
    g.move_object(bear, Zone::Hand);
    let choices = source_candidates(&mut g);
    assert!(
        choices.contains(&old),
        "the until-EOT prevention still refers to its exact recipient"
    );
    assert!(!choices.contains(&g.object_ref(bear)));
}
#[test]
fn waiting_source_boundary_until_cleanup_target_only_would_die_effect_retains_departed_target() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let dragon = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
    let old = g.object_ref(dragon);
    let bolt = put(&mut g, p, "Obliterating Bolt", Zone::Hand);
    cast(&mut g, bolt, Some(Target::Card(dragon)));
    let source = g.object_ref(bolt);
    drain(&mut g);
    assert!(g.on_battlefield(dragon));
    g.move_object(dragon, Zone::Hand);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old));
    assert!(
        !choices.contains(&source),
        "target-only wording does not refer to the historical creator"
    );
    assert!(!choices.contains(&g.object_ref(dragon)));
    assert!(!choices.contains(&g.object_ref(bolt)));
}
const EARTH_WAIT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::AnimateEarthbendTargets,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::RegisterEarthbendReturn,
        ],
    ),
    ..CardRules::EMPTY
};
#[test]
fn waiting_source_boundary_target_death_callback_remains_waiting_after_a_nontriggering_departure() {
    const WATCH: CardRules = CardRules {
        effect: Effect::Spell(
            CREATURE,
            &[Instruction::DelayTargetEvent {
                event: DelayedTargetEvent::DiesOnce,
                ability: 0,
            }],
        ),
        triggers: &[event_trigger(TriggerCondition::DelayedTargetEvent)],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &WATCH)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let old = g.object_ref(bear);
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(bear)));
    drain(&mut g);
    g.move_object(bear, Zone::Hand);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old));
    assert!(!choices.contains(&g.object_ref(bear)));
}
#[test]
fn waiting_source_boundary_earthbend_remains_waiting_after_nontriggering_departure() {
    let _scope =
        fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &EARTH_WAIT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p, "Forest", Zone::Battlefield);
    let old = g.object_ref(land);
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(land)));
    drain(&mut g);
    assert!(g.is_creature(land));
    g.move_object(land, Zone::Hand);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&old));
    assert!(!choices.contains(&g.object_ref(land)));
}
#[test]
fn earthbend_returns_a_land_that_lost_creature_type() {
    const LAND: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ability(&[Instruction::BecomeSourceLandUntilNextTurn])],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (card("Opt"), &NWF),
        (card("Lightning Bolt"), &EARTH_WAIT),
        (card("Forest"), &LAND),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p, "Forest", Zone::Battlefield);
    let old = g.object_ref(land);
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(land)));
    drain(&mut g);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: land,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut g);
    drain(&mut g);
    assert!(!g.is_creature(land));
    g.move_object(land, Zone::Graveyard);
    let departed = g.object_ref(land);
    let trigger = g
        .state
        .pending_triggers
        .iter()
        .find(|item| item.kind == StackKind::DelayedReturn)
        .expect("the exact land moved from the battlefield to the graveyard");
    assert_eq!(trigger.return_link.as_ref().unwrap().object, departed);
    g.give_priority();
    drain(&mut g);
    assert!(g.on_battlefield(land));
    assert!(g.object_view(land).tapped);
    assert_ne!(g.object_ref(land), old);
    assert_ne!(g.object_ref(land), departed);
}
#[test]
fn waiting_source_boundary_earthbend_returns_under_the_effect_controllers_control_even_for_an_opponents_land()
 {
    let _scope =
        fixture_rules::activate(&[(card("Opt"), &NWF), (card("Lightning Bolt"), &EARTH_WAIT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
    g.record_control(g.object_ref(land), p, None);
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(land)));
    drain(&mut g);
    g.move_object(land, Zone::Graveyard);
    g.give_priority();
    drain(&mut g);
    assert_eq!(
        g.object_view(land).controller,
        p,
        "CR701.66a says under your control, not the physical owner's"
    );
    assert_eq!(g.object_view(land).owner, p.other());
    assert!(g.object_view(land).tapped);
}
#[test]
fn waiting_source_boundary_named_torch_damage_refers_to_both_old_damage_source_and_recipient() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let dragon = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
    let recipient = g.object_ref(dragon);
    let torch = put(&mut g, p, "Torch the Tower", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: torch });
    act(&mut g, Action::ChooseBargain { sacrifice: None });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(dragon),
        },
    );
    pay(&mut g);
    let source = g.object_ref(torch);
    drain(&mut g);
    g.move_object(dragon, Zone::Hand);
    let choices = source_candidates(&mut g);
    assert!(choices.contains(&recipient));
    assert!(choices.contains(&source));
    assert!(!choices.contains(&g.object_ref(torch)));
    assert!(!choices.contains(&g.object_ref(dragon)));
}
#[test]
fn waiting_source_boundary_plain_prevention_and_target_only_exile_do_not_refer_to_their_creators() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    for name in ["Fleeting Flight", "Scorching Dragonfire"] {
        let p = Player::P0;
        let mut g = scenario(p);
        let dragon = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        let recipient = g.object_ref(dragon);
        let spell = put(&mut g, p, name, Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(dragon)));
        let creator = g.object_ref(spell);
        drain(&mut g);
        g.move_object(dragon, Zone::Hand);
        let choices = source_candidates(&mut g);
        assert!(choices.contains(&recipient));
        assert!(
            !choices.contains(&creator),
            "{name} creator is provenance only"
        );
        reject(&mut g, Action::ChooseDamageSource { source: creator });
    }
}

#[test]
fn waiting_source_boundary_mana_delayed_trigger_remains_inert_after_the_actual_lot_is_spent_or_emptied()
 {
    const SPEND: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                ..ActivationCost::ZERO
            },
            ..ability(&[])
        }],
        ..PLAIN
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &SPEND)]);
    for spend in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let safe = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let goggles = put(&mut g, p, "Pyromancer's Goggles", Zone::Battlefield);
        let old = g.object_ref(goggles);
        g.clear_step_mana(false, false);
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::AnnounceMana{card,..} if *card==goggles))
            .unwrap();
        act(&mut g, action);
        pay(&mut g);
        g.move_object(goggles, Zone::Graveyard);
        g.state.players[p.index()].mana[1] = 3; // Ordinary blue, distinct from the actual tagged red lot.
        let choices = source_candidates(&mut g);
        assert!(choices.contains(&old));
        let safe = g.object_ref(safe);
        act(&mut g, Action::ChooseDamageSource { source: safe });
        drain(&mut g);
        if spend {
            g.state.players[p.index()].mana[1] = 0;
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: safe.card,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut g);
            drain(&mut g);
        } else {
            g.clear_step_mana(false, false);
        }
        assert!(g.mana_lot_views().is_empty());
        g.state.players[p.index()].mana[1] = 3;
        let choices = source_candidates(&mut g);
        assert!(
            choices.contains(&old),
            "106.6/603.7a/b: no qualifying trigger and no stated duration"
        );
        assert_eq!(g.observe(p).inert_mana_copy_sources, vec![old]);
        assert_eq!(g.observe(p.other()).inert_mana_copy_sources, vec![old]);
    }
}
#[test]
fn waiting_source_boundary_prevention_and_death_replacement_references_expire_at_cleanup_without_rebinding()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    for name in ["Fleeting Flight", "Scorching Dragonfire"] {
        let p = Player::P0;
        let mut g = scenario(p);
        let safe = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let recipient = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        let old = g.object_ref(recipient);
        let spell = put(&mut g, p, name, Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(recipient)));
        drain(&mut g);
        g.move_object(recipient, Zone::Hand);
        g.move_object(recipient, Zone::Battlefield);
        let current = g.object_ref(recipient);
        assert_ne!(old, current);
        let choices = source_candidates(&mut g);
        assert!(choices.contains(&old));
        assert!(choices.contains(&current));
        let safe = g.object_ref(safe);
        act(&mut g, Action::ChooseDamageSource { source: safe });
        drain(&mut g);
        // Neither old fixed-recipient effect follows the newly entered dragon.
        g.move_object(recipient, Zone::Graveyard);
        assert_eq!(g.object_view(recipient).zone, Zone::Graveyard);
        g.finish_cleanup();
        for player in &mut g.state.players {
            player.mana = [8; 6];
        }
        g.state.step = Step::Main1;
        g.give_priority();
        let choices = source_candidates(&mut g);
        assert!(!choices.contains(&old));
        assert!(!choices.contains(&current));
    }
}
#[test]
fn waiting_source_boundary_paradigm_waiting_reference_is_owned_and_the_copy_keeps_its_existing_lifecycle()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let bear = put(&mut parent, p, "Bear Cub", Zone::Graveyard);
    let seminar = put(&mut parent, p, "Restoration Seminar", Zone::Hand);
    cast(&mut parent, seminar, Some(Target::Card(bear)));
    drain(&mut parent);
    let exiled = parent.object_ref(seminar);
    parent.move_object(seminar, Zone::Hand);
    parent.move_object(bear, Zone::Graveyard);
    let choices = source_candidates(&mut parent);
    assert!(choices.contains(&exiled));
    assert!(
        !choices.contains(&parent.object_ref(bear)),
        "the initiating target was cleared from the recurring delayed template"
    );
    let action = Action::ChooseDamageSource { source: exiled };
    let saved = parent.snapshot();
    let revision = parent.revision();
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
    indexed.apply_index(p, revision, index).unwrap();
    for g in [&mut raw, &mut indexed] {
        drain(g);
        g.begin_step(Step::Main1);
        wait(
            g,
            |g| matches!(g.state.pending, Pending::ResolutionCasts{player} if player==p),
        );
        assert!(
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::CastExiledDuringResolution { .. })),
            "the original card may leave exile without removing the recurring copy offer"
        );
        act(g, Action::FinishResolutionCasts);
        drain(g);
        assert_eq!(g.observe(p).paradigms.len(), 1);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    drain(&mut indexed);
    indexed.begin_step(Step::Main1);
    wait(
        &mut indexed,
        |g| matches!(g.state.pending,Pending::ResolutionCasts{player}if player==p),
    );
    act(&mut indexed, Action::FinishResolutionCasts);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn waiting_source_boundary_generous_plunderer_replacement_group_is_exact_and_decline_has_no_reflexive()
 {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    for accept in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let plunderer = put(&mut g, p, "Generous Plunderer", Zone::Battlefield);
        let elspeth = put(&mut g, p, "Elspeth, Storm Slayer", Zone::Battlefield);
        g.add_counter(elspeth, CounterKind::Loyalty, 5);
        g.begin_step(Step::Upkeep);
        wait(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        act(&mut g, Action::ChooseOptional { accept });
        for _ in 0..64 {
            if g.state.stack.iter().any(|item| {
                item.source.is_some_and(|s| s.id == plunderer)
                    && item.kind == StackKind::TriggeredAbility { ability: 1 }
            }) || idle(&g)
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
                ) || matches!(a,Action::ChooseTarget{target:Target::Player(player)}if *player==p.other())
            });
        }
        let created: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
            .map(|(id, _)| g.object_ref(id as CardId))
            .collect();
        assert_eq!(created.len(), if accept { 2 } else { 0 });
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|item| item.source.is_some_and(|s| s.id == plunderer)
                    && item.kind == StackKind::TriggeredAbility { ability: 1 })
                .count(),
            usize::from(accept)
        );
        if accept {
            let ids: Vec<_> = created.iter().map(|r| r.card).collect();
            g.move_objects_raw(&ids, Zone::Graveyard);
            let choices = source_candidates(&mut g);
            for object in created {
                assert!(choices.contains(&object));
                assert!(!choices.contains(&g.object_ref(object.card)));
            }
        }
    }
}

#[test]
fn waiting_source_boundary_paradigm_never_fabricates_an_exile_reference_when_the_move_is_prevented()
{
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        }
    }
    let _reset = Reset;
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    let seminar = put(&mut g, p, "Restoration Seminar", Zone::Hand);
    cast(&mut g, seminar, Some(Target::Card(bear)));
    let packet = g.state.stack.last().unwrap().clone();
    let original = g.object_ref(seminar);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(Some(original)));
    g.resolve_paradigm(&packet, 0);
    assert_eq!(g.object_ref(seminar), original);
    assert_eq!(g.object_view(seminar).zone, Zone::Stack);
    let waiting: Vec<_> = g.state.paradigm.as_ref().unwrap().packets().collect();
    assert_eq!(waiting.len(), 1);
    assert!(waiting[0].targets.is_empty());
    assert!(
        waiting[0].event_object.is_none(),
        "no successful public result exists to refer to"
    );
    assert!(waiting[0].entering_object.is_none());
    assert!(waiting[0].return_link.is_none());
}

fn final_boundary_goggles_root_after_mana_disappears(spend: bool) {
    const SPEND: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                ..ActivationCost::ZERO
            },
            ..ability(&[])
        }],
        ..PLAIN
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF), (card("Bear Cub"), &SPEND)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let spender = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let goggles = put(&mut g, p, "Pyromancer's Goggles", Zone::Battlefield);
    let old = g.object_ref(goggles);
    g.clear_step_mana(false, false);
    choose(
        &mut g,
        |a| matches!(a,Action::AnnounceMana{card,..}if *card==goggles),
    );
    pay(&mut g);
    g.move_object(goggles, Zone::Graveyard);
    if spend {
        act(
            &mut g,
            Action::AnnounceActivate {
                card: spender,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        drain(&mut g);
    } else {
        g.clear_step_mana(false, false);
    }
    assert!(g.mana_lot_views().is_empty());
    g.state.players[p.index()].mana[1] = 3;
    let choices = source_candidates(&mut g);
    assert!(
        choices.contains(&old),
        "106.6 creates a DTA with no stated duration; an impossible event does not end it under603.7a/b"
    );
    assert!(!choices.contains(&g.object_ref(goggles)));
}
#[test]
fn semantic_final_boundary_goggles_mana_loss_keeps_the_untriggered_delayed_ability_source() {
    final_boundary_goggles_root_after_mana_disappears(false);
}
#[test]
fn semantic_final_boundary_goggles_nonqualifying_payment_keeps_the_untriggered_delayed_ability_source()
 {
    final_boundary_goggles_root_after_mana_disappears(true);
}
fn final_boundary_native_graveyard_departure_to_battlefield(observer: &str) {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    // A reachable post-ETB state: the observer's separate entry ability is finished.
    let source = put(&mut g, p, observer, Zone::Battlefield);
    let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    let old = g.object_ref(bear);
    let zombify = put(&mut g, p, "Zombify", Zone::Hand);
    cast(&mut g, zombify, Some(Target::Card(bear)));
    wait(&mut g, |g| {
        g.state
            .stack
            .iter()
            .any(|item| item.source.is_some_and(|s| s.id == source))
    });
    let entered = g.object_ref(bear);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    g.move_object(bear, Zone::Hand);
    let choices = source_candidates(&mut g);
    assert!(
        choices.contains(&old),
        "the leave-GY condition explicitly refers to the departing old graveyard object"
    );
    assert!(
        choices.contains(&entered),
        "400.7e also permits finding the resulting public object"
    );
    assert!(!choices.contains(&g.object_ref(bear)));
}
#[test]
fn semantic_final_boundary_native_dredgers_insight_keeps_old_graveyard_and_new_battlefield_subjects()
 {
    final_boundary_native_graveyard_departure_to_battlefield("Dredger's Insight");
}
#[test]
fn semantic_final_boundary_native_along_the_crooked_way_keeps_old_graveyard_and_new_battlefield_subjects()
 {
    final_boundary_native_graveyard_departure_to_battlefield("Along the Crooked Way");
}
#[test]
fn semantic_final_boundary_inert_mana_source_is_exact_public_owned_and_not_generic_provenance() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let goggles = put(&mut parent, p, "Pyromancer's Goggles", Zone::Battlefield);
    let old = parent.object_ref(goggles);
    parent.clear_step_mana(false, false);
    choose(
        &mut parent,
        |a| matches!(a,Action::AnnounceMana{card,..}if *card==goggles),
    );
    pay(&mut parent);
    parent.move_object(goggles, Zone::Hand);
    parent.clear_step_mana(false, false);
    parent.move_object(goggles, Zone::Battlefield);
    parent.move_object(goggles, Zone::Hand);
    let fresh = parent.object_ref(goggles);
    let forest = put(&mut parent, p, "Forest", Zone::Battlefield);
    let ordinary = parent.object_ref(forest);
    act(&mut parent, Action::TapMana { card: forest });
    parent.move_object(forest, Zone::Hand);
    parent.clear_step_mana(false, false);
    parent.finish_cleanup();
    // The inert DTA has no duration, so cleanup/new turn does not remove it.
    parent.state.active = p;
    parent.state.priority = p;
    parent.state.step = Step::Main1;
    parent.give_priority();
    parent.state.players[p.index()].mana = [8; 6];
    let choices = source_candidates(&mut parent);
    assert!(choices.contains(&old));
    assert!(!choices.contains(&fresh));
    assert!(!choices.contains(&ordinary));
    for viewer in [p, p.other()] {
        assert_eq!(parent.observe(viewer).inert_mana_copy_sources, vec![old]);
    }
    reject(&mut parent, Action::ChooseDamageSource { source: fresh });
    let action = Action::ChooseDamageSource { source: old };
    let saved = parent.snapshot();
    let revision = parent.revision();
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
    indexed.apply_index(p, revision, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
const AMPLIFIED_GOGGLES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ManaSpentForSpell,
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCastSpell {
            prior_spells: false,
        }],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        ..ability(&[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed([0, 0, 0, 3, 0, 0]),
            quality: &ManaQuality {
                restriction: ManaRestriction::Unrestricted,
                spell_bonus: ManaSpellBonus::CopyRedInstantSorcery,
                retention: ManaRetention::Step,
            },
        })])
    }],
    ..CardRules::EMPTY
};
#[test]
fn semantic_final_boundary_each_qualifying_mana_unit_triggers_once_and_leaves_no_inert_duplicate() {
    // Complete typed amplification isolates106.6a's three-mana result without
    // introducing a new ordinary mana-doubling card definition.
    let _scope = fixture_rules::activate(&[
        (card("Opt"), &NWF),
        (card("Pyromancer's Goggles"), &AMPLIFIED_GOGGLES),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let goggles = put(&mut g, p, "Pyromancer's Goggles", Zone::Battlefield);
    let old = g.object_ref(goggles);
    let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    g.clear_step_mana(false, false);
    choose(
        &mut g,
        |a| matches!(a,Action::AnnounceMana{card,..}if *card==goggles),
    );
    pay(&mut g);
    g.move_object(goggles, Zone::Graveyard);
    let spell = put(&mut g, p, "Fiery Annihilation", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(bear)));
    let original = g
        .state
        .stack
        .iter()
        .find(|item| item.kind == StackKind::Spell)
        .unwrap()
        .id;
    stack_event_observers(&mut g);
    let triggers: Vec<_> = g
        .state
        .stack
        .iter()
        .filter(|item| item.copy_event.is_some() && item.source.is_some_and(|s| s.object == old))
        .map(|item| item.id)
        .collect();
    assert_eq!(triggers.len(), 3);
    assert!(g.mana_lot_views().is_empty());
    assert!(g.observe(p).inert_mana_copy_sources.is_empty());
    for &id in &triggers[..2] {
        assert!(g.counter_stack_item(id, false));
    }
    assert!(g.state.stack.iter().any(|item| item.id == triggers[2]));
    assert!(g.counter_stack_item(triggers[2], false));
    assert!(g.counter_stack_item(original, false));
    g.state.players[p.index()].mana = [8; 6];
    let choices = source_candidates(&mut g);
    assert!(
        !choices.contains(&old),
        "all three DTAs already triggered; countered packets create no new waiting ability"
    );
}
#[test]
fn semantic_final_boundary_mixed_ineligible_and_qualifying_units_keep_only_one_inert_exact_source()
{
    const SPEND: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                ..ActivationCost::ZERO
            },
            ..ability(&[])
        }],
        ..PLAIN
    };
    let _scope = fixture_rules::activate(&[
        (card("Opt"), &NWF),
        (card("Pyromancer's Goggles"), &AMPLIFIED_GOGGLES),
        (card("Bear Cub"), &SPEND),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let spender = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let goggles = put(&mut g, p, "Pyromancer's Goggles", Zone::Battlefield);
    let old = g.object_ref(goggles);
    g.clear_step_mana(false, false);
    choose(
        &mut g,
        |a| matches!(a,Action::AnnounceMana{card,..}if *card==goggles),
    );
    pay(&mut g);
    g.move_object(goggles, Zone::Graveyard);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: spender,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut g);
    drain(&mut g);
    let spell = put(&mut g, p, "Lightning Bolt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Player(p.other())));
    stack_event_observers(&mut g);
    let triggers: Vec<_> = g
        .state
        .stack
        .iter()
        .filter(|item| item.copy_event.is_some() && item.source.is_some_and(|s| s.object == old))
        .map(|item| item.id)
        .collect();
    assert_eq!(triggers.len(), 1);
    for id in triggers {
        assert!(g.counter_stack_item(id, false));
    }
    let original = g
        .state
        .stack
        .iter()
        .find(|item| item.kind == StackKind::Spell)
        .unwrap()
        .id;
    assert!(g.counter_stack_item(original, false));
    g.clear_step_mana(false, false);
    assert_eq!(
        g.observe(p).inert_mana_copy_sources,
        vec![old],
        "one ineligible unit plus one lost unit are existentially deduplicated; the qualifying unit was consumed"
    );
    g.state.players[p.index()].mana = [8; 6];
    assert!(source_candidates(&mut g).contains(&old));
}
#[test]
fn semantic_final_boundary_ordinary_reanimation_etb_does_not_inherit_the_old_graveyard_object() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &NWF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let observer = put(&mut g, p, "Lifecreed Duo", Zone::Battlefield);
    let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    let old = g.object_ref(bear);
    let spell = put(&mut g, p, "Zombify", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(bear)));
    wait(&mut g, |g| {
        g.state
            .stack
            .iter()
            .any(|item| item.source.is_some_and(|s| s.id == observer))
    });
    let entered = g.object_ref(bear);
    g.move_object(bear, Zone::Hand);
    let choices = source_candidates(&mut g);
    assert!(
        !choices.contains(&old),
        "the ETB condition names the new entrant, unlike a graveyard-departure observer"
    );
    assert!(choices.contains(&entered));
    assert!(!choices.contains(&g.object_ref(bear)));
}
