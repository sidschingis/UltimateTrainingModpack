use crate::common::consts::*;
use crate::common::*;
use crate::training::fast_fall;
use crate::training::shield;
use smash::app::{self, lua_bind::*};
use smash::hash40;
use smash::lib::lua_const::*;

static mut BUFFERED_ACTION: Mash = Mash::None;
static mut BUFFERED_ATTACK: Attack = Attack::Nair;

static mut CURRENT_FOLLOW_UP: Action = Action::None;

pub fn buffer_action(action: Mash) {
    unsafe {
        if BUFFERED_ACTION != Mash::None {
            return;
        }
    }

    unsafe {
        BUFFERED_ACTION = action;
    }
}

pub fn buffer_follow_up(action: Mash) {
    unsafe {
        if BUFFERED_ACTION != Mash::None {
            return;
        }
    }

    unsafe {
        BUFFERED_ACTION = action;
        CURRENT_FOLLOW_UP = Action::None;

        println!("buffering followup");
    }
}

pub fn get_current_buffer() -> Mash {
    unsafe { BUFFERED_ACTION }
}

pub fn set_attack(attack: Attack) {
    unsafe {
        if BUFFERED_ATTACK == attack {
            return;
        }
    }
    unsafe {
        BUFFERED_ATTACK = attack;
        println!("Setting Attack {}", attack as i32);
    }
}

pub fn get_current_attack() -> Attack {
    unsafe { BUFFERED_ATTACK }
}

pub fn reset() {
    unsafe {
        BUFFERED_ACTION = Mash::None;
    }
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

    BUFFERED_ATTACK.into_attack_air_kind()
}

pub unsafe fn get_command_flag_cat(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    category: i32,
) -> i32 {
    // Only do once per frame
    if category != FIGHTER_PAD_COMMAND_CATEGORY1 {
        return 0;
    }

    if !is_training_mode() {
        return 0;
    }

    if !is_operation_cpu(module_accessor) {
        return 0;
    }

    check_buffer(module_accessor);

    perform_action(module_accessor)
}

unsafe fn check_buffer(module_accessor: &mut app::BattleObjectModuleAccessor) {
    if BUFFERED_ACTION != Mash::None {
        return;
    }

    if !is_in_hitstun(module_accessor) && MENU.mash_in_neutral != OnOff::On {
        return;
    }

    let mut action = MENU.mash_state;

    if action == Mash::Random {
        let mut random_cmds = vec![Mash::Jump, Mash::Attack];

        if is_airborne(module_accessor) {
            random_cmds.push(Mash::Airdodge);
        }

        if is_grounded(module_accessor) {
            random_cmds.push(Mash::RollBack);
            random_cmds.push(Mash::RollForward);
            random_cmds.push(Mash::Spotdodge);
        }

        let random_cmd_index =
            app::sv_math::rand(hash40("fighter"), random_cmds.len() as i32) as usize;

        action = random_cmds[random_cmd_index];
    }

    buffer_action(action);
    set_attack(MENU.mash_attack_state);
}

unsafe fn perform_action(module_accessor: &mut app::BattleObjectModuleAccessor) -> i32 {
    match BUFFERED_ACTION {
        Mash::Airdodge => {
            // Shield if grounded instead
            if is_grounded(module_accessor) {
                reset();
                buffer_action(Mash::Shield);
                return 0;
            }

            return get_flag(
                module_accessor,
                *FIGHTER_STATUS_KIND_ESCAPE_AIR,
                *FIGHTER_PAD_CMD_CAT1_FLAG_AIR_ESCAPE,
            );
        }
        Mash::Jump => {
            return update_jump_flag(module_accessor);
        }
        Mash::Spotdodge => {
            return get_flag(
                module_accessor,
                *FIGHTER_STATUS_KIND_ESCAPE,
                *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE,
            );
        }
        Mash::RollForward => {
            return get_flag(
                module_accessor,
                *FIGHTER_STATUS_KIND_ESCAPE_F,
                *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_F,
            );
        }
        Mash::RollBack => {
            return get_flag(
                module_accessor,
                *FIGHTER_STATUS_KIND_ESCAPE_B,
                *FIGHTER_PAD_CMD_CAT1_FLAG_ESCAPE_B,
            );
        }
        Mash::Attack => {
            return get_attack_flag(module_accessor);
        }
        Mash::Shield => {
            /*
            Doesn't actually cause the shield, but will clear the buffer once shield is possible.
            Shield hold is performed trough shield::should_hold_shield
            */
            // return get_flag(
            //     module_accessor,
            //     *FIGHTER_STATUS_KIND_GUARD_ON,
            //     *FIGHTER_PAD_CMD_CAT1_FLAG_AIR_ESCAPE,
            // );
            return get_flag(
                module_accessor,
                *FIGHTER_STATUS_KIND_GUARD_ON,
                *FIGHTER_PAD_CMD_CAT1_FLAG_AIR_ESCAPE,
            );
        }
        _ => return 0,
    }
}

unsafe fn update_jump_flag(module_accessor: &mut app::BattleObjectModuleAccessor) -> i32 {
    let check_flag: i32;

    if is_grounded(module_accessor) {
        check_flag = *FIGHTER_STATUS_KIND_JUMP_SQUAT;
    } else if is_airborne(module_accessor) {
        check_flag = *FIGHTER_STATUS_KIND_JUMP_AERIAL;
    } else {
        check_flag = *FIGHTER_STATUS_KIND_JUMP;
    }

    return get_flag(
        module_accessor,
        check_flag,
        *FIGHTER_PAD_CMD_CAT1_FLAG_JUMP_BUTTON,
    );
}

unsafe fn get_attack_flag(module_accessor: &mut app::BattleObjectModuleAccessor) -> i32 {
    use Attack::*;

    let action_flag: i32;
    let status: i32;

    match BUFFERED_ATTACK {
        Nair | Fair | Bair | UpAir | Dair => {
            return get_aerial_flag(module_accessor, BUFFERED_ATTACK);
        }
        NeutralB => {
            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_N;
            status = *FIGHTER_STATUS_KIND_SPECIAL_N;
        }
        SideB => {
            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_S;
            status = *FIGHTER_STATUS_KIND_SPECIAL_S;
        }
        UpB => {
            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_HI;
            status = *FIGHTER_STATUS_KIND_SPECIAL_HI;
        }
        DownB => {
            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_SPECIAL_LW;
            status = *FIGHTER_STATUS_KIND_SPECIAL_LW;
        }
        UpSmash => {
            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_HI4;
            status = *FIGHTER_STATUS_KIND_ATTACK_HI4_START;
        }
        FSmash => {
            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_S4;
            status = *FIGHTER_STATUS_KIND_ATTACK_S4_START;
        }
        DSmash => {
            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_LW4;
            status = *FIGHTER_STATUS_KIND_ATTACK_LW4_START;
        }
        Grab => {
            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_CATCH;
            status = *FIGHTER_STATUS_KIND_CATCH;
        }
        Jab => {
            // Prevent nair when airborne
            if !is_grounded(module_accessor) {
                return 0;
            }

            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_N;
            status = *FIGHTER_STATUS_KIND_ATTACK;
        }
        Ftilt => {
            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_S3;
            status = *FIGHTER_STATUS_KIND_ATTACK_S3;
        }
        Utilt => {
            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_HI3;
            status = *FIGHTER_STATUS_KIND_ATTACK_HI3;
        }
        Dtilt => {
            action_flag = *FIGHTER_PAD_CMD_CAT1_FLAG_ATTACK_LW3;
            status = *FIGHTER_STATUS_KIND_ATTACK_LW3;
        }
        _ => return 0,
    }

    return get_flag(module_accessor, status, action_flag);
}

unsafe fn get_aerial_flag(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    attack: Attack,
) -> i32 {
    let mut flag: i32 = 0;

    // If we are grounded we also need to jump
    if is_grounded(module_accessor) {
        flag += *FIGHTER_PAD_CMD_CAT1_FLAG_JUMP_BUTTON;

        // Delay attack until we are airborne to get a full hop
        if MENU.full_hop == OnOff::On {
            return flag;
        }
    }

    let status = *FIGHTER_STATUS_KIND_ATTACK_AIR;

    if MENU.falling_aerials == OnOff::On && !fast_fall::is_falling(module_accessor) {
        return flag;
    }

    let action_flag: i32;

    match attack {
        Attack::Nair => {
            action_flag = *FIGHTER_COMMAND_ATTACK_AIR_KIND_N;
        }
        Attack::Fair => {
            // For some reason the game doesn't trigger the fair correctly
            // action_flag = *FIGHTER_COMMAND_ATTACK_AIR_KIND_F;
            action_flag = *FIGHTER_COMMAND_ATTACK_AIR_KIND_N;
        }
        Attack::Bair => {
            action_flag = *FIGHTER_COMMAND_ATTACK_AIR_KIND_B;
        }
        Attack::UpAir => {
            // For some reason the game doesn't trigger the uair correctly
            // action_flag = *FIGHTER_COMMAND_ATTACK_AIR_KIND_HI;
            action_flag = *FIGHTER_COMMAND_ATTACK_AIR_KIND_N;
        }
        Attack::Dair => {
            action_flag = *FIGHTER_COMMAND_ATTACK_AIR_KIND_LW;
        }
        _ => {
            action_flag = 0;
        }
    }

    flag |= get_flag(module_accessor, status, action_flag);

    flag
}

/**
 * Returns the flag and resets, once the action is performed
 */
unsafe fn get_flag(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    status: i32,
    action_flag: i32,
) -> i32 {
    if StatusModule::status_kind(module_accessor) == status {
        // Reset Buffer
        reset();
        handle_follow_up();
    }

    return action_flag;
}

fn handle_follow_up() {
    let action;

    unsafe {
        action = CURRENT_FOLLOW_UP;
    }

    match action {
        Action::None => return,
        Action::Airdodge => buffer_follow_up(Mash::Airdodge),
        Action::Jump => buffer_follow_up(Mash::Jump),
        Action::Spotdodge => buffer_follow_up(Mash::Spotdodge),
        Action::RollForward => buffer_follow_up(Mash::RollForward),
        Action::RollBack => buffer_follow_up(Mash::RollBack),
        _ => handle_attack_follow_up(action),
    }
}

fn handle_attack_follow_up(action: Action) {
    buffer_follow_up(Mash::Attack);

    match action {
        Action::Nair => set_attack(Attack::Nair),
        Action::Fair => set_attack(Attack::Fair),
        Action::Bair => set_attack(Attack::Bair),
        Action::UpAir => set_attack(Attack::UpAir),
        Action::Dair => set_attack(Attack::Dair),
        Action::NeutralB => set_attack(Attack::NeutralB),
        Action::SideB => set_attack(Attack::SideB),
        Action::UpB => set_attack(Attack::UpB),
        Action::DownB => set_attack(Attack::DownB),
        Action::UpSmash => set_attack(Attack::UpSmash),
        Action::FSmash => set_attack(Attack::FSmash),
        Action::DSmash => set_attack(Attack::DSmash),
        Action::Grab => set_attack(Attack::Grab),
        Action::Jab => set_attack(Attack::Jab),
        Action::Ftilt => set_attack(Attack::Ftilt),
        Action::Utilt => set_attack(Attack::Utilt),
        Action::Dtilt => set_attack(Attack::Dtilt),
        _ => {}
    }
}

pub unsafe fn perform_defensive_option() {
    reset();

    let mut shield_suspension_frames = 60;

    match MENU.defensive_state {
        Defensive::Random => {
            let random_cmds = vec![
                Mash::Spotdodge,
                Mash::RollBack,
                Mash::RollForward,
                Mash::Attack,
            ];

            let random_cmd_index =
                app::sv_math::rand(hash40("fighter"), random_cmds.len() as i32) as usize;

            buffer_action(random_cmds[random_cmd_index]);
            set_attack(Attack::Jab);
        }
        Defensive::Roll => {
            if app::sv_math::rand(hash40("fighter"), 2) == 0 {
                buffer_action(Mash::RollForward);
            } else {
                buffer_action(Mash::RollBack);
            }
        }
        Defensive::Spotdodge => buffer_action(Mash::Spotdodge),
        Defensive::Jab => {
            buffer_action(Mash::Attack);
            set_attack(Attack::Jab);
        }
        Defensive::Shield => {
            shield_suspension_frames = 0;
            buffer_action(Mash::Shield);
        }
        _ => (shield_suspension_frames = 0),
    }

    // Suspend shield hold to allow for other defensive options
    shield::suspend_shield(shield_suspension_frames);
}
