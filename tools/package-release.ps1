[CmdletBinding()]
param(
    [string]$OutputDirectory = 'dist/release-assets',
    [string]$Version = '',
    [string]$Commit = ''
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$workspace = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$semver = '\A(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*))*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?\z'

function Assert-OutputPath([string]$Path) {
    $full = [IO.Path]::GetFullPath($Path)
    $allowed = @('dist', 'portable-test') | Where-Object {
        $prefix = (Join-Path $workspace $_) + [IO.Path]::DirectorySeparatorChar
        $full.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)
    }
    if (!$allowed) { throw 'Package output must be below the workspace dist/ or portable-test/ directory.' }
    # Reject junctions/symlinks in every existing descendant, including files.
    # No recursive deletion is used anywhere in this script.
    $probe = $full
    while ($probe.Length -gt $workspace.Length) {
        if (Test-Path -LiteralPath $probe) {
            if ((Get-Item -LiteralPath $probe -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
                throw "Package output cannot traverse a reparse point: $probe"
            }
        }
        $probe = [IO.Path]::GetDirectoryName($probe)
    }
    return $full
}

Push-Location -LiteralPath $workspace
try {
    $metadataText = & cargo metadata --no-deps --format-version 1 --locked
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read the locked Cargo package metadata.' }
    $metadata = $metadataText | ConvertFrom-Json
    $packages = @($metadata.packages | Where-Object { $_.name -ceq 'redeye' })
    if ($packages.Count -ne 1) { throw 'Expected exactly one RedEye Cargo package.' }
    $cargoVersion = [string]$packages[0].version
    if (!$Version) { $Version = $cargoVersion }
    if ($Version -cnotmatch $semver -or $Version -cne $cargoVersion) {
        throw 'Package version must be strict SemVer and exactly match Cargo.toml.'
    }
    if (!$Commit) {
        $Commit = & git rev-parse HEAD
        if ($LASTEXITCODE -ne 0) { throw 'Cannot identify the source commit.' }
    }
    if ($Commit -cnotmatch '\A[0-9a-f]{40}\z') { throw 'Source commit must be a full lowercase Git SHA.' }

    if (![IO.Path]::IsPathRooted($OutputDirectory)) { $OutputDirectory = Join-Path $workspace $OutputDirectory }
    $output = Assert-OutputPath $OutputDirectory
    $stage = Assert-OutputPath (Join-Path $workspace ('portable-test/package-staging/' + [Guid]::NewGuid().ToString('N')))
    $source = Join-Path $workspace 'target/release/redeye.exe'
    if (![IO.File]::Exists($source)) { throw 'Build target/release/redeye.exe before packaging.' }
    if ([Diagnostics.FileVersionInfo]::GetVersionInfo($source).ProductVersion -cne $Version) {
        throw 'The executable version does not match Cargo.toml. Rebuild the release executable.'
    }
    foreach ($name in @('RedEye.exe', 'RedEye-windows-x64.zip', 'SHA256SUMS.txt')) {
        [void](Assert-OutputPath (Join-Path $output $name))
    }
    [void][IO.Directory]::CreateDirectory($output)
    [void][IO.Directory]::CreateDirectory($stage)
    $stageExe = Join-Path $stage 'RedEye.exe'
    $stageReadme = Join-Path $stage 'README.txt'
    $stageSums = Join-Path $stage 'SHA256SUMS.txt'
    Copy-Item -LiteralPath $source -Destination $stageExe
    $template = [IO.File]::ReadAllText((Join-Path $workspace 'docs/PORTABLE-README.txt'))
    $readme = $template.Replace('{{VERSION}}', $Version).Replace('{{COMMIT}}', $Commit)
    if ($readme -match '\{\{[A-Z]+\}\}') { throw 'Portable README has an unresolved placeholder.' }
    [IO.File]::WriteAllText($stageReadme, $readme, [Text.UTF8Encoding]::new($false))
    $inside = foreach ($name in @('RedEye.exe', 'README.txt')) {
        $hash = (Get-FileHash -LiteralPath (Join-Path $stage $name) -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $name"
    }
    [IO.File]::WriteAllLines($stageSums, [string[]]$inside, [Text.Encoding]::ASCII)
    $archive = Join-Path $output 'RedEye-windows-x64.zip'
    Compress-Archive -LiteralPath @($stageExe, $stageReadme, $stageSums) -DestinationPath $archive -CompressionLevel Optimal -Force
    Copy-Item -LiteralPath $stageExe -Destination (Join-Path $output 'RedEye.exe') -Force
    $outside = foreach ($name in @('RedEye.exe', 'RedEye-windows-x64.zip')) {
        $hash = (Get-FileHash -LiteralPath (Join-Path $output $name) -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $name"
    }
    [IO.File]::WriteAllLines((Join-Path $output 'SHA256SUMS.txt'), [string[]]$outside, [Text.Encoding]::ASCII)
    Write-Output "Packaged RedEye $Version at $Commit in $output"
} finally {
    Pop-Location
}
