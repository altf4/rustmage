//! Acquired embalm is a graveyard activation with copiable white/Zombie/no-cost results.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;
const GRANT: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::GrantEmbalmTargets],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const VANILLA: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    extras: &CardExtras {
        has_rules_abilities: Some(false),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ALT_SPELL: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0; 6],
            generic: 0,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PROHIBIT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::GraveyardCardsCannotBeExiled],
    }],
    ..CardRules::EMPTY
};
const COUNTER_PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::OtherControlledEntryCountersByManaValue {
                filter: ReplacementFilter::CreatureOrVehicle,
                kind: CounterKind::PlusOne,
                maximum: 0,
                at_most: 1,
                above: 3,
            },
        ],
    }],
    ..CardRules::EMPTY
};
fn finish(g: &mut Game) {
    for _ in 0..80 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        if g.state.pending == Pending::Priority {
            act(g, Action::Pass);
            continue;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseReplacement { .. } | Action::ChooseEntryTimestamp { .. }
                )
            })
            .expect("explicit supported entry/replacement choice");
        act(g, action);
    }
    panic!("embalm resolution did not finish")
}
fn grant(g: &mut Game, target: CardId) -> CardId {
    let source = put(g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets)
    }
    pay(g);
    finish(g);
    source
}
fn action(g: &Game, id: CardId) -> Action {
    let row = g
        .embalm_views()
        .unwrap()
        .into_iter()
        .find(|r| r.object == g.object_ref(id))
        .unwrap();
    Action::AnnounceEmbalm {
        source: row.object,
        acquisition: row.acquisition,
    }
}
fn choose_cost_and_pay(g: &mut Game) {
    for _ in 0..4 {
        if let Some(a) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseActivationCost { .. }))
        {
            act(g, a)
        } else {
            break;
        }
    }
    pay(g);
}
fn embalm(g: &mut Game, id: CardId) {
    let a = action(g, id);
    act(g, a);
    choose_cost_and_pay(g);
    finish(g);
}
fn tokens(g: &Game) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.is_token()
                && g.copy_recipe(g.object_ref(id as CardId))
                    .is_some_and(|r| r.exceptions.embalm))
            .then_some(id as CardId)
        })
        .collect()
}
#[test]
fn embalm_is_not_a_cast_and_grant_survives_the_grantors_departure() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &GRANT)]);
    let mut g = scenario(Player::P0);
    let card = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    let source = grant(&mut g, card);
    g.move_object(source, Zone::Hand);
    let before_spells = g.state.players[0].spells_cast_this_turn;
    let a = action(&g, card);
    act(&mut g, a);
    choose_cost_and_pay(&mut g);
    assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
    assert!(
        matches!(g.state.stack.last().unwrap().kind,StackKind::ActivatedAbility{ability} if ability==embalm::ACTIVATION_INDEX)
    );
    assert!(g.embalm_views().is_none());
    finish(&mut g);
    let made = tokens(&g);
    assert_eq!(made.len(), 1);
    let token = made[0];
    assert_eq!(g.colors(token), 1);
    assert!(g.has_subtype(token, "Bear") && g.has_subtype(token, "Zombie"));
    assert_eq!(g.object_mana_value(token), 0);
    assert_eq!(stats(&g, token), (2, 2));
    assert_eq!(g.state.players[0].spells_cast_this_turn, before_spells);
    let goal = FundingGoal {
        actor: Player::P0,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&g, &goal);
    assert_eq!(
        characteristic_query::CharacteristicQuery::in_world(&world).object_view(token),
        g.object_view(token)
    );
    assert_eq!(CharacteristicWorld::mana_value(&world, token), 0);
}
#[test]
fn an_embalmed_token_and_a_copy_of_it_keep_copiable_color_type_and_no_mana_cost() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &GRANT)]);
    let mut g = scenario(Player::P0);
    let card = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    grant(&mut g, card);
    embalm(&mut g, card);
    let first = tokens(&g)[0];
    let recipe = g.copiable_recipe(g.object_ref(first));
    let lord = put(&mut g, Player::P0, "Wilt-Leaf Liege", Zone::Battlefield);
    assert_eq!(
        stats(&g, first),
        (3, 3),
        "only white, not the original green"
    );
    g.stage_copy_creation(Player::P0, recipe, 1);
    g.advance_replacements(true);
    g.advance_entries();
    finish(&mut g);
    let made = tokens(&g);
    assert_eq!(made.len(), 2);
    for id in made {
        assert_eq!(g.colors(id), 1);
        assert!(g.has_subtype(id, "Zombie") && g.has_subtype(id, "Bear"));
        assert_eq!(g.object_mana_value(id), 0);
        assert_eq!(stats(&g, id), (3, 3));
    }
    g.move_object(lord, Zone::Hand);
}
#[test]
fn embalm_pays_hybrid_and_twobrid_printed_symbols_without_using_spell_alternatives() {
    const PLAIN: CardRules = CardRules {
        effect: Effect::Creature(4, 4),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &GRANT),
        (Card::from_name("Wildgrowth Archaic").unwrap(), &PLAIN),
        (Card::from_name("Savannah Lions").unwrap(), &ALT_SPELL),
    ]);
    for pool in [[0, 0, 0, 0, 2, 0], [0, 0, 0, 0, 1, 2], [0, 0, 0, 0, 0, 4]] {
        let mut g = scenario(Player::P0);
        let card = put(&mut g, Player::P0, "Wildgrowth Archaic", Zone::Graveyard);
        grant(&mut g, card);
        g.state.players[0].mana = pool;
        embalm(&mut g, card);
        assert_eq!(tokens(&g).len(), 1);
    }
    for color in [0, 4] {
        let mut g = scenario(Player::P0);
        let card = put(&mut g, Player::P0, "Dryad Militant", Zone::Graveyard);
        grant(&mut g, card);
        g.state.players[0].mana = [0; 6];
        g.state.players[0].mana[color] = 1;
        embalm(&mut g, card);
        assert_eq!(g.colors(tokens(&g)[0]), 1);
    }
    let mut g = scenario(Player::P0);
    let card = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    grant(&mut g, card);
    g.state.players[0].mana = [0; 6];
    let a = action(&g, card);
    reject(&mut g, a);
    g.state.players[0].mana[0] = 1;
    embalm(&mut g, card);
}
#[test]
fn chosen_activation_x_is_paid_but_a_noncast_embalmed_token_enters_with_x_zero() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &GRANT),
        (
            Card::from_name("Cultivator's Caravan").unwrap(),
            &COUNTER_PROVIDER,
        ),
    ]);
    let mut g = scenario(Player::P0);
    put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    let card = put(&mut g, Player::P0, "Pterafractyl", Zone::Graveyard);
    grant(&mut g, card);
    let before = g.state.players[0]
        .mana
        .iter()
        .map(|&n| u64::from(n))
        .sum::<u64>();
    let a = action(&g, card);
    act(&mut g, a);
    act(&mut g, Action::ChooseX { value: 3 });
    choose_cost_and_pay(&mut g);
    let after = g.state.players[0]
        .mana
        .iter()
        .map(|&n| u64::from(n))
        .sum::<u64>();
    assert_eq!(before - after, 5);
    finish(&mut g);
    let token = tokens(&g)[0];
    assert_eq!(
        g.counter_count(token, CounterKind::PlusOne),
        1,
        "only the ordinary MV0 entry replacement adds a counter"
    );
    assert_eq!(g.object_mana_value(token), 0);
    assert_eq!(stats(&g, token), (2, 1));
    assert_eq!(g.state.players[0].life, 22, "the copied ETB still triggers");
}
#[test]
fn embalmed_chosen_x_graveyard_exile_entry_uses_zero_as_its_maximum() {
    const ENTRY: CardRules = CardRules {
        effect: Effect::Creature(0, 1),
        extras: &CardExtras {
            entry_exile_counters: Some(EntryExileCountersSpec {
                filter: SearchFilter::Creature,
                kind: CounterKind::PlusOne,
                per_card: 3,
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &GRANT),
        (Card::from_name("Pterafractyl").unwrap(), &ENTRY),
    ]);
    let mut g = scenario(Player::P0);
    let available = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    let card = put(&mut g, Player::P0, "Pterafractyl", Zone::Graveyard);
    grant(&mut g, card);
    let a = action(&g, card);
    act(&mut g, a);
    act(&mut g, Action::ChooseX { value: 2 });
    choose_cost_and_pay(&mut g);
    finish(&mut g);
    let token = tokens(&g)[0];
    assert_eq!(g.counter_count(token, CounterKind::PlusOne), 0);
    assert_eq!(g.state.objects[available as usize].zone, Zone::Graveyard);
    assert!(
        g.state
            .linked_choices
            .as_ref()
            .is_none_or(|s| s.links.is_empty())
    );
}
#[test]
fn acquired_embalm_counts_as_an_ability_until_expiry_and_never_follows_a_new_incarnation() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &GRANT),
        (Card::from_name("Savannah Lions").unwrap(), &VANILLA),
    ]);
    let mut g = scenario(Player::P0);
    let card = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    assert!(!g.object_has_rules_abilities(card));
    grant(&mut g, card);
    assert!(g.object_has_rules_abilities(card));
    assert!(additional_costs::has_craft_activated_ability(&g, card));
    let stale = action(&g, card);
    g.move_object(card, Zone::Hand);
    g.move_object(card, Zone::Graveyard);
    assert!(!g.object_has_rules_abilities(card));
    reject(&mut g, stale);
    grant(&mut g, card);
    g.finish_cleanup();
    assert!(g.embalm_views().is_none());
    assert!(!g.object_has_rules_abilities(card));
    let mut fresh = scenario(Player::P0);
    let vanilla = put(&mut fresh, Player::P0, "Savannah Lions", Zone::Graveyard);
    grant(&mut fresh, vanilla);
    embalm(&mut fresh, vanilla);
    assert!(
        !fresh.object_has_rules_abilities(tokens(&fresh)[0]),
        "the acquired graveyard ability is not a copiable printed ability"
    );
}
#[test]
fn embalm_timing_and_owner_restrictions_are_activation_restrictions() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &GRANT)]);
    let mut g = scenario(Player::P0);
    let card = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    grant(&mut g, card);
    let a = action(&g, card);
    g.state.step = Step::BeginCombat;
    reject(&mut g, a.clone());
    g.state.step = Step::Main1;
    g.state.active = Player::P1;
    reject(&mut g, a.clone());
    g.state.active = Player::P0;
    g.state.priority = Player::P1;
    reject(&mut g, a.clone());
    g.state.priority = Player::P0;
    assert!(g.legal_actions().contains(&a));
    let spell = put(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    cast(&mut g, spell, None);
    assert!(!g.legal_actions().contains(&a));
    finish(&mut g);
    assert!(g.legal_actions().contains(&a));
}
#[test]
fn countering_embalm_leaves_its_exile_cost_paid_and_creates_no_token() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &GRANT)]);
    let mut g = scenario(Player::P0);
    let card = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    grant(&mut g, card);
    let a = action(&g, card);
    act(&mut g, a);
    choose_cost_and_pay(&mut g);
    let ability = g.state.stack.last().unwrap().id;
    assert!(g.counter_stack_item(ability, false));
    finish(&mut g);
    assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
    assert!(tokens(&g).is_empty());
}
#[test]
fn multiple_acquisitions_are_distinct_owned_rows_and_raw_indexed_activation_matches() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &GRANT)]);
    let mut parent = scenario(Player::P0);
    let card = put(&mut parent, Player::P0, "Savannah Lions", Zone::Graveyard);
    grant(&mut parent, card);
    grant(&mut parent, card);
    let rows = parent.embalm_views().unwrap();
    assert_eq!(rows.len(), 2);
    assert_ne!(rows[0].acquisition, rows[1].acquisition);
    let a = action(&parent, card);
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|v| *v == a)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, a);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    choose_cost_and_pay(&mut raw);
    finish(&mut raw);
    choose_cost_and_pay(&mut indexed);
    finish(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.embalm_views().unwrap().len(), 2);
    assert_eq!(indexed.state.objects[card as usize].zone, Zone::Graveyard);
    assert!(tokens(&indexed).is_empty());
}
#[test]
fn graveyard_exile_prohibition_is_shared_by_actual_embalm_and_funding() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &GRANT),
        (Card::from_name("Cultivator's Caravan").unwrap(), &PROHIBIT),
    ]);
    let mut g = scenario(Player::P0);
    let card = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    grant(&mut g, card);
    let blocker = put(
        &mut g,
        Player::P1,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    let a = action(&g, card);
    reject(&mut g, a.clone());
    g.move_object(blocker, Zone::Hand);
    assert!(g.legal_actions().contains(&a));
    embalm(&mut g, card);
}
#[test]
fn graveyard_borrower_sees_the_acquired_ability_but_cannot_pay_its_graveyard_only_source_cost() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &GRANT)]);
    let mut g = scenario(Player::P0);
    let card = put(&mut g, Player::P0, "Llanowar Elves", Zone::Graveyard);
    grant(&mut g, card);
    let borrower = put(
        &mut g,
        Player::P0,
        "Thranduil, the Elvenking",
        Zone::Battlefield,
    );
    let descriptors = borrowed_activations::abilities(&g, borrower);
    assert!(
        descriptors
            .iter()
            .any(|d| d.ability == embalm::ACTIVATION_INDEX)
    );
    assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::AnnounceGraveyardBorrowedAbility{card,ability,..} if *card==borrower && *ability==embalm::ACTIVATION_INDEX)));
    g.expire_embalm();
    assert!(
        !borrowed_activations::abilities(&g, borrower)
            .iter()
            .any(|d| d.ability == embalm::ACTIVATION_INDEX)
    );
}
#[test]
fn token_creation_replacements_apply_to_the_complete_embalm_recipe() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &GRANT)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Doubling Season", Zone::Battlefield);
    let card = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    grant(&mut g, card);
    embalm(&mut g, card);
    let made = tokens(&g);
    assert_eq!(made.len(), 2);
    for token in made {
        assert_eq!(g.colors(token), 1);
        assert!(g.has_subtype(token, "Zombie"));
        assert_eq!(g.object_mana_value(token), 0);
    }
}
#[test]
fn absent_mana_cost_is_unpayable_and_reserved_templates_are_disjoint() {
    assert!(mana_symbols::printed_cost("").is_none());
    assert_eq!(
        mana_symbols::printed_cost("{0}").unwrap().options,
        vec![ManaCost {
            colored: [0; 6],
            generic: 0
        }]
    );
    let cost = mana_symbols::printed_cost("{X}{X}{2/G}{W/U}{C}").unwrap();
    assert_eq!(cost.x_symbols, 2);
    assert_eq!(cost.options.len(), 4);
    assert!(cost.options.iter().all(|c| c.colored[5] == 1));
    assert_eq!(embalm::ACTIVATION_INDEX, activation_slots::EMBALM);
    assert!(activation_slots::EMBALM < activation_slots::GRANTED_SACRIFICE_MANA);
    for color in 0..6 {
        let index = activation_slots::intrinsic(color);
        let rule = Card::Forest.activation(index);
        let mut expected = [0; 6];
        expected[color] = 1;
        assert_eq!(
            rule.mana_production(),
            Some(ManaProduction::Fixed(expected))
        );
    }
    assert!(
        Card::Forest
            .activation(activation_slots::GRANTED_TAP_MANA)
            .cost
            .tap
    );
    assert!(
        Card::Forest
            .activation(activation_slots::GRANTED_SACRIFICE_MANA)
            .cost
            .sacrifice
    );
    let rule = Card::Forest.activation(activation_slots::EMBALM);
    assert_eq!(rule.origin, ActivationOrigin::Graveyard);
    assert!(rule.cost.exile_self && rule.sorcery && rule.mana_production().is_none());
}

const ENTRY_COPY: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 17,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
#[test]
fn a_later_as_entry_copy_replaces_embalm_exceptions_while_decline_keeps_them() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &GRANT),
        (Card::from_name("Bear Cub").unwrap(), &ENTRY_COPY),
    ]);
    for copy in [false, true] {
        let mut g = scenario(Player::P0);
        let donor = put(&mut g, Player::P0, "Druid of the Cowl", Zone::Battlefield);
        let card = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
        grant(&mut g, card);
        let a = action(&g, card);
        act(&mut g, a);
        choose_cost_and_pay(&mut g);
        for _ in 0..8 {
            if g.state.pending != Pending::Priority {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::EntryCopy { .. }));
        let object = copy.then(|| g.object_ref(donor));
        act(&mut g, Action::ChooseEntryCopy { object });
        finish(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
            .unwrap()
            .0 as CardId;
        if copy {
            assert_eq!(g.colors(token), 16);
            assert_eq!(g.object_mana_value(token), 2);
            assert!(!g.has_subtype(token, "Zombie"));
            assert!(!g.copiable_recipe(g.object_ref(token)).exceptions.embalm);
            let reference = g.object_ref(token);
            let physical = g.temporary_copy_view(reference).unwrap();
            assert!(physical.physical_token);
            assert_eq!(physical.physical_card, Card::from_name("Bear Cub").unwrap());
            g.apply_temporary_copy(
                reference,
                CopyRecipe::new(Card::from_name("Savannah Lions").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            g.expire_temporary_copies(true);
            assert!(g.state.objects[token as usize].is_token());
            assert_eq!(
                g.state.objects[token as usize].card,
                Card::from_name("Druid of the Cowl").unwrap()
            );
            assert_eq!(g.colors(token), 16);
            assert_eq!(g.object_mana_value(token), 2);
            assert!(!g.has_subtype(token, "Zombie"));
        } else {
            assert_eq!(g.colors(token), 1);
            assert_eq!(g.object_mana_value(token), 0);
            assert!(g.has_subtype(token, "Zombie"));
        }
    }
}
#[test]
fn an_embalmed_as_entry_copier_can_copy_an_embalmed_donor_or_choose_a_new_aura_attachment() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &GRANT),
        (Card::from_name("Bear Cub").unwrap(), &ENTRY_COPY),
    ]);
    let mut g = scenario(Player::P0);
    let first = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    grant(&mut g, first);
    embalm(&mut g, first);
    let donor = tokens(&g)[0];
    let card = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    grant(&mut g, card);
    let a = action(&g, card);
    act(&mut g, a);
    choose_cost_and_pay(&mut g);
    for _ in 0..8 {
        if g.state.pending != Pending::Priority {
            break;
        }
        act(&mut g, Action::Pass);
    }
    let object = Some(g.object_ref(donor));
    act(&mut g, Action::ChooseEntryCopy { object });
    finish(&mut g);
    assert_eq!(tokens(&g).len(), 2);
    for id in tokens(&g) {
        assert_eq!(g.colors(id), 1);
        assert_eq!(g.object_mana_value(id), 0);
        assert!(g.has_subtype(id, "Zombie") && g.has_subtype(id, "Cat"));
    }
    let recipient = put(&mut g, Player::P0, "Druid of the Cowl", Zone::Battlefield);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(recipient));
    g.change_object_form(
        g.object_ref(aura),
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    let card = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    grant(&mut g, card);
    let a = action(&g, card);
    act(&mut g, a);
    choose_cost_and_pay(&mut g);
    for _ in 0..8 {
        if g.state.pending != Pending::Priority {
            break;
        }
        act(&mut g, Action::Pass);
    }
    let object = Some(g.object_ref(aura));
    act(&mut g, Action::ChooseEntryCopy { object });
    assert!(matches!(g.state.pending, Pending::AuraEntry { .. }));
    let object = g.object_ref(recipient);
    act(&mut g, Action::ChooseAuraAttachment { object });
    finish(&mut g);
    let made = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.is_token()
                && o.card == Card::from_name("Pacifism").unwrap()
        })
        .unwrap()
        .0 as CardId;
    assert_eq!(g.state.objects[made as usize].attached_to, Some(object));
    assert_eq!(g.object_mana_value(made), 2);
    assert!(!g.has_subtype(made, "Zombie"));
}

struct ClearPrevention;
impl Drop for ClearPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}
#[test]
fn a_replacement_modified_exile_cost_is_paid_without_consuming_an_unmoved_acquisition() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &GRANT)]);
    let mut g = scenario(Player::P0);
    let card = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    grant(&mut g, card);
    let reference = g.object_ref(card);
    let _clear = ClearPrevention;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(reference)));
    embalm(&mut g, card);
    assert_eq!(g.object_ref(card), reference);
    assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
    assert_eq!(tokens(&g).len(), 1);
    assert_eq!(g.embalm_views().unwrap().len(), 1);
}

#[test]
fn an_embalmed_aerial_doombot_has_no_own_mana_cost_to_reduce_its_power_up_cost() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &GRANT)]);
    let mut g = scenario(Player::P0);
    let ordinary = put(&mut g, Player::P0, "Aerial Doombot", Zone::Hand);
    cast(&mut g, ordinary, None);
    finish(&mut g);
    assert!(g.entered_this_turn(g.object_ref(ordinary)));
    g.state.players[0].mana = [0, 0, 0, 0, 0, 5];
    assert!(
        g.legal_actions().contains(&Action::AnnounceActivate {
            card: ordinary,
            ability: 0,
            counter_payment: Counters::default()
        }),
        "ordinary Doombot receives its U entry-turn reduction"
    );
    g.state.players[0].mana = [8; 6];
    let card = put(&mut g, Player::P0, "Aerial Doombot", Zone::Graveyard);
    grant(&mut g, card);
    embalm(&mut g, card);
    let token = tokens(&g)[0];
    assert!(g.entered_this_turn(g.object_ref(token)));
    let action = Action::AnnounceActivate {
        card: token,
        ability: 0,
        counter_payment: Counters::default(),
    };
    g.state.players[0].mana = [0, 0, 0, 0, 0, 5];
    reject(&mut g, action.clone());
    let goal = FundingGoal {
        actor: Player::P0,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&g, &goal);
    assert_eq!(
        CharacteristicWorld::activation_power_up_costs(
            &world,
            world.reference(token),
            Card::from_name("Aerial Doombot").unwrap(),
            0
        )
        .unwrap()
        .reductions,
        vec![ManaCost {
            colored: [0; 6],
            generic: 0
        }]
    );
    g.state.players[0].mana[1] = 1;
    assert!(g.legal_actions().contains(&action));
    act(&mut g, action);
    pay(&mut g);
    finish(&mut g);
    assert_eq!(g.counter_count(token, CounterKind::PlusOne), 3);
}
