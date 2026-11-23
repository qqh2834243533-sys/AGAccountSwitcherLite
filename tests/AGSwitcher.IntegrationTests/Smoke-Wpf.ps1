[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$Executable,
    [string]$DotnetRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes

function Wait-For([scriptblock]$Probe, [string]$Description) {
    $timer = [Diagnostics.Stopwatch]::StartNew()
    do {
        $value = & $Probe
        if ($value) { return $value }
        Start-Sleep -Milliseconds 100
    } while ($timer.Elapsed.TotalSeconds -lt 12)
    throw "Timed out waiting for $Description"
}

function Find-Window([int]$ProcessId) {
    $condition = [Windows.Automation.PropertyCondition]::new(
        [Windows.Automation.AutomationElement]::ProcessIdProperty, $ProcessId)
    [Windows.Automation.AutomationElement]::RootElement.FindFirst(
        [Windows.Automation.TreeScope]::Children, $condition)
}

function Find-Control([string]$Id) {
    $condition = [Windows.Automation.PropertyCondition]::new(
        [Windows.Automation.AutomationElement]::AutomationIdProperty, $Id)
    Wait-For { $window.FindFirst([Windows.Automation.TreeScope]::Descendants, $condition) } $Id
}

function Invoke-Button([string]$Id) {
    (Find-Control $Id).GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
}

function Get-AccountItems {
    $list = Find-Control 'AccountList'
    $list.FindAll([Windows.Automation.TreeScope]::Children, [Windows.Automation.Condition]::TrueCondition)
}

function Select-Account([int]$Index) {
    $items = @(Get-AccountItems)
    $items[$Index].GetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern).Select()
}

function Type-Confirmation([string]$Value) {
    (Find-Control 'ConfirmationInput').GetCurrentPattern([Windows.Automation.ValuePattern]::Pattern).SetValue($Value)
}

function Close-TestProcess($Process) {
    if ($null -eq $Process) { return }
    $Process.Refresh()
    if ($Process.HasExited) { return }
    if (-not $Process.CloseMainWindow() -or -not $Process.WaitForExit(4000)) {
        $Process.Kill()
        $Process.WaitForExit(4000) | Out-Null
    }
}

$first = $null
$second = $null
$previousDotnetRoot = $env:DOTNET_ROOT
try {
    if ($DotnetRoot) { $env:DOTNET_ROOT = (Resolve-Path -LiteralPath $DotnetRoot).Path }
    $appPath = (Resolve-Path -LiteralPath $Executable).Path
    $first = Start-Process -FilePath $appPath -PassThru
    $window = Wait-For { Find-Window $first.Id } 'the simulation window'

    $count = @(Get-AccountItems).Count
    if ($count -ne 4) { throw "Expected four visible accounts, got $count" }
    $active = Find-Control 'ActiveAccount'
    if ($active.Current.Name -notmatch '01$') { throw 'Unexpected starting account' }

    $second = Start-Process -FilePath $appPath -PassThru
    $mutexDialog = Wait-For { Find-Window $second.Id } 'the second instance message'
    $mutexDialog.GetCurrentPattern([Windows.Automation.WindowPattern]::Pattern).Close()
    if (-not $second.WaitForExit(5000) -or $second.ExitCode -ne 2) {
        throw 'Second process did not exit with the instance-conflict code'
    }

    Select-Account 1
    Invoke-Button 'BeginSwitch'
    $challenge = (Find-Control 'Challenge').Current.Name
    if ($challenge -notmatch '^[1-9][0-9]{3}$') { throw 'Invalid confirmation display' }

    Type-Confirmation '0000'
    Invoke-Button 'ConfirmSwitch'
    Wait-For { (Find-Control 'ConfirmationError').Current.Name } 'the incorrect-code message' | Out-Null
    if ($active.Current.Name -notmatch '01$') { throw 'Incorrect code changed the active account' }

    Invoke-Button 'CancelSwitch'
    Wait-For { (Find-Control 'BeginSwitch').Current.IsEnabled } 'return from cancellation' | Out-Null
    Invoke-Button 'BeginSwitch'
    $challenge = (Find-Control 'Challenge').Current.Name
    Type-Confirmation $challenge
    Invoke-Button 'ConfirmSwitch'
    Wait-For { $active.Current.Name -match '02$' } 'the verified active account' | Out-Null

    Invoke-Button 'ResetSimulation'
    Wait-For { $active.Current.Name -match '01$' } 'restored initial account' | Out-Null
    $window.GetCurrentPattern([Windows.Automation.WindowPattern]::Pattern).Close()
    if (-not $first.WaitForExit(5000) -or $first.ExitCode -ne 0) { throw 'Normal window close failed' }

    $first = Start-Process -FilePath $appPath -PassThru
    $window = Wait-For { Find-Window $first.Id } 'reopened simulation window'
    if ((Find-Control 'ActiveAccount').Current.Name -notmatch '01$') { throw 'Incorrect state after restart' }
    Write-Output 'WPF smoke passed: launch, four accounts, second process rejection, wrong code, cancellation, switch, reset, and relaunch.'
}
finally {
    Close-TestProcess $second
    Close-TestProcess $first
    $env:DOTNET_ROOT = $previousDotnetRoot
}
