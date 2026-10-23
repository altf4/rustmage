//! Owned copy recipes for tokens and physical permanents. Immutable definitions stay shared; copying a copy
//! flattens to its existing recipe, never to a pointer into another game object.
use super::*;

/// Physical shape is independent of the copied face's characteristics (CR707.8a/712.9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub enum CopyFaces {
    #[default]
    Definition,
    Single,
    Double {
        front: Card,
        back: Card,
        back_up: bool,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CopyRecipe {
    /// The complete immutable copiable definition, including fixed token exceptions.
    pub definition: Card,
    pub exceptions: CopyExceptions,
    pub faces: CopyFaces,
}
/// Frozen copiable values and the linked-ability instance which produced a packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceCopyFacts {
    #[serde(flatten)]
    pub recipe: CopyRecipe,
    #[serde(skip_serializing_if = "is_original_ability")]
    pub ability_origin: u64,
}
fn is_original_ability(value: &u64) -> bool {
    *value == 0
}
impl CopyRecipe {
    pub const fn new(definition: Card) -> Self {
        Self {
            definition,
            exceptions: CopyExceptions::NONE,
            faces: CopyFaces::Definition,
        }
    }

    pub(super) fn faces(self) -> CopyFaces {
        match self.faces {
            CopyFaces::Definition => {
                self.definition
                    .transform_spec()
                    .map_or(CopyFaces::Single, |s| CopyFaces::Double {
                        front: s.front,
                        back: s.back,
                        back_up: self.definition == s.back,
                    })
            }
            faces => faces,
        }
    }
    pub(super) fn mana_value(self) -> u64 {
        let card = match self.faces() {
            CopyFaces::Double { front, .. } => front,
            _ => self.definition,
        };
        let (colored, generic) = card.cost();
        u64::from(generic) + colored.into_iter().map(u64::from).sum::<u64>()
    }
    pub(super) fn overlay(self, native: Self) -> Self {
        let faces = match native.faces() {
            CopyFaces::Double { back_up, .. } => CopyFaces::Double {
                front: self.definition,
                back: self.definition,
                back_up,
            },
            _ => CopyFaces::Single,
        };
        Self { faces, ..self }
    }
    pub const fn factory_identity(self) -> &'static str {
        "Copy"
    }
    fn token_info(self) -> TokenInfo {
        let card = self.definition;
        TokenInfo {
            factory_identity: self.factory_identity(),
            name: card.name(),
            subtypes: card
                .info()
                .type_line
                .split_once('—')
                .map_or("", |(_, types)| types.trim()),
            colors: card.colors(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CopyRow {
    pub ability_origin: u64,
    /// The underlying physical card or transforming token, before its first overlay.
    pub native: Option<CopyRecipe>,
    pub loyalty_used_turn: Option<u32>,
    pub object: ObjectRef,
    pub recipe: CopyRecipe,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CopyOriginView {
    pub object: ObjectRef,
    pub native: CopyRecipe,
    pub loyalty_used_turn: Option<u32>,
}
pub(super) type Rows = Option<SharedVec<CopyRow>>;
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<CopyRow>>() + rows.len() * std::mem::size_of::<CopyRow>()
    })
}
impl Game {
    pub(super) fn copy_origins_view(&self) -> Vec<CopyOriginView> {
        self.state
            .copy_recipes
            .as_ref()
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter(|row| {
                self.on_battlefield(row.object.card)
                    && self.object_ref(row.object.card) == row.object
            })
            .filter_map(|row| {
                Some(CopyOriginView {
                    object: row.object,
                    native: row.native?,
                    loyalty_used_turn: row.loyalty_used_turn,
                })
            })
            .collect()
    }
    fn copy_recipe_index(&self, object: ObjectRef) -> Option<usize> {
        let rows = self.state.copy_recipes.as_ref()?;
        let index = rows
            .binary_search_by_key(&object.card, |row| row.object.card)
            .ok()?;
        (rows[index].object == object).then_some(index)
    }
    pub(super) fn copy_recipe(&self, object: ObjectRef) -> Option<CopyRecipe> {
        let index = self.copy_recipe_index(object)?;
        Some(self.state.copy_recipes.as_ref().unwrap()[index].recipe)
    }
    pub(super) fn native_copy_recipe(&self, object: ObjectRef) -> Option<CopyRecipe> {
        let at = self.copy_recipe_index(object)?;
        self.state.copy_recipes.as_ref().unwrap()[at].native
    }
    pub(super) fn object_mana_value(&self, id: CardId) -> u64 {
        self.mana_value_with_object(id, &self.state.objects[id as usize])
    }
    pub(super) fn mana_value_with_object(&self, id: CardId, object: &Object) -> u64 {
        if object.zone == Zone::Battlefield || object.is_token() {
            self.copy_recipe(ObjectRef {
                card: id,
                generation: object.generation,
            })
            .map_or_else(|| object.card.fixed_mana_value(), CopyRecipe::mana_value)
        } else {
            object.card.fixed_mana_value()
        }
    }
    pub(super) fn copy_ability_origin(&self, object: ObjectRef) -> u64 {
        self.copy_recipe_index(object).map_or(0, |index| {
            self.state.copy_recipes.as_ref().unwrap()[index].ability_origin
        })
    }
    pub(super) fn source_copy_facts(&self, object: ObjectRef) -> Option<Box<SourceCopyFacts>> {
        let recipe = self.copy_recipe(object)?;
        let ability_origin = self.copy_ability_origin(object);
        (recipe.exceptions != CopyExceptions::NONE
            || recipe.faces != CopyFaces::Definition
            || ability_origin != 0)
            .then(|| {
                Box::new(SourceCopyFacts {
                    recipe,
                    ability_origin,
                })
            })
    }
    /// CR111.13/608.3f: resolution makes a token, without a token-creation event.
    /// Call after target legality, before staging either an Aura or another permanent.
    pub(super) fn normalize_permanent_spell_copy(&mut self, item: &SpellView) {
        if !item.is_copy || !self.state.objects[item.card as usize].is_spell_copy() {
            return;
        }
        let recipe = item
            .source_copy_recipe
            .as_deref()
            .map_or_else(|| CopyRecipe::new(item.source_card), |facts| facts.recipe);
        self.clear_copy_recipe(item.object);
        let rows = self.state.copy_recipes.get_or_insert_with(Default::default);
        let at = rows.partition_point(|row| row.object.card < item.card);
        rows.insert(
            at,
            CopyRow {
                object: item.object,
                recipe,
                ability_origin: 0,
                native: None,
                loyalty_used_turn: None,
            },
        );
        let object = &mut self.state.objects[item.card as usize];
        object.copy_kind = CopyKind::Token;
        object.card = recipe.definition;
    }
    pub(super) fn object_token_info(&self, id: CardId) -> Option<TokenInfo> {
        let object = &self.state.objects[id as usize];
        if object.is_copy_token() {
            Some(
                // Ceasing releases the sparse recipe, but the reusable Outside
                // slot retains token identity so it cannot move as a physical
                // card. Live copies still require their exact-incarnation recipe.
                self.copy_recipe(self.object_ref(id))
                    .or_else(|| {
                        (object.zone == Zone::Outside).then(|| CopyRecipe::new(object.card))
                    })
                    .expect("runtime copy recipe")
                    .token_info(),
            )
        } else if object.copy_kind == CopyKind::Physical {
            None
        } else {
            object.card.token_info().copied()
        }
    }
    pub(super) fn clear_copy_recipe(&mut self, object: ObjectRef) {
        let Some(index) = self.copy_recipe_index(object) else {
            return;
        };
        let rows = self.state.copy_recipes.as_mut().unwrap();
        rows.remove(index);
        if rows.is_empty() {
            self.state.copy_recipes = None;
        }
    }
    pub(super) fn advance_copy_recipe(&mut self, before: ObjectRef, after: ObjectRef) {
        let Some(index) = self.copy_recipe_index(before) else {
            return;
        };
        assert_eq!(before.card, after.card);
        self.state.copy_recipes.as_mut().unwrap()[index].object = after;
    }
    pub(super) fn release_unentered_copy(&mut self, object: ObjectRef) {
        self.clear_copy_recipe(object);
        let slot = &mut self.state.objects[object.card as usize];
        assert_eq!(slot.zone, Zone::Outside);
        slot.generation = slot
            .generation
            .checked_add(1)
            .capacity("object generation exhausted");
        self.state.free_tokens.push(object.card);
    }
    pub(super) fn create_copy_groups(
        &mut self,
        player: Player,
        groups: &[TokenGroupView],
        recipes: &[Option<CopyRecipe>],
    ) {
        self.create_copy_groups_bound(player, groups, recipes, None, &[]);
    }
    pub(super) fn create_copy_groups_bound(
        &mut self,
        player: Player,
        groups: &[TokenGroupView],
        recipes: &[Option<CopyRecipe>],
        defenders: Option<&[CombatDefender]>,
        recipients: &[ObjectRef],
    ) -> Vec<ObjectRef> {
        self.create_copy_groups_bound_for_players(
            player, groups, recipes, defenders, recipients, None,
        )
    }
    pub(super) fn create_mixed_token_groups(
        &mut self,
        players: &[Player],
        groups: &[TokenGroupView],
        recipes: &[Option<CopyRecipe>],
    ) {
        let Some(&player) = players.first() else {
            assert!(groups.is_empty());
            return;
        };
        self.create_copy_groups_bound_for_players(
            player,
            groups,
            recipes,
            None,
            &[],
            Some(players),
        );
    }
    fn create_copy_groups_bound_for_players(
        &mut self,
        player: Player,
        groups: &[TokenGroupView],
        recipes: &[Option<CopyRecipe>],
        defenders: Option<&[CombatDefender]>,
        recipients: &[ObjectRef],
        players: Option<&[Player]>,
    ) -> Vec<ObjectRef> {
        assert_eq!(groups.len(), recipes.len());
        assert!(players.is_none_or(|rows| rows.len() == groups.len()));
        let total = groups
            .iter()
            .try_fold(0u128, |total, g| total.checked_add(g.amount))
            .capacity("token quantity exhausted");
        let amount = usize::try_from(total).capacity("token count representation exhausted");
        let fresh = amount.saturating_sub(self.state.free_tokens.len());
        (self.state.objects.len() as u128)
            .checked_add(fresh as u128)
            .filter(|&end| end <= u128::from(CardId::MAX) + 1)
            .capacity("object slots exhausted");
        self.state
            .objects
            .try_reserve(fresh)
            .capacity("object allocation exhausted");
        let mut moves = Vec::new();
        moves
            .try_reserve_exact(amount)
            .capacity("token batch allocation exhausted");
        let mut created = Vec::new();
        created
            .try_reserve_exact(amount)
            .capacity("token reference allocation exhausted");
        let mut defender_index = 0;
        for (index, (group, recipe)) in groups.iter().zip(recipes).enumerate() {
            let player = players.map_or(player, |rows| rows[index]);
            let attachment = group
                .token
                .role()
                .and_then(|_| recipients.get(index).copied());
            if group.token.role().is_some() && !recipients.is_empty() && attachment.is_none()
                || attachment.is_some_and(|r| !self.aura_can_enchant(group.token, player, r))
            {
                continue;
            }
            assert!(recipe.is_some() || group.token.is_token());
            for _ in 0..group.amount {
                let (id, generation) = if let Some(id) = self.state.free_tokens.pop() {
                    let old = &self.state.objects[id as usize];
                    assert!((old.is_token() || old.is_spell_copy()) && old.zone == Zone::Outside);
                    (id, old.generation)
                } else {
                    (
                        CardId::try_from(self.state.objects.len())
                            .capacity("object slots exhausted"),
                        0,
                    )
                };
                let object = Object {
                    card: group.token,
                    copy_kind: if recipe.is_some() {
                        CopyKind::Token
                    } else {
                        CopyKind::None
                    },
                    owner: player,
                    controller: player,
                    zone: Zone::Outside,
                    generation,
                    tapped: false,
                    summoning_sick: false,
                    flashback: false,
                    damage: 0,
                    damaged_this_turn: false,
                    dealt_damage_this_turn: false,
                    deathtouch_since_sba: false,
                    growth: Stats::default(),
                    gained_keywords: Keywords::NONE,
                    counters: InitialCounters::default(),
                    used_abilities: 0,
                    attached_to: None,
                };
                if id as usize == self.state.objects.len() {
                    self.state.objects.push(object);
                } else {
                    self.state.objects[id as usize] = object;
                }
                if let Some(recipe) = recipe {
                    let reference = self.object_ref(id);
                    let rows = self.state.copy_recipes.get_or_insert_with(Default::default);
                    rows.try_reserve(1)
                        .capacity("copy recipe allocation exhausted");
                    let index = rows.partition_point(|row| row.object.card < id);
                    rows.insert(
                        index,
                        CopyRow {
                            ability_origin: 0,
                            native: None,
                            loyalty_used_turn: None,
                            object: reference,
                            recipe: *recipe,
                        },
                    );
                }
                let mut movement = self.entry_move(id, Zone::Battlefield, group.tapped);
                movement.attachment = attachment;
                if group.token.is_creature()
                    && let Some(defenders) = defenders
                {
                    movement.attacking = Some(defenders[defender_index]);
                    defender_index += 1;
                }
                created.push(ObjectRef {
                    card: id,
                    generation: generation
                        .checked_add(1)
                        .capacity("object generation exhausted"),
                });
                moves.push(movement);
            }
        }
        if !moves.is_empty() {
            self.track_nightmare_copy_entries(&moves);
            self.stage_entries(moves, None);
        }
        if let Some(defenders) = defenders {
            assert_eq!(defender_index, defenders.len());
        }
        created
    }
}
