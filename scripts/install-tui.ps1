$ErrorActionPreference = "Stop"

$Repo = "yumilengjiao/yumi-bilibili-download"
$InstallName = "ybdtui"
$InstallDir = "$env:LOCALAPPDATA\Programs\$InstallName"
$Target = "x86_64-pc-windows-msvc"

# 获取最新版本号
$ApiUrl = "https://api.github.com/repos/$Repo/releases/latest"
$Headers = @{ "User-Agent" = "ybdtui-installer" }
$Release = Invoke-RestMethod -Uri $ApiUrl -Headers $Headers
$Version = $Release.tag_name
if (-not $Version) {
    Write-Error "无法获取最新版本号"
    exit 1
}

$FileName = "${InstallName}-${Target}.exe"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$FileName"
$TmpFile = Join-Path $env:TEMP $FileName

Write-Host "目标组件: $InstallName (TUI终端界面版)"
Write-Host "版本: $Version"
Write-Host "平台: $Target"
Write-Host "正在下载..."
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TmpFile

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

$InstallPath = Join-Path $InstallDir "${InstallName}.exe"
Move-Item -Force $TmpFile $InstallPath

$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
    Write-Host "已将 $InstallDir 加入 PATH（重启终端后生效）"
}

Write-Host "安装完成: $InstallPath"
Write-Host "运行 '$InstallName' 即可启动"
