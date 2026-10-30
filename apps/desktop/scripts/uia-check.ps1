# SPDX-FileCopyrightText: 2026 SONARITHM contributors
# SPDX-License-Identifier: MIT OR Apache-2.0

<#
.SYNOPSIS
  Dumps the desktop app's UI Automation tree in WebView2 and checks it (B02-T03, OD-15).

.DESCRIPTION
  The A02 prototype's UIA dump (F-045) as a CI check on Windows. The script starts the app,
  waits until the element named by the expectation file's "ready" text appears, then walks
  the control view. It fails if an expected element (localized control type and name) is
  missing, or if any interactive element (button, check box, radio button, edit, combo box,
  slider, list, tree or tab item, menu item, hyperlink) has an empty name. The dump is
  written next to the expectation file's name for the CI artifacts. This is a smoke check of
  roles and names, not a screen-reader result.

.EXAMPLE
  powershell -File apps/desktop/scripts/uia-check.ps1 -Exe target/debug/sonarithm-desktop.exe `
    -Expect apps/desktop/test/uia/workspace.json -Out uia-workspace.txt
#>
param(
  [Parameter(Mandatory)] [string] $Exe,
  [Parameter(Mandatory)] [string] $Expect,
  [Parameter(Mandatory)] [string] $Out,
  [int] $TimeoutSeconds = 60
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$spec = Get-Content -Raw -Encoding UTF8 $Expect | ConvertFrom-Json

$interactive = @('Button', 'CheckBox', 'RadioButton', 'Edit', 'ComboBox', 'Slider', 'ListItem',
  'TreeItem', 'TabItem', 'MenuItem', 'Hyperlink', 'SplitButton', 'Spinner')

function Get-Tree([System.Windows.Automation.AutomationElement] $root) {
  $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
  $stack = New-Object System.Collections.Stack
  $stack.Push([pscustomobject]@{ El = $root; Depth = 0 })
  $rows = New-Object System.Collections.ArrayList
  while ($stack.Count -gt 0) {
    $item = $stack.Pop()
    $el = $item.El; $depth = $item.Depth
    $c = $el.Current
    [void]$rows.Add([pscustomobject]@{
        Depth = $depth
        Type  = ($c.ControlType.ProgrammaticName -replace '^ControlType\.', '')
        LocalizedType = $c.LocalizedControlType
        Name  = $c.Name
      })
    $children = New-Object System.Collections.ArrayList
    $child = $walker.GetFirstChild($el)
    while ($null -ne $child) { [void]$children.Add($child); $child = $walker.GetNextSibling($child) }
    for ($i = $children.Count - 1; $i -ge 0; $i--) {
      $stack.Push([pscustomobject]@{ El = $children[$i]; Depth = $depth + 1 })
    }
  }
  return $rows
}

$proc = Start-Process -FilePath $Exe -PassThru
try {
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  $tree = $null
  while ((Get-Date) -lt $deadline) {
    $proc.Refresh()
    if ($proc.HasExited) { throw "The app exited with code $($proc.ExitCode) before it was ready." }
    if ($proc.MainWindowHandle -ne [IntPtr]::Zero) {
      $window = $A::FromHandle($proc.MainWindowHandle)
      $tree = Get-Tree $window
      if ($tree | Where-Object { $_.Name -eq $spec.ready }) { break }
    }
    Start-Sleep -Milliseconds 500
  }
  if ($null -eq $tree -or -not ($tree | Where-Object { $_.Name -eq $spec.ready })) {
    throw "The app did not show '$($spec.ready)' within $TimeoutSeconds s."
  }

  $lines = $tree | ForEach-Object { ('  ' * $_.Depth) + "$($_.Type) [$($_.LocalizedType)] $($_.Name)" }
  $lines | Set-Content -Encoding UTF8 $Out

  $problems = New-Object System.Collections.ArrayList
  foreach ($want in $spec.expect) {
    $found = $tree | Where-Object { $_.LocalizedType -eq $want.localizedType -and $_.Name -eq $want.name }
    if (-not $found) { [void]$problems.Add("missing: $($want.localizedType) named '$($want.name)'") }
  }
  foreach ($row in $tree) {
    if ($interactive -contains $row.Type -and [string]::IsNullOrWhiteSpace($row.Name)) {
      [void]$problems.Add("unnamed $($row.Type) [$($row.LocalizedType)] at depth $($row.Depth)")
    }
  }
  if ($problems.Count -gt 0) {
    $problems | ForEach-Object { Write-Host "::error::UIA check: $_" }
    exit 1
  }
  Write-Host "UIA check passed: $($spec.expect.Count) expected elements, $($tree.Count) elements in the tree."
}
finally {
  if (-not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
}
