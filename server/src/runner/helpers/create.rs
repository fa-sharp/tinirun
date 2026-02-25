use bollard::{
    models::{ContainerCreateBody, HostConfig, HostConfigCgroupnsModeEnum, ResourcesUlimits},
    query_parameters::{CreateContainerOptions, CreateContainerOptionsBuilder},
};

use crate::runner::constants::ID_LABEL;

/// Setup container creation for code execution. Attempts to isolate the
/// container as much as possible:
/// - Isolates the container from the host system by disabling networking and setting a read-only root filesystem.
/// - Locks down the container by setting a maximum memory limit, CPU limit, and PID limit.
/// - Drops all capabilities and sets the `no-new-privileges` security option.
/// - Uses a private IPC namespace to prevent shared-memory side-channels.
/// - Uses a private cgroup namespace to hide host cgroup hierarchy.
/// - Mounts /tmp as a size-limited tmpfs with noexec, nosuid, and nodev.
pub fn setup_container(
    run_id: &str,
    image_tag: &str,
    command: &str,
    stdin: bool,
    timeout: u32,
    mem_limit_mb: u32,
    cpu_limit: f32,
) -> (ContainerCreateBody, CreateContainerOptions) {
    let run_command = ["timeout", &format!("{timeout}s"), "sh", "-c", command];
    let container_body = ContainerCreateBody {
        image: Some(image_tag.to_owned()),
        cmd: Some(run_command.into_iter().map(str::to_owned).collect()),
        env: Some(vec!["HOME=/tmp/home".into()]),
        network_disabled: Some(true),
        open_stdin: Some(stdin),
        stdin_once: Some(stdin),
        labels: Some([(ID_LABEL.into(), run_id.into())].into()),
        host_config: Some(HostConfig {
            readonly_rootfs: Some(true),
            tmpfs: Some([("/tmp".into(), "rw,noexec,nosuid,nodev,size=100m".into())].into()),
            memory: Some((mem_limit_mb * 1024 * 1024).into()),
            nano_cpus: Some((cpu_limit * 1000.0).round() as i64 * 1_000_000),
            pids_limit: Some(50),
            ulimits: Some(vec![ResourcesUlimits {
                name: Some("nproc".into()),
                soft: Some(50),
                hard: Some(50),
            }]),
            cap_drop: Some(vec!["ALL".into()]),
            security_opt: Some(vec!["no-new-privileges".into()]),
            ipc_mode: Some("private".into()),
            cgroupns_mode: Some(HostConfigCgroupnsModeEnum::PRIVATE),
            ..Default::default()
        }),
        ..Default::default()
    };
    let container_options = CreateContainerOptionsBuilder::new().name(run_id).build();

    (container_body, container_options)
}
