# Yumi-BiliBili-Download

用于下载 Bilibili 资源的小工具

## Disclaimer

⚠️ 本项目仅用于学习、研究和技术交流

## 仓库结构

本项目为 Cargo workspace，包含两个 crate：

- `ybd-core`：核心库（登录、视频解析、下载、进度等）
- `ybd-cli`：命令行工具（二进制名为 `ybd`）

---

## 使用

注意: 如果下载视频，则需要确认自己本地已安装ffmpeg并添加到PATH环境变量

```bash
# 查看下载用法
ybd download --help
# 扫描二维码登录账户
ybd login

ybd download BVxxxx/包含BV号的链接

ybd audio BVxxxx/包含BV号的链接

ybd cover BVxxxx/包含BV号的链接
# 列表下载
ybd download audio -b ml240xxxx/包含ml的链接
```

## 下载

### linux下

`curl -fsSL https://raw.githubusercontent.com/yumilengjiao/yumi-bilibili-download/refs/heads/master/scripts/install.sh | sh`

### windows下

`irm https://raw.githubusercontent.com/yumilengjiao/yumi-bilibili-download/refs/heads/master/scripts/install.ps1 | iex`

或者你可以从[发布页面](https://github.com/yumilengjiao/yumi-bilibili-download/releases)手动安装并加入PATH环境变量

## license

本项目采用MIT开源许可
