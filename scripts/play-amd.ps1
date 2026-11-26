param(
    [Parameter(Mandatory)][string]$Exe,
    [ValidateSet('Stable', 'Performance', 'Fsr4K')][string]$Profile = 'Stable',
    [string]$FidelityFxDir
)

$ErrorActionPreference = 'Stop'
$options = @{
    Exe = (Resolve-Path -LiteralPath $Exe).Path
    Console = 'series-s'
    More = @{
        VICTORY_INDIRECT_WINDOW = '0'
        VICTORY_TEXTURE_NO_WAIT = '0'
    }
}

switch ($Profile) {
    'Stable' { $options.Safe = $true }
    'Performance' { }
    'Fsr4K' {
        $options.Console = 'series-x'
        $options.Safe = $true
        $options.Fsr4K = 'performance'
        if ($FidelityFxDir) {
            $env:VICTORY_FIDELITYFX_DIR = (Resolve-Path -LiteralPath $FidelityFxDir).Path
        }
        if (-not $env:VICTORY_FIDELITYFX_DIR) {
            throw 'Set -FidelityFxDir to the folder containing the official AMD DX12 upscaler.'
        }
        $dll = Join-Path $env:VICTORY_FIDELITYFX_DIR 'amd_fidelityfx_upscaler_dx12.dll'
        if (-not (Test-Path -LiteralPath $dll -PathType Leaf)) {
            throw "AMD DX12 upscaler not found at $dll"
        }
    }
}

Write-Host "Victory AMD: $Profile ($($options.Console))"
& (Join-Path $PSScriptRoot 'play.ps1') @options
exit $LASTEXITCODE
