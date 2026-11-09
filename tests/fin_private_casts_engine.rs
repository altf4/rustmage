//! Immediate play permissions preserve hand and face-down exile privacy.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DAMAGE_OFFER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Attached,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::CastHandFreeUpTo(AmountValue::DamageEventAmount),
        ],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn combat(game: &mut Game, attacker: CardId) {
    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(game, Action::Pass);
    }
    assert_eq!(game.state.pending, Pending::Attackers);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
    pass_pair(game);
    act(game, Action::FinishBlockers);
    pass_pair(game);
}

#[test]
fn combat_offer_uses_the_actual_damage_and_keeps_eligibility_private() {
    let sword = Card::from_name("Buster Sword").unwrap();
    let _rules = fixture_rules::activate(&[(sword, &DAMAGE_OFFER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut parent = scenario(player);
    let source = spawn(&mut parent, player, sword, Zone::Battlefield);
    let bear = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut parent, player, "Island", Zone::Hand);
    let expensive = put(&mut parent, player, "Hill Giant", Zone::Hand);
    let drawn = put(&mut parent, player, "Swiftfoot Boots", Zone::Library);
    let foreign = put(&mut parent, opponent, "Shock", Zone::Hand);
    assert!(parent.attach_equipment(parent.object_ref(source), parent.object_ref(bear)));
    combat(&mut parent, bear);
    assert_eq!(parent.state.step, Step::CombatDamage);
    assert_eq!(parent.state.players[opponent.index()].life, 18);
    assert_eq!(
        parent
            .state
            .stack
            .last()
            .unwrap()
            .damage_event
            .unwrap()
            .amount,
        2
    );
    parent.move_object(bear, Zone::Graveyard);
    let mut other_hand = parent.clone();
    other_hand.state.objects[drawn as usize].card = Card::from_name("Hill Giant").unwrap();
    for branch in [&mut parent, &mut other_hand] {
        pass_pair(branch);
        assert_eq!(branch.object_view(drawn).zone, Zone::Hand);
        assert_eq!(branch.state.players[player.index()].mana, [0; 6]);
        assert!(branch.observe(opponent).resolution_free_casts.is_none());
    }
    assert_eq!(
        serde_json::to_value(parent.observe(opponent)).unwrap(),
        serde_json::to_value(other_hand.observe(opponent)).unwrap()
    );
    assert_eq!(
        other_hand.legal_actions(),
        [Action::Concede, Action::FinishResolutionCasts]
    );

    let free = |card| Action::CastFreeDuringResolution {
        object: parent.object_ref(card),
        face: CastFace::Normal,
    };
    let action = free(drawn);
    let invalid = [free(land), free(expensive), free(foreign)];
    for disallowed in invalid {
        reject(&mut parent, disallowed);
    }
    let offer = parent.observe(player).resolution_free_casts.unwrap();
    assert_eq!(offer.origin, Zone::Hand);
    assert_eq!(offer.remaining_mana_value, Some(2));
    let index = parent
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let saved = parent.snapshot();
    let revision = parent.revision();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    act(&mut raw, action);
    indexed.apply_index(player, revision, index).unwrap();
    for branch in [&mut raw, &mut indexed] {
        assert_eq!(
            branch
                .observe(player)
                .announcing
                .unwrap()
                .cast
                .unwrap()
                .method,
            CastMethod::Free
        );
        pay(branch);
        if branch
            .legal_actions()
            .contains(&Action::FinishResolutionCasts)
        {
            act(branch, Action::FinishResolutionCasts);
        }
        settle(branch);
        assert_eq!(branch.object_view(drawn).zone, Zone::Battlefield);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    raw.restore(&saved).unwrap();
    act(&mut raw, Action::FinishResolutionCasts);
    assert_eq!(raw.object_view(drawn).zone, Zone::Hand);
    assert_eq!(raw.state.pending, Pending::Priority);
}

const LEGENDARY_HIDEAWAY: CardRules = CardRules {
    effect: Effect::Land,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 4,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Any,
                optional: false,
                reveal: false,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(4),
            destination: LibraryPickDestination::Hideaway,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            tap: true,
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::LegendaryCreaturesYouControl),
                minimum: 4,
                skip: 1,
            },
            Instruction::PlayHideaway,
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn hideaway_counts_current_legends_at_resolution_and_keeps_its_link_private() {
    let hideaway = Card::from_name("Clive's Hideaway").unwrap();
    let _rules = fixture_rules::activate(&[(hideaway, &LEGENDARY_HIDEAWAY)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, hideaway, Zone::Hand);
    let hidden = put(&mut game, player, "Island", Zone::Library);
    for name in [
        "Yargle, Glutton of Urborg",
        "Kellan, Planar Trailblazer",
        "Alesha, Who Laughs at Fate",
    ] {
        put(&mut game, player, name, Zone::Battlefield);
    }
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    put(
        &mut game,
        opponent,
        "Tatyova, Benthic Druid",
        Zone::Battlefield,
    );
    act(&mut game, Action::PlayLand { card: source });
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::EffectLibrarySelection { .. }) {
            break;
        }
        act(&mut game, Action::Pass);
    }
    act(&mut game, Action::SelectLibrary { card: hidden });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    settle(&mut game);
    assert_eq!(game.object_view(hidden).zone, Zone::Exile);
    assert!(!game.can_view_hideaway(opponent, game.object_ref(hidden)));
    game.state.land_played = false;
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    let mut insufficient = game.clone();
    pass_pair(&mut insufficient);
    assert!(insufficient.observe(player).resolution_free_casts.is_none());
    assert_eq!(insufficient.object_view(hidden).zone, Zone::Exile);

    put(
        &mut game,
        player,
        "Tatyova, Benthic Druid",
        Zone::Battlefield,
    );
    game.move_object(source, Zone::Graveyard);
    pass_pair(&mut game);
    let offer = game.observe(player).resolution_free_casts.unwrap();
    assert_eq!(offer.remaining, [game.object_ref(hidden)]);
    assert!(
        game.observe(opponent)
            .resolution_free_casts
            .unwrap()
            .remaining
            .is_empty()
    );
    act(&mut game, Action::PlayLand { card: hidden });
    if game
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    assert_eq!(game.object_view(hidden).zone, Zone::Battlefield);
    assert_eq!(game.object_view(hidden).controller, player);
    assert!(game.state.land_played);
}
