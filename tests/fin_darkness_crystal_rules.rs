//! Linked opposing-creature death replacements and countered exile entry.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::engine::*;
use support::*;

const REPLACEMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        zone_replacement: Some(
            ZoneReplacementRule::ExileOpposingNontokenCreatureDeathLinked { life: 2 },
        ),
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 2, 0, 0, 0],
            generic: 4,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::LinkedExileCreature {
            ability_origin: None,
        })),
        program: &[Instruction::ReturnExiledTargetTappedWithCounters(&[(
            CounterKind::PlusOne,
            2,
        )])],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn crystal() -> Card {
    Card::from_name("Soul-Guide Lantern").unwrap()
}

fn animate(g: &mut Game, id: CardId) {
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
}

fn linked(g: &Game, source: ObjectRef, card: CardId) -> bool {
    let object = g.object_ref(card);
    g.observe(Player::P0)
        .linked_exile
        .iter()
        .any(|link| link.source == source && link.exiled == object)
}

fn choose_target(id: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(id),
    }
}

#[test]
fn replacement_uses_controller_and_links_only_the_exact_exile() {
    let _rules = fixture_rules::activate(&[(crystal(), &REPLACEMENT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p.other(), "Soul-Guide Lantern", Zone::Battlefield);
    g.state.objects[source as usize].controller = p;
    let provider = g.object_ref(source);
    let stolen = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    g.state.objects[stolen as usize].controller = p.other();
    let friendly = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.state.objects[friendly as usize].controller = p;
    let foreign = put(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
    let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
    animate(&mut g, land);
    let unlinked = put(&mut g, p.other(), "Hill Giant", Zone::Exile);
    let token = g.state.objects.len() as CardId;
    g.create_tokens(p.other(), Card::from_catalogue_id(5324), 1, false);

    for id in [stolen, foreign, land] {
        let moved = g.move_object(id, Zone::Graveyard);
        assert_eq!(moved.destination, Zone::Exile);
        assert!(linked(&g, provider, id));
    }
    for id in [friendly, token] {
        assert_eq!(
            g.move_object(id, Zone::Graveyard).destination,
            Zone::Graveyard
        );
        assert!(!linked(&g, provider, id));
    }
    assert_eq!(g.state.players[p.index()].life, 26);
    assert_eq!(g.state.players[p.index()].life_gain_events_this_turn, 3);
    assert_eq!(g.state.players[p.other().index()].life, 20);

    let activate = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    let mut returned_source = g.clone();
    returned_source.move_object(source, Zone::Exile);
    returned_source.move_object(source, Zone::Battlefield);
    returned_source.state.objects[source as usize].controller = p;
    assert_ne!(returned_source.object_ref(source), provider);
    reject(&mut returned_source, activate.clone());

    act(&mut g, activate);
    reject(&mut g, choose_target(land));
    reject(&mut g, choose_target(unlinked));
    act(&mut g, choose_target(foreign));
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    g.move_object(source, Zone::Hand);
    settle(&mut g);

    let creature = g.object_view(foreign);
    assert_eq!(creature.zone, Zone::Battlefield);
    assert_eq!(creature.owner, p.other());
    assert_eq!(creature.controller, p);
    assert!(creature.tapped);
    assert_eq!(g.counter_amounts(foreign).get(CounterKind::PlusOne), 2);
    assert!(linked(&g, provider, stolen));
    assert!(!linked(&g, provider, foreign));
}

#[test]
fn simultaneous_death_choices_freeze_source_and_replay_each_actual_gain() {
    let _rules = fixture_rules::activate(&[(crystal(), &REPLACEMENT)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Soul-Guide Lantern", Zone::Battlefield);
    animate(&mut g, source);
    let provider = g.object_ref(source);
    let first = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let second = put(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
    put(&mut g, p, "Rest in Peace", Zone::Battlefield);
    let wrath = put(&mut g, p, "Day of Judgment", Zone::Hand);
    cast(&mut g, wrath, None);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);

    let choose_link = Action::ChooseZoneReplacement {
        effect: ZoneReplacementKind::ExileLinked { source: provider },
    };
    let choose_plain = Action::ChooseZoneReplacement {
        effect: ZoneReplacementKind::Exile,
    };
    assert_eq!(g.actor(), Some(p.other()));
    assert!(g.observe(p).zone_replacement_choice.is_none());
    assert_eq!(g.apply(p, choose_link.clone()), Err(Error::WrongActor));
    let saved = g.snapshot();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| a == &choose_link)
        .unwrap();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let mut alternate = g.clone();
    drop(g);
    act(&mut raw, choose_link.clone());
    indexed.apply_index(p.other(), revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for branch in [&mut raw, &mut indexed] {
        assert_eq!(branch.object_view(source).zone, Zone::Battlefield);
        assert_eq!(branch.object_view(first).zone, Zone::Battlefield);
        assert_eq!(branch.state.players[p.index()].life, 20);
        assert_eq!(
            branch
                .observe(p.other())
                .zone_replacement_choice
                .unwrap()
                .prior_public_choices,
            [ZoneReplacementPickView {
                object: branch.object_ref(first),
                player: p.other(),
                effect: ZoneReplacementKind::ExileLinked { source: provider },
            }]
        );
        act(branch, choose_link.clone());
        settle(branch);
        assert_eq!(branch.object_view(source).zone, Zone::Exile);
        assert!(linked(branch, provider, first));
        assert!(linked(branch, provider, second));
        assert_eq!(branch.state.players[p.index()].life, 24);
        assert_eq!(
            branch.state.players[p.index()].life_gain_events_this_turn,
            2
        );
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(p.other(), revision, index),
        Err(Error::StaleRevision)
    );

    act(&mut alternate, choose_plain);
    act(&mut alternate, choose_link);
    settle(&mut alternate);
    assert!(!linked(&alternate, provider, first));
    assert!(linked(&alternate, provider, second));
    assert_eq!(alternate.state.players[p.index()].life, 22);
    assert_eq!(
        alternate.state.players[p.index()].life_gain_events_this_turn,
        1
    );
}

#[test]
fn projected_costs_distinguish_identical_destinations_with_different_life_results() {
    use rustmage_core::engine::ability_world::CharacteristicWorld;
    use rustmage_core::engine::funding::{FundingGoal, FundingState};

    const ADD_LIFE: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LifeGainAdditional(1)],
    }];
    const EXTRA_GAIN: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: ADD_LIFE,
        extras: &CardExtras {
            static_ability_groups: ADD_LIFE,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules =
        fixture_rules::activate(&[(crystal(), &REPLACEMENT), (Card::GrizzlyBears, &EXTRA_GAIN)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p.other(), "Soul-Guide Lantern", Zone::Battlefield);
    let victim = put(&mut g, p, "Forest", Zone::Battlefield);
    animate(&mut g, victim);
    put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
    put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let source = g.object_ref(source);
    let victim = g.object_ref(victim);
    let before = g.snapshot();
    let goal = FundingGoal {
        actor: p,
        required: [0; 6],
        generic: 0,
        // Retain the source even though it has no mana ability of its own.
        reserve: Some((
            source,
            AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
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
            },
        )),
        source: None,
    };
    let world = FundingState::from_game(&g, &goal);
    let mut outcomes = Vec::new();
    world.cost_departures(&[source, victim], Zone::Graveyard, &mut |next| {
        assert_eq!(next.object(source.card).zone, Zone::Exile);
        assert_eq!(next.object(victim.card).zone, Zone::Exile);
        outcomes.push((next.life(p.other()), next.gained_life(p.other())));
        false
    });
    assert_eq!(outcomes, [(20, false), (23, true)]);
    assert_eq!(g.snapshot(), before);
}
