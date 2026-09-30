#!/usr/bin/env pwsh

param(
    [Parameter(Mandatory=$true, Position=0)]
    [string]$TemplateDocument,

    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$SourceFiles
)

if (-not (Test-Path $TemplateDocument -PathType Leaf)) {
    Write-Error "Template file not found"
    exit 1
}

if ($SourceFiles.Count -eq 0) {
    Write-Error "Specify source files"
    exit 1
}

$workDir = New-Item -ItemType Directory -Path ([System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), [System.IO.Path]::GetRandomFileName()))

try {
    Copy-Item $TemplateDocument -Destination (Join-Path $workDir "template.typ")

    foreach ($sourceFile in $SourceFiles) {
        if (-not (Test-Path $sourceFile -PathType Leaf)) {
            Write-Warning "Source ${sourceFile} not found"
            continue
        }

        Write-Host "Processing ${sourceFile}"

        $sourceFilename = Split-Path $sourceFile -Leaf
        Copy-Item $sourceFile -Destination (Join-Path $workDir $sourceFilename)

        $templatePath = Join-Path $workDir "template.typ"
        $outputPath = Join-Path $workDir "${sourceFilename}.pdf"

        typst compile `
            --input "data_json=${sourceFilename}" `
            $templatePath `
            $outputPath

        Copy-Item $outputPath -Destination "./"
    }
}
finally {
    Remove-Item -Recurse -Force $workDir
}

