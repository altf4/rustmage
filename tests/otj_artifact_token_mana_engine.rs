//! Artifact token tap bonuses use the actual mana event and exact providers.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const BLUE_COLORLESS: Mana = [0, 1, 0, 0, 0, 1];
const BONUS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledArtifactTokenTapManaBonus],
    }],
    ..CardRules::EMPTY
};
const PRODUCER_ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        sacrifice: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Tagged {
        production: &ManaProduction::Fixed(BLUE_COLORLESS),
        quality: &ManaQuality {
            restriction: ManaRestriction::ArtifactSpell,
            spell_bonus: ManaSpellBonus::None,
            retention: ManaRetention::Step,
        },
    })],
    sorcery: false,
    once_per_turn: false,
};
const PRODUCER: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        PRODUCER_ABILITY,
        ActivatedAbility {
            cost: ActivationCost {
                tap: false,
                ..PRODUCER_ABILITY.cost
            },
            ..PRODUCER_ABILITY
        },
    ],
    ..CardRules::EMPTY
};

fn token(game: &mut Game, player: Player, definition: Card) -> CardId {
    let id = game.state.objects.len() as CardId;
    game.create_tokens(player, definition, 1, false);
    assert!(game.object_view(id).is_token());
    id
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: BLUE_COLORLESS,
    }
}

#[test]
fn a_tapped_and_sacrificed_token_can_fund_a_spell_with_only_the_bonus() {
    let treasure = Card::from_catalogue_id(5302);
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &BONUS), (treasure, &PRODUCER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let provider = game.object_ref(provider);
    put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let token = token(&mut game, player, treasure);
    let token_ref = game.object_ref(token);
    game.state.players[player.index()].mana = [0; 6];
    let spell = put(&mut game, player, "Opt", Zone::Hand);

    let mut unfunded = game.clone();
    unfunded.move_object(provider.card, Zone::Graveyard);
    reject(&mut unfunded, Action::AnnounceCast { card: spell });

    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, activate(token, 0));
    assert!(!game.on_battlefield(token));
    assert_eq!(game.observe(player).mana_tap_bonus_source, Some(provider));
    reject(
        &mut game,
        Action::ChooseMana {
            production: COLORLESS,
        },
    );
    let choice = Action::ChooseMana { production: BLUE };
    let saved = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choice)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, choice);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);

    let lots = game.mana_lots(player);
    let original = lots
        .iter()
        .find(|lot| lot.quality.producer == Some(token_ref))
        .unwrap();
    assert_eq!(original.amounts, BLUE_COLORLESS.map(u128::from));
    assert_ne!(
        original.quality.restriction,
        mana_lots::ManaRestrictionView::Unrestricted
    );
    let bonus = lots
        .iter()
        .find(|lot| lot.quality.producer == Some(provider))
        .unwrap();
    assert_eq!(bonus.amounts, BLUE.map(u128::from));
    assert_eq!(
        bonus.quality.restriction,
        mana_lots::ManaRestrictionView::Unrestricted
    );
    assert!(!bonus.quality.treasure_source);
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, BLUE_COLORLESS);
    assert_eq!(game.state.stack.last().unwrap().source.unwrap().id, spell);
}

#[test]
fn the_bonus_accepts_colorless_and_excludes_nontoken_and_nontap_production() {
    let treasure = Card::from_catalogue_id(5302);
    let bow = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &BONUS),
        (treasure, &PRODUCER),
        (bow, &PRODUCER),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let token = token(&mut game, player, treasure);
    let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
    game.state.players[player.index()].mana = [0; 6];

    let mut nontoken = game.clone();
    act(&mut nontoken, activate(artifact, 0));
    assert_eq!(nontoken.state.pending, Pending::Priority);
    assert_eq!(nontoken.state.players[player.index()].mana, BLUE_COLORLESS);

    let mut nontap = game.clone();
    act(&mut nontap, activate(token, 1));
    assert_eq!(nontap.state.pending, Pending::Priority);
    assert_eq!(nontap.state.players[player.index()].mana, BLUE_COLORLESS);

    act(&mut game, activate(token, 0));
    reject(
        &mut game,
        Action::ChooseMana {
            production: [0, 0, 1, 0, 0, 0],
        },
    );
    act(
        &mut game,
        Action::ChooseMana {
            production: COLORLESS,
        },
    );
    assert_eq!(game.state.players[player.index()].mana, [0, 1, 0, 0, 0, 2]);
}
