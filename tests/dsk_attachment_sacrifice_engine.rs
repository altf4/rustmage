//! Nontargeted sacrifices with exact attachment exclusions.
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
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

const GEAR: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 2,
        toughness: 0,
    },
    keywords: Keywords::NONE,
};

const COMBAT_ATTACHMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::EquippedAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::SacrificeExceptAttackingAttachment,
                players: PlayerSet::Controller,
                filter: SacrificeFilter { other: true, ..ANY },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    extras: &CardExtras {
        equipment: Some(GEAR),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const STAY: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::PreventControlledPermanentGraveyardMove],
}];

const PREVENT_MOVE: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        static_ability_groups: STAY,
        ..CardExtras::EMPTY
    },
    static_groups: STAY,
    ..CardRules::EMPTY
};

const ENCHANT_CREATURE: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 0,
            toughness: 0,
        },
        keywords: Keywords::NONE,
    },
};

const AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        program: &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::SacrificeExceptSourceAttachment,
            players: PlayerSet::Targeted,
            filter: SacrificeFilter { kinds: 1, ..ANY },
            amount: 1,
            optional_skip: None,
        }],
    }],
    aura: Some(ENCHANT_CREATURE),
    extras: &CardExtras {
        aura: Some(ENCHANT_CREATURE),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn choose_object(game: &Game, id: CardId) -> Action {
    Action::ChooseEffectObject {
        object: game.object_ref(id),
    }
}

#[test]
fn attack_choice_excludes_the_original_attacker_and_equipment_after_reattaching() {
    let _scope = fixture_rules::activate(&[
        (card("Short Bow"), &COMBAT_ATTACHMENT),
        (Card::Forest, &PREVENT_MOVE),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let gear = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let new_recipient = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let initial_library = game.state.players[player.index()].library.len();
    let gear_ref = game.object_ref(gear);
    assert!(game.attach_equipment(gear_ref, game.object_ref(attacker)));
    game.begin_step(Step::BeginCombat);
    pass_pair(&mut game);
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    assert!(game.attach_equipment(gear_ref, game.object_ref(new_recipient)));

    let mut prevented = game.clone();
    let forest = put(&mut prevented, player, "Forest", Zone::Battlefield);
    pass_pair(&mut prevented);
    let sacrifice = choose_object(&prevented, forest);
    act(&mut prevented, sacrifice);
    assert_eq!(prevented.object_view(forest).zone, Zone::Battlefield);
    // CR 118.11-12: the optional cost was chosen even though the move was replaced.
    assert_eq!(
        prevented.state.players[player.index()].library.len(),
        initial_library - 1
    );

    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    let gear_choice = choose_object(&game, gear);
    let attacker_choice = choose_object(&game, attacker);
    reject(&mut game, gear_choice);
    reject(&mut game, attacker_choice);

    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert_eq!(
        decline.state.players[player.index()].library.len(),
        initial_library
    );

    let save = game.snapshot();
    let action = choose_object(&game, new_recipient);
    let mut indexed = game.clone();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, action);
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&save).unwrap();
    assert_eq!(indexed.object_view(new_recipient).zone, Zone::Battlefield);
    assert_eq!(game.object_view(new_recipient).zone, Zone::Graveyard);
    assert_eq!(game.object_view(attacker).zone, Zone::Battlefield);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        initial_library - 1
    );
}

#[test]
fn targeted_player_sacrifices_around_the_current_or_departed_aura_attachment() {
    let _scope = fixture_rules::activate(&[(card("Pacifism"), &AURA)]);
    let caster = Player::P0;
    let player = caster.other();
    let mut game = scenario(caster);
    let original = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let aura = put(&mut game, caster, "Pacifism", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(original)));
    pass_pair(&mut game);
    choose(
        &mut game,
        |action| matches!(action, Action::ChooseTarget { target: Target::Player(target) } if *target == player),
    );
    game.state.objects[second as usize].gained_keywords = Keywords::HEXPROOF;

    let mut changed_then_departed = game.clone();
    changed_then_departed.state.objects[aura as usize].attached_to = Some(game.object_ref(second));
    let destroy = put(&mut changed_then_departed, caster, "Disenchant", Zone::Hand);
    cast(
        &mut changed_then_departed,
        destroy,
        Some(Target::Card(aura)),
    );
    pass_pair(&mut changed_then_departed);
    assert_eq!(
        changed_then_departed.object_view(aura).zone,
        Zone::Graveyard
    );

    for (branch, excluded, paid) in [
        (&mut game, original, second),
        (&mut changed_then_departed, second, original),
    ] {
        pass_pair(branch);
        assert_eq!(branch.actor(), Some(player));
        let illegal = choose_object(branch, excluded);
        let legal = choose_object(branch, paid);
        reject(branch, illegal);
        reject(branch, Action::DeclineEffectPayment);
        act(branch, legal);
        assert_eq!(branch.object_view(excluded).zone, Zone::Battlefield);
        assert_eq!(branch.object_view(paid).zone, Zone::Graveyard);
    }
}
