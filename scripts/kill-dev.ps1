# Kill all dev-related processes (Vite dev server + Tauri app)
# Usage: ./scripts/kill-dev.ps1 [-Force]
#   -Force: Skip confirmation prompt
param(
    [switch]$Force
)

$port = 1420
$toKill = @{}

function Add-Proc([int]$procId) {
    if ($toKill.ContainsKey($procId)) { return }
    try {
        $proc = Get-Process -Id $procId -ErrorAction Stop
        $toKill[$procId] = [PSCustomObject]@{
            Id   = $procId
            Name = $proc.ProcessName
            Path = $proc.Path
        }
    } catch {
        $toKill[$procId] = [PSCustomObject]@{
            Id   = $procId
            Name = "unknown"
            Path = "unknown"
        }
    }
}

# 1. Find processes using port 1420
netstat -ano | Select-String ":$port.*LISTENING" | ForEach-Object {
    Add-Proc ([int]($_ -split '\s+')[-1])
}

# 2. Find all bsr-monitor.exe processes
Get-Process -Name "bsr-monitor" -ErrorAction SilentlyContinue | ForEach-Object {
    Add-Proc $_.Id
}

# 3. Find all node.exe and cargo.exe that are children of the above
$allNodeCargo = Get-Process -Name "node","cargo" -ErrorAction SilentlyContinue
foreach ($p in $allNodeCargo) {
    $parentPid = (Get-CimInstance Win32_Process -Filter "ProcessId = $($p.Id)").ParentProcessId
    # Kill node/cargo if parent is one of the processes we're already killing
    if ($toKill.ContainsKey($parentPid)) {
        Add-Proc $p.Id
    }
}

if ($toKill.Count -gt 0) {
    $procs = $toKill.Values | Sort-Object {
        # Sort: node first, then cargo, then bsr-monitor last
        if ($_.Name -eq 'node') { 0 }
        elseif ($_.Name -eq 'cargo') { 1 }
        else { 2 }
    }

    foreach ($p in $procs) {
        Write-Host "PID $($p.Id) - $($p.Name)"
        if ($p.Path -and $p.Path -ne 'unknown') {
            Write-Host "  Path: $($p.Path)"
        }
    }

    $doKill = $false
    if ($Force) {
        $doKill = $true
    } else {
        $confirm = Read-Host "Kill these $($procs.Count) processes? (y/N)"
        if ($confirm -eq 'y' -or $confirm -eq 'Y') {
            $doKill = $true
        }
    }

    if ($doKill) {
        foreach ($p in $procs) {
            Write-Host "Killing PID $($p.Id) ($($p.Name))..."
            Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
        }
        # Wait for processes to fully release port
        Start-Sleep -Seconds 2

        # Verify: if port still in use, force kill remaining
        $remaining = netstat -ano | Select-String ":$port.*LISTENING"
        if ($remaining) {
            $remainingPids = $remaining | ForEach-Object { [int]($_ -split '\s+')[-1] } | Select-Object -Unique
            foreach ($procId in $remainingPids) {
                Write-Host "Port $port still occupied by PID $procId, force killing..."
                Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue
            }
            Start-Sleep -Seconds 1
        }

        Write-Host "All processes cleared."
    } else {
        Write-Host "Cancelled."
    }
} else {
    Write-Host "No dev processes found."
}
