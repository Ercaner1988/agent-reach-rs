<#
.SYNOPSIS
  Bedava kapilar. Iç halkanin tamami: sifir ag, ~30 saniye.

.DESCRIPTION
  Hermes bunlari istedigi kadar kosabilir. Ilk kirmizida durur ve sifir disi
  cikis kodu verir.

  Kapi 5 (hile grep'i) bu turun sebebi: gecen tur altin kumenin sorgu metni
  kaynak koda kopyalanmisti ve hicbir sinama bunu gormedi. Artik goruyor.
#>
[CmdletBinding()]
param(
    [string]$Depo = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = 'Stop'
Set-Location $Depo

$script:Basarisiz = @()

function Kapi {
    param([string]$Ad, [scriptblock]$Is)
    Write-Host "── $Ad" -ForegroundColor Cyan
    & $Is
    if ($LASTEXITCODE -ne 0) {
        Write-Host "   KIRMIZI: $Ad" -ForegroundColor Red
        $script:Basarisiz += $Ad
        exit 1
    }
    Write-Host "   yesil" -ForegroundColor Green
}

Kapi 'derleme'  { cargo build --workspace 2>&1 | Select-Object -Last 3 }
Kapi 'clippy'   { cargo clippy --workspace --all-targets -- -D warnings 2>&1 | Select-Object -Last 3 }
Kapi 'birim'    { cargo test --workspace 2>&1 | Select-String 'test result' }
Kapi 'bicim'    { cargo fmt --check }

# ── Kapi 5: hile grep'i ───────────────────────────────────────────────────────
# Altin kumeden turetilmis metnin kaynak kodda isi yok. Iki sey aranir:
#
#   1. Hedef slug'in tamami  ("cursor/minisqlite")
#   2. Sorgulardan cikarilan IKILI kelime obekleri ("cross platform")
#
# Neden ikili, tekil degil: sorgular "rust", "python", "written" gibi siradan
# kelimeler tasiyor ve bunlar mesru kod listelerinde (LANGUAGES, FUNCTION_WORDS)
# zaten geciyor. Tekil arama kendi dogru kodumuzu suclardi. Ikili obek ise
# gecen turun ihlallerinin HEPSINI yakaliyor: "written in", "cross platform",
# "desktop apps", "devtools protocol", "compatible database", "disk usaage".
Write-Host "── hile grep'i" -ForegroundColor Cyan
$altin = Get-Content 'crates/agent-reach-channels/tests/golden_search.json' -Raw -Encoding UTF8 | ConvertFrom-Json

# Siradan Ingilizce ikililer. Bunlar altin kumede de geciyor ama her yazilim
# deposunda da geciyor; cevap anahtarindan kopyalandiklarina dair hicbir sey
# soylemiyorlar. Liste bu dosyada duruyor ve bu dosya HAKEM dosyasidir --
# puanlamadan once git'ten geri yuklenir, yani ajan buyutemez. Buyutmek
# isteyen insan, ayri bir commit'te ve gerekcesiyle buyutur.
$jenerik = @(
    'web search', 'for web', 'for rust', 'rust http', 'http client',
    'client library', 'search for', 'in rust', 'package manager'
)

$aranan = [System.Collections.Generic.List[string]]::new()
foreach ($vaka in $altin) {
    $aranan.Add($vaka.target.ToLower())
    $kelimeler = ($vaka.query.ToLower() -split '\s+') | Where-Object { $_ }
    for ($i = 0; $i -lt $kelimeler.Count - 1; $i++) {
        $ikili = "$($kelimeler[$i]) $($kelimeler[$i+1])"
        if ($jenerik -notcontains $ikili) { $aranan.Add($ikili) }
    }
}
$aranan = $aranan | Select-Object -Unique

$kaynak = Get-ChildItem -Path 'crates' -Recurse -Filter '*.rs' |
          Where-Object { $_.FullName -notmatch 'tests' }

$ihlal = @()
foreach ($dosya in $kaynak) {
    $metin = (Get-Content $dosya.FullName -Raw -Encoding UTF8).ToLower()
    foreach ($obek in $aranan) {
        if ($metin.Contains($obek)) {
            $ihlal += "$($dosya.Name): `"$obek`""
        }
    }
}

if ($ihlal.Count -gt 0) {
    Write-Host "   KIRMIZI: altin kume metni kaynak kodda" -ForegroundColor Red
    $ihlal | ForEach-Object { Write-Host "     $_" -ForegroundColor Red }
    Write-Host "   Sinavi gecmek ile sinavi silmek ayri seylerdir." -ForegroundColor Yellow
    exit 1
}
Write-Host "   yesil ($($aranan.Count) obek arandi)" -ForegroundColor Green

# ── Kapi 6: esik bekcisi ──────────────────────────────────────────────────────
# Hakem dosyalari surucu tarafindan git'ten geri yuklenir, yani kurcalamak
# puani degistiremez. Ama denenmis olmasi bilinmeli.
Write-Host "── esik bekcisi" -ForegroundColor Cyan
$hakem = @(
    'crates/agent-reach-channels/tests/golden_search.json',
    'crates/agent-reach-channels/tests/search_gauntlet.rs',
    'harness/kapilar.ps1'
)
$kirli = git diff --name-only -- $hakem
if ($kirli) {
    Write-Host "   UYARI: hakem dosyalari degistirilmis:" -ForegroundColor Yellow
    $kirli | ForEach-Object { Write-Host "     $_" -ForegroundColor Yellow }
    Write-Host "   Bunlar puanlamadan once git'ten geri yuklenecek." -ForegroundColor Yellow
} else {
    Write-Host "   yesil" -ForegroundColor Green
}

Write-Host ""
Write-Host "Butun kapilar yesil." -ForegroundColor Green
exit 0
