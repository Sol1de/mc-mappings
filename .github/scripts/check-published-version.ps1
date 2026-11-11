$ErrorActionPreference = 'Stop'

$metadata = cargo metadata --no-deps --format-version 1 --locked | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$package = $metadata.packages | Where-Object { $_.name -eq 'mc-mappings' }
if (-not $package) { throw 'mc-mappings package not found in Cargo metadata' }
"version=$($package.version)" >> $env:GITHUB_OUTPUT
# `publish = false` in Cargo.toml: publication is switched off, nothing to publish or tag.
if ($null -ne $package.publish -and @($package.publish).Count -eq 0) {
  Write-Host 'Publication is disabled (publish = false in Cargo.toml); skipping.'
  'publish=false' >> $env:GITHUB_OUTPUT
  'tag=false' >> $env:GITHUB_OUTPUT
  return
}
$uri = "https://crates.io/api/v1/crates/$($package.name)/$($package.version)"
try {
  $response = Invoke-WebRequest -Uri $uri -Headers @{ 'User-Agent' = 'mc-mappings-ci (https://github.com/Sol1de/mc-mappings)' } -UseBasicParsing -TimeoutSec 30
  if ($response.StatusCode -ne 200) { throw "Unexpected crates.io response: $($response.StatusCode)" }
  Write-Host "$($package.name) $($package.version) is already published; skipping."
  'publish=false' >> $env:GITHUB_OUTPUT
} catch {
  if ($_.Exception.Response.StatusCode -ne 404) { throw }
  'publish=true' >> $env:GITHUB_OUTPUT
}
'tag=true' >> $env:GITHUB_OUTPUT
