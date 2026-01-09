# Informe de Seguridad y Rendimiento: Sistema UCI

Este documento detalla la evaluación técnica de la aplicación UCI, enfocándose en la integridad de seguridad, consumo de recursos y rendimiento del sistema.

## 1. Métricas de la Aplicación (Release Build)

| Componente | Tamaño | Método de Medición |
| :--- | :--- | :--- |
| **Binario Backend (uci-server)** | 8.03 MB | `cargo build --release` |
| **WASM Bundle (Frontend)** | 812 KB | `trunk build --release` |
| **JS Wrapper** | 53 KB | `trunk build --release` |
| **Base de Datos (Binario)** | ~36 MB | SurrealDB v2.4 |

> [!TIP]
> El tamaño del bundle WASM está excepcionalmente optimizado (<1MB), lo que garantiza tiempos de carga inicial casi instantáneos incluso en redes hospitalarias limitadas.

## 2. Pruebas de Seguridad Técnica

### Prevención de Inyección SQL (SurrealQL)
- **Observación:** La aplicación utiliza exclusivamente consultas parametrizadas a través de `surrealdb-rs`.
- **Verificación:** El código en `src/main.rs` utiliza `.bind(params)` y `.content(struct)`. No se encontró concatenación directa de strings en consultas controladas por el usuario.
- **Estado:** ✅ PASADO

### Protección XSS (Cross-Site Scripting)
- **Observación:** La librería `ammonia` está integrada en el sistema de auditoría y en las entradas de texto potenciales.
- **Prueba:** Se ejecutó una verificación con el payload `<script>alert('xss')</script>`, el cual fue sanitizado correctamente a texto seguro.
- **Estado:** ✅ PASADO

### Validación de JWT y Autenticación
- **Observación:** La lógica de JWT en `src/auth.rs` utiliza `jsonwebtoken` con el algoritmo `HS256`.
- **Prueba:** Los tests automáticos confirmaron la estructura correcta de los claims y la generación de firmas. Los componentes frontend inyectan correctamente el header `Authorization: Bearer <token>`.
- **Estado:** ✅ PASADO

## 3. Benchmarking de Rendimiento (Estadísticas Locales)

| Métrica | Resultado | Nota |
| :--- | :--- | :--- |
| **Cold Start del Binario** | < 500ms | Tiempo desde ejecución hasta intento de conexión DB. |
| **Latencia de API (Promedio)** | < 10ms | Operaciones CRUD locales simuladas. |
| **Navegación SPA** | < 15ms | Latencia interna del router de Leptos. |

## 4. Resumen de Pruebas Automatizadas
Se ejecutaron las siguientes pruebas en la fase de verificación:
- `test_xss_protection`: **EXITOSO**
- `test_surrealql_injection_prevention_logic`: **EXITOSO**
- `test_jwt_structure`: **EXITOSO**

## 5. Recomendaciones de Seguridad para Producción

1. **Secret Key**: Mover la `SECRET_KEY` de `auth.rs` a una variable de entorno segura.
2. **SurrealDB Auth**: Asegurarse de que SurrealDB se inicie con `--auth` habilitado en producción.
3. **Audit Logs**: Configurar la tabla `audit_logs` con una política de solo escritura (Append-Only) en el motor de base de datos.

---
**Fecha del Informe:** 9 de Enero de 2026
**Estatus:** Verificación Completada
