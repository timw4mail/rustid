use alloc::string::String;

pub fn ucfirst(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn cleanup_soc_vendor(s: &str) -> String {
    let lower = s.to_lowercase();
    let vendor = match lower.as_str() {
        "allwinner" | "sunxi" => "Allwinner",
        "amlogic" | "meson" => "Amlogic",
        "apple" => "Apple",
        "bigtreetech" => "BigTreeTech",
        "brcm" | "broadcom" => "Broadcom",
        "hisilicon" | "hi" => "HiSilicon",
        "mediatek" | "mtk" => "MediaTek",
        "nxp" | "freescale" => "NXP",
        "qcom" | "qualcomm" => "Qualcomm",
        "raspberrypi" => "Raspberry Pi",
        "realtek" => "Realtek",
        "renesas" => "Renesas",
        "rk" | "rockchip" => "Rockchip",
        "samsung" | "exynos" => "Samsung",
        "st" | "stmicro" => "STMicroelectronics",
        "ti" => "Texas Instruments",
        "xilinx" => "Xilinx",
        _ => return ucfirst(s),
    };

    String::from(vendor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ucfirst_empty() {
        assert_eq!(ucfirst(""), "");
    }

    #[test]
    fn test_ucfirst_already_upper() {
        assert_eq!(ucfirst("Hello"), "Hello");
    }

    #[test]
    fn test_ucfirst_lowercase() {
        assert_eq!(ucfirst("hello"), "Hello");
    }

    #[test]
    fn test_ucfirst_single_char() {
        assert_eq!(ucfirst("a"), "A");
    }

    #[test]
    fn test_cleanup_soc_vendor_brcm() {
        assert_eq!(cleanup_soc_vendor("brcm"), "Broadcom");
    }

    #[test]
    fn test_cleanup_soc_vendor_qcom() {
        assert_eq!(cleanup_soc_vendor("qcom"), "Qualcomm");
    }

    #[test]
    fn test_cleanup_soc_vendor_rk() {
        assert_eq!(cleanup_soc_vendor("rk"), "Rockchip");
    }

    #[test]
    fn test_cleanup_soc_vendor_allwinner() {
        assert_eq!(cleanup_soc_vendor("sunxi"), "Allwinner");
    }

    #[test]
    fn test_cleanup_soc_vendor_raspberrypi() {
        assert_eq!(cleanup_soc_vendor("raspberrypi"), "Raspberry Pi");
    }

    #[test]
    fn test_cleanup_soc_vendor_bigtreetech() {
        assert_eq!(cleanup_soc_vendor("bigtreetech"), "BigTreeTech");
    }

    #[test]
    fn test_cleanup_soc_vendor_other() {
        assert_eq!(cleanup_soc_vendor("unknown_vendor"), "Unknown_vendor");
    }
}
