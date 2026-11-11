//! Mixed-zone ordered Craft costs and exact material linkage.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const ANY: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const CRAFT: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        generic: 1,
        mana_component: true,
        exile_self: true,
        additional: Some(AdditionalCost {
            kind: EffectChoiceKind::ExileCraftMaterials(CraftMaterials::Count {
                minimum: 2,
                maximum: Some(2),
            }),
            filter: ANY,
        }),
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::ReturnCraftedSourceTransformed],
    sorcery: true,
    once_per_turn: false,
};
const EXACT_TWO: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[CRAFT],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const RANGE: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileCraftMaterials(CraftMaterials::Count {
                    minimum: 1,
                    maximum: Some(3),
                }),
                filter: ANY,
            }),
            ..CRAFT.cost
        },
        ..CRAFT
    }],
    ..EXACT_TWO
};
const SUBTYPES: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileCraftMaterials(CraftMaterials::DistinctSubtypes(&[
                    "Dinosaur", "Merfolk", "Pirate", "Vampire",
                ])),
                filter: ANY,
            }),
            ..CRAFT.cost
        },
        ..CRAFT
    }],
    ..EXACT_TWO
};
const CREATURES: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileCraftMaterials(CraftMaterials::Count {
                    minimum: 2,
                    maximum: Some(2),
                }),
                filter: SacrificeFilter { kinds: 1, ..ANY },
            }),
            ..CRAFT.cost
        },
        ..CRAFT
    }],
    ..EXACT_TWO
};
const SACRIFICE_MANA: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 0, 1,
        ]))],
        sorcery: false,
        ..CRAFT
    }],
    ..CardRules::EMPTY
};

fn front() -> Card {
    Card::from_name("Braided Net").unwrap()
}
fn announce(source: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card: source,
        ability,
        counter_payment: Counters::default(),
    }
}
fn select(game: &mut Game, material: CardId) {
    let object = game.object_ref(material);
    act(game, Action::ChooseAdditionalCost { object });
}
fn select_clause(game: &mut Game, kind: CastCostClause) {
    let clause = game
        .observe(game.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == kind && !row.paid)
        .expect("unpaid Craft cost") as u16;
    act(game, Action::ChooseCastCost { clause });
}
fn begin(game: &mut Game, source: CardId, ability: u16) {
    act(game, announce(source, ability));
    act(game, Action::BeginOrderedCosts);
}
fn finish_costs(game: &mut Game) {
    for _ in 0..8 {
        if game.state.announcing.is_none() {
            return;
        }
        match game.state.pending {
            Pending::CastCosts { .. } => {
                choose(game, |action| {
                    matches!(action, Action::ChooseCastCost { .. })
                });
            }
            Pending::PayCosts { .. } => pay(game),
            _ => panic!("unexpected remaining Craft cost {:?}", game.state.pending),
        }
    }
    panic!("Craft cost completion exceeded its budget");
}

#[test]
fn exact_mixed_zone_materials_exile_once_and_tokens_supply_no_card_link() {
    let _rules = fixture_rules::activate(&[(front(), &EXACT_TWO)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let companion = put(&mut game, player, "Cartographer's Companion", Zone::Hand);
        cast(&mut game, companion, None);
        settle(&mut game);
        let token = game
            .state
            .objects
            .iter()
            .position(|object| object.is_token() && object.card.has_subtype("Map"))
            .unwrap() as CardId;
        let source = put(&mut game, player.other(), "Braided Net", Zone::Battlefield);
        game.state.objects[source as usize].controller = player;
        let graveyard = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
        let enemy = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
        let source_before = game.object_ref(source);
        begin(&mut game, source, 0);
        select_clause(&mut game, CastCostClause::Additional);
        let enemy_object = game.object_ref(enemy);
        reject(
            &mut game,
            Action::ChooseAdditionalCost {
                object: enemy_object,
            },
        );
        reject(
            &mut game,
            Action::ChooseAdditionalCost {
                object: source_before,
            },
        );
        select(&mut game, token);
        let token_object = game.object_ref(token);
        reject(
            &mut game,
            Action::ChooseAdditionalCost {
                object: token_object,
            },
        );
        reject(&mut game, Action::FinishCraftMaterials);
        assert_eq!(game.object_view(token).zone, Zone::Battlefield);
        select(&mut game, graveyard);
        assert_eq!(game.object_view(graveyard).zone, Zone::Exile);
        assert_eq!(game.object_view(source).zone, Zone::Battlefield);
        finish_costs(&mut game);
        let exile_source = game.object_ref(source);
        assert_ne!(exile_source, source_before);
        settle(&mut game);
        let returned = game.object_ref(source);
        assert_ne!(returned, exile_source);
        assert_eq!(game.object_view(source).card.name(), "Braided Quipu");
        assert_eq!(game.object_view(source).controller, player.other());
        assert_eq!(
            game.crafted_exiled_cards(returned),
            vec![game.object_ref(graveyard)]
        );
        assert!(game.crafted_exiled_cards(source_before).is_empty());
        assert!(game.observe(player).borrowed_abilities.is_empty());
        assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
        assert!(
            game.observe(player)
                .linked_exile
                .iter()
                .all(|link| link.source != returned || link.exiled.card != token)
        );
    }
}

#[test]
fn bounded_material_selection_branches_finish_at_the_selected_cardinality() {
    let _rules = fixture_rules::activate(&[(front(), &RANGE)]);
    let player = Player::P0;
    let mut root = scenario(player);
    let source = put(&mut root, player, "Braided Net", Zone::Battlefield);
    let first = put(&mut root, player, "Forest", Zone::Graveyard);
    let second = put(&mut root, player, "Opt", Zone::Graveyard);
    let third = put(&mut root, player, "Shock", Zone::Graveyard);
    begin(&mut root, source, 0);
    select_clause(&mut root, CastCostClause::CraftSource);
    select_clause(&mut root, CastCostClause::Additional);
    reject(&mut root, Action::FinishCraftMaterials);
    select(&mut root, first);
    let saved = root.snapshot();
    let action = Action::FinishCraftMaterials;
    let index = root
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let mut raw = root.clone();
    let mut indexed = root.clone();
    let mut more = root.clone();
    drop(root);
    act(&mut raw, action);
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    select(&mut more, second);
    select(&mut more, third);
    finish_costs(&mut raw);
    finish_costs(&mut indexed);
    finish_costs(&mut more);
    settle(&mut raw);
    settle(&mut indexed);
    settle(&mut more);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.crafted_exiled_cards(raw.object_ref(source)).len(), 1);
    assert_eq!(more.crafted_exiled_cards(more.object_ref(source)).len(), 3);
    assert_eq!(raw.object_view(second).zone, Zone::Graveyard);

    let complete = raw.snapshot();
    raw.restore(&saved).unwrap();
    act(&mut raw, Action::FinishCraftMaterials);
    finish_costs(&mut raw);
    settle(&mut raw);
    assert_eq!(raw.snapshot(), complete);

    let old_exile = raw.object_ref(first);
    raw.move_object(first, Zone::Graveyard);
    raw.move_object(first, Zone::Exile);
    assert_ne!(old_exile, raw.object_ref(first));
    assert!(raw.crafted_exiled_cards(raw.object_ref(source)).is_empty());
}

#[test]
fn distinct_subtype_slots_assign_multitype_objects_without_reusing_one_object() {
    let _rules = fixture_rules::activate(&[(front(), &SUBTYPES)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Braided Net", Zone::Battlefield);
    let changeling = put(&mut game, player, "Mutable Explorer", Zone::Battlefield);
    assert!(!game.legal_actions().contains(&announce(source, 0)));
    let dinosaur = put(&mut game, player, "Colossadactyl", Zone::Graveyard);
    let second_dinosaur = put(&mut game, player, "Colossadactyl", Zone::Battlefield);
    let merfolk = put(&mut game, player, "Brineborn Cutthroat", Zone::Graveyard);
    let pirate = put(&mut game, player, "Goblin Boarders", Zone::Battlefield);
    begin(&mut game, source, 0);
    select_clause(&mut game, CastCostClause::Additional);
    select(&mut game, changeling);
    select(&mut game, dinosaur);
    let duplicate_slot = game.object_ref(second_dinosaur);
    reject(
        &mut game,
        Action::ChooseAdditionalCost {
            object: duplicate_slot,
        },
    );
    select(&mut game, merfolk);
    select(&mut game, pirate);
    finish_costs(&mut game);
    settle(&mut game);
    assert_eq!(game.crafted_exiled_cards(game.object_ref(source)).len(), 4);
    assert_eq!(game.object_view(second_dinosaur).zone, Zone::Battlefield);
}

#[test]
fn material_funding_uses_tapped_and_sacrificed_materials_that_remain_eligible() {
    let player = Player::P0;
    {
        let _rules = fixture_rules::activate(&[(front(), &CREATURES)]);
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let source = put(&mut game, player, "Braided Net", Zone::Battlefield);
        let elf = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        assert!(game.legal_actions().contains(&announce(source, 0)));
        act(&mut game, announce(source, 0));
        act(&mut game, Action::TapMana { card: elf });
        act(&mut game, Action::BeginOrderedCosts);
        select_clause(&mut game, CastCostClause::Additional);
        select(&mut game, elf);
        select(&mut game, second);
        finish_costs(&mut game);
        settle(&mut game);
        assert_eq!(game.crafted_exiled_cards(game.object_ref(source)).len(), 2);
    }
    {
        let _rules = fixture_rules::activate(&[
            (front(), &CREATURES),
            (Card::GrizzlyBears, &SACRIFICE_MANA),
        ]);
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let source = put(&mut game, player, "Braided Net", Zone::Battlefield);
        let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let mut replaced = game.clone();
        put(
            &mut replaced,
            player.other(),
            "Rest in Peace",
            Zone::Battlefield,
        );
        assert!(!replaced.legal_actions().contains(&announce(source, 0)));
        reject(&mut replaced, announce(source, 0));

        act(&mut game, announce(source, 0));
        choose(
            &mut game,
            |action| matches!(action, Action::ActivateMana { card, .. } if *card == first),
        );
        assert_eq!(game.object_view(first).zone, Zone::Graveyard);
        act(&mut game, Action::BeginOrderedCosts);
        select_clause(&mut game, CastCostClause::Additional);
        select(&mut game, first);
        select(&mut game, second);
        finish_costs(&mut game);
        settle(&mut game);
        assert_eq!(game.crafted_exiled_cards(game.object_ref(source)).len(), 2);
    }
}

#[test]
fn legacy_single_artifact_and_enigma_material_routes_keep_exact_receipts() {
    let player = Player::P0;
    let mut single = scenario(player);
    let source = put(&mut single, player, "Braided Net", Zone::Battlefield);
    let artifact = put(
        &mut single,
        player,
        "Cartographer's Companion",
        Zone::Graveyard,
    );
    begin(&mut single, source, 1);
    select_clause(&mut single, CastCostClause::Additional);
    select(&mut single, artifact);
    finish_costs(&mut single);
    settle(&mut single);
    assert_eq!(
        single.crafted_exiled_cards(single.object_ref(source)),
        vec![single.object_ref(artifact)]
    );

    let mut enigma = scenario(player);
    let jewel = put(&mut enigma, player, "The Enigma Jewel", Zone::Battlefield);
    let donors = (0..4)
        .map(|index| {
            put(
                &mut enigma,
                player,
                "Llanowar Elves",
                if index & 1 == 0 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                },
            )
        })
        .collect::<Vec<_>>();
    begin(&mut enigma, jewel, 1);
    select_clause(&mut enigma, CastCostClause::Additional);
    for donor in &donors {
        select(&mut enigma, *donor);
    }
    act(&mut enigma, Action::FinishCraftMaterials);
    finish_costs(&mut enigma);
    settle(&mut enigma);
    assert_eq!(
        enigma.object_view(jewel).card.name(),
        "Locus of Enlightenment"
    );
    let links = enigma.crafted_exiled_cards(enigma.object_ref(jewel));
    assert_eq!(links.len(), 4);
    let abilities = enigma.observe(player).borrowed_abilities;
    assert_eq!(abilities.len(), 4);
    assert!(
        abilities
            .iter()
            .all(|ability| ability.acquired_once_per_turn)
    );
}
