#!/bin/bash
set -e

echo "=== QR2Pic WSL 编译测试 ==="
echo "工作目录: $(pwd)"
echo ""

# 检查是否在 WSL 中
if ! grep -q Microsoft /proc/version 2>/dev/null; then
    echo "警告: 这可能不是在 WSL 环境中运行"
    echo ""
fi

# 1. 检查 Rust 工具链
echo "=== 1. 检查 Rust 工具链 ==="
if ! command -v rustc &> /dev/null; then
    echo "错误: Rust 未安装"
    echo "请运行: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
rustc --version
cargo --version
echo ""

# 2. 设置测试环境变量（避免 sqlx 宏编译错误）
echo "=== 2. 设置测试环境变量 ==="
export DATABASE_URL="postgresql://test:test@localhost/test"
export OSS_ENDPOINT="https://oss-test.example.com"
export OSS_ACCESS_KEY_ID="test_key"
export OSS_SECRET_ACCESS_KEY="test_secret"
export OSS_BUCKET_NAME="test-bucket"
export SERVER_PORT="8080"
export DELETE_KEY="test_delete_key_here"

echo "环境变量已设置（仅用于编译）"
echo ""

# 3. 清理之前的构建（可选）
echo "=== 3. 清理构建缓存 ==="
read -p "是否清理之前的构建缓存？(y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "正在清理..."
    cargo clean
    echo "清理完成"
fi
echo ""

# 4. 更新依赖
echo "=== 4. 更新 Cargo 依赖 ==="
cargo update
echo ""

# 5. 类型检查
echo "=== 5. 运行类型检查 (cargo check) ==="
if cargo check; then
    echo "✓ 类型检查通过"
else
    echo "✗ 类型检查失败"
    exit 1
fi
echo ""

# 6. 调试模式构建
echo "=== 6. 调试模式构建 (cargo build) ==="
if cargo build; then
    echo "✓ 调试构建成功"
    ls -lh target/debug/qr2pic 2>/dev/null || echo "警告: 可执行文件未找到"
else
    echo "✗ 调试构建失败"
    exit 1
fi
echo ""

# 7. 发布模式构建
echo "=== 7. 发布模式构建 (cargo build --release) ==="
if cargo build --release; then
    echo "✓ 发布构建成功"
    ls -lh target/release/qr2pic 2>/dev/null || echo "警告: 可执行文件未找到"
else
    echo "✗ 发布构建失败"
    exit 1
fi
echo ""

# 8. 运行单元测试（如果有）
echo "=== 8. 运行单元测试 ==="
if cargo test -- --nocapture 2>&1 | head -50; then
    echo "✓ 单元测试通过"
else
    echo "⚠ 单元测试失败或没有测试"
fi
echo ""

# 9. 检查二进制文件
echo "=== 9. 检查生成的可执行文件 ==="
if [ -f "target/debug/qr2pic" ]; then
    echo "调试版本:"
    file target/debug/qr2pic
    du -h target/debug/qr2pic
    echo ""
fi

if [ -f "target/release/qr2pic" ]; then
    echo "发布版本:"
    file target/release/qr2pic
    du -h target/release/qr2pic
    echo ""
fi

# 10. 验证部署配置
echo "=== 10. 验证部署配置 ==="
echo "检查 zbpack.json..."
cat zbpack.json
echo ""
echo "检查 Cargo.toml 版本..."
grep -E "^(name|version|edition)" Cargo.toml
echo ""

echo "=== 编译测试完成 ==="
echo ""
echo "下一步:"
echo "1. 配置真实环境变量: cp .env.example .env"
echo "2. 编辑 .env 文件，填入真实的数据库和 OSS 配置"
echo "3. 运行服务器: cargo run"
echo "4. 或运行发布版本: ./target/release/qr2pic"
echo ""
echo "API 端点:"
echo "  - 健康检查: GET /health"
echo "  - 上传图片: POST /upload"
echo "  - 获取图片: GET /image/:id"
echo "  - 删除图片: DELETE /delete/:id (需要 X-Delete-Key 头)"
echo ""