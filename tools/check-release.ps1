[CmdletBinding()]
param(
    [string]$Repository = $env:GITHUB_REPOSITORY,
    [string]$EventName = $env:GITHUB_EVENT_NAME,
    [string]$Ref = $env:GITHUB_REF
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$workspace = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$semver = '\A(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*))*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?\z'
if ($Repository -cnotmatch '\A[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+\z') { throw 'Invalid GitHub repository name.' }

Push-Location -LiteralPath $workspace
try {
    $metadataText = & cargo metadata --no-deps --format-version 1 --locked
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read the locked Cargo version.' }
    $packages = @(($metadataText | ConvertFrom-Json).packages | Where-Object { $_.name -ceq 'redeye' })
    if ($packages.Count -ne 1) { throw 'Expected exactly one RedEye Cargo package.' }
    $version = [string]$packages[0].version
    if ($version -cnotmatch $semver) { throw 'Cargo version must be strict SemVer.' }
    $tag = "v$version"
    if ($EventName -ceq 'push') {
        if ($Ref -cne "refs/tags/$tag") { throw "Release tag must exactly match Cargo version: $tag" }
    } elseif ($EventName -ceq 'workflow_dispatch') {
        if ($Ref -cne 'refs/heads/main') { throw 'Manual release preview must run from main.' }
    } else { throw 'Only a version-tag push or manual main preview can enter the release build.' }

    $sha = & git rev-parse HEAD
    if ($LASTEXITCODE -ne 0 -or $sha -cnotmatch '\A[0-9a-f]{40}\z') { throw 'Cannot identify the checked-out source commit.' }
    & git fetch --no-tags origin '+refs/heads/main:refs/remotes/origin/main'
    if ($LASTEXITCODE -ne 0) { throw 'Cannot fetch main to verify release ancestry.' }
    & git merge-base --is-ancestor $sha refs/remotes/origin/main
    if ($LASTEXITCODE -ne 0) { throw 'Release commit must already be part of origin/main.' }

    # Only real runs of this repository's specific CI workflow qualify. A
    # similarly named check/status or a PR/manual run cannot satisfy this gate.
    $endpoint = "repos/$Repository/actions/workflows/ci.yml/runs?branch=main&event=push&head_sha=$sha&per_page=100"
    $response = & gh api --method GET $endpoint --header 'Accept: application/vnd.github+json' --header 'X-GitHub-Api-Version: 2026-03-10'
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read main CI evidence. Check Actions read permission and retry.' }
    $runs = @((($response -join "`n") | ConvertFrom-Json).workflow_runs)
    $passed = @($runs | Where-Object {
        $_.head_sha -ceq $sha -and $_.head_branch -ceq 'main' -and $_.event -ceq 'push' -and
        $_.path -ceq '.github/workflows/ci.yml' -and $_.status -ceq 'completed' -and $_.conclusion -ceq 'success' -and
        $_.head_repository.full_name -ceq $Repository
    })
    if ($passed.Count -eq 0) { throw 'This exact commit has no successful main-push CI run. Wait for Windows CI on main to pass, then rerun this release workflow.' }
    $ciUrl = [string]$passed[0].html_url
    if (!$env:GITHUB_OUTPUT) { throw 'GITHUB_OUTPUT is required for release-gate outputs.' }
    $prerelease = if ($version.Split('+')[0].Contains('-')) { 'true' } else { 'false' }
    @("version=$version", "tag=$tag", "sha=$sha", "ci_url=$ciUrl", "prerelease=$prerelease") |
        Add-Content -LiteralPath $env:GITHUB_OUTPUT -Encoding utf8
    Write-Output "Release gate passed for $tag at $sha using $ciUrl"
} finally {
    Pop-Location
}
