use std::path::Path;

fn main() {
    // `tauri::generate_context!` exige l'interface construite (`ui/dist`).
    // `tauri build` la construit d'abord ; pour `cargo check` sur un dépôt
    // neuf, une page d'attente suffit.
    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ui/dist");
    if !dist.join("index.html").exists() {
        std::fs::create_dir_all(&dist).expect("ui/dist");
        std::fs::write(
            dist.join("index.html"),
            "<!doctype html><meta charset=\"utf-8\"><title>FabOMX</title>\
             <p>Interface non construite : lancer <code>npm --prefix ui run build</code>.</p>\n",
        )
        .expect("ui/dist/index.html");
    }
    tauri_build::build()
}
