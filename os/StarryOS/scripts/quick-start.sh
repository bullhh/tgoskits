#!/bin/bash
#
# StarryOS quick-start helper.
#
# Usage:
#   ./scripts/quick-start.sh list
#   ./scripts/quick-start.sh <platform> build [options]
#   ./scripts/quick-start.sh <platform> run [options]
#
# Platforms:
#   qemu-aarch64
#   qemu-riscv64
#   qemu-loongarch64
#   qemu-x86_64
#   orangepi-5-plus
#
# Commands:
#   list   Print supported platforms and template locations
#   build  Refresh tmp config copies from tracked templates and build StarryOS
#   run    Run StarryOS using the tmp config copies (creates them if missing)
#
# Options for `orangepi-5-plus`:
#   --serial <device>   Override serial device in tmp U-Boot config
#   --baud <rate>       Override baud rate in tmp U-Boot config
#   --dtb <path>        Override DTB path in tmp U-Boot config

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

run_cmd() {
    echo -e "${BLUE}$*${NC}"
    "$@"
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
STARRY_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
WORKSPACE_ROOT="$(cd "${STARRY_DIR}/../.." && pwd)"
TMP_CONFIG_DIR="${STARRY_DIR}/tmp/configs"

ensure_workspace() {
    if [ ! -f "${WORKSPACE_ROOT}/Cargo.toml" ]; then
        error "workspace Cargo.toml not found at ${WORKSPACE_ROOT}"
        exit 1
    fi
    if [ ! -f "${STARRY_DIR}/Cargo.toml" ]; then
        error "StarryOS Cargo.toml not found at ${STARRY_DIR}"
        exit 1
    fi
}

usage() {
    cat <<EOF
Usage:
  ./scripts/quick-start.sh list
  ./scripts/quick-start.sh <platform> build [options]
  ./scripts/quick-start.sh <platform> run [options]

Platforms:
  qemu-aarch64
  qemu-riscv64
  qemu-loongarch64
  qemu-x86_64
  orangepi-5-plus

Commands:
  list   Print supported platforms and template locations
  build  Refresh tmp config copies from tracked templates and build StarryOS
  run    Run StarryOS using the tmp config copies (creates them if missing)

Options for orangepi-5-plus:
  --serial <device>   Override serial device in tmp U-Boot config
  --baud <rate>       Override baud rate in tmp U-Boot config
  --dtb <path>        Override DTB path in tmp U-Boot config
EOF
}

list_platforms() {
    cat <<EOF
Supported platforms:
  qemu-aarch64
    build template: ${STARRY_DIR}/configs/qemu/build-aarch64.toml
    run template:   ${STARRY_DIR}/configs/qemu/qemu-aarch64.toml
    commands:
      ./scripts/quick-start.sh qemu-aarch64 build
      ./scripts/quick-start.sh qemu-aarch64 run
  qemu-riscv64
    build template: ${STARRY_DIR}/configs/qemu/build-riscv64.toml
    run template:   ${STARRY_DIR}/configs/qemu/qemu-riscv64.toml
    commands:
      ./scripts/quick-start.sh qemu-riscv64 build
      ./scripts/quick-start.sh qemu-riscv64 run
  qemu-loongarch64
    build template: ${STARRY_DIR}/configs/qemu/build-loongarch64.toml
    run template:   ${STARRY_DIR}/configs/qemu/qemu-loongarch64.toml
    commands:
      ./scripts/quick-start.sh qemu-loongarch64 build
      ./scripts/quick-start.sh qemu-loongarch64 run
  qemu-x86_64
    build template: ${STARRY_DIR}/configs/qemu/build-x86_64.toml
    run template:   ${STARRY_DIR}/configs/qemu/qemu-x86_64.toml
    commands:
      ./scripts/quick-start.sh qemu-x86_64 build
      ./scripts/quick-start.sh qemu-x86_64 run
  orangepi-5-plus
    build template: ${STARRY_DIR}/configs/board/orangepi-5-plus.toml
    run template:   ${STARRY_DIR}/configs/board/orangepi-5-plus-uboot.toml
    commands:
      ./scripts/quick-start.sh orangepi-5-plus build --serial /dev/ttyUSB0
      ./scripts/quick-start.sh orangepi-5-plus run
EOF
}

arch_for_platform() {
    case "$1" in
        qemu-aarch64|orangepi-5-plus) echo "aarch64" ;;
        qemu-riscv64) echo "riscv64" ;;
        qemu-loongarch64) echo "loongarch64" ;;
        qemu-x86_64) echo "x86_64" ;;
        *)
            error "unsupported platform: $1"
            exit 1
            ;;
    esac
}

qemu_build_template() {
    case "$1" in
        qemu-aarch64) echo "${STARRY_DIR}/configs/qemu/build-aarch64.toml" ;;
        qemu-riscv64) echo "${STARRY_DIR}/configs/qemu/build-riscv64.toml" ;;
        qemu-loongarch64) echo "${STARRY_DIR}/configs/qemu/build-loongarch64.toml" ;;
        qemu-x86_64) echo "${STARRY_DIR}/configs/qemu/build-x86_64.toml" ;;
        *)
            error "unsupported qemu platform: $1"
            exit 1
            ;;
    esac
}

qemu_run_template() {
    case "$1" in
        qemu-aarch64) echo "${STARRY_DIR}/configs/qemu/qemu-aarch64.toml" ;;
        qemu-riscv64) echo "${STARRY_DIR}/configs/qemu/qemu-riscv64.toml" ;;
        qemu-loongarch64) echo "${STARRY_DIR}/configs/qemu/qemu-loongarch64.toml" ;;
        qemu-x86_64) echo "${STARRY_DIR}/configs/qemu/qemu-x86_64.toml" ;;
        *)
            error "unsupported qemu platform: $1"
            exit 1
            ;;
    esac
}

tmp_qemu_build_config() {
    case "$1" in
        qemu-aarch64) echo "${TMP_CONFIG_DIR}/build-aarch64.toml" ;;
        qemu-riscv64) echo "${TMP_CONFIG_DIR}/build-riscv64.toml" ;;
        qemu-loongarch64) echo "${TMP_CONFIG_DIR}/build-loongarch64.toml" ;;
        qemu-x86_64) echo "${TMP_CONFIG_DIR}/build-x86_64.toml" ;;
        *)
            error "unsupported qemu platform: $1"
            exit 1
            ;;
    esac
}

tmp_qemu_run_config() {
    case "$1" in
        qemu-aarch64) echo "${TMP_CONFIG_DIR}/qemu-aarch64.toml" ;;
        qemu-riscv64) echo "${TMP_CONFIG_DIR}/qemu-riscv64.toml" ;;
        qemu-loongarch64) echo "${TMP_CONFIG_DIR}/qemu-loongarch64.toml" ;;
        qemu-x86_64) echo "${TMP_CONFIG_DIR}/qemu-x86_64.toml" ;;
        *)
            error "unsupported qemu platform: $1"
            exit 1
            ;;
    esac
}

copy_template() {
    local src="$1"
    local dst="$2"
    run_cmd mkdir -p "$(dirname "${dst}")"
    run_cmd cp "${src}" "${dst}"
}

set_toml_string() {
    local file="$1"
    local key="$2"
    local value="$3"
    if grep -q "^${key} = " "${file}"; then
        run_cmd sed -i "s|^${key} = .*|${key} = \"${value}\"|g" "${file}"
    else
        printf '%s = "%s"\n' "${key}" "${value}" >> "${file}"
    fi
}

refresh_qemu_configs() {
    local platform="$1"
    copy_template "$(qemu_build_template "${platform}")" "$(tmp_qemu_build_config "${platform}")"
    copy_template "$(qemu_run_template "${platform}")" "$(tmp_qemu_run_config "${platform}")"
}

ensure_qemu_configs() {
    local platform="$1"
    local build_cfg
    local run_cfg
    build_cfg="$(tmp_qemu_build_config "${platform}")"
    run_cfg="$(tmp_qemu_run_config "${platform}")"
    if [ ! -f "${build_cfg}" ] || [ ! -f "${run_cfg}" ]; then
        warn "tmp QEMU config missing, refreshing from tracked templates"
        refresh_qemu_configs "${platform}"
    fi
}

refresh_orangepi_configs() {
    copy_template "${STARRY_DIR}/configs/board/orangepi-5-plus.toml" \
        "${TMP_CONFIG_DIR}/orangepi-5-plus.toml"
    copy_template "${STARRY_DIR}/configs/board/orangepi-5-plus-uboot.toml" \
        "${TMP_CONFIG_DIR}/orangepi-5-plus-uboot.toml"
}

ensure_orangepi_configs() {
    if [ ! -f "${TMP_CONFIG_DIR}/orangepi-5-plus.toml" ] || \
       [ ! -f "${TMP_CONFIG_DIR}/orangepi-5-plus-uboot.toml" ]; then
        warn "tmp Orange Pi config missing, refreshing from tracked templates"
        refresh_orangepi_configs
    fi
}

apply_orangepi_overrides() {
    local serial_device="$1"
    local baud_rate="$2"
    local dtb_path="$3"
    local runtime_cfg="${TMP_CONFIG_DIR}/orangepi-5-plus-uboot.toml"

    if [ -n "${serial_device}" ]; then
        set_toml_string "${runtime_cfg}" "serial" "${serial_device}"
    fi
    if [ -n "${baud_rate}" ]; then
        set_toml_string "${runtime_cfg}" "baud_rate" "${baud_rate}"
    fi
    if [ -n "${dtb_path}" ]; then
        set_toml_string "${runtime_cfg}" "dtb_file" "${dtb_path}"
    fi
}

run_cargo_in_workspace() {
    (
        cd "${WORKSPACE_ROOT}"
        run_cmd cargo "$@"
    )
}

build_qemu_platform() {
    local platform="$1"
    local arch
    arch="$(arch_for_platform "${platform}")"
    refresh_qemu_configs "${platform}"
    run_cargo_in_workspace xtask starry rootfs --arch "${arch}"
    run_cargo_in_workspace xtask starry build \
        --arch "${arch}" \
        --config "$(tmp_qemu_build_config "${platform}")"
}

run_qemu_platform() {
    local platform="$1"
    local arch
    arch="$(arch_for_platform "${platform}")"
    ensure_qemu_configs "${platform}"
    run_cargo_in_workspace xtask starry qemu \
        --arch "${arch}" \
        --config "$(tmp_qemu_build_config "${platform}")" \
        --qemu-config "$(tmp_qemu_run_config "${platform}")"
}

build_orangepi_platform() {
    local serial_device="$1"
    local baud_rate="$2"
    local dtb_path="$3"
    refresh_orangepi_configs
    apply_orangepi_overrides "${serial_device}" "${baud_rate}" "${dtb_path}"
    run_cargo_in_workspace xtask starry build \
        --arch aarch64 \
        --config "${TMP_CONFIG_DIR}/orangepi-5-plus.toml"
}

run_orangepi_platform() {
    local serial_device="$1"
    local baud_rate="$2"
    local dtb_path="$3"
    ensure_orangepi_configs
    apply_orangepi_overrides "${serial_device}" "${baud_rate}" "${dtb_path}"
    run_cargo_in_workspace xtask starry uboot \
        --arch aarch64 \
        --config "${TMP_CONFIG_DIR}/orangepi-5-plus.toml" \
        --uboot-config "${TMP_CONFIG_DIR}/orangepi-5-plus-uboot.toml"
}

parse_orangepi_options() {
    SERIAL_DEVICE=""
    BAUD_RATE=""
    DTB_PATH=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --serial)
                SERIAL_DEVICE="$2"
                shift 2
                ;;
            --baud)
                BAUD_RATE="$2"
                shift 2
                ;;
            --dtb)
                DTB_PATH="$2"
                shift 2
                ;;
            *)
                error "unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    if [ -z "${DTB_PATH}" ]; then
        DTB_PATH="${STARRY_DIR}/configs/board/orangepi-5-plus.dtb"
    fi
}

main() {
    ensure_workspace

    if [ $# -lt 1 ]; then
        usage
        exit 1
    fi

    case "$1" in
        -h|--help)
            usage
            exit 0
            ;;
        list)
            list_platforms
            exit 0
            ;;
    esac

    if [ $# -lt 2 ]; then
        usage
        exit 1
    fi

    local platform="$1"
    local command="$2"
    shift 2

    case "${platform}" in
        qemu-aarch64|qemu-riscv64|qemu-loongarch64|qemu-x86_64)
            if [ $# -ne 0 ]; then
                error "QEMU platforms do not accept extra options"
                exit 1
            fi
            case "${command}" in
                build)
                    build_qemu_platform "${platform}"
                    ;;
                run)
                    run_qemu_platform "${platform}"
                    ;;
                *)
                    error "unsupported command: ${command}"
                    usage
                    exit 1
                    ;;
            esac
            ;;
        orangepi-5-plus)
            parse_orangepi_options "$@"
            case "${command}" in
                build)
                    build_orangepi_platform "${SERIAL_DEVICE}" "${BAUD_RATE}" "${DTB_PATH}"
                    ;;
                run)
                    run_orangepi_platform "${SERIAL_DEVICE}" "${BAUD_RATE}" "${DTB_PATH}"
                    ;;
                *)
                    error "unsupported command: ${command}"
                    usage
                    exit 1
                    ;;
            esac
            ;;
        *)
            error "unsupported platform: ${platform}"
            usage
            exit 1
            ;;
    esac
}

main "$@"
