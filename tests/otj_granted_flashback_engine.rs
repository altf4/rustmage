//! Exact temporary flashback grants with independent alternative costs.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GRANT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 4 | 8,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::GrantGraveyardCast(GraveyardCastKind::Flashback),
            Instruction::GrantGraveyardCast(GraveyardCastKind::FlashbackZero),
        ],
    ),
    ..CardRules::EMPTY
};

const TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OpponentGraveyardOrExileSpellTax(2)],
    }],
    ..CardRules::EMPTY
};

fn grant(game: &mut Game, player: Player, card: CardId) {
    let grant = put(game, player, "Divination", Zone::Hand);
    cast(game, grant, Some(Target::Card(card)));
    settle(game);
}

fn zero(card: CardId) -> Action {
    Action::AnnounceGrantedZeroFlashback {
        card,
        flash_surcharge: false,
    }
}

#[test]
fn zero_and_printed_cost_grants_are_distinct_and_zero_x_still_pays_a_tax() {
    let _rules = fixture_rules::activate(&[
        (Card::Divination, &GRANT),
        (Card::from_name("Short Bow").unwrap(), &TAX),
    ]);
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(player);
    put(&mut game, other, "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, player, "Exsanguinate", Zone::Graveyard);
    let bystander = put(&mut game, player, "Exsanguinate", Zone::Graveyard);
    let counter = put(&mut game, other, "Negate", Zone::Hand);
    grant(&mut game, player, spell);

    let printed = Action::AnnounceGrantedFlashback {
        card: spell,
        flash_surcharge: false,
    };
    assert!(game.legal_actions().contains(&printed));
    assert!(game.legal_actions().contains(&zero(spell)));
    reject(&mut game, zero(bystander));
    let mut stale = game.clone();
    stale.move_object(spell, Zone::Hand);
    stale.move_object(spell, Zone::Graveyard);
    reject(&mut stale, zero(spell));

    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, zero(spell));
    game.state.players[player.index()].mana[5] = 2;
    reject(&mut game, printed);
    act(&mut game, zero(spell));
    reject(&mut game, Action::ChooseX { value: 1 });
    pay(&mut game);
    let packet = game.state.stack.last().unwrap();
    assert_eq!(packet.x, Some(0));
    assert_eq!(packet.mana_value(), 2);
    let cast_info = packet.cast.unwrap();
    assert_eq!(cast_info.method, CastMethod::GrantedZeroFlashback);
    assert_eq!(cast_info.origin, Zone::Graveyard);
    assert_eq!(cast_info.payment, [0, 0, 0, 0, 0, 2]);

    if game.actor() != Some(other) {
        act(&mut game, Action::Pass);
    }
    cast(&mut game, counter, Some(Target::Spell(spell)));
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(game.object_view(bystander).zone, Zone::Graveyard);
}

#[test]
fn zero_flashback_keeps_the_mandatory_discard_and_expires_at_cleanup() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &GRANT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Seize the Spoils", Zone::Graveyard);
    grant(&mut game, player, spell);
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, zero(spell));

    let discard = put(&mut game, player, "Forest", Zone::Hand);
    let discard_ref = game.object_ref(discard);
    let mut expired = game.clone();
    expired.begin_step(Step::End);
    for _ in 0..12 {
        if expired.state.turn > game.state.turn {
            break;
        }
        act(&mut expired, Action::Pass);
    }
    assert!(expired.state.turn > game.state.turn);
    assert!(!expired.has_graveyard_cast(player, spell, GraveyardCastKind::FlashbackZero));

    act(&mut game, zero(spell));
    act(
        &mut game,
        Action::ChooseAdditionalCost {
            object: discard_ref,
        },
    );
    pay(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().payment,
        [0; 6]
    );
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
}
