param(
  [Parameter(Mandatory=$true)][string]$Manifest,
  [Parameter(Mandatory=$true)][string]$InDir,
  [Parameter(Mandatory=$true)][string]$OutDir
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 2.0

# ---------------- X11 named colours (only the names this batch can contain) ----------------
$X11 = @{
  'black'  = 0x000000; 'white'  = 0xFFFFFF
  'red'    = 0xFF0000; 'green'  = 0x00FF00; 'blue'   = 0x0000FF
  'yellow' = 0xFFFF00; 'cyan'   = 0x00FFFF; 'magenta'= 0xFF00FF
  'gray'   = 0xBEBEBE; 'grey'   = 0xBEBEBE
  'darkgray' = 0xA9A9A9; 'darkgrey' = 0xA9A9A9
  'lightgray'= 0xD3D3D3; 'lightgrey'= 0xD3D3D3
}

# X11 grey ramp: value = round(n*255/100), with the published rgb.txt down-roundings
# at n=50 (127) and n=90 (229); all other published values match round-half-up
# (gray87=222 gray88=224 gray89=227 gray91=232 gray92=235 gray93=237 gray94=240).
function GrayLevel([int]$n) {
  if ($n -eq 50) { return 127 }
  if ($n -eq 90) { return 229 }
  return [int][math]::Round(($n * 255.0) / 100.0, 0, [System.MidpointRounding]::AwayFromZero)
}

function Resolve-Color([string]$val, [string]$src, [int]$lineNo) {
  $v = $val.Trim()
  if ($v.Length -eq 0) { throw "unresolved XPM colour `"`" at line $lineNo - supply an RGB value" }
  if ($v -match '^(?i:none)$') { return @{ rgb = 0x000000; a = 0 } }
  if ($v.StartsWith('#')) {
    $hx = $v.Substring(1)
    if ($hx -notmatch '^[0-9A-Fa-f]+$') { throw "unresolved XPM colour `"$v`" at line $lineNo - supply an RGB value" }
    switch ($hx.Length) {
      3  { $r = [Convert]::ToInt32([string]$hx[0] + $hx[0], 16)
           $g = [Convert]::ToInt32([string]$hx[1] + $hx[1], 16)
           $b = [Convert]::ToInt32([string]$hx[2] + $hx[2], 16) }
      6  { $r = [Convert]::ToInt32($hx.Substring(0,2), 16)
           $g = [Convert]::ToInt32($hx.Substring(2,2), 16)
           $b = [Convert]::ToInt32($hx.Substring(4,2), 16) }
      9  { $r = [int][math]::Round([Convert]::ToInt32($hx.Substring(0,3), 16) * 255.0 / 4095.0)
           $g = [int][math]::Round([Convert]::ToInt32($hx.Substring(3,3), 16) * 255.0 / 4095.0)
           $b = [int][math]::Round([Convert]::ToInt32($hx.Substring(6,3), 16) * 255.0 / 4095.0) }
      12 { $r = [int][math]::Round([Convert]::ToInt32($hx.Substring(0,4), 16) * 255.0 / 65535.0)
           $g = [int][math]::Round([Convert]::ToInt32($hx.Substring(4,4), 16) * 255.0 / 65535.0)
           $b = [int][math]::Round([Convert]::ToInt32($hx.Substring(8,4), 16) * 255.0 / 65535.0) }
      default { throw "unresolved XPM colour `"$v`" at line $lineNo - supply an RGB value" }
    }
    return @{ rgb = (($r -shl 16) -bor ($g -shl 8) -bor $b); a = 255 }
  }
  $key = $v.ToLowerInvariant()
  if ($X11.ContainsKey($key)) { return @{ rgb = $X11[$key]; a = 255 } }
  if ($key -match '^(gray|grey)(\d{1,3})$') {
    $n = [int]$Matches[2]
    if ($n -ge 0 -and $n -le 100) {
      $g = GrayLevel $n
      return @{ rgb = (($g -shl 16) -bor ($g -shl 8) -bor $g); a = 255 }
    }
  }
  throw "unresolved XPM colour `"$v`" at line $lineNo - supply an RGB value"
}

function Convert-Xpm([string]$inPath, [string]$outPath, [string]$srcName) {
  $text  = [System.IO.File]::ReadAllText($inPath)
  $lines = $text -split "`n"
  $q = New-Object System.Collections.Generic.List[string]
  $qLine = New-Object System.Collections.Generic.List[int]
  for ($i = 0; $i -lt $lines.Count; $i++) {
    $t = $lines[$i].Trim()
    if ($t.StartsWith('"')) {
      $fq = $t.IndexOf('"'); $lq = $t.LastIndexOf('"')
      if ($lq -gt $fq) { $q.Add($t.Substring($fq + 1, $lq - $fq - 1)); $qLine.Add($i + 1) }
    }
  }
  if ($q.Count -lt 1) { throw "malformed XPM $inPath - no quoted strings" }

  $hdr = @($q[0].Trim() -split '\s+' | Where-Object { $_ -ne '' })
  if ($hdr.Count -lt 4) { throw "malformed XPM $inPath - bad header `"$($q[0])`"" }
  $W = [int]$hdr[0]; $H = [int]$hdr[1]; $NC = [int]$hdr[2]; $CPP = [int]$hdr[3]
  if ($q.Count -lt (1 + $NC + $H)) { throw "malformed XPM $inPath - declared pixel lines ($H) exceed file" }

  $palHex = New-Object 'System.Collections.Generic.List[string]'
  $palA   = New-Object 'System.Collections.Generic.List[int]'
  $keyIdx = New-Object 'System.Collections.Generic.Dictionary[string,int]'
  $namesUsed = New-Object 'System.Collections.Generic.SortedSet[string]'
  $dupKeys = New-Object 'System.Collections.Generic.List[string]'

  for ($c = 0; $c -lt $NC; $c++) {
    $cl = $q[1 + $c]
    if ($cl.Length -lt $CPP) { throw "malformed XPM $inPath - colour line $($c+1) shorter than chars-per-pixel" }
    $key  = $cl.Substring(0, $CPP)
    $rest = $cl.Substring($CPP).Trim()
    $tok  = @($rest -split '\s+' | Where-Object { $_ -ne '' })
    $specs = @{}
    $k = 0
    while ($k -lt $tok.Count) {
      if ($tok[$k].Length -eq 1 -and 'cmgs'.Contains($tok[$k]) -and ($k + 1) -lt $tok.Count) {
        $specs[$tok[$k]] = $tok[$k + 1]; $k += 2
      } else { break }
    }
    $val = $null
    foreach ($pref in @('c','g','m','s')) { if ($specs.ContainsKey($pref)) { $val = $specs[$pref]; break } }
    if ($null -eq $val) { throw "malformed XPM $inPath - colour line $($qLine[1+$c]) has no c/m/g/s value" }
    $col = Resolve-Color $val $srcName $qLine[1 + $c]
    [void]$namesUsed.Add($val)
    if ($keyIdx.ContainsKey($key)) { [void]$dupKeys.Add("'$key'") }
    else { $keyIdx[$key] = $palHex.Count }
    $r = ($col.rgb -shr 16) -band 0xFF; $g = ($col.rgb -shr 8) -band 0xFF; $b = $col.rgb -band 0xFF
    $palHex.Add(('{0:X2}{1:X2}{2:X2}' -f $r, $g, $b))
    $palA.Add([int]$col.a)
  }

  # canonical colour groups: palette entries with identical RGBA collapse to one id
  $canonMap = New-Object 'System.Collections.Generic.Dictionary[string,int]'
  $canon = New-Object 'int[]' $palHex.Count
  for ($i = 0; $i -lt $palHex.Count; $i++) {
    $ck = "$($palHex[$i])|$($palA[$i])"
    if ($canonMap.ContainsKey($ck)) { $canon[$i] = $canonMap[$ck] } else { $canonMap[$ck] = $i; $canon[$i] = $i }
  }

  # pixel grid -> palette indices
  $grid = New-Object 'int[][]' $H
  for ($y = 0; $y -lt $H; $y++) {
    $pl = $q[1 + $NC + $y]
    if ($pl.Length -ne ($W * $CPP)) {
      throw "malformed XPM $inPath - pixel line $y has length $($pl.Length), expected $($W * $CPP)"
    }
    $row = New-Object 'int[]' $W
    for ($x = 0; $x -lt $W; $x++) {
      $pk = $pl.Substring($x * $CPP, $CPP)
      if (-not $keyIdx.ContainsKey($pk)) {
        throw "malformed XPM $inPath - pixel line $y uses key '$pk' not in palette"
      }
      $row[$x] = $canon[$keyIdx[$pk]]
    }
    $grid[$y] = $row
  }

  # horizontal runs -> vertical merge
  $runs = 0
  $rects = New-Object 'System.Collections.Generic.List[object]'
  $open = @{}
  for ($y = 0; $y -lt $H; $y++) {
    $row = $grid[$y]
    $newOpen = @{}
    $x = 0
    while ($x -lt $W) {
      $idx = $row[$x]
      $x0 = $x
      while ($x -lt $W -and $row[$x] -eq $idx) { $x++ }
      $rw = $x - $x0
      $runs++
      $colKey = "$($palHex[$idx])|$($palA[$idx])"
      $k2 = "$x0|$rw|$colKey"
      if ($open.ContainsKey($k2) -and ($open[$k2].y + $open[$k2].h -eq $y)) {
        $open[$k2].h = $open[$k2].h + 1
        $newOpen[$k2] = $open[$k2]
      } else {
        $r = [pscustomobject]@{ x = $x0; y = $y; w = $rw; h = 1; idx = $idx }
        $rects.Add($r); $newOpen[$k2] = $r
      }
    }
    $open = $newOpen
  }

  $sorted = $rects | Sort-Object -Property y, x
  $sb = New-Object System.Text.StringBuilder
  [void]$sb.AppendLine("<svg xmlns=`"http://www.w3.org/2000/svg`" version=`"1.2`" baseProfile=`"full`" width=`"$W`" height=`"$H`" viewBox=`"0 0 $W $H`" shape-rendering=`"crispEdges`">")
  [void]$sb.AppendLine("<!-- source: $srcName -->")
  $emitted = 0
  $used = New-Object 'System.Collections.Generic.SortedSet[string]'
  foreach ($r in $sorted) {
    $a = $palA[$r.idx]
    if ($a -eq 0) { continue }
    if (($r.x + $r.w) -gt $W -or ($r.y + $r.h) -gt $H) { throw "internal error: rect out of bounds on $srcName" }
    $fill = '#' + $palHex[$r.idx]
    if ($a -eq 255) {
      [void]$sb.AppendLine("  <rect x=`"$($r.x)`" y=`"$($r.y)`" width=`"$($r.w)`" height=`"$($r.h)`" fill=`"$fill`"/>")
    } else {
      $op = [math]::Round($a / 255.0, 3).ToString('0.###', [System.Globalization.CultureInfo]::InvariantCulture)
      [void]$sb.AppendLine("  <rect x=`"$($r.x)`" y=`"$($r.y)`" width=`"$($r.w)`" height=`"$($r.h)`" fill=`"$fill`" fill-opacity=`"$op`"/>")
    }
    $emitted++
    [void]$used.Add("$fill/$a")
  }
  [void]$sb.AppendLine("</svg>")
  [System.IO.File]::WriteAllText($outPath, $sb.ToString(), (New-Object System.Text.UTF8Encoding($false)))
  $bytes = (Get-Item -LiteralPath $outPath).Length

  $dup = ''
  if ($dupKeys.Count -gt 0) { $dup = ($dupKeys -join ',') }
  $sym = @($namesUsed | Where-Object { -not $_.StartsWith('#') -and $_.ToLowerInvariant() -ne 'none' })
  return [pscustomobject]@{
    W = $W; H = $H; runs = $runs; rects = $emitted; colors = $used.Count
    bytes = $bytes; nc = $NC; cpp = $CPP; dup = $dup
    sym = ($sym -join ',')
  }
}

# ---------------- driver ----------------
$names = Get-Content -LiteralPath $Manifest | Where-Object { $_.Trim() -ne '' }
foreach ($n in $names) {
  $n = $n.Trim()
  $in  = Join-Path $InDir  $n
  $base = [System.IO.Path]::GetFileNameWithoutExtension($n)
  $out = Join-Path $OutDir ($base + '.svg')
  try {
    $s = Convert-Xpm $in $out $n
    Write-Output ("OK|{0}|{1}x{2}|runs={3}|rects={4}|colors={5}|bytes={6}|nc={7}|cpp={8}|sym={9}{10}" -f `
      $n, $s.W, $s.H, $s.runs, $s.rects, $s.colors, $s.bytes, $s.nc, $s.cpp, $s.sym, `
      $(if ($s.dup) { "|DUPKEYS=" + $s.dup } else { '' }))
  } catch {
    if (Test-Path -LiteralPath $out) { Remove-Item -LiteralPath $out -Force }
    Write-Output ("FAIL|{0}|{1}" -f $n, $_.Exception.Message)
  }
}
