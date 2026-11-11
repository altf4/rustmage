//! Mana source rules observe the committed payment, not a possible funding plan.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WHITE_TWO: Mana = [2, 0, 0, 0, 0, 0];

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

const fn producer(tap: bool, sacrifice: bool) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap,
            sacrifice,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(WHITE_TWO))],
        sorcery: false,
        once_per_turn: false,
    }
}

const SOURCE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[producer(true, false)],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCastUsingSourceMana(CastFilter::AnyOf(&[
            CastFilter::Artifact,
            CastFilter::Creature,
        ])),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::CastSpellManaValue,
        )],
    }],
    ..CardRules::EMPTY
};

fn ready(game: &mut Game, goal: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if goal(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::PayCosts { .. }
            )
        });
    }
    panic!("source mana scenario did not reach its next decision");
}

fn quiet(game: &mut Game) {
    ready(game, |game| {
        game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending_triggers.is_empty()
    });
}

fn generate(game: &mut Game, source: CardId) {
    choose(
        game,
        |action| matches!(action, Action::ActivateMana { card, production, .. } if *card == source && *production == WHITE_TWO),
    );
}

#[test]
fn source_trigger_uses_the_chosen_lot_and_replays_the_payment_without_duplicating_it() {
    let _fixture = fixture_rules::activate(&[(card("Short Bow"), &SOURCE)]);
    let player = Player::P1;
    let mut parent = scenario(player);
    parent.state.players[player.index()].mana = [1, 0, 0, 0, 0, 0];
    let source = put(&mut parent, player, "Short Bow", Zone::Battlefield);
    let source_ref = parent.object_ref(source);
    generate(&mut parent, source);
    let spell = put(&mut parent, player, "Healer's Hawk", Zone::Hand);
    act(&mut parent, Action::AnnounceCast { card: spell });
    pay(&mut parent);
    assert_eq!(parent.state.pending, Pending::ManaLotSpend { player });
    assert!(parent.state.pending_triggers.is_empty());
    assert_eq!(
        parent.state.players[player.index()].spells_cast_this_turn,
        0
    );
    let chosen = Action::ChooseManaLotSpend { amount: 1 };
    let ordinary = Action::ChooseManaLotSpend { amount: 0 };
    assert!(parent.legal_actions().contains(&ordinary));
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == chosen)
        .unwrap();
    let revision = parent.revision();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut unspent = parent.clone();
    drop(parent);

    act(&mut raw, chosen.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let cast = raw
        .state
        .stack
        .iter()
        .find(|packet| packet.kind == StackKind::Spell)
        .unwrap();
    assert_eq!(
        cast.mana_provenance.as_ref().unwrap().sources.as_slice(),
        &[ManaSourcePayment {
            source: source_ref,
            amount: 1,
            cave: false
        }]
    );
    quiet(&mut raw);
    quiet(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.players[player.index()].life, 21);
    assert_eq!(raw.observe(player).mana_lots[0].amounts[0], 1);

    act(&mut unspent, ordinary);
    quiet(&mut unspent);
    assert_eq!(unspent.state.players[player.index()].life, 20);
    assert_eq!(unspent.observe(player).mana_lots[0].amounts[0], 2);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, chosen);
    quiet(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn mana_activated_during_announcement_triggers_once_for_two_units_and_not_for_an_instant() {
    let _fixture = fixture_rules::activate(&[(card("Short Bow"), &SOURCE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    let source = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, player, "Bishop's Soldier", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    generate(&mut game, source);
    pay(&mut game);
    quiet(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);

    game.untap_objects(&[source], untap_rules::UntapCause::Effect);
    let instant = put(&mut game, player, "Disenchant", Zone::Hand);
    generate(&mut game, source);
    cast(&mut game, instant, Some(Target::Card(source)));
    quiet(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn producer_identity_survives_an_acquired_ability_and_rejects_a_new_incarnation() {
    let _fixture = fixture_rules::activate(&[(card("Short Bow"), &SOURCE)]);
    let player = Player::P0;
    for blink in [false, true] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let source = put(&mut game, player, "Plains", Zone::Battlefield);
        let old_ref = game.object_ref(source);
        act(&mut game, Action::TapMana { card: source });
        assert_eq!(game.mana_lots(player)[0].quality.producer, Some(old_ref));
        if blink {
            game.move_object(source, Zone::Hand);
            game.move_object(source, Zone::Battlefield);
        }
        let current = game.object_ref(source);
        game.apply_temporary_copy(
            current,
            CopyRecipe::new(card("Short Bow")),
            CopyDeadline::EndOfTurn,
        );
        let spell = put(&mut game, player, "Healer's Hawk", Zone::Hand);
        cast(&mut game, spell, None);
        quiet(&mut game);
        assert_eq!(
            game.state.players[player.index()].life,
            if blink { 20 } else { 21 }
        );
    }
}

const CAVE: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[producer(false, true)],
    ..CardRules::EMPTY
};
const CAVE_ENTRY: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::CaveManaSpent,
        )],
    }],
    ..CardRules::EMPTY
};

#[test]
fn self_entry_counts_each_cave_unit_spent_even_after_the_producer_is_sacrificed() {
    let _fixture = fixture_rules::activate(&[
        (card("Captivating Cave"), &CAVE),
        (card("Goldvein Pick"), &CAVE_ENTRY),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    let cave = put(&mut game, player, "Captivating Cave", Zone::Battlefield);
    let reference = game.object_ref(cave);
    generate(&mut game, cave);
    assert_eq!(game.object_view(cave).zone, Zone::Graveyard);
    let pool = game.observe(player).mana_lots;
    assert_eq!(pool[0].producer, Some(reference));
    assert!(pool[0].cave_source);
    assert_eq!(pool[0].amounts, WHITE_TWO);
    let spell = put(&mut game, player, "Goldvein Pick", Zone::Hand);
    cast(&mut game, spell, None);
    quiet(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].life, 22);

    game.move_object(spell, Zone::Hand);
    game.move_object(spell, Zone::Battlefield);
    quiet(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
}

const COPY_SOURCE: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCastUsingSourceMana(CastFilter::Permanent),
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: 0,
                other: true,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::CopyTargetsFromCastSpell],
    }],
    ..SOURCE
};

#[test]
fn cast_mana_observer_copies_frozen_spell_characteristics_after_a_counter() {
    let _fixture = fixture_rules::activate(&[(card("Short Bow"), &COPY_SOURCE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    let source = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let recipient = put(&mut game, player, "Forest", Zone::Battlefield);
    let reference = game.object_ref(recipient);
    let response = put(&mut game, player.other(), "Essence Scatter", Zone::Hand);
    let spell = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    generate(&mut game, source);
    cast(&mut game, spell, None);
    ready(&mut game, |game| {
        matches!(game.state.pending, Pending::Targets { .. })
    });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    ready(&mut game, |game| {
        game.state.pending == Pending::Priority
            && game.actor() == Some(player.other())
            && game.state.stack.len() == 2
    });
    cast(&mut game, response, Some(Target::Spell(spell)));
    quiet(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    let copy = game.object_view(recipient);
    assert_eq!(copy.reference, reference);
    assert_eq!(copy.card, card("Healer's Hawk"));
    assert_eq!(stats(&game, recipient), (1, 1));
    assert!(copy.keywords.contains(Keywords::FLYING));
    assert!(game.expire_temporary_copies(true));
    assert_eq!(game.object_view(recipient).card, Card::Forest);
}

#[test]
fn neutral_sources_spend_automatically_without_exposing_a_hidden_observer() {
    let _fixture = fixture_rules::activate(&[(card("Short Bow"), &SOURCE)]);
    let p = Player::P0;
    let mut worlds = Vec::new();
    for hidden in ["Short Bow", "Goldvein Pick"] {
        let mut g = scenario(p);
        g.state.players[p.index()].mana = [0; 6];
        for _ in 0..2 {
            let land = put(&mut g, p, "Plains", Zone::Battlefield);
            act(&mut g, Action::TapMana { card: land });
        }
        put(&mut g, p.other(), hidden, Zone::Hand);
        assert_eq!(g.mana_lots(p).len(), 2);
        let spell = put(&mut g, p, "Healer's Hawk", Zone::Hand);
        cast(&mut g, spell, None);
        assert_ne!(g.state.pending, Pending::ManaLotSpend { player: p });
        assert_eq!(g.mana_lots(p).len(), 1);
        assert!(g.observe(p).mana_lots.is_empty());
        worlds.push(serde_json::to_value(g.observe(p)).unwrap());
    }
    assert_eq!(worlds[0], worlds[1]);
}

const PAID_ABILITY: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        ..producer(false, false)
    }],
    ..CardRules::EMPTY
};

#[test]
fn a_public_source_remains_a_choice_when_paying_an_unrelated_activated_ability() {
    let _fixture = fixture_rules::activate(&[
        (card("Short Bow"), &SOURCE),
        (card("Goldvein Pick"), &PAID_ABILITY),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    g.state.players[p.index()].mana = [0; 6];
    let land = put(&mut g, p, "Plains", Zone::Battlefield);
    let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
    let payer = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
    act(&mut g, Action::TapMana { card: land });
    generate(&mut g, source);
    act(
        &mut g,
        Action::Activate {
            card: payer,
            ability: 0,
            target: None,
            payment: [1, 0, 0, 0, 0, 0],
            counter_payment: Counters::default(),
        },
    );
    let range = g.observe(p).mana_lot_spend.unwrap();
    assert_eq!((range.minimum, range.maximum), (0, 1));
    assert_eq!(g.observe(p).mana_lots[0].lot, range.lot);
    assert_eq!(
        g.observe(p).mana_lots[0].producer,
        Some(g.object_ref(source))
    );
    assert!(g.observe(p.other()).mana_lot_spend.is_none());
    act(&mut g, Action::ChooseManaLotSpend { amount: 0 });
    quiet(&mut g);
    assert_eq!(g.observe(p).mana_lots[0].amounts, WHITE_TWO);
}

#[test]
fn a_cave_payment_branches_after_public_announcement_and_survives_unrelated_spending() {
    let _fixture = fixture_rules::activate(&[
        (card("Captivating Cave"), &CAVE),
        (card("Goldvein Pick"), &CAVE_ENTRY),
    ]);
    let p = Player::P0;
    let mut parent = scenario(p);
    parent.state.players[p.index()].mana = WHITE_TWO;
    let source = put(&mut parent, p, "Captivating Cave", Zone::Battlefield);
    let spell = put(&mut parent, p, "Goldvein Pick", Zone::Hand);
    let unrelated = put(&mut parent, p, "Short Bow", Zone::Hand);
    generate(&mut parent, source);
    let mut preserve = parent.clone();
    cast(&mut preserve, unrelated, None);
    assert_ne!(preserve.state.pending, Pending::ManaLotSpend { player: p });
    assert_eq!(preserve.observe(p).mana_lots[0].amounts, WHITE_TWO);

    assert!(
        !parent
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Cast { card, .. } if *card == spell))
    );
    act(&mut parent, Action::AnnounceCast { card: spell });
    assert_eq!(
        parent
            .observe(p.other())
            .announcing
            .as_ref()
            .unwrap()
            .source
            .unwrap()
            .card,
        card("Goldvein Pick")
    );
    pay(&mut parent);
    for amount in [0, 1, 2] {
        let mut branch = parent.clone();
        act(&mut branch, Action::ChooseManaLotSpend { amount });
        quiet(&mut branch);
        assert_eq!(
            branch.state.players[p.index()].life,
            20 + i128::from(amount)
        );
    }
}

const TAGGED: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed(WHITE_TWO),
            quality: &ManaQuality {
                restriction: ManaRestriction::CreatureSpells,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        ..producer(true, false)
    }],
    ..CardRules::EMPTY
};

#[test]
fn equivalent_tagged_producers_share_one_spending_choice_and_keep_exact_receipts() {
    let _fixture = fixture_rules::activate(&[(card("Short Bow"), &TAGGED)]);
    let p = Player::P0;
    for ordinary in [0, 1] {
        let mut g = scenario(p);
        g.state.players[p.index()].mana = [ordinary, 0, 0, 0, 0, 0];
        let first = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let second = put(&mut g, p, "Short Bow", Zone::Battlefield);
        generate(&mut g, first);
        generate(&mut g, second);
        let lots = g.observe(p).mana_lots;
        assert_eq!(lots.len(), 1);
        assert_eq!(lots[0].amounts[0], 4);
        assert_eq!(lots[0].restriction, ManaRestrictionView::CreatureSpells);
        assert_eq!(g.mana_lots(p).len(), 2);
        let spell = put(&mut g, p, "Healer's Hawk", Zone::Hand);
        cast(&mut g, spell, None);
        assert_eq!(
            matches!(g.state.pending, Pending::ManaLotSpend { .. }),
            ordinary > 0
        );
        if ordinary > 0 {
            let range = g.observe(p).mana_lot_spend.unwrap();
            assert_eq!((range.minimum, range.maximum), (0, 1));
            act(&mut g, Action::ChooseManaLotSpend { amount: 1 });
        }
        let paid = g
            .state
            .stack
            .last()
            .unwrap()
            .mana_provenance
            .as_ref()
            .unwrap();
        assert_eq!(
            paid.sources.as_slice(),
            &[ManaSourcePayment {
                source: g.object_ref(second),
                amount: 1,
                cave: false,
            }]
        );
        assert_eq!(g.observe(p).mana_lots[0].amounts[0], 3);
    }
}
