#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <stdlib.h>
#include "classic_mac_engine.h"

int main(void) {
    // 1. Verify model name lookup for NewWorld Mac model identifiers
    assert(classic_mac_model_from_identifier("PowerMac1,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerMac1,1"), "Power Macintosh G3 (Blue and White)") == 0);

    assert(classic_mac_model_from_identifier("PowerMac2,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerMac2,1"), "iMac (Slot Loading)") == 0);

    assert(classic_mac_model_from_identifier("PowerMac3,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerMac3,1"), "Power Mac G4 (AGP Graphics)") == 0);

    assert(classic_mac_model_from_identifier("PowerMac3,2") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerMac3,2"), "Power Mac G4 (Uni-N)") == 0);

    assert(classic_mac_model_from_identifier("PowerMac3,3") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerMac3,3"), "Power Mac G4 (Gigabit Ethernet)") == 0);

    assert(classic_mac_model_from_identifier("PowerMac3,6") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerMac3,6"), "Power Mac G4 (FW 800 | Mirrored Drive Doors)") == 0);

    assert(classic_mac_model_from_identifier("PowerMac4,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerMac4,1"), "iMac (Early/Summer 2001)") == 0);

    assert(classic_mac_model_from_identifier("PowerMac4,2") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerMac4,2"), "iMac (15-inch Flat Panel)") == 0);

    assert(classic_mac_model_from_identifier("PowerMac5,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerMac5,1"), "Power Mac G4 Cube") == 0);

    assert(classic_mac_model_from_identifier("PowerMac7,2") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerMac7,2"), "Power Mac G5 (June 2003 | Early 2005)") == 0);

    assert(classic_mac_model_from_identifier("PowerMac10,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerMac10,1"), "Mac mini") == 0);

    // PowerBooks and iBooks
    assert(classic_mac_model_from_identifier("PowerBook1,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerBook1,1"), "PowerBook G3 (Bronze Keyboard)") == 0);

    assert(classic_mac_model_from_identifier("PowerBook2,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerBook2,1"), "iBook") == 0);

    assert(classic_mac_model_from_identifier("PowerBook3,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerBook3,1"), "PowerBook (Firewire)") == 0);

    assert(classic_mac_model_from_identifier("PowerBook3,2") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerBook3,2"), "PowerBook G4 (Titanium)") == 0);

    assert(classic_mac_model_from_identifier("PowerBook6,8") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("PowerBook6,8"), "PowerBook G4 (12-inch 1.5GHz)") == 0);

    // iMac and Server lines
    assert(classic_mac_model_from_identifier("iMac,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("iMac,1"), "iMac (Original, 5 Flavors)") == 0);

    assert(classic_mac_model_from_identifier("RackMac1,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("RackMac1,1"), "XServe") == 0);

    assert(classic_mac_model_from_identifier("RackMac3,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("RackMac3,1"), "XServe G5") == 0);

    // OldWorld Open Firmware entries
    assert(classic_mac_model_from_identifier("AAPL,PowerMac G3") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("AAPL,PowerMac G3"), "Power Macintosh G3 (Beige)") == 0);

    assert(classic_mac_model_from_identifier("AAPL,7500") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("AAPL,7500"), "Power Macintosh 7500") == 0);

    // 2. Prefix stripping ("Apple " prefix)
    assert(classic_mac_model_from_identifier("Apple PowerMac3,1") != NULL);
    assert(strcmp(classic_mac_model_from_identifier("Apple PowerMac3,1"), "Power Mac G4 (AGP Graphics)") == 0);

    // 3. Unknown identifiers should return NULL
    assert(classic_mac_model_from_identifier("UnknownMac99,1") == NULL);
    assert(classic_mac_model_from_identifier("") == NULL);
    assert(classic_mac_model_from_identifier(NULL) == NULL);

    // 4. Probing via environment variable simulation
    char probed_id[32] = {0};
    setenv("RUSTID_MAC_MODEL_ID", "PowerMac3,1", 1);
    bool probed = classic_mac_probe_model_identifier(probed_id, sizeof(probed_id));
    assert(probed == true);
    assert(strcmp(probed_id, "PowerMac3,1") == 0);

    // 5. Detection with probed model
    MacCpuInfo info;
    memset(&info, 0, sizeof(info));
    classic_mac_detect_cpu(&info);
    assert(strcmp(info.model_id, "PowerMac3,1") == 0);
    assert(strcmp(info.system_name, "Power Mac G4 (AGP Graphics)") == 0);
    assert(info.clock_mhz == 450);
    assert(info.bus_mhz == 100);

    // 6. Test report generation contains Model ID in Debug view
    char report[2048] = {0};
    classic_mac_generate_report(&info, 202 /* VIEW_DEBUG */, false, report, sizeof(report), NULL, NULL, 0);
    assert(strstr(report, "Model ID:") != NULL);
    assert(strstr(report, "PowerMac3,1") != NULL);

    // 7. Test report generation contains Model ID in Everything view
    memset(report, 0, sizeof(report));
    classic_mac_generate_report(&info, 203 /* VIEW_EVERYTHING */, false, report, sizeof(report), NULL, NULL, 0);
    assert(strstr(report, "Model ID:") != NULL);
    assert(strstr(report, "PowerMac3,1") != NULL);

    printf("All Classic Mac model identifier tests passed successfully!\n");
    return 0;
}
