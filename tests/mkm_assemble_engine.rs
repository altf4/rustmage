//! Power-qualified casting from the library and exact once-per-turn providers.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, generic, object, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const ASSEMBLE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LookAtLibraryTop],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::CastCreaturePowerAtMostFromLibraryTopOnceEachTurn(2),
            ],
        },
    ],
    ..CardRules::EMPTY
};
const HIGH_POWER_DISGUISE: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(generic(2))),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const BLINK: CardRules = CardRules {
    effect: Effect::Spell(
        one(object(0)),
        &[Instruction::BlinkImmediately {
            counter: None,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn library_casts(game: &Game, card: CardId) -> Vec<(Action, LibraryTopCastPermission)> {
    let object = game.object_ref(card);
    game.legal_actions()
        .into_iter()
        .filter_map(|action| match action {
            Action::AnnounceLibraryTop {
                object: chosen,
                permission,
                ..
            } if chosen == object => Some((action, permission)),
            _ => None,
        })
        .collect()
}

fn visible_top(game: &Game, viewer: Player, owner: Player) -> Option<ObjectRef> {
    game.observe(viewer).known_library_top[owner.index()]
        .as_ref()
        .map(|top| top.object)
}

#[test]
fn top_cast_uses_the_creature_spells_characteristic_defining_power_and_timing() {
    let provider = named("Authority of the Consuls");
    let _rules = fixture_rules::activate(&[(provider, &ASSEMBLE)]);
    let player = Player::P1;
    let mut game = scenario(player);
    spawn(&mut game, player, provider, Zone::Battlefield);
    spawn(&mut game, player, provider, Zone::Battlefield);
    let bunny = put(&mut game, player, "Regal Bunnicorn", Zone::Library);
    assert_eq!(library_casts(&game, bunny).len(), 2);
    let mut wrong_timing = game.clone();
    wrong_timing.begin_step(Step::Upkeep);
    assert!(library_casts(&wrong_timing, bunny).is_empty());

    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    assert!(library_casts(&game, bunny).is_empty());
    game.move_object(bow, Zone::Graveyard);
    let (cast_action, _) = library_casts(&game, bunny).remove(0);
    act(&mut game, cast_action);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(bunny).zone, Zone::Battlefield);
    assert_eq!(stats(&game, bunny), (3, 3));
}

#[test]
fn high_power_disguise_consumes_only_its_exact_provider_and_defers_the_next_top() {
    let provider = named("Authority of the Consuls");
    let disguise = named("Bubble Smuggler");
    let blink = named("Opt");
    let _rules = fixture_rules::activate(&[
        (provider, &ASSEMBLE),
        (disguise, &HIGH_POWER_DISGUISE),
        (blink, &BLINK),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = spawn(&mut game, player, provider, Zone::Battlefield);
    let second = spawn(&mut game, player, provider, Zone::Battlefield);
    let next_turn = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let next = put(&mut game, player, "Bear Cub", Zone::Library);
    let hidden = spawn(&mut game, player, disguise, Zone::Library);
    let blink = spawn(&mut game, player, blink, Zone::Hand);
    let reference = game.object_ref(hidden);
    assert_eq!(visible_top(&game, player, player), Some(reference));
    assert_eq!(visible_top(&game, opponent, player), None);
    let actions = library_casts(&game, hidden);
    assert_eq!(actions.len(), 2);
    assert!(actions.iter().all(|(action, _)| matches!(
        action,
        Action::AnnounceLibraryTop {
            method: CastMethod::Disguise,
            face: CastFace::Normal,
            flash_surcharge: false,
            ..
        }
    )));

    let (action, permission) = actions
        .into_iter()
        .find(|(_, permission)| permission.source == game.object_ref(first))
        .unwrap();
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(visible_top(&game, player, player), None);
    let private = game.observe(player);
    assert_eq!(
        private.announcing_library_top_cast_permission,
        Some(permission)
    );
    assert!(private.used_library_top_cast_permissions.is_empty());
    assert_eq!(private.face_down_spells[0].underlying, Some(disguise));
    assert_eq!(game.observe(opponent).face_down_spells[0].underlying, None);

    indexed.restore(&saved).unwrap();
    assert!(indexed.apply_index(player, revision, index).is_err());
    assert_eq!(library_casts(&indexed, hidden).len(), 2);
    assert_eq!(visible_top(&indexed, player, player), Some(reference));

    reject(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    assert_eq!(
        visible_top(&game, player, player),
        Some(game.object_ref(next))
    );
    assert_eq!(visible_top(&game, opponent, player), None);
    assert_eq!(
        game.observe(player).used_library_top_cast_permissions,
        [LibraryTopCastUse { player, permission }]
    );
    settle(&mut game);
    assert_eq!(stats(&game, hidden), (2, 2));
    assert_eq!(
        game.observe(opponent).face_down_permanents[0].underlying,
        None
    );

    let [(action, remaining)] = library_casts(&game, next).try_into().unwrap();
    assert_eq!(remaining.source, game.object_ref(second));
    act(&mut game, action);
    pay(&mut game);
    settle(&mut game);
    assert!(library_casts(&game, next_turn).is_empty());

    cast(&mut game, blink, Some(Target::Card(first)));
    settle(&mut game);
    let [(_, refreshed)] = library_casts(&game, next_turn).try_into().unwrap();
    assert_eq!(refreshed.source, game.object_ref(first));
    assert_ne!(refreshed.source, permission.source);
}
