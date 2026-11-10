//! Restricted mana uses the actual spell origin and activation source.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const TWO_BLUE: Mana = [0, 2, 0, 0, 0, 0];
const TWO_WHITE: Mana = [2, 0, 0, 0, 0, 0];
const NONHAND: ManaProduction = ManaProduction::Tagged {
    production: &ManaProduction::Fixed(BLUE),
    quality: &ManaQuality {
        restriction: ManaRestriction::CastFromNonhand,
        spell_bonus: ManaSpellBonus::None,
        retention: ManaRetention::Step,
    },
};
const SPELL_MANA: ManaProduction = ManaProduction::Tagged {
    production: &ManaProduction::Fixed(BLUE),
    quality: &ManaQuality {
        restriction: ManaRestriction::ArtifactSpell,
        spell_bonus: ManaSpellBonus::None,
        retention: ManaRetention::Step,
    },
};
const ACTIVATION_MANA: ManaProduction = ManaProduction::Tagged {
    production: &ManaProduction::Fixed(TWO_BLUE),
    quality: &ManaQuality {
        restriction: ManaRestriction::ArtifactActivation,
        spell_bonus: ManaSpellBonus::None,
        retention: ManaRetention::Step,
    },
};
const ARTIFACT_MANA: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::LookAtLibraryTop,
            StaticAbilityFragment::CastFromLibraryTop(CastFilter::Artifact),
        ],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentTapMana {
            kinds: 16,
            production: NONHAND,
        }],
    },
];
const PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: ARTIFACT_MANA,
    extras: &CardExtras {
        static_ability_groups: ARTIFACT_MANA,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const CREATURE_PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    ..PROVIDER
};
const LIFE_ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        generic: 2,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    sorcery: false,
    once_per_turn: false,
};
const ABILITIES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        LIFE_ABILITY,
        ActivatedAbility {
            origin: ActivationOrigin::Hand,
            ..LIFE_ABILITY
        },
        ActivatedAbility {
            origin: ActivationOrigin::Graveyard,
            ..LIFE_ABILITY
        },
    ],
    ..CardRules::EMPTY
};
const MANA_ABILITY: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    program: &[Instruction::AddMana(SPELL_MANA)],
    ..LIFE_ABILITY
};
const PRODUCER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        MANA_ABILITY,
        ActivatedAbility {
            program: &[Instruction::AddMana(ACTIVATION_MANA)],
            ..MANA_ABILITY
        },
    ],
    ..CardRules::EMPTY
};
const BLANK_ARTIFACT: CardRules = CardRules {
    effect: Effect::Permanent,
    ..CardRules::EMPTY
};
const SACRIFICIAL_SEED: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed(BLUE))],
        ..MANA_ABILITY
    }],
    ..CardRules::EMPTY
};
const RESOLUTION_COST: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    generic: 2,
                    ..ManaCost::ZERO
                },
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 4),
        ],
    ),
    ..CardRules::EMPTY
};
const FOOD_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Type,
    fragments: &[StaticAbilityFragment::ControlledArtifactFood(0)],
}];
const FOOD_PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            sacrifice: true,
            ..LIFE_ABILITY.cost
        },
        ..LIFE_ABILITY
    }],
    static_groups: FOOD_GROUPS,
    extras: &CardExtras {
        static_ability_groups: FOOD_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn mana(card: CardId, ability: u16, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn add_artifact(g: &mut Game, id: CardId) {
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
}

fn reject_resolution_spend(g: &mut Game, p: Player) {
    let opt = put(g, p, "Opt", Zone::Hand);
    g.state.players[p.index()].mana = [2, 3, 0, 0, 0, 0];
    cast(g, opt, None);
    act(g, Action::Pass);
    act(g, Action::Pass);
    assert!(g.legal_actions().contains(&Action::DeclineResolutionMana));
    reject(g, Action::PayResolutionMana { payment: TWO_BLUE });
    act(g, Action::PayResolutionMana { payment: TWO_WHITE });
    assert_eq!(g.state.players[p.index()].life, 24);
    assert_eq!(g.state.players[p.index()].mana, TWO_BLUE);
}

#[test]
fn live_artifact_mana_grant_funds_library_spells_and_only_nonhand_casts() {
    let _rules = fixture_rules::activate(&[
        (named("Swiftfoot Boots"), &PROVIDER),
        (named("Candy Trail"), &ABILITIES),
        (named("Opt"), &RESOLUTION_COST),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let grantor = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
    let artifact = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let hand = put(&mut g, p, "Swiftfoot Boots", Zone::Hand);
    let top = put(&mut g, p, "Swiftfoot Boots", Zone::Library);
    g.state.players[p.index()].mana = [0; 6];
    let exact = g.object_ref(grantor);
    let tap = |card| Action::TapGrantedMana {
        card,
        grantor: exact,
        production: BLUE,
    };
    assert_eq!(
        g.observe(p).known_library_top[p.index()].unwrap().object,
        g.object_ref(top)
    );
    assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
    reject(&mut g, tap(other));
    reject(&mut g, tap(bear));
    let mut animated = g.clone();
    add_artifact(&mut animated, bear);
    animated.state.objects[bear as usize].summoning_sick = true;
    reject(&mut animated, tap(bear));
    animated.state.objects[bear as usize].summoning_sick = false;
    act(&mut animated, tap(bear));
    assert!(animated.object_view(bear).tapped);
    let mut removed = g.clone();
    removed.remove_abilities_exact(&[exact], exact, p, AnimationDuration::EndOfTurn);
    reject(&mut removed, tap(artifact));
    let mut changed_control = g.clone();
    changed_control.change_controller(exact, p.other());
    reject(&mut changed_control, tap(artifact));

    reject(&mut g, Action::AnnounceCast { card: hand });
    reject(&mut g, activation(artifact, 0));
    let mut spent = g.clone();
    act(&mut spent, tap(grantor));
    act(&mut spent, tap(artifact));
    assert_eq!(
        spent.observe(p).mana_lots[0].restriction,
        ManaRestrictionView::CastFromNonhand
    );
    reject(&mut spent, Action::AnnounceCast { card: hand });
    reject(&mut spent, activation(artifact, 0));
    let mut resolution = spent.clone();
    reject_resolution_spend(&mut resolution, p);
    let mut exiled = spent.clone();
    exiled.move_object(top, Zone::Exile);
    exiled.grant_exile_play_permission(p, exiled.object_ref(top), false);
    act(&mut exiled, Action::AnnounceExile { card: top });
    pay(&mut exiled);
    settle(&mut exiled);
    assert_eq!(exiled.object_view(top).zone, Zone::Battlefield);

    act(&mut g, Action::AnnounceCast { card: top });
    act(&mut g, tap(grantor));
    let action = tap(artifact);
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let revision = g.revision();
    let snapshot = g.snapshot();
    let mut indexed = g.clone();
    indexed.apply_index(p, revision, index).unwrap();
    act(&mut g, action);
    assert_eq!(g.snapshot(), indexed.snapshot());
    indexed.restore(&snapshot).unwrap();
    assert_eq!(
        indexed.apply_index(p, revision, index),
        Err(Error::StaleRevision)
    );
    pay(&mut g);
    assert_eq!(
        g.state.stack.last().unwrap().cast.unwrap().origin,
        Zone::Library
    );
    settle(&mut g);
    assert_eq!(g.object_view(top).zone, Zone::Battlefield);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn blank_artifacts_fund_spells_through_live_and_newly_entering_grants() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &CREATURE_PROVIDER),
        (Card::GoblinPiker, &SACRIFICIAL_SEED),
        (named("Candy Trail"), &BLANK_ARTIFACT),
    ]);
    let p = Player::P0;
    for (provider_zone, recipient_zone) in [
        (Zone::Battlefield, Zone::Battlefield),
        (Zone::Exile, Zone::Battlefield),
        (Zone::Battlefield, Zone::Exile),
    ] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Grizzly Bears", provider_zone);
        let recipient = put(&mut g, p, "Candy Trail", recipient_zone);
        let returning = if provider_zone == Zone::Exile {
            Some(provider)
        } else {
            (recipient_zone == Zone::Exile).then_some(recipient)
        };
        let spell = put(&mut g, p, "Swiftfoot Boots", Zone::Exile);
        g.grant_exile_play_permission(p, g.object_ref(spell), false);
        g.state.players[p.index()].mana = [0; 6];
        let announce = Action::AnnounceExile { card: spell };
        let extra_artifact = if let Some(returning) = returning {
            let seed = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
            reject(&mut g, announce.clone());
            g.state.exile_links.push(ExileLink {
                ability_origin: None,
                source: g.object_ref(seed),
                exiled: g.object_ref(returning),
                return_zone: Zone::Battlefield,
            });
            act(&mut g, announce);
            act(&mut g, mana(seed, 0, BLUE));
            assert_eq!(g.object_view(returning).zone, Zone::Battlefield);
            None
        } else {
            let other = put(&mut g, p, "Candy Trail", Zone::Battlefield);
            act(&mut g, announce);
            Some(other)
        };
        for card in std::iter::once(recipient).chain(extra_artifact) {
            let grantor = g.object_ref(provider);
            act(
                &mut g,
                Action::TapGrantedMana {
                    card,
                    grantor,
                    production: BLUE,
                },
            );
            assert!(g.object_view(card).tapped);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn artifact_activation_mana_qualifies_by_source_in_each_activation_zone() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PRODUCER),
        (named("Candy Trail"), &ABILITIES),
        (Card::GoblinPiker, &ABILITIES),
        (named("Opt"), &RESOLUTION_COST),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let producer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let field = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    let hand = put(&mut g, p, "Candy Trail", Zone::Hand);
    let graveyard = put(&mut g, p, "Candy Trail", Zone::Graveyard);
    let other = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let artifact_spell = put(&mut g, p, "Swiftfoot Boots", Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];
    reject(
        &mut g,
        Action::AnnounceCast {
            card: artifact_spell,
        },
    );
    reject(&mut g, activation(other, 0));
    for (card, index) in [(field, 0), (hand, 1), (graveyard, 2)] {
        let mut branch = g.clone();
        act(&mut branch, activation(card, index));
        act(&mut branch, mana(producer, 1, TWO_BLUE));
        assert_eq!(
            branch.observe(p).mana_lots[0].restriction,
            ManaRestrictionView::ArtifactActivation
        );
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.state.players[p.index()].life, 23);
        assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
    }

    let mut projected = g.clone();
    add_artifact(&mut projected, other);
    act(&mut projected, activation(other, 0));
    act(&mut projected, mana(producer, 1, TWO_BLUE));
    pay(&mut projected);
    settle(&mut projected);
    assert_eq!(projected.state.players[p.index()].life, 23);

    let mut activations_only = g.clone();
    act(&mut activations_only, mana(producer, 1, TWO_BLUE));
    reject(&mut activations_only, mana(producer, 0, BLUE));
    reject(
        &mut activations_only,
        Action::AnnounceCast {
            card: artifact_spell,
        },
    );
    reject(&mut activations_only, activation(other, 0));
    reject_resolution_spend(&mut activations_only, p);

    let mut spells_only = g;
    spells_only.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut spells_only, mana(producer, 0, BLUE));
    reject(&mut spells_only, activation(field, 0));
    act(
        &mut spells_only,
        Action::AnnounceCast {
            card: artifact_spell,
        },
    );
    pay(&mut spells_only);
    settle(&mut spells_only);
    assert_eq!(
        spells_only.object_view(artifact_spell).zone,
        Zone::Battlefield
    );
}

#[test]
fn food_subtype_and_ability_follow_live_artifacts_and_layer_order() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &FOOD_PROVIDER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let giver = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let boots = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
    let grantor = g.object_ref(giver);
    let food = |g: &Game, id| Action::AnnounceArtifactFood {
        source: g.object_ref(id),
        grantor,
        ability: 0,
    };
    assert!(g.object_view(boots).has_subtype("Food"));
    assert!(g.object_view(boots).has_subtype("Equipment"));
    assert!(!g.object_view(enemy).has_subtype("Food"));
    assert!(!g.object_view(giver).has_subtype("Food"));
    let giver_food = food(&g, giver);
    reject(&mut g, giver_food.clone());
    reject(&mut g, activation(giver, 0));

    add_artifact(&mut g, giver);
    assert!(g.object_view(giver).has_subtype("Food"));
    assert!(g.object_view(giver).has_subtype("Bear"));
    let mut self_food = g.clone();
    act(&mut self_food, giver_food.clone());
    pay(&mut self_food);
    settle(&mut self_food);
    assert_eq!(self_food.object_view(giver).zone, Zone::Graveyard);
    assert!(!self_food.object_view(boots).has_subtype("Food"));
    assert_eq!(self_food.state.players[p.index()].life, 23);

    let mut removed = g.clone();
    removed.remove_abilities_exact(&[grantor], grantor, p, AnimationDuration::EndOfTurn);
    assert!(removed.object_view(giver).has_subtype("Food"));
    assert!(removed.object_view(boots).has_subtype("Food"));
    reject(&mut removed, giver_food);
    let action = food(&removed, boots);
    let index = removed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let revision = removed.revision();
    let snapshot = removed.snapshot();
    let mut indexed = removed.clone();
    act(&mut removed, action);
    indexed.apply_index(p, revision, index).unwrap();
    for branch in [&mut removed, &mut indexed] {
        pay(branch);
        assert_eq!(branch.state.stack.last().unwrap().source.unwrap().id, boots);
        settle(branch);
        assert_eq!(branch.object_view(boots).zone, Zone::Graveyard);
        assert_eq!(branch.state.players[p.index()].life, 23);
    }
    assert_eq!(removed.snapshot(), indexed.snapshot());
    indexed.restore(&snapshot).unwrap();
    assert_eq!(
        indexed.apply_index(p, revision, index),
        Err(Error::StaleRevision)
    );

    g.change_controller(grantor, p.other());
    let boots_food = food(&g, boots);
    reject(&mut g, boots_food);
    assert!(!g.object_view(boots).has_subtype("Food"));
    assert!(g.object_view(enemy).has_subtype("Food"));
}
