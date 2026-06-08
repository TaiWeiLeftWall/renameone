# 🏛️ 图片归档工具 (Image Archive Tool)

一款轻量级桌面工具，自动从图片 EXIF 中提取拍摄日期，按 `YYYY_MM_DD_地点_标题` 格式重命名文件夹。

## 功能

- **📂 文件夹选择** — 原生系统对话框，一键选择要归档的图片文件夹
- **📷 自动扫描** — 递归扫描文件夹内所有图片（支持 jpg/png/tiff/webp/heic）
- **📅 智能日期提取** — 5 级降级策略，自动找出最佳代表日期
  1. EXIF `DateTimeOriginal`（相机原始拍摄时间）
  2. EXIF `DateTimeDigitized`
  3. EXIF `DateTime`
  4. 文件修改时间 (`mtime`)
  5. 文件名正则提取（如 `IMG_20240315_*.jpg`）
- **🎯 日期推断** — 众数优先，多张图片选出现最多的日期，并列时取最早
- **⚠️ 离群检测** — 自动识别与众数相差超过 30 天的图片并给出警告
- **📝 统一命名** — `YYYY_MM_DD_地点_自定义标题`，自动过滤非法字符、处理重名冲突
- **🎨 Claude 设计** — 奶油画布 + 珊瑚主色，简洁舒适的视觉体验

## 截图

```
┌──────────────────────────────────────────────┐
│  图片归档工具                                  │
├──────────────────────────────────────────────┤
│  [ 📂 选择文件夹 ]                            │
│  E:\Photos\未分类                             │
├──────────────────────────────────────────────┤
│  共 23 张图片                                  │
│  推断日期  2024/03/15  (20/23 张)              │
│                                              │
│  地点  [北京颐和园_____________]              │
│  标题  [踏春赏花_______________]              │
│                                              │
│  文件夹将重命名为                              │
│  2024_03_15_北京颐和园_踏春赏花                │
├──────────────────────────────────────────────┤
│  [          开始归档            ]              │
└──────────────────────────────────────────────┘
```

## 快速开始

### 前提条件

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) （编译 Tauri 后端）
- Windows 10+（含 WebView2）

### 安装 & 运行

```bash
# 克隆仓库
git clone https://github.com/your-username/image-archive.git
cd image-archive

# 安装前端依赖
npm install

# 开发模式运行
npm run tauri dev
```

### 构建可分发安装包

```bash
npm run tauri build
# 输出路径: src-tauri/target/release/
```

也可以直接双击 `start.bat` 启动开发模式。

## 技术栈

| 层次 | 技术 | 用途 |
|------|------|------|
| 桌面框架 | [Tauri v2](https://v2.tauri.app/) | 原生窗口、文件系统操作 |
| 前端 | React 19 + TypeScript + Vite 8 | 用户界面 |
| 后端 | Rust | 文件扫描、EXIF 解析、重命名 |
| EXIF 解析 | [kamadak-exif](https://crates.io/crates/kamadak-exif) | 纯 Rust EXIF 元数据读取 |
| 日期处理 | [chrono](https://crates.io/crates/chrono) | 日期解析与计算 |
| 目录遍历 | [walkdir](https://crates.io/crates/walkdir) | 递归文件扫描 |
| 设计系统 | [Claude Design](docs/DESIGN-claude.md) | UI 视觉风格 |

## 项目结构

```
image-archive/
├── src/                    # 前端 (React + TypeScript)
│   ├── App.tsx             # 主界面
│   ├── App.css             # Claude 设计系统样式
│   └── main.tsx            # 入口
├── src-tauri/              # 后端 (Rust)
│   ├── src/
│   │   ├── main.rs         # Tauri 入口
│   │   ├── lib.rs          # 命令注册
│   │   ├── scanner.rs      # 文件扫描
│   │   ├── exif.rs         # EXIF 日期提取
│   │   ├── inference.rs    # 日期推断
│   │   └── renamer.rs      # 文件夹重命名
│   └── tauri.conf.json     # Tauri 配置
├── docs/
│   ├── 设计文档.md          # 完整设计文档
│   └── DESIGN-claude.md    # Claude 设计规范
└── start.bat               # 启动脚本
```

## 开发路线图

**Phase 1 ✅ MVP**
- [x] 文件夹选择 + 图片扫描
- [x] EXIF 日期提取（5 级降级）
- [x] 日期智能推断（众数优先）
- [x] 文件夹重命名（冲突处理 + 非法字符过滤）
- [x] Claude 设计系统 UI

**Phase 2**（计划中）
- [ ] 拖拽文件夹支持
- [ ] 多日期冲突时手动选择
- [ ] 操作日志记录
- [ ] 操作回滚

**Phase 3**（远期）
- [ ] 批量处理多个文件夹
- [ ] 文件名日期逆向提取
- [ ] GPS → 自动地点（反向地理编码）

## 许可证

MIT