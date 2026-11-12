//! Mandatory mana-or-object costs keep their own identity across cast routes.
mod support;

use rustmage::AdditionalCostRoute;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use std::num::NonZeroU32;
use support::*;

const RED: Mana = [0, 0, 0, 1, 0, 0];
const WUBRG: Mana = [1, 1, 1, 1, 1, 0];
const FIVE_COLORS: ManaCost = ManaCost {
    colored: WUBRG,
    generic: 0,
};
const FEE: ManaCost = ManaCost {
    generic: 2,
    ..ManaCost::ZERO
};
const ADDITIONAL: CardExtras = CardExtras {
    additional_cost: Some(AdditionalCost {
        kind: EffectChoiceKind::Sacrifice,
        filter: SacrificeFilter {
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 1 | 32,
            nonland: false,
            other: false,
            tokens: None,
            artifact_enchantment_or_token: false,
            subtype: None,
        },
    }),
    additional_mana_alternative: Some(FEE),
    ..CardExtras::EMPTY
};
const SPELL: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Creature), &[Instruction::Damage(5)]),
    extras: &ADDITIONAL,
    ..CardRules::EMPTY
};
const KICKED_SPELL: CardRules = CardRules {
    extras: &CardExtras {
        kicker_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::ZERO
        }),
        ..ADDITIONAL
    },
    ..SPELL
};
const EXILE_ALTERNATIVES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::ExileCastForZeroOncePerTurn,
            StaticAbilityFragment::SpellAlternativeManaCost(FIVE_COLORS),
        ],
    }],
    ..CardRules::EMPTY
};
const EXILE_TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OpponentGraveyardOrExileSpellTax(2)],
    }],
    ..CardRules::EMPTY
};
const RESTRICTED_MANA: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed([0, 0, 0, 0, 0, 2]),
            quality: &ManaQuality {
                restriction: ManaRestriction::ArtifactSpell,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const DISCOUNT: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::InstantSorcery,
    amount: SpellReductionAmount::Fixed(3),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const REDUCER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, DISCOUNT)],
    }],
    extras: &CardExtras {
        cost_reductions: &[DISCOUNT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Spell),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn choose_route(game: &Game, route: AdditionalCostRoute) -> Action {
    let choice = game
        .observe(game.actor().unwrap())
        .cost_choices
        .into_iter()
        .find(|choice| choice.additional_cost_route == Some(route))
        .expect("fundable additional cost route");
    Action::ChooseSpellCost { cost: choice.index }
}

fn target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

fn finish_costs(game: &mut Game, payment: Mana, sacrifice: ObjectRef) {
    for _ in 0..12 {
        match game.state.pending {
            Pending::Priority => return,
            Pending::CastCosts { .. } => {
                choose(game, |action| {
                    matches!(action, Action::ChooseCastCost { .. })
                });
            }
            Pending::AdditionalCost { .. } => {
                act(game, Action::ChooseAdditionalCost { object: sacrifice });
            }
            Pending::PayCosts { .. } => act(game, Action::PayCosts { payment }),
            ref pending => panic!("unexpected cast choice {pending:?}"),
        }
    }
    panic!("cast did not finish");
}

#[test]
fn the_printed_cost_accepts_either_mandatory_branch_with_its_exact_payment() {
    let _rules = fixture_rules::activate(&[(card("Abrade"), &SPELL)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut parent = scenario(player);
    let spell = put(&mut parent, player, "Abrade", Zone::Hand);
    let victim = put(&mut parent, opponent, "Gigantosaurus", Zone::Battlefield);
    let enchantment = put(
        &mut parent,
        player,
        "Deeproot Pilgrimage",
        Zone::Battlefield,
    );
    let artifact = put(&mut parent, player, "Short Bow", Zone::Battlefield);
    let enchantment = parent.object_ref(enchantment);
    let artifact = parent.object_ref(artifact);
    parent.state.players[player.index()].mana = [0, 0, 0, 1, 0, 3];
    act(&mut parent, Action::AnnounceCast { card: spell });
    assert_eq!(parent.observe(player).cost_choices.len(), 2);
    assert!(parent.observe(opponent).cost_choices.is_empty());

    for route in [AdditionalCostRoute::Nonmana, AdditionalCostRoute::Mana] {
        let mut game = parent.clone();
        act(&mut game, choose_route(&parent, route));
        target(&mut game, victim);
        if route == AdditionalCostRoute::Nonmana {
            reject(&mut game, Action::ChooseAdditionalCost { object: artifact });
            let enemy = game.object_ref(victim);
            reject(&mut game, Action::ChooseAdditionalCost { object: enemy });
        }
        let mut payment = RED;
        payment[5] = if route == AdditionalCostRoute::Mana {
            3
        } else {
            1
        };
        finish_costs(&mut game, payment, enchantment);
        let cast = game.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.payment, payment);
        assert_eq!(cast.additional_cost_route, Some(route));
        assert_eq!(
            game.object_view(enchantment.card).zone,
            if route == AdditionalCostRoute::Mana {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        settle(&mut game);
        assert_eq!(game.object_view(victim).damage, 5);
    }
}

#[test]
fn restricted_mana_does_not_make_the_fee_route_payable() {
    let _rules = fixture_rules::activate(&[
        (card("Abrade"), &SPELL),
        (card("Short Bow"), &RESTRICTED_MANA),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Abrade", Zone::Hand);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 1];
    reject(&mut game, Action::AnnounceCast { card: spell });
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let sacrifice = game.object_ref(creature);
    act(&mut game, Action::AnnounceCast { card: spell });
    let choices = game.observe(player).cost_choices;
    assert_eq!(choices.len(), 1);
    assert_eq!(
        choices[0].additional_cost_route,
        Some(AdditionalCostRoute::Nonmana)
    );
    reject(&mut game, Action::ChooseSpellCost { cost: 1 });
    let action = choose_route(&game, AdditionalCostRoute::Nonmana);
    act(&mut game, action);
    target(&mut game, victim);
    finish_costs(&mut game, [0, 0, 0, 1, 0, 1], sacrifice);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
}

#[test]
fn the_additional_choice_combines_with_exile_alternatives_taxes_and_kicker() {
    let _rules = fixture_rules::activate(&[
        (card("Abrade"), &KICKED_SPELL),
        (card("Anthem of Champions"), &EXILE_ALTERNATIVES),
        (card("Short Bow"), &EXILE_TAX),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    for wubrg in [false, true] {
        for route in [AdditionalCostRoute::Nonmana, AdditionalCostRoute::Mana] {
            let mut game = scenario(player);
            let source = put(&mut game, player, "Anthem of Champions", Zone::Battlefield);
            let provider = game.object_ref(source);
            let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            let sacrifice = game.object_ref(creature);
            put(&mut game, opponent, "Short Bow", Zone::Battlefield);
            let victim = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
            let spell = put(&mut game, opponent, "Abrade", Zone::Exile);
            let object = game.object_ref(spell);
            let surcharge = NonZeroU32::new(1).unwrap();
            game.grant_taxed_exile_play(player, object, surcharge);
            let mut payment = if wubrg { WUBRG } else { [0; 6] };
            payment[5] = 4 + if route == AdditionalCostRoute::Mana {
                2
            } else {
                0
            };
            game.state.players[player.index()].mana = payment;
            let action = if wubrg {
                Action::AnnounceCastWithManaAlternative {
                    object,
                    provider,
                    face: CastFace::Normal,
                    cost: FIVE_COLORS,
                    flash_surcharge: false,
                    origin: PaidCastOrigin::TaxedExile { surcharge },
                }
            } else {
                Action::AnnounceExileForZero {
                    object,
                    provider,
                    face: CastFace::Normal,
                    flash_surcharge: false,
                    origin: ExileZeroOrigin::Taxed { surcharge },
                }
            };
            act(&mut game, action);
            act(&mut game, Action::ChooseKicker { pay: true });
            let action = choose_route(&game, route);
            act(&mut game, action);
            target(&mut game, victim);
            finish_costs(&mut game, payment, sacrifice);
            let cast = game.state.stack.last().unwrap().cast.unwrap();
            assert_eq!(cast.kicker, Some(true));
            assert_eq!(cast.payment, payment);
            assert_eq!(cast.additional_cost_route, Some(route));
            assert_eq!(cast.exile_play_surcharge, 1);
            assert_eq!(
                game.observe(player).exile_zero_uses.len(),
                usize::from(!wubrg)
            );
        }
    }
}

#[test]
fn a_life_base_still_requires_an_independent_mana_or_sacrifice_payment() {
    let _rules = fixture_rules::activate(&[(card("Abrade"), &SPELL)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player.other(), "Abrade", Zone::Exile);
    let object = game.object_ref(spell);
    let victim = put(
        &mut game,
        player.other(),
        "Gigantosaurus",
        Zone::Battlefield,
    );
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let sacrifice = game.object_ref(creature);
    game.grant_exile_play_permission_with_expiry(
        player,
        object,
        None,
        ExilePlayExpiry::WhileExiled,
    )
    .unwrap()
    .life_instead_of_mana = true;
    let payment = [0, 0, 0, 0, 0, 2];
    game.state.players[player.index()].mana = payment;
    act(
        &mut game,
        Action::AnnounceExileLife {
            card: spell,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    let action = choose_route(&game, AdditionalCostRoute::Mana);
    act(&mut game, action);
    target(&mut game, victim);
    finish_costs(&mut game, payment, sacrifice);
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.method, CastMethod::ExileLife);
    assert_eq!(cast.payment, payment);
    assert_eq!(cast.additional_cost_route, Some(AdditionalCostRoute::Mana));
    assert_eq!(game.state.players[player.index()].life, 18);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
}

#[test]
fn equal_locked_totals_keep_the_route_on_copies_and_across_rollback() {
    let _rules = fixture_rules::activate(&[
        (card("Abrade"), &SPELL),
        (card("Deeproot Pilgrimage"), &REDUCER),
        (card("Opt"), &COPY),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Abrade", Zone::Exile);
    let spell_ref = game.object_ref(spell);
    let reducer = put(&mut game, player, "Deeproot Pilgrimage", Zone::Battlefield);
    let sacrifice = game.object_ref(reducer);
    let victim = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let linked = put(&mut game, opponent, "Savannah Lions", Zone::Exile);
    game.state.exile_links.push(ExileLink {
        ability_origin: None,
        source: sacrifice,
        exiled: game.object_ref(linked),
        return_zone: Zone::Battlefield,
    });
    game.grant_exile_play_permission_with_expiry(
        player,
        spell_ref,
        None,
        ExilePlayExpiry::WhileExiled,
    )
    .unwrap();
    game.state.players[player.index()].mana = [0, 1, 0, 1, 0, 0];
    let before = game.snapshot();
    act(&mut game, Action::AnnounceExile { card: spell });
    let choosing = game.snapshot();
    let revision = game.revision();
    let nonmana = choose_route(&game, AdditionalCostRoute::Nonmana);
    let mana = choose_route(&game, AdditionalCostRoute::Mana);
    let index = game
        .legal_actions()
        .iter()
        .position(|choice| *choice == mana)
        .unwrap();
    let mut raw = game.clone();
    act(&mut raw, mana);
    let mut aborted = raw.clone();
    assert!(aborted.state.cast_rollback.is_some());
    aborted.finish_cast_rollback(false);
    assert_eq!(aborted.snapshot(), before);
    game.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), raw.snapshot());
    game.restore(&choosing).unwrap();
    assert_eq!(
        game.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut game, nonmana);

    for (route, branch) in [
        (AdditionalCostRoute::Mana, &mut raw),
        (AdditionalCostRoute::Nonmana, &mut game),
    ] {
        target(branch, victim);
        finish_costs(branch, RED, sacrifice);
        assert_eq!(
            branch.state.stack.last().unwrap().cast.unwrap().payment,
            RED
        );
        let copy = put(branch, player, "Opt", Zone::Hand);
        cast(branch, copy, Some(Target::Spell(spell)));
        act(branch, Action::Pass);
        act(branch, Action::Pass);
        let copied = branch.state.stack.last().unwrap();
        assert!(copied.is_copy);
        assert_eq!(copied.cast.unwrap().additional_cost_route, Some(route));
        assert_eq!(copied.cast.unwrap().payment, [0; 6]);
        settle(branch);
        assert_eq!(branch.object_view(victim).zone, Zone::Graveyard);
    }
    assert_eq!(raw.object_view(reducer).zone, Zone::Battlefield);
    assert_eq!(game.object_view(reducer).zone, Zone::Graveyard);
}
