#!/bin/bash
# 简单检查 rootfs（不需要挂载）

ROOTFS="rootfs.img"

echo "=== StarryOS Rootfs 快速检查 ==="
echo

# 1. 检查文件信息
echo "→ 文件信息:"
file "$ROOTFS"
ls -lh "$ROOTFS"
echo

# 2. 检查是否为 ext4
if file "$ROOTFS" | grep -q "ext4"; then
    echo "✓ 文件系统类型: ext4 (StarryOS 推荐)"
else
    echo "⚠ 文件系统类型不是 ext4"
fi
echo

# 3. 使用 debugfs 检查（如果可用）
if command -v debugfs >/dev/null 2>&1; then
    echo "→ 使用 debugfs 检查内部结构:"
    
    # 检查根目录
    echo "  根目录内容:"
    debugfs -R "ls -l /" "$ROOTFS" 2>/dev/null | head -20
    echo
    
    # 检查关键文件
    echo "  检查关键文件:"
    for file in /bin/busybox /bin/sh /bin/init /sbin/init; do
        if debugfs -R "stat $file" "$ROOTFS" >/dev/null 2>&1; then
            echo "    ✓ 找到: $file"
        else
            echo "    ✗ 缺少: $file"
        fi
    done
    echo
    
    # 检查关键目录
    echo "  检查关键目录:"
    for dir in /bin /lib /usr /etc /dev /proc /sys; do
        if debugfs -R "ls $dir" "$ROOTFS" >/dev/null 2>&1; then
            echo "    ✓ 找到目录: $dir"
        else
            echo "    ✗ 缺少目录: $dir"
        fi
    done
else
    echo "⚠ debugfs 不可用，无法深入检查"
    echo "  安装方法: sudo apt install e2fsprogs"
fi

echo
echo "=== 建议 ==="
echo "1. 如果需要完整检查，请使用 sudo 运行:"
echo "   sudo ./check-rootfs.sh rootfs.img"
echo
echo "2. StarryOS 对 rootfs 的基本要求:"
echo "   - 包含 /bin/sh 或 /bin/busybox"
echo "   - 包含基本目录: /bin, /lib, /usr, /etc"
echo "   - 如果有 /bin/init 会更好"
echo "   - /dev, /proc, /sys 可以在启动时自动创建"
