// –13, 14 rule 2
//! Doors (10), wells (11, event 2), portals (12) and torches (13).

use crate::units::{RoomId, UnitId};

use super::{
    clear_selectable, oflags, schedule_endanim, set_mode, set_object_mode, sound, ObjectControl,
    ObjectError, ObjectHost, ObjectTables, ObjectWorld, Operate, Operator,
};

#[cfg(test)]
mod tests;

/// Door debounce in host ticks (10 rule 1,).
pub const DOOR_DEBOUNCE: u32 = 500;
/// Door re-close mask: player 0x80, monster 0x100, 0x8000 (10,
pub const DOOR_CLEAR_MASK: u16 = 0x8180;
/// Door corpse mask (10,).
pub const DOOR_CORPSE_MASK: u16 = 0x8000;
/// Portal hostile delay in host ticks (12 rule 2,).
pub const PORTAL_HOSTILE_DELAY: u32 = 5000;
/// Town portal and permanent portal classes (12).
pub const TOWN_PORTAL_CLASS: u16 = 59;
pub const PERMANENT_PORTAL_CLASS: u16 = 60;
/// The arrival free-point mask (12 rule 10).
pub const PORTAL_SPOT_MASK: u32 = 0x1C09;
/// State 102 duration (12 rule 13).
pub const JUST_PORTALED_FRAMES: i32 = 75;

/// Stat ids the well heal reads (11 rule 2;).
pub mod stat {
    pub const LIFE: u16 = 6;
    pub const MAX_LIFE: u16 = 7;
    pub const MANA: u16 = 8;
    pub const MAX_MANA: u16 = 9;
    pub const STAMINA: u16 = 10;
    pub const MAX_STAMINA: u16 = 11;
}

/// States the well heal removes (11 rule 2), in order.
pub const WELL_REMOVED_STATES: [u16; 2] = [2, 1];

/// The door, well, portal and update-pass seams beyond [`ObjectWorld`].
/// Every default is the narrowest reading: nothing happens, or none.
#[allow(unused_variables)]
pub trait MiscWorld: ObjectWorld {
    /// Event 11: the creator of the portal object `row`
    fn create_level_portal(&mut self, object: UnitId, row: u16) {}
    /// rule 2 (14): flag 0x400 sound, flag 0x100 hover
    /// flags 2 bit 0, then.
    fn update_extras(&mut self, object: UnitId) {}
    /// (10,): some cell
    /// object's footprint has collision bits in `mask`. Default: none.
    fn footprint_collides(&self, object: UnitId, mask: u16) -> bool {
        false
    }
    /// A vital stat of `unit` (11 rule 2;): 6 life, 8 mana
    /// 10 stamina; the maxima 7, 9, 11 are read through the getters
    /// (8.8 fixed). Default 0.
    fn vital_stat(&self, unit: UnitId, id: u16) -> u32 {
        0
    }
    /// Set a vital stat (6, 8 or 10) and send the client its stat update
    /// (11 rule 2,; message owned by).
    fn set_vital_stat(&mut self, unit: UnitId, id: u16, value: u32) {}
    /// Remove the stat list of `state` from `unit` when present (11 rule
    /// 2;). Returns whether one was removed. Default
    /// none present.
    fn remove_state_list(&mut self, unit: UnitId, state: u16) -> bool {
        false
    }
    /// (11 rule 2, step 4): remove every
    /// curable state's stat list. Returns whether one was removed.
    fn cure_states(&mut self, unit: UnitId) -> bool {
        false
    }
    /// The well's pet heal (11 rule 2, callback over
    /// player's pets; pet order owned by). Returns whether
    /// pet changed. Default: no pets.
    fn well_heal_pets(&mut self, player: UnitId, object: UnitId) -> bool {
        false
    }
    /// `(game, player)` (12 rule 1; D2MOO
    /// `D2GAME_IteratePlayers`). Default 0: the owner test is skipped.
    fn iterate_players(&self, player: UnitId) -> u32 {
        0
    }
    /// `(player, 0)` (12 rule 2,)
    /// the host tick of the player's last hostility declaration. Default 0.
    fn hostile_time(&self, player: UnitId) -> u32 {
        0
    }
    /// the unit's party id (0xFFFF = none).
    fn party_id(&self, unit: UnitId) -> u16 {
        0xFFFF
    }
    /// `(game, O)` (12 rule 6): the partner portal
    /// destination point (object data +0x18, +0x1C) in the act of
    /// `InteractType`, streaming and populating the room there when
    /// absent; then the unit of type O +0x94 and GUID O +0x98 when it is
    /// an object.
    fn portal_partner(&mut self, object: UnitId) -> Option<UnitId> {
        None
    }
    /// P's quest record for the game difficulty exists (player data +0x10
    /// + 4 · game +0x6D; 12 rule 7).
    fn has_quest_record(&self, player: UnitId) -> bool {
        false
    }
    /// The game is an expansion game (game +0x70 ≠ 0).
    fn expansion(&self) -> bool {
        false
    }
    /// `(Q, q, bit)`: the bit of quest `q` in P's quest record.
    fn player_quest_bit(&self, player: UnitId, quest: u32, bit: u8) -> bool {
        false
    }
    /// player data +0x48 (D2MOO `dwUniqueId`), the GUID
    /// the player's own portal.
    fn player_portal_guid(&self, player: UnitId) -> u32 {
        0
    }
    /// `(game +0xBC + 4 · act, level, 0, &x, &y, 3)`
    /// level's spawn point for tile index 0 and free-point size 3
    fn level_spawn_point(&mut self, level: u32) -> Option<(RoomId, i32, i32)> {
        None
    }
    /// `(game, from, to, P)`: the quest change-level hook
    fn quest_level_change(&mut self, player: UnitId, from: u32, to: u32) {}
    /// Unit size of the player.
    fn player_size(&self, player: UnitId) -> i32 {
        2
    }
    /// `(game, P, 0, 2, x, y, 0)`: the player's walk-mode
    /// request toward (x, y) (player-mode spec).
    fn player_mode_xy(&mut self, player: UnitId, mode: u8, x: i32, y: i32) {}
    /// Remove a portal: `(room, 2, GUID)`, free
    /// `(room, 1)` (12 rule 12).
    fn remove_portal(&mut self, object: UnitId) {}
    /// `(game, L)`: the Act V quest hook on the partner.
    fn portal_act5_hook(&mut self, partner: UnitId) {}
    /// 12 rule 13: state 102 (`just_portaled`) on P until `expire`
    /// (stat list `(pool, 2, expire, 0, P's type)`, event 12
    /// P at `expire`, remove callback, attach).
    fn just_portaled(&mut self, player: UnitId, expire: i32) {}
}

/// Operate 8 (10).
pub fn door<W: ObjectHost>(
    ctl: &mut ObjectControl,
    t: &ObjectTables,
    w: &mut W,
    op: &Operate,
) -> Result<i32, ObjectError> {
    let obj = op.object;
    // Rule 1 (host clock, edge case 9).
    // 10: unsigned 32-bit compare against (+0xD4 + 500) mod 2^32.
    let now = w.host_tick();
    if now < ctl.get(obj)?.last_tick.wrapping_add(DOOR_DEBOUNCE) {
        return Ok(1);
    }
    // Rule 2.
    let mode = w.mode(obj);
    match mode {
        0 => open_door(ctl, t, w, obj, now)?,
        6 => {
            // 10: the key test with no assassin exemption; with no
            // operator it is fatal as for chests (rule 6
            // unreachable with live data).
            let Some(p) = op.operator else {
                return Err(ObjectError::KeyTestNoUnit);
            };
            if !w.key_test(p) {
                w.sound(p, sound::LOCKED, None, false);
                return Ok(1);
            }
            open_door(ctl, t, w, obj, now)?;
        }
        2 | 5 => {
            if !w.footprint_collides(obj, DOOR_CLEAR_MASK) {
                let (x, y) = w.position(obj);
                w.stamp_footprint(obj, w.room(obj), x, y);
                set_object_mode(ctl, t, w, obj, 0)?;
                ctl.get_mut(obj)?.last_tick = now;
            } else if w.footprint_collides(obj, DOOR_CORPSE_MASK) {
                set_object_mode(ctl, t, w, obj, 4)?;
                ctl.get_mut(obj)?.last_tick = now;
            } else if mode != 5 {
                set_object_mode(ctl, t, w, obj, 5)?;
                ctl.get_mut(obj)?.last_tick = now;
            }
        }
        // 1, 3, 4: nothing; > 6: return 1.
        _ => {}
    }
    // Rule 3.
    Ok(1)
}

/// 10 rule 2, mode 0: free the footprint, mode 2, store the tick.
fn open_door<W: ObjectHost>(
    ctl: &mut ObjectControl,
    t: &ObjectTables,
    w: &mut W,
    obj: UnitId,
    now: u32,
) -> Result<(), ObjectError> {
    w.free_footprint(obj);
    set_object_mode(ctl, t, w, obj, 2)?;
    ctl.get_mut(obj)?.last_tick = now;
    Ok(())
}

/// Operate 11 (13).
pub fn torch<W: ObjectHost>(
    ctl: &mut ObjectControl,
    t: &ObjectTables,
    w: &mut W,
    op: &Operate,
) -> Result<i32, ObjectError> {
    let obj = op.object;
    match w.mode(obj) {
        0 => {
            set_object_mode(ctl, t, w, obj, 1)?;
            clear_selectable(w, obj);
        }
        1 | 2 => set_object_mode(ctl, t, w, obj, 0)?,
        _ => {}
    }
    Ok(1)
}

/// Operate 15 (12 rules 1–14). Every path returns 0
/// result is always given (`Some`).
pub fn portal<W: ObjectHost>(
    ctl: &mut ObjectControl,
    t: &ObjectTables,
    w: &mut W,
    op: &Operate,
) -> Result<Option<i32>, ObjectError> {
    let obj = op.object;
    // Rule 5: P must exist and be a player, else fatal
    // monsters are stopped by 7.1 (`MonsterOK` 0) before this.
    let Some(p) = op
        .operator
        .filter(|&p| matches!(w.operator(p), Operator::Player(_)))
    else {
        return Err(ObjectError::PortalOperator);
    };
    // Rule 1: busy (interact info
    // cursor item, player data +0x4C).
    if w.interact_active(p) || w.cursor_item(p) || w.player_busy(p) {
        return Ok(Some(0));
    }
    if w.iterate_players(p) != 0 {
        // Owner: the timer argument's GUID.
        let owner = ctl.get(obj)?.owner;
        if owner != Some(w.guid(p) as i32) {
            return Ok(Some(0));
        }
    }
    // Rule 2 (host clock, edge case 9).
    // 10 last paragraph: (hostile + 5000) mod 2^32, unsigned compare.
    if w.host_tick() < w.hostile_time(p).wrapping_add(PORTAL_HOSTILE_DELAY) {
        w.sound(p, sound::PORTAL_REFUSED, None, false);
        return Ok(Some(0));
    }
    portal_travel(ctl, t, w, op.class, obj, p).map(Some)
}

/// The refusal of 12 rules 4 and 7: sound 19 on P (target P), 0.
fn refuse<W: ObjectHost>(w: &mut W, p: UnitId) -> i32 {
    w.sound(p, sound::PORTAL_REFUSED, Some(p), false);
    0
}

/// 12 rules 4–13 (open question 7 answered).
fn portal_travel<W: ObjectHost>(
    ctl: &mut ObjectControl,
    t: &ObjectTables,
    w: &mut W,
    class: u16,
    obj: UnitId,
    p: UnitId,
) -> Result<i32, ObjectError> {
    // Rule 4: the owner gate.
    let o = ctl.get(obj)?.owner.unwrap_or(-1);
    if o != -1 && o != w.guid(p) as i32 {
        let party = w.party_id(p);
        if party == 0xFFFF {
            return Ok(refuse(w, p));
        }
        if let Some(owner) = w.find_player(o as u32) {
            if w.party_id(owner) != party {
                return Ok(refuse(w, p));
            }
        }
    }
    // Rule 6.
    let partner = w.portal_partner(obj);
    let dest = u32::from(ctl.get(obj)?.interact);
    let u = w.player_portal_guid(p);
    // Rule 7, only when O has a room.
    if w.room(obj).is_some() {
        if !w.has_quest_record(p) {
            return Ok(refuse(w, p));
        }
        let Some(d) = t.leveldefs.get(dest as usize) else {
            return Ok(refuse(w, p));
        };
        if class == TOWN_PORTAL_CLASS {
            if let Some(l) = partner {
                if u != w.guid(obj) && u != w.guid(l) {
                    let q = if w.expansion() {
                        d.questflagex
                    } else {
                        d.questflag
                    };
                    if q > 0 && !w.player_quest_bit(p, q, 0) {
                        return Ok(refuse(w, p));
                    }
                }
            }
        }
    }
    // Rule 8: the destination.
    let (room, x, y) = match partner {
        Some(l) => {
            let (x, y) = w.position(l);
            let room = w.room(l).ok_or(ObjectError::NoPortalDestination(obj))?;
            (room, x, y)
        }
        None => w
            .level_spawn_point(dest)
            .ok_or(ObjectError::NoPortalDestination(obj))?,
    };
    // Rule 9.
    ctl.get_mut(obj)?.portal_flags |= 5;
    if let Some(pr) = w.room(p) {
        if w.in_town(pr) {
            let from = w.level(p).unwrap_or(0);
            let to = w.room_level(room).unwrap_or(0);
            w.quest_level_change(p, from, to);
        }
    }
    // Rule 10.
    let size = w.player_size(p);
    let Some((room, x, y)) = w.free_point(room, x, y, size, PORTAL_SPOT_MASK) else {
        return Ok(0);
    };
    // Rule 11.
    if !w.place_unit(p, room, x, y) {
        return Err(ObjectError::PortalPlacement(p));
    }
    w.sound(p, sound::PORTAL, None, false);
    w.player_mode_xy(p, 2, x + 5, y + 5);
    let msg = crate::path::walk::messages::player_stop(
        0,
        w.guid(p),
        1,
        (x + 5) as u16,
        (y + 5) as u16,
        0,
        0,
    );
    w.send(p, &msg);
    // Rule 12.
    match partner {
        Some(l) if class != PERMANENT_PORTAL_CLASS && u == w.guid(l) => {
            w.remove_portal(obj);
            w.portal_act5_hook(l);
            w.remove_portal(l);
        }
        Some(_) if class == TOWN_PORTAL_CLASS => {}
        _ => {
            if w.mode(obj) == 1 {
                schedule_endanim(w, t.object(class)?, obj);
            }
        }
    }
    // Rule 13.
    let expire = w.frame() + w.tick_rate().ticks(JUST_PORTALED_FRAMES);
    w.just_portaled(p, expire);
    Ok(0)
}

/// The well mode rule (11 rule 3, event 2): `c ≤ M` and `c mod (M / 2) =
/// 0` → mode 2 − c / `Parm2`.
fn well_mode(c: u32, parm2: u32) -> Result<Option<u8>, ObjectError> {
    let m = parm2.wrapping_mul(2);
    if c > m {
        return Ok(None);
    }
    // `Parm2` = 0 divides by zero in the original (a crash): fatal here.
    let half = m / 2;
    if half == 0 {
        return Err(ObjectError::WellCharges(c as u8));
    }
    if !c.is_multiple_of(half) {
        return Ok(None);
    }
    Ok(Some(2u32.wrapping_sub(c / parm2) as u8))
}

/// (11 rule 2): heal one vital stat. Returns whether it
/// wrote (the total was below its maximum).
fn heal_vital<W: ObjectHost>(w: &mut W, p: UnitId, id: u16, max_id: u16, parm1: u32) -> bool {
    let cur = w.vital_stat(p, id);
    let max = w.vital_stat(p, max_id);
    if cur >= max {
        return false;
    }
    let v = cur.wrapping_add(max.wrapping_mul(parm1) >> 8).min(max);
    w.set_vital_stat(p, id, v);
    // rule 5: "used" is set by the write, also when
    // value is unchanged.
    true
}

/// Operate 22 (11). Returns 0 always (edge case 10).
pub fn well<W: ObjectHost>(
    ctl: &mut ObjectControl,
    t: &ObjectTables,
    w: &mut W,
    op: &Operate,
) -> Result<i32, ObjectError> {
    let obj = op.object;
    let o = t.object(op.class)?;
    // Rule 1.
    let c = ctl.get(obj)?.interact;
    if c == 0 {
        return Ok(0);
    }
    // Rule 2.
    let Some(p) = op.operator else {
        return Ok(0);
    };
    let mut used = false;
    if o.parm3 & 2 != 0 {
        used |= heal_vital(w, p, stat::LIFE, stat::MAX_LIFE, o.parm1);
    }
    if o.parm3 & 1 != 0 {
        used |= heal_vital(w, p, stat::MANA, stat::MAX_MANA, o.parm1);
    }
    used |= heal_vital(w, p, stat::STAMINA, stat::MAX_STAMINA, o.parm1);
    for s in WELL_REMOVED_STATES {
        used |= w.remove_state_list(p, s);
    }
    used |= w.cure_states(p);
    used |= w.well_heal_pets(p, obj);
    // Rule 3.
    if !used {
        return Ok(0);
    }
    let c = c - 1;
    if let Some(mode) = well_mode(u32::from(c), o.parm2)? {
        set_object_mode(ctl, t, w, obj, mode)?;
    }
    ctl.get_mut(obj)?.interact = c;
    let at = w
        .frame()
        .wrapping_add(w.tick_rate().ticks(o.parm0 as i32))
        .wrapping_add(1);
    w.schedule(obj, super::oevent::WELL_REFILL, at);
    Ok(0)
}

/// Event 2 (11): refill one charge.
pub fn well_refill<W: ObjectHost>(
    ctl: &mut ObjectControl,
    t: &ObjectTables,
    w: &mut W,
    obj: UnitId,
) -> Result<(), ObjectError> {
    let d = ctl.get(obj)?;
    let class = d.class;
    let c = u32::from(d.interact);
    let parm2 = t.object(class)?.parm2;
    if parm2 == 0 || c / parm2 >= 2 {
        return Err(ObjectError::WellCharges(c as u8));
    }
    let c = c + 1;
    if c > parm2.wrapping_mul(2) {
        return Err(ObjectError::WellCharges(c as u8));
    }
    if let Some(mode) = well_mode(c, parm2)? {
        // rule 7: the ordinary mode set (queues, flag
        // 0x1); the explicit queue and flag below repeat it.
        set_mode(t, w, obj, class, mode, true)?;
    }
    ctl.get_mut(obj)?.interact = c as u8;
    w.queue_update(obj);
    let f = w.flags(obj);
    w.set_flags(obj, f | oflags::CHANGED);
    Ok(())
}
