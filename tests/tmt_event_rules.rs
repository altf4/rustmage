//! Shared event and payment proofs for the TMNT engine port.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const THREE_RATS: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::SacrificePermanents(3),
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: Some("Rat"),
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

const REMOVE_ONE: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::RemoveCreatureCounter(CounterKind::PlusOne),
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
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

const LAND_BONUS: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledLandTapManaBonus([
            0, 0, 0, 0, 1, 0,
        ])],
    }],
    ..CardRules::EMPTY
};

const WATCH_DEPARTURE: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentLeaves(DepartureFilter {
            kinds: 1,
            relation: CastRelation::You,
            other: true,
            graveyard_only: true,
            counters: DepartureCounters::Any,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::DeathEventToughness,
        )],
    }],
    ..CardRules::EMPTY
};

const DISAPPEAR: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::PermanentLeftBattlefieldThisTurn,
            event: &TriggerCondition::SelfEnters,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn assert_owned_action(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let revision = g.revision();
    let parent = g.snapshot();
    let mut indexed = g.clone();
    indexed.apply_index(actor, revision, index).unwrap();
    act(g, action.clone());
    assert_eq!(g.snapshot(), indexed.snapshot());
    g.restore(&parent).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(g, action);
    assert_eq!(g.snapshot(), indexed.snapshot());
}

#[test]
fn exact_group_sacrifice_allows_same_names_and_the_tapped_source() {
    let _fixture =
        fixture_rules::activate(&[(Card::from_name("Burglar Rat").unwrap(), &THREE_RATS)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Burglar Rat", Zone::Battlefield);
        let second = put(&mut g, p, "Burglar Rat", Zone::Battlefield);
        let wrong = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Burglar Rat", Zone::Battlefield);
        assert!(!g.legal_actions().contains(&activate(source)));
        let third = put(&mut g, p, "Tunnel Rats", Zone::Battlefield);
        act(&mut g, activate(source));
        for id in [wrong, foreign] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let first = g.object_ref(source);
        assert_owned_action(&mut g, Action::ChooseAdditionalCost { object: first });
        reject(&mut g, Action::ChooseAdditionalCost { object: first });
        for id in [second, third] {
            let object = g.object_ref(id);
            act(&mut g, Action::ChooseAdditionalCost { object });
        }
        pay(&mut g);
        for id in [source, second, third] {
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        }
        for id in [wrong, foreign] {
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        }
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
    }
}

#[test]
fn activation_reserves_and_pays_only_the_specified_creature_counter_kind() {
    let _fixture = fixture_rules::activate(&[(Card::GoblinPiker, &REMOVE_ONE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let creature = put(&mut g, p, "Savannah Lions", Zone::Battlefield);
        g.state.objects[creature as usize].counters.stun = 1;
        assert!(!g.legal_actions().contains(&activate(source)));
        g.state.objects[creature as usize].counters.plus_one = 1;
        act(&mut g, activate(source));
        let object = g.object_ref(creature);
        reject(
            &mut g,
            Action::ChooseCreatureCounterCost {
                object,
                counter: CounterKind::Stun,
            },
        );
        assert_owned_action(
            &mut g,
            Action::ChooseCreatureCounterCost {
                object,
                counter: CounterKind::PlusOne,
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(creature, CounterKind::Stun), 1);
        assert_eq!(g.hand_count(p), 1);
    }
}

#[test]
fn land_tap_bonus_can_fund_a_spell_of_a_different_color() {
    let _fixture = fixture_rules::activate(&[(Card::GoblinPiker, &LAND_BONUS)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Plains", Zone::Battlefield);
        let spell = put(&mut g, p, "Llanowar Elves", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let cast = Action::AnnounceCast { card: spell };
        put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        assert!(!g.legal_actions().contains(&cast));
        put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        act(&mut g, cast);
        assert_owned_action(&mut g, Action::TapMana { card: land });
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 1, 0]);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
    }
}

#[test]
fn simultaneous_departures_keep_pre_event_observer_and_toughness() {
    let _fixture = fixture_rules::activate(&[
        (Card::GoblinPiker, &WATCH_DEPARTURE),
        (Card::from_name("Llanowar Elves").unwrap(), &DISAPPEAR),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let watcher = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let creature = put(&mut g, p, "Savannah Lions", Zone::Battlefield);
        let other = put(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
        g.state.objects[creature as usize].counters.plus_one = 2;
        g.state.objects[other as usize].counters.plus_one = 3;
        let early = put(&mut g, p, "Llanowar Elves", Zone::Hand);
        cast(&mut g, early, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        let wrath = put(&mut g, p, "Day of Judgment", Zone::Hand);
        cast(&mut g, wrath, None);
        settle(&mut g);
        assert_eq!(g.state.objects[watcher as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 23);
        let later = put(&mut g, p, "Llanowar Elves", Zone::Hand);
        cast(&mut g, later, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 24);
    }
}
