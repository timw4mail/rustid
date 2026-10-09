#include "common/gestalt.h"

#if defined(__APPLE__) || defined(__MACOS__) || defined(TARGET_API_MAC_CARBON) || defined(macintosh) || defined(__Retro68__)
#include <Gestalt.h>
#include <Types.h>
#include <Multiverse.h>
#include <OSUtils.h>

bool classic_mac_is_gestalt_available(void) {
#if defined(__m68k__) || defined(__mc68000__)
    static short s_avail = -1;
    if (s_avail != -1) return (s_avail == 1);
    ProcPtr gestaltAddr = GetOSTrapAddress(0xAD);
    ProcPtr unimpAddr = GetToolTrapAddress(_Unimplemented);
    s_avail = (gestaltAddr != unimpAddr && gestaltAddr != NULL) ? 1 : 0;
    return (s_avail == 1);
#else
    return true;
#endif
}

bool classic_mac_safe_gestalt(uint32_t selector, long* response) {
    if (response) *response = 0;
#if defined(__m68k__) || defined(__mc68000__)
    if (!classic_mac_is_gestalt_available()) return false;
    register unsigned long reg_d0 __asm__("d0") = selector;
    register long reg_a0 __asm__("a0") = 0;
    register short err __asm__("d0");
    __asm__ volatile(
        "dc.w 0xa1ad"
        : "=d"(err), "=a"(reg_a0)
        : "0"(reg_d0)
        : "d1", "d2", "a1", "memory"
    );
    if (err == 0) {
        if (response) *response = reg_a0;
        return true;
    }
    return false;
#elif defined(TARGET_API_MAC_CARBON) || defined(__APPLE__) || defined(__MACOS__) || defined(__Retro68__)
    long val = 0;
    OSErr err = Gestalt((OSType)selector, &val);
    if (err == 0) {
        if (response) *response = val;
        return true;
    }
    return false;
#else
    (void)selector;
    return false;
#endif
}

bool classic_mac_has_color_qd(void) {
#if TARGET_API_MAC_CARBON
    return true;
#elif defined(__APPLE__) || defined(__MACOS__) || defined(macintosh) || defined(__Retro68__)
    long qd_ver = 0;
    if (classic_mac_safe_gestalt('qd  ', &qd_ver) && qd_ver >= 256) {
        return true;
    }
#if defined(__m68k__) || defined(__mc68000__)
    SysEnvRec env;
    if (SysEnvirons(curSysEnvVers, &env) == noErr) {
        return env.hasColorQD;
    }
#endif
    return false;
#else
    return false;
#endif
}

bool classic_mac_has_styled_te(void) {
#if TARGET_API_MAC_CARBON
    return true;
#elif defined(__APPLE__) || defined(__MACOS__) || defined(macintosh) || defined(__Retro68__)
    long te_ver = 0;
    if (classic_mac_safe_gestalt('te  ', &te_ver) && te_ver >= 2) {
        return true;
    }
    return false;
#else
    return false;
#endif
}

#else

bool classic_mac_is_gestalt_available(void) {
    return false;
}

bool classic_mac_safe_gestalt(uint32_t selector, long* response) {
    (void)selector;
    if (response) *response = 0;
    return false;
}

bool classic_mac_has_color_qd(void) {
    return false;
}

bool classic_mac_has_styled_te(void) {
    return false;
}

#endif
