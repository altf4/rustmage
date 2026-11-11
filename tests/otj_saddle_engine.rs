//! Exact Saddle cost history and designation across owned games.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const MOUNT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::BecomeSaddled { minimum: 2 }],
        sorcery: true,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::If {
                guard: StepGuard::SourceSaddled,
                skip: 2,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
            Instruction::Skip(1),
            Instruction::LoseLife(PlayerSet::Controller, 4),
        ],
    }],
    ..CardRules::EMPTY
};
const COUNTER_ABILITY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
const DESIGNATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::MarkSaddled(Subject::Targets)],
    ),
    ..CardRules::EMPTY
};

fn start_saddle(g: &mut Game, mount: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: mount,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    assert!(matches!(g.state.pending, Pending::Saddle { .. }));
}

fn choose_rider(g: &mut Game, object: ObjectRef) {
    act(g, Action::ChooseSaddler { object });
}

fn saddle(g: &mut Game, mount: CardId, rider: ObjectRef) {
    start_saddle(g, mount);
    choose_rider(g, rider);
    act(g, Action::FinishSaddle);
}

#[test]
fn paid_saddle_records_exact_riders_even_if_countered_and_designation_is_separate() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MOUNT),
        (Card::Counterspell, &COUNTER_ABILITY),
        (Card::GiantGrowth, &DESIGNATE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let mount = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let first = put(&mut g, player, "Goblin Piker", Zone::Battlefield);
        let second = put(&mut g, player, "Goblin Piker", Zone::Battlefield);
        let exact_mount = g.object_ref(mount);
        let first_rider = g.object_ref(first);
        let second_rider = g.object_ref(second);

        start_saddle(&mut g, mount);
        reject(&mut g, Action::FinishSaddle);
        choose_rider(&mut g, first_rider);
        assert!(g.saddle_contributors(exact_mount).is_empty());
        act(&mut g, Action::FinishSaddle);
        assert_eq!(g.saddle_contributors(exact_mount), &[first_rider]);
        assert!(g.object_view(first).tapped);
        assert!(!g.object_view(mount).saddled);

        let activation = g.state.stack.last().unwrap().id;
        act(&mut g, Action::Pass);
        let counter = put(&mut g, player.other(), "Counterspell", Zone::Hand);
        cast(&mut g, counter, Some(Target::Stack(activation)));
        settle(&mut g);
        assert!(!g.object_view(mount).saddled);
        assert_eq!(g.saddle_contributors(exact_mount), &[first_rider]);

        // A direct designation can resolve without a rider. A later payment
        // on an already saddled mount still records its own contributor.
        let designate = put(&mut g, player, "Giant Growth", Zone::Hand);
        cast(&mut g, designate, Some(Target::Card(mount)));
        settle(&mut g);
        assert!(g.object_view(mount).saddled);
        assert_eq!(g.saddle_contributors(exact_mount), &[first_rider]);

        saddle(&mut g, mount, second_rider);
        settle(&mut g);
        assert_eq!(
            g.saddle_contributors(exact_mount),
            &[first_rider, second_rider]
        );

        // An untapped rider can pay again, without being counted as a new rider.
        g.state.objects[first as usize].tapped = false;
        saddle(&mut g, mount, first_rider);
        settle(&mut g);
        assert_eq!(
            g.saddle_contributors(exact_mount),
            &[first_rider, second_rider]
        );

        g.finish_cleanup();
        assert!(!g.source_saddled(exact_mount));
        assert!(g.saddle_contributors(exact_mount).is_empty());
    }
}

#[test]
fn saddle_payment_branches_restore_and_keep_departed_incarnation_links() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &MOUNT)]);
    let player = Player::P0;
    let mut root = scenario(player);
    let mount = put(&mut root, player, "Grizzly Bears", Zone::Battlefield);
    let first = put(&mut root, player, "Goblin Piker", Zone::Battlefield);
    let second = put(&mut root, player, "Goblin Piker", Zone::Battlefield);
    let mount_ref = root.object_ref(mount);
    let first_ref = root.object_ref(first);
    let second_ref = root.object_ref(second);
    start_saddle(&mut root, mount);

    let saved = root.snapshot();
    let mut raw = root.clone();
    let mut indexed = root.clone();
    let mut restored = root.clone();
    drop(root);
    assert!(raw.observe(player.other()).saddle_choice.is_none());

    choose_rider(&mut raw, first_ref);
    reject(&mut raw, Action::ChooseSaddler { object: first_ref });
    act(&mut raw, Action::FinishSaddle);
    settle(&mut raw);
    assert_eq!(raw.saddle_contributors(mount_ref), &[first_ref]);

    let other_action = Action::ChooseSaddler { object: second_ref };
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == other_action)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut indexed, Action::FinishSaddle);
    settle(&mut indexed);
    assert_eq!(indexed.saddle_contributors(mount_ref), &[second_ref]);
    assert!(!indexed.object_view(first).tapped);

    restored.restore(&saved).unwrap();
    choose_rider(&mut restored, first_ref);
    act(&mut restored, Action::FinishSaddle);
    settle(&mut restored);
    assert_eq!(restored.snapshot(), raw.snapshot());

    raw.move_object(mount, Zone::Hand);
    raw.move_object(mount, Zone::Battlefield);
    raw.move_object(first, Zone::Hand);
    raw.move_object(first, Zone::Battlefield);
    assert_ne!(raw.object_ref(first), first_ref);
    assert_ne!(raw.object_ref(mount), mount_ref);
    assert!(raw.source_saddled(mount_ref));
    assert_eq!(raw.saddle_contributors(mount_ref), &[first_ref]);
    assert!(!raw.object_view(mount).saddled);
    assert!(raw.saddle_contributors(raw.object_ref(mount)).is_empty());
}

#[test]
fn resolution_reads_live_saddle_or_the_departed_sources_designation() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &MOUNT),
        (Card::GiantGrowth, &DESIGNATE),
    ]);
    for (saddled_before_attack, remove_mount) in [(true, true), (false, true), (false, false)] {
        let player = Player::P0;
        let mut g = scenario(player);
        let mount = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let rider = put(&mut g, player, "Goblin Piker", Zone::Battlefield);
        let rider_ref = g.object_ref(rider);
        if saddled_before_attack {
            saddle(&mut g, mount, rider_ref);
            settle(&mut g);
        }
        for _ in 0..4 {
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.state.pending, Pending::Attackers);
        act(&mut g, Action::Attack { card: mount });
        act(&mut g, Action::FinishAttackers);
        assert!(!g.state.stack.is_empty());

        let life = g.state.players[player.index()].life;
        if remove_mount {
            g.move_object(mount, Zone::Hand);
            g.move_object(mount, Zone::Battlefield);
            assert!(!g.object_view(mount).saddled);
        } else {
            let designate = put(&mut g, player, "Giant Growth", Zone::Hand);
            cast(&mut g, designate, Some(Target::Card(mount)));
        }
        settle(&mut g);
        let change = if saddled_before_attack || !remove_mount {
            3
        } else {
            -4
        };
        assert_eq!(g.state.players[player.index()].life, life + change);
    }
}

const RIDER_COPY: Instruction =
    Instruction::ChooseSaddleRiders(SaddleRiderEffect::CopyNonlegendaryAttacking);
const COPY_RIDERS: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksWhileSaddled,
        targets: TargetPlan::NONE,
        program: &[RIDER_COPY, RIDER_COPY],
    }],
    ..MOUNT
};
const BLINK_RIDER: CardRules = CardRules {
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacksWhileSaddled,
            targets: TargetPlan::NONE,
            program: &[Instruction::ScheduleStepThisTurn {
                step: TriggerStep::EndCombat,
                relation: CastRelation::Any,
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseSaddleRiders(
                SaddleRiderEffect::BlinkSourceAndUpToOne,
            )],
        },
    ],
    ..MOUNT
};
const RETURN_RIDERS: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksWhileSaddled,
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseSaddleRiders(
            SaddleRiderEffect::ReturnAnyToHand,
        )],
    }],
    ..MOUNT
};
const SACRIFICE_RIDER: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseSaddleRiders(SaddleRiderEffect::SacrificeOneForPower),
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::SacrificedPower),
            Instruction::PutHandLands {
                maximum: AmountValue::SacrificedPower,
                tapped: true,
            },
        ],
    }],
    ..MOUNT
};
const TARGET_RIDER: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksWhileSaddled,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::SaddledSourceThisTurn,
        })),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..MOUNT
};
const COUNT_CREW: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &rustmage_core::cards::authored::CardExtras {
        vehicle_stats: Some(Stats {
            power: 6,
            toughness: 5,
        }),
        ..rustmage_core::cards::authored::CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        program: &[Instruction::Crew { minimum: 1 }],
        once_per_turn: true,
        sorcery: false,
        ..MOUNT.activated[0]
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensScaled {
            token: Card(5302),
            amount: AmountValue::CrewContributorCount,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};

fn pass_until(g: &mut Game, reached: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if reached(g) {
            return;
        }
        match g.state.pending {
            Pending::Priority => act(g, Action::Pass),
            Pending::Blockers => act(g, Action::FinishBlockers),
            Pending::OrderTriggers { .. } => choose(g, |a| matches!(a, Action::PutTrigger { .. })),
            ref other => panic!("unexpected phase before checkpoint: {other:?}"),
        }
    }
    panic!("combat checkpoint exceeded 64 actions");
}

fn attack(g: &mut Game, mount: CardId) {
    pass_until(g, |g| g.state.pending == Pending::Attackers);
    act(g, Action::Attack { card: mount });
    act(g, Action::FinishAttackers);
}

fn saddle_two(g: &mut Game, mount: CardId, first: ObjectRef, second: ObjectRef) {
    start_saddle(g, mount);
    choose_rider(g, first);
    choose_rider(g, second);
    act(g, Action::FinishSaddle);
    settle(g);
}

#[test]
fn repeated_rider_copies_choose_separate_defenders_and_expire_as_exact_tokens() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &COPY_RIDERS)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let mount = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let rider = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let rider_ref = g.object_ref(rider);
    let outsider = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let walker = put(&mut g, p.other(), "Oko, the Ringleader", Zone::Battlefield);
    g.set_counters(walker, Counters::of(CounterKind::Loyalty, 6));
    let walker_ref = g.object_ref(walker);
    saddle(&mut g, mount, rider_ref);
    settle(&mut g);
    attack(&mut g, mount);
    pass_until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });

    let outsider_ref = g.object_ref(outsider);
    reject(
        &mut g,
        Action::ChooseEffectObject {
            object: outsider_ref,
        },
    );
    reject(&mut g, Action::FinishEffectChoice);
    act(&mut g, Action::ChooseEffectObject { object: rider_ref });
    act(
        &mut g,
        Action::ChooseEntryAttackingDefender {
            defender: CombatDefender::Planeswalker(walker_ref),
        },
    );
    // The process starts over with a new choice, including the same rider.
    act(&mut g, Action::ChooseEffectObject { object: rider_ref });
    act(
        &mut g,
        Action::ChooseEntryAttackingDefender {
            defender: CombatDefender::Player(p.other()),
        },
    );
    let copies: Vec<_> = g
        .state
        .attackers
        .iter()
        .filter(|attack| attack.card != mount)
        .cloned()
        .collect();
    assert_eq!(copies.len(), 2);
    assert_eq!(
        copies[0].defender,
        Some(CombatDefender::Planeswalker(walker_ref))
    );
    assert_eq!(copies[1].defender, Some(CombatDefender::Player(p.other())));
    for copy in &copies {
        let token = g.object_view(copy.card);
        assert_eq!(token.card, Card::GoblinPiker);
        assert!(token.is_token() && token.tapped);
    }
    assert!(!g.state.attackers.iter().any(|attack| attack.card == rider));

    pass_until(&mut g, |g| g.state.pending == Pending::Blockers);
    act(&mut g, Action::FinishBlockers);
    pass_until(&mut g, |g| {
        g.state.step == Step::End
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
    });
    assert!(copies.iter().all(|copy| !g.on_battlefield(copy.card)));
    assert_eq!(g.object_ref(rider), rider_ref);
    assert!(g.on_battlefield(rider));
}

#[test]
fn delayed_blink_uses_the_departed_mounts_riders_and_returns_to_their_owners() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &BLINK_RIDER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let mount = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let rider = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
    let rider_ref = g.object_ref(rider);
    g.change_controller(rider_ref, p);
    saddle(&mut g, mount, rider_ref);
    settle(&mut g);
    attack(&mut g, mount);
    settle(&mut g);
    // The delayed packet must use the departed mount's exact history.
    g.move_object(mount, Zone::Hand);
    g.move_object(mount, Zone::Battlefield);
    let new_mount = g.object_ref(mount);
    pass_until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    assert_eq!(g.state.step, Step::EndCombat);
    let mut decline = g.clone();
    act(&mut decline, Action::FinishEffectChoice);
    assert_eq!(decline.object_ref(rider), rider_ref);

    act(&mut g, Action::ChooseEffectObject { object: rider_ref });
    settle(&mut g);
    assert_eq!(g.object_ref(mount), new_mount);
    assert_ne!(g.object_ref(rider), rider_ref);
    let returned = g.object_view(rider);
    assert_eq!(returned.controller, p.other());
    assert_eq!(returned.zone, Zone::Battlefield);
}

#[test]
fn any_number_rider_return_accepts_a_subset_without_targeting() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RETURN_RIDERS)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let mount = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let first = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let second = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let first_ref = g.object_ref(first);
    let second_ref = g.object_ref(second);
    saddle_two(&mut g, mount, first_ref, second_ref);
    attack(&mut g, mount);
    assert!(g.state.stack.last().unwrap().targets.is_empty());
    pass_until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    act(&mut g, Action::ChooseEffectObject { object: second_ref });
    reject(&mut g, Action::ChooseEffectObject { object: second_ref });
    act(&mut g, Action::FinishEffectChoice);
    assert_eq!(g.object_view(second).zone, Zone::Hand);
    assert_eq!(g.object_ref(first), first_ref);
}

#[test]
fn the_paid_rider_target_does_not_follow_a_new_incarnation() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TARGET_RIDER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let mount = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let rider = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let other = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let rider_ref = g.object_ref(rider);
    saddle(&mut g, mount, rider_ref);
    settle(&mut g);
    attack(&mut g, mount);
    assert!(!g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(other)
    }));
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(rider),
        },
    );

    let mut moved = g.clone();
    moved.move_object(rider, Zone::Hand);
    moved.move_object(rider, Zone::Battlefield);
    settle(&mut moved);
    assert_eq!(moved.counter_count(rider, CounterKind::PlusOne), 0);

    g.move_object(mount, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.counter_count(rider, CounterKind::PlusOne), 1);
}

#[test]
fn rider_sacrifice_uses_last_power_and_private_hand_only_tapped_land_choices() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SACRIFICE_RIDER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let mount = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let rider = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let rider_ref = g.object_ref(rider);
    let land = put(&mut g, p, "Plains", Zone::Hand);
    let land_ref = g.object_ref(land);
    let graveyard = put(&mut g, p, "Forest", Zone::Graveyard);
    let graveyard_ref = g.object_ref(graveyard);
    saddle(&mut g, mount, rider_ref);
    settle(&mut g);
    attack(&mut g, mount);
    pass_until(&mut g, |g| g.state.pending == Pending::Blockers);
    act(&mut g, Action::FinishBlockers);
    pass_until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });

    let mut decline = g.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert!(decline.on_battlefield(rider));
    assert_eq!(decline.hand_count(p), 1);

    act(&mut g, Action::ChooseEffectObject { object: rider_ref });
    assert_eq!(g.object_view(rider).zone, Zone::Graveyard);
    assert_eq!(g.hand_count(p), 3);
    assert!(!g.legal_actions().contains(&Action::SelectLand {
        object: graveyard_ref
    }));
    assert!(
        g.observe(p.other())
            .effect_choice
            .unwrap()
            .candidates
            .is_empty()
    );
    act(&mut g, Action::SelectLand { object: land_ref });
    assert!(g.observe(p.other()).effect_choice.unwrap().selected[p.index()].is_empty());
    act(&mut g, Action::FinishLandSelection);
    assert!(g.on_battlefield(land));
    assert!(g.object_view(land).tapped);
    assert_eq!(g.object_ref(graveyard), graveyard_ref);
}

#[test]
fn crew_count_includes_departed_exact_creatures_and_crew_can_be_limited() {
    let vehicle = Card::from_name("Broadcast Rambler").unwrap();
    let _rules = fixture_rules::activate(&[(vehicle, &COUNT_CREW)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let vehicle = put(&mut g, p, "Broadcast Rambler", Zone::Battlefield);
    let first = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let second = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let crew = Action::AnnounceActivate {
        card: vehicle,
        ability: 0,
        counter_payment: Counters::default(),
    };
    act(&mut g, crew.clone());
    for creature in [first, second] {
        let object = g.object_ref(creature);
        act(&mut g, Action::ChooseCrewer { object });
    }
    act(&mut g, Action::FinishCrew);
    settle(&mut g);
    reject(&mut g, crew);
    attack(&mut g, vehicle);
    g.move_object(vehicle, Zone::Graveyard);
    g.move_object(first, Zone::Hand);
    settle(&mut g);

    let treasures = g
        .state
        .objects
        .iter()
        .filter(|object| object.zone == Zone::Battlefield && object.card == Card(5302))
        .count();
    assert_eq!(treasures, 2);
}
