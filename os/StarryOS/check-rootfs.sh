#!/bin/bash
# 检查 rootfs.img 是否符合 StarryOS 要求

set -e

ROOTFS_IMG="${1:-rootfs.img}"
MOUNT_DIR="/tmp/rootfs_check"

echo "=== StarryOS Rootfs 检查工具 ==="
echo "检查文件: $ROOTFS_IMG"
echo

# 检查文件是否存在
if [ ! -f "$ROOTFS_IMG" ]; then
    echo "✗ 错误: 文件不存在: $ROOTFS_IMG"
    exit 1
fi

# 检查文件类型
echo "→ 文件信息:"
file "$ROOTFS_IMG"
ls -lh "$ROOTFS_IMG"
echo

# 挂载镜像
echo "→ 挂载 rootfs..."
mkdir -p "$MOUNT_DIR"
sudo mount -o loop,ro "$ROOTFS_IMG" "$MOUNT_DIR" 2>/dev/null || {
    echo "✗ 挂载失败，可能需要 sudo 权限"
    exit 1
}

# 捕获退出信号，确保卸载
trap "sudo umount $MOUNT_DIR 2>/dev/null; rmdir $MOUNT_DIR 2>/dev/null" EXIT

echo "✓ 挂载成功: $MOUNT_DIR"
echo

# 检查项目
PASS=0
FAIL=0
WARN=0

check_file() {
    local file=$1
    local desc=$2
    local required=${3:-true}
    
    if [ -e "$MOUNT_DIR/$file" ]; then
        echo "✓ $desc: $file"
        [ $required = true ] && ((PASS++))
        return 0
    else
        if [ $required = true ]; then
            echo "✗ 缺少 $desc: $file"
            ((FAIL++))
        else
            echo "⚠ 可选 $desc: $file (未找到)"
            ((WARN++))
        fi
        return 1
    fi
}

check_dir() {
    local dir=$1
    local desc=$2
    local required=${3:-true}
    
    if [ -d "$MOUNT_DIR/$dir" ]; then
        local count=$(ls "$MOUNT_DIR/$dir" 2>/dev/null | wc -l)
        echo "✓ $desc: $dir ($count 项)"
        [ $required = true ] && ((PASS++))
        return 0
    else
        if [ $required = true ]; then
            echo "✗ 缺少目录 $desc: $dir"
            ((FAIL++))
        else
            echo "⚠ 可选目录 $desc: $dir (未找到)"
            ((WARN++))
        fi
        return 1
    fi
}

echo "=== 1. 基本目录结构 ==="
check_dir "bin" "基本命令目录"
check_dir "sbin" "系统命令目录" false
check_dir "lib" "基本库目录"
check_dir "usr" "用户程序目录"
check_dir "etc" "配置文件目录"
check_dir "dev" "设备文件目录" false
check_dir "proc" "进程文件系统" false
check_dir "sys" "系统文件系统" false
check_dir "tmp" "临时文件目录" false
check_dir "root" "root 用户目录" false
echo

echo "=== 2. 关键可执行文件 ==="
check_file "bin/busybox" "BusyBox 工具集" false
check_file "bin/sh" "Shell"
check_file "bin/init" "初始化程序" false

# 检查是否有其他 shell
if [ ! -e "$MOUNT_DIR/bin/sh" ]; then
    echo "  检查其他 shell..."
    check_file "bin/bash" "Bash" false
    check_file "bin/ash" "Ash" false
    check_file "bin/dash" "Dash" false
fi

# 如果没有 /bin/init，检查可能的 init 路径
if [ ! -e "$MOUNT_DIR/bin/init" ] && [ ! -e "$MOUNT_DIR/sbin/init" ]; then
    echo "  检查其他 init 路径..."
    check_file "sbin/init" "系统 init" false
    check_file "init" "根目录 init" false
fi
echo

echo "=== 3. C 标准库 ==="
# 检查常见的 C 库
if [ -d "$MOUNT_DIR/lib" ]; then
    echo "  查找 libc..."
    if ls "$MOUNT_DIR/lib/"*libc* 1>/dev/null 2>&1; then
        echo "✓ 找到 C 库:"
        ls -lh "$MOUNT_DIR/lib/"*libc* 2>/dev/null | head -5
        ((PASS++))
    else
        echo "⚠ 未找到 libc（可能是静态链接）"
        ((WARN++))
    fi
fi

# 检查 ld-linux（动态链接器）
if [ -d "$MOUNT_DIR/lib" ] || [ -d "$MOUNT_DIR/lib64" ]; then
    echo "  查找动态链接器..."
    if ls "$MOUNT_DIR/lib/"ld-linux* 1>/dev/null 2>&1 || ls "$MOUNT_DIR/lib64/"ld-linux* 1>/dev/null 2>&1; then
        echo "✓ 找到动态链接器"
        ((PASS++))
    else
        echo "⚠ 未找到动态链接器（可能是静态链接）"
        ((WARN++))
    fi
fi
echo

echo "=== 4. BusyBox 链接（如果使用 busybox） ==="
if [ -e "$MOUNT_DIR/bin/busybox" ]; then
    echo "  BusyBox 命令链接:"
    local cmds=("ls" "cat" "mkdir" "rm" "cp" "mv" "echo" "mount" "umount")
    for cmd in "${cmds[@]}"; do
        if [ -e "$MOUNT_DIR/bin/$cmd" ]; then
            if [ -L "$MOUNT_DIR/bin/$cmd" ]; then
                link=$(readlink "$MOUNT_DIR/bin/$cmd")
                echo "    ✓ /bin/$cmd -> $link"
            else
                echo "    ✓ /bin/$cmd (独立程序)"
            fi
        else
            echo "    ✗ 缺少 /bin/$cmd"
        fi
    done
fi
echo

echo "=== 5. 统计信息 ==="
echo "总文件数: $(find "$MOUNT_DIR" -type f | wc -l)"
echo "总目录数: $(find "$MOUNT_DIR" -type d | wc -l)"
echo "总大小: $(du -sh "$MOUNT_DIR" | cut -f1)"
echo

echo "=== 6. 检查结果 ==="
echo "通过: $PASS"
echo "失败: $FAIL"
echo "警告: $WARN"
echo

if [ $FAIL -eq 0 ]; then
    echo "✅ 文件系统结构完整，符合基本要求"
    echo
    echo "注意事项:"
    if [ ! -e "$MOUNT_DIR/bin/busybox" ]; then
        echo "  - 未使用 BusyBox，可能需要额外的工具"
    fi
    if [ ! -e "$MOUNT_DIR/bin/init" ] && [ ! -e "$MOUNT_DIR/sbin/init" ]; then
        echo "  - 未找到 init 程序，StarryOS 可能需要指定 init 路径"
    fi
    echo "  - StarryOS 启动后会自动挂载 /dev, /proc, /sys, /tmp"
else
    echo "❌ 文件系统缺少必要组件"
    echo
    echo "建议:"
    echo "  1. 使用 StarryOS 官方 rootfs:"
    echo "     make ARCH=aarch64 rootfs"
    echo "  2. 或手动补充缺失的文件和目录"
fi

echo
echo "=== 完成 ==="
