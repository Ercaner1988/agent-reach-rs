<#
.SYNOPSIS
  Kosum dongusu surucusu. Dis halkayi ISLETIR; ic halkayi Hermes doner.

.DESCRIPTION
  Uc hata bicimini yapisal olarak imkansiz kilmak icin var:

    1. Esik kaydirma  -> hakem dosyalari puanlamadan once git'ten geri yuklenir
    2. Hedefe uydurma -> kapilar.ps1 kapi 5, ve tur sonunda deepseek denetimi
    3. Kirmizi commit -> kapilar.ps1 hicbir kapiyi atlamaz

  Olcum kittir (exa 429, ddg 202 verir ve yasaklar geregi asilmaz), bu yuzden
  canli gauntlet tur basina en fazla iki kez kosar. Geri kalan her sey kasetten.

.EXAMPLE
  ./harness/kosum.ps1 -Bilet A
  ./harness/kosum.ps1 -Bilet B -KuruKosu
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateSet('A', 'B', 'C')][string]$Bilet,

    # Hakemin geri yuklenecegi SABIT ref. HEAD degil -- HEAD olsaydi ajan
    # hakemi degistirip commit'leyerek esigi kaydirabilirdi.
    [string]$HakemRef = 'hakem',

    [string]$Model = '',

    # Tur sonu denetci. Isi kusur aramak, o yuzden isi YAPAN modelden BASKA
    # bir model olmali -- kendi isini aklayan denetci denetci degildir.
    #
    # Olculdu ve calisiyor: custom:kervan + kervan/gemini.
    # deepseek-harness (dsh) kurulu ama DEEPSEEK_API_KEY istiyor; bagladiginda
    #   -Denetci dsh
    # de gecerli olur (asagida ayri kol).
    [string]$Denetci = 'custom:kervan',
    [string]$DenetciModel = 'kervan/gemini',

    [switch]$KuruKosu
)

$ErrorActionPreference = 'Stop'
$Depo = Split-Path -Parent $PSScriptRoot
Set-Location $Depo

# Hakem = olcutu tutan dosyalar. Altin kume ve kosucu BURADA DEGIL: ikisi de
# mesru olarak buyur/degisir (yeni vaka iyi bir seydir), ve ikisini de geri
# yuklemek ya buyumeyi engeller ya da -- gecen turda oldugu gibi -- buyumeyle
# birlikte gelen silinmis bir assert'i kutsar. Olcut ayri dosyada, oran
# cinsinden; payda buyudu diye cita dusmez.
$Hakem = @(
    'harness/kabul.json',
    'harness/kapilar.ps1'
)
$env:AGENT_REACH_CASSETTE = Join-Path $PSScriptRoot 'kaset'

function Bolum($m) { Write-Host "`n=== $m ===" -ForegroundColor Magenta }

function Geri-Yukle {
    # Hakemi sabit ref'ten geri yukle. Ajan bunlari degistirmis olabilir;
    # degistirmis olmasi kayda gecer ama puanlamayi etkilemez.
    $kurcalanan = git diff --name-only $HakemRef -- $Hakem
    if ($kurcalanan) {
        Write-Host "  hakem kurcalanmis, geri yukleniyor:" -ForegroundColor Yellow
        $kurcalanan | ForEach-Object { Write-Host "    $_" -ForegroundColor Yellow }
    }
    git checkout $HakemRef -- $Hakem
    if ($LASTEXITCODE -ne 0) { throw "hakem geri yuklenemedi ($HakemRef)" }
}

# ── 0. Hakem ref'i gercekten var mi ───────────────────────────────────────────
git rev-parse --verify --quiet "$HakemRef" > $null
if ($LASTEXITCODE -ne 0) {
    throw "hakem ref'i bulunamadi: $HakemRef  (`-HakemRef <ref>` ile ver)"
}
$turBasi = git rev-parse HEAD

Bolum "Bilet $Bilet · hakem ref: $HakemRef ($(git rev-parse --short $HakemRef)) · tur basi: $($turBasi.Substring(0,7))"
Geri-Yukle

$biletYolu = Join-Path $PSScriptRoot "biletler/bilet_$Bilet.md"
if (-not (Test-Path $biletYolu)) { throw "bilet yok: $biletYolu" }

if ($KuruKosu) {
    Write-Host "`n[kuru kosu] Hermes cagrilmiyor. Kapilar ve gauntlet kosulacak." -ForegroundColor DarkGray
} else {
    # ── 1. Ic halka: Hermes kendi icinde doner, ag'a cikmaz ───────────────────
    Bolum 'Hermes'
    $istem = @"
$(Get-Content $biletYolu -Raw -Encoding UTF8)

--- KOSUM NOTU ---
Ic halkada su komutu istedigin kadar kosabilirsin, bedava ve ~30 saniye:
    pwsh -File harness/kapilar.ps1
Altisi da yesil olmadan teslim etme.

AGENT_REACH_CASSETTE ayarlandi: arama cagrilari kasetten donuyor. Yeni sorgu
bir kez aga cikar ve kasete yazilir. Canli gauntlet'i SEN kosma -- surucu kosar.

Hakem dosyalarina dokunma; puanlamadan once git'ten geri yuklenirler:
$($Hakem -join "`n")
"@
    $hermesArgs = @('-z', $istem, '--worktree')
    if ($Model) { $hermesArgs += @('-m', $Model) }
    & hermes @hermesArgs
    if ($LASTEXITCODE -ne 0) { Write-Host "  Hermes sifir disi cikti: $LASTEXITCODE" -ForegroundColor Yellow }
}

# ── 2. Kapilar ────────────────────────────────────────────────────────────────
Bolum 'Kapilar'
Geri-Yukle
& pwsh -File (Join-Path $PSScriptRoot 'kapilar.ps1')
if ($LASTEXITCODE -ne 0) {
    Write-Host "`nKAPI KIRMIZI. Tur onaya gitmiyor." -ForegroundColor Red
    exit 1
}

# ── 3. Canli gauntlet, en fazla iki kosu ──────────────────────────────────────
Bolum 'Gauntlet (canli, en fazla 2 kosu)'
Geri-Yukle
$gecti = $false
foreach ($deneme in 1..2) {
    Write-Host "  kosu $deneme/2" -ForegroundColor Cyan
    $cikti = cargo test --test search_gauntlet -- --ignored --nocapture 2>&1
    $ozet = $cikti | Select-String 'recall@10|Zero-result|Not measured|must be|Only \d+/'
    $ozet | ForEach-Object { Write-Host "    $_" }
    if ($LASTEXITCODE -eq 0) { $gecti = $true; break }

    # Olculemeyen kosu bir sonuc degildir; ikinci deneme ancak bunun icin.
    if ($cikti -match 'could be measured') {
        Write-Host "    kisitlanmis kosu, 60 sn bekleyip bir kez daha" -ForegroundColor Yellow
        Start-Sleep -Seconds 60
    } else {
        break
    }
}

# ── 4. Deepseek: hirsli denetim ───────────────────────────────────────────────
Bolum 'Deepseek denetimi'
$fark = git diff "$turBasi..HEAD"
if (-not $fark) { $fark = git diff }
$denetimDurumu = 'kosulmadi'
if (-not $fark) {
    $denetimDurumu = 'degisiklik yok'
    Write-Host "  degisiklik yok, denetim atlandi" -ForegroundColor DarkGray
} else {
    $denetimIstemi = @"
Asagidaki fark bir YZ ajaninin turudur. Tek isin kusur aramak; ovgu isteme.
Bulamazsan "temiz" de ve dur. Sirali liste ver, en agir ustte.

Ozellikle su dordunu ara:
1. Hedefe uydurma  -- sinav kumesinden turetilmis sabit, ozel dal, cevap
   anahtarina bakarak yazilmis liste.
2. Kapsam asimi    -- bilette olmayan dosya, kanal, bagimlilik.
3. Yorum-kod celiskisi -- yorumun soyledigi ile kodun yaptigi ayri.
4. Sessizce gevsetilmis sinama -- esik, assert, silinmis vaka.

--- FARK ---
$fark
"@
    $denetimIstemi | Out-File -FilePath (Join-Path $PSScriptRoot 'son-denetim-istemi.txt') -Encoding UTF8
    $denetimCiktisi = Join-Path $PSScriptRoot 'son-denetim.txt'
    if ($Denetci -eq 'dsh') {
        # DeepSeek Harness, tek gorev kipinde. DEEPSEEK_API_KEY gerekir.
        & dsh --profile headless $denetimIstemi 2>&1 | Tee-Object -FilePath $denetimCiktisi
    } else {
        & hermes -z $denetimIstemi --provider $Denetci -m $DenetciModel 2>&1 |
            Tee-Object -FilePath $denetimCiktisi
    }
    if ($LASTEXITCODE -eq 0) {
        $denetimDurumu = "kosuldu ($Denetci/$DenetciModel)"
    } else {
        Write-Host "  '$Denetci' cagrilamadi. Hermes'te tanimli mi? -> hermes fallback list" -ForegroundColor Yellow
        Write-Host "  Istem yazildi: harness/son-denetim-istemi.txt" -ForegroundColor Yellow
        Write-Host "  Turu onaylamadan once bu istemi bir modele elle ver." -ForegroundColor Yellow
    }
}

# ── 5. Insan kapisi ───────────────────────────────────────────────────────────
Bolum 'Tur bitti'
Write-Host "  gauntlet : $(if ($gecti) { 'YESIL' } else { 'KIRMIZI' })" -ForegroundColor $(if ($gecti) { 'Green' } else { 'Red' })
$stat = git diff --shortstat "$turBasi..HEAD"
if (-not $stat) { $stat = git diff --shortstat }
Write-Host "  fark     : $(if ($stat) { $stat.Trim() } else { 'yok' })"
$denetimRengi = if ($denetimDurumu -like 'kosuldu*') { 'Green' } else { 'Yellow' }
Write-Host "  denetim  : $denetimDurumu" -ForegroundColor $denetimRengi
if ($denetimDurumu -eq 'kosulmadi') {
    Write-Host "`n  UYARI: fark denetlenmedi. Denetlenmemis tur onaylanmis tur degildir." -ForegroundColor Red
}
Write-Host "`n  Sonraki bilet insan onayindan sonra acilir." -ForegroundColor Cyan
exit $(if ($gecti) { 0 } else { 1 })
