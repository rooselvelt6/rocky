# bin/zeus-start.ps1 - El Orquestador Supremo de UCI Scales (Windows Edition)
Write-Host "⚡ UCI SCALES - SISTEMA DE ARRANQUE ZEUS (WINDOWS) ⚡" -ForegroundColor Yellow
Write-Host "----------------------------------------------------" -ForegroundColor Cyan

# 1. PERCEPCIÓN
$OSVersion = [Environment]::OSVersion.Version
$IsWSL = Test-Path "/etc/os-release" 
Write-Host "🖥️  Entorno: Windows ($OSVersion)" -ForegroundColor Green

# 2. CAPA DE DECISIÓN: Docker Desktop
$dockerAvailable = Get-Command docker -ErrorAction SilentlyContinue
if ($dockerAvailable) {
    Write-Host "🐳 Docker Detectado. Verificando estado..."
    $dockerInfo = docker info 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Docker está listo. Lanzando Modo Robusto..."
        # En Windows a menudo se usa docker-compose directamente
        docker-compose up -d --build
        Write-Host "🚀 Aplicación corriendo en http://localhost:3000" -ForegroundColor Green
        exit
    }
}

Write-Host "⚠️  Docker Desktop no está instalado o no está iniciado." -ForegroundColor Red

# 3. INTERNET Y AUTO-INSTALACIÓN
Write-Host "🌐 Verificando conexión..."
$ping = Test-Connection -ComputerName 1.1.1.1 -Count 1 -Quiet
if ($ping) {
    $choice = Read-Host "❓ ¿Deseas intentar instalar Docker Desktop vía Winget? (s/n)"
    if ($choice -eq "s") {
        Write-Host "📦 Iniciando instalador Winget..." -ForegroundColor Cyan
        winget install Docker.DockerDesktop
        Write-Host "✅ Por favor, reinicia tu computadora y ejecuta este script de nuevo."
        exit
    }
}

# 4. FALLBACK: MODO ZEUS NATIVO
Write-Host "🚀 Iniciando MODO ZEUS NATIVO (Escalamiento Local)..." -ForegroundColor Yellow

# Verificar binario
if (!(Test-Path "target\release\uci-server.exe")) {
    Write-Host "🛠️  Binario no encontrado. Intentando forjar aplicación..."
    if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "❌ Error: No se encontró Rust (cargo). Instala Rust para modo nativo." -ForegroundColor Red
        exit 1
    }
    cargo build --release --features ssr
}

# Configuración de Entorno
$env:DB_MODE = "embedded"
$env:DB_PATH = "rocksdb:uci_data"
$env:RUST_LOG = "info"

Write-Host "✅ ZEUS NATIVO EN MARCHA EN http://localhost:3000" -ForegroundColor Green
.\target\release\uci-server.exe
