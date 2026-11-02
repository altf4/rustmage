//! Exact declaration groups and live opponent-player combat keyword grants.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const DEFENDER_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreaturesAttackYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowDeclaredAttackers {
            power: -1,
            toughness: 0,
            keywords: Keywords::NONE,
        }],
    }],
    ..CardRules::EMPTY
};
const TRAMPLE_PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::OpponentPlayerAttackerKeywords(
            Keywords::TRAMPLE,
        )],
    }],
    ..CardRules::EMPTY
};
const SACRIFICE_PROVIDER: CardRules = CardRules {
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 1, 0,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..TRAMPLE_PROVIDER
};
const CONTROL_MANA_AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Control,
        fragments: &[StaticAbilityFragment::ControlAttached],
    }],
    activated: SACRIFICE_PROVIDER.activated,
    ..CardRules::EMPTY
};
const ABILITY_COUNTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};

fn battle(g: &mut Game, owner: Player) -> CardId {
    // A non-Siege Battle is protected by its own controller.
    let id = put(g, owner.other(), "Goblin Piker", Zone::Battlefield);
    let mut recipe = CopyRecipe::new(Card::GoblinPiker);
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(BATTLE_KIND),
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(g.object_ref(id), recipe, CopyDeadline::EndOfTurn);
    g.add_counter(id, CounterKind::Defense, 9);
    g.set_battle_protector(g.object_ref(id), owner.other());
    id
}
fn declare(g: &mut Game, actions: &[Action]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for action in actions {
        act(g, action.clone());
    }
    act(g, Action::FinishAttackers);
}
fn packets(g: &Game, source: CardId) -> Vec<&SpellView> {
    g.state
        .stack
        .iter()
        .chain(g.state.pending_triggers.iter())
        .chain(g.state.ordering_triggers.iter())
        .filter(|item| {
            item.source.is_some_and(|s| s.id == source)
                && matches!(item.kind, StackKind::TriggeredAbility { .. })
        })
        .collect()
}
fn goal(p: Player) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
fn trample<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> bool {
    world.activation_keywords(id).contains(Keywords::TRAMPLE)
}

#[test]
fn defender_group_captures_one_player_only_declaration_and_exact_nontargeted_attackers() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &DEFENDER_OBSERVER)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let a = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let b = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let at_walker = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let at_battle = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.add_counter(walker, CounterKind::Loyalty, 4);
        let siege = battle(&mut g, p);
        g.state.objects[a as usize].gained_keywords = Keywords::HEXPROOF;
        let walker_ref = g.object_ref(walker);
        let battle_ref = g.object_ref(siege);
        declare(
            &mut g,
            &[
                Action::Attack { card: a },
                Action::Attack { card: b },
                Action::AttackPlaneswalker {
                    card: at_walker,
                    defender: walker_ref,
                },
                Action::AttackBattle {
                    card: at_battle,
                    defender: battle_ref,
                },
            ],
        );
        let captured = packets(&g, source);
        assert_eq!(
            captured.len(),
            1,
            "one event for both creatures attacking the player"
        );
        assert!(captured[0].targets.is_empty());
        let subjects = captured[0].reflexive_subject.as_ref().unwrap();
        assert_eq!(subjects.object, g.object_ref(a));
        assert_eq!(subjects.additional_objects, vec![g.object_ref(b)]);
        g.move_object(source, Zone::Graveyard);
        g.move_object(b, Zone::Exile);
        g.move_object(b, Zone::Battlefield);
        let later = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        // A later object already attacking was not part of the declaration event.
        g.state.attackers.push(AttackDeclaration {
            card: later,
            defender: Some(CombatDefender::Player(p.other())),
            blocked: false,
        });
        settle(&mut g);
        assert_eq!(stats(&g, a), (1, 1));
        for id in [b, at_walker, at_battle, later] {
            assert_eq!(stats(&g, id), (2, 1));
        }
        g.finish_cleanup();
        assert_eq!(stats(&g, a), (2, 1));
    }
}

#[test]
fn walker_and_battle_attacks_alone_do_not_create_a_player_attack_group() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &DEFENDER_OBSERVER)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let walker = put(
        &mut g,
        Player::P1,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    g.add_counter(walker, CounterKind::Loyalty, 4);
    let siege = battle(&mut g, Player::P0);
    let a = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let b = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let walker_ref = g.object_ref(walker);
    let battle_ref = g.object_ref(siege);
    declare(
        &mut g,
        &[
            Action::AttackPlaneswalker {
                card: a,
                defender: walker_ref,
            },
            Action::AttackBattle {
                card: b,
                defender: battle_ref,
            },
        ],
    );
    assert!(packets(&g, source).is_empty());
    assert_eq!(stats(&g, a), (2, 1));
    assert_eq!(stats(&g, b), (2, 1));
}

#[test]
fn declared_group_is_owned_through_raw_indexed_and_restored_resolution() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &DEFENDER_OBSERVER)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let a = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let b = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    declare(
        &mut g,
        &[Action::Attack { card: a }, Action::Attack { card: b }],
    );
    g.move_object(source, Zone::Exile);
    let snapshot = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let mut restored = g.clone();
    restored.restore(&snapshot).unwrap();
    drop(g);
    for (which, branch) in [&mut raw, &mut indexed, &mut restored]
        .into_iter()
        .enumerate()
    {
        let index = branch
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        if which == 1 {
            branch
                .apply_index(branch.actor().unwrap(), branch.revision(), index)
                .unwrap();
        } else {
            act(branch, Action::Pass);
        }
        settle(branch);
        assert_eq!(stats(branch, a), (1, 1));
        assert_eq!(stats(branch, b), (1, 1));
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.snapshot(), restored.snapshot());
}

#[test]
fn countered_group_does_not_change_attackers() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &DEFENDER_OBSERVER),
        (Card::Counterspell, &ABILITY_COUNTER),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let a = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    declare(&mut g, &[Action::Attack { card: a }]);
    let trigger = packets(&g, source)[0].id;
    let counter = put(&mut g, Player::P0, "Counterspell", Zone::Hand);
    cast(&mut g, counter, Some(Target::Stack(trigger)));
    settle(&mut g);
    assert_eq!(stats(&g, a), (2, 1));
}

#[test]
fn opponent_player_keywords_match_defender_in_actual_and_sparse_funding_worlds() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TRAMPLE_PROVIDER)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let attacker = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.add_counter(walker, CounterKind::Loyalty, 4);
        let siege = battle(&mut g, p);
        for (defender, expected) in [
            (Some(CombatDefender::Player(p.other())), true),
            (Some(CombatDefender::Player(p)), false),
            (
                Some(CombatDefender::Planeswalker(g.object_ref(walker))),
                false,
            ),
            (Some(CombatDefender::Battle(g.object_ref(siege))), false),
            (None, false),
        ] {
            g.state.attackers.clear();
            g.state.attackers.push(AttackDeclaration {
                card: attacker,
                defender,
                blocked: false,
            });
            assert_eq!(trample(&g, attacker), expected);
            let world = FundingState::from_game(&g, &goal(p));
            assert_eq!(
                trample(&world, attacker),
                expected,
                "sparse payment query: {defender:?}"
            );
        }
        g.state.attackers[0].defender = Some(CombatDefender::Player(p.other()));
        // Primitive recipient selection is unqualified by controller, as required
        // even when a containing game supplies attackers beyond two-player declarations.
        g.state.objects[attacker as usize].controller = p.other();
        assert!(trample(&g, attacker));
    }
}

#[test]
fn opponent_player_grant_follows_provider_control_and_effective_ability_removal() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TRAMPLE_PROVIDER)]);
    let mut g = scenario(Player::P0);
    let provider = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let attacker = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    declare(&mut g, &[Action::Attack { card: attacker }]);
    assert!(trample(&g, attacker));
    g.record_control(g.object_ref(provider), Player::P1, None);
    assert!(!trample(&g, attacker));
    assert!(!trample(
        &FundingState::from_game(&g, &goal(Player::P0)),
        attacker
    ));
    g.record_control(g.object_ref(provider), Player::P0, None);
    assert!(trample(&g, attacker));
    let aura = put(&mut g, Player::P0, "Stop Cold", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(provider)));
    settle(&mut g);
    assert!(!trample(&g, attacker));
    assert!(!trample(
        &FundingState::from_game(&g, &goal(Player::P0)),
        attacker
    ));
    g.move_object(aura, Zone::Graveyard);
    assert!(trample(&g, attacker));
    g.state.attackers.clear();
    assert!(!trample(&g, attacker));
}

#[test]
fn funding_sacrifice_removes_static_provider_before_nested_payment_queries() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SACRIFICE_PROVIDER)]);
    let mut g = scenario(Player::P0);
    let provider = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let attacker = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    declare(&mut g, &[Action::Attack { card: attacker }]);
    assert!(trample(&g, attacker));
    g.state.players[0].mana = [0; 6];
    let goal = FundingGoal {
        required: [0, 0, 0, 0, 1, 0],
        ..goal(Player::P0)
    };
    let world = FundingState::from_game(&g, &goal);
    let step = world
        .moves(&goal)
        .into_iter()
        .find(|step| world.resources[step.resource].id == provider)
        .unwrap();
    let mut reached = false;
    world.visit_applied(&step, &mut |next| {
        assert!(!trample(next, attacker));
        reached = true;
        false
    });
    assert!(reached);
    let spell = put(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(attacker),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    choose(
        &mut g,
        |a| matches!(a, Action::ActivateMana { card, ability: 0, .. } if *card == provider),
    );
    assert_eq!(g.object_view(provider).zone, Zone::Graveyard);
    assert!(!trample(&g, attacker));
    pay(&mut g);
    settle(&mut g);
    assert_eq!(stats(&g, attacker), (5, 4));
}

#[test]
fn funding_uses_projected_provider_controller_after_control_aura_is_sacrificed() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &TRAMPLE_PROVIDER),
        (Card::from_name("Stop Cold").unwrap(), &CONTROL_MANA_AURA),
    ]);
    let mut g = scenario(Player::P0);
    let provider = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let attacker = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    declare(&mut g, &[Action::Attack { card: attacker }]);
    let aura = put(&mut g, Player::P1, "Stop Cold", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(provider));
    g.refresh_controllers();
    assert_eq!(g.object_view(provider).controller, Player::P1);
    assert!(!trample(&g, attacker));
    g.state.players[1].mana = [0; 6];
    let goal = FundingGoal {
        required: [0, 0, 0, 0, 1, 0],
        ..goal(Player::P1)
    };
    let world = FundingState::from_game(&g, &goal);
    let step = world
        .moves(&goal)
        .into_iter()
        .find(|step| world.resources[step.resource].id == aura)
        .unwrap();
    let mut reached = false;
    world.visit_applied(&step, &mut |next| {
        assert_eq!(next.object(provider).controller, Player::P0);
        assert!(trample(next, attacker));
        reached = true;
        false
    });
    assert!(reached);
    g.state.priority = Player::P1;
    choose(
        &mut g,
        |a| matches!(a, Action::ActivateMana { card, ability: 0, .. } if *card == aura),
    );
    assert_eq!(g.object_view(provider).controller, Player::P0);
    assert!(trample(&g, attacker));
}
