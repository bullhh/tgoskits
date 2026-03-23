# StarryOS 启动手册

本文档介绍如何在 TGOSKits 仓库中启动 StarryOS，包括支持的架构与平台、启动方法、启动流程说明等。

---

## 目录

1. [环境准备](#1-环境准备)
2. [支持的架构与平台](#2-支持的架构与平台)
3. [快速开始](#3-快速开始)
4. [启动方法详解](#4-启动方法详解)
   - [方法一：使用 cargo xtask（推荐）](#方法一使用-cargo-xtask推荐)
   - [方法二：使用 Makefile](#方法二使用-makefile)
   - [全部支持平台配置一览](#43-全部支持平台配置一览)
5. [启动流程详解](#5-启动流程详解)
6. [启动脚本解析](#6-启动脚本解析)
7. [实体开发板启动](#7-实体开发板启动)
8. [常见问题](#8-常见问题)

---

## 1. 环境准备

### 1.1 系统要求

- Ubuntu 22.04 或类似的 Linux 系统
- Rust 1.75+ / Python 3.6+ / Git 2.0+

### 1.2 安装依赖

```bash
# 安装系统依赖
sudo apt update
sudo apt install -y build-essential cmake clang qemu-system curl xz-utils

# 安装 ostool
cargo install ostool --version ^0.8
```

### 1.3 安装 Musl 工具链（可选，用于 C 应用）

1. 从 [setup-musl releases](https://github.com/arceos-org/setup-musl/releases/tag/prebuilt) 下载
2. 解压到 `/opt/<arch>-linux-musl-cross`
3. 添加到 PATH：

```bash
export PATH=/opt/riscv64-linux-musl-cross/bin:$PATH
```

---

## 2. 支持的架构与平台

### 2.1 支持的架构

| 架构 | Target Triple | 说明 |
|------|---------------|------|
| **riscv64** | `riscv64gc-unknown-none-elf` | RISC-V 64位（默认） |
| **aarch64** | `aarch64-unknown-none-softfloat` | ARM 64位 |
| **loongarch64** | `loongarch64-unknown-none-softfloat` | 龙芯架构 |
| **x86_64** | `x86_64-unknown-none` | x86 64位（开发中） |

### 2.2 支持的平台

#### QEMU 虚拟平台（默认）

| 架构 | Platform 包名 | QEMU Machine |
|------|---------------|--------------|
| riscv64 | `axplat-riscv64-qemu-virt` | virt |
| aarch64 | `axplat-aarch64-qemu-virt` | virt |
| loongarch64 | `axplat-loongarch64-qemu-virt` | virt |
| x86_64 | `axplat-x86-pc` | q35 |

#### 实体开发板

| 开发板 | 架构 | Platform 包名 | 说明 |
|--------|------|---------------|------|
| Raspberry Pi 4B | aarch64 | `axplat-aarch64-raspi` | 树莓派 4B |
| Phytium Pi | aarch64 | `axplat-aarch64-phytium-pi` | 飞腾派 |
| VisionFive 2 | riscv64 | `axplat-riscv64-visionfive2` | RISC-V 开发板 |
| BSTA1000B | aarch64 | `axplat-aarch64-bsta1000b` | 百度 Apollo 开发板 |

---

## 3. 快速开始

```bash
# 1. 克隆仓库（如果还没有）
git clone https://github.com/rcore-os/tgoskits.git
cd tgoskits

# 2. 准备 rootfs（首次运行）
cargo xtask starry rootfs --arch riscv64

# 3. 构建并运行
cargo xtask starry run --arch riscv64
```

启动成功后会看到 Shell 提示符：
```
starry:~#
```

---

## 4. 启动方法详解

### 方法一：使用 cargo xtask（推荐）

这是 TGOSKits 推荐的方式，命令统一在仓库根目录执行。

#### 4.1.1 子命令概览

```bash
# 查看帮助
cargo xtask starry --help

# 可用子命令：
cargo xtask starry build   # 构建
cargo xtask starry run     # 构建并运行
cargo xtask starry rootfs  # 下载/准备 rootfs 镜像
```

#### 4.1.2 构建命令参数

```bash
cargo xtask starry build [OPTIONS]

Options:
  --arch <ARCH>        目标架构 (x86_64, aarch64, riscv64, loongarch64)
                       默认: riscv64
  -p, --package <PKG>  构建的包名
                       默认: starryos
  --platform <PLAT>    平台包名 (如 axplat-aarch64-raspi)
  --release            Release 模式构建 (默认 true)
  --features <FEAT>    启用的 features (逗号分隔)
  --smp <NUM>          CPU 数量
  --plat-dyn           启用动态平台支持
```

#### 4.1.3 运行命令参数

```bash
cargo xtask starry run [OPTIONS]

Options:
  # 继承所有 build 参数，额外支持：
  --blk                启用块设备 (默认 true)
  --disk-img <PATH>    磁盘镜像路径
  --net                启用网络 (默认 true)
  --net-dev <TYPE>     网络设备类型 (user, tap, bridge)
  --graphic            启用图形输出
  --accel              启用硬件加速 (KVM/HVF)
```

#### 4.1.4 查看支持的架构与平台

##### 列出可用平台（推荐）

```bash
# 列出 StarryOS 支持的所有平台
cargo xtask starry list-platforms
```

输出示例：
```
Available platforms for StarryOS:

QEMU Virtual Platforms (available by default):
--------------------------------------------------------------------------------
Platform                  Arch       Description
--------------------------------------------------------------------------------
riscv64-qemu-virt         riscv64    QEMU RISC-V 64-bit virt machine (default for riscv64)
aarch64-qemu-virt         aarch64    QEMU AArch64 virt machine (default for aarch64)
loongarch64-qemu-virt     loongarch64 QEMU LoongArch64 virt machine (default for loongarch64)

Physical Boards (available by default):
--------------------------------------------------------------------------------
Platform                  Arch       Status     Description
--------------------------------------------------------------------------------
x86-pc                    x86_64     ✓ Ready    x86 PC (QEMU q35 machine, default for x86_64)
aarch64-raspi4            aarch64    Need add   Raspberry Pi 4B
aarch64-phytium-pi        aarch64    Need add   Phytium Pi (飞腾派)
aarch64-bsta1000b         aarch64    Need add   BST A1000B (百度 Apollo 开发板)
riscv64-visionfive2       riscv64    ✓ Ready    VisionFive 2 (enable with --features vf2)

How to add a platform that is not yet available:
--------------------------------------------------------------------------------
  cd os/StarryOS/starryos
  cargo axplat add <platform-package>
```

##### 添加新平台依赖

对于标记为 "Need add" 的平台，需要先添加依赖：

```bash
# 进入 StarryOS 包目录
cd os/StarryOS/starryos

# 添加平台依赖
cargo axplat add axplat-aarch64-phytium-pi
cargo axplat add axplat-aarch64-raspi
cargo axplat add axplat-aarch64-bsta1000b

# 添加后即可使用该平台
cd ../../..
cargo xtask starry build --arch aarch64 --platform axplat-aarch64-phytium-pi
```

##### 查看已安装的 Rust 目标

```bash
# 查看所有已安装的目标
rustup target list --installed

# 安装所需目标
rustup target add riscv64gc-unknown-none-elf
rustup target add aarch64-unknown-none-softfloat
rustup target add loongarch64-unknown-none-softfloat
rustup target add x86_64-unknown-none
```

##### 查看帮助信息

```bash
# 查看所有 starry 子命令
cargo xtask starry --help

# 查看 build 子命令帮助
cargo xtask starry build --help

# 查看 run 子命令帮助
cargo xtask starry run --help
```

##### 平台配置文件位置

各平台的配置文件位于 `components/axplat_crates/platforms/<platform>/axconfig.toml`：

```
components/axplat_crates/platforms/
├── axplat-riscv64-qemu-virt/axconfig.toml     # RISC-V QEMU
├── axplat-aarch64-qemu-virt/axconfig.toml     # AArch64 QEMU
├── axplat-aarch64-raspi/axconfig.toml         # 树莓派 4B
├── axplat-aarch64-phytium-pi/axconfig.toml    # 飞腾派
├── axplat-aarch64-bsta1000b/axconfig.toml     # BST A1000B
├── axplat-loongarch64-qemu-virt/axconfig.toml # 龙芯 QEMU
└── axplat-x86-pc/axconfig.toml                # x86 PC
```

#### 4.1.5 完整启动命令参数

##### 构建命令 (build)

```bash
cargo xtask starry build [OPTIONS]

参数说明:
  --arch <ARCH>           目标架构
                          可选值: riscv64, aarch64, loongarch64, x86_64
                          默认值: riscv64

  -p, --package <PKG>     构建的包名
                          默认值: starryos

  --platform <PLATFORM>   平台包名，用于指定实体开发板
                          示例: axplat-aarch64-raspi, axplat-aarch64-phytium-pi

  --release               Release 模式构建 (优化级别高)
                          默认: true

  --features <FEATURES>   启用的 Cargo features (逗号分隔)
                          示例: smp,driver-sdmmc

  --smp <NUM>             CPU 核心数量
                          示例: --smp 4

  --plat-dyn              启用动态平台支持 (运行时检测平台)
```

##### 运行命令 (run)

```bash
cargo xtask starry run [OPTIONS]

参数说明:
  # 继承所有 build 参数，额外支持:

  --blk                   启用块设备 (virtio-blk)
                         默认: true

  --no-blk                禁用块设备

  --disk-img <PATH>       指定磁盘镜像路径
                         默认: target/<arch>/debug/disk.img

  --net                   启用网络设备 (virtio-net)
                         默认: true

  --no-net                禁用网络

  --net-dev <TYPE>        QEMU 网络后端类型
                         可选值: user, tap, bridge
                         默认: user

  --graphic               启用图形输出 (virtio-gpu)

  --accel                 启用硬件加速
                         Linux: KVM
                         macOS: HVF

  --help                  显示帮助信息
```

##### rootfs 命令

```bash
cargo xtask starry rootfs [OPTIONS]

参数说明:
  --arch <ARCH>           目标架构
                          默认: riscv64
```

#### 4.1.6 QEMU 启动示例

##### 基本启动（各架构默认 QEMU 平台）

```bash
# RISC-V 64位 (默认)
cargo xtask starry run
# 等价于
cargo xtask starry run --arch riscv64 --platform axplat-riscv64-qemu-virt

# AArch64
cargo xtask starry run --arch aarch64
# 等价于
cargo xtask starry run --arch aarch64 --platform axplat-aarch64-qemu-virt

# LoongArch64
cargo xtask starry run --arch loongarch64
# 等价于
cargo xtask starry run --arch loongarch64 --platform axplat-loongarch64-qemu-virt

# x86_64
cargo xtask starry run --arch x86_64
# 等价于
cargo xtask starry run --arch x86_64 --platform axplat-x86-pc
```

##### 启用设备选项

```bash
# 启用网络和存储（默认）
cargo xtask starry run --arch riscv64 --net --blk

# 启用图形界面
cargo xtask starry run --arch riscv64 --graphic

# 启用硬件加速 (需要 KVM 支持)
cargo xtask starry run --arch riscv64 --accel

# 使用 TAP 网络模式 (需要 root 权限)
sudo cargo xtask starry run --arch riscv64 --net-dev tap

# 指定自定义磁盘镜像
cargo xtask starry run --arch riscv64 --disk-img /path/to/custom.img
```

##### 多核启动

```bash
# 启动 4 核
cargo xtask starry build --arch aarch64 --smp 4
```

#### 4.1.7 实体开发板启动示例

> **重要提示**: 对于标记为 "Need add" 的平台，需要先添加平台依赖：
> ```bash
> cd os/StarryOS/starryos
> cargo axplat add <platform-package>
> ```
>
> 使用 `cargo xtask starry list-platforms` 查看所有平台及其可用状态。

##### Raspberry Pi 4B (aarch64-raspi)

```bash
# 1. 首先添加平台依赖（首次使用）
cd os/StarryOS/starryos
cargo axplat add axplat-aarch64-raspi
cd ../../..

# 2. 构建树莓派镜像
cargo xtask starry build --arch aarch64 --platform axplat-aarch64-raspi

# 产物位置:
# - target/aarch64-unknown-none-softfloat/release/starryos_aarch64-raspi4.elf
# - target/aarch64-unknown-none-softfloat/release/starryos_aarch64-raspi4.bin
```

加载方法（需要串口连接）:
1. 参考 [rust-raspberrypi-OS-tutorials](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials)
2. 使用 `make chainboot` 通过串口加载

##### Phytium Pi 飞腾派 (aarch64-phytium-pi)

```bash
# 1. 首先添加平台依赖（首次使用）
cd os/StarryOS/starryos
cargo axplat add axplat-aarch64-phytium-pi
cd ../../..

# 2. 构建飞腾派镜像
cargo xtask starry build --arch aarch64 --platform axplat-aarch64-phytium-pi

# 产物位置:
# - target/aarch64-unknown-none-softfloat/release/starryos_aarch64-phytium-pi.bin
```

##### VisionFive 2 (riscv64-visionfive2)

VisionFive 2 已内置支持，无需额外添加依赖，但需要启用 `vf2` feature：

```bash
# 方式一：使用 xtask（如果支持 --features）
cargo xtask starry build --arch riscv64 --features vf2

# 方式二：使用 Makefile（推荐）
cd os/StarryOS
make ARCH=riscv64 APP_FEATURES=vf2 MYPLAT=axplat-riscv64-visionfive2 BUS=mmio build
```

##### BST A1000B (aarch64-bsta1000b)

```bash
# 1. 首先添加平台依赖（首次使用）
cd os/StarryOS/starryos
cargo axplat add axplat-aarch64-bsta1000b
cd ../../..

# 2. 构建 BST A1000B 镜像
cargo xtask starry build --arch aarch64 --platform axplat-aarch64-bsta1000b --smp 8
```

#### 4.1.8 开发调试示例

##### 调试日志

```bash
# 使用 Makefile 设置日志级别
cd os/StarryOS
make ARCH=riscv64 LOG=debug run

# 日志级别: error, warn, info, debug, trace
make ARCH=riscv64 LOG=trace run
```

##### GDB 调试

```bash
cd os/StarryOS

# 启动 QEMU 并等待 GDB 连接
make ARCH=riscv64 debug

# 在另一个终端启动 GDB
gdb-multiarch target/riscv64gc-unknown-none-elf/release/starryos_riscv64-qemu-virt.elf
(gdb) target remote :1234
(gdb) break __axplat_main
(gdb) continue
```

##### 只构建不运行

```bash
cargo xtask starry build --arch riscv64

# 查看构建产物
ls target/riscv64gc-unknown-none-elf/release/
# - starryos_riscv64-qemu-virt.elf  (ELF 格式)
# - starryos_riscv64-qemu-virt.bin  (裸机二进制)
```

#### 4.1.9 准备 rootfs

```bash
# 自动下载并准备 rootfs
cargo xtask starry rootfs --arch riscv64
cargo xtask starry rootfs --arch aarch64
cargo xtask starry rootfs --arch loongarch64

# rootfs 存储位置
# target/<arch>/debug/disk.img
```

如果下载失败（网络问题），手动下载:
```bash
# 使用镜像下载
curl -L -o rootfs-riscv64.img.xz \
  "https://mirror.ghproxy.com/https://github.com/Starry-OS/rootfs/releases/download/20260214/rootfs-riscv64.img.xz"

# 解压并放置
xz -d rootfs-riscv64.img.xz
cp rootfs-riscv64.img target/riscv64gc-unknown-none-elf/debug/disk.img
```

#### 4.1.10 运行测试

```bash
# 运行 StarryOS 测试
cargo xtask test starry --target riscv64gc-unknown-none-elf
cargo xtask test starry --target aarch64-unknown-none-softfloat
```

---

## 4.3 全部支持平台配置一览

### 4.3.1 支持的架构总览

| 架构 | Target Triple | QEMU Machine | 默认平台包 |
|------|---------------|--------------|------------|
| **riscv64** | `riscv64gc-unknown-none-elf` | virt | `axplat-riscv64-qemu-virt` |
| **aarch64** | `aarch64-unknown-none-softfloat` | virt | `axplat-aarch64-qemu-virt` |
| **loongarch64** | `loongarch64-unknown-none-softfloat` | virt | `axplat-loongarch64-qemu-virt` |
| **x86_64** | `x86_64-unknown-none` | q35 | `axplat-x86-pc` |

### 4.3.2 全部支持平台详解

#### QEMU 虚拟平台

| 平台名称 | 架构 | 平台包名 | QEMU 命令 | 内存布局 |
|----------|------|----------|-----------|----------|
| **riscv64-qemu-virt** | riscv64 | `axplat-riscv64-qemu-virt` | `qemu-system-riscv64 -machine virt` | 内核基址: `0x8020_0000` |
| **aarch64-qemu-virt** | aarch64 | `axplat-aarch64-qemu-virt` | `qemu-system-aarch64 -machine virt -cpu cortex-a72` | 内核基址: `0x4020_0000` |
| **loongarch64-qemu-virt** | loongarch64 | `axplat-loongarch64-qemu-virt` | `qemu-system-loongarch64 -machine virt` | 内核基址: `0x0020_0000` |
| **x86-pc** | x86_64 | `axplat-x86-pc` | `qemu-system-x86_64 -machine q35` | 内核基址: `0x20_0000` |

#### 实体开发板

| 开发板 | 架构 | 平台包名 | CPU 核心 | 内存 | 内核基址 | 特性 |
|--------|------|----------|----------|------|----------|------|
| **Raspberry Pi 4B** | aarch64 | `axplat-aarch64-raspi` | 4 | 2GB | `0x8_0000` | UART, eMMC, GICv2 |
| **Phytium Pi** | aarch64 | `axplat-aarch64-phytium-pi` | 4 | 2GB | `0x9000_0000` | PCIe, UART, GICv3 |
| **BST A1000B** | aarch64 | `axplat-aarch64-bsta1000b` | 8 | 1.75GB | `0x8100_0000` | GICv2, PSCI |
| **VisionFive 2** | riscv64 | `axplat-riscv64-visionfive2` | - | - | - | SDMMC, SMP |

### 4.3.3 各平台配置参数

#### RISC-V 64 QEMU Virt

**配置文件**: `components/axplat_crates/platforms/axplat-riscv64-qemu-virt/axconfig.toml`

```toml
arch = "riscv64"
platform = "riscv64-qemu-virt"

[plat]
max-cpu-num = 1
phys-memory-base = 0x8000_0000
phys-memory-size = 0x800_0000      # 128MB
kernel-base-paddr = 0x8020_0000
phys-virt-offset = "0xffff_ffc0_0000_0000"

[devices]
uart-paddr = 0x1000_0000
plic-paddr = 0x0c00_0000           # 中断控制器
virtio-mmio-ranges = [[0x1000_1000, 0x1000], ...]  # 8个 VirtIO 设备
```

**启动命令**:
```bash
cargo xtask starry run --arch riscv64
# 或
cd os/StarryOS && make ARCH=riscv64 run
```

#### AArch64 QEMU Virt

**配置文件**: `components/axplat_crates/platforms/axplat-aarch64-qemu-virt/axconfig.toml`

```toml
arch = "aarch64"
platform = "aarch64-qemu-virt"

[plat]
max-cpu-num = 1
phys-memory-base = 0x4000_0000
phys-memory-size = 0x800_0000      # 128MB
kernel-base-paddr = 0x4020_0000
phys-virt-offset = "0xffff_0000_0000_0000"
psci-method = "hvc"

[devices]
uart-paddr = 0x0900_0000           # PL011 UART
gicd-paddr = 0x0800_0000           # GIC Distributor
gicc-paddr = 0x0801_0000           # GIC CPU Interface
rtc-paddr = 0x901_0000             # PL031 RTC
virtio-mmio-ranges = [[0x0a00_0000, 0x200], ...]  # 32个 VirtIO 设备
```

**启动命令**:
```bash
cargo xtask starry run --arch aarch64
# 或
cd os/StarryOS && make ARCH=aarch64 run
```

#### LoongArch64 QEMU Virt

**配置文件**: `components/axplat_crates/platforms/axplat-loongarch64-qemu-virt/axconfig.toml`

```toml
arch = "loongarch64"
platform = "loongarch64-qemu-virt"

[plat]
max-cpu-num = 1
low-memory-base = 0x0
low-memory-size = 0x1000_0000      # 256MB
phys-memory-size = 0x800_0000      # 128MB
kernel-base-paddr = 0x0020_0000
phys-virt-offset = "0xffff_8000_0000_0000"

[devices]
uart-paddr = 0x1FE001E0            # NS16550A UART
pch-pic-paddr = 0x10000000         # 中断控制器
eiointc-paddr = 0x1400             # EIO 中断控制器
rtc-paddr = 0x100d_0100
```

**启动命令**:
```bash
cargo xtask starry run --arch loongarch64
# 或
cd os/StarryOS && make ARCH=loongarch64 run
```

**注意**: LoongArch64 需要 QEMU 10.0 或更高版本。

#### x86_64 PC

**配置文件**: `components/axplat_crates/platforms/axplat-x86-pc/axconfig.toml`

```toml
arch = "x86_64"
platform = "x86-pc"

[plat]
max-cpu-num = 1
phys-memory-base = 0
phys-memory-size = 0x800_0000      # 128MB
kernel-base-paddr = 0x20_0000
phys-virt-offset = "0xffff_8000_0000_0000"

[devices]
pci-ecam-base = 0xb000_0000
mmio-ranges = [
    [0xb000_0000, 0x1000_0000],    # PCI config space
    [0xfe00_0000, 0xc0_0000],      # PCI devices
    [0xfec0_0000, 0x1000],         # IO APIC
    [0xfed0_0000, 0x1000],         # HPET
    [0xfee0_0000, 0x1000],         # Local APIC
]
```

**启动命令**:
```bash
cargo xtask starry run --arch x86_64
# 或
cd os/StarryOS && make ARCH=x86_64 run
```

#### Raspberry Pi 4B

**配置文件**: `components/axplat_crates/platforms/axplat-aarch64-raspi/axconfig.toml`

```toml
arch = "aarch64"
platform = "aarch64-raspi4"
package = "axplat-aarch64-raspi"

[plat]
max-cpu-num = 4
phys-memory-base = 0x0
phys-memory-size = 0x8000_0000     # 2GB
kernel-base-paddr = 0x8_0000
kernel-base-vaddr = "0xffff_0000_0008_0000"
phys-virt-offset = "0xffff_0000_0000_0000"
phys-bus-offset = 0xC0000000       # 总线地址偏移

[devices]
uart-paddr = 0xFE20_1000           # PL011 UART
uart-irq = 0x99                    # SPI 0x79 + 0x20
gicd-paddr = 0xFF84_1000           # GICv2 Distributor
gicc-paddr = 0xFF84_2000           # GICv2 CPU Interface
mmio-ranges = [
    [0xFE20_1000, 0x1000],         # PL011 UART
    [0xFE34_0000, 0x1000],         # eMMC
    [0xFF84_1000, 0x3000],         # GICv2
]
```

**启动命令**:
```bash
# 构建
cargo xtask starry build --arch aarch64 --platform axplat-aarch64-raspi

# 或使用 Makefile
cd os/arceos
make ARCH=aarch64 MYPLAT=axplat-aarch64-raspi build
make chainboot  # 通过串口加载
```

**串口加载前置条件**:
1. 参考 [rust-raspberrypi-OS-tutorials](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials)
2. 完成 `06_uart_chainloader` 章节设置

#### Phytium Pi (飞腾派)

**配置文件**: `components/axplat_crates/platforms/axplat-aarch64-phytium-pi/axconfig.toml`

```toml
arch = "aarch64"
platform = "aarch64-phytium-pi"
package = "axplat-aarch64-phytium-pi"

[plat]
max-cpu-num = 4
phys-memory-base = 0x8000_0000
phys-memory-size = 0x8000_0000     # 2GB
kernel-base-paddr = 0x9000_0000
phys-virt-offset = "0xffff_0000_0000_0000"
psci-method = "smc"
cpu-id-list = [0x200, 0x201, 0x00, 0x100]  # CPU 硬件 ID

[devices]
uart-paddr = 0x2800_D000           # UART 1
uart-irq = 0x74                    # SPI 0x54 + 0x20
gicd-paddr = 0x3088_0000           # GICv3 Distributor
gicc-paddr = 0x3080_0000           # GICv3 CPU Interface
pci-ecam-base = 0x4000_0000        # PCIe ECAM
pci-ranges = [
    [0x0, 0x5000_0000],            # PIO space
    [0x5800_0000, 0x2800_0000],    # 32-bit MMIO
    [0x10_0000_0000, 0x10_0000_0000],  # 64-bit MMIO
]
```

**启动命令**:
```bash
cargo xtask starry build --arch aarch64 --platform axplat-aarch64-phytium-pi
```

#### BST A1000B

**配置文件**: `components/axplat_crates/platforms/axplat-aarch64-bsta1000b/axconfig.toml`

```toml
arch = "aarch64"
platform = "aarch64-bsta1000b"
package = "axplat-aarch64-bsta1000b"

[plat]
max-cpu-num = 8                    # 8 核
phys-memory-base = 0x8000_0000
phys-memory-size = 0x7000_0000     # 1.75GB
kernel-base-paddr = 0x81000000
psci-method = "smc"
cpu-id-list = [0x00, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x700]

[devices]
uart-paddr = 0x2000_8000           # UART8250
uart-irq = 0xf5                    # SPI 0xd5 + 0x20
gicd-paddr = 0x3200_1000           # GIC-400 Distributor
gicc-paddr = 0x3200_2000           # GIC-400 CPU Interface
```

**启动命令**:
```bash
cargo xtask starry build --arch aarch64 --platform axplat-aarch64-bsta1000b --smp 8
```

#### VisionFive 2

**注意**: VisionFive 2 目前需要通过 Makefile 构建。

**启动命令**:
```bash
cd os/StarryOS

# 构建单核版本
make ARCH=riscv64 APP_FEATURES=vf2 MYPLAT=axplat-riscv64-visionfive2 BUS=mmio build

# 构建 SMP 多核版本
make ARCH=riscv64 APP_FEATURES=vf2,smp MYPLAT=axplat-riscv64-visionfive2 BUS=mmio build
```

**启用的特性**:
- `vf2`: 启用 VisionFive 2 支持 (SDMMC 驱动)
- `smp`: 启用多核支持

### 4.3.4 平台选择速查表

| 使用场景 | 架构 | 命令 |
|----------|------|------|
| **QEMU 快速测试 (默认)** | riscv64 | `cargo xtask starry run` |
| **QEMU AArch64 测试** | aarch64 | `cargo xtask starry run --arch aarch64` |
| **QEMU 龙芯测试** | loongarch64 | `cargo xtask starry run --arch loongarch64` |
| **树莓派 4B** | aarch64 | `cargo xtask starry build --arch aarch64 --platform axplat-aarch64-raspi` |
| **飞腾派** | aarch64 | `cargo xtask starry build --arch aarch64 --platform axplat-aarch64-phytium-pi` |
| **BST A1000B** | aarch64 | `cargo xtask starry build --arch aarch64 --platform axplat-aarch64-bsta1000b` |
| **VisionFive 2** | riscv64 | `make ARCH=riscv64 APP_FEATURES=vf2 MYPLAT=axplat-riscv64-visionfive2 build` |

### 4.3.5 rootfs 镜像下载地址

| 架构 | 下载链接 |
|------|----------|
| riscv64 | `https://github.com/Starry-OS/rootfs/releases/download/20260214/rootfs-riscv64.img.xz` |
| aarch64 | `https://github.com/Starry-OS/rootfs/releases/download/20260214/rootfs-aarch64.img.xz` |
| loongarch64 | `https://github.com/Starry-OS/rootfs/releases/download/20260214/rootfs-loongarch64.img.xz` |

**镜像代理** (国内用户):
```bash
# 使用 ghproxy 加速
https://mirror.ghproxy.com/https://github.com/Starry-OS/rootfs/releases/download/20260214/rootfs-riscv64.img.xz
```

---

### 方法二：使用 Makefile

这是传统方式，在 `os/StarryOS` 目录下执行。

#### 4.2.1 基本命令

```bash
cd os/StarryOS

# 准备 rootfs
make rootfs

# 构建并运行
make run

# 指定架构
make ARCH=riscv64 run
make ARCH=aarch64 run
make ARCH=loongarch64 run
```

#### 4.2.2 Makefile 参数说明

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `ARCH` | `riscv64` | 目标架构 |
| `MYPLAT` | (自动) | 平台包名 |
| `MODE` | `release` | 构建模式 |
| `LOG` | `warn` | 日志级别 |
| `SMP` | (平台配置) | CPU 数量 |
| `BLK` | `y` | 启用块设备 |
| `NET` | `y` | 启用网络 |
| `MEM` | `1G` | 内存大小 |
| `BUS` | `mmio` | 设备总线类型 |
| `DISK_IMG` | `disk.img` | 磁盘镜像路径 |
| `GRAPHIC` | `n` | 启用图形 |

#### 4.2.3 常用示例

```bash
# 启用网络和存储
make ARCH=riscv64 BLK=y NET=y run

# 启用图形
make ARCH=riscv64 GRAPHIC=y run

# 调试模式
make ARCH=riscv64 LOG=debug run

# 指定内存大小
make ARCH=riscv64 MEM=2G run

# 硬件加速
make ARCH=riscv64 ACCEL=y run

# 使用 GDB 调试
make ARCH=riscv64 debug
```

#### 4.2.4 快捷别名

Makefile 提供了快捷别名：

```bash
make rv    # 等同于 make ARCH=riscv64 run
make la    # 等同于 make ARCH=loongarch64 run
```

---

## 5. 启动流程详解

### 5.1 整体流程

```
┌─────────────────────────────────────────────────────────────────┐
│                        启动流程                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. 准备阶段                                                     │
│     ├── 解析命令行参数 (ARCH, PLATFORM, FEATURES 等)             │
│     ├── 加载平台配置 (axconfig.toml)                             │
│     └── 准备 rootfs 镜像                                         │
│                                                                 │
│  2. 构建阶段                                                     │
│     ├── 生成 linker script (根据平台配置)                         │
│     ├── 编译内核 (cargo build)                                   │
│     └── 生成二进制镜像 (.bin/.elf)                                │
│                                                                 │
│  3. 运行阶段                                                     │
│     ├── 启动 QEMU/加载到开发板                                    │
│     ├── Bootloader 初始化 (OpenSBI/UEFI等)                       │
│     ├── 内核启动 (__axplat_main)                                 │
│     └── 进入 Shell                                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 关键组件

#### 5.2.1 平台抽象层 (axplat)

`axplat` 是硬件抽象层，定义了平台初始化接口：

```rust
// 平台入口点
#[axplat::main]
fn kernel_main(cpu_id: usize, arg: usize) -> ! {
    // 1. 早期初始化（trap, console, time）
    axplat::init::init_early(cpu_id, arg);
    
    // 2. 后期初始化（外设）
    axplat::init::init_later(cpu_id, arg);
    
    // 3. 内核主逻辑
    // ...
}
```

#### 5.2.2 rootfs

rootfs 是用户态根文件系统，包含 busybox 等工具：

- 存储位置：`target/<arch>/debug/disk.img`
- 下载源：`https://github.com/Starry-OS/rootfs/releases/`

### 5.3 内存布局

以 riscv64 QEMU virt 为例：

```
物理地址空间:
┌────────────────────┐ 0x8000_0000
│   Bootloader       │ (OpenSBI)
│   (~128KB)         │
├────────────────────┤
│   Kernel Image     │
│   (.text, .rodata, │
│    .data, .bss)    │
├────────────────────┤
│   Kernel Heap      │
├────────────────────┤
│   Page Tables      │
├────────────────────┤
│   User Space       │
└────────────────────┘
```

---

## 6. 启动脚本解析

### 6.1 xtask 命令入口

**文件**: `xtask/src/main.rs`

```rust
// 定义 starry 子命令
#[derive(Subcommand)]
enum Commands {
    /// StarryOS build commands
    Starry {
        #[command(subcommand)]
        command: starry::StarryCommand,
    },
    // ...
}
```

**文件**: `xtask/src/starry/mod.rs`

```rust
// StarryOS 子命令定义
pub enum StarryCommand {
    Build { args: BuildArgs },    // 构建
    Run { args: RunArgs },        // 运行
    Rootfs { arch: ... },         // 准备 rootfs
    Img { arch: ... },            // (已弃用)
}
```

### 6.2 构建参数处理

**文件**: `xtask/src/starry/build.rs`

```rust
pub struct BuildArgs {
    pub arch: Option<String>,      // 架构
    pub package: String,           // 包名 (默认 "starryos")
    pub platform: Option<String>,  // 平台
    pub release: bool,             // Release 模式
    pub features: Option<String>,  // Features
    pub smp: Option<usize>,        // CPU 数量
    pub plat_dyn: bool,            // 动态平台
}

impl BuildArgs {
    pub fn into_config_override(self) -> Result<ArceosConfigOverride> {
        // 解析架构 (默认 riscv64)
        let arch = parse_starry_arch(self.arch.as_deref())?;
        
        // 构建配置覆盖
        Ok(ArceosConfigOverride {
            arch: Some(arch),
            platform: self.platform.or_else(|| 
                Some(PlatformResolver::resolve_default_platform_name(&arch))),
            mode: self.release.then_some(BuildMode::Release),
            app_features: Some(vec!["qemu".to_string()]),
            // ...
        })
    }
}
```

### 6.3 运行参数处理

**文件**: `xtask/src/starry/run.rs`

```rust
pub struct RunArgs {
    pub build: BuildArgs,          // 构建参数
    pub blk: bool,                 // 块设备
    pub disk_img: Option<String>,  // 磁盘镜像
    pub net: bool,                 // 网络
    pub net_dev: Option<String>,   // 网络设备类型
    pub graphic: bool,             // 图形
    pub accel: bool,               // 硬件加速
}

impl RunArgs {
    pub fn into_config_override(self) -> Result<ArceosConfigOverride> {
        let arch = parse_starry_arch(self.build.arch.as_deref())?;
        
        // 处理磁盘镜像
        let disk_img = if self.blk {
            let default_disk_img = starry_default_disk_image(arch)?;
            // 如果镜像不存在，自动下载 rootfs
            if !disk_img_path.exists() {
                ensure_rootfs_in_target_dir(arch, &disk_img_path)?;
            }
            Some(disk_img_path)
        } else {
            None
        };
        
        // 生成 QEMU 参数
        overrides.qemu = Some(parse_qemu_options(
            self.blk, disk_img, self.net, self.net_dev,
            self.graphic, self.accel, vec![], vec![]
        ));
        
        Ok(overrides)
    }
}
```

### 6.4 rootfs 下载逻辑

**文件**: `xtask/src/starry/config.rs`

```rust
const ROOTFS_URL: &str = "https://github.com/Starry-OS/rootfs/releases/download/20260214";

pub fn ensure_rootfs_in_target_dir(arch: Arch, disk_img: &Path) -> Result<()> {
    let rootfs_name = format!("rootfs-{}.img", arch);
    let rootfs_img = down_dir.join(&rootfs_name);
    
    // 检查是否已下载
    if !rootfs_img.exists() {
        println!("image not found, downloading {}...", rootfs_name);
        
        // 下载 .xz 压缩包
        let url = format!("{ROOTFS_URL}/{rootfs_name}.xz");
        Command::new("curl").arg("-f").arg("-L").arg(&url)...;
        
        // 解压
        Command::new("xz").arg("-d").arg("-f")...;
    }
    
    // 复制到目标位置
    fs::copy(&rootfs_img, disk_img)?;
    Ok(())
}
```

### 6.5 Makefile 流程

**文件**: `os/StarryOS/make/Makefile`

```makefile
# 主要目标
build: $(OUT_DIR) $(FINAL_IMG)    # 构建二进制

run: build justrun                # 构建并运行

debug: build                      # 调试模式
    $(call run_qemu_debug) &
    $(GDB) $(OUT_ELF) -ex 'target remote localhost:1234' ...

# 包含的子 Makefile
include deps.mk      # 依赖安装 (cargo-axplat)
include platform.mk  # 平台解析
include config.mk    # 配置生成
include features.mk  # Feature 解析
include build.mk     # 构建逻辑
include qemu.mk      # QEMU 运行
```

**平台解析**: `os/StarryOS/make/platform.mk`

```makefile
ifeq ($(MYPLAT),)
  # 未指定平台时，使用各架构默认平台
  ifeq ($(ARCH), x86_64)
    PLAT_PACKAGE := axplat-x86-pc
  else ifeq ($(ARCH), aarch64)
    PLAT_PACKAGE := axplat-aarch64-qemu-virt
  else ifeq ($(ARCH), riscv64)
    PLAT_PACKAGE := axplat-riscv64-qemu-virt
  else ifeq ($(ARCH), loongarch64)
    PLAT_PACKAGE := axplat-loongarch64-qemu-virt
  endif
else
  # 指定平台时，使用指定的包名
  PLAT_PACKAGE := $(MYPLAT)
endif
```

**QEMU 参数**: `os/StarryOS/make/qemu.mk`

```makefile
# 各架构的 QEMU 配置
qemu_args-riscv64 := \
  -machine virt \
  -bios default \
  -kernel $(FINAL_IMG)

qemu_args-aarch64 := \
  -cpu cortex-a72 \
  -machine virt \
  -kernel $(FINAL_IMG)

# 块设备
qemu_args-$(BLK) += \
  -device virtio-blk-$(vdev-suffix),drive=disk0 \
  -drive id=disk0,if=none,format=raw,file=$(DISK_IMG)

# 网络
qemu_args-$(NET) += \
  -device virtio-net-$(vdev-suffix),netdev=net0 \
  -netdev user,id=net0,hostfwd=tcp::5555-:5555

# 硬件加速
qemu_args-$(ACCEL) += -cpu host -accel kvm
```

---

## 7. 实体开发板启动

### 7.1 Raspberry Pi 4B

#### 准备工作

1. 参考 [rust-raspberrypi-OS-tutorials](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials)
2. 完成 `06_uart_chainloader` 章节，设置串口连接
3. 安装 JTAG 调试环境（可选）

#### 构建和加载

```bash
cd os/arceos

# 构建
make ARCH=aarch64 MYPLAT=axplat-aarch64-raspi build

# 使用 chainboot 加载到板子
make chainboot

# JTAG 调试
make jtagboot  # 终端1: 启动 halt 程序
make openocd   # 终端2: 连接 JTAG
make gdb       # 终端3: 启动 GDB
```

### 7.2 VisionFive 2

```bash
cd os/StarryOS

# 构建为 VisionFive 2
make ARCH=riscv64 APP_FEATURES=vf2 MYPLAT=axplat-riscv64-visionfive2 BUS=mmio build

# 产物位于: starryos_aarch64-visionfive2.bin
```

### 7.3 Phytium Pi

```bash
# 使用 xtask
cargo xtask starry build --arch aarch64 --platform axplat-aarch64-phytium-pi

# 或使用 Makefile
cd os/arceos
make ARCH=aarch64 MYPLAT=axplat-aarch64-phytium-pi build
```

### 7.4 通过 Axvisor 在 RK3568 上运行

RK3568 目前通过 Axvisor 虚拟化方案支持：

```bash
cd os/axvisor

# 查看 RK3568 配置
cat configs/board/roc-rk3568-pc.toml

# 构建 Axvisor
cargo xtask axvisor build --board roc-rk3568-pc
```

详见 `os/axvisor/README.md`。

---

## 8. 常见问题

### 8.1 rootfs 下载失败

**问题**: GitHub 下载超时或失败

**解决方案**:
```bash
# 使用镜像代理下载
curl -L -o rootfs-riscv64.img.xz \
  "https://mirror.ghproxy.com/https://github.com/Starry-OS/rootfs/releases/download/20260214/rootfs-riscv64.img.xz"

# 解压并放置到目标位置
xz -d rootfs-riscv64.img.xz
cp rootfs-riscv64.img target/riscv64gc-unknown-none-elf/debug/disk.img
```

### 8.2 QEMU 无法启动

**问题**: `qemu-system-xxx: command not found`

**解决方案**:
```bash
# 安装 QEMU
sudo apt install qemu-system-misc qemu-system-arm qemu-system-x86
```

### 8.3 链接器错误

**问题**: `linker 'rust-lld' not found`

**解决方案**:
```bash
# 确保 Rust 工具链完整
rustup component add llvm-tools-preview
```

### 8.4 特定架构工具链缺失

**问题**: 编译时报 `can't find crate for std`

**解决方案**:
```bash
# 添加目标
rustup target add riscv64gc-unknown-none-elf
rustup target add aarch64-unknown-none-softfloat
rustup target add loongarch64-unknown-none-softfloat
```

### 8.5 网络在 QEMU 中不工作

**问题**: Guest OS 无法访问网络

**解决方案**:
```bash
# 使用用户模式网络（默认）
cargo xtask starry run --arch riscv64 --net --net-dev user

# 或使用 TAP（需要 root）
sudo cargo xtask starry run --arch riscv64 --net --net-dev tap
```

---

## 附录

### A. 相关文件路径

```
tgoskits/
├── xtask/src/starry/           # xtask starry 命令实现
│   ├── mod.rs                  # 子命令定义
│   ├── build.rs                # 构建逻辑
│   ├── run.rs                  # 运行逻辑
│   └── config.rs               # 配置和 rootfs 下载
├── os/StarryOS/                # StarryOS 源码
│   ├── starryos/               # 主程序入口
│   ├── kernel/                 # 内核模块
│   ├── make/                   # Makefile 脚本
│   │   ├── Makefile            # 主 Makefile
│   │   ├── platform.mk         # 平台解析
│   │   ├── qemu.mk             # QEMU 配置
│   │   └── ...
│   └── Makefile                # 顶层 Makefile
└── components/axplat_crates/   # 平台抽象层实现
    └── platforms/
        ├── axplat-riscv64-qemu-virt/
        ├── axplat-aarch64-qemu-virt/
        ├── axplat-aarch64-raspi/
        └── ...
```

### B. 参考链接

- [StarryOS GitHub](https://github.com/Starry-OS/StarryOS)
- [ArceOS 文档](https://github.com/arceos-org/arceos)
- [rust-raspberrypi-OS-tutorials](https://github.com/rust-embedded/rust-raspberrypi-OS-tutorials)
- [rootfs 镜像下载](https://github.com/Starry-OS/rootfs/releases)
