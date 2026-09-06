# Yumi-BiliBili-Download

用于下载 Bilibili 资源的高性能命令行 (CLI) 与极简终端交互 (TUI) 工具。

## Disclaimer

⚠️ 本项目仅用于学习、研究和技术交流

## 仓库结构

本项目为 Cargo workspace，包含三个 crate：

- `ybd-core`：核心库（登录、视频解析、下载、并发控制、元数据注入等）
- `ybd-cli`：命令行工具（可执行文件名 `ybd`）
- `ybd-tui`：终端图形交互工具（可执行文件名 `ybdtui`）

---

## 使用

> 注意: 如果下载视频，请确认本地已安装 ffmpeg 并添加到 PATH 环境变量中。

### 1. 终端交互版 (TUI)

运行 `ybdtui` 即可开启终端交互界面：
```bash
ybdtui
```
- **快捷键**：`1`~`4` / `h` / `l` 切换标签页，`j` / `k` / `Tab` 移动选择，`i` 进入输入编辑（支持 Emacs `Ctrl+A/E/U/K` 快捷键），`Enter` 执行，`?` / `:help` 呼出帮助浮层。

### 2. 命令行版 (CLI)

```bash
# 扫描二维码登录账户
ybd login

# 单视频下载
ybd download BVxxxx/包含BV号的链接

# 仅下载音频 / 仅下载封面
ybd audio BVxxxx/包含BV号的链接
ybd cover BVxxxx/包含BV号的链接

# 合集/收藏夹批量下载
ybd download audio -b https://www.bilibili.com/list/ml240xxxx...

# 查看详细帮助
ybd download --help
```

---

## 一键安装与下载

### Windows (PowerShell)

```powershell
# 安装 CLI 命令行版 (ybd)
irm https://raw.githubusercontent.com/yumilengjiao/yumi-bilibili-download/refs/heads/master/scripts/install.ps1 | iex

# 安装 TUI 终端界面版 (ybdtui)
irm https://raw.githubusercontent.com/yumilengjiao/yumi-bilibili-download/refs/heads/master/scripts/install-tui.ps1 | iex
```

### Linux / macOS

```bash
# 安装 CLI 命令行版 (ybd)
curl -fsSL https://raw.githubusercontent.com/yumilengjiao/yumi-bilibili-download/refs/heads/master/scripts/install.sh | sh

# 安装 TUI 终端界面版 (ybdtui)
curl -fsSL https://raw.githubusercontent.com/yumilengjiao/yumi-bilibili-download/refs/heads/master/scripts/install-tui.sh | sh
```

或者你可以直接前往 [GitHub Releases](https://github.com/yumilengjiao/yumi-bilibili-download/releases) 手动下载对应的二进制文件（`ybd` 或 `ybdtui`）并加入 PATH 环境变量。

---

## License

本项目采用 MIT 开源许可。
