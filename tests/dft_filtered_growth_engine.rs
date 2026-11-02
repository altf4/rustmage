//! Generic signed group growth; fixtures preserve physical catalogue metadata.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
const GROUP: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: -1,
    kind: 1,
    exclude: 16,
    subtype: None,
    basic: false,
    other: true,
    cards_only: false,
};
const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    activated: &[ActivatedAbility {
        exhaust: true,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            colored: [0, 1, 1, 0, 0, 0],
            mana_component: true,
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileGraveyardRange { minimum: 0 },
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 16,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowFilteredScaled {
            filter: GROUP,
            amount: AmountValue::PaidAdditionalCostSelectionCount,
            power: -1,
            toughness: -1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn start(g: &mut Game, source: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
fn select(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::ChooseAdditionalCost { object });
}
fn finish(g: &mut Game) {
    act(g, Action::FinishAdditionalCostGroup);
    pay(g);
}
fn resolve(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("bounded filtered growth resolution");
}
#[test]
fn signed_filter_excludes_artifact_creatures_and_exact_source_for_both_seats_at_zero_and_two() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    for p in [Player::P0, Player::P1] {
        for x in [0, 2] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let own = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let artifact = put(&mut g, p, "Gleaming Barrier", Zone::Battlefield);
            let fuel = [
                put(&mut g, p, "Swiftfoot Boots", Zone::Graveyard),
                put(&mut g, p, "Gleaming Barrier", Zone::Graveyard),
            ];
            start(&mut g, source);
            for id in fuel.into_iter().take(x) {
                select(&mut g, id);
            }
            finish(&mut g);
            resolve(&mut g);
            let x = x as i128;
            assert_eq!(stats(&g, source), (3, 2));
            assert_eq!(stats(&g, own), (3 - x, 3 - x));
            assert_eq!(stats(&g, enemy), (4 - x, 4 - x));
            assert_eq!(stats(&g, artifact), (0, 4));
        }
    }
}
struct ResetPrevention;
impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}
#[test]
fn declared_nonmana_x_survives_a_prevented_exile_and_bad_cost_choices_are_transactional() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    let _reset = ResetPrevention;
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let fuel = [
        put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Graveyard),
        put(&mut g, Player::P0, "Gleaming Barrier", Zone::Graveyard),
    ];
    let foreign = put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Graveyard);
    let nonartifact = put(&mut g, Player::P0, "Opt", Zone::Graveyard);
    start(&mut g, source);
    for id in [foreign, nonartifact] {
        let object = g.object_ref(id);
        reject(&mut g, Action::ChooseAdditionalCost { object });
    }
    select(&mut g, fuel[0]);
    let duplicate = g.object_ref(fuel[0]);
    reject(&mut g, Action::ChooseAdditionalCost { object: duplicate });
    select(&mut g, fuel[1]);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(fuel[0]))));
    g.state.zone_rules.get_or_insert_default();
    finish(&mut g);
    let paid = g
        .state
        .stack
        .last()
        .unwrap()
        .paid_additional_cost
        .as_ref()
        .unwrap();
    assert_eq!((paid.selected, paid.count), (2, 1));
    resolve(&mut g);
    assert_eq!(stats(&g, victim), (1, 1));
    assert_eq!(g.state.objects[fuel[0] as usize].zone, Zone::Graveyard);
}
#[test]
fn affected_membership_stays_frozen_through_later_type_changes_and_new_entries() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let changed = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut g, Player::P0, "Gleaming Barrier", Zone::Battlefield);
    let fuel = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Graveyard);
    start(&mut g, source);
    select(&mut g, fuel);
    finish(&mut g);
    resolve(&mut g);
    g.apply_temporary_copy(
        g.object_ref(changed),
        CopyRecipe::new(Card::from_name("Gleaming Barrier").unwrap()),
        CopyDeadline::EndOfTurn,
    );
    g.apply_temporary_copy(
        g.object_ref(artifact),
        CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
        CopyDeadline::EndOfTurn,
    );
    assert_eq!(stats(&g, changed), (-1, 3));
    assert_eq!(stats(&g, artifact), (3, 3));
    let later = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    assert_eq!(stats(&g, later), (3, 3));
    g.finish_cleanup();
    assert_eq!(stats(&g, changed), (3, 3));
    assert_eq!(stats(&g, artifact), (0, 4));
}
#[test]
fn a_source_new_incarnation_is_other_than_the_independent_resolving_source() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let fuel = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Graveyard);
    start(&mut g, source);
    select(&mut g, fuel);
    finish(&mut g);
    let old = g.object_ref(source);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    assert_ne!(old, g.object_ref(source));
    resolve(&mut g);
    assert_eq!(stats(&g, source), (2, 1));
}
#[test]
fn one_scalar_is_sampled_before_modifying_its_source_and_all_recipients() {
    const BASED: CardRules = CardRules {
        effect: Effect::Creature(2, 4),
        activated: &[ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::GrowFilteredScaled {
                filter: CountFilter {
                    other: false,
                    exclude: 0,
                    ..GROUP
                },
                amount: AmountValue::SourcePower,
                power: 1,
                toughness: 0,
            }],
            sorcery: false,
            once_per_turn: false,
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &BASED)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    start(&mut g, source);
    pay(&mut g);
    resolve(&mut g);
    assert_eq!(stats(&g, source), (4, 4));
    assert_eq!(stats(&g, other), (5, 3));
}
#[test]
fn owned_counter_cost_selection_and_group_resolution_match_raw_index_and_restore() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    let mut parent = scenario(Player::P0);
    let source = put(&mut parent, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut parent, Player::P1, "Hill Giant", Zone::Battlefield);
    let fuel = put(&mut parent, Player::P0, "Swiftfoot Boots", Zone::Graveyard);
    start(&mut parent, source);
    let saved = parent.snapshot();
    let action = Action::ChooseAdditionalCost {
        object: parent.object_ref(fuel),
    };
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    finish(&mut raw);
    finish(&mut indexed);
    resolve(&mut raw);
    resolve(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(stats(&raw, victim), (2, 2));
    indexed.restore(&saved).unwrap();
    finish(&mut indexed);
    resolve(&mut indexed);
    assert_eq!(stats(&indexed, victim), (3, 3));
}

#[test]
fn signed_growth_registers_toughness_and_named_counter_lki_before_source_sacrifice() {
    const TOUGHNESS: CardRules = CardRules {
        effect: Effect::Creature(2, 3),
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
            program: &[Instruction::GrowFilteredScaled {
                filter: GROUP,
                amount: AmountValue::SourceToughness,
                power: 1,
                toughness: 0,
            }],
            sorcery: false,
            once_per_turn: false,
        }],
        ..CardRules::EMPTY
    };
    const NAMED: CardRules = CardRules {
        activated: &[ActivatedAbility {
            program: &[Instruction::GrowFilteredScaled {
                filter: GROUP,
                amount: AmountValue::SourceCounterQuantity(CounterKind::Charge),
                power: 1,
                toughness: 0,
            }],
            ..TOUGHNESS.activated[0]
        }],
        ..TOUGHNESS
    };
    for (rules, expected) in [(&TOUGHNESS, 8), (&NAMED, 6)] {
        let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, rules)]);
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let recipient = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        g.add_counter(source, CounterKind::PlusOne, 2);
        g.add_counter(source, CounterKind::Charge, 3);
        start(&mut g, source);
        pay(&mut g);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        resolve(&mut g);
        assert_eq!(stats(&g, recipient), (expected, 3));
    }
}
