# QR2Pic

基于二维码扫描的图片展示系统后端。上传图片后获得 UUID，将 UUID 生成二维码，扫码即可查看对应图片。注重稳定性与错误处理。

## 技术栈

- **语言**: Rust (2021 edition)
- **Web 框架**: axum + tokio
- **数据库**: PostgreSQL，使用 sqlx 交互（表 `images`：`id` UUID、`file_path` String、`created_at` Timestamp）
- **存储**: 本地文件存储，通过 `STORAGE_PATH` / `STORAGE_URL` 配置

## 项目结构

```
QR2Pic/
├── Cargo.toml              # Rust 依赖配置
├── Dockerfile              # 多阶段镜像构建
├── docker-compose.yml      # 应用 + PostgreSQL 编排
├── .env.example            # 环境变量示例
├── migrations/             # 数据库迁移
│   └── 0001_create_images.sql
├── scripts/                # 批量上传/处理工具（Python，仅代码入库）
└── src/
    ├── main.rs             # 应用入口
    ├── lib.rs              # 模块导出
    ├── error.rs            # 错误类型定义
    ├── config/             # 配置管理
    ├── db/                 # 数据库模型和存储库
    ├── storage/            # 本地文件存储
    ├── routes/             # API 路由定义
    ├── handlers/           # 请求处理器
    └── middleware/         # 中间件（预留）
```

## API 接口

### POST `/upload`
- **功能**: 上传图片文件，保存到本地存储并将映射关系写入数据库
- **认证**: 设置了 `UPLOAD_KEY` 环境变量时，必须携带匹配的 `X-Upload-Key` 请求头（生产环境务必设置）
- **请求**: `multipart/form-data`，字段 `file`
- **限制**: 最大 10MB（流式校验，超限即断）；支持 JPG、PNG、GIF、WebP，扩展名与文件内容魔数双重校验
- **响应**:
```json
{
  "id": "uuid-string",
  "url": "https://your-domain/images/uuid-string.jpg"
}
```

### GET `/image/:id`
- **功能**: 直接返回图片二进制数据（带正确的 `Content-Type` 和长缓存头）

### GET `/view/:id`
- **功能**: 返回一个自适应全屏展示图片的 HTML 页面（扫码落地页）

### GET `/view-data/:id`
- **功能**: 供 `/view/:id` 页面引用的图片数据接口

### DELETE `/delete/:id`
- **功能**: 删除图片（数据库记录 + 存储文件）
- **认证**: `X-Delete-Key` 请求头，需与环境变量 `DELETE_KEY` 匹配
- **响应**: HTTP 204 No Content

### PUT `/restore/:key`
- **功能**: 按指定文件名把图片文件恢复到存储（仅写文件，不动数据库），用于数据库记录仍在但文件丢失的灾难恢复
- **认证**: 同 `X-Delete-Key`
- **请求体**: 原始文件字节，最大 10MB

### GET `/health`
- **功能**: 健康检查
- **响应**: `OK`

## 使用示例

```bash
# 上传图片
curl -X POST http://localhost:3000/upload \
  -F "file=@/path/to/image.jpg"

# 获取图片
curl http://localhost:3000/image/<uuid>

# 删除图片
curl -X DELETE http://localhost:3000/delete/<uuid> \
  -H "X-Delete-Key: your_delete_key_here"

# 健康检查
curl http://localhost:3000/health
```

## 环境变量

| 变量名 | 说明 | 示例 |
|--------|------|------|
| DATABASE_URL | PostgreSQL 连接字符串（必填） | `postgresql://user:pass@host:5432/qr2pic` |
| STORAGE_PATH | 图片存储目录（默认 `/app/uploads`） | `./uploads` |
| STORAGE_URL | 图片外链基础 URL | `http://localhost:3000/images` |
| SERVER_PORT | 服务端口（默认 3000） | `3000` |
| DELETE_KEY | 删除/恢复接口密钥（必填，建议 32 位随机串） | `your_delete_key_here` |
| UPLOAD_KEY | 上传接口密钥（可选，生产强烈建议设置） | `your_upload_key_here` |
| RUST_LOG | 日志级别（可选） | `info` |

## 本地开发

1. **安装 Rust**: [rustup.rs](https://rustup.rs/)
2. **配置环境**: 复制 `.env.example` 为 `.env` 并填写实际值
3. **准备数据库**: 启动 PostgreSQL（也可用 `docker compose up -d postgres`）
4. **编译运行**:
   ```bash
   cargo build
   cargo run
   ```
   启动时会自动执行 `migrations/` 下的数据库迁移。

也可以用 Docker Compose 一键启动应用和数据库：

```bash
docker compose up -d --build
```

## 数据库迁移

```bash
# 创建新迁移
sqlx migrate add migration_name

# 手动运行迁移（应用启动时也会自动执行）
sqlx migrate run
```

## 批量上传工具

`scripts/` 目录提供 Python 批量处理脚本（批量上传、二维码重命名、按文件名恢复等），仅代码入库，图片数据不参与版本控制。依赖见 `scripts/requirements.txt`。

## 安全说明

1. **写操作鉴权**: `DELETE /delete/:id` 与 `PUT /restore/:key` 通过 `X-Delete-Key` 请求头认证；设置 `UPLOAD_KEY` 后 `POST /upload` 同样需要 `X-Upload-Key`。密钥比较使用恒定时间算法，防时序侧信道
2. **文件验证**: 扩展名白名单 + 文件内容魔数双重校验；大小限制为流式校验，超限立即截断，防止内存耗尽攻击
3. **路径安全**: 上传文件名不参与路径拼接；恢复接口拒绝包含路径分隔符或 `..` 的文件名，防止路径穿越
4. **错误脱敏**: 5xx 错误只向客户端返回通用提示，SQL 错误、文件路径等细节仅写入服务端日志
5. **数据库**: docker-compose 中 PostgreSQL 仅绑定本地回环地址，密码无默认值必须显式配置
6. **CORS**: 默认允许所有来源，生产环境可按需收紧
