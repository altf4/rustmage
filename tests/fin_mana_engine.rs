//! Mana grant layers, spending restrictions and exact subtype discounts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const RED: Mana = [0, 0, 0, 1, 0, 0];
const SHIP: CardRules = CardRules {
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
            production: &ManaProduction::Fixed(COLORLESS),
            quality: &ManaQuality {
                restriction: ManaRestriction::ArtifactSpellOrArtifactActivation,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

const GENERIC_DRAW: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        generic: 1,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    sorcery: false,
    once_per_turn: false,
};

const NONARTIFACT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[GENERIC_DRAW],
    ..CardRules::EMPTY
};

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn ship_mana(card: CardId) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: COLORLESS,
    }
}

#[test]
fn cargo_mana_pays_only_for_artifact_spells_and_abilities_of_artifacts() {
    let ship = Card::from_name("Cargo Ship").unwrap();
    let bear = Card::from_name("Grizzly Bears").unwrap();
    let _rules = fixture_rules::activate(&[(ship, &SHIP), (bear, &NONARTIFACT)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let source = spawn(&mut parent, player, ship, Zone::Battlefield);
    let bearer = spawn(&mut parent, player, bear, Zone::Battlefield);
    let boots = put(&mut parent, player, "Swiftfoot Boots", Zone::Battlefield);
    let artifact = put(&mut parent, player, "Swiftfoot Boots", Zone::Hand);
    let giant = put(&mut parent, player, "Hill Giant", Zone::Hand);

    parent.state.players[player.index()].mana = [0, 0, 0, 1, 0, 2];
    reject(&mut parent, Action::AnnounceCast { card: giant });
    parent.state.players[player.index()].mana = COLORLESS;
    let mut cast_branch = parent.clone();
    act(&mut cast_branch, Action::AnnounceCast { card: artifact });
    act(&mut cast_branch, ship_mana(source));
    pay(&mut cast_branch);
    settle(&mut cast_branch);
    assert_eq!(cast_branch.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(cast_branch.state.players[player.index()].mana, [0; 6]);

    parent.state.players[player.index()].mana = [0; 6];
    reject(&mut parent, activate(bearer));
    act(&mut parent, ship_mana(source));
    assert_eq!(parent.state.players[player.index()].mana, COLORLESS);
    assert_eq!(
        parent.observe(player).mana_lots[0].restriction,
        ManaRestrictionView::ArtifactSpellOrArtifactActivation
    );
    reject(&mut parent, activate(bearer));
    act(&mut parent, activate(boots));
    act(
        &mut parent,
        Action::ChooseTarget {
            target: Target::Card(bearer),
        },
    );
    if parent.legal_actions().contains(&Action::FinishTargets) {
        act(&mut parent, Action::FinishTargets);
    }
    pay(&mut parent);
    settle(&mut parent);
    assert_eq!(
        parent.object_view(boots).attached_to,
        Some(parent.object_ref(bearer))
    );
    assert_eq!(parent.state.players[player.index()].mana, [0; 6]);
}

const REALM_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::OtherControlledPermanentTapMana(
        ManaProduction::any_color(1),
    )],
}];
const REALM: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: REALM_GROUP,
    extras: &CardExtras {
        static_ability_groups: REALM_GROUP,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn realm_grants_fundable_mana_to_other_permanents_through_ability_layers() {
    let realm = Card::from_name("A Realm Reborn").unwrap();
    let _rules = fixture_rules::activate(&[(realm, &REALM)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let grantor = spawn(&mut parent, player, realm, Zone::Battlefield);
    let artifact = put(&mut parent, player, "Swiftfoot Boots", Zone::Battlefield);
    let creature = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let opponent = put(
        &mut parent,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let shock = put(&mut parent, player, "Shock", Zone::Hand);
    parent.state.objects[creature as usize].summoning_sick = true;
    parent.state.players[player.index()].mana = [0; 6];
    let provider = parent.object_ref(grantor);
    let tap = |card| Action::TapGrantedMana {
        card,
        grantor: provider,
        production: RED,
    };
    for disallowed in [grantor, creature, opponent] {
        reject(&mut parent, tap(disallowed));
    }
    let artifact_ref = parent.object_ref(artifact);
    let mut removed = parent.clone();
    removed.remove_abilities_exact(
        &[artifact_ref],
        provider,
        player,
        AnimationDuration::EndOfTurn,
    );
    reject(&mut removed, tap(artifact));
    reject(&mut removed, Action::AnnounceCast { card: shock });
    let mut departed = parent.clone();
    departed.move_object(grantor, Zone::Graveyard);
    reject(&mut departed, tap(artifact));

    act(&mut parent, Action::AnnounceCast { card: shock });
    act(
        &mut parent,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    if parent.legal_actions().contains(&Action::FinishTargets) {
        act(&mut parent, Action::FinishTargets);
    }
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == tap(artifact))
        .unwrap();
    let mut indexed = parent.clone();
    act(&mut parent, tap(artifact));
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(parent.snapshot(), indexed.snapshot());
    for branch in [&mut parent, &mut indexed] {
        assert!(branch.object_view(artifact).tapped);
        assert_eq!(branch.state.players[player.index()].mana, RED);
        pay(branch);
        settle(branch);
        assert_eq!(branch.state.players[player.other().index()].life, 18);
    }
    assert_eq!(parent.snapshot(), indexed.snapshot());
}

const TOWN: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 1, 0],
            generic: 5,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        ..GENERIC_DRAW
    }],
    extras: &CardExtras {
        activation_other_subtype_discounts: &[(0, "Town")],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn town_discount_uses_other_controlled_towns_in_projection_and_payment() {
    let balamb = Card::from_name("Balamb Garden, SeeD Academy // Balamb Garden, Airborne").unwrap();
    let _rules = fixture_rules::activate(&[(balamb, &TOWN)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, balamb, Zone::Battlefield);
    let foreign = put(
        &mut game,
        player.other(),
        "Vector, Imperial Capital",
        Zone::Battlefield,
    );
    game.state.objects[foreign as usize].tapped = true;
    let available = [0, 1, 0, 0, 1, 4];
    game.state.players[player.index()].mana = available;
    assert!(game.has_subtype(source, "Town"));
    reject(&mut game, activate(source));
    let other = put(
        &mut game,
        player,
        "Vector, Imperial Capital",
        Zone::Battlefield,
    );
    game.state.objects[other as usize].tapped = true;
    let drawn = game.state.players[player.index()].cards_drawn_this_turn;
    act(&mut game, activate(source));
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 4);
    pay(&mut game);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].cards_drawn_this_turn,
        drawn + 1
    );
}
