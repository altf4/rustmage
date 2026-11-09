//! A legendary creature sacrifice is a live actual and projected cost.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const FEE: Mana = [0, 1, 0, 0, 0, 2];

const COUNTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
            TargetRule::Ability,
            TargetRule::SpellKinds {
                kinds: 0,
                excluded: 1,
            },
        ]))),
        &[Instruction::Counter],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::FilteredCost {
                action: CostAction::Sacrifice,
                any: &[CostMatch::Legendary],
            },
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
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

const LEGENDARY_MANA_AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::AttachedLegendary],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(BLUE))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn legendary_recipe(card: Card) -> CopyRecipe {
    let mut recipe = CopyRecipe::new(card);
    recipe.overrides = Some(CopyOverrides {
        legendary: Some(true),
        ..CopyOverrides::NONE
    });
    recipe
}

fn make_legendary(game: &mut Game, card: CardId) {
    game.apply_temporary_copy(
        game.object_ref(card),
        legendary_recipe(game.object_view(card).card),
        CopyDeadline::EndOfTurn,
    );
}

fn target_spell(game: &mut Game, player: Player) -> CardId {
    let spell = put(game, player.other(), "Opt", Zone::Hand);
    act(game, Action::Pass);
    cast(game, spell, None);
    act(game, Action::Pass);
    assert_eq!(game.actor(), Some(player));
    spell
}

fn announce(game: &mut Game, card: CardId, cost: u16, target: Target) {
    act(game, Action::AnnounceCast { card });
    assert!(matches!(game.state.pending, Pending::SpellCost { .. }));
    act(game, Action::ChooseSpellCost { cost });
    act(game, Action::ChooseTarget { target });
}

fn select(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

#[test]
fn the_sacrifice_route_checks_live_type_supertype_and_control() {
    let _rules = fixture_rules::activate(&[(named("Spell Pierce"), &COUNTER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let controlled = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let plain = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Llanowar Elves",
        Zone::Battlefield,
    );
    for id in [controlled, artifact, opposing] {
        make_legendary(&mut game, id);
    }
    game.record_control(game.object_ref(controlled), player, None);
    let token = game.state.objects.len() as CardId;
    game.stage_copy_creation(player, legendary_recipe(named("Bear Cub")), 1);
    assert!(!game.advance_entries());
    assert!(game.object_view(token).is_token() && game.object_view(token).is_legendary());

    let spell = put(&mut game, player, "Spell Pierce", Zone::Hand);
    let victim = target_spell(&mut game, player);
    game.state.players[player.index()].mana = BLUE;
    announce(&mut game, spell, 0, Target::Spell(victim));
    for invalid in [plain, artifact, opposing] {
        let action = select(&game, invalid);
        reject(&mut game, action);
    }
    assert!(game.legal_actions().contains(&select(&game, token)));
    let action = select(&game, controlled);
    let revision = game.revision();
    let before = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&before).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut game, Action::PayCosts { payment: BLUE });
    assert_eq!(game.object_view(controlled).zone, Zone::Graveyard);
    assert_eq!(game.object_view(controlled).owner, player.other());
    assert_eq!(game.object_view(token).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
}

#[test]
fn a_mana_payment_cannot_consume_its_only_legendary_grant() {
    let serum = named("Super-Soldier Serum");
    let _rules = fixture_rules::activate(&[
        (named("Spell Pierce"), &COUNTER),
        (serum, &LEGENDARY_MANA_AURA),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let aura = spawn(&mut game, player, serum, Zone::Battlefield);
    game.state.objects[aura as usize].attached_to = Some(game.object_ref(creature));
    assert!(game.object_view(creature).is_legendary());
    let spell = put(&mut game, player, "Spell Pierce", Zone::Hand);
    let victim = target_spell(&mut game, player);
    game.state.players[player.index()].mana = [0; 6];

    // This Aura can pay U by sacrificing itself, but that would remove the
    // only legendary creature witness from the resulting funding world.
    reject(&mut game, Action::AnnounceCast { card: spell });
    let mut primary = game.clone();
    let land = put(&mut primary, player, "Island", Zone::Battlefield);
    announce(&mut primary, spell, 0, Target::Spell(victim));
    let selected = select(&primary, creature);
    act(&mut primary, selected);
    assert!(!primary.legal_actions().iter().any(|action| {
        matches!(action, Action::ActivateMana { card, .. } | Action::AnnounceMana { card, .. }
            if *card == aura)
    }));
    act(&mut primary, Action::TapMana { card: land });
    act(&mut primary, Action::PayCosts { payment: BLUE });
    assert_eq!(primary.object_view(creature).zone, Zone::Graveyard);
    settle(&mut primary);

    game.move_object(aura, Zone::Graveyard);
    game.state.players[player.index()].mana = FEE;
    announce(&mut game, spell, 1, Target::Spell(victim));
    act(&mut game, Action::PayCosts { payment: FEE });
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}
