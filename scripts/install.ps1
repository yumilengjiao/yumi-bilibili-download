#Requires -Version 5.0
$ErrorActionPreference = "Stop"

$Repo = "yumilengjiao/yumi-bilibili-download"
$BinName = "yumi-bilibili-download"
$InstallName = "ybd"
$InstallDir = "$env:LOCALAPPDATA\Programs\ybd"

# 检测平台
function Get-Target {
    $Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($Arch) {
        "X64" { return "x86_64-pc-windows-msvc" }
        default {
            Write-Error "不支持的架构: $Arch，目前只提供 x86_64 Windows 版本"
            exit 1
        }
    }
}

# 获取最新版本号
function Get-LatestVersion {
    $Response = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    return $Response.tag_name
}

$Target = Get-Target
$Version = Get-LatestVersion

if (-not $Version) {
    Write-Error "无法获取最新版本号"
    exit 1
}

$FileName = "${BinName}-${Version}-${Target}.exe"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$FileName"
$TmpFile = Join-Path $env:TEMP $FileName

Write-Host "版本: $Version"
Write-Host "平台: $Target"
Write-Host "正在下载..."

Invoke-WebRequest -Uri $DownloadUrl -OutFile $TmpFile

# 创建安装目录
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

$InstallPath = Join-Path $InstallDir "${InstallName}.exe"
Move-Item -Force $TmpFile $InstallPath

# 检查 PATH 里有没有安装目录
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable(
        "PATH",
        "$UserPath;$InstallDir",
        "User"
    )
    Write-Host "已将 $InstallDir 加入 PATH（重启终端后生效）"
}

Write-Host "安装完成: $InstallPath"
Write-Host "运行 '$InstallName --version' 验证安装"
