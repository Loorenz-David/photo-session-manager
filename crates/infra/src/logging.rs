use crate::paths::AppPaths;

pub fn init(paths: &AppPaths) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let file = tracing_appender::rolling::daily(paths.root.join("logs"), "psm");
    let (writer, guard) = tracing_appender::non_blocking(file);
    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
    Some(guard)
}
