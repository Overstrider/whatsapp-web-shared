//! Adapters to the outside world: SQLite, SMTP, whatsapp-rust, session store.

pub mod db;
pub mod email;
pub mod session_store;
pub mod wa_client;
