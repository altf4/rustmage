//! Live artifact activation grants, exact sacrifices, and damage from source LKI.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROVIDER: Card = Card::from_catalogue_id(1583);
const ARTIFACT: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    ..CardRules::EMPTY
};
const GRANT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledArtifactActivation(0)],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[Instruction::DamageScaled(AmountValue::Stat(
            StatValue::Sum(&[
                StatValue::Fixed(1),
                StatValue::NamedCount {
                    filter: CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: false,
                    },
                    name: "Food Fight",
                },
            ]),
        ))],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const BLANK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16 | 32,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
    ),
    ..CardRules::EMPTY
};

fn granted(game: &Game, artifact: CardId, provider: CardId) -> Action {
    Action::AnnounceArtifactFood {
        source: game.object_ref(artifact),
        grantor: game.object_ref(provider),
        ability: 0,
    }
}

fn choose_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

#[test]
fn paid_sacrifice_uses_the_artifact_lki_and_counts_live_named_permanents() {
    let _rules = fixture_rules::activate(&[(PROVIDER, &GRANT)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let first = spawn(&mut game, player, PROVIDER, Zone::Battlefield);
        let second = spawn(&mut game, player, PROVIDER, Zone::Battlefield);
        let theirs = spawn(&mut game, opponent, PROVIDER, Zone::Battlefield);
        let removal = put(&mut game, opponent, "Disenchant", Zone::Hand);
        let artifact = put(&mut game, player, "Kavaron Turbodrone", Zone::Battlefield);
        let collar = put(&mut game, player, "Basilisk Collar", Zone::Battlefield);
        let land = put(&mut game, player, "Forest", Zone::Battlefield);
        for source in [artifact, collar] {
            act(
                &mut game,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            choose_target(&mut game, Target::Card(artifact));
            pay(&mut game);
            settle(&mut game);
        }
        assert!(game.object_view(artifact).tapped);
        assert!(
            game.object_view(artifact)
                .keywords
                .contains(Keywords::LIFELINK)
        );
        game.state.players[player.index()].mana = [0; 6];

        let from_first = granted(&game, artifact, first);
        let from_second = granted(&game, artifact, second);
        let from_theirs = granted(&game, artifact, theirs);
        let artifact_lki = game.object_ref(artifact);
        assert!(game.has_subtype(artifact, "Robot"));
        assert!(!game.has_subtype(artifact, "Food"));
        let mut unfunded = game.clone();
        reject(&mut unfunded, from_first.clone());

        let second_land = put(&mut game, player, "Forest", Zone::Battlefield);
        let actions = game.legal_actions();
        assert!(actions.contains(&from_second));
        assert!(!actions.contains(&from_theirs));
        let index = actions.iter().position(|a| *a == from_first).unwrap();
        let revision = game.revision();
        let mut indexed = game.clone();
        act(&mut game, from_first);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());

        let target = Target::Player(opponent);
        choose_target(&mut game, target);
        let payment = game.snapshot();
        let tap = Action::TapMana { card: land };
        assert!(game.apply(opponent, tap.clone()).is_err());
        assert_eq!(game.snapshot(), payment);

        act(&mut game, tap.clone());
        act(&mut game, Action::TapMana { card: second_land });
        pay(&mut game);
        let departed = game.object_view(artifact);
        assert_eq!(departed.zone, Zone::Graveyard);
        assert_ne!(departed.reference, artifact_lki);
        assert!(!departed.keywords.contains(Keywords::LIFELINK));
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        let ability = game.state.stack.last().unwrap();
        let source = ability.source.unwrap();
        assert_eq!(source.object, artifact_lki);
        assert_eq!(source.facts.controller, player);
        assert_ne!(source.facts.kinds & 16, 0);
        assert!(ability.source_keywords.contains(Keywords::LIFELINK));

        indexed.restore(&payment).unwrap();
        act(&mut indexed, tap);
        act(&mut indexed, Action::TapMana { card: second_land });
        pay(&mut indexed);
        assert_eq!(indexed.snapshot(), game.snapshot());

        let your_life = game.state.players[player.index()].life;
        let their_life = game.state.players[opponent.index()].life;
        act(&mut indexed, Action::Pass);
        cast(&mut indexed, removal, Some(Target::Card(first)));
        settle(&mut indexed);
        assert_eq!(indexed.object_view(first).zone, Zone::Graveyard);
        assert_eq!(indexed.state.players[opponent.index()].life, their_life - 2);
        assert_eq!(indexed.state.players[player.index()].life, your_life + 2);

        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, their_life - 3);
        assert_eq!(game.state.players[player.index()].life, your_life + 3);
    }
}

#[test]
fn suppression_and_controller_changes_are_checked_at_the_grants_own_timestamp() {
    let blank = Card::from_name("Opt").unwrap();
    let artifact_creature = Card::from_name("Gingerbrute").unwrap();
    let _rules = fixture_rules::activate(&[
        (PROVIDER, &GRANT),
        (blank, &BLANK),
        (artifact_creature, &ARTIFACT),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = spawn(&mut game, player, PROVIDER, Zone::Battlefield);
    let later = spawn(&mut game, player, PROVIDER, Zone::Hand);
    let theirs = spawn(&mut game, opponent, PROVIDER, Zone::Battlefield);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let borrowed = spawn(&mut game, opponent, artifact_creature, Zone::Battlefield);
    let blank = spawn(&mut game, player, blank, Zone::Hand);
    let theft = put(&mut game, player, "Goatnap", Zone::Hand);

    let original = granted(&game, boots, first);
    assert!(game.legal_actions().contains(&original));
    let nonartifact = granted(&game, bear, first);
    reject(&mut game, nonartifact);
    let opposing_recipient = granted(&game, borrowed, first);
    reject(&mut game, opposing_recipient);
    let mut blanked_provider = game.clone();
    cast(&mut blanked_provider, blank, Some(Target::Card(first)));
    settle(&mut blanked_provider);
    reject(&mut blanked_provider, original.clone());

    cast(&mut game, blank, Some(Target::Card(boots)));
    settle(&mut game);
    reject(&mut game, original.clone());
    cast(&mut game, later, None);
    settle(&mut game);
    reject(&mut game, original);
    assert!(game.legal_actions().contains(&granted(&game, boots, later)));
    assert!(game.has_subtype(boots, "Equipment"));
    assert!(!game.has_subtype(boots, "Food"));

    cast(&mut game, theft, Some(Target::Card(borrowed)));
    settle(&mut game);
    assert_eq!(game.object_view(borrowed).controller, player);
    let opposing_provider = granted(&game, borrowed, theirs);
    reject(&mut game, opposing_provider);
    assert!(
        game.legal_actions()
            .contains(&granted(&game, borrowed, later))
    );
    let activation = granted(&game, borrowed, first);
    act(&mut game, activation);
    choose_target(&mut game, Target::Player(opponent));
    pay(&mut game);
    let life = game.state.players[opponent.index()].life;
    settle(&mut game);
    assert_eq!(game.object_view(borrowed).zone, Zone::Graveyard);
    assert_eq!(game.object_view(borrowed).owner, opponent);
    assert_eq!(game.state.players[opponent.index()].life, life - 3);
}
