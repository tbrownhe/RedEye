# Regenerate original RedEye tray assets using Windows' built-in drawing APIs.
# This development tool has no application/runtime dependency.
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$assetDirectory = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\assets'))
[void][IO.Directory]::CreateDirectory($assetDirectory)
$sizes = @(16, 20, 24, 32, 48, 64, 128, 256)

function New-LensBitmap([int]$Size, [bool]$Active) {
    # Draw every size separately at 4x, then reduce it once for smooth edges.
    $edge = $Size * 4
    $large = [Drawing.Bitmap]::new($edge, $edge, [Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [Drawing.Graphics]::FromImage($large)
    $graphics.SmoothingMode = [Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.Clear([Drawing.Color]::Transparent)
    $brush = $null
    $path = $null
    $result = $null
    try {
        # Graphite housing, a readable cool rim, and a recessed inner lip.
        foreach ($circle in @(
            @{ Inset = 0.035; Diameter = 0.93; Color = '#76818f' },
            @{ Inset = 0.085; Diameter = 0.83; Color = '#19212b' },
            @{ Inset = 0.15; Diameter = 0.70; Color = '#080d14' }
        )) {
            $brush = [Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml($circle.Color))
            $graphics.FillEllipse($brush, [single]($edge * $circle.Inset), [single]($edge * $circle.Inset),
                [single]($edge * $circle.Diameter), [single]($edge * $circle.Diameter))
            $brush.Dispose()
            $brush = $null
        }

        if ($Active) {
            # A broad red lens with a small warm center is visible at 16px.
            # These are geometric gradients, not sampled film/reference pixels.
            $path = [Drawing.Drawing2D.GraphicsPath]::new()
            $path.AddEllipse([single]($edge * 0.195), [single]($edge * 0.195),
                [single]($edge * 0.61), [single]($edge * 0.61))
            $brush = [Drawing.Drawing2D.PathGradientBrush]::new($path)
            $brush.CenterPoint = [Drawing.PointF]::new([single]($edge * 0.48), [single]($edge * 0.47))
            $brush.CenterColor = [Drawing.ColorTranslator]::FromHtml('#ff9365')
            $brush.SurroundColors = @([Drawing.ColorTranslator]::FromHtml('#8c1026'))
            $blend = [Drawing.Drawing2D.ColorBlend]::new(4)
            $blend.Positions = [single[]]@(0.0, 0.43, 0.84, 1.0)
            $blend.Colors = [Drawing.Color[]]@(
                [Drawing.ColorTranslator]::FromHtml('#8c1026'),
                [Drawing.ColorTranslator]::FromHtml('#e32637'),
                [Drawing.ColorTranslator]::FromHtml('#ff5146'),
                [Drawing.ColorTranslator]::FromHtml('#ff9365')
            )
            $brush.InterpolationColors = $blend
            $graphics.FillPath($brush, $path)
            $brush.Dispose()
            $brush = $null
            $path.Dispose()
            $path = $null
            # A compact luminous core distinguishes active by lightness/shape.
            $brush = [Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#ffd0a4'))
            $graphics.FillEllipse($brush, [single]($edge * 0.425), [single]($edge * 0.415),
                [single]($edge * 0.11), [single]($edge * 0.11))
        } else {
            # Smaller matte, desaturated lens: no glow and no luminous core.
            $brush = [Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#59363f'))
            $graphics.FillEllipse($brush, [single]($edge * 0.245), [single]($edge * 0.245),
                [single]($edge * 0.51), [single]($edge * 0.51))
            $brush.Dispose()
            $brush = [Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#38252e'))
            $graphics.FillEllipse($brush, [single]($edge * 0.33), [single]($edge * 0.33),
                [single]($edge * 0.34), [single]($edge * 0.34))
        }

        $result = [Drawing.Bitmap]::new($Size, $Size, [Drawing.Imaging.PixelFormat]::Format32bppArgb)
        $reducer = [Drawing.Graphics]::FromImage($result)
        try {
            $reducer.CompositingMode = [Drawing.Drawing2D.CompositingMode]::SourceCopy
            $reducer.InterpolationMode = [Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
            $reducer.PixelOffsetMode = [Drawing.Drawing2D.PixelOffsetMode]::HighQuality
            $reducer.DrawImage($large, [Drawing.Rectangle]::new(0, 0, $Size, $Size))
        } finally {
            $reducer.Dispose()
        }
        return $result
    } catch {
        if ($null -ne $result) { $result.Dispose() }
        throw
    } finally {
        if ($null -ne $brush) { $brush.Dispose() }
        if ($null -ne $path) { $path.Dispose() }
        $graphics.Dispose()
        $large.Dispose()
    }
}

function Write-LensIcon([string]$Path, [bool]$Active) {
    $frames = foreach ($size in $sizes) {
        $bitmap = New-LensBitmap $size $Active
        $stream = [IO.MemoryStream]::new()
        try {
            $bitmap.Save($stream, [Drawing.Imaging.ImageFormat]::Png)
            @{ Size = $size; Bytes = $stream.ToArray() }
        } finally {
            $stream.Dispose()
            $bitmap.Dispose()
        }
    }

    $file = [IO.File]::Create($Path)
    $writer = [IO.BinaryWriter]::new($file)
    try {
        $writer.Write([uint16]0)
        $writer.Write([uint16]1) # ICO, not CUR.
        $writer.Write([uint16]$frames.Count)
        $offset = 6 + 16 * $frames.Count
        foreach ($frame in $frames) {
            $dimension = if ($frame.Size -eq 256) { 0 } else { $frame.Size }
            $writer.Write([byte]$dimension)
            $writer.Write([byte]$dimension)
            $writer.Write([byte]0)
            $writer.Write([byte]0)
            $writer.Write([uint16]1)
            $writer.Write([uint16]32)
            $writer.Write([uint32]$frame.Bytes.Length)
            $writer.Write([uint32]$offset)
            $offset += $frame.Bytes.Length
        }
        foreach ($frame in $frames) { $writer.Write([byte[]]$frame.Bytes) }
    } finally {
        $writer.Dispose()
        $file.Dispose()
    }
}

Write-LensIcon (Join-Path $assetDirectory 'redeye-active.ico') $true
Write-LensIcon (Join-Path $assetDirectory 'redeye-inactive.ico') $false

# Contact sheet preserves actual tray pixel sizes against light/dark surfaces.
$preview = [Drawing.Bitmap]::new(352, 256)
$canvas = [Drawing.Graphics]::FromImage($preview)
$font = [Drawing.Font]::new('Segoe UI', 9)
$dark = [Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#20242c'))
$light = [Drawing.SolidBrush]::new([Drawing.ColorTranslator]::FromHtml('#f3f4f6'))
try {
    $canvas.FillRectangle($light, 0, 0, 352, 128)
    $canvas.FillRectangle($dark, 0, 128, 352, 128)
    foreach ($top in @(0, 128)) {
        $text = if ($top -eq 0) { $dark } else { $light }
        $canvas.DrawString('Actual pixels', $font, $text, 12, ($top + 10))
        $canvas.DrawString('Awake', $font, $text, 12, ($top + 48))
        $canvas.DrawString('Sleep allowed', $font, $text, 12, ($top + 88))
        $column = 120
        foreach ($size in @(16, 20, 24, 32)) {
            $canvas.DrawString([string]$size, $font, $text, $column, ($top + 10))
            foreach ($active in @($true, $false)) {
                $bitmap = New-LensBitmap $size $active
                try {
                    $row = if ($active) { $top + 34 } else { $top + 82 }
                    $canvas.DrawImageUnscaled($bitmap, $column, $row)
                } finally { $bitmap.Dispose() }
            }
            $column += 56
        }
    }
    $preview.Save((Join-Path $assetDirectory 'tray-preview.png'), [Drawing.Imaging.ImageFormat]::Png)
} finally {
    $light.Dispose()
    $dark.Dispose()
    $font.Dispose()
    $canvas.Dispose()
    $preview.Dispose()
}

Get-Item (Join-Path $assetDirectory 'redeye-active.ico'),
    (Join-Path $assetDirectory 'redeye-inactive.ico'),
    (Join-Path $assetDirectory 'tray-preview.png') | Select-Object Name, Length
