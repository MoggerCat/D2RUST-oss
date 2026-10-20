//! The rules a game runs under.
//!
//! This public tree has one fixed ruleset: the game's own. There is no
//! rule override layer and no mod wire.

use std::fmt::Debug;
use d2_data::bin::DataRules;
use d2_data::calc::{CalcRegistry, Family};

use crate::combat::hit::{BlockResult, HitTerms};
use crate::combat::DamageRecord;
use crate::game::Game;
use crate::items::provider::ItemProvider;
pub use crate::missiles::provider::{MissileProvider, MissileSlot};
use crate::monsters::init::{GameInfo, PlayerBonus};
pub use crate::monsters::provider::MonsterProvider;
use crate::rng::Seed;
use crate::skills::calc::CalcContext;
pub use crate::skills::use_::table::Kind as SkillFn;
use crate::skills::use_::UseWorld;
use crate::skills::{ManaUnits, SkillUnits};
use crate::units::{UnitId, UnitType};
pub use crate::world::cube::CubeProvider;
pub use crate::world::npc::NpcProvider;
pub use crate::world::objects::ObjectProvider;
pub use crate::world::quests::QuestProvider;
pub use crate::world::vendors::StoreProvider;

mod events;
pub use events::{EventHooks, EventHost};

/// D2RUST's rules.
pub static ORIGINAL: Ruleset = Ruleset;

/// The rules a game runs under. The public tree has one ruleset.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ruleset;

impl Ruleset {
    /// Always true: this tree has only the game's own rules.
    pub fn is_original(&self) -> bool {
        true
    }

    /// The data rules the game's tables load under.
    pub fn data_rules(&self) -> DataRules {
        DataRules::Original
    }

    pub fn events(&self) -> Option<&dyn EventHooks> {
        None
    }

    pub fn calc(&self) -> Option<&dyn CalcProvider> {
        None
    }

    pub fn items(&self) -> Option<&dyn ItemProvider> {
        None
    }

    pub fn formulas(&self) -> Option<&dyn FormulaHooks> {
        None
    }

    pub fn player_bonus(&self, info: &GameInfo, align: u8) -> PlayerBonus {
        crate::monsters::init::player_bonus(info, align)
    }

    pub fn cube(&self) -> Option<&dyn CubeProvider> {
        None
    }

    pub fn objects(&self) -> Option<&dyn ObjectProvider> {
        None
    }

    pub fn npcs(&self) -> Option<&dyn NpcProvider> {
        None
    }

    pub fn quests(&self) -> Option<&dyn QuestProvider> {
        None
    }

    pub fn stores(&self) -> Option<&dyn StoreProvider> {
        None
    }

    pub fn monsters(&self) -> Option<&dyn MonsterProvider> {
        None
    }

    pub fn missiles(&self) -> Option<&dyn MissileProvider> {
        None
    }

    pub fn skills(&self) -> Option<&dyn SkillProvider> {
        None
    }

    pub fn has_skill_slot(&self, _kind: SkillFn, _index: u16) -> bool {
        false
    }
}

/// A game's formula functions (MS9).
///
/// The game registers its keywords in [`CalcProvider::registry`] and
/// compiles its tables with that registry
/// (`d2_data::compile_set::compile_all_with`), so a keyword compiles to a
/// CALL with a function index past the family's own. The evaluator sends
/// exactly those calls here; calls to D2RUST's own functions, and to an
/// index nothing is registered at, never reach the provider.
pub trait CalcProvider: Send + Sync + Debug {
    /// The keywords this game adds, per family.
    fn registry(&self) -> &CalcRegistry;
    /// Function `index` of `family` (registered in
    /// [`CalcProvider::registry`]) with its arguments in source order.
    /// `ctx` is the evaluation's own context: its `param` and `call` give
    /// D2RUST's parameters and functions (`rand` draws from the game's
    /// seeded RNG).
    fn call(&self, family: Family, index: u8, args: &[i32], ctx: &mut dyn CalcContext) -> i32;
}

/// The unit of a formula a [`FormulaHost`] reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// The unit attacking; absent in formulas of one unit.
    Attacker,
    /// The unit hit, blocking or resisting.
    Defender,
}

/// What a game's combat formulas read (MS3): the formula's units, the
/// game's difficulty and expansion flag, and the units' seeds. A side the
/// formula has no unit for reads as absent (`unit_type` `None`, stats 0).
pub trait FormulaHost {
    /// The side's unit type; `None` when the formula has no such unit.
    fn unit_type(&self, side: Side) -> Option<UnitType>;
    /// The side's class (monstats or player class row).
    fn class_id(&self, side: Side) -> i32;
    /// The side's total value of `stat` at `layer`.
    fn stat(&self, side: Side, stat: u16, layer: u16) -> i32;
    /// The side has `state`.
    fn has_state(&self, side: Side, state: u16) -> bool;
    /// The game's difficulty (0 normal, 1 nightmare, 2 hell).
    fn difficulty(&self) -> usize;
    /// The game runs the expansion rules.
    fn expansion(&self) -> bool;
    /// The side's seed, the only RNG a formula may draw from. `None` in
    /// the formulas that never draw (resist and block chance) and for an
    /// absent side.
    fn seed(&mut self, side: Side) -> Option<&mut Seed>;
}

/// A game's combat and progression formulas (MS3).
///
/// Each hook answers first; `None` runs D2RUST's own formula. A hook gets
/// the formula's inputs: its units through a [`FormulaHost`], or the plain
/// values D2RUST's formula reads.
pub trait FormulaHooks: Send + Sync + Debug {
    /// The attacker's chance in percent to hit the defender (D2RUST: the
    /// attack rating against the defense and both levels, 5 to 95). The
    /// roll is one draw from the attacker's seed against the answer.
    fn hit_chance(&self, h: &mut dyn FormulaHost, terms: HitTerms) -> Option<i32> {
        let _ = (h, terms);
        None
    }
    /// The defender's block chance in percent (D2RUST: shield, block
    /// stat, class block factor, dexterity and level, capped at
    /// [`crate::combat::hit::BLOCK_CAP`]). The roll still divides it by 3 for a running
    /// player.
    fn block_chance(&self, h: &mut dyn FormulaHost) -> Option<i32> {
        let _ = h;
        None
    }
    /// The defender's chance in percent of one way to avoid a hit:
    /// `kind` is [`BlockResult::Evade`] (while moving),
    /// [`BlockResult::WeaponBlock`], [`BlockResult::Avoid`] (missiles) or
    /// [`BlockResult::Dodge`] (melee). D2RUST: the passive stat's value.
    /// 0 or less never rolls.
    fn avoid_chance(&self, h: &mut dyn FormulaHost, kind: BlockResult) -> Option<i32> {
        let _ = (h, kind);
        None
    }
    /// The resist penalty a player or hireling takes at `difficulty`,
    /// added to each resist (D2RUST: the difficulty table's penalty in
    /// the expansion, -20 / -50 in the classic game).
    fn resist_penalty(&self, difficulty: usize, expansion: bool) -> Option<i32> {
        let _ = (difficulty, expansion);
        None
    }
    /// The cap on a player's or hireling's positive resist of stat
    /// `resist` (`None` for the leech rows). D2RUST: 75 plus the matching
    /// max-resist stat, at most 95; damage reduction 50. Monsters have no
    /// cap.
    fn resist_cap(&self, h: &mut dyn FormulaHost, resist: Option<u16>) -> Option<i32> {
        let _ = (h, resist);
        None
    }
    /// The attacker's weapon and elemental damage against the defender:
    /// `rec` is the record before the roll, `offhand` the hand used and
    /// `scale` the skill's damage share (128 = all). The answer is the
    /// rolled record; resists and absorbs still apply after it.
    fn damage(
        &self,
        h: &mut dyn FormulaHost,
        rec: &DamageRecord,
        offhand: bool,
        scale: i32,
    ) -> Option<DamageRecord> {
        let _ = (h, rec, offhand, scale);
        None
    }
    /// The players-X scaling of a monster created in a game of `info`
    /// (`align` its monstats alignment): the player count it counts and
    /// its life and experience bonus percents.
    fn player_bonus(&self, info: &GameInfo, align: u8) -> Option<PlayerBonus> {
        let _ = (info, align);
        None
    }
    /// The highest level of player class `class` (D2RUST: the experience
    /// table's max level row). Levels past the table need
    /// [`FormulaHooks::exp_threshold`].
    fn max_level(&self, class: i32) -> Option<u32> {
        let _ = class;
        None
    }
    /// The experience a player of `class` needs to reach level
    /// `level + 1` (D2RUST: the experience table).
    fn exp_threshold(&self, class: i32, level: u32) -> Option<u32> {
        let _ = (class, level);
        None
    }
}

/// What a game's skill bodies read and change (MS1). Every skill world
/// on [`UnitId`]s is one.
pub trait SkillHost {
    /// Game frame.
    fn frame(&self) -> i32;
    /// The unit's class (monstats or player class row).
    fn class_id(&self, u: UnitId) -> i32;
    /// The unit's total value of `stat` at `layer`.
    fn stat(&self, u: UnitId, stat: u16, layer: u16) -> i32;
    /// The unit's base value of `stat` at `layer`.
    fn base_stat(&self, u: UnitId, stat: u16, layer: u16) -> i32;
    /// The unit has `state`.
    fn has_state(&self, u: UnitId, state: u16) -> bool;
    /// Sets the unit's base `stat` (layer 0).
    fn set_stat(&mut self, u: UnitId, stat: u16, value: i32);
    /// The unit's seed: the only RNG a mod may draw from.
    fn seed(&mut self, u: UnitId) -> &mut Seed;
}

impl<W: UseWorld<Unit = UnitId>> SkillHost for W {
    fn frame(&self) -> i32 {
        UseWorld::frame(self)
    }
    fn class_id(&self, u: UnitId) -> i32 {
        SkillUnits::class_id(self, u)
    }
    fn stat(&self, u: UnitId, stat: u16, layer: u16) -> i32 {
        SkillUnits::stat(self, u, stat, layer)
    }
    fn base_stat(&self, u: UnitId, stat: u16, layer: u16) -> i32 {
        SkillUnits::base_stat(self, u, stat, layer)
    }
    fn has_state(&self, u: UnitId, state: u16) -> bool {
        SkillUnits::has_state(self, u, state)
    }
    fn set_stat(&mut self, u: UnitId, stat: u16, value: i32) {
        ManaUnits::set_stat(self, u, stat, value);
    }
    fn seed(&mut self, u: UnitId) -> &mut Seed {
        SkillUnits::seed(self, u)
    }
}

/// A game's skill start and do bodies, by slot (MS1).
///
/// The skill use core calls a slot the provider
/// [`has`](SkillProvider::has) even where D2RUST's tables have no
/// function, and past their end. The provider answers first; `None`
/// keeps what D2RUST does for that slot (its own body, or nothing for
/// an empty slot).
pub trait SkillProvider: Send + Sync + Debug {
    /// The provider answers slot `index` of the `kind` table.
    fn has(&self, kind: SkillFn, index: u16) -> bool;
    /// `srvst[index]`; 0 refuses.
    fn srvst(
        &self,
        w: &mut dyn SkillHost,
        index: u16,
        u: UnitId,
        skill: i32,
        lvl: i32,
    ) -> Option<i32> {
        let _ = (w, index, u, skill, lvl);
        None
    }
    /// `srvdo[index]` with the core's arguments; 0 = nothing happened.
    #[allow(clippy::too_many_arguments)]
    fn srvdo(
        &self,
        w: &mut dyn SkillHost,
        index: u16,
        u: UnitId,
        skill: i32,
        lvl: i32,
        charge: bool,
        item: bool,
        aim: bool,
    ) -> Option<i32> {
        let _ = (w, index, u, skill, lvl, charge, item, aim);
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_original() {
        let r = Ruleset::default();
        assert!(r.is_original());
        assert_eq!(r.data_rules(), DataRules::Original);
    }
}
