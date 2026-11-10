//! Paid Mayhem rules, independent of the set's card definitions.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const GRAVEYARD_REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Graveyard,
    guard: StepGuard::Always,
};
const GRANT_GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, GRAVEYARD_REDUCTION)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::GrantManaCostMayhem],
    },
];
const GRANT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GRANT_GROUPS,
    extras: &CardExtras {
        static_ability_groups: GRANT_GROUPS,
        cost_reductions: &[GRAVEYARD_REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MAYHEM_REWARD: &[Instruction] = &[
    Instruction::IfMayhemPaid {
        paid: true,
        skip: 1,
    },
    Instruction::GainLife(PlayerSet::Controller, 3),
];
const HYBRID: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, MAYHEM_REWARD),
    extras: &CardExtras {
        mayhem_cost: Some(ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            ..ManaCost::ZERO
        }),
        mayhem_cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            ..ManaCost::ZERO
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const X_WITH_ADDITIONAL_COST: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfMayhemPaid {
                paid: true,
                skip: 1,
            },
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::ChosenX),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Discard,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        additional_mana_alternative: Some(ManaCost {
            generic: 2,
            ..ManaCost::ZERO
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MAYHEM_LAND: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersFromGraveyard,
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseLife(PlayerSet::Controller, 2)],
    }],
    extras: &CardExtras {
        mayhem_land: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const CAST_ENTRY: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCastFromYourGraveyard,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
    }],
    ..CardRules::EMPTY
};
const RETURN_PERMANENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 0,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::ReturnToBattlefield],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap_or_else(|| panic!("unknown test card {name}"))
}

fn discard(game: &mut Game, player: Player, card: CardId) {
    let source = put(game, player, "Masked Meower", Zone::Battlefield);
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = game.object_ref(card);
    act(game, Action::ChooseAdditionalCost { object });
    pay(game);
    settle(game);
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
}

fn granted(card: CardId, face: CastFace) -> Action {
    Action::AnnounceGrantedMayhem {
        card,
        face,
        flash_surcharge: false,
    }
}

#[test]
fn printed_hybrid_uses_both_colors_and_owned_cost_branches() {
    let _rules = fixture_rules::activate(&[(card("Opt"), &HYBRID)]);
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        let id = put(&mut parent, player, "Opt", Zone::Hand);
        let unrelated = put(&mut parent, player, "Opt", Zone::Graveyard);
        discard(&mut parent, player, id);
        reject(&mut parent, Action::AnnounceMayhem { card: unrelated });
        parent.state.players[player.index()].mana = [0, 0, 1, 1, 0, 0];
        act(&mut parent, Action::AnnounceMayhem { card: id });
        let saved = parent.snapshot();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut restored = parent.clone();
        drop(parent);

        let first = Action::ChooseSpellCost { cost: 0 };
        let second = Action::ChooseSpellCost { cost: 1 };
        let second_index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == second)
            .unwrap();
        let revision = indexed.revision();
        assert_eq!(
            indexed.apply(player.other(), second),
            Err(Error::WrongActor)
        );
        assert_eq!(indexed.snapshot(), saved);
        indexed.apply_index(player, revision, second_index).unwrap();
        act(&mut raw, first.clone());
        reject(&mut raw, Action::PayCosts { payment: [0; 6] });
        for game in [&mut raw, &mut indexed] {
            pay(game);
            assert_eq!(
                game.state.stack.last().unwrap().cast.unwrap().method,
                CastMethod::Mayhem
            );
            settle(game);
            assert_eq!(game.state.players[player.index()].life, 23);
        }
        assert_eq!(raw.state.players[player.index()].mana, [0, 0, 0, 1, 0, 0]);
        assert_eq!(
            indexed.state.players[player.index()].mana,
            [0, 0, 1, 0, 0, 0]
        );
        restored.restore(&saved).unwrap();
        act(&mut restored, first);
        pay(&mut restored);
        settle(&mut restored);
        assert_eq!(restored.snapshot(), raw.snapshot());
        reject(&mut raw, Action::AnnounceMayhem { card: id });
    }
}

#[test]
fn granted_mayhem_tracks_discard_before_provider_and_locks_x_and_additional_cost() {
    let _rules = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GRANT),
        (card("Exsanguinate"), &X_WITH_ADDITIONAL_COST),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let id = put(&mut game, player, "Exsanguinate", Zone::Hand);
    let unrelated = put(&mut game, player, "Exsanguinate", Zone::Graveyard);
    discard(&mut game, player, id);
    let method = granted(id, CastFace::Normal);
    reject(&mut game, method.clone());
    let provider = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    reject(&mut game, granted(unrelated, CastFace::Normal));
    assert!(game.legal_actions().contains(&method));

    let mut departed = game.clone();
    departed.move_object(provider, Zone::Graveyard);
    reject(&mut departed, method.clone());
    let mut reentered = game.clone();
    reentered.move_object(id, Zone::Exile);
    reentered.move_object(id, Zone::Graveyard);
    reject(&mut reentered, method.clone());

    let mut fee = game.clone();
    for (pay_fee, child) in [(false, &mut game), (true, &mut fee)] {
        child.state.players[player.index()].mana = [0, 0, 2, 0, 0, 3];
        let hand_before = child.hand_count(player);
        act(child, method.clone());
        act(child, Action::ChooseX { value: 3 });
        // Printed {X}{B}{B} plus optional {2}, less the provider's {2}.
        act(
            child,
            Action::ChooseSpellCost {
                cost: u16::from(pay_fee),
            },
        );
        if !pay_fee {
            let reference = child.observe(player).additional_cost.unwrap().candidates[0];
            assert!(
                child
                    .observe(player.other())
                    .additional_cost
                    .is_none_or(|choice| { choice.candidates.is_empty() })
            );
            act(child, Action::ChooseAdditionalCost { object: reference });
        }
        pay(child);
        let item = child.state.stack.last().unwrap();
        assert_eq!(item.cast.unwrap().method, CastMethod::GrantedMayhem);
        assert_eq!(item.cast.unwrap().origin, Zone::Graveyard);
        assert_eq!(item.x, Some(3));
        assert_eq!(
            child.state.players[player.index()].mana[5],
            if pay_fee { 0 } else { 2 }
        );
        assert_eq!(
            child.hand_count(player),
            hand_before - usize::from(!pay_fee)
        );
        settle(child);
        assert_eq!(child.state.players[player.index()].life, 23);
    }
}

#[test]
fn mayhem_land_consumes_land_play_and_entry_uses_actual_origin() {
    let _rules = fixture_rules::activate(&[
        (card("Evolving Wilds"), &MAYHEM_LAND),
        (card("Murder"), &RETURN_PERMANENT),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let id = put(&mut game, player, "Evolving Wilds", Zone::Hand);
    let other_land = put(&mut game, player, "Evolving Wilds", Zone::Graveyard);
    let hand_land = put(&mut game, player, "Evolving Wilds", Zone::Hand);
    discard(&mut game, player, id);
    reject(&mut game, Action::PlayLand { card: other_land });

    let mut alternate = game.clone();
    act(&mut alternate, Action::PlayLand { card: hand_land });
    assert_eq!(alternate.state.players[player.index()].life, 20);
    reject(&mut alternate, Action::PlayLand { card: id });

    act(&mut game, Action::PlayLand { card: id });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 18);
    assert!(game.object_view(id).tapped);
    reject(&mut game, Action::PlayLand { card: hand_land });
    let spell = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(other_land)));
    settle(&mut game);
    assert_eq!(game.object_view(other_land).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].life, 16);
}

#[test]
fn origin_discount_and_cast_entry_apply_to_independent_graveyard_permission() {
    let _rules = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GRANT),
        (card("Mazemind Tome"), &CAST_ENTRY),
        (card("Murder"), &RETURN_PERMANENT),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let from_grave = put(&mut game, player, "Mazemind Tome", Zone::Graveyard);
    let direct_return = put(&mut game, player, "Mazemind Tome", Zone::Graveyard);
    let from_hand = put(&mut game, player, "Mazemind Tome", Zone::Hand);
    reject(&mut game, granted(from_grave, CastFace::Normal));
    let object = game.object_ref(from_grave);
    game.grant_graveyard_permission(player, object, GraveyardCastKind::Cast);
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, Action::AnnounceCast { card: from_hand });

    act(&mut game, Action::AnnounceCast { card: from_grave });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 24);
    assert_eq!(game.object_view(from_grave).zone, Zone::Battlefield);

    game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 1];
    let spell = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(direct_return)));
    settle(&mut game);
    assert_eq!(game.object_view(direct_return).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].life, 24);
}

#[test]
fn granted_mayhem_prices_the_adventure_face_of_a_discarded_land() {
    let _rules = fixture_rules::activate(&[(card("Swiftfoot Boots"), &GRANT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let id = put(&mut game, player, "Ishgard, the Holy See", Zone::Hand);
    discard(&mut game, player, id);
    reject(&mut game, Action::PlayLand { card: id });
    reject(&mut game, granted(id, CastFace::Normal));

    game.state.players[player.index()].mana = [2, 0, 0, 0, 0, 0];
    let adventure = granted(id, CastFace::Adventure);
    reject(&mut game, adventure.clone());
    game.state.players[player.index()].mana[5] = 1;
    act(&mut game, adventure);
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().face,
        CastFace::Adventure
    );
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(&mut game);
    assert_eq!(game.object_view(id).zone, Zone::Exile);
    assert!(
        game.legal_actions()
            .contains(&Action::PlayLand { card: id })
    );
}
