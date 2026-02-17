// src/system/runner.rs
// OLYMPUS v15 - Actor Runner
// Motor de ejecución genérico para actores del Olimpo (OTP-like)

#![allow(dead_code)]

use tokio::sync::mpsc;
use tracing::{info, error};

use crate::traits::OlympianActor;
use crate::traits::message::ActorMessage;
use crate::actors::GodName;

/// Ejecutor de un actor individual
/// Mantiene el ciclo de vida, procesa mensajes y maneja errores
pub struct ActorRunner {
    actor: Box<dyn OlympianActor>,
    inbox: mpsc::Receiver<ActorMessage>,
    notify_exit: Option<mpsc::Sender<(GodName, String)>>, // Para notificar muerte a Erinyes/Zeus
}

impl ActorRunner {
    pub fn new(
        actor: Box<dyn OlympianActor>, 
        inbox: mpsc::Receiver<ActorMessage>,
    ) -> Self {
        Self {
            actor,
            inbox,
            notify_exit: None,
        }
    }

    pub fn with_notifier(mut self, notifier: mpsc::Sender<(GodName, String)>) -> Self {
        self.notify_exit = Some(notifier);
        self
    }

    /// Inicia el loop del actor (consume el hilo actual/task)
    pub async fn run(mut self) {
        let name = self.actor.name();
        info!("🌟 [{:?}] ActorRunner iniciado", name);

        // 1. Inicialización
        if let Err(e) = self.actor.initialize().await {
            error!("🚨 [{:?}] Fallo al inicializar: {}", name, e);
            self.notify_death(format!("Initialization failed: {}", e)).await;
            return;
        }

        info!("✨ [{:?}] Actor inicializado y listo", name);

        // 2. Loop principal
        loop {
            // TODO: Implementar select! para heartbeats o señales de control externas si es necesario
            // Por ahora, usamos recv puro.
            match self.inbox.recv().await {
                Some(msg) => {
                    let msg_id = msg.id.clone();
                    // debug!("📨 [{:?}] Recibido mensaje: {}", name, msg_id);

                    // Procesar mensaje protegindolo de pánicos
                    // Nota: CatchUnwind en async es complicado, asumimos que handle_message no paniquea catastróficamente
                    // o que el Runtime de Tokio maneja el panic del task.
                    
                    let result = self.actor.handle_message(msg).await;
                    
                    match result {
                        Ok(_response) => {
                            // Si la respuesta requiere envío, se manejaría aquí o el actor ya lo hizo
                            // Por ahora solo logueamos errores de lógica interna
                        }
                        Err(e) => {
                            error!("⚠️ [{:?}] Error procesando mensaje {}: {}", name, msg_id, e);
                            // No matamos al actor por un error de mensaje, a menos que sea crítico
                        }
                    }
                }
                None => {
                    info!("🛑 [{:?}] Canal cerrado. Iniciando shutdown.", name);
                    break;
                }
            }
        }

        // 3. Shutdown
        if let Err(e) = self.actor.shutdown().await {
            error!("💀 [{:?}] Error durante shutdown: {}", name, e);
        } else {
            info!("💤 [{:?}] Actor detenido correctamente", name);
        }
    }

    async fn notify_death(&self, reason: String) {
        if let Some(tx) = &self.notify_exit {
            let _ = tx.send((self.actor.name(), reason)).await;
        }
    }
}
