$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptRoot
$srcRoot = Join-Path $projectRoot "src"
$src = Join-Path $srcRoot "slint_ui.rs"
$uiDir = Join-Path $srcRoot "slint_ui"

$content = [System.IO.File]::ReadAllText($src)
$marker = "slint::slint! {"
$start = $content.IndexOf($marker)
if ($start -lt 0) {
    throw "slint block start not found"
}

$braceStart = $content.IndexOf("{", $start)
$depth = 0
$end = -1
for ($i = $braceStart; $i -lt $content.Length; $i++) {
    $ch = $content[$i]
    if ($ch -eq "{") {
        $depth++
    }
    elseif ($ch -eq "}") {
        $depth--
        if ($depth -eq 0) {
            $end = $i
            break
        }
    }
}

if ($end -lt 0) {
    throw "slint block end not found"
}

$inner = $content.Substring($braceStart + 1, $end - $braceStart - 1)
$before = $content.Substring(0, $start)
$after = $content.Substring($end + 1)

function Get-ComponentBlock {
    param(
        [string]$Source,
        [string]$Name
    )

    $pattern = "(?ms)(^|\n)\s*(export\s+)?component\s+$([regex]::Escape($Name))\b"
    $match = [regex]::Match($Source, $pattern)
    if (-not $match.Success) {
        throw "component not found: $Name"
    }

    $componentStart = $match.Index + $match.Groups[1].Length
    $openBrace = $Source.IndexOf("{", $componentStart)
    if ($openBrace -lt 0) {
        throw "open brace not found for component: $Name"
    }

    $depth = 0
    $closeBrace = -1
    for ($i = $openBrace; $i -lt $Source.Length; $i++) {
        $ch = $Source[$i]
        if ($ch -eq "{") {
            $depth++
        }
        elseif ($ch -eq "}") {
            $depth--
            if ($depth -eq 0) {
                $closeBrace = $i
                break
            }
        }
    }

    if ($closeBrace -lt 0) {
        throw "close brace not found for component: $Name"
    }

    return $Source.Substring($componentStart, $closeBrace - $componentStart + 1).Trim()
}

$commonNames = @(
    "MenuButton",
    "SubMenuButton",
    "InfoRow",
    "FormRow",
    "EditableRow",
    "PathBrowseRow",
    "LargeEditableText",
    "QuestionTableRow",
    "QuestionIdTableRow",
    "QuestionTile",
    "QuestionOverviewGrid",
    "QuestionConfigPanel",
    "ActionButton",
    "SettingsIconButton",
    "StaticRow",
    "LogPanel"
)

$hostNames = @("HostDashboard", "SettingsDialog")
$storyNames = @("StoryLockCoreApp", "StoryLockCoreSettingsDialog", "AnswerEditorDialog")
$requestNames = @("RequestConfirmation")

New-Item -ItemType Directory -Force -Path $uiDir | Out-Null

$commonHeader = @'
import { Button, ComboBox, LineEdit, ScrollView, TextEdit, VerticalBox, HorizontalBox } from "std-widgets.slint";

'@

$commonBody = ($commonNames | ForEach-Object { Get-ComponentBlock -Source $inner -Name $_ }) -join "`r`n`r`n"
[System.IO.File]::WriteAllText((Join-Path $uiDir "common.slint"), $commonHeader + $commonBody + "`r`n")

$hostHeader = @'
import { Button, ComboBox, LineEdit, ScrollView, TextEdit, VerticalBox, HorizontalBox } from "std-widgets.slint";
import { MenuButton, SubMenuButton, InfoRow, FormRow, EditableRow, PathBrowseRow, LargeEditableText, QuestionTableRow, QuestionIdTableRow, QuestionOverviewGrid, QuestionConfigPanel, ActionButton, SettingsIconButton, StaticRow, LogPanel } from "common.slint";

'@

$hostBody = ($hostNames | ForEach-Object { Get-ComponentBlock -Source $inner -Name $_ }) -join "`r`n`r`n"
[System.IO.File]::WriteAllText((Join-Path $uiDir "host_dashboard.slint"), $hostHeader + $hostBody + "`r`n")

$storyHeader = @'
import { Button, ComboBox, LineEdit, ScrollView, TextEdit, VerticalBox, HorizontalBox } from "std-widgets.slint";
import { MenuButton, SubMenuButton, InfoRow, FormRow, EditableRow, PathBrowseRow, LargeEditableText, QuestionTableRow, QuestionIdTableRow, QuestionOverviewGrid, QuestionConfigPanel, ActionButton, SettingsIconButton, StaticRow, LogPanel } from "common.slint";

'@

$storyBody = ($storyNames | ForEach-Object { Get-ComponentBlock -Source $inner -Name $_ }) -join "`r`n`r`n"
[System.IO.File]::WriteAllText((Join-Path $uiDir "storylock_core.slint"), $storyHeader + $storyBody + "`r`n")

$requestHeader = @'
import { Button, ComboBox, LineEdit, ScrollView, TextEdit, VerticalBox, HorizontalBox } from "std-widgets.slint";
import { ActionButton, InfoRow, StaticRow } from "common.slint";

'@

$requestBody = ($requestNames | ForEach-Object { Get-ComponentBlock -Source $inner -Name $_ }) -join "`r`n`r`n"
[System.IO.File]::WriteAllText((Join-Path $uiDir "request_confirmation.slint"), $requestHeader + $requestBody + "`r`n")

$newSlintBlock = @'
slint::slint! {
    import { HostDashboard, SettingsDialog } from "slint_ui/host_dashboard.slint";
    import { StoryLockCoreApp, StoryLockCoreSettingsDialog, AnswerEditorDialog } from "slint_ui/storylock_core.slint";
    import { RequestConfirmation } from "slint_ui/request_confirmation.slint";

    export {
        HostDashboard,
        SettingsDialog,
        StoryLockCoreApp,
        StoryLockCoreSettingsDialog,
        AnswerEditorDialog,
        RequestConfirmation
    }
}
'@

[System.IO.File]::WriteAllText($src, $before + $newSlintBlock + $after)
Write-Output "split complete"
