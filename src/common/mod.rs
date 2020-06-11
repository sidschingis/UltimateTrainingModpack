pub mod consts;
pub mod frame_counter;

use crate::common::consts::*;
use smash::app::{self, lua_bind::*};
use smash::hash40;
use smash::lib::lua_const::*;

pub static mut MENU_STRUCT: consts::TrainingModpackMenu = consts::TrainingModpackMenu {
    hitbox_vis: true,
    di_state: DirectionalInfluence::None,
    mash_attack_state: Attack::Nair,
    ledge_state: LedgeOption::Random,
    tech_state: TechOption::Random,
    mash_state: Mash::None,
    shield_state: Shield::None,
    defensive_state: Defensive::Random,
    oos_offset: 0,
};

pub static MENU: &'static mut consts::TrainingModpackMenu = unsafe { &mut MENU_STRUCT };

pub static mut FIGHTER_MANAGER_ADDR: usize = 0;

extern "C" {
    #[link_name = "\u{1}_ZN3app9smashball16is_training_modeEv"]
    pub fn is_training_mode() -> bool;

//#[link_name = "\u{1}_ZN3app7utility8get_kindEPKNS_26BattleObjectModuleAccessorE"]
//pub fn get_kind(module_accessor: &mut app::BattleObjectModuleAccessor) -> i32;
}

pub fn get_category(module_accessor: &mut app::BattleObjectModuleAccessor) -> i32 {
    return (module_accessor.info >> 28) as u8 as i32;
}
pub unsafe fn is_fighter(module_accessor: &mut app::BattleObjectModuleAccessor) -> bool {
    get_category(module_accessor) == BATTLE_OBJECT_CATEGORY_FIGHTER
}

pub unsafe fn is_operation_cpu(module_accessor: &mut app::BattleObjectModuleAccessor) -> bool {
    if !is_fighter(module_accessor) {
        return false;
    }

    let entry_id_int =
        WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as i32;
    let entry_id = app::FighterEntryID(entry_id_int);
    let mgr = *(FIGHTER_MANAGER_ADDR as *mut *mut app::FighterManager);
    let fighter_information =
        FighterManager::get_fighter_information(mgr, entry_id) as *mut app::FighterInformation;

    FighterInformation::is_operation_cpu(fighter_information)
}

pub unsafe fn is_idle(module_accessor: &mut app::BattleObjectModuleAccessor) -> bool {
    let status_kind = StatusModule::status_kind(module_accessor);
    status_kind == FIGHTER_STATUS_KIND_WAIT
}

pub unsafe fn is_in_hitstun(module_accessor: &mut app::BattleObjectModuleAccessor) -> bool {
    let status_kind = StatusModule::status_kind(module_accessor);
    (*FIGHTER_STATUS_KIND_DAMAGE..=*FIGHTER_STATUS_KIND_DAMAGE_FALL).contains(&status_kind)
}

pub unsafe fn is_shielding(module_accessor: &mut app::BattleObjectModuleAccessor) -> bool {
    let status_kind = StatusModule::status_kind(module_accessor);
    status_kind == FIGHTER_STATUS_KIND_GUARD
}

pub unsafe fn is_in_shieldstun(module_accessor: &mut app::BattleObjectModuleAccessor) -> bool {
    let status_kind = StatusModule::status_kind(module_accessor);
    let prev_status = StatusModule::prev_status_kind(module_accessor, 0);
    // If we are taking shield damage or we are droping shield from taking shield damage we are in hitstun
    status_kind == FIGHTER_STATUS_KIND_GUARD_DAMAGE
        || (prev_status == FIGHTER_STATUS_KIND_GUARD_DAMAGE
            && status_kind == FIGHTER_STATUS_KIND_GUARD_OFF)
}

pub unsafe fn is_in_landing(module_accessor: &mut app::BattleObjectModuleAccessor) -> bool {
    let status_kind = StatusModule::status_kind(module_accessor);
    (*FIGHTER_STATUS_KIND_LANDING..=*FIGHTER_STATUS_KIND_LANDING_DAMAGE_LIGHT)
        .contains(&status_kind)
}

pub unsafe fn is_in_footstool(module_accessor: &mut app::BattleObjectModuleAccessor) -> bool {
    let status_kind = StatusModule::status_kind(module_accessor);
    (*FIGHTER_STATUS_KIND_TREAD_DAMAGE..=*FIGHTER_STATUS_KIND_TREAD_FALL).contains(&status_kind)
}

pub unsafe fn perform_defensive_option(
    _module_accessor: &mut app::BattleObjectModuleAccessor,
    flag: &mut i32,
) {
    match MENU.defensive_state {
        Defensive::Random => {
            let random_cmds = vec![
                *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE,
                *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_F,
                *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_B,
                *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_N,
            ];

            let random_cmd_index =
                app::sv_math::rand(hash40("fighter"), random_cmds.len() as i32) as usize;
            *flag |= random_cmds[random_cmd_index];
        }
        Defensive::Roll => {
            if app::sv_math::rand(hash40("fighter"), 2) == 0 {
                *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_F;
            } else {
                *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_B;
            }
        }
        Defensive::Spotdodge => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE,
        Defensive::Jab => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_N,
        _ => (),
    }
}

pub unsafe fn perform_attack(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    flag: &mut i32,
    attack: Attack,
) {
    match attack {
        Attack::Nair | Attack::Fair | Attack::Bair | Attack::UpAir | Attack::Dair =>{
            // Nair will be replaced by the correct attack through get_attack_air_kind
            *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_N;
            // If we are grounded we also need to jump
            if is_grounded(module_accessor) {
                *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_JUMP_BUTTON;
            }
        }

        Attack::NeutralB => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_N,
        Attack::SideB => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_S,
        Attack::UpB => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_HI,
        Attack::DownB => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_LW,
        Attack::UpSmash => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_HI4,
        Attack::Grab => *flag |= *FIGHTER_PAD_CMD_CAT1_FLAG_CATCH,
    }
}
