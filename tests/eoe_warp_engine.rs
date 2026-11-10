//! Warp's selected alternative, payment and exact-object delayed permission.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const RED: ManaCost = ManaCost {
    colored: [0, 0, 0, 1, 0, 0],
    generic: 0,
};
const BLUE: ManaCost = ManaCost {
    colored: [0, 1, 0, 0, 0, 0],
    generic: 0,
};
const BLACK: ManaCost = ManaCost {
    colored: [0, 0, 1, 0, 0, 0],
    generic: 0,
};
const GREEN: ManaCost = ManaCost {
    colored: [0, 0, 0, 0, 1, 0],
    generic: 0,
};
const GRANTED: ManaCost = ManaCost { generic: 2, ..RED };
const X_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    extras: &CardExtras {
        x_symbols: 1,
        x_entry_counters: true,
        warp: Some(&WarpSpec {
            x_symbols: 1,
            ..WarpSpec::fixed(GREEN)
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const HYBRID: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    extras: &CardExtras {
        warp: Some(&WarpSpec {
            alternatives: &[RED],
            ..WarpSpec::fixed(BLUE)
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const HYBRID_WITH_FEE: CardRules = CardRules {
    extras: &CardExtras {
        warp: Some(&WarpSpec {
            alternatives: &[RED],
            ..WarpSpec::fixed(BLUE)
        }),
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: SacrificeFilter {
                kinds: 1,
                other: true,
                keywords: Keywords::NONE,
                source_only: false,
                nonland: false,
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
    ..HYBRID
};
const LIFE: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    extras: &CardExtras {
        warp: Some(&WarpSpec {
            life: 2,
            from_graveyard: true,
            ..WarpSpec::fixed(BLACK)
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::GrantHandWarp {
            filter: CastFilter::AnyOf(&[
                CastFilter::Artifact,
                CastFilter::Colors {
                    filter: &CastFilter::Creature,
                    mask: 8,
                },
            ]),
            cost: GRANTED,
        }],
    }],
    ..CardRules::EMPTY
};
const ARTIFACT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        x_symbols: 2,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};
const COUNTER_ABILITY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::TriggeredAbility {
            relation: -1,
        })),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};

fn advance_until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if done(g) {
            return;
        }
        choose(g, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("advance budget exhausted at {:?}", g.state.step);
}

fn clause(g: &mut Game, wanted: CastCostClause) {
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && row.clause == wanted)
        .unwrap();
    act(
        g,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}

fn granted(card: CardId) -> Action {
    Action::AnnounceGrantedWarp {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
        cost: GRANTED,
    }
}

#[test]
fn warp_x_payment_preserves_printed_mana_value_and_exact_exile_recast() {
    let name = "Mutagen Man, Living Ooze";
    let _rules = fixture_rules::activate(&[(Card::from_name(name).unwrap(), &X_CREATURE)]);
    for p in [Player::P0, Player::P1] {
        let mut game = scenario(p);
        let card = put(&mut game, p, name, Zone::Hand);
        game.state.players[p.index()].mana = [0, 0, 0, 0, 3, 0];
        act(&mut game, Action::AnnounceWarp { card });
        assert_eq!(
            game.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 0,
                maximum: 2,
            }]
        );
        assert!(game.observe(p.other()).x_choice.is_none());
        reject(&mut game, Action::ChooseX { value: 3 });
        let action = Action::ChooseX { value: 2 };
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let revision = game.revision();
        let snapshot = game.snapshot();
        let mut raw = game.clone();
        let mut indexed = game.clone();
        drop(game);
        assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
        assert_eq!(raw.snapshot(), snapshot);
        act(&mut raw, action.clone());
        indexed.apply_index(p, revision, index).unwrap();
        for child in [&mut raw, &mut indexed] {
            pay(child);
            let cast = child.state.stack.last().unwrap();
            assert_eq!(cast.cast.unwrap().method, CastMethod::Warp);
            assert_eq!(cast.cast.unwrap().payment, [0, 0, 0, 0, 3, 0]);
            assert_eq!(cast.mana_value(), 4);
            settle(child);
            assert_eq!(child.object_view(card).counters.plus_one, 2);
            assert!(child.object_view(card).cast_for_warp);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&snapshot).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        pay(&mut indexed);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());

        advance_until(&mut raw, |g| g.object_view(card).zone == Zone::Exile);
        let permission = raw.observe(p.other()).warped_cards[0];
        assert_eq!(permission.object, raw.object_ref(card));
        assert_eq!(permission.owner, p);
        assert_eq!(permission.exiled_turn, raw.state.turn);
        raw.state.players[p.index()].mana = [0, 0, 0, 0, 3, 0];
        reject(&mut raw, Action::AnnounceExile { card });
        let exile_turn = raw.state.turn;
        advance_until(&mut raw, |g| {
            g.state.turn > exile_turn && g.state.active == p && g.state.step == Step::Main1
        });
        raw.state.players[p.index()].mana = [0, 0, 0, 0, 3, 0];
        reject(&mut raw, Action::AnnounceWarp { card });
        act(&mut raw, Action::AnnounceExile { card });
        act(&mut raw, Action::ChooseX { value: 1 });
        pay(&mut raw);
        assert_eq!(
            raw.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Normal
        );
        settle(&mut raw);
        assert_eq!(raw.object_view(card).zone, Zone::Battlefield);
        assert_eq!(raw.object_view(card).counters.plus_one, 1);
        assert!(!raw.object_view(card).cast_for_warp);
        assert!(raw.observe(p).warped_cards.is_empty());
    }
}

#[test]
fn hybrid_warp_announces_a_fundable_colored_equivalent() {
    let _rules = fixture_rules::activate(&[(Card::GoblinPiker, &HYBRID)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let card = put(&mut game, p, "Goblin Piker", Zone::Hand);
    game.state.players[p.index()].mana = [0, 1, 0, 1, 0, 0];
    act(&mut game, Action::AnnounceWarp { card });
    for (index, cost) in [BLUE, RED].into_iter().enumerate() {
        let mut child = game.clone();
        act(&mut child, Action::ChooseSpellCost { cost: index as u16 });
        pay(&mut child);
        let cast = child.state.stack.last().unwrap();
        assert_eq!(cast.cast.unwrap().payment, cost.colored);
        assert_eq!(cast.mana_value(), 2);
        settle(&mut child);
        assert!(child.object_view(card).cast_for_warp);
    }
    reject(&mut game, Action::ChooseSpellCost { cost: 2 });
}

#[test]
fn hybrid_warp_keeps_the_additional_cost_for_each_colored_equivalent() {
    let _rules = fixture_rules::activate(&[(Card::GoblinPiker, &HYBRID_WITH_FEE)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let card = put(&mut game, p, "Goblin Piker", Zone::Hand);
    let warp = Action::AnnounceWarp { card };
    game.state.players[p.index()].mana = RED.colored;
    reject(&mut game, warp.clone());

    let mut sacrifice = game.clone();
    let donor = put(&mut sacrifice, p, "Bear Cub", Zone::Battlefield);
    act(&mut sacrifice, warp.clone());
    act(&mut sacrifice, Action::ChooseSpellCost { cost: 1 });
    let object = sacrifice.object_ref(donor);
    act(&mut sacrifice, Action::ChooseAdditionalCost { object });
    pay(&mut sacrifice);
    settle(&mut sacrifice);
    assert_eq!(sacrifice.object_view(donor).zone, Zone::Graveyard);
    assert!(sacrifice.object_view(card).cast_for_warp);

    game.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
    act(&mut game, warp);
    reject(&mut game, Action::ChooseSpellCost { cost: 1 });
    act(&mut game, Action::ChooseSpellCost { cost: 3 });
    pay(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().payment,
        [0, 0, 0, 1, 0, 2]
    );
    settle(&mut game);
    assert!(game.object_view(card).cast_for_warp);
}

#[test]
fn warp_life_cost_and_graveyard_origin_are_part_of_the_same_cast() {
    let name = "Vampire Interloper";
    let _rules = fixture_rules::activate(&[(Card::from_name(name).unwrap(), &LIFE)]);
    for origin in [Zone::Hand, Zone::Graveyard] {
        let p = Player::P1;
        let mut game = scenario(p);
        let card = put(&mut game, p, name, origin);
        game.state.players[p.index()].mana = BLACK.colored;
        game.state.players[p.index()].life = 1;
        reject(&mut game, Action::AnnounceWarp { card });
        game.state.players[p.index()].life = 3;
        act(&mut game, Action::AnnounceWarp { card });
        act(&mut game, Action::BeginOrderedCosts);
        clause(&mut game, CastCostClause::Life(2));
        clause(&mut game, CastCostClause::Mana);
        pay(&mut game);
        let cast = game.state.stack.last().unwrap().cast.unwrap();
        assert_eq!(cast.method, CastMethod::Warp);
        assert_eq!(cast.origin, origin);
        assert_eq!(cast.payment, BLACK.colored);
        assert_eq!(game.state.players[p.index()].life, 1);
        settle(&mut game);
        assert!(game.object_view(card).cast_for_warp);
    }
}

#[test]
fn granted_warp_freezes_the_cost_and_resolves_with_printed_x_zero() {
    let marketback = Card::from_name("Marketback Walker").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PROVIDER),
        (Card::GoblinPiker, &HYBRID),
        (marketback, &ARTIFACT),
    ]);
    let p = Player::P0;
    let mut game = scenario(p);
    let provider = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let red = put(&mut game, p, "Goblin Piker", Zone::Hand);
    let artifact = put(&mut game, p, "Marketback Walker", Zone::Hand);
    let secret = put(&mut game, p, "Bear Cub", Zone::Hand);
    let off_zone = put(&mut game, p, "Goblin Piker", Zone::Graveyard);
    let removal = put(&mut game, p.other(), "Murder", Zone::Hand);
    let actions = game.legal_actions();
    assert!(actions.contains(&Action::AnnounceWarp { card: red }));
    assert!(actions.contains(&granted(red)));
    assert!(actions.contains(&granted(artifact)));
    reject(&mut game, granted(secret));
    reject(&mut game, granted(off_zone));
    assert!(
        !game
            .observe(p.other())
            .objects
            .iter()
            .any(|o| o.id == artifact)
    );

    act(&mut game, granted(artifact));
    assert!(game.x_choice().is_none());
    pay(&mut game);
    assert_eq!(game.state.stack.last().unwrap().x, Some(0));
    assert_eq!(game.state.stack.last().unwrap().mana_value(), 0);
    act(&mut game, Action::Pass);
    cast(&mut game, removal, Some(Target::Card(provider)));
    settle(&mut game);
    assert_eq!(game.object_view(provider).zone, Zone::Graveyard);
    assert!(game.object_view(artifact).cast_for_warp);
    reject(&mut game, granted(red));
    advance_until(&mut game, |g| g.object_view(artifact).zone == Zone::Exile);
    assert_eq!(
        game.observe(p).warped_cards[0].object,
        game.object_ref(artifact)
    );
}

#[test]
fn copied_warp_spell_exiles_only_if_the_paid_ability_was_copiable() {
    let copy = Card::from_name("Think Twice").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GoblinPiker, &HYBRID),
        (Card::GrizzlyBears, &PROVIDER),
        (copy, &COPY),
    ]);
    let p = Player::P0;
    for use_grant in [false, true] {
        let mut game = scenario(p);
        let card = put(&mut game, p, "Goblin Piker", Zone::Hand);
        let duplicator = put(&mut game, p, "Think Twice", Zone::Hand);
        put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
        if use_grant {
            act(&mut game, granted(card));
        } else {
            act(&mut game, Action::AnnounceWarp { card });
            act(&mut game, Action::ChooseSpellCost { cost: 1 });
        }
        pay(&mut game);
        cast(&mut game, duplicator, Some(Target::Spell(card)));
        settle(&mut game);
        let delayed = game.observe(p).delayed_warp_exiles;
        assert_eq!(delayed.len(), if use_grant { 1 } else { 2 });
        let token = game
            .observe(p)
            .objects
            .iter()
            .find(|object| {
                object.zone == Zone::Battlefield
                    && object.card == Card::GoblinPiker
                    && object.id != card
            })
            .unwrap()
            .id;
        assert!(!game.object_view(token).cast_for_warp);
        advance_until(&mut game, |g| {
            g.object_view(card).zone == Zone::Exile
                && g.state.stack.is_empty()
                && g.state.pending == Pending::Priority
        });
        assert_eq!(game.on_battlefield(token), use_grant);
        let permissions = game.observe(p).warped_cards;
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].object, game.object_ref(card));
    }
}

#[test]
fn countering_either_warp_spell_or_delayed_trigger_creates_no_recast_permission() {
    let counter = Card::from_name("Think Twice").unwrap();
    let _rules =
        fixture_rules::activate(&[(Card::GoblinPiker, &HYBRID), (counter, &COUNTER_ABILITY)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let card = put(&mut game, p, "Goblin Piker", Zone::Hand);
    let cancel = put(&mut game, p.other(), "Cancel", Zone::Hand);
    let stifle = put(&mut game, p.other(), "Think Twice", Zone::Hand);
    act(&mut game, Action::AnnounceWarp { card });
    act(&mut game, Action::ChooseSpellCost { cost: 1 });
    pay(&mut game);

    let mut countered = game.clone();
    act(&mut countered, Action::Pass);
    cast(&mut countered, cancel, Some(Target::Spell(card)));
    settle(&mut countered);
    assert_eq!(countered.object_view(card).zone, Zone::Graveyard);
    assert!(countered.observe(p).delayed_warp_exiles.is_empty());
    assert!(countered.observe(p).warped_cards.is_empty());

    settle(&mut game);
    advance_until(&mut game, |g| {
        g.state
            .stack
            .last()
            .is_some_and(|item| item.kind == StackKind::DelayedWarpExile)
    });
    if game.actor() != Some(p.other()) {
        act(&mut game, Action::Pass);
    }
    let trigger = game.state.stack.last().unwrap().id;
    game.state.players[p.other().index()].mana = [0, 2, 0, 0, 0, 0];
    cast(&mut game, stifle, Some(Target::Stack(trigger)));
    settle(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    assert!(game.observe(p).warped_cards.is_empty());
}
