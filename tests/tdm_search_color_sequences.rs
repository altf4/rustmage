//! Owned top-exile, optional-discard and private constrained-search continuations.
//! Typed fixtures establish engine behavior without admitting TDM definitions.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
const BREACH: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileUntilNonlandCastOrHand {
                maximum_mana_value: 8,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::PermanentEntersYou {
                kinds: 0,
                nontoken: false,
                subtype: Some("Dragon"),
                include_source: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnSourceToHand(TargetZone::Battlefield)],
        },
    ],
    ..CardRules::EMPTY
};
const GLACIAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::MayDiscardNonlandReflexive(0),
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::Damage(3)],
    }],
    extras: &CardExtras {
        harmonize_cost: Some(ManaCost {
            colored: [0, 1, 0, 1, 0, 0],
            generic: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const LOTUS: CardRules = CardRules {
    effect: Effect::Creature(3, 6),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SearchDistinctColors {
            spec: SearchSpec {
                filter: SearchFilter::Any,
                maximum: 3,
                destination: SearchDestination::Graveyard,
                tapped: false,
                reveal: false,
            },
            colors: (1 << 1) | (1 << 2) | (1 << 4),
        }],
    }],
    ..CardRules::EMPTY
};
const RHYTHM: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchCreaturesUpToChosenX(SearchSpec {
            filter: SearchFilter::Creature,
            maximum: 1,
            destination: SearchDestination::Battlefield,
            tapped: false,
            reveal: false,
        })],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        harmonize_x_symbols: 1,
        harmonize_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 4, 0],
            generic: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn idle(g: &Game) -> bool {
    g.outcome().is_some()
        || (g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty())
}
fn ready(g: &mut Game, pred: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if pred(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::ChooseReplacement { .. }
                    | Action::ChooseCastCost { .. }
            )
        });
    }
    panic!("sequence action ceiling");
}
fn drain(g: &mut Game) {
    ready(g, idle)
}
fn breach(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Short Bow", Zone::Hand);
    announce(g, id, &[]);
    ready(g, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. }) || idle(g)
    });
    id
}
fn take_free(g: &mut Game, id: CardId, face: CastFace) {
    let object = g.object_ref(id);
    act(g, Action::CastExiledDuringResolution { object, face });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    act(g, Action::FinishResolutionCasts);
}
fn glacial(g: &mut Game, p: Player, harmonize: bool) -> CardId {
    let id = put(
        g,
        p,
        "Divination",
        if harmonize {
            Zone::Graveyard
        } else {
            Zone::Hand
        },
    );
    if harmonize {
        act(g, Action::AnnounceHarmonize { card: id });
        act(g, Action::ChooseHarmonizer { object: None });
        pay(g);
    } else {
        announce(g, id, &[]);
    }
    ready(g, |g| {
        matches!(g.state.pending, Pending::HandCardChoice { .. })
    });
    id
}
fn hand_pick(g: &mut Game, id: Option<CardId>) {
    let object = id.map(|id| g.object_ref(id));
    act(g, Action::ChooseHandCard { object });
}
fn choose_creature(g: &mut Game, id: CardId) {
    let action = Action::ChooseTarget {
        target: Target::Card(id),
    };
    ready(g, |g| g.legal_actions().contains(&action));
    act(g, action);
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn lotus(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Grizzly Bears", Zone::Hand);
    announce(g, id, &[]);
    ready(g, |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
    id
}
fn search_pick(g: &mut Game, id: CardId) {
    act(g, Action::ChooseSearch { card: id });
}
fn rhythm(g: &mut Game, p: Player, x: u32, harmonize: Option<CardId>) -> CardId {
    let id = put(
        g,
        p,
        "Pterafractyl",
        if harmonize.is_some() {
            Zone::Graveyard
        } else {
            Zone::Hand
        },
    );
    act(
        g,
        if harmonize.is_some() {
            Action::AnnounceHarmonize { card: id }
        } else {
            Action::AnnounceCast { card: id }
        },
    );
    act(g, Action::ChooseX { value: x });
    if let Some(helper) = harmonize {
        let object = g.object_ref(helper);
        act(
            g,
            Action::ChooseHarmonizer {
                object: Some(object),
            },
        );
    }
    pay(g);
    ready(g, |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
    id
}
#[test]
fn breaching_exiles_sequential_lands_and_decline_puts_only_first_nonland_into_hand() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &BREACH)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let nonland = put(&mut g, p, "Bear Cub", Zone::Library);
        let a = put(&mut g, p, "Forest", Zone::Library);
        let b = put(&mut g, p, "Mountain", Zone::Library);
        breach(&mut g, p);
        for id in [a, b, nonland] {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
        assert!(
            g.legal_actions()
                .contains(&Action::CastExiledDuringResolution {
                    object: g.object_ref(nonland),
                    face: CastFace::Normal
                })
        );
        act(&mut g, Action::FinishResolutionCasts);
        drain(&mut g);
        assert_eq!(g.object_view(nonland).zone, Zone::Hand);
        for id in [a, b] {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
        assert!(g.state.exile_permissions.is_empty());
    }
}
#[test]
fn breaching_casts_eight_free_and_stops_on_nine_without_exiling_a_later_eligible_card() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &BREACH)]);
    for (name, accept) in [("Zetalpa, Primal Dawn", true), ("Bygone Colossus", false)] {
        let p = Player::P0;
        let mut g = scenario(p);
        let later = put(&mut g, p, "Bear Cub", Zone::Library);
        let found = put(&mut g, p, name, Zone::Library);
        breach(&mut g, p);
        assert_eq!(g.object_view(later).zone, Zone::Library);
        if accept {
            g.state.players[p.index()].mana = [0; 6];
            take_free(&mut g, found, CastFace::Normal);
        } else {
            assert!(
                !g.legal_actions()
                    .contains(&Action::CastExiledDuringResolution {
                        object: g.object_ref(found),
                        face: CastFace::Normal
                    })
            );
            act(&mut g, Action::FinishResolutionCasts);
        }
        drain(&mut g);
        assert_eq!(
            g.object_view(found).zone,
            if accept {
                Zone::Battlefield
            } else {
                Zone::Hand
            }
        );
    }
}
#[test]
fn breaching_uses_chosen_spell_face_mana_value_and_does_not_skip_large_front() {
    let _scope = fixture_rules::activate(&[
        (card("Short Bow"), &BREACH),
        (
            card("Bygone Colossus"),
            &CardRules {
                effect: Effect::Creature(9, 9),
                extras: &CardExtras {
                    adventure_spell: Some(Card::Divination),
                    ..CardExtras::EMPTY
                },
                ..CardRules::EMPTY
            },
        ),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let found = put(&mut g, p, "Bygone Colossus", Zone::Library);
    breach(&mut g, p);
    assert!(
        !g.legal_actions()
            .contains(&Action::CastExiledDuringResolution {
                object: g.object_ref(found),
                face: CastFace::Normal
            })
    );
    assert!(
        g.legal_actions()
            .contains(&Action::CastExiledDuringResolution {
                object: g.object_ref(found),
                face: CastFace::Adventure
            })
    );
    act(&mut g, Action::FinishResolutionCasts);
    drain(&mut g);
    assert_eq!(g.object_view(found).zone, Zone::Hand);
}
#[test]
fn breaching_empty_all_land_and_prevented_top_terminate_and_leave_exiles() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(None));
        }
    }
    let _reset = Reset;
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &BREACH)]);
    for prevented in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let top = g.top_library_card(p).unwrap();
        if prevented {
            let object = g.object_ref(top);
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(Some(object)));
        }
        breach(&mut g, p);
        drain(&mut g);
        assert_eq!(
            g.state.players[p.index()].library.len(),
            if prevented { 8 } else { 0 }
        );
        assert!(g.state.resolution_free_casts.is_none());
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(None));
    }
}
#[test]
fn breaching_free_or_hand_offer_is_owned_raw_index_restore_and_source_can_leave() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &BREACH)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let found = put(&mut parent, p, "Bear Cub", Zone::Library);
    let source = breach(&mut parent, p);
    parent.move_object(source, Zone::Exile);
    let action = Action::CastExiledDuringResolution {
        object: parent.object_ref(found),
        face: CastFace::Normal,
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
    act(&mut raw, action.clone());
    indexed.apply_index(p, rev, index).unwrap();
    pay(&mut raw);
    pay(&mut indexed);
    act(&mut raw, Action::FinishResolutionCasts);
    act(&mut indexed, Action::FinishResolutionCasts);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    act(&mut sibling, Action::FinishResolutionCasts);
    drain(&mut sibling);
    assert_eq!(sibling.object_view(found).zone, Zone::Hand);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    pay(&mut indexed);
    act(&mut indexed, Action::FinishResolutionCasts);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn breaching_dragon_entry_return_is_independent_and_does_not_rebind_after_blink() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &BREACH)]);
    for blink in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let found = put(&mut g, p, "Bear Cub", Zone::Library);
        let source = breach(&mut g, p);
        act(&mut g, Action::FinishResolutionCasts);
        drain(&mut g);
        assert_eq!(g.object_view(found).zone, Zone::Hand);
        let old = g.object_ref(source);
        let dragon = put(&mut g, p, "Shivan Dragon", Zone::Hand);
        announce(&mut g, dragon, &[]);
        if blink {
            ready(&mut g, |g| {
                g.state.pending == Pending::Priority
                    && g.state.stack.iter().any(|item| {
                        item.kind == StackKind::TriggeredAbility { ability: 1 }
                            && item.source.is_some_and(|s| s.object == old)
                    })
            });
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Battlefield);
            g.prepare_triggers();
        }
        drain(&mut g);
        assert_eq!(
            g.object_view(source).zone,
            if blink { Zone::Battlefield } else { Zone::Hand }
        );
        assert_ne!(g.object_ref(source), old);
    }
}
#[test]
fn glacial_draws_before_optional_discard_and_only_actual_nonland_queues_targeted_three_damage() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &GLACIAL)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let nonland = put(&mut g, p, "Bear Cub", Zone::Library);
        let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let source = glacial(&mut g, p, false);
        assert_eq!(g.object_view(nonland).zone, Zone::Hand);
        assert!(g.observe(p.other()).hand_choice.is_none());
        assert!(
            !g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(nonland))
        );
        hand_pick(&mut g, Some(nonland));
        choose_creature(&mut g, target);
        drain(&mut g);
        assert_eq!(g.object_view(nonland).zone, Zone::Graveyard);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}
#[test]
fn glacial_land_discard_or_decline_never_requests_a_creature_target() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &GLACIAL)]);
    for discard in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let top = g.top_library_card(p).unwrap();
        glacial(&mut g, p, false);
        hand_pick(&mut g, discard.then_some(top));
        drain(&mut g);
        assert_eq!(
            g.object_view(top).zone,
            if discard { Zone::Graveyard } else { Zone::Hand }
        );
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn glacial_redirected_discard_still_triggers_but_prevented_discard_does_not() {
    const EXILE: CardRules = CardRules {
        effect: Effect::Permanent,
        extras: &CardExtras {
            zone_replacement: Some(ZoneReplacementRule::ExileAllCardsAndTokens),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(None));
        }
    }
    let _reset = Reset;
    let _scope =
        fixture_rules::activate(&[(Card::Divination, &GLACIAL), (card("Short Bow"), &EXILE)]);
    for prevented in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let nonland = put(&mut g, p, "Bear Cub", Zone::Library);
        let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        glacial(&mut g, p, false);
        if prevented {
            let object = g.object_ref(nonland);
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(Some(object)));
        }
        hand_pick(&mut g, Some(nonland));
        if prevented {
            act(
                &mut g,
                Action::ChooseZoneReplacement {
                    effect: zone_replacements::ZoneReplacementKind::Prevent,
                },
            );
        }
        if !prevented {
            choose_creature(&mut g, target);
        }
        drain(&mut g);
        assert_eq!(
            g.object_view(nonland).zone,
            if prevented { Zone::Hand } else { Zone::Exile }
        );
        assert_eq!(
            g.object_view(target).zone,
            if prevented {
                Zone::Battlefield
            } else {
                Zone::Exile
            }
        );
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(None));
    }
}
#[test]
fn harmonized_glacial_is_exiled_before_its_counterable_reflexive_target_is_chosen() {
    const COUNTER: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
            &[Instruction::Counter],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::Divination, &GLACIAL), (card("Opt"), &COUNTER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let nonland = put(&mut g, p, "Bear Cub", Zone::Library);
    let target = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    let source = glacial(&mut g, p, true);
    hand_pick(&mut g, Some(nonland));
    let action = Action::ChooseTarget {
        target: Target::Card(target),
    };
    ready(&mut g, |g| g.legal_actions().contains(&action));
    assert_eq!(g.object_view(source).zone, Zone::Exile);
    act(&mut g, action);
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    let packet = g.state.stack.last().unwrap();
    assert_eq!(packet.kind, StackKind::TriggeredAbility { ability: 0 });
    let trigger_id = packet.id;
    let counter = put(&mut g, p, "Opt", Zone::Hand);
    announce(&mut g, counter, &[Target::Stack(trigger_id)]);
    drain(&mut g);
    assert_eq!(g.object_view(target).damage, 0);
}
#[test]
fn glacial_private_optional_discard_is_owned_and_rejects_wrong_actor_and_stale_hand_card() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &GLACIAL)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let nonland = put(&mut parent, p, "Bear Cub", Zone::Library);
    glacial(&mut parent, p, false);
    let action = Action::ChooseHandCard { object: None };
    let saved = parent.snapshot();
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut stale = parent.clone();
    let old = stale.object_ref(nonland);
    stale.move_object(nonland, Zone::Exile);
    stale.move_object(nonland, Zone::Hand);
    let stale_saved = stale.snapshot();
    assert!(
        stale
            .apply(p, Action::ChooseHandCard { object: Some(old) })
            .is_err()
    );
    assert_eq!(stale.snapshot(), stale_saved);
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, action.clone());
    indexed.apply_index(p, rev, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(nonland).zone, Zone::Hand);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn lotus_private_search_allows_every_subset_and_reassigns_multicolor_roles() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &LOTUS)]);
    for count in 0..=3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let blue = put(&mut g, p, "Divination", Zone::Library);
        let green = put(&mut g, p, "Bear Cub", Zone::Library);
        let black = put(&mut g, p, "Duress", Zone::Library);
        let cards = [blue, green, black];
        lotus(&mut g, p);
        for &id in &cards[..count] {
            search_pick(&mut g, id);
        }
        assert!(g.observe(p.other()).search_choice.is_none());
        assert!(
            cards
                .iter()
                .all(|id| !g.observe(p.other()).objects.iter().any(|o| o.id == *id))
        );
        act(&mut g, Action::FinishSearch);
        drain(&mut g);
        for (i, id) in cards.into_iter().enumerate() {
            assert_eq!(
                g.object_view(id).zone,
                if i < count {
                    Zone::Graveyard
                } else {
                    Zone::Library
                }
            );
        }
    }
}
#[test]
fn lotus_rejects_duplicate_monocolor_role_wrong_owner_and_reuses_multicolor_via_bounded_matching() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &LOTUS)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let black = put(&mut g, p, "Duress", Zone::Library);
    let black2 = put(&mut g, p, "Duress", Zone::Library);
    let multi = put(&mut g, p, "Deadly Brew", Zone::Library);
    let blue = put(&mut g, p, "Divination", Zone::Library);
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Library);
    lotus(&mut g, p);
    search_pick(&mut g, multi);
    search_pick(&mut g, black);
    let saved = g.snapshot();
    for id in [black, black2, enemy] {
        assert!(g.apply(p, Action::ChooseSearch { card: id }).is_err());
        assert_eq!(g.snapshot(), saved);
    }
    search_pick(&mut g, blue);
    act(&mut g, Action::FinishSearch);
    drain(&mut g);
    for id in [multi, black, blue] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn lotus_partial_role_selection_is_owned_raw_index_restore_and_remains_private() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &LOTUS)]);
    let p = Player::P0;
    let mut parent = scenario(p);
    let blue = put(&mut parent, p, "Divination", Zone::Library);
    let green = put(&mut parent, p, "Bear Cub", Zone::Library);
    lotus(&mut parent, p);
    search_pick(&mut parent, blue);
    let action = Action::ChooseSearch { card: green };
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
    act(&mut raw, action.clone());
    indexed.apply_index(p, rev, index).unwrap();
    assert!(raw.observe(p.other()).search_choice.is_none());
    act(&mut raw, Action::FinishSearch);
    act(&mut indexed, Action::FinishSearch);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    act(&mut sibling, Action::FinishSearch);
    drain(&mut sibling);
    assert_eq!(sibling.object_view(green).zone, Zone::Library);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    act(&mut indexed, Action::FinishSearch);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn chosen_x_search_is_creature_mana_value_bound_not_number_of_cards_and_can_fail_to_find() {
    let _scope = fixture_rules::activate(&[(card("Pterafractyl"), &RHYTHM)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Library);
        let high = put(&mut g, p, "Shivan Dragon", Zone::Library);
        let noncreature = put(&mut g, p, "Duress", Zone::Library);
        rhythm(&mut g, p, 2, None);
        let saved = g.snapshot();
        for id in [high, noncreature] {
            assert!(g.apply(p, Action::ChooseSearch { card: id }).is_err());
            assert_eq!(g.snapshot(), saved);
        }
        search_pick(&mut g, bear);
        act(&mut g, Action::FinishSearch);
        drain(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert!(g.observe(p.other()).search_choice.is_none());
    }
    let p = Player::P0;
    let mut g = scenario(p);
    rhythm(&mut g, p, 3, None);
    act(&mut g, Action::FinishSearch);
    drain(&mut g);
}
#[test]
fn harmonize_chosen_x_survives_power_discount_and_zero_x_finds_zero_mana_value_creature() {
    let _scope = fixture_rules::activate(&[(card("Pterafractyl"), &RHYTHM)]);
    for x in [0, 3] {
        let p = Player::P0;
        let mut g = scenario(p);
        let found = put(
            &mut g,
            p,
            if x == 0 {
                "Marketback Walker"
            } else {
                "Hill Giant"
            },
            Zone::Library,
        );
        let helper = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 4, 0];
        let source = rhythm(&mut g, p, x, Some(helper));
        assert!(g.object_view(helper).tapped);
        if x == 0 {
            search_pick(&mut g, found);
        } else {
            assert!(
                !g.legal_actions()
                    .contains(&Action::ChooseSearch { card: found })
            );
        }
        act(&mut g, Action::FinishSearch);
        drain(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert_eq!(
            g.object_view(found).zone,
            if x == 0 {
                Zone::Graveyard
            } else {
                Zone::Library
            }
        );
    }
}

const CALL: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutColorCountersAndWin { subtype: "Dragon" }],
    }],
    ..CardRules::EMPTY
};
const LAND_AMOUNT: AmountValue = AmountValue::Stat(StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
}));
const URENI: CardRules = CardRules {
    effect: Effect::Creature(10, 10),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1 | 64,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: usize::MAX,
        }),
        program: &[Instruction::DamageDividedScaled(LAND_AMOUNT)],
    }],
    ..CardRules::EMPTY
};
const DOUBLE_COUNTER: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::Permanent,
    kind: None,
    factor: 2,
    add: 0,
    effects_only: false,
};
const DOUBLE_COUNTER_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE_COUNTER)],
    }],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE_COUNTER],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn dragon(g: &mut Game, p: Player, colors: u8) -> CardId {
    let id = put(g, p, "Shivan Dragon", Zone::Battlefield);
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    g.state
        .roles
        .as_mut()
        .unwrap()
        .animations
        .last_mut()
        .unwrap()
        .colors = Some(colors);
    id
}
fn call(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Short Bow", Zone::Hand);
    announce(g, id, &[]);
    drain(g);
    g.begin_step(Step::Upkeep);
    g.give_priority();
    ready(g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. }) || idle(g)
    });
    id
}
fn color_pick(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::ChooseColorCounter { object });
}
fn five(g: &mut Game, p: Player) -> Vec<CardId> {
    (0..5).map(|i| dragon(g, p, 1 << i)).collect()
}
fn ureni(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Hill Giant", Zone::Hand);
    announce(g, id, &[]);
    ready(g, |g| {
        g.state
            .announcing
            .as_ref()
            .is_some_and(|a| a.item.announced_division.is_some())
            || g.state.stack.iter().any(|i| {
                matches!(i.kind, StackKind::TriggeredAbility { .. })
                    && i.announced_division.is_some()
            })
    });
    id
}
fn division(g: &mut Game, ids: &[CardId], amounts: &[u64]) {
    for &id in ids {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if matches!(g.state.pending, Pending::Targets { .. }) {
        act(g, Action::FinishTargets);
    }
    for &amount in amounts {
        if matches!(g.state.pending, Pending::TargetAmount { .. }) {
            act(g, Action::AllocateTargetAmount { amount });
        }
    }
}
#[test]
fn one_multicolor_dragon_receives_one_simultaneous_five_counter_batch_without_winning() {
    const WATCH: CardRules = CardRules {
        effect: Effect::Permanent,
        triggers: &[
            TriggeredAbility {
                condition: TriggerCondition::ControlledCreatureCountersPlaced { first_only: false },
                targets: TargetPlan::NONE,
                program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
            },
            TriggeredAbility {
                condition: TriggerCondition::CounterBatchPlacedByYou {
                    kind: CounterKind::PlusOne,
                    filter: CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 1,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: false,
                    },
                },
                targets: TargetPlan::NONE,
                program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
            },
        ],
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(card("Short Bow"), &CALL), (card("Gilded Lotus"), &WATCH)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = dragon(&mut g, p, 31);
        put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        call(&mut g, p);
        for _ in 0..5 {
            color_pick(&mut g, id);
        }
        drain(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 5);
        assert_eq!(g.state.players[p.index()].life, 25);
        assert_eq!(g.outcome(), None);
    }
}
#[test]
fn five_distinct_dragons_receive_counters_and_win_even_after_call_source_departure() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &CALL)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ids = five(&mut g, p);
        let source = call(&mut g, p);
        g.move_object(source, Zone::Exile);
        for id in &ids {
            color_pick(&mut g, *id);
        }
        assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
        for id in ids {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
    }
}
#[test]
fn color_counter_choice_requires_matching_controlled_subtype_and_cannot_skip_available_color() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &CALL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let white = dragon(&mut g, p, 1);
    let red = dragon(&mut g, p, 8);
    let enemy = dragon(&mut g, p.other(), 1);
    let wrong = put(&mut g, p, "Savannah Lions", Zone::Battlefield);
    call(&mut g, p);
    let saved = g.snapshot();
    for id in [enemy, wrong, red] {
        let object = g.object_ref(id);
        assert!(g.apply(p, Action::ChooseColorCounter { object }).is_err());
        assert_eq!(g.snapshot(), saved);
    }
    assert!(g.apply(p, Action::DeclineEffectPayment).is_err());
    assert_eq!(g.snapshot(), saved);
    color_pick(&mut g, white);
    color_pick(&mut g, red);
    drain(&mut g);
    assert_eq!(g.counter_count(white, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(red, CounterKind::PlusOne), 1);
    assert_eq!(g.outcome(), None);
}
#[test]
fn counter_replacements_do_not_turn_one_dragon_into_five_successes_and_prohibition_blocks_win() {
    const STOP: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttachedCounterProhibition],
        }],
        extras: &CardExtras {
            equipment: Some(EquipmentSpec {
                stats: Stats {
                    power: 0,
                    toughness: 0,
                },
                keywords: Keywords::NONE,
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (card("Short Bow"), &CALL),
        (card("Gilded Lotus"), &DOUBLE_COUNTER_RULES),
        (card("Swiftfoot Boots"), &STOP),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
    let ids = five(&mut g, p);
    let stop = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
    g.state.objects[stop as usize].attached_to = Some(g.object_ref(ids[4]));
    call(&mut g, p);
    for id in &ids {
        color_pick(&mut g, *id);
    }
    drain(&mut g);
    assert_eq!(g.outcome(), None);
    for id in &ids[..4] {
        assert_eq!(g.counter_count(*id, CounterKind::PlusOne), 2);
    }
    assert_eq!(g.counter_count(ids[4], CounterKind::PlusOne), 0);
    let mut g = scenario(p);
    put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
    let id = dragon(&mut g, p, 31);
    call(&mut g, p);
    for _ in 0..5 {
        color_pick(&mut g, id);
    }
    drain(&mut g);
    assert_eq!(g.counter_count(id, CounterKind::PlusOne), 10);
    assert_eq!(g.outcome(), None);
}
#[test]
fn color_counter_win_respects_opponents_win_restriction() {
    const NO_WIN: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CantLoseGameAndOpponentsCantWin],
        }],
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(card("Short Bow"), &CALL), (card("Gilded Lotus"), &NO_WIN)]);
    let p = Player::P1;
    let mut g = scenario(p);
    put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
    let ids = five(&mut g, p);
    call(&mut g, p);
    for id in ids {
        color_pick(&mut g, id);
    }
    drain(&mut g);
    assert_eq!(g.outcome(), None);
}
#[test]
fn partial_public_color_choices_and_replacement_receipts_are_owned_raw_index_restore() {
    let _scope = fixture_rules::activate(&[
        (card("Short Bow"), &CALL),
        (card("Gilded Lotus"), &DOUBLE_COUNTER_RULES),
        (card("Ghost Vacuum"), &DOUBLE_COUNTER_RULES),
    ]);
    let p = Player::P1;
    let mut parent = scenario(p);
    put(&mut parent, p, "Gilded Lotus", Zone::Battlefield);
    put(&mut parent, p, "Ghost Vacuum", Zone::Battlefield);
    let id = dragon(&mut parent, p, 31);
    call(&mut parent, p);
    color_pick(&mut parent, id);
    assert_eq!(
        parent.observe(p.other()).effect_choice.unwrap().selected[p.index()],
        vec![parent.object_ref(id)]
    );
    let action = Action::ChooseColorCounter {
        object: parent.object_ref(id),
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
    act(&mut raw, action.clone());
    indexed.apply_index(p, rev, index).unwrap();
    for _ in 0..3 {
        color_pick(&mut raw, id);
        color_pick(&mut indexed, id);
    }
    assert!(matches!(raw.state.pending, Pending::Replacement { .. }));
    let replacement_saved = raw.snapshot();
    let mut child = raw.clone();
    let replacement = raw
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseReplacement { .. }))
        .unwrap();
    act(&mut raw, replacement.clone());
    act(&mut child, replacement);
    drain(&mut raw);
    drain(&mut child);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), child.snapshot());
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.counter_count(id, CounterKind::PlusOne), 20);
    child.restore(&replacement_saved).unwrap();
    drain(&mut child);
    assert_eq!(raw.snapshot(), child.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    for _ in 0..3 {
        color_pick(&mut indexed, id);
    }
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn ureni_freezes_lands_at_target_announcement_and_positive_shares_before_later_land_changes() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &URENI)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let lands: Vec<_> = (0..3)
            .map(|_| put(&mut g, p, "Forest", Zone::Battlefield))
            .collect();
        let a = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let source = ureni(&mut g, p);
        assert_eq!(
            g.state.announcing.as_ref().unwrap().item.division_total(),
            Some(3)
        );
        division(&mut g, &[a, b], &[1, 2]);
        for id in lands {
            g.move_object(id, Zone::Graveyard);
        }
        put(&mut g, p, "Forest", Zone::Battlefield);
        g.move_object(source, Zone::Exile);
        drain(&mut g);
        assert_eq!(g.object_view(a).damage, 1);
        assert_eq!(g.object_view(b).damage, 2);
    }
}
#[test]
fn scaled_division_allows_zero_targets_with_positive_total_and_zero_lands() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &URENI)]);
    for count in [0, 3] {
        let p = Player::P0;
        let mut g = scenario(p);
        for _ in 0..count {
            put(&mut g, p, "Forest", Zone::Battlefield);
        }
        let target = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        ureni(&mut g, p);
        if matches!(g.state.pending, Pending::Targets { .. }) {
            act(&mut g, Action::FinishTargets);
        }
        drain(&mut g);
        assert_eq!(g.object_view(target).damage, 0);
    }
}
#[test]
fn scaled_division_rejects_excess_targets_wrong_controller_and_zero_or_incomplete_shares() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &URENI)]);
    let p = Player::P0;
    let mut g = scenario(p);
    for _ in 0..2 {
        put(&mut g, p, "Forest", Zone::Battlefield);
    }
    let a = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    let c = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    ureni(&mut g, p);
    let saved = g.snapshot();
    assert!(
        g.apply(
            p,
            Action::ChooseTarget {
                target: Target::Card(own)
            }
        )
        .is_err()
    );
    assert_eq!(g.snapshot(), saved);
    for id in [a, b] {
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    let saved = g.snapshot();
    assert!(
        g.apply(
            p,
            Action::ChooseTarget {
                target: Target::Card(c)
            }
        )
        .is_err()
    );
    assert_eq!(g.snapshot(), saved);
    if matches!(g.state.pending, Pending::Targets { .. }) {
        act(&mut g, Action::FinishTargets);
    }
    let saved = g.snapshot();
    for amount in [0, 2] {
        assert!(g.apply(p, Action::AllocateTargetAmount { amount }).is_err());
        assert_eq!(g.snapshot(), saved);
    }
    act(&mut g, Action::AllocateTargetAmount { amount: 1 });
    act(&mut g, Action::AllocateTargetAmount { amount: 1 });
    drain(&mut g);
    assert_eq!(g.object_view(a).damage, 1);
    assert_eq!(g.object_view(b).damage, 1);
}
#[test]
fn illegal_or_blinked_division_target_loses_only_its_share_and_planeswalker_is_a_legal_recipient() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &URENI)]);
    for blink in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        for _ in 0..3 {
            put(&mut g, p, "Forest", Zone::Battlefield);
        }
        let a = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        ureni(&mut g, p);
        division(&mut g, &[a, b], &[1, 2]);
        g.move_object(a, Zone::Hand);
        if blink {
            g.move_object(a, Zone::Battlefield);
        }
        drain(&mut g);
        assert_eq!(g.object_view(a).damage, 0);
        assert_eq!(g.object_view(b).damage, 2);
    }
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Forest", Zone::Battlefield);
    let walker = put(
        &mut g,
        p.other(),
        "Elspeth, Storm Slayer",
        Zone::Battlefield,
    );
    g.add_counter(walker, CounterKind::Loyalty, 5);
    ureni(&mut g, p);
    division(&mut g, &[walker], &[1]);
    drain(&mut g);
    assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 4);
}
#[test]
fn copied_scaled_trigger_retains_original_total_and_per_target_shares() {
    const COPY: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
            &[Instruction::CopyTargetAbility { group: 0 }],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &URENI), (card("Opt"), &COPY)]);
    let p = Player::P0;
    let mut g = scenario(p);
    for _ in 0..3 {
        put(&mut g, p, "Forest", Zone::Battlefield);
    }
    let a = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    ureni(&mut g, p);
    division(&mut g, &[a, b], &[1, 2]);
    let trigger = g.state.stack.last().unwrap().id;
    for _ in 0..4 {
        put(&mut g, p, "Forest", Zone::Battlefield);
    }
    let copy = put(&mut g, p, "Opt", Zone::Hand);
    announce(&mut g, copy, &[Target::Stack(trigger)]);
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::Retarget { .. })
    });
    act(&mut g, Action::FinishStackTargets);
    drain(&mut g);
    assert_eq!(g.object_view(a).damage, 2);
    assert_eq!(g.object_view(b).damage, 4);
}
#[test]
fn wide_scaled_division_action_round_trips_and_is_owned_raw_index_restore() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &URENI)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    for _ in 0..2 {
        put(&mut parent, p, "Forest", Zone::Battlefield);
    }
    let a = put(&mut parent, p.other(), "Shivan Dragon", Zone::Battlefield);
    let b = put(&mut parent, p.other(), "Shivan Dragon", Zone::Battlefield);
    ureni(&mut parent, p);
    let wide = u64::from(u32::MAX) + 2;
    parent
        .state
        .announcing
        .as_mut()
        .unwrap()
        .item
        .announced_division = Some(Box::new(wide));
    for id in [a, b] {
        act(
            &mut parent,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if matches!(parent.state.pending, Pending::Targets { .. }) {
        act(&mut parent, Action::FinishTargets);
    }
    let action = Action::AllocateTargetAmount { amount: wide - 1 };
    let encoded = serde_json::to_string(&action).unwrap();
    let decoded: Action = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, action);
    let saved = parent.snapshot();
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|v| *v == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, action.clone());
    indexed.apply_index(p, rev, index).unwrap();
    for g in [&mut raw, &mut indexed] {
        act(g, Action::AllocateTargetAmount { amount: 1 });
        drain(g);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(a).zone, Zone::Graveyard);
    assert_eq!(raw.object_view(b).damage, 1);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    act(&mut indexed, Action::AllocateTargetAmount { amount: 1 });
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
