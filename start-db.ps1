# Script para iniciar SurrealDB
# Ejecuta este archivo para levantar la base de datos

Write-Host "🚀 Iniciando SurrealDB..." -ForegroundColor Green
Write-Host "📊 Interfaz web: http://localhost:8000" -ForegroundColor Cyan
Write-Host "🔑 Usuario: root | Contraseña: root" -ForegroundColor Yellow
Write-Host ""
Write-Host "Presiona Ctrl+C para detener el servidor" -ForegroundColor Gray
Write-Host ""

.\surreal.exe start --log info --user root --pass root file:uci.db
