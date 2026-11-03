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
        g.object_ref(permanent)
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
    assert_eq!(g.observe(p).chosen_source_shields.unwrap()[0].chosen, old);
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
fn source_prevention_uses_only_cold_state_and_preserves_hot_object_and_state_layout() {
    assert_eq!(std::mem::size_of::<Object>(), 160);
    // The integrated pre-prevention baseline includes the u128 Endure pending
    // alignment (4976 bytes), unlike this mechanic's isolated 4960-byte base.
    // The prevention state remains behind the existing cold RoleState pointer.
    assert_eq!(std::mem::size_of::<State>(), 4976);
    if let Ok(path) = std::env::var("RUSTMAGE_TDM_WORLD_LAYOUT") {
        let layout = serde_json::json!({
            "Object":std::mem::size_of::<Object>(),"ObjectView":std::mem::size_of::<ObjectView>(),
            "State":std::mem::size_of::<State>(),"DamageSource":std::mem::size_of::<DamageSource>(),
            "RoleState":std::mem::size_of::<roles::RoleState>(),
            "SourcePreventionState":std::mem::size_of::<source_prevention::SourcePreventionState>(),
            "DamagePreventionKind":std::mem::size_of::<DamagePreventionKind>(),
            "DamagePreventionView":std::mem::size_of::<DamagePreventionView>()
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
    assert_eq!(g.observe(p).chosen_source_shields.unwrap()[0].chosen, old);
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
