//! Owned hand reveal and exact exile choices.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BINDING: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[Instruction::RevealTargetHandDiscardOrMoveExileToGraveyard],
    ),
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn hand_reveal_and_exile_fallback_are_exclusive_exact_choices() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &BINDING)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    let nonland = put(&mut game, opponent, "Murder", Zone::Hand);
    let land = put(&mut game, opponent, "Forest", Zone::Hand);
    let public = put(&mut game, opponent, "Hill Giant", Zone::Exile);
    let hidden = put(&mut game, opponent, "Opt", Zone::Exile);
    let ours = put(&mut game, player, "Opt", Zone::Exile);
    let [nonland, land, public, hidden, ours] =
        [nonland, land, public, hidden, ours].map(|id| game.object_ref(id));
    game.remember_face_down_exile(hidden, 1 << opponent.index(), None);

    cast(&mut game, spell, Some(Target::Player(opponent)));
    pass_pair(&mut game);
    for viewer in [player, opponent] {
        let view = game.observe(viewer);
        assert!(view.known_hand.contains(&nonland));
        assert!(view.known_hand.contains(&land));
    }
    reject(&mut game, Action::ChooseHandCard { object: Some(land) });
    let mut discard = game.clone();
    act(
        &mut discard,
        Action::ChooseHandCard {
            object: Some(nonland),
        },
    );
    assert_eq!(discard.object_view(nonland.card).zone, Zone::Graveyard);
    assert_eq!(discard.object_view(public.card).zone, Zone::Exile);

    act(&mut game, Action::ChooseHandCard { object: None });
    reject(&mut game, Action::ChooseEffectObject { object: hidden });
    reject(&mut game, Action::ChooseEffectObject { object: ours });
    let saved = game.snapshot();
    let revision = game.revision();
    let action = Action::ChooseEffectObject { object: public };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = game.clone();
    act(&mut raw, action);
    game.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), raw.snapshot());
    assert_eq!(game.object_view(public.card).zone, Zone::Graveyard);
    assert_eq!(game.object_view(nonland.card).zone, Zone::Hand);
    game.restore(&saved).unwrap();
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.object_view(public.card).zone, Zone::Exile);
}

const COMBAT_LOOK: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedDealtCombatDamageToPlayer,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::SelectLibraryToExileRemember(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::ExcludingKinds(2),
                    optional: true,
                    reveal: false,
                    remainder: LibraryRemainder::BottomRandom,
                },
                look: LibraryLook::Amount(AmountValue::CreatureCombatDamageAmount),
                destination: LibraryPickDestination::Exile,
                public: false,
                maximum_mana_value: None,
            }),
            Instruction::CastRememberedExileFree,
        ],
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

#[test]
fn attached_combat_damage_looks_privately_then_offers_only_the_exact_exile() {
    let bow = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[(bow, &COMBAT_LOOK)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let dealer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let blocker = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let below = put(&mut game, player, "Healer's Hawk", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let selected = put(&mut game, player, "Hill Giant", Zone::Library);
    let top = put(&mut game, player, "Murder", Zone::Library);
    let unrelated = put(&mut game, player, "Hill Giant", Zone::Exile);
    assert!(game.attach_equipment(game.object_ref(equipment), game.object_ref(dealer)));

    let source = game.damage_source(dealer);
    game.apply_damage_batch(&[
        DamageEvent {
            preventable: true,
            source,
            target: Target::Player(opponent),
            amount: 3,
            combat: true,
        },
        DamageEvent {
            preventable: true,
            source,
            target: Target::Card(blocker),
            amount: 1,
            combat: true,
        },
    ]);
    game.give_priority();
    pass_pair(&mut game);
    let view = game.observe(player).library_selection.unwrap();
    assert_eq!(
        view.remaining.iter().map(|row| row.id).collect::<Vec<_>>(),
        [top, selected, land]
    );
    assert!(game.observe(opponent).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: land });
    let mut no_pick = game.clone();
    act(&mut no_pick, Action::FinishLibrarySelection);
    assert!(no_pick.state.resolution_free_casts.is_none());
    assert_eq!(no_pick.top_library_card(player), Some(below));
    act(&mut game, Action::SelectLibrary { card: selected });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.top_library_card(player), Some(below));
    let unrelated = game.object_ref(unrelated);
    reject(
        &mut game,
        Action::CastExiledDuringResolution {
            object: unrelated,
            face: CastFace::Normal,
        },
    );
    let exact = game.object_ref(selected);
    game.state.players[player.index()].mana = [0; 6];
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object: exact,
            face: CastFace::Normal,
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    if game
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    assert_eq!(game.object_view(selected).zone, Zone::Battlefield);
}
