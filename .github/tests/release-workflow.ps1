# Regression tests of the crates.io publication: no token, no network (HTTP and cargo are mocked).
$ErrorActionPreference = 'Stop'
$workflow = Get-Content "$PSScriptRoot/../workflows/ci.yml" -Raw
$publishJob = [regex]::Match($workflow, '(?ms)^  publish:\r?\n(?<job>.*)\z').Groups['job'].Value
if (-not $publishJob) { throw 'Publication job not found' }
if ($publishJob -notmatch '(?m)^    needs: check\r?$') { throw 'Publication must wait for validation' }
if ($publishJob -notmatch '(?m)^      queue: max\r?$' -or $publishJob -notmatch '(?m)^      cancel-in-progress: false\r?$') {
    throw 'Publication must retain queued and running jobs'
}
if ($publishJob -notmatch "(?m)^        if: steps\.version\.outputs\.publish == 'true'\r?$") { throw 'Publish step must follow the version check' }
if ($publishJob -notmatch "(?m)^        if: steps\.version\.outputs\.tag == 'true'\r?$") { throw 'Tag step must follow the publication switch' }

$condition = [regex]::Match($publishJob, '(?m)^    if: (.+)').Groups[1].Value.Trim()
if (-not $condition) { throw 'Publication gate not found' }
$condition = $condition.Replace('github.event_name', '$eventName').Replace('github.ref', '$eventRef').Replace('github.repository', '$repo').Replace(' == ', ' -eq ').Replace(' && ', ' -and ')
$gate = [scriptblock]::Create($condition)
foreach ($eventName in @('push', 'pull_request', 'workflow_dispatch')) {
    foreach ($eventRef in @('refs/heads/main', 'refs/heads/feature', 'refs/tags/v0.1.0')) {
        foreach ($repo in @('Sol1de/mc-mappings', 'fork/mc-mappings')) {
            $expected = $eventName -eq 'push' -and $eventRef -eq 'refs/heads/main' -and $repo -eq 'Sol1de/mc-mappings'
            if ((& $gate) -ne $expected) { throw "Incorrect publication gate: $eventName $eventRef $repo" }
        }
    }
}
Write-Host 'PASS: 18 publication gate scenarios'

foreach ($scriptName in @('check-published-version', 'create-release-tag')) {
    if (-not $publishJob.Contains("run: ./.github/scripts/$scriptName.ps1")) { throw "Release script not called: $scriptName" }
}

class HttpFailure : System.Exception {
    [object] $Response
    HttpFailure([int] $status) { $this.Response = [pscustomobject]@{ StatusCode = $status } }
}

function Invoke-RestMethod {
    param($Method, $Uri, $Headers, $TimeoutSec, $ContentType, $Body)
    if ($Method -eq 'Post') {
        if ($Uri -ne 'https://api.github.com/repos/Sol1de/mc-mappings/git/refs') { throw "Unexpected write: $Uri" }
        $releaseTest.Created += $Body | ConvertFrom-Json
        return
    }
    if ($Method -and $Method -ne 'Get') { throw "Unexpected HTTP method: $Method" }
    if ($Uri -ne 'https://api.github.com/repos/Sol1de/mc-mappings/git/ref/tags/v0.1.0') { throw "Unexpected lookup: $Uri" }
    if ($releaseTest.TagExists) { return @{ object = @{ sha = 'c' * 40 } } }
    throw [HttpFailure]::new(404)
}

function Invoke-WebRequest {
    param($Uri, $Headers, $OutFile, $TimeoutSec, [switch] $UseBasicParsing)
    if ($Uri -eq 'https://crates.io/api/v1/crates/mc-mappings/0.1.0') {
        $releaseTest.Lookups++
        if ($releaseTest.RegistryStatus -ne 200) { throw [HttpFailure]::new($releaseTest.RegistryStatus) }
        return @{ StatusCode = 200 }
    }
    if ($Uri -ne 'https://crates.io/api/v1/crates/mc-mappings/0.1.0/download') { throw "Unexpected download: $Uri" }
    $releaseTest.Downloads++
    Copy-Item -LiteralPath $releaseTest.Archive -Destination $OutFile
}

function cargo {
    $global:LASTEXITCODE = 0
    if ($args[0] -ne 'metadata') { throw "Unexpected cargo call: $args" }
    return '{"packages":[{"name":"mc-mappings","version":"0.1.0","publish":' + $releaseTest.Publish + '}]}'
}

$fixtureDir = Join-Path ([System.IO.Path]::GetTempPath()) ('mc-mappings-release-test-' + [guid]::NewGuid().ToString('N'))
$releaseTest = @{}
$savedEnv = @{}
foreach ($name in @('GITHUB_API_URL', 'GITHUB_REPOSITORY', 'GITHUB_SHA', 'CRATE_VERSION', 'GH_TOKEN', 'PUBLISH_REQUIRED', 'GITHUB_OUTPUT')) {
    $savedEnv[$name] = [Environment]::GetEnvironmentVariable($name)
}
try {
    $null = New-Item -ItemType Directory -Path "$fixtureDir/mc-mappings-0.1.0" -Force
    $fixtureArchive = Join-Path $fixtureDir 'fixture.crate'
    $releaseTest.Archive = $fixtureArchive
    $env:GITHUB_OUTPUT = Join-Path $fixtureDir 'outputs.txt'

    # publish = false: no registry lookup, nothing published or tagged.
    $releaseTest.Publish = '[]'
    $releaseTest.Lookups = 0
    [System.IO.File]::WriteAllText($env:GITHUB_OUTPUT, '')
    & "$PSScriptRoot/../scripts/check-published-version.ps1"
    $outputs = Get-Content $env:GITHUB_OUTPUT
    if ($releaseTest.Lookups -ne 0 -or $outputs -notcontains 'publish=false' -or $outputs -notcontains 'tag=false') { throw 'Disabled publication was not skipped' }
    Write-Host 'PASS: publication disabled'

    $releaseTest.Publish = 'null'
    foreach ($registryStatus in @(200, 404, 403, 429, 500)) {
        $releaseTest.RegistryStatus = $registryStatus
        [System.IO.File]::WriteAllText($env:GITHUB_OUTPUT, '')
        $failed = $false
        try { & "$PSScriptRoot/../scripts/check-published-version.ps1" } catch { $failed = $true }
        $outputs = Get-Content $env:GITHUB_OUTPUT
        if ($registryStatus -in @(200, 404)) {
            $expected = if ($registryStatus -eq 200) { 'publish=false' } else { 'publish=true' }
            if ($failed -or $outputs -notcontains 'version=0.1.0' -or $outputs -notcontains $expected -or $outputs -notcontains 'tag=true') { throw "Incorrect version check: $registryStatus" }
        } elseif (-not $failed -or ($outputs -match '^(publish|tag)=')) { throw "Registry error ignored: $registryStatus" }
        Write-Host "PASS: registry response $registryStatus"
    }

    $vcs = @{ git = @{ sha1 = 'a' * 40 }; path_in_vcs = '' } | ConvertTo-Json
    [System.IO.File]::WriteAllText("$fixtureDir/mc-mappings-0.1.0/.cargo_vcs_info.json", $vcs)
    tar -czf $fixtureArchive -C $fixtureDir mc-mappings-0.1.0
    if ($LASTEXITCODE -ne 0) { throw 'Could not create fixture archive' }

    $env:GITHUB_API_URL = 'https://api.github.com'
    $env:GITHUB_REPOSITORY = 'Sol1de/mc-mappings'
    $env:GITHUB_SHA = 'b' * 40
    $env:CRATE_VERSION = '0.1.0'
    $env:GH_TOKEN = 'test-token'
    foreach ($case in @('new-publication', 'published-version', 'existing-tag')) {
        $releaseTest.Created = @()
        $releaseTest.Downloads = 0
        $tagExists = $case -eq 'existing-tag'
        $releaseTest.TagExists = $tagExists
        $env:PUBLISH_REQUIRED = if ($case -eq 'new-publication') { 'true' } else { 'false' }
        & "$PSScriptRoot/../scripts/create-release-tag.ps1"
        if ($tagExists) {
            if ($releaseTest.Created.Count -ne 0 -or $releaseTest.Downloads -ne 0) { throw 'Existing tag was modified' }
        } else {
            $expectedSha = if ($case -eq 'new-publication') { 'b' * 40 } else { 'a' * 40 }
            $expectedDownloads = if ($case -eq 'new-publication') { 0 } else { 1 }
            if ($releaseTest.Created.Count -ne 1 -or $releaseTest.Created[0].ref -ne 'refs/tags/v0.1.0' -or $releaseTest.Created[0].sha -ne $expectedSha -or $releaseTest.Downloads -ne $expectedDownloads) {
                throw "Incorrect release tag: $case"
            }
        }
        Write-Host "PASS: $case"
    }
} finally {
    foreach ($name in $savedEnv.Keys) { [Environment]::SetEnvironmentVariable($name, $savedEnv[$name]) }
    $resolved = [System.IO.Path]::GetFullPath($fixtureDir)
    $tempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
    if (-not $resolved.StartsWith($tempRoot, [System.StringComparison]::OrdinalIgnoreCase) -or (Split-Path $resolved -Leaf) -notlike 'mc-mappings-release-test-*') {
        throw 'Unsafe fixture cleanup path'
    }
    if (Test-Path -LiteralPath $resolved) { Remove-Item -LiteralPath $resolved -Recurse -Force }
}
