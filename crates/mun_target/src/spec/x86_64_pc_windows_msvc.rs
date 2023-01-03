use crate::spec::{Target, TargetOptions};

pub fn target() -> Target {
    Target {
        llvm_target: "x86_64-pc-windows-msvc".into(),
        pointer_width: 64,
        arch: "x86_64".into(),
        data_layout: "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128".into(),
        options: TargetOptions {
            cpu: "x86-64".to_string(),
            ..super::windows_msvc_base::opts()
        },
    }
}
