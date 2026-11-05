//! TDM shared characteristic queries; these fixtures do not admit any set cards.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const fn activated(cost: ActivationCost, program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost,
        targets: TargetPlan::NONE,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const MANA: &[Instruction] = &[Instruction::AddMana(ManaProduction::Fixed([
    0, 0, 0, 0, 0, 1,
]))];
const SACRIFICIAL: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[activated(
        ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        MANA,
    )],
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
fn values<W: CharacteristicWorld + ?Sized>(world: &W, source: CardId) -> (i128, i128) {
    (
        world.value(StatValue::CounterBearingCreatures, source),
        world.value(StatValue::DistinctCreatureCounterKinds, source),
    )
}

#[test]
fn counter_queries_count_creatures_and_union_kinds_with_live_control_and_types() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        assert_eq!(values(&g, source), (0, 0));
        let a = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let b = put(&mut g, p, "Savannah Lions", Zone::Battlefield);
        for id in [source, a, b] {
            g.add_counter(id, CounterKind::PlusOne, 1);
        }
        assert_eq!(
            values(&g, source),
            (3, 1),
            "duplicate kinds across three creatures are one kind"
        );
        g.add_counter(a, CounterKind::Flying, 1);
        g.add_counter(b, CounterKind::Page, 2);
        assert_eq!(
            values(&g, source),
            (3, 3),
            "sparse kinds join the same union"
        );
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        g.add_counter(land, CounterKind::Stun, 1);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.add_counter(enemy, CounterKind::Reach, 1);
        let grave = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.add_counter(grave, CounterKind::Vigilance, 1);
        assert_eq!(
            values(&g, source),
            (3, 3),
            "noncreatures, graveyard, and enemy counters excluded"
        );
        g.record_control(g.object_ref(a), p.other(), None);
        assert_eq!(values(&g, source), (2, 2));
        g.pay_counters(b, &Counters::of(CounterKind::Page, 2));
        assert_eq!(values(&g, source), (2, 1));
        g.change_object_form(
            g.object_ref(land),
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((1, 1)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        assert_eq!(
            values(&g, source),
            (3, 2),
            "effective creature type is used"
        );
        let world = FundingState::from_game(&g, &goal(p));
        assert_eq!(values(&world, source), values(&g, source));
    }
}

#[test]
fn virtual_sacrifice_removes_hot_and_sparse_counter_kinds_without_mutating_parent() {
    let _rules = fixture_rules::activate(&[(Card::LlanowarElves, &SACRIFICIAL)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let fuel = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.add_counter(source, CounterKind::PlusOne, 1);
        g.add_counter(fuel, CounterKind::PlusOne, 1);
        g.add_counter(fuel, CounterKind::Soul, 1);
        g.state.players[p.index()].mana = [0; 6];
        let saved = g.snapshot();
        let request = goal(p);
        let world = FundingState::from_game(&g, &request);
        assert_eq!(values(&world, source), (2, 2));
        let step = world
            .moves(&request)
            .into_iter()
            .find(|s| world.resources[s.resource].id == fuel && s.ability == Some(0))
            .unwrap();
        let mut visited = false;
        world.visit_applied(&step, &mut |next| {
            assert_eq!(next.object(fuel).zone, Zone::Graveyard);
            assert_eq!(values(next, source), (1, 1));
            visited = true;
            false
        });
        assert!(visited);
        assert_eq!(g.snapshot(), saved);
        assert_eq!(values(&g, source), (2, 2));
    }
}

const HISTORY_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
        condition: StatValue::SourceHasNotDealtDamage,
        keywords: Keywords::HEXPROOF,
    }],
}];
const HISTORY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: HISTORY_GROUPS,
    extras: &CardExtras {
        static_ability_groups: HISTORY_GROUPS,
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        ..activated(ActivationCost::ZERO, &[Instruction::Damage(1)])
    }],
    ..CardRules::EMPTY
};
const PROTECTED: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::PROTECTION_EVERYTHING,
    ..CardRules::EMPTY
};
fn event(g: &Game, source: CardId, target: Target, amount: i128, combat: bool) -> DamageEvent {
    DamageEvent {
        preventable: true,
        source: g.damage_source(source),
        target,
        amount,
        combat,
    }
}
fn unspent(g: &Game, source: CardId) -> bool {
    g.value(StatValue::SourceHasNotDealtDamage, source) == 1
}

#[test]
fn only_positive_unprevented_damage_marks_exact_incarnation_across_turn_control_and_copy() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &HISTORY),
        (Card::HillGiant, &PROTECTED),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let protected = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        assert!(unspent(&g, source) && g.keywords(source).contains(Keywords::HEXPROOF));
        for amount in [0, -1, 1] {
            let hit = event(&g, source, Target::Card(protected), amount, false);
            g.apply_damage_batch(&[hit]);
            assert!(
                unspent(&g, source),
                "nonpositive or protected damage is not dealt"
            );
        }
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
        settle(&mut g);
        assert!(!unspent(&g, source));
        assert!(!g.keywords(source).contains(Keywords::HEXPROOF));
        assert!(
            !g.state.objects[source as usize].dealt_combat_damage_this_incarnation,
            "noncombat was sufficient"
        );
        let old = g.object_ref(source);
        g.state.active = p.other();
        g.begin_turn();
        assert!(!g.state.objects[source as usize].dealt_damage_this_turn);
        assert!(
            !unspent(&g, source),
            "new turn does not reset incarnation history"
        );
        g.record_control(old, p.other(), None);
        assert!(!unspent(&g, source));
        g.apply_temporary_copy(
            old,
            CopyRecipe::new(Card::HillGiant),
            CopyDeadline::EndOfTurn,
        );
        assert!(
            !unspent(&g, source),
            "copy acquisition is not a new incarnation"
        );
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        assert_ne!(old, g.object_ref(source));
        assert!(unspent(&g, source));
        assert!(g.keywords(source).contains(Keywords::HEXPROOF));
        let hit = event(&g, source, Target::Player(p), 1, true);
        g.apply_damage_batch(&[hit]);
        assert!(!unspent(&g, source));
        assert!(g.state.objects[source as usize].dealt_combat_damage_this_incarnation);
    }
}

#[test]
fn queued_damage_does_not_mark_reentered_source_and_history_branches_restore_independently() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &HISTORY),
        (Card::HillGiant, &PROTECTED),
    ]);
    let mut parent = scenario(Player::P0);
    let source = put(&mut parent, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let stale = put(&mut parent, Player::P1, "Bear Cub", Zone::Battlefield);
    act(
        &mut parent,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut parent,
        Action::ChooseTarget {
            target: Target::Stack(u64::MAX),
        },
    );
    let target = Action::ChooseTarget {
        target: Target::Player(Player::P1),
    };
    let list = parent.legal_actions();
    let index = list.iter().position(|a| *a == target).unwrap();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    let mut reentered = parent.clone();
    act(&mut raw, target);
    indexed
        .apply_index(indexed.actor().unwrap(), indexed.revision(), index)
        .unwrap();
    act(
        &mut sibling,
        Action::ChooseTarget {
            target: Target::Card(stale),
        },
    );
    act(
        &mut reentered,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    drop(parent);
    for g in [&mut raw, &mut indexed, &mut sibling, &mut reentered] {
        pay(g);
    }
    sibling.move_object(stale, Zone::Hand);
    sibling.move_object(stale, Zone::Battlefield);
    reentered.move_object(source, Zone::Hand);
    reentered.move_object(source, Zone::Battlefield);
    for g in [&mut raw, &mut indexed, &mut sibling, &mut reentered] {
        settle(g);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert!(!unspent(&raw, source));
    assert!(unspent(&sibling, source));
    assert!(
        unspent(&reentered, source),
        "old packet cannot mark a new generation"
    );
    indexed.restore(&saved).unwrap();
    assert!(unspent(&indexed, source));
    act(
        &mut indexed,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    pay(&mut indexed);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let world = FundingState::from_game(&raw, &goal(Player::P0));
    assert_eq!(world.value(StatValue::SourceHasNotDealtDamage, source), 0);
}

const TOUGHNESS_SAMPLE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::GreatestCreatureToughness,
        )],
    ),
    ..CardRules::EMPTY
};
#[test]
fn greatest_toughness_samples_live_derived_stats_and_virtual_departures() {
    let _rules = fixture_rules::activate(&[
        (Card::LlanowarElves, &SACRIFICIAL),
        (Card::from_name("Opt").unwrap(), &TOUGHNESS_SAMPLE),
    ]);
    let mut g = scenario(Player::P0);
    assert_eq!(g.greatest_creature_toughness(Player::P0), 0);
    let small = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let tall = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    g.add_counter(tall, CounterKind::PlusOne, 4);
    let enemy = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    g.add_counter(enemy, CounterKind::PlusOne, 20);
    let land = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    g.add_counter(land, CounterKind::PlusOne, 20);
    assert_eq!(g.greatest_creature_toughness(Player::P0), 6);
    g.state.players[0].mana = [0; 6];
    let request = goal(Player::P0);
    let world = FundingState::from_game(&g, &request);
    assert_eq!(world.greatest_creature_toughness(Player::P0), 6);
    let step = world
        .moves(&request)
        .into_iter()
        .find(|s| world.resources[s.resource].id == tall && s.ability == Some(0))
        .unwrap();
    let mut visited = false;
    world.visit_applied(&step, &mut |next| {
        assert_eq!(next.greatest_creature_toughness(Player::P0), 2);
        visited = true;
        false
    });
    assert!(visited);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    // Free printed Opt is not assumed: keep the ordinary blue casting cost.
    g.state.players[0].mana = [0, 1, 0, 0, 0, 0];
    cast(&mut g, spell, None);
    g.pay_counters(tall, &Counters::of(CounterKind::PlusOne, 2));
    settle(&mut g);
    assert_eq!(
        g.state.players[0].life, 24,
        "greatest toughness sampled at resolution after response"
    );
    g.move_object(tall, Zone::Hand);
    g.state.objects[small as usize].growth.toughness = -5;
    assert_eq!(g.greatest_creature_toughness(Player::P0), -3);
    assert_eq!(
        g.amount_value(
            AmountValue::GreatestCreatureToughness,
            g.object_ref(small),
            Player::P0
        ),
        0
    );
}

const DRAGON_GRANT: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::ControlledUntappedSubtypeKeywords {
        subtype: "Dragon",
        keywords: Keywords::HEXPROOF,
    }],
}];
const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: DRAGON_GRANT,
    extras: &CardExtras {
        static_ability_groups: DRAGON_GRANT,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const DRAGON_MANA: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    activated: &[activated(
        ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        MANA,
    )],
    ..CardRules::EMPTY
};
const BLANK: CardRules = CardRules {
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
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::RemoveAttachedAbilities],
    }],
    ..CardRules::EMPTY
};
#[test]
fn untapped_subtype_grants_follow_paid_and_virtual_taps_control_and_provider_suppression() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PROVIDER),
        (Card::from_name("Shivan Dragon").unwrap(), &DRAGON_MANA),
        (Card::from_name("Pacifism").unwrap(), &BLANK),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let dragon = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        assert!(g.keywords(dragon).contains(Keywords::HEXPROOF));
        assert!(!g.keywords(enemy).contains(Keywords::HEXPROOF));
        assert!(!g.keywords(provider).contains(Keywords::HEXPROOF));
        g.state.players[p.index()].mana = [0; 6];
        let request = goal(p);
        let world = FundingState::from_game(&g, &request);
        let step = world
            .moves(&request)
            .into_iter()
            .find(|s| world.resources[s.resource].id == dragon)
            .unwrap();
        let mut visited = false;
        world.visit_applied(&step, &mut |next| {
            assert!(next.object(dragon).tapped);
            assert!(
                !next
                    .keywords(next.resources.iter().find(|r| r.id == dragon).unwrap())
                    .contains(Keywords::HEXPROOF)
            );
            visited = true;
            false
        });
        assert!(visited);
        act(
            &mut g,
            Action::ActivateMana {
                card: dragon,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        assert!(!g.keywords(dragon).contains(Keywords::HEXPROOF));
        g.state.objects[dragon as usize].tapped = false;
        g.record_control(g.object_ref(dragon), p.other(), None);
        assert!(!g.keywords(dragon).contains(Keywords::HEXPROOF));
        g.record_control(g.object_ref(dragon), p, None);
        assert!(g.keywords(dragon).contains(Keywords::HEXPROOF));
        let blank = put(&mut g, p.other(), "Pacifism", Zone::Battlefield);
        g.state.objects[blank as usize].attached_to = Some(g.object_ref(provider));
        assert!(
            !g.keywords(dragon).contains(Keywords::HEXPROOF),
            "blank provider cannot grant"
        );
        let world = FundingState::from_game(&g, &goal(p));
        assert!(
            !world
                .keywords(world.resources.iter().find(|r| r.id == dragon).unwrap())
                .contains(Keywords::HEXPROOF)
        );
    }
}

#[test]
fn query_history_layout_is_measured_without_public_observation_fields() {
    eprintln!(
        "tdm-counter-world layout: Object={} ObjectView={} State={}",
        std::mem::size_of::<Object>(),
        std::mem::size_of::<ObjectView>(),
        std::mem::size_of::<State>()
    );
    if let Ok(path) = std::env::var("RUSTMAGE_TDM_COUNTER_LAYOUT") {
        std::fs::write(
            path,
            format!(
                "Object={} ObjectView={} State={}\n",
                std::mem::size_of::<Object>(),
                std::mem::size_of::<ObjectView>(),
                std::mem::size_of::<State>()
            ),
        )
        .unwrap();
    }
    let g = scenario(Player::P0);
    let observation = serde_json::to_value(g.observe(Player::P0)).unwrap();
    assert!(
        !observation
            .to_string()
            .contains("dealt_damage_this_incarnation")
    );
}
