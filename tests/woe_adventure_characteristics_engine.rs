//! Adventure card characteristics in public zones, independent of cast permission.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OWN_GRAVE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const OWN_EXILE: TargetRule = TargetRule::FaceUpExiledCard {
    owner: CastRelation::You,
};
const OTHER_EXILE: TargetRule = TargetRule::FaceUpExiledCard {
    owner: CastRelation::Opponent,
};

const fn adventure_targets(base: &'static TargetRule) -> TargetPlan {
    TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
        base,
        predicate: &TargetPredicate::HasAdventure,
    }))
}

const RETURN_GRAVE_ADVENTURE: CardRules = CardRules {
    effect: Effect::Spell(adventure_targets(&OWN_GRAVE), &[Instruction::ReturnToHand]),
    ..CardRules::EMPTY
};
const RETURN_EXILED_ADVENTURE: CardRules = CardRules {
    effect: Effect::Spell(adventure_targets(&OWN_EXILE), &[Instruction::ReturnToHand]),
    ..CardRules::EMPTY
};
const BOTTOM_EXILED_ADVENTURE: CardRules = CardRules {
    effect: Effect::Spell(
        adventure_targets(&OTHER_EXILE),
        &[Instruction::BottomLibrary],
    ),
    ..CardRules::EMPTY
};
const ADVENTURE_HASTE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
            condition: StatValue::OwnExiledAdventurerCard,
            keywords: Keywords::HASTE,
        }],
    }],
    ..CardRules::EMPTY
};
const GRAVEYARD_DAMAGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DamagePlayersScaled(
            PlayerSet::Opponent,
            AmountValue::Sum(&[
                AmountValue::Fixed(2),
                AmountValue::Stat(StatValue::GraveyardInstantSorceryAdventure),
            ]),
        )],
    ),
    ..CardRules::EMPTY
};

fn fixture_card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn choose_only_target(game: &mut Game, spell: CardId, candidate: CardId) {
    act(game, Action::AnnounceCast { card: spell });
    let targets: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter_map(|action| match action {
            Action::ChooseTarget { target } => Some(target),
            _ => None,
        })
        .collect();
    assert_eq!(targets, [Target::Card(candidate)]);
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(candidate),
        },
    );
    pay(game);
}

#[test]
fn graveyard_and_exile_targets_require_the_current_adventure_card_and_correct_owner() {
    let _rules = fixture_rules::activate(&[
        (Card::Divination, &RETURN_GRAVE_ADVENTURE),
        (
            fixture_card("Kellan's Lightblades"),
            &RETURN_EXILED_ADVENTURE,
        ),
        (fixture_card("Unsummon"), &BOTTOM_EXILED_ADVENTURE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let grave_spell = put(&mut game, player, "Divination", Zone::Hand);
        let return_spell = put(&mut game, player, "Kellan's Lightblades", Zone::Hand);
        let bottom_spell = put(&mut game, player, "Unsummon", Zone::Hand);
        let grave = put(&mut game, player, "Hollow Scavenger", Zone::Graveyard);
        put(&mut game, player, "Sagu Wildling", Zone::Graveyard);
        put(
            &mut game,
            player.other(),
            "Hollow Scavenger",
            Zone::Graveyard,
        );
        let own_exile = put(&mut game, player, "Ishgard, the Holy See", Zone::Exile);
        let opponent_exile = put(&mut game, player.other(), "Hollow Scavenger", Zone::Exile);
        let hidden = put(&mut game, player, "Frolicking Familiar", Zone::Exile);
        game.remember_face_down_exile(game.object_ref(hidden), 1 << player.index(), None);
        put(&mut game, player, "Sagu Wildling", Zone::Exile);
        put(&mut game, player.other(), "Sagu Wildling", Zone::Exile);

        choose_only_target(&mut game, grave_spell, grave);
        settle(&mut game);
        assert_eq!(game.object_view(grave).zone, Zone::Hand);

        assert!(
            game.observe(player)
                .objects
                .iter()
                .any(|row| row.id == hidden)
        );
        assert!(
            game.observe(player.other())
                .objects
                .iter()
                .all(|row| row.id != hidden)
        );
        act(&mut game, Action::AnnounceCast { card: return_spell });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(hidden),
            },
        );
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(opponent_exile),
            },
        );
        let action = Action::ChooseTarget {
            target: Target::Card(own_exile),
        };
        let revision = game.revision();
        let checkpoint = game.snapshot();
        let index = game
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let mut indexed = game.clone();
        act(&mut game, action);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert!(indexed.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(own_exile),
        }));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(own_exile).zone, Zone::Hand);

        choose_only_target(&mut game, bottom_spell, opponent_exile);
        let mut stale = game.clone();
        stale.move_object(opponent_exile, Zone::Hand);
        stale.move_object(opponent_exile, Zone::Exile);
        settle(&mut stale);
        assert_eq!(stale.object_view(opponent_exile).zone, Zone::Exile);
        settle(&mut game);
        assert_eq!(
            game.state.players[player.other().index()].library.first(),
            Some(&opponent_exile)
        );
    }
}

#[test]
fn only_an_owned_face_up_adventure_enables_the_live_exile_condition() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &ADVENTURE_HASTE)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let observer = put(&mut game, player, "Hill Giant", Zone::Hand);
        let hidden = put(&mut game, player, "Hollow Scavenger", Zone::Exile);
        game.remember_face_down_exile(game.object_ref(hidden), 1 << player.index(), None);
        put(&mut game, player.other(), "Hollow Scavenger", Zone::Exile);
        put(&mut game, player, "Sagu Wildling", Zone::Exile);
        let land = put(&mut game, player, "Ishgard, the Holy See", Zone::Graveyard);
        cast(&mut game, observer, None);
        settle(&mut game);
        let haste = |game: &Game| {
            game.object_view(observer)
                .keywords
                .contains(Keywords::HASTE)
        };
        assert!(!haste(&game));

        game.move_object(land, Zone::Exile);
        assert!(haste(&game), "a directly exiled Adventurer qualifies");
        game.move_object(land, Zone::Hand);
        assert!(!haste(&game));
        game.reveal_hideaway_object(game.object_ref(hidden));
        assert!(haste(&game));
    }
}

#[test]
fn graveyard_damage_and_hearth_discount_share_the_physical_card_union() {
    let _rules =
        fixture_rules::activate(&[(fixture_card("Kellan's Lightblades"), &GRAVEYARD_DAMAGE)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let damage = put(&mut game, player, "Kellan's Lightblades", Zone::Hand);
        let hearth = put(&mut game, player, "Hearth Elemental", Zone::Hand);
        for name in [
            "Shock",
            "Divination",
            "Hollow Scavenger",
            "Ishgard, the Holy See",
        ] {
            put(&mut game, player, name, Zone::Graveyard);
        }
        for name in ["Sagu Wildling", "Grizzly Bears"] {
            put(&mut game, player, name, Zone::Graveyard);
        }
        put(&mut game, player.other(), "Shock", Zone::Graveyard);

        let mut discount = game.clone();
        let announce = Action::AnnounceCast { card: hearth };
        discount.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
        reject(&mut discount, announce.clone());
        discount.state.players[player.index()].mana[3] = 2;
        act(&mut discount, announce);
        pay(&mut discount);
        assert_eq!(discount.state.players[player.index()].mana, [0; 6]);
        settle(&mut discount);
        assert_eq!(discount.object_view(hearth).zone, Zone::Battlefield);

        cast(&mut game, damage, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.other().index()].life, 14);
    }
}
