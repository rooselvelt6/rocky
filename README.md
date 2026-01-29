# 🩺 UCI - ICU Medical Scales System
### Infraestructura Crítica de Automatización Clínica para Unidades de Cuidados Intensivos

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![Axum](https://img.shields.io/badge/Axum-0.8-blue)
![Leptos](https://img.shields.io/badge/Leptos-WASM-purple)
![SurrealDB](https://img.shields.io/badge/SurrealDB-v2.1.4-cc00ff)
![Portability](https://img.shields.io/badge/Portability-Universal-green?logo=docker)

---

## 🚀 "Born for Performance, Built for Portability"
**UCI System** es una solución de ingeniería de software de grado industrial diseñada para automatizar el cálculo e interpretación de escalas médicas críticas (Glasgow, APACHE II, SOFA, SAPS II, NEWS2). 

Tras las últimas actualizaciones, el sistema ahora es **Universalmente Portable**, capaz de correr con el mismo rendimiento y estabilidad en un servidor potente, una estación de trabajo Windows, o hardware Edge como **Raspberry Pi** y **Banana Pi**.

---

## ✨ Características que lo hacen Único

### 🏗️ Arquitectura de Estado Sólido
- **Core en Rust**: Garantía total de seguridad de memoria y ausencia de errores en tiempo de ejecución.
- **Frontend WASM**: Una interfaz ultra-fluida construida con **Leptos**, sin la sobrecarga de los frameworks tradicionales de JS.
- **Binarios Estáticos (musl)**: El programa se compila de forma que no depende de las librerías de tu Linux. Funciona en Fedora, Arch, Debian o Alpine por igual.

### 🛡️ Resiliencia de Datos con SurrealDB
- **Conexión Inteligente**: Lógica de reintento integrada que espera a la base de datos si esta tarda en arrancar.
- **Persistencia Robusta**: Uso de volúmenes industriales y motores de almacenamiento de alto rendimiento.

### 🎨 Visualización de Inteligencia Clínica
- **Gráficos de Radar Dinámicos**: Visualiza el estado multi-orgánico de un paciente de un vistazo.
- **Seguridad RBAC y Auditoría**: Control de acceso granular y registro histórico (Audit Logs) de cada acción clínica.

---

## 🛠️ Stack Tecnológico

| Capa | Tecnologías | Ventajas Clínicas |
| :--- | :--- | :--- |
| **Lenguaje** | Rust (Edition 2021) | Cero fallos de segmentación y máxima velocidad. |
| **Backend** | Axum + Tokio | Capacidad para manejar cientos de peticiones simultáneas sin latencia. |
| **Frontend** | Leptos (WebAssembly) | Interfaz instantánea con reactividad de grano fino. |
| **Base de Datos** | SurrealDB | Base de datos multi-modelo con relaciones de grafo ultra-rápidas. |
| **Portabilidad** | Docker + Musl Static | Despliegue en 10 segundos en cualquier sistema operativo. |

---

## 🛡️ Infraestructura Ultra-Robusta (Novedad 2026)

Tras una reingeniería completa, el sistema ahora cuenta con una arquitectura de **"Cero Fallos"**:

1.  **Auto-Corrección**: Si alguno de los servicios (App o DB) falla, Docker lo reinicia automáticamente en segundos.
2.  **Resiliencia de Conexión**: Lógica de reintento inteligente en Rust que garantiza la reconexión con SurrealDB ante cualquier parpadeo de red.
3.  **Health Monitoring Proactivo**: Verificación continua de la salud de cada componente mediante contenedores.
4.  **Persistencia Garantizada**: Uso de volúmenes industriales para que tus datos clínicos nunca se pierdan.

---

## 🌀 Instalación y Operación "Zero Friction"

### 🚀 Inicio Rápido con Auto-Verificación
Para una experiencia premium sin errores, utiliza el nuevo script de inicio robusto:

```bash
# Otorgar permisos
chmod +x start-robust.sh healthcheck.sh

# Iniciar sistema con monitoreo inteligente
./start-robust.sh
```
*Este script construirá las imágenes, iniciará los servicios y esperará a que todo esté funcional antes de darte el acceso.*

### 📊 Diagnóstico en Tiempo Real
¿Quieres saber cómo está tu sistema? Ejecuta nuestra herramienta de diagnóstico:
```bash
./healthcheck.sh
```

---

### Ejecución Nativa
Si prefieres no usar Docker y tienes el entorno de Rust instalado:
```bash
# 1. Iniciar la base de datos (SurrealDB local)
surreal start --user root --pass root file:uci.db

# 2. Iniciar el servidor
cargo run --release --bin uci-server
```

La aplicación estará disponible inmediatamente en `http://localhost:3000`.

---

## 📈 Roadmap y Visión 2026
- [x] **Portabilidad Universal**: Binarios estáticos y soporte ARM/x86.
- [x] **Endpoints de Salud**: Monitoreo automático mediante `/api/health`.
- [ ] **AI Sepsis Prediction**: Integración de modelos de ML nativos en Rust.
- [ ] **HL7 FHIR Integration**: Interoperabilidad con otros sistemas hospitalarios.

---

## 👨‍💻 Autor y Visión
Desarrollado por **rooselvelt6** con el objetivo de democratizar la tecnología de alta precisión en entornos de cuidados críticos, manteniendo la soberanía de los datos médicos y la máxima eficiencia en costos de hardware.

---
> [!IMPORTANT]  
> **Aviso Médico:** Este sistema es una herramienta de apoyo. Todas las decisiones clínicas deben ser validadas por personal médico calificado.
