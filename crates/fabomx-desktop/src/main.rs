// Application de bureau FabOMX : l'interface (`ui/dist`) et son moteur
// WebAssembly dans une fenêtre, le dialogue Ouvrir natif, les fichiers
// récents (10, dans le dossier de configuration de l'application), le
// dialogues Enregistrer (OMX, travail), le titre de la fenêtre, la demande
// avant de fermer un travail modifié, et le menu macOS, en français ou en anglais
// selon la langue de l'interface (sous Windows, une barre de menus dans la
// fenêtre, sans les entrées propres au Mac).
// Les choix du menu sont envoyés à l'interface par l'événement `menu`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri_plugin_dialog::DialogExt;

const RECENT_MAX: usize = 10;

/// Langue des menus, suivie depuis l'interface.
struct Locale(Mutex<String>);

/// Le travail ouvert a des modifications non enregistrées.
struct Dirty(Mutex<bool>);

#[derive(Serialize)]
struct Opened {
    path: String,
    name: String,
    text: String,
}

fn recent_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("recent.json"))
}

fn read_recent(app: &AppHandle) -> Vec<String> {
    recent_path(app)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_recent(app: &AppHandle, list: &[String]) {
    if let Some(p) = recent_path(app) {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, serde_json::to_string(list).unwrap_or_default());
    }
    rebuild_menu(app);
}

fn read(path: &Path) -> Result<Opened, String> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "missing".to_string()
        } else {
            "unreadable".to_string()
        }
    })?;
    Ok(Opened {
        path: path.to_string_lossy().into_owned(),
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        text,
    })
}

/// Dialogue Ouvrir natif, filtré sur SVG et DXF ; `None` si annulé.
#[tauri::command]
async fn open_file(app: AppHandle) -> Result<Option<Opened>, String> {
    let fr = app
        .state::<Locale>()
        .0
        .lock()
        .map(|l| l.as_str() == "fr")
        .unwrap_or(true);
    let picked = app
        .dialog()
        .file()
        .add_filter(
            if fr {
                "Dessins SVG et DXF, travaux FabOMX"
            } else {
                "SVG and DXF drawings, FabOMX jobs"
            },
            &["svg", "dxf", "fabomx", "SVG", "DXF", "FABOMX"],
        )
        .blocking_pick_file();
    match picked.and_then(|f| f.into_path().ok()) {
        Some(p) => read(&p).map(Some),
        None => Ok(None),
    }
}

/// Dialogue Enregistrer natif, filtré sur OMX (`kind` = `omx`) ou sur les
/// travaux FabOMX (`job`), puis écriture de `text` ; rend le chemin, ou
/// `None` si annulé.
#[tauri::command]
async fn save_file(
    app: AppHandle,
    name: String,
    text: String,
    kind: String,
) -> Result<Option<String>, String> {
    let fr = app
        .state::<Locale>()
        .0
        .lock()
        .map(|l| l.as_str() == "fr")
        .unwrap_or(true);
    let picked = app
        .dialog()
        .file()
        .set_file_name(&name)
        .add_filter(
            match (kind.as_str(), fr) {
                ("job", true) => "Travail FabOMX",
                ("job", false) => "FabOMX job",
                (_, true) => "Parcours OMX (ProtoMAX)",
                (_, false) => "OMX toolpath (ProtoMAX)",
            },
            if kind == "job" {
                &["fabomx"]
            } else {
                &["omx", "OMX"]
            },
        )
        .blocking_save_file();
    match picked.and_then(|f| f.into_path().ok()) {
        Some(p) => std::fs::write(&p, text)
            .map(|_| Some(p.to_string_lossy().into_owned()))
            .map_err(|_| "unwritable".to_string()),
        None => Ok(None),
    }
}

/// Écrit `text` dans `path` (Enregistrer un travail déjà nommé).
#[tauri::command]
fn write_file(path: String, text: String) -> Result<(), String> {
    std::fs::write(path, text).map_err(|_| "unwritable".to_string())
}

/// Le travail de l'interface : titre de la fenêtre, et modifications non
/// enregistrées (la fermeture demande alors à l'interface).
#[tauri::command]
fn job_state(app: AppHandle, title: String, dirty: bool) {
    if let Ok(mut d) = app.state::<Dirty>().0.lock() {
        *d = dirty;
    }
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_title(&title);
    }
}

/// Fermer sans plus rien demander (l'interface a déjà demandé).
#[tauri::command]
fn force_close(app: AppHandle) {
    if let Ok(mut d) = app.state::<Dirty>().0.lock() {
        *d = false;
    }
    app.exit(0);
}

fn dirty(app: &AppHandle) -> bool {
    app.state::<Dirty>().0.lock().map(|d| *d).unwrap_or(false)
}

/// Relit un fichier récent. Un fichier disparu sort de la liste (`missing`).
#[tauri::command]
fn open_path(app: AppHandle, path: String) -> Result<Opened, String> {
    let r = read(Path::new(&path));
    if r.as_ref().is_err_and(|e| e == "missing") {
        let list: Vec<String> = read_recent(&app)
            .into_iter()
            .filter(|p| p != &path)
            .collect();
        write_recent(&app, &list);
    }
    r
}

/// Un dessin importé passe en tête des récents.
#[tauri::command]
fn remember(app: AppHandle, path: String) {
    let mut list: Vec<String> = read_recent(&app)
        .into_iter()
        .filter(|p| p != &path)
        .collect();
    list.insert(0, path);
    list.truncate(RECENT_MAX);
    write_recent(&app, &list);
}

#[tauri::command]
fn recent_files(app: AppHandle) -> Vec<String> {
    read_recent(&app)
}

#[tauri::command]
fn clear_recent(app: AppHandle) {
    write_recent(&app, &[]);
}

/// La langue de l'interface change : le menu suit.
#[tauri::command]
fn set_locale(app: AppHandle, locale: State<Locale>, value: String) {
    if let Ok(mut l) = locale.0.lock() {
        *l = value;
    }
    rebuild_menu(&app);
}

fn rebuild_menu(app: &AppHandle) {
    let fr = app
        .state::<Locale>()
        .0
        .lock()
        .map(|l| l.as_str() == "fr")
        .unwrap_or(true);
    match build_menu(app, fr, &read_recent(app)) {
        Ok(menu) => {
            let _ = app.set_menu(menu);
        }
        Err(e) => eprintln!("menu : {e}"),
    }
}

fn build_menu<R: Runtime>(
    app: &AppHandle<R>,
    fr: bool,
    recent: &[String],
) -> tauri::Result<Menu<R>> {
    let l = |f: &'static str, e: &'static str| if fr { f } else { e };
    let item =
        |id: &str, text: &str, accel: Option<&str>| MenuItem::with_id(app, id, text, true, accel);

    let about = AboutMetadata {
        name: Some("FabOMX".into()),
        version: Some(app.package_info().version.to_string()),
        copyright: Some("© 2026 Stéphane Muller — GPL-3.0-or-later".into()),
        ..Default::default()
    };
    // Mac : le menu de l'application (À propos, Masquer, Quitter). Windows :
    // pas de menu d'application ; Quitter va dans Fichier, À propos dans Aide.
    let mac = cfg!(target_os = "macos");
    let about_item = PredefinedMenuItem::about(
        app,
        Some(l("À propos de FabOMX", "About FabOMX")),
        Some(about),
    )?;
    let quit = PredefinedMenuItem::quit(app, Some(l("Quitter FabOMX", "Quit FabOMX")))?;
    let app_menu = Submenu::with_items(
        app,
        "FabOMX",
        true,
        &[
            &about_item,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, Some(l("Masquer FabOMX", "Hide FabOMX")))?,
            &PredefinedMenuItem::hide_others(app, Some(l("Masquer les autres", "Hide Others")))?,
            &PredefinedMenuItem::show_all(app, Some(l("Tout afficher", "Show All")))?,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let recent_menu = Submenu::new(
        app,
        l("Importer un fichier récent", "Import Recent File"),
        true,
    )?;
    for (i, path) in recent.iter().enumerate() {
        let name = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.clone());
        recent_menu.append(&item(&format!("recent:{i}"), &name, None)?)?;
    }
    if !recent.is_empty() {
        recent_menu.append(&PredefinedMenuItem::separator(app)?)?;
    }
    let clear = MenuItem::with_id(
        app,
        "recent:clear",
        l("Effacer la liste", "Clear Menu"),
        !recent.is_empty(),
        None::<&str>,
    )?;
    recent_menu.append(&clear)?;
    let file = Submenu::with_items(
        app,
        l("Fichier", "File"),
        true,
        &[
            &item("new", l("Nouveau travail", "New Job"), Some("CmdOrCtrl+N"))?,
            &item("import", l("Ouvrir…", "Open…"), Some("CmdOrCtrl+O"))?,
            &recent_menu,
            &PredefinedMenuItem::separator(app)?,
            &item("save", l("Enregistrer", "Save"), Some("CmdOrCtrl+S"))?,
            &item(
                "save-as",
                l("Enregistrer sous…", "Save As…"),
                Some("CmdOrCtrl+Shift+S"),
            )?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, Some(l("Fermer la fenêtre", "Close Window")))?,
        ],
    )?;
    if !mac {
        file.append(&PredefinedMenuItem::separator(app)?)?;
        file.append(&quit)?;
    }

    // Annuler et Rétablir passent par l'interface : elle agit sur le dessin,
    // ou sur le texte d'un champ de saisie.
    let edit = Submenu::with_items(
        app,
        l("Édition", "Edit"),
        true,
        &[
            &item("undo", l("Annuler", "Undo"), Some("CmdOrCtrl+Z"))?,
            &item("redo", l("Rétablir", "Redo"), Some("CmdOrCtrl+Shift+Z"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, Some(l("Couper", "Cut")))?,
            &PredefinedMenuItem::copy(app, Some(l("Copier", "Copy")))?,
            &PredefinedMenuItem::paste(app, Some(l("Coller", "Paste")))?,
            // L'interface sélectionne le texte d'un champ, ou les contours (outil Découpe).
            &item(
                "select-all",
                l("Tout sélectionner", "Select All"),
                Some("CmdOrCtrl+A"),
            )?,
        ],
    )?;

    // Sans raccourci dans le menu : F et ? sont des touches de l'interface,
    // qui doivent rester libres dans les champs de saisie.
    let view = Submenu::with_items(
        app,
        l("Présentation", "View"),
        true,
        &[&item(
            "fit",
            l("Recadrer sur le plateau (F)", "Fit the Bed (F)"),
            None,
        )?],
    )?;
    if mac {
        view.append(&PredefinedMenuItem::separator(app)?)?;
        view.append(&PredefinedMenuItem::fullscreen(
            app,
            Some(l("Plein écran", "Enter Full Screen")),
        )?)?;
    }
    let window = Submenu::with_items(
        app,
        l("Fenêtre", "Window"),
        true,
        &[
            &PredefinedMenuItem::minimize(
                app,
                Some(match (mac, fr) {
                    (true, true) => "Placer dans le Dock",
                    (false, true) => "Réduire",
                    _ => "Minimize",
                }),
            )?,
            &PredefinedMenuItem::maximize(app, Some(l("Réduire/agrandir", "Zoom")))?,
        ],
    )?;
    let help = Submenu::with_items(
        app,
        l("Aide", "Help"),
        true,
        &[&item("help", l("Raccourcis (?)", "Shortcuts (?)"), None)?],
    )?;
    if mac {
        Menu::with_items(app, &[&app_menu, &file, &edit, &view, &window, &help])
    } else {
        help.append(&PredefinedMenuItem::separator(app)?)?;
        help.append(&about_item)?;
        Menu::with_items(app, &[&file, &edit, &view, &window, &help])
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Locale(Mutex::new("fr".into())))
        .manage(Dirty(Mutex::new(false)))
        .setup(|app| {
            let handle = app.handle().clone();
            let menu = build_menu(&handle, true, &read_recent(&handle))?;
            app.set_menu(menu)?;
            Ok(())
        })
        .on_menu_event(|app, event| {
            let _ = app.emit("menu", event.id().as_ref());
        })
        // Fermer la fenêtre avec un travail modifié : l'interface demande d'abord.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if dirty(window.app_handle()) {
                    api.prevent_close();
                    let _ = window.app_handle().emit("menu", "close");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            open_file,
            open_path,
            save_file,
            write_file,
            job_state,
            force_close,
            remember,
            recent_files,
            clear_recent,
            set_locale
        ])
        .build(tauri::generate_context!())
        .expect("lancement de FabOMX")
        // Quitter (⌘Q) avec un travail modifié : l'interface demande d'abord.
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() && dirty(app) {
                    api.prevent_exit();
                    let _ = app.emit("menu", "close");
                }
            }
        });
}
