use crate::common::consts::*;
use crate::common::*;
use smash::app;
use smash::hash40;
use smash::lib::lua_const::*;
use smash::app::lua_bind::*;
use smash::app::sv_system;
use smash::lib::L2CValue;
use smash::lua2cpp::L2CFighterCommon;

// Toggle for infinite shield decay
static mut SHIELD_FLAG: bool = false;


// How many hits to hold shield until picking an OOS option
static mut MULTI_HIT_OFFSET : u8 = MENU.oos_offset;
//
static mut WAS_IN_SHIELDSTUN: bool = false;

unsafe fn set_shield_flag(value:bool){
    SHIELD_FLAG = value;
}

unsafe fn get_shield_flag() ->bool {
    SHIELD_FLAG
}

unsafe fn reset_multi_hit_offset(){
    MULTI_HIT_OFFSET = MENU.oos_offset + 1; // Menu starts with 0
}

pub unsafe fn get_command_flag_cat(module_accessor: &mut app::BattleObjectModuleAccessor) {
    if !is_training_mode() {
        return;
    }

    if !is_operation_cpu(module_accessor) {
        return;
    }

    //
    if is_neutral_pos(module_accessor)
    || is_in_hitstun(module_accessor){
        reset_multi_hit_offset();
    }

    // Reset when not shielding
    if !is_shielding(module_accessor){
        set_shield_flag(false);
    }
}

pub unsafe fn get_param_float(
    _module_accessor: &mut app::BattleObjectModuleAccessor,
    param_type: u64,
    param_hash: u64,
) -> Option<f32> {
    if !is_training_mode() {
        return None;
    }

    if !is_operation_cpu(_module_accessor) {
        return None;
    }

    handle_oos_offset(_module_accessor);

    if MENU.shield_state != Shield::Infinite {
        return None;
    }

    if get_shield_flag() {
        return None;
    }

    if param_type != hash40("common") {
        return None;
    }

    if param_hash == hash40("shield_dec1") {
        return Some(0.0);
    }

    if param_hash == hash40("shield_recovery1") {
        return Some(999.0);
    }

    // doesn't work, somehow. This parameter isn't checked?
    if param_hash == hash40("shield_damage_mul") {
        return Some(0.0);
    }

    None
}

unsafe fn handle_oos_offset(module_accessor: &mut app::BattleObjectModuleAccessor)
{
    if is_in_shieldstun(module_accessor)
    {
        if !WAS_IN_SHIELDSTUN {
            if MULTI_HIT_OFFSET > 0{
                MULTI_HIT_OFFSET -= 1;
            }
            frame_counter::start_counting();
        }

        WAS_IN_SHIELDSTUN = true;
        return;
    }

    if WAS_IN_SHIELDSTUN {
        println!("[Training Modpack] exited shield stun {}, {}", frame_counter::get_frame_count(), MULTI_HIT_OFFSET);
        frame_counter::stop_counting();
        frame_counter::reset_frame_count();
    }

    WAS_IN_SHIELDSTUN = false;
}

pub unsafe fn should_hold_shield(module_accessor: &mut app::BattleObjectModuleAccessor) -> bool {
    // We should not hold shield if the state doesn't require it
    if ![Shield::Hold, Shield::Infinite].contains(&MENU.shield_state) {
        return false;
    }

    // If we are not mashing attack then we will always hold shield
    if MENU.mash_state != Mash::Attack {
        return true;
    }

    // Hold shield while oos is not allowd
    if !allow_oos(){
        return true;
    }

    // If we are not in shield stun then we will always hold shield
    if !is_in_shieldstun(module_accessor) {
        return true;
    }

    // and our attack can be performed OOS
    if [Attack::NeutralB, Attack::SideB, Attack::DownB].contains(&MENU.mash_attack_state) {
        return false;
    }

    if MENU.mash_attack_state == Attack::Grab {
        return true;
    }

    false
}

#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_sub_guard_cont)]
pub unsafe fn handle_sub_guard_cont(fighter: &mut L2CFighterCommon) -> L2CValue {
    if !is_training_mode() {
        return original!()(fighter);
    }

    let module_accessor = sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    if !is_operation_cpu(module_accessor) {
        return original!()(fighter);
    }

    if StatusModule::prev_status_kind(module_accessor, 0) != FIGHTER_STATUS_KIND_GUARD_DAMAGE {
        return original!()(fighter);
    }

    // Continue with normal shield behavior
    if MENU.shield_state == Shield::Infinite {
        set_shield_flag(true);
    }

    if !allow_oos() {
        return original!()(fighter);
    }

    // OOS Options

    // Defensive

    if WorkModule::is_enable_transition_term(
        module_accessor,
        *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_ESCAPE,
    ) {
        handle_escape_mash(fighter);
    }

    // Offensive
    if MENU.mash_state == Mash::Attack {
        handle_attack_mash(module_accessor, fighter);
    }

    original!()(fighter)
}

pub unsafe fn allow_oos()->bool {
    // Delay OOS
    MULTI_HIT_OFFSET == 0
}

unsafe fn handle_escape_mash(fighter: &mut L2CFighterCommon)
{
    if MENU.mash_state == Mash::Spotdodge {
        fighter.fighter_base.change_status(
            L2CValue::new_int(*FIGHTER_STATUS_KIND_ESCAPE as u64),
            L2CValue::new_bool(true),
        );
        return;
    }

    if MENU.mash_state == Mash::RollForward {
        fighter.fighter_base.change_status(
            L2CValue::new_int(*FIGHTER_STATUS_KIND_ESCAPE_F as u64),
            L2CValue::new_bool(true),
        );
        return;
    }

    if MENU.mash_state == Mash::RollBack {
        fighter.fighter_base.change_status(
            L2CValue::new_int(*FIGHTER_STATUS_KIND_ESCAPE_B as u64),
            L2CValue::new_bool(true),
        );
        return;
    }
}

unsafe fn handle_attack_mash(module_accessor: &mut app::BattleObjectModuleAccessor,fighter: &mut L2CFighterCommon){

    match MENU.mash_attack_state {
        Attack::Grab => {
            //
            if WorkModule::get_int(
                module_accessor,
                *FIGHTER_INSTANCE_WORK_ID_INT_INVALID_CATCH_FRAME,
            ) != 0
            {
                return;
            }

            // Check Transition, shield uses it's own check because of the additional 4f window
            let can_transition = WorkModule::is_enable_transition_term(
                module_accessor,
                *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_CATCH,
            );
            if !can_transition {
                return;
            }

            fighter.fighter_base.change_status(
                L2CValue::new_int(*FIGHTER_STATUS_KIND_CATCH as u64),
                L2CValue::new_bool(true),
            );
        },
        Attack::UpB => {
            // Check Transition
            if !can_jump(module_accessor) {
                return;
            }

            fighter.fighter_base.change_status(
                L2CValue::new_int(*FIGHTER_STATUS_KIND_SPECIAL_HI as u64),
                L2CValue::new_bool(false),
            );
        },
        Attack::UpSmash => {
            // Check Transition
            if !can_jump(module_accessor) {
                return;
            }

            fighter.fighter_base.change_status(
                L2CValue::new_int(*FIGHTER_STATUS_KIND_ATTACK_HI4_START as u64),
                L2CValue::new_bool(false),
            );
        },
        _=> {},
    }
}

unsafe fn can_jump(module_accessor: &mut app::BattleObjectModuleAccessor) -> bool {
    WorkModule::is_enable_transition_term(
        module_accessor,
        *FIGHTER_STATUS_TRANSITION_TERM_ID_CONT_JUMP_SQUAT_BUTTON,
    )
}

pub unsafe fn check_button_on(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    button: i32,
) -> Option<bool> {
    if should_return_none_in_check_button(module_accessor, button) {
        return None;
    }

    Some(true)
}

pub unsafe fn check_button_off(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    button: i32,
) -> Option<bool> {
    if should_return_none_in_check_button(module_accessor, button) {
        return None;
    }

    Some(false)
}

unsafe fn should_return_none_in_check_button(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    button: i32,
)
->bool {
    if !is_training_mode() {
        return true;
    }

    if !is_operation_cpu(module_accessor) {
        return true;
    }

    if ![*CONTROL_PAD_BUTTON_GUARD_HOLD, *CONTROL_PAD_BUTTON_GUARD].contains(&button) {
        return true;
    }

    if !should_hold_shield(module_accessor) {
        return true;
    }

    false
}
