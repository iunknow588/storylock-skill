param(
    [string]$ExePath = "E:\2026OPC大赛\skill\src\host\windows-host\target\codex-build\debug\yian-windows-host.exe",
    [string]$OutputDir = "",
    [int]$TimeoutSeconds = 20
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path (Split-Path $PSScriptRoot -Parent) "ui-screenshots"
}

Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;

namespace UiCapture {
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    public static class NativeMethods {
        public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

        [DllImport("user32.dll")]
        public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

        [DllImport("user32.dll")]
        public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);

        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int maxCount);

        [DllImport("user32.dll")]
        public static extern bool IsWindowVisible(IntPtr hWnd);

        [DllImport("user32.dll")]
        public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);

        [DllImport("user32.dll")]
        public static extern bool SetForegroundWindow(IntPtr hWnd);

        [DllImport("user32.dll")]
        public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

        [DllImport("user32.dll")]
        public static extern bool SetCursorPos(int x, int y);

        [DllImport("user32.dll")]
        public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extraInfo);

        [DllImport("user32.dll")]
        public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdcBlt, int nFlags);
    }
}
"@

$mouseLeftDown = 0x0002
$mouseLeftUp = 0x0004
$showNormal = 5

function Get-VisibleWindowsForProcess {
    param([int]$ProcessId)

    $windows = New-Object System.Collections.Generic.List[object]
    $callback = [UiCapture.NativeMethods+EnumWindowsProc]{
        param([IntPtr]$Handle, [IntPtr]$LParam)

        $windowProcessId = 0
        [void][UiCapture.NativeMethods]::GetWindowThreadProcessId($Handle, [ref]$windowProcessId)
        if ($windowProcessId -ne $ProcessId) {
            return $true
        }
        if (-not [UiCapture.NativeMethods]::IsWindowVisible($Handle)) {
            return $true
        }

        $titleBuilder = New-Object System.Text.StringBuilder 512
        [void][UiCapture.NativeMethods]::GetWindowText($Handle, $titleBuilder, $titleBuilder.Capacity)
        $title = $titleBuilder.ToString()
        if ([string]::IsNullOrWhiteSpace($title)) {
            return $true
        }

        $rect = New-Object UiCapture.RECT
        [void][UiCapture.NativeMethods]::GetWindowRect($Handle, [ref]$rect)
        $windows.Add([pscustomobject]@{
            Handle = $Handle
            Title = $title
            Left = $rect.Left
            Top = $rect.Top
            Width = $rect.Right - $rect.Left
            Height = $rect.Bottom - $rect.Top
        }) | Out-Null
        return $true
    }

    [void][UiCapture.NativeMethods]::EnumWindows($callback, [IntPtr]::Zero)
    return $windows
}

function Wait-Window {
    param(
        [int]$ProcessId,
        [string]$TitlePattern,
        [int]$TimeoutSeconds = 20
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        $match = Get-VisibleWindowsForProcess -ProcessId $ProcessId |
            Where-Object { $_.Title -match $TitlePattern } |
            Select-Object -First 1
        if ($null -ne $match) {
            return $match
        }
        Start-Sleep -Milliseconds 250
    }

    throw "Window not found for pattern: $TitlePattern"
}

function Wait-NewWindow {
    param(
        [int]$ProcessId,
        [string[]]$ExcludeTitles,
        [int]$TimeoutSeconds = 20
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        $match = Get-VisibleWindowsForProcess -ProcessId $ProcessId |
            Where-Object { $ExcludeTitles -notcontains $_.Title } |
            Select-Object -First 1
        if ($null -ne $match) {
            return $match
        }
        Start-Sleep -Milliseconds 250
    }

    throw "New window not found."
}

function Focus-Window {
    param([object]$Window)

    [void][UiCapture.NativeMethods]::ShowWindow($Window.Handle, $showNormal)
    Start-Sleep -Milliseconds 200
    [void][UiCapture.NativeMethods]::SetForegroundWindow($Window.Handle)
    Start-Sleep -Milliseconds 400
}

function Capture-Window {
    param(
        [object]$Window,
        [string]$OutputPath
    )

    Focus-Window -Window $Window
    $rect = New-Object UiCapture.RECT
    [void][UiCapture.NativeMethods]::GetWindowRect($Window.Handle, [ref]$rect)
    $width = [Math]::Max(1, $rect.Right - $rect.Left)
    $height = [Math]::Max(1, $rect.Bottom - $rect.Top)

    $OutputPath = [string]$OutputPath
    Write-Host ("Capture -> " + $OutputPath)
    $bitmap = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $hdc = $graphics.GetHdc()
    try {
        $printed = [UiCapture.NativeMethods]::PrintWindow($Window.Handle, $hdc, 0)
    } finally {
        $graphics.ReleaseHdc($hdc)
    }
    if (-not $printed) {
        $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
    }
    $bitmap.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose()
    $bitmap.Dispose()
}

function Click-WindowRelative {
    param(
        [object]$Window,
        [int]$X,
        [int]$Y
    )

    Focus-Window -Window $Window
    $rect = New-Object UiCapture.RECT
    [void][UiCapture.NativeMethods]::GetWindowRect($Window.Handle, [ref]$rect)
    $screenX = $rect.Left + $X
    $screenY = $rect.Top + $Y
    [void][UiCapture.NativeMethods]::SetCursorPos($screenX, $screenY)
    Start-Sleep -Milliseconds 150
    [UiCapture.NativeMethods]::mouse_event($mouseLeftDown, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 60
    [UiCapture.NativeMethods]::mouse_event($mouseLeftUp, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 700
}

function Ensure-HostProcess {
    param([string]$ExePath)

    $existing = Get-Process | Where-Object {
        $_.Path -eq $ExePath -or $_.ProcessName -eq [System.IO.Path]::GetFileNameWithoutExtension($ExePath)
    } | Select-Object -First 1

    if ($null -ne $existing) {
        return $existing
    }

    if (-not (Test-Path -LiteralPath $ExePath)) {
        throw "Executable not found: $ExePath"
    }

    return Start-Process -FilePath $ExePath -PassThru
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$process = Ensure-HostProcess -ExePath $ExePath
$hostWindow = Wait-Window -ProcessId $process.Id -TitlePattern "Yian Windows Host" -TimeoutSeconds $TimeoutSeconds

Capture-Window -Window $hostWindow -OutputPath (Join-Path $OutputDir "01_host_remote_web.png")

Click-WindowRelative -Window $hostWindow -X 90 -Y 110
$hostWindow = Wait-Window -ProcessId $process.Id -TitlePattern "Yian Windows Host" -TimeoutSeconds $TimeoutSeconds
Capture-Window -Window $hostWindow -OutputPath (Join-Path $OutputDir "02_host_storylock_page.png")

Click-WindowRelative -Window $hostWindow -X 790 -Y 58
$settingsWindow = Wait-NewWindow -ProcessId $process.Id -ExcludeTitles @("Yian Windows Host") -TimeoutSeconds $TimeoutSeconds
Capture-Window -Window $settingsWindow -OutputPath (Join-Path $OutputDir "03_host_settings.png")
[System.Windows.Forms.SendKeys]::SendWait("%{F4}")
Start-Sleep -Milliseconds 800

$hostWindow = Wait-Window -ProcessId $process.Id -TitlePattern "Yian Windows Host" -TimeoutSeconds $TimeoutSeconds
Click-WindowRelative -Window $hostWindow -X 90 -Y 265
$coreWindow = Wait-Window -ProcessId $process.Id -TitlePattern "^StoryLock Core$" -TimeoutSeconds $TimeoutSeconds
Capture-Window -Window $coreWindow -OutputPath (Join-Path $OutputDir "04_storylock_core_empty_mode.png")

Click-WindowRelative -Window $coreWindow -X 85 -Y 122
$challengeWindow = Wait-NewWindow -ProcessId $process.Id -ExcludeTitles @("Yian Windows Host", "StoryLock Core") -TimeoutSeconds $TimeoutSeconds
Capture-Window -Window $challengeWindow -OutputPath (Join-Path $OutputDir "05_storylock_authorization_challenge.png")

Get-ChildItem -LiteralPath $OutputDir | Sort-Object Name | Select-Object Name, Length, LastWriteTime
