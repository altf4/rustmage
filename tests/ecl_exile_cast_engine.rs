//! Exact exile permissions and current ordered casting continuations.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;
fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn ready(g: &mut Game, test: impl Fn(&Game) -> bool) {
    for _ in 0..100 {
        if test(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::PayCosts { .. }
                    | Action::FinishTargets
                    | Action::ChooseReplacement { .. }
                    | Action::ArrangeLibrary {
                        destination: LibraryDestination::Top,
                        ..
                    }
            )
        });
    }
    panic!("bounded exile continuation");
}
fn drain(g: &mut Game) {
    ready(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn costs(g: &mut Game) {
    for _ in 0..30 {
        if g.state.announcing.is_none() {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::PayCosts { .. }
                    | Action::ChooseCastCost { .. }
                    | Action::BeginOrderedCosts
                    | Action::FinishTargets
            )
        });
    }
    panic!("bounded payment");
}
const BRE: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::ExileNonlandLifeFreeCast]),
    ..CardRules::EMPTY
};
#[test]
fn immediate_nonland_cast_has_strict_life_gain_bound_and_uncast_card_goes_to_hand() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &BRE)]);
    for p in [Player::P0, Player::P1] {
        for gained in [2, 3] {
            for accept in [false, true] {
                let mut g = scenario(p);
                g.state.players[p.index()].life_gained_this_turn = gained;
                let offered = put(&mut g, p, "Bear Cub", Zone::Library);
                let land = put(&mut g, p, "Forest", Zone::Library);
                let spell = put(&mut g, p, "Opt", Zone::Hand);
                cast(&mut g, spell, None);
                ready(&mut g, |g| {
                    matches!(g.state.pending, Pending::ResolutionCasts { .. })
                });
                let action = Action::CastExiledDuringResolution {
                    object: g.object_ref(offered),
                    face: CastFace::Normal,
                };
                assert_eq!(g.legal_actions().contains(&action), gained > 2);
                assert_eq!(g.object_view(land).zone, Zone::Exile);
                if accept && gained > 2 {
                    g.state.players[p.index()].mana = [0; 6];
                    act(&mut g, action);
                    costs(&mut g);
                }
                act(&mut g, Action::FinishResolutionCasts);
                drain(&mut g);
                assert_eq!(
                    g.object_view(offered).zone,
                    if accept && gained > 2 {
                        Zone::Battlefield
                    } else {
                        Zone::Hand
                    }
                );
                if !accept || gained == 2 {
                    assert!(g.state.known_hand.contains(&g.object_ref(offered)));
                }
            }
        }
    }
}
const HARVEST: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileUntilManaValueFreeCast {
            players: PlayerSet::Opponent,
            minimum: 5,
        }],
    ),
    ..CardRules::EMPTY
};
#[test]
fn cumulative_library_exile_grants_exact_free_casts_until_end_without_land_plays() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &HARVEST)]);
    let mut g = scenario(Player::P0);
    let stop = put(&mut g, Player::P1, "Gigantosaurus", Zone::Library);
    let cub = put(&mut g, Player::P1, "Bear Cub", Zone::Library);
    let land = put(&mut g, Player::P1, "Forest", Zone::Library);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    drain(&mut g);
    for id in [stop, cub, land] {
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        assert!(
            g.state
                .exile_permissions
                .iter()
                .any(|p| p.object == g.object_ref(id)
                    && p.without_paying_mana_cost
                    && p.expiry == ExilePlayExpiry::Turn(g.state.turn))
        );
    }
    reject(&mut g, Action::PlayLand { card: land });
    g.state.players[0].mana = [0; 6];
    act(
        &mut g,
        Action::AnnounceExileFree {
            card: cub,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    costs(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(cub).zone, Zone::Battlefield);
    assert_eq!(g.object_view(cub).controller, Player::P0);
}
const HAND: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[Instruction::RevealHandForControlledCast(
            AmountValue::Fixed(2),
        )],
    }],
    ..CardRules::EMPTY
};
fn hand_setup(g: &mut Game) -> (CardId, CardId, CardId) {
    let a = put(g, Player::P1, "Shock", Zone::Hand);
    let b = put(g, Player::P1, "Bear Cub", Zone::Hand);
    put(g, Player::P1, "Hill Giant", Zone::Hand);
    let source = put(g, Player::P0, "Short Bow", Zone::Hand);
    cast(g, source, None);
    ready(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    ready(g, |g| {
        matches!(
            g.state.pending,
            Pending::HandCardChoice { player: Player::P1 }
        )
    });
    (source, a, b)
}
#[test]
fn private_hand_subset_is_owned_then_grants_control_duration_instant_cast_with_any_mana() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &HAND)]);
    let mut parent = scenario(Player::P0);
    let (source, a, b) = hand_setup(&mut parent);
    assert!(parent.hand_choice_view(Player::P0).is_none());
    let object = Some(parent.object_ref(a));
    act(&mut parent, Action::ChooseHandCard { object });
    assert!(!parent.state.known_hand.contains(&parent.object_ref(a)));
    let saved = parent.snapshot();
    let action = Action::ChooseHandCard {
        object: Some(parent.object_ref(b)),
    };
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap();
    let rev = parent.revision();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    raw.apply(Player::P1, action.clone()).unwrap();
    indexed.apply_index(Player::P1, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert!(raw.state.known_hand.contains(&raw.object_ref(a)));
    assert!(raw.hand_choice_view(Player::P0).is_some());
    let object = Some(raw.object_ref(a));
    act(&mut raw, Action::ChooseHandCard { object });
    drain(&mut raw);
    assert!(
        raw.state
            .exile_permissions
            .iter()
            .any(|p| p.source == Some(raw.object_ref(source)) && p.object == raw.object_ref(a))
    );
    let mut broken = raw.clone();
    broken.change_controller(broken.object_ref(source), Player::P1);
    broken.change_controller(broken.object_ref(source), Player::P0);
    assert!(!broken.can_cast_exiled_card(Player::P0, a));
    raw.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    act(&mut raw, Action::AnnounceExile { card: a });
    act(
        &mut raw,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    costs(&mut raw);
    drain(&mut raw);
    assert_eq!(raw.state.players[1].life, 18);
    assert_eq!(raw.object_view(a).zone, Zone::Graveyard);
    indexed.restore(&saved).unwrap();
    assert!(indexed.hand_choice_view(Player::P0).is_none());
}
const COUNTER_LINK: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LinkedExileCreatureCounterCasting(3)],
    }],
    ..CardRules::EMPTY
};
const FILTER: SacrificeFilter = SacrificeFilter {
    kinds: 1,
    keywords: Keywords::NONE,
    source_only: false,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const EXTRA: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: FILTER,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn link(g: &mut Game, source: CardId, exiled: CardId) {
    let row = LinkedExileView {
        source: g.object_ref(source),
        exiled: g.object_ref(exiled),
        ability_origin: g.copy_ability_origin(g.object_ref(source)),
        exiled_turn: g.state.turn,
    };
    g.state
        .linked_choices
        .get_or_insert_default()
        .links
        .push(row);
}
#[test]
fn linked_counter_permission_keeps_mana_and_printed_sacrifice_cost_in_either_payment_order() {
    let _scope = fixture_rules::activate(&[
        (card("Bear Cub"), &COUNTER_LINK),
        (card("Hill Giant"), &EXTRA),
    ]);
    for counters_first in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let donor = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
        g.add_counter(donor, CounterKind::MinusOne, 2);
        g.add_counter(donor, CounterKind::Charge, 1);
        g.state.objects[donor as usize].growth.toughness = 5;
        let exiled = put(&mut g, Player::P0, "Hill Giant", Zone::Exile);
        link(&mut g, source, exiled);
        g.state.players[0].mana = [0; 6];
        let reference = g.object_ref(source);
        let announcement = Action::AnnounceLinkedExileCounterCost {
            card: exiled,
            source: reference,
            flash_surcharge: false,
        };
        reject(&mut g, announcement);
        g.state.players[0].mana = [0, 0, 0, 1, 0, 3];
        act(
            &mut g,
            Action::AnnounceLinkedExileCounterCost {
                card: exiled,
                source: reference,
                flash_surcharge: false,
            },
        );
        for _ in 0..30 {
            if g.state.announcing.is_none() {
                break;
            }
            match g.state.pending {
                Pending::CastCosts { .. } => {
                    let rows = &g
                        .state
                        .announcing
                        .as_ref()
                        .unwrap()
                        .cast_costs
                        .as_ref()
                        .unwrap()
                        .clauses;
                    let wanted = rows.iter().position(|r| {
                        !r.paid
                            && if counters_first {
                                matches!(r.clause, CastCostClause::OriginCounters(_))
                            } else {
                                r.clause == CastCostClause::Additional
                            }
                    });
                    if let Some(index) = wanted {
                        act(
                            &mut g,
                            Action::ChooseCastCost {
                                clause: index as u16,
                            },
                        );
                    } else {
                        choose(&mut g, |a| matches!(a, Action::ChooseCastCost { .. }));
                    }
                }
                Pending::AdditionalCost { .. } => {
                    if g.state
                        .announcing
                        .as_ref()
                        .unwrap()
                        .creature_counter_cost_requirement()
                        .is_some()
                    {
                        choose(
                            &mut g,
                            |a| matches!(a,Action::ChooseCreatureCounterCost{object,..}if object.card==donor),
                        );
                    } else {
                        let object = g.object_ref(source);
                        act(&mut g, Action::ChooseAdditionalCost { object });
                    }
                }
                Pending::PayCosts { .. } => {
                    if g.legal_actions().contains(&Action::BeginOrderedCosts) {
                        act(&mut g, Action::BeginOrderedCosts);
                    } else {
                        pay(&mut g);
                    }
                }
                _ => panic!("linked payment {:?}", g.state.pending),
            }
        }
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(donor, CounterKind::MinusOne), 0);
        assert_eq!(g.counter_count(donor, CounterKind::Charge), 0);
        assert_eq!(g.object_view(exiled).zone, Zone::Battlefield);
    }
}
const FREE_LINK: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LinkedExileFreeCastBySubtypeUnion(&[
            "Bear", "Elf",
        ])],
    }],
    ..CardRules::EMPTY
};
#[test]
fn linked_free_cast_is_same_turn_source_scoped_and_quota_consumes_only_committed_cast() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &FREE_LINK)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let first = put(&mut g, Player::P1, "Llanowar Elves", Zone::Exile);
    let second = put(&mut g, Player::P1, "Llanowar Elves", Zone::Exile);
    link(&mut g, source, first);
    link(&mut g, source, second);
    let source = g.object_ref(source);
    let action = Action::AnnounceLinkedFree {
        card: first,
        source,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    assert!(g.legal_actions().contains(&action));
    let saved = g.snapshot();
    act(&mut g, action);
    assert!(g.observe(Player::P0).linked_free_cast_uses.is_empty());
    costs(&mut g);
    assert_eq!(g.observe(Player::P0).linked_free_cast_uses.len(), 1);
    drain(&mut g);
    reject(
        &mut g,
        Action::AnnounceLinkedFree {
            card: second,
            source,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    g.restore(&saved).unwrap();
    g.state.turn += 1;
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::AnnounceLinkedFree { .. }))
    );
}
const DREAM: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CastFromHand(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::InstantSorcery,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::MarkCastDreamExile],
    }],
    ..CardRules::EMPTY
};
const DREAM_CAST: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::CastOwnedDreamCard]),
    ..CardRules::EMPTY
};
#[test]
fn dream_mark_survives_source_departure_and_attack_window_uses_owned_cards_and_one_cast() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &DREAM), (card("Opt"), &DREAM_CAST)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, spell, Some(Target::Player(Player::P1)));
    ready(&mut g, |g| !g.observe(Player::P0).dream_exiles.is_empty());
    g.move_object(source, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Exile);
    assert_eq!(g.counter_count(spell, CounterKind::Dream), 1);
    let foreign = put(&mut g, Player::P1, "Llanowar Elves", Zone::Exile);
    g.add_counter(foreign, CounterKind::Dream, 1);
    let other = put(&mut g, Player::P0, "Llanowar Elves", Zone::Exile);
    g.add_counter(other, CounterKind::Dream, 1);
    let open = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, open, None);
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
    let object = g.object_ref(foreign);
    reject(
        &mut g,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    let object = g.object_ref(other);
    act(
        &mut g,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    costs(&mut g);
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::CastExiledDuringResolution { .. }))
    );
    act(&mut g, Action::FinishResolutionCasts);
    drain(&mut g);
    assert_eq!(g.object_view(other).zone, Zone::Battlefield);
    assert_eq!(g.counter_count(other, CounterKind::Dream), 0);
}

#[test]
fn dream_competing_adventure_and_omen_replacements_have_owned_distinct_completion() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &DREAM)]);
    for omen in [false, true] {
        let mut parent = scenario(Player::P0);
        let provider = put(&mut parent, Player::P0, "Bear Cub", Zone::Battlefield);
        let name = if omen {
            "Whirlwing Stormbrood // Dynamic Soar"
        } else {
            "Bilbo Baggins, Burglar // Take a Glance"
        };
        let spell = put(&mut parent, Player::P0, name, Zone::Hand);
        if omen {
            act(&mut parent, Action::AnnounceOmen { card: spell });
            act(
                &mut parent,
                Action::ChooseTarget {
                    target: Target::Card(provider),
                },
            );
        } else {
            act(&mut parent, Action::AnnounceAdventure { card: spell });
        }
        ready(&mut parent, |g| {
            matches!(g.state.pending, Pending::ZoneReplacement { .. })
        });
        let other = if omen {
            ZoneReplacementKind::OmenShuffle
        } else {
            ZoneReplacementKind::AdventureExile
        };
        let choice = parent.zone_choice_view(Player::P0).unwrap();
        assert!(choice.choices.contains(&other));
        assert!(choice.choices.contains(&ZoneReplacementKind::DreamExile));
        let action = Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::DreamExile,
        };
        let index = parent
            .legal_actions()
            .iter()
            .position(|a| a == &action)
            .unwrap();
        let revision = parent.revision();
        let saved = parent.snapshot();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut other_branch = parent.clone();
        drop(parent);
        assert_eq!(
            raw.apply(Player::P1, action.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(raw.snapshot(), saved);
        raw.apply(Player::P0, action).unwrap();
        indexed.apply_index(Player::P0, revision, index).unwrap();
        drain(&mut raw);
        drain(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(spell).zone, Zone::Exile);
        assert_eq!(raw.counter_count(spell, CounterKind::Dream), 1);
        other_branch.restore(&saved).unwrap();
        act(
            &mut other_branch,
            Action::ChooseZoneReplacement { effect: other },
        );
        drain(&mut other_branch);
        assert_eq!(other_branch.counter_count(spell, CounterKind::Dream), 0);
        assert_eq!(
            other_branch.object_view(spell).zone,
            if omen { Zone::Library } else { Zone::Exile }
        );
        if !omen {
            assert!(other_branch.can_play_adventure(Player::P0, spell));
            assert!(!raw.can_play_adventure(Player::P0, spell));
        }
    }
}
#[test]
fn dream_mark_never_exiles_a_countered_or_illegal_target_spell() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &DREAM)]);
    for countered in [false, true] {
        let mut g = scenario(Player::P0);
        put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let victim = put(&mut g, Player::P1, "Llanowar Elves", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(victim)));
        ready(&mut g, |g| !g.observe(Player::P0).dream_exiles.is_empty());
        if countered {
            let counter = put(&mut g, Player::P0, "Negate", Zone::Hand);
            cast(&mut g, counter, Some(Target::Spell(spell)));
        } else {
            g.move_object(victim, Zone::Graveyard);
        }
        drain(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(spell, CounterKind::Dream), 0);
        assert!(g.observe(Player::P0).dream_exiles.is_empty());
    }
}
#[test]
fn source_bound_cast_permission_requires_the_exiled_card_to_be_an_instant_or_sorcery() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &HAND)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    let adventure = put(
        &mut g,
        Player::P1,
        "Bilbo Baggins, Burglar // Take a Glance",
        Zone::Exile,
    );
    g.grant_source_bound_exile_cast(g.object_ref(source), Player::P0, g.object_ref(adventure));
    assert!(
        !g.state
            .exile_permissions
            .iter()
            .any(|p| p.object == g.object_ref(adventure))
    );
    reject(&mut g, Action::AnnounceAdventure { card: adventure });
    reject(&mut g, Action::AnnounceExile { card: adventure });
}

fn pay_origin_counters(g: &mut Game, donor: CardId, counter: CounterKind) {
    for _ in 0..30 {
        if g.state.announcing.is_none() {
            return;
        }
        match g.state.pending {
            Pending::PayCosts { .. } => {
                if g.legal_actions().contains(&Action::BeginOrderedCosts) {
                    act(g, Action::BeginOrderedCosts);
                } else {
                    pay(g);
                }
            }
            Pending::CastCosts { .. } => choose(g, |a| matches!(a, Action::ChooseCastCost { .. })),
            Pending::AdditionalCost { .. } => {
                let object = g.object_ref(donor);
                act(g, Action::ChooseCreatureCounterCost { object, counter });
            }
            _ => panic!("origin counter payment {:?}", g.state.pending),
        }
    }
    panic!("bounded origin counter payment");
}
#[test]
fn linked_counter_permission_reserves_counters_against_mana_abilities() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &COUNTER_LINK)]);
    for amount in [7, 8] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let donor = put(
            &mut g,
            Player::P0,
            "Ramos, Dragon Engine",
            Zone::Battlefield,
        );
        g.add_counter(donor, CounterKind::PlusOne, amount);
        let spell = put(&mut g, Player::P0, "Grizzly Bears", Zone::Exile);
        link(&mut g, source, spell);
        g.state.players[0].mana = [0; 6];
        let action = Action::AnnounceLinkedExileCounterCost {
            card: spell,
            source: g.object_ref(source),
            flash_surcharge: false,
        };
        if amount == 7 {
            reject(&mut g, action);
            continue;
        }
        act(&mut g, action);
        choose(
            &mut g,
            |a| matches!(a, Action::ActivateMana { card, .. } if *card == donor),
        );
        assert_eq!(g.counter_count(donor, CounterKind::PlusOne), 3);
        pay_origin_counters(&mut g, donor, CounterKind::PlusOne);
        assert_eq!(g.counter_count(donor, CounterKind::PlusOne), 0);
        drain(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}
#[test]
fn linked_counter_permission_preserves_declared_x_and_requires_existing_counters() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &COUNTER_LINK)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Wildwood Scourge", Zone::Exile);
    link(&mut g, source, spell);
    g.state.players[0].mana = [0, 0, 0, 0, 1, 4];
    let action = Action::AnnounceLinkedExileCounterCost {
        card: spell,
        source: g.object_ref(source),
        flash_surcharge: false,
    };
    reject(&mut g, action.clone());
    g.add_counter(source, CounterKind::Flying, 3);
    act(&mut g, action);
    assert!(g.x_choice().unwrap().contains(4));
    assert!(!g.x_choice().unwrap().contains(5));
    reject(&mut g, Action::ChooseX { value: 5 });
    act(&mut g, Action::ChooseX { value: 4 });
    pay_origin_counters(&mut g, source, CounterKind::Flying);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    drain(&mut g);
    assert_eq!(g.counter_count(spell, CounterKind::PlusOne), 4);
    assert_eq!(g.counter_count(source, CounterKind::Flying), 0);
}

#[test]
fn linked_counter_shortfall_rejects_without_spending_interchangeable_lands() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &COUNTER_LINK)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.add_counter(source, CounterKind::Flying, 2);
    let spell = put(&mut g, Player::P0, "Grizzly Bears", Zone::Exile);
    link(&mut g, source, spell);
    for _ in 0..12 {
        put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    }
    g.state.players[0].mana = [0; 6];
    let source = g.object_ref(source);
    reject(
        &mut g,
        Action::AnnounceLinkedExileCounterCost {
            card: spell,
            source,
            flash_surcharge: false,
        },
    );
}

const COUNTER_MANA_LINK: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 2, 0,
        ]))],
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        activation_put_counters: &[(0, CounterKind::Flying, 1)],
        ..CardExtras::EMPTY
    },
    ..COUNTER_LINK
};
const COST_COUNTER_DOUBLER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(
            0,
            ReplacementSpec::Counters {
                filter: ReplacementFilter::Permanent,
                kind: None,
                factor: 2,
                add: 0,
                effects_only: false,
            },
        )],
    }],
    extras: &CardExtras {
        replacement_effects: &[ReplacementSpec::Counters {
            filter: ReplacementFilter::Permanent,
            kind: None,
            factor: 2,
            add: 0,
            effects_only: false,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const NO_COUNTERS: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(1)],
    }],
    ..CardRules::EMPTY
};
#[test]
fn linked_counter_shortfall_can_be_repaired_by_a_mana_ability() {
    let _scope = fixture_rules::activate(&[
        (card("Bear Cub"), &COUNTER_MANA_LINK),
        (card("Short Bow"), &COST_COUNTER_DOUBLER),
        (card("Goldvein Pick"), &NO_COUNTERS),
    ]);
    for (initial, provider, succeeds) in [
        (2, None, true),
        (1, Some("Short Bow"), true),
        (1, Some("Doubling Season"), false),
        (2, Some("Goldvein Pick"), false),
    ] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        g.add_counter(source, CounterKind::Flying, initial);
        if let Some(provider) = provider {
            put(&mut g, Player::P0, provider, Zone::Battlefield);
        }
        let spell = put(&mut g, Player::P0, "Grizzly Bears", Zone::Exile);
        link(&mut g, source, spell);
        g.state.players[0].mana = [0; 6];
        let origin = g.object_ref(source);
        let cast = Action::AnnounceLinkedExileCounterCost {
            card: spell,
            source: origin,
            flash_surcharge: false,
        };
        if !succeeds {
            reject(&mut g, cast);
            continue;
        }
        act(&mut g, cast);
        choose(
            &mut g,
            |a| matches!(a, Action::ActivateMana { card, .. } if *card == source),
        );
        assert_eq!(g.counter_count(source, CounterKind::Flying), 3);
        pay_origin_counters(&mut g, source, CounterKind::Flying);
        drain(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Flying), 0);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}
