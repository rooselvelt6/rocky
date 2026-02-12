# Justfile for OLYMPUS v15
# Comandos útiles para desarrollo

# Variable por defecto
set shell := ["bash", "-cu"]

# Comando por defecto: mostrar ayuda
_default:
    @just --list

# Desarrollo
# =========

# Iniciar servidor en modo desarrollo
dev:
    cargo run --bin olympus-server --features ssr

# Iniciar frontend en modo desarrollo
frontend:
    cargo run --bin frontend --features csr

# Observar cambios y reiniciar automáticamente
watch:
    cargo watch -x "run --bin olympus-server --features ssr"

# Testing
# =======

# Ejecutar todos los tests
test:
    cargo test --all --all-features

# Ejecutar tests unitarios
test-unit:
    cargo test --lib --all-features

# Ejecutar tests de integración
test-integration:
    cargo test --test '*' --all-features

# Ejecutar tests de seguridad
test-security:
    cargo test --test security --all-features

# Ejecutar tests específicos de un actor
test-actor actor:
    cargo test {{actor}} --all-features

# Ejecutar tests con cobertura
test-coverage:
    cargo tarpaulin --all-features --workspace --timeout 300

# Generar reporte HTML de cobertura
coverage-html:
    cargo tarpaulin --all-features --workspace --timeout 300 --out Html

# Calidad de Código
# =================

# Formatear código
fmt:
    cargo fmt

# Verificar formato
fmt-check:
    cargo fmt -- --check

# Ejecutar clippy
lint:
    cargo clippy --all-features -- -D warnings

# Fix automático de issues
fix:
    cargo fix --all-features --allow-dirty

# Verificar todo (formato, clippy, tests)
check: fmt-check lint test
    @echo "✅ All checks passed!"

# Build
# =====

# Build de desarrollo
build:
    cargo build --all-features

# Build de release
build-release:
    cargo build --release --all-features

# Build para producción (optimizado)
build-prod:
    cargo build --release --all-features --bin olympus-server

# Build del frontend (WASM)
build-frontend:
    cargo build --release --features csr --bin frontend

# Docker
# ======

# Construir imagen Docker
docker-build:
    docker build -t olympus:latest .

# Construir imagen Docker sin cache
docker-build-no-cache:
    docker build --no-cache -t olympus:latest .

# Ejecutar con Docker Compose
docker-up:
    docker-compose up -d

# Detener Docker Compose
docker-down:
    docker-compose down

# Ver logs
docker-logs:
    docker-compose logs -f

# Base de Datos
# =============

# Iniciar servicios de base de datos
db-up:
    docker-compose up -d surrealdb valkey

# Detener servicios de base de datos
db-down:
    docker-compose stop surrealdb valkey

# Limpiar datos de base de datos
db-clean:
    docker-compose down -v surrealdb valkey
    docker-compose up -d surrealdb valkey

# Documentación
# =============

# Generar documentación
doc:
    cargo doc --no-deps --all-features

# Generar y abrir documentación
doc-open:
    cargo doc --no-deps --all-features --open

# Benchmarks
# ==========

# Ejecutar benchmarks
bench:
    cargo bench --all-features

# Ejecutar benchmarks específicos
bench-actor actor:
    cargo bench {{actor}} --all-features

# Performance
# ===========

# Perfil de performance (requiere flamegraph)
flamegraph:
    cargo flamegraph --bin olympus-server --features ssr

# Análisis de heap (requiere heaptrack)
heaptrack:
    heaptrack cargo run --bin olympus-server --features ssr

# Limpieza
# ========

# Limpiar builds
clean:
    cargo clean

# Limpiar todo incluyendo caches
clean-all: clean
    rm -rf target/
    rm -rf Cargo.lock
    cargo clean

# CI/CD Local
# ===========

# Simular CI localmente
ci-local: fmt-check lint test-unit
    @echo "✅ CI checks passed locally"

# Preparar para commit
pre-commit: fmt lint test-unit
    @echo "✅ Ready to commit!"

# Release
# =======

# Crear release local
release-local: check build-release
    @echo "✅ Release build complete"
    @ls -lh target/release/olympus-server

# Generar changelog
changelog:
    git cliff --output CHANGELOG.md

# Utilidades
# ==========

# Actualizar dependencias
update:
    cargo update

# Auditar dependencias (seguridad)
audit:
    cargo audit

# Verificar dependencias obsoletas
outdated:
    cargo outdated

# Instalar herramientas de desarrollo necesarias
tools:
    cargo install cargo-watch
    cargo install cargo-tarpaulin
    cargo install cargo-audit
    cargo install cargo-outdated
    cargo install cargo-flamegraph
    @echo "✅ Development tools installed"

# Información del sistema
info:
    @echo "Rust version: $(rustc --version)"
    @echo "Cargo version: $(cargo --version)"
    @echo "Project: OLYMPUS v15"
    @echo "Actors: 20"
    @echo "Lines of code: $(find src -name '*.rs' -exec wc -l {} + | tail -1 | awk '{print $1}')"
