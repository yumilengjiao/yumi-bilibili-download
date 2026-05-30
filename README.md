# Yumi-BiliBili-Download

用于下载 Bilibili 资源的小工具

## Disclaimer

⚠️ 本项目仅用于学习、研究和技术交流

---

## 使用

注意: 如果下载视频，则需要确认自己本地已安装ffmpeg并添加到PATH环境变量

```bash
# 查看下载用法
ybd download --help 
# 扫描二维码登录账户
yumi login

yumi download BVxxxx/包含BV号的链接

yumi audio BVxxxx/包含BV号的链接

yumi cover BVxxxx/包含BV号的链接
# 列表下载
ybd download audio -b ml240**/包含ml的链接
```

## 下载

### linux下

`curl -fsSL https://raw.githubusercontent.com/yumilengjiao/yumi-bilibili-download/master/install.sh | sh`

### windows下

`irm https://raw.githubusercontent.com/yumilengjiao/yumi-bilibili-download/master/install.ps1 | iex`

或者你可以从[发布页面](https://github.com/yumilengjiao/yumi-bilibili-download/releases)手动安装并加入PATH环境变量
