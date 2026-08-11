use crate::Cpu;
use crate::common::{Cache, CliFlags, CpuDisplay, Level1Cache};
use alloc::collections::BTreeMap;
use slint::{ModelRc, VecModel};
use std::rc::Rc;

slint::slint! {
    import { ScrollView } from "std-widgets.slint";

    export struct RowData {
        label: string,
        sublabel: string,
        value: string,
    }

    export component AppWindow inherits Window {
        min-width: 640px;
        preferred-height: 900px;
        title: "Rustid - CPU Information";
        in property <[RowData]> rows;

        VerticalLayout {
            ScrollView {
                viewport-width: parent.width;
                viewport-height: max(grid.preferred-height, parent.height);

                grid := GridLayout {
                    width: parent.width;
                    spacing: 4px;
                    padding: 12px;

                    for data[idx] in rows: Text {
                        text: data.label;
                        col: 0;
                        row: idx;
                        horizontal-alignment: right;
                        // color: #00DD00;
                        font-size: 14px;
                        font-weight: FontWeight.bold;
                    }
                    for data[idx] in rows: Text {
                        text: data.sublabel;
                        col: 0;
                        row: idx;
                        horizontal-alignment: right;
                        color: #569cd6;
                        font-size: 14px;
                        font-weight: FontWeight.bold;
                    }
                    for data[idx] in rows: HorizontalLayout {
                        padding-left: 12px;
                        col: 1;
                        row: idx;
                        Text {
                            text: data.value;
                            font-size: 14px;
                        }
                    }
                }
            }
        }
    }
}

pub fn run() {
    use crate::common::TDetect;

    let cpu = Cpu::detect();
    let rows = build_rows(&cpu);

    let model: ModelRc<RowData> = Rc::new(VecModel::from(rows)).into();

    let ui = AppWindow::new().expect("Failed to create Window");
    ui.set_rows(model);
    ui.run().expect("Failed to run GUI");
}

fn build_rows(cpu: &Cpu) -> Vec<RowData> {
    let mut rows = Vec::new();

    rows.push(RowData {
        label: "".into(),
        sublabel: "".into(),
        value: format!(
            "Rustid {} ({} - {})",
            crate::VERSION,
            crate::ARCH,
            crate::OS
        )
        .into(),
    });

    blank_row(&mut rows);

    #[cfg(x86_cpu)]
    build_x86_rows(&mut rows, cpu);

    #[cfg(arm_cpu)]
    build_arm_rows(&mut rows, cpu);

    #[cfg(ppc_cpu)]
    build_ppc_rows(&mut rows, cpu);

    rows
}

fn blank_row(rows: &mut Vec<RowData>) {
    rows.push(RowData {
        label: slint::SharedString::default(),
        sublabel: slint::SharedString::default(),
        value: slint::SharedString::default(),
    });
}

fn push_row(rows: &mut Vec<RowData>, label: &str, sublabel: &str, value: &str) {
    if !label.is_empty() && !sublabel.is_empty() {
        rows.push(RowData {
            label: label.into(),
            sublabel: slint::SharedString::default(),
            value: slint::SharedString::default(),
        });
        rows.push(RowData {
            label: slint::SharedString::default(),
            sublabel: sublabel.into(),
            value: value.into(),
        });
    } else {
        rows.push(RowData {
            label: label.into(),
            sublabel: sublabel.into(),
            value: value.into(),
        });
    }
}

fn push_row_and_blank(rows: &mut Vec<RowData>, label: &str, sublabel: &str, value: &str) {
    push_row(rows, label, sublabel, value);
    blank_row(rows);
}

// ---------------------------------------------------------------------------
// x86 / x86_64
// ---------------------------------------------------------------------------
#[cfg(x86_cpu)]
fn build_x86_rows(rows: &mut Vec<RowData>, cpu: &crate::x86::Cpu) {
    use crate::common::UNK;
    use crate::x86::{FeatureClass, HypervisorBrand, micro_arch::MicroArch};

    let disp = CpuDisplay {
        flags: CliFlags {
            color: false,
            verbose: false,
        },
    };

    if let Some(system) = &cpu.system {
        push_row_and_blank(rows, "System", "", &disp.format_system_name(system));
    }

    push_row_and_blank(rows, "Architecture", "", FeatureClass::detect().to_str());

    if cpu.arch.brand_name != UNK {
        push_row_and_blank(
            rows,
            "Vendor",
            "",
            &format!("{} ({})", cpu.arch.vendor_string, cpu.arch.brand_name),
        );
    }

    #[cfg(not(dos))]
    if let Some(hyp_str) = &cpu.hyp_vendor_str {
        let hyp = HypervisorBrand::from(hyp_str.as_str());
        push_row_and_blank(
            rows,
            "Hypervisor",
            "",
            &format!("{} ({})", hyp_str, hyp.to_str()),
        );
    }

    build_x86_model(rows, cpu);
    blank_row(rows);

    let ma = cpu.arch.micro_arch.as_str();
    if ma != UNK {
        push_row_and_blank(rows, "MicroArch", "", ma);
    }

    if cpu.arch.code_name != UNK
        && cpu.arch.code_name != ma
        && cpu.arch.micro_arch != MicroArch::I486
    {
        push_row_and_blank(rows, "Codename", "", cpu.arch.code_name);
    }

    if let Some(tech) = &cpu.arch.technology {
        push_row_and_blank(rows, "Process Node", "", tech);
    }

    if let Some(egg) = &cpu.easter_egg {
        push_row_and_blank(rows, "Easter Egg", "", egg);
    }

    if !cpu.has_cpuid {
        push_row_and_blank(rows, "x86", "", "No");
    }
    if cpu.signature.is_overdrive {
        push_row_and_blank(rows, "Overdrive", "", "Yes");
    }

    if !cpu.cores.is_empty() {
        build_x86_cores(rows, cpu);
    } else {
        build_x86_topology(rows, cpu);
    }
    blank_row(rows);

    if cpu.cores.is_empty() {
        let cache_count = |share_count: u32| -> String {
            let count = if share_count == 0 {
                cpu.topology.sockets.count
            } else {
                cpu.topology.threads.count / share_count
            };
            if count < 2 {
                String::new()
            } else {
                alloc::format!("{}x ", count)
            }
        };
        build_cache_rows(
            rows,
            cpu.topology.cache.as_ref(),
            &cache_count,
            cpu.topology.sockets.count,
        );
        blank_row(rows);
    }

    build_speed_rows(rows, &cpu.topology.speed);
    if cpu.topology.speed.base > 0 {
        blank_row(rows);
    }
    build_signature_rows(rows, &cpu.signature);
    blank_row(rows);
    build_features_rows(
        rows,
        &[
            "Base", "SSE", "AVX", "AVX512", "Security", "Math", "Other", "Centaur",
        ],
        &cpu.features,
    );
    if !cpu.features.is_empty() {
        blank_row(rows);
    }

    #[cfg(not(dos))]
    if crate::x86::is_centaur() {
        let centaur_map = crate::x86::vendor::Centaur::get_feature_list();
        if !centaur_map.is_empty() {
            let list: Vec<&str> = centaur_map
                .iter()
                .filter(|(_, enabled)| *enabled)
                .map(|(name, _)| *name)
                .collect();
            if !list.is_empty() {
                push_row(rows, "", "Centaur", &list.join(", "));
            }
        }
    }

    #[cfg(target_arch = "x86")]
    if crate::x86::is_cyrix() {
        let cyrix = crate::x86::vendor::Cyrix::detect();
        if cyrix.dir0 != 0xFF {
            push_row(
                rows,
                "Cyrix",
                "",
                &format!("Model number: {:X}h", cyrix.dir0),
            );
            push_row(rows, "", "Revision", &format!("{:X}h", cyrix.revision));
            push_row(rows, "", "Stepping", &format!("{:X}h", cyrix.stepping));
            if !cyrix.multiplier.is_empty() && cyrix.multiplier != "0" {
                push_row(
                    rows,
                    "",
                    "Bus Multiplier",
                    &format!("{}x", cyrix.multiplier),
                );
            }
            blank_row(rows);
        }
    }
}

#[cfg(x86_cpu)]
fn build_x86_model(rows: &mut Vec<RowData>, cpu: &crate::x86::Cpu) {
    use crate::common::UNK;

    let raw_model = crate::Cpu::raw_model_string();
    let disp_model = cpu.display_model_string();

    if disp_model != UNK {
        if raw_model == UNK || raw_model.trim() == disp_model {
            push_row(rows, "Model", "", &disp_model);
        } else {
            push_row(rows, "Model", "", &disp_model);
            push_row(rows, "Model (raw)", "", &raw_model);
        }
    }
}

#[cfg(x86_cpu)]
fn build_x86_cores(rows: &mut Vec<RowData>, cpu: &crate::x86::Cpu) {
    push_row(
        rows,
        "Cpu Topology",
        "",
        &format!(
            "{} cores ({} threads) across {} core types",
            cpu.topology.cores.count,
            cpu.topology.threads.count,
            cpu.cores.len()
        ),
    );

    for (i, core) in cpu.cores.iter().enumerate() {
        let core_label = alloc::format!("Core #{}", i + 1);
        push_row(rows, &core_label, "", "");

        let type_str: &str = core.kind.into();
        push_row(rows, "", "Type", type_str);

        if let Some(name) = &core.name {
            push_row(rows, "", "Codename", name);
        }

        if core.count != core.threads {
            push_row(
                rows,
                "",
                "Topology",
                &format!("{} cores ({} threads)", core.count, core.threads),
            );
        } else {
            push_row(rows, "", "Topology", &format!("{} cores", core.count));
        }

        let cc = |s: u32| CpuDisplay::cache_count(s, core.count);
        build_cache_rows(rows, core.cache.as_ref(), &cc, cpu.topology.sockets.count);
    }
}

#[cfg(x86_cpu)]
fn build_x86_topology(rows: &mut Vec<RowData>, cpu: &crate::x86::Cpu) {
    let multi_core = cpu.topology.cores.count > 1 || cpu.topology.sockets.count > 1;
    if multi_core {
        if cpu.topology.sockets.count > 1 {
            push_row(
                rows,
                "Topology",
                "",
                &format!(
                    "{} sockets, {} cores, {} threads",
                    cpu.topology.sockets.count,
                    cpu.topology.cores.count,
                    cpu.topology.threads.count
                ),
            );
        } else if cpu.topology.cores.count != cpu.topology.threads.count {
            push_row(
                rows,
                "Topology",
                "",
                &format!(
                    "{} cores ({} threads)",
                    cpu.topology.cores.count, cpu.topology.threads.count
                ),
            );
        } else {
            push_row(
                rows,
                "Topology",
                "",
                &format!("{} cores", cpu.topology.cores.count),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// ARM / AArch64
// ---------------------------------------------------------------------------
#[cfg(arm_cpu)]
fn build_arm_rows(rows: &mut Vec<RowData>, cpu: &crate::arm::Cpu) {
    let disp = CpuDisplay {
        flags: CliFlags {
            color: false,
            verbose: false,
        },
    };

    if let Some(system) = &cpu.system {
        push_row_and_blank(rows, "System", "", &disp.format_system_name(system));
    }

    if let Some(soc_model) = &cpu.soc_model {
        push_row_and_blank(rows, "SoC", "", soc_model);
    }

    push_row_and_blank(rows, "Implementer", "", &cpu.vendor);
    push_row_and_blank(rows, "Model", "", &cpu.cpu_arch.model);
    push_row_and_blank(rows, "Codename", "", cpu.cpu_arch.code_name);

    if let Some(tech) = cpu.cpu_arch.technology {
        push_row_and_blank(rows, "Process", "", tech);
    }

    if cpu.cores.len() > 1 {
        for (i, (_key, core)) in cpu.cores.iter().enumerate() {
            let core_label = alloc::format!("Core #{}", i + 1);
            push_row(rows, &core_label, "", "");
            push_row(rows, "", "Count", &alloc::format!("{}", core.count));
            let type_str: &str = core.kind.into();
            push_row(rows, "", "Type", type_str);

            if let Some(name) = &core.name {
                push_row(rows, "", "Name", name);
            }

            let cc = |s: u32| CpuDisplay::cache_count(s, core.count);
            build_cache_rows(rows, core.cache.as_ref(), &cc, 0);
            blank_row(rows);
        }
    } else {
        push_row(rows, "Cores", "", "");
        if let Some(core) = cpu.cores.values().next() {
            push_row(rows, "", "Count", &alloc::format!("{}", core.count));
            if let Some(name) = &core.name {
                push_row(rows, "", "Name", name);
            }
            let cc = |s: u32| CpuDisplay::cache_count(s, core.count);
            build_cache_rows(rows, core.cache.as_ref(), &cc, 0);
        }
        blank_row(rows);
    }

    build_features_rows(
        rows,
        &["Base", "SIMD", "Security", "Atomics", "Fp", "Misc"],
        &cpu.features,
    );
    if !cpu.features.is_empty() {
        blank_row(rows);
    }
}

// ---------------------------------------------------------------------------
// PowerPC
// ---------------------------------------------------------------------------
#[cfg(ppc_cpu)]
fn build_ppc_rows(rows: &mut Vec<RowData>, cpu: &crate::ppc::cpu::Cpu) {
    let disp = CpuDisplay {
        flags: CliFlags {
            color: false,
            verbose: false,
        },
    };

    if let Some(system) = &cpu.system {
        push_row_and_blank(rows, "System", "", &disp.format_system_name(system));
    }

    push_row_and_blank(rows, "Model", "", cpu.cpu_arch.marketing_name);

    let ma: &str = cpu.cpu_arch.micro_arch.into();
    push_row_and_blank(rows, "MicroArch", "", ma);

    push_row_and_blank(rows, "Code Name", "", cpu.cpu_arch.code_name);

    if let Some(tech) = cpu.cpu_arch.technology {
        push_row_and_blank(rows, "Process", "", tech);
    }

    if let Some(clock_mhz) = cpu.clock_speed {
        push_row_and_blank(
            rows,
            "Frequency",
            "",
            &CpuDisplay::format_frequency(clock_mhz),
        );
    }

    let cc = |s: u32| CpuDisplay::cache_count(s, 1);
    build_cache_rows(rows, cpu.cache.as_ref(), &cc, 0);
}

// ---------------------------------------------------------------------------
// Shared formatting helpers
// ---------------------------------------------------------------------------

fn build_cache_rows(
    rows: &mut Vec<RowData>,
    cache: Option<&Cache>,
    cache_count: &dyn Fn(u32) -> String,
    l3_socket_count: u32,
) {
    let Some(cache) = cache else {
        return;
    };

    match &cache.l1 {
        Level1Cache::Unified(l1) => {
            push_row(rows, "Cache", "L1", &format!("Unified {} KB", l1.size));
        }
        Level1Cache::Split { data, instruction } => {
            let data_c = cache_count(data.share_count);
            let inst_c = cache_count(instruction.share_count);

            let val = if data.assoc > 0 {
                format!("{}{} KB, {}-way", data_c, data.size / 1024, data.assoc)
            } else {
                format!("{}{} KB", data_c, data.size / 1024)
            };
            push_row(rows, "Cache", "L1d", &val);

            let val = if instruction.assoc > 0 {
                format!(
                    "{}{} KB, {}-way",
                    inst_c,
                    instruction.size / 1024,
                    instruction.assoc
                )
            } else {
                format!("{}{} KB", inst_c, instruction.size / 1024)
            };
            push_row(rows, "", "L1i", &val);
        }
    }

    if let Some(l2) = &cache.l2 {
        let count = cache_count(l2.share_count);
        let (num, unit) = CpuDisplay::cache_size(l2.size);
        let val = if l2.assoc > 0 {
            format!("{}{} {}, {}-way", count, num, unit, l2.assoc)
        } else {
            format!("{}{} {}", count, num, unit)
        };
        push_row(rows, "", "L2", &val);
    }

    if let Some(l3) = &cache.l3 {
        let (num, unit) = CpuDisplay::cache_size(l3.size);
        let count = if l3_socket_count > 1 {
            alloc::format!("{}x ", l3_socket_count)
        } else {
            cache_count(l3.share_count)
        };
        let val = if l3.assoc > 0 {
            format!("{}{} {}, {}-way", count, num, unit, l3.assoc)
        } else {
            format!("{}{} {}", count, num, unit)
        };
        push_row(rows, "", "L3", &val);
    }
}

#[cfg(x86_cpu)]
fn build_speed_rows(rows: &mut Vec<RowData>, speed: &crate::common::Speed) {
    if speed.base > 0 {
        if speed.boost > speed.base {
            push_row(
                rows,
                "Frequency",
                "Base",
                &CpuDisplay::format_frequency(speed.base),
            );
            push_row(
                rows,
                "",
                "Boost",
                &CpuDisplay::format_frequency(speed.boost),
            );
        } else {
            push_row(
                rows,
                "Frequency",
                "",
                &CpuDisplay::format_frequency(speed.base),
            );
        }
    }
}

#[cfg(x86_cpu)]
fn build_signature_rows(rows: &mut Vec<RowData>, sig: &crate::x86::cpu::CpuSignature) {
    use crate::common::DataSource;
    use crate::x86::cpu::CpuSignature;

    if *sig == CpuSignature::default() {
        return;
    }

    let key = if sig.source == DataSource::Cpuid || sig.source == DataSource::CpuidDump {
        "Signature"
    } else {
        "Synthetic Sig"
    };

    push_row(
        rows,
        key,
        "",
        &format!(
            "Family {:X}h, Model {:X}h, Stepping {:X}h",
            sig.display_family, sig.display_model, sig.stepping
        ),
    );
    push_row(
        rows,
        "",
        "",
        &format!(
            "({}, {}, {}, {}, {})",
            sig.extended_family, sig.family, sig.extended_model, sig.model, sig.stepping
        ),
    );
}

fn build_features_rows(
    rows: &mut Vec<RowData>,
    keys: &[&str],
    features: &BTreeMap<&'static str, String>,
) {
    if features.is_empty() {
        return;
    }

    if features.len() == 1 {
        if let Some(val) = features.get("Base") {
            push_row(rows, "Features", "", val);
        }
        return;
    }

    for key in keys {
        if let Some(feat_str) = features.get(key) {
            if *key == "Base" {
                push_row(rows, "Features", "Base", feat_str);
            } else {
                push_row(rows, "", key, feat_str);
            }
        }
    }
}
