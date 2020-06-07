use crate::common::consts::*;
use crate::common::*;
use smash::app::{self, lua_bind::*};
use smash::hash40;
use smash::lib::lua_const::*;

static mut BUFFERED_OPTION: Mash = Mash::None;
static mut MASH_IN_NEUTRAL: bool = false;

pub unsafe fn buffer_option(option: Mash)
{
    BUFFERED_OPTION = option;
}

pub unsafe fn set_neutral_mash(value: bool)
{
    MASH_IN_NEUTRAL = value;
}

pub unsafe fn get_attack_air_kind(
    module_accessor: &mut app::BattleObjectModuleAccessor,
) -> Option<i32> {
    if !is_training_mode() {
        return None;
    }

    if !is_operation_cpu(module_accessor) {
        return None;
    }

    if MENU.mash_state == Mash::Attack {
        MENU.mash_attack_state.into_attack_air_kind()
    } else if MENU.mash_state == Mash::Random {
        Some(app::sv_math::rand(hash40("fighter"), 5) + 1)
    } else {
        None
    }
}

pub unsafe fn get_command_flag_cat(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    category: i32,
    flag: &mut i32,
) {
    if !is_training_mode() {
        return;
    }

    // Only do once per frame
    if category != FIGHTER_PAD_COMMAND_CATEGORY1 {
        return;
    }

    check_mash_toggle(module_accessor);

    if !is_operation_cpu(module_accessor) {
        return;
    }

    if !(is_in_hitstun(module_accessor)
    || is_in_landing(module_accessor)
    || is_in_shieldstun(module_accessor)
    || is_in_footstool(module_accessor)
    || BUFFERED_OPTION != Mash::None
    || StatusModule::status_kind(module_accessor) == FIGHTER_STATUS_KIND_CLIFF_ROBBED){
        return;
    }

    // Prefer buffered option over default
    let mut option = BUFFERED_OPTION;
    if option == Mash::None
    {
        option = MENU.mash_state
    }

    // Reset buffer unless full mash flag
    if !MASH_IN_NEUTRAL {
        BUFFERED_OPTION = Mash::None;
    }

    match option {
        Mash::Airdodge => {
            *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_AIR_ESCAPE;
        }
        Mash::Jump => {
            if !is_in_landing(module_accessor) {
                *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_JUMP_BUTTON;
            }
        }
        Mash::Spotdodge => {
            *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE;
        }
        Mash::RollForward => {
            *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_F;
        }
        Mash::RollBack => {
            *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_B;
        }
        Mash::Attack => {
            perform_attack(module_accessor,flag, MENU.mash_attack_state);
        }
        Mash::Random => {
            if is_airborne(module_accessor) {
                let random_commands = vec![
                    *FIGHTER_PAD_CMD_CAT1_FLAG_AIR_ESCAPE,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_JUMP_BUTTON,
                    // one for each aerial
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_N,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_N,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_N,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_N,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_N,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_N,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_S,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_HI,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_LW,
                ];

                let random_cmd_index =
                    app::sv_math::rand(hash40("fighter"), random_commands.len() as i32)
                        as usize;

                *flag |= random_commands[random_cmd_index];
            } else if is_grounded(module_accessor) {
                let random_commands = vec![
                    *FIGHTER_PAD_CMD_CAT1_FLAG_JUMP_BUTTON,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_N,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_S3,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_HI3,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_LW3,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_S4,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_HI4,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_LW4,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_HI,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_S,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_HI,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_LW,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_CATCH,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_F,
                    *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_B,
                ];

                let random_cmd_index =
                    app::sv_math::rand(hash40("fighter"), random_commands.len() as i32)
                        as usize;

                *flag |= random_commands[random_cmd_index];
            }
        }
        _ => (),
    }
}

unsafe fn check_mash_toggle(
    module_accessor: &mut app::BattleObjectModuleAccessor,
){
    // Grab + Dpad left -> start mashing
    if ControlModule::check_button_on(module_accessor, *CONTROL_PAD_BUTTON_CATCH)
    && ControlModule::check_button_trigger(module_accessor, *CONTROL_PAD_BUTTON_APPEAL_S_L)
    {
        buffer_option(MENU.mash_state);
        println!("[Training Modpack] Now Mashing {}", MENU.mash_state as u32);
        set_neutral_mash(true);
        return;
    }

     // Grab + Dpad right -> stop mashing
     if ControlModule::check_button_on(module_accessor, *CONTROL_PAD_BUTTON_CATCH)
     && ControlModule::check_button_trigger(module_accessor, *CONTROL_PAD_BUTTON_APPEAL_S_R)
     {
        println!("[Training Modpack] Stop Mashing");
        buffer_option(Mash::None);
        set_neutral_mash(false);
        return;
     }
}

pub unsafe fn check_button_on(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    button: i32,
) -> Option<bool> {
    if !is_training_mode() {
        return None;
    }

    if !is_operation_cpu(module_accessor) {
        return None;
    }

    if ![*CONTROL_PAD_BUTTON_GUARD_HOLD, *CONTROL_PAD_BUTTON_GUARD].contains(&button) {
        return None;
    }

    if MENU.mash_state != Mash::Airdodge {
        return None;
    }

    // Buffer only in hitstun/landing/footstool
    if !is_in_hitstun(module_accessor)
    && !is_in_landing(module_accessor)
    && !is_in_footstool(module_accessor){
        return None;
    }

    Some(true)
}
