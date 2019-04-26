#include <switch.h>
#include "saltysd_core.h"
#include "saltysd_ipc.h"
#include "saltysd_dynamic.h"
#include "l2c_imports.h"
#include "l2c.h"

void get_lua_stack(L2CAgent* l2c_agent, int index, L2CValue* l2c_val) {
	asm("mov x8, %x0" : : "r"(l2c_val) : "x8" );
    lib_L2CAgent_pop_lua_stack(l2c_agent, index);
}
