# RustIM - 基于Rust的云原生IM系统

这是一个使用Rust语言开发的微服务架构即时通讯系统，采用云原生设计理念。

## 系统架构

系统由以下微服务组成：

1. **认证服务 (auth-service)**：负责token管理、权限校验
2. **用户服务 (user-service)**：负责用户注册、登录、认证和查询用户信息
3. **群组服务 (group-service)**：管理群组及成员关系
4. **好友服务 (friend-service)**：管理用户之间的好友关系
5. **私聊消息服务器 (private-message-server)**：负责一对一聊天消息的处理和分发
6. **群聊消息服务 (group-message-server)**：负责群组聊天消息的处理和分发
7. **消息网关（message-gateway）**：负责与客户端的WebSocket连接管理和消息推送
8. **API网关服务 (gateway-service)**：统一认证、路由转发和负载均衡

## 技术栈

- **语言**: Rust
- **通信协议**: gRPC (服务间), REST/WebSocket (客户端)
- **数据库**: PostgreSQL, Redis
- **消息队列**: Kafka
- **API框架**: Axum (HTTP), Tonic (gRPC)
- **容器化**: Docker
- **配置管理**: dotenv + config
- **监控**: Prometheus

## 开发环境要求

- Rust 1.75+
- Docker & Docker Compose
- PostgreSQL
- Redis
- Kafka

## 快速开始

1. 克隆仓库

```bash
git clone https://github.com/yourusername/rustIM_demo.git
cd rustIM_demo
```

2. 启动依赖服务 (PostgreSQL, Redis, Kafka)

```bash
docker-compose up -d
```

3. 设置环境变量

```bash
cp .env.example .env
# 编辑.env文件，设置数据库凭证等
```

4. 构建所有服务

```bash
cargo build
```

5. 运行所有服务

```bash
./scripts/start-all.sh
```

## 服务间通信流程

1. 客户端通过API网关进行认证并获取token
2. 客户端通过WebSocket连接到消息网关
3. 用户发送消息时:
   - 私聊消息流向: 客户端 -> 消息网关 -> 私聊消息服务 -> 消息队列 -> 消息网关 -> 接收方客户端
   - 群聊消息流向: 客户端 -> 消息网关 -> 群聊消息服务 -> 消息队列 -> 消息网关 -> 多个接收方客户端

## 项目结构

```
rustIM_demo/
├── auth-service/           # 认证服务
├── user-service/           # 用户服务
├── group-service/          # 群组服务
├── friend-service/         # 好友服务
├── private-message-server/ # 私聊消息服务
├── group-message-server/   # 群聊消息服务
├── message-gateway/        # 消息网关
├── gateway-service/        # API网关
├── common/                 # 共享代码库
├── docker-compose.yml      # 容器编排
└── scripts/                # 运维脚本
```

## 贡献指南

欢迎提交Issue和Pull Request。请确保代码通过测试并符合项目的代码规范。

## 配置系统

RustIM 支持灵活的配置管理，特别适合容器化环境（如 Docker 和 Kubernetes）。

### 配置来源

配置按以下优先级顺序加载（高优先级会覆盖低优先级）：

1. 环境变量（最高优先级）
2. 指定的配置文件（通过 `--config` 参数）
3. 默认配置文件（按顺序查找）：
   - config.yaml
   - config.json
   - config.toml
   - .env
4. 默认值（最低优先级）

### 支持的配置格式

- YAML 文件 (*.yaml, *.yml)
- JSON 文件 (*.json)
- TOML 文件 (*.toml)
- 环境变量文件 (.env)

### 动态配置

系统支持动态配置更新，无需重启服务：

```bash
# 启动服务时指定配置刷新间隔（秒）
./auth-service --config config.yaml --refresh 30
```

### Docker 环境配置

在 Docker 环境中，可以：

1. 挂载配置文件：
   ```bash
   docker run -v ./config.yaml:/app/config.yaml your-image --config /app/config.yaml
   ```

2. 使用环境变量：
   ```bash
   docker run -e REDIS_URL=redis://redis:6379 -e JWT_SECRET=your_secret your-image
   ```

### Kubernetes 环境配置

在 Kubernetes 中，推荐使用 ConfigMap 和 Secret 管理配置：

1. 创建配置文件的 ConfigMap：
   ```bash
   kubectl create configmap auth-service-config --from-file=config.yaml
   ```

2. 在 Deployment 中挂载配置：
   ```yaml
   volumes:
   - name: config-volume
     configMap:
       name: auth-service-config
   volumeMounts:
   - name: config-volume
     mountPath: /config
   ```

3. 启用 Kubernetes 配置：
   ```yaml
   command: ["/app/auth-service", "--k8s-config"]
   ```

4. 敏感信息使用 Secret：
   ```yaml
   env:
   - name: JWT_SECRET
     valueFrom:
       secretKeyRef:
         name: auth-service-secrets
         key: jwt-secret
   ```

## 配置变更通知

当配置发生变更时，服务会自动重新加载配置而无需重启。日志会记录配置更新事件：

```
[INFO] 配置已更新
``` 