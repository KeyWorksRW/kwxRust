$ErrorActionPreference = 'Stop'

$src     = 'C:/rwCode/wxSamples/src/sample.xpm'
$out     = 'c:/rwCode/wxLanguages/kwxRust/samples/minimal/data/sample.svg'
$maxRuns = 250000

$x11 = @{
  'black'='#000000'; 'white'='#FFFFFF'; 'red'='#FF0000'; 'green'='#00FF00'; 'blue'='#0000FF'
  'yellow'='#FFFF00'; 'cyan'='#00FFFF'; 'magenta'='#FF00FF'; 'gray'='#BEBEBE'; 'grey'='#BEBEBE'
  'darkgray'='#A9A9A9'; 'darkgrey'='#A9A9A9'; 'lightgray'='#D3D3D3'; 'lightgrey'='#D3D3D3'
}

function Convert-XpmColor([string]$raw, [string]$keyChar) {
  if ($raw -match '^(?i)none$') { return $null }
  if ($raw.StartsWith('#')) {
    $hex = $raw.Substring(1)
    if ($hex.Length -eq 3) {
      $r = [Convert]::ToInt32("$($hex[0])$($hex[0])", 16)
      $g = [Convert]::ToInt32("$($hex[1])$($hex[1])", 16)
      $b = [Convert]::ToInt32("$($hex[2])$($hex[2])", 16)
    } elseif ($hex.Length -eq 6) {
      $r = [Convert]::ToInt32($hex.Substring(0,2), 16)
      $g = [Convert]::ToInt32($hex.Substring(2,2), 16)
      $b = [Convert]::ToInt32($hex.Substring(4,2), 16)
    } elseif ($hex.Length -eq 12) {
      $r = [Convert]::ToInt32($hex.Substring(0,2), 16)
      $g = [Convert]::ToInt32($hex.Substring(4,2), 16)
      $b = [Convert]::ToInt32($hex.Substring(8,2), 16)
    } else {
      throw "unsupported hex colour '$raw'"
    }
    return ('#{0:X2}{1:X2}{2:X2}' -f $r, $g, $b)
  }
  $name = $raw.ToLowerInvariant()
  if ($x11.ContainsKey($name)) { return $x11[$name] }
  throw "unresolved XPM colour `"$raw`" for palette char '$keyChar'"
}

$raw    = [System.IO.File]::ReadAllText($src)
$quoted = @([regex]::Matches($raw, '"([^"]*)"') | ForEach-Object { $_.Groups[1].Value })
if ($quoted.Count -lt 3) { throw 'malformed XPM: no quoted lines' }

$hdr     = $quoted[0].Trim() -split '\s+'
$W       = [int]$hdr[0]
$H       = [int]$hdr[1]
$ncolors = [int]$hdr[2]
$cpp     = [int]$hdr[3]
if ($quoted.Count -lt (1 + $ncolors + $H)) {
  throw "malformed XPM: fewer pixel lines than declared height $H"
}

$palette = @{}
for ($i = 0; $i -lt $ncolors; $i++) {
  $line = $quoted[1 + $i]
  if ($line.Length -lt $cpp) { throw "malformed XPM: colour line $i too short" }
  $ch   = $line.Substring(0, $cpp)
  $rest = $line.Substring($cpp).Trim()
  $tokens = @($rest -split '\s+')
  $cval = $null
  for ($t = 0; $t -lt ($tokens.Count - 1); $t++) {
    if ($tokens[$t] -eq 'c') { $cval = $tokens[$t + 1]; break }
  }
  if ($null -eq $cval) { throw "malformed XPM: colour line $i has no c key" }
  $palette[$ch] = Convert-XpmColor $cval $ch
}

# horizontal runs
$runs = New-Object System.Collections.ArrayList
for ($y = 0; $y -lt $H; $y++) {
  $line = $quoted[1 + $ncolors + $y]
  if ($line.Length -lt ($W * $cpp)) {
    throw "malformed XPM: pixel line $y shorter than declared width $W (cpp $cpp)"
  }
  $x = 0
  while ($x -lt $W) {
    $ch = $line.Substring($x * $cpp, $cpp)
    if (-not $palette.ContainsKey($ch)) {
      throw "malformed XPM: pixel char '$ch' at row $y col $x not in palette"
    }
    $len = 1
    while (($x + $len) -lt $W -and $line.Substring(($x + $len) * $cpp, $cpp) -eq $ch) { $len++ }
    $null = $runs.Add([pscustomobject]@{ x = $x; y = $y; w = $len; color = $palette[$ch] })
    $x += $len
  }
}

$runCount = $runs.Count
if ($runCount -gt $maxRuns) {
  if (Test-Path -LiteralPath $out) { Remove-Item -LiteralPath $out -Force }
  Write-Output "FAILURE: $runCount runs exceed MAX_RUNS ($maxRuns) - rerun with MODE: quantize COLORS: 64"
  exit 2
}

# vertical merge of horizontally identical, vertically contiguous runs
$rects = New-Object System.Collections.ArrayList
$prev  = @{}
$prevY = -1
foreach ($r in $runs) {
  if ($r.y -ne $prevY) { $prev = @{}; $prevY = $r.y }
  if ($null -eq $r.color) { continue }
  $key    = "$($r.x)|$($r.w)|$($r.color)"
  $merged = $false
  if ($prev.ContainsKey($key)) {
    $idx = $prev[$key]
    if (($rects[$idx].y + $rects[$idx].h) -eq $r.y) {
      $rects[$idx].h += 1
      $prev[$key] = $idx
      $merged = $true
    }
  }
  if (-not $merged) {
    $null = $rects.Add([pscustomobject]@{ x = $r.x; y = $r.y; w = $r.w; h = 1; color = $r.color })
    $prev[$key] = $rects.Count - 1
  }
}

# bounds assertions
foreach ($rc in $rects) {
  if ($rc.x -lt 0 -or $rc.y -lt 0 -or ($rc.x + $rc.w) -gt $W -or ($rc.y + $rc.h) -gt $H) {
    throw "internal error: rect out of bounds x=$($rc.x) y=$($rc.y) w=$($rc.w) h=$($rc.h)"
  }
}

$sb = New-Object System.Text.StringBuilder
$null = $sb.AppendLine("<svg xmlns=`"http://www.w3.org/2000/svg`" width=`"$W`" height=`"$H`" viewBox=`"0 0 $W $H`" shape-rendering=`"crispEdges`">")
$null = $sb.AppendLine("<!-- source: sample.xpm -->")
foreach ($rc in $rects) {
  $null = $sb.AppendLine("<rect x=`"$($rc.x)`" y=`"$($rc.y)`" width=`"$($rc.w)`" height=`"$($rc.h)`" fill=`"$($rc.color)`"/>")
}
$null = $sb.AppendLine('</svg>')

$dir = Split-Path -Parent $out
if (-not (Test-Path -LiteralPath $dir)) { $null = New-Item -ItemType Directory -Path $dir -Force }
[System.IO.File]::WriteAllText($out, $sb.ToString(), (New-Object System.Text.UTF8Encoding($false)))

$colorCount = @($rects | ForEach-Object { $_.color } | Sort-Object -Unique).Count
Write-Output "W=$W H=$H ncolors=$ncolors cpp=$cpp"
Write-Output "runs=$runCount"
Write-Output "rects=$($rects.Count)"
Write-Output "colors=$colorCount"
Write-Output "bytes=$((Get-Item -LiteralPath $out).Length)"
Write-Output 'bounds=ok'
exit 0
