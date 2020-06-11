use crate::common::consts::*;
use crate::common::*;
use crate::training::shield;
use smash::app::{self, lua_bind::*};
use smash::hash40;
use smash::lib::lua_const::*;

static mut BUFFERED_OPTION: Mash = Mash::None;
static mut MASH_IN_NEUTRAL: bool = false;

pub unsafe fn buffer_option(option: Mash) {
    BUFFERED_OPTION = option;
}

pub unsafe fn set_neutral_mash(value: bool) {
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

    match MENU.mash_state {
        Mash::Attack => {
            return MENU.mash_attack_state.into_attack_air_kind();
        }
        Mash::Random => {
            return Some(app::sv_math::rand(hash40("fighter"), 5) + 1);
        }
        _ => {
            return None;
        }
    }
}

pub unsafe fn get_command_flag_cat(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    category: i32,
    flag: &mut i32,
) {
    if category != FIGHTER_PAD_COMMAND_CATEGORY1 {
        return;
    }

    if !is_training_mode() {
        return;
    }

    if !is_operation_cpu(module_accessor) {
        return;
    }

    // Check for OOS delay
    if is_in_shieldstun(module_accessor) && !shield::allow_oos() {
        return;
    }

    if !(is_in_hitstun(module_accessor)
        || is_in_landing(module_accessor)
        || is_in_shieldstun(module_accessor)
        || is_in_footstool(module_accessor)
        || StatusModule::status_kind(module_accessor) == FIGHTER_STATUS_KIND_CLIFF_ROBBED)
    {
        return;
    }

    match MENU.mash_state {
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
            use Attack::*;

            match MENU.mash_attack_state {
                Nair | Fair | Bair | UpAir | Dair => {
                    *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_N;
                    // If we are performing the attack OOS we also need to jump
                    if is_in_shieldstun(module_accessor) {
                        *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_JUMP_BUTTON;
                    }
                }
                NeutralB => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_N,
                SideB => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_S,
                UpB => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_HI,
                DownB => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_LW,
                UpSmash => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_HI4,
                Grab => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_CATCH,
            }
        }
        Mash::Random => {
            let random_commands = get_random_command_list(module_accessor);

            if random_commands.len() == 0 {
                return;
            }

            let random_cmd_index =
                app::sv_math::rand(hash40("fighter"), random_commands.len() as i32) as usize;

            *flag |= random_commands[random_cmd_index];
        }
        _ => (),
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

unsafe fn get_random_command_list(
    module_accessor: &mut app::BattleObjectModuleAccessor,
) -> Vec<i32> {
    let situation_kind = StatusModule::situation_kind(module_accessor) as i32;

    if situation_kind == SITUATION_KIND_AIR {
        return vec![
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
    }

    if situation_kind == SITUATION_KIND_GROUND {
        return vec![
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
    }

    return vec![];
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

    if MENU.mash_state == Mash::Airdodge
        && !(is_in_hitstun(module_accessor)
            || is_in_landing(module_accessor)
            || is_in_footstool(module_accessor))
    {
        return Some(true);
    }

    Some(true)
}
