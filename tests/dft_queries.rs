//! Shared query/search/event mechanics, independent of DFT admission.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const OWN_GRAVE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const ANY_PERMANENT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const ABILITYLESS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        has_rules_abilities: Some(false),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const GRAVE_DEPARTURE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardsLeaveGraveyardKinds(1 | 16),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const RETURN_GRAVE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: OWN_GRAVE,
            min: 1,
            max: 4,
        }),
        &[Instruction::ReturnAllTargetsToHand],
    ),
    ..CardRules::EMPTY
};
const EXILE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::YourTurn,
            event: &TriggerCondition::CardsExiledFromBattlefieldOrGraveyard,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::LoseLife(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
const EXILE_MIXED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Union(&[OWN_GRAVE, ANY_PERMANENT]),
            min: 1,
            max: 4,
        }),
        &[Instruction::Exile],
    ),
    ..CardRules::EMPTY
};
const SEARCH_ZONES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchLibraryAndOrGraveyardToHand(
            SearchFilter::AllOf(&[SearchFilter::Creature, SearchFilter::NoAbilities]),
        )],
    ),
    ..CardRules::EMPTY
};
const SEARCH_ARTIFACT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::Artifact,
            maximum: 1,
            destination: SearchDestination::BattlefieldIfManaValueAtMostElseHand(2),
            tapped: false,
            reveal: true,
        })],
    ),
    ..CardRules::EMPTY
};
const SEARCH_SCALED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchScaled {
            spec: SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 0,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            },
            maximum: AmountValue::Stat(StatValue::MountOrVehicleEntriesThisTurn),
        }],
    ),
    ..CardRules::EMPTY
};
const REVEAL_LANDS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::All,
                filter: LibrarySelectionFilter::Kinds(2),
                optional: false,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::UntilKindsCount { kinds: 2, count: 2 },
            destination: LibraryPickDestination::BattlefieldTapped,
            public: true,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
const THIRD_FROM_TOP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(ANY_PERMANENT)),
        &[Instruction::OwnerLibraryNthFromTop(3)],
    ),
    ..CardRules::EMPTY
};

fn reach_choice(g: &mut Game) {
    for _ in 0..16 {
        if g.state.pending != Pending::Priority {
            return;
        }
        assert!(!g.state.stack.is_empty() || g.state.resolving.is_some());
        act(g, Action::Pass);
    }
    panic!("choice did not arrive");
}

#[test]
fn graveyard_departure_is_one_filtered_owner_batch() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &GRAVE_DEPARTURE_OBSERVER),
        (Card::Opt, &RETURN_GRAVE),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let a = put(&mut g, Player::P0, "Llanowar Elves", Zone::Graveyard);
    let b = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    announce(&mut g, spell, &[Target::Card(a), Target::Card(b)]);
    settle(&mut g);
    assert_eq!(
        g.state.players[0].life, 21,
        "two creature departures are one batch"
    );
    assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
    assert_eq!(g.state.objects[b as usize].zone, Zone::Hand);
    let land = put(&mut g, Player::P0, "Forest", Zone::Graveyard);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(land)));
    settle(&mut g);
    assert_eq!(
        g.state.players[0].life, 21,
        "land-only departure must not trigger"
    );
}

#[test]
fn global_exile_batch_counts_physical_noncreatures_and_source_departure() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &EXILE_OBSERVER),
        (Card::Opt, &EXILE_MIXED),
    ]);
    let mut g = scenario(Player::P0);
    let observer = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let grave_land = put(&mut g, Player::P0, "Forest", Zone::Graveyard);
    let enemy_land = put(&mut g, Player::P1, "Forest", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    announce(
        &mut g,
        spell,
        &[
            Target::Card(observer),
            Target::Card(grave_land),
            Target::Card(enemy_land),
        ],
    );
    settle(&mut g);
    assert_eq!(
        g.state.players[0].life, 19,
        "mixed simultaneous event triggers once even when observer leaves"
    );
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.state.objects[observer as usize].zone, Zone::Exile);
}

#[test]
fn global_exile_trigger_uses_observer_turn_and_excludes_tokens() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &EXILE_OBSERVER),
        (Card::Opt, &EXILE_MIXED),
    ]);
    let mut g = scenario(Player::P1);
    put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let spell = put(&mut g, Player::P1, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(victim)));
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 20);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let token = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    g.state.objects[token as usize].copy_kind = CopyKind::Token;
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(token)));
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 20);
}

#[test]
fn optional_zone_search_has_private_owned_choices_and_reveals_only_result() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &ABILITYLESS),
        (Card::Opt, &SEARCH_ZONES),
    ]);
    let mut g = scenario(Player::P0);
    let library = put(&mut g, Player::P0, "Grizzly Bears", Zone::Library);
    let grave = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let cycling = put(&mut g, Player::P0, "Migrating Ketradon", Zone::Graveyard);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    reach_choice(&mut g);
    act(
        &mut g,
        Action::ChooseSearchZones {
            library: true,
            graveyard: true,
        },
    );
    assert!(
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: library })
    );
    assert!(
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: grave })
    );
    reject(&mut g, Action::ChooseSearch { card: cycling });
    assert!(
        g.observe(Player::P1)
            .zone_search
            .as_ref()
            .unwrap()
            .cards
            .is_none()
    );
    assert!(
        g.observe(Player::P0)
            .zone_search
            .as_ref()
            .unwrap()
            .cards
            .is_some()
    );
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let action = Action::ChooseSearch { card: grave };
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
    act(&mut raw, Action::FinishSearch);
    settle(&mut raw);
    assert_eq!(raw.state.objects[grave as usize].zone, Zone::Hand);
    assert!(
        raw.observe(Player::P1)
            .known_hand
            .contains(&raw.object_ref(grave))
    );
    assert_eq!(g.state.objects[grave as usize].zone, Zone::Graveyard);
    g.restore(&saved).unwrap();
    act(&mut g, Action::ChooseSearch { card: library });
    act(&mut g, Action::FinishSearch);
    settle(&mut g);
    assert_eq!(g.state.objects[library as usize].zone, Zone::Hand);
    assert_eq!(g.state.objects[grave as usize].zone, Zone::Graveyard);
}

#[test]
fn optional_zone_search_can_decline_without_shuffling_or_moving_public_card() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &ABILITYLESS),
        (Card::Opt, &SEARCH_ZONES),
    ]);
    let mut g = scenario(Player::P0);
    let grave = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let library = g.state.players[0].library.clone();
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    reach_choice(&mut g);
    act(
        &mut g,
        Action::ChooseSearchZones {
            library: false,
            graveyard: false,
        },
    );
    act(&mut g, Action::FinishSearch);
    settle(&mut g);
    assert_eq!(g.state.players[0].library, library);
    assert_eq!(g.state.objects[grave as usize].zone, Zone::Graveyard);
    assert!(Card::Forest.has_rules_abilities());
    assert!(
        Card::from_name("Migrating Ketradon")
            .unwrap()
            .has_rules_abilities()
    );
}

#[test]
fn conditional_search_destination_reveals_and_uses_selected_mana_value() {
    let _scope = fixture_rules::activate(&[(Card::Opt, &SEARCH_ARTIFACT)]);
    for (name, expected) in [
        ("Swiftfoot Boots", Zone::Battlefield),
        ("Meteor Golem", Zone::Hand),
    ] {
        let mut g = scenario(Player::P0);
        let found = put(&mut g, Player::P0, name, Zone::Library);
        let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
        cast(&mut g, spell, None);
        reach_choice(&mut g);
        act(&mut g, Action::ChooseSearch { card: found });
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.state.objects[found as usize].zone, expected, "{name}");
        if expected == Zone::Hand {
            assert!(
                g.observe(Player::P1)
                    .known_hand
                    .contains(&g.object_ref(found))
            );
        }
    }
}

#[test]
fn historical_entry_quantity_survives_departure_and_bounds_tapped_search() {
    let _scope = fixture_rules::activate(&[(Card::Opt, &SEARCH_SCALED)]);
    let mut g = scenario(Player::P0);
    let first = put(&mut g, Player::P0, "Veloheart Bike", Zone::Hand);
    cast(&mut g, first, None);
    settle(&mut g);
    let second = put(&mut g, Player::P0, "Alacrian Jaguar", Zone::Hand);
    cast(&mut g, second, None);
    settle(&mut g);
    assert_eq!(
        g.observe(Player::P0).players[0].mount_or_vehicle_entries_this_turn,
        2
    );
    g.move_object(first, Zone::Graveyard);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    reach_choice(&mut g);
    let cards: Vec<_> = g
        .legal_actions()
        .into_iter()
        .filter_map(|a| {
            if let Action::ChooseSearch { card } = a {
                Some(card)
            } else {
                None
            }
        })
        .take(3)
        .collect();
    act(&mut g, Action::ChooseSearch { card: cards[0] });
    act(&mut g, Action::ChooseSearch { card: cards[1] });
    reject(&mut g, Action::ChooseSearch { card: cards[2] });
    act(&mut g, Action::FinishSearch);
    settle(&mut g);
    for id in &cards[..2] {
        assert_eq!(g.state.objects[*id as usize].zone, Zone::Battlefield);
        assert!(g.state.objects[*id as usize].tapped);
    }
    assert_eq!(g.state.objects[cards[2] as usize].zone, Zone::Library);
    g.begin_turn();
    assert_eq!(
        g.observe(Player::P0).players[0].mount_or_vehicle_entries_this_turn,
        0
    );
}

#[test]
fn reveal_until_second_land_preserves_lower_library_and_handles_shortage() {
    let _scope = fixture_rules::activate(&[(Card::Opt, &REVEAL_LANDS)]);
    for lands in [1, 2] {
        let mut g = scenario(Player::P0);
        for object in &mut g.state.objects {
            if object.zone == Zone::Library && object.owner == Player::P0 {
                object.zone = Zone::Exile;
            }
        }
        g.state.players[0].library.clear();
        let untouched = put(&mut g, Player::P0, "Grizzly Bears", Zone::Library);
        let bottom_land = (lands == 2).then(|| put(&mut g, Player::P0, "Forest", Zone::Library));
        let nonland = put(&mut g, Player::P0, "Savannah Lions", Zone::Library);
        let top_land = put(&mut g, Player::P0, "Forest", Zone::Library);
        let top_nonland = put(&mut g, Player::P0, "Grizzly Bears", Zone::Library);
        let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
        cast(&mut g, spell, None);
        reach_choice(&mut g);
        let shown = g.observe(Player::P1).library_selection.unwrap();
        assert_eq!(shown.remaining.len(), 4);
        act(&mut g, Action::SelectLibrary { card: top_land });
        if let Some(id) = bottom_land {
            act(&mut g, Action::SelectLibrary { card: id });
        }
        act(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.state.objects[top_land as usize].zone, Zone::Battlefield);
        assert!(g.state.objects[top_land as usize].tapped);
        if let Some(id) = bottom_land {
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        }
        assert_eq!(g.state.objects[nonland as usize].zone, Zone::Library);
        assert_eq!(g.state.objects[top_nonland as usize].zone, Zone::Library);
        assert_eq!(g.state.objects[untouched as usize].zone, Zone::Library);
    }
}

#[test]
fn target_third_from_top_uses_owner_and_short_library_bottom() {
    let _scope = fixture_rules::activate(&[(Card::Opt, &THIRD_FROM_TOP)]);
    for library_len in [0, 1, 3] {
        let mut g = scenario(Player::P0);
        for object in &mut g.state.objects {
            if object.zone == Zone::Library && object.owner == Player::P1 {
                object.zone = Zone::Exile;
            }
        }
        g.state.players[1].library.clear();
        let original: Vec<_> = (0..library_len)
            .map(|_| put(&mut g, Player::P1, "Forest", Zone::Library))
            .collect();
        let victim = put(&mut g, Player::P1, "Savannah Lions", Zone::Battlefield);
        g.change_controller(g.object_ref(victim), Player::P0);
        let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(victim)));
        settle(&mut g);
        let library = &g.state.players[1].library;
        assert_eq!(library.len(), library_len + 1);
        assert_eq!(library[library_len.saturating_sub(2)], victim);
        assert!(original.iter().all(|id| library.contains(id)));
        assert_eq!(
            g.state
                .last_target_library_move
                .as_ref()
                .unwrap()
                .destination,
            LibraryDestination::NthFromTop(3)
        );
    }
}

#[test]
fn abilityless_graveyard_targets_reject_cycling_and_acquired_abilities() {
    const RETURN_ABILITYLESS: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &OWN_GRAVE,
                predicate: &TargetPredicate::NoAbilities,
            })),
            &[Instruction::ReturnAllTargetsToHand],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &ABILITYLESS),
        (Card::Opt, &RETURN_ABILITYLESS),
    ]);
    let mut g = scenario(Player::P0);
    let vanilla = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let cycling = put(&mut g, Player::P0, "Migrating Ketradon", Zone::Graveyard);
    let acquired = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    g.grant_graveyard_permission(
        Player::P0,
        g.object_ref(acquired),
        GraveyardCastKind::Flashback,
    );
    assert!(g.object_has_rules_abilities(acquired));
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(cycling),
        },
    );
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(acquired),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(vanilla),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.objects[vanilla as usize].zone, Zone::Hand);
    assert_eq!(g.state.objects[cycling as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[acquired as usize].zone, Zone::Graveyard);
}
