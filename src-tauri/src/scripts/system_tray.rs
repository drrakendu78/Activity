use tauri::{
    menu::{MenuBuilder, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

use super::poller::PollerState;
use std::sync::{atomic::Ordering, OnceLock, Mutex};

pub struct TrayStrings {
    pub discord_connected: &'static str,
    pub discord_disconnected: &'static str,
    pub no_app: &'static str,
    pub start_service: &'static str,
    pub stop_service: &'static str,
    pub no_music: &'static str,
    pub play_pause: &'static str,
    pub previous: &'static str,
    pub next: &'static str,
    pub music: &'static str,
    pub open: &'static str,
    pub settings: &'static str,
    pub check_update: &'static str,
    pub hide_tray: &'static str,
    pub quit: &'static str,
}

pub fn get_tray_strings(lang: &str) -> TrayStrings {
    match lang {
        "fr" => TrayStrings {
            discord_connected: "Discord : connecté ✓",
            discord_disconnected: "Discord : déconnecté",
            no_app: "Aucune app détectée",
            start_service: "▶ Démarrer le service",
            stop_service: "■ Arrêter le service",
            no_music: "Aucune musique",
            play_pause: "⏯ Lecture / Pause",
            previous: "⏮ Précédent",
            next: "⏭ Suivant",
            music: "Musique",
            open: "Ouvrir Activity",
            settings: "Paramètres",
            check_update: "Vérifier les mises à jour",
            hide_tray: "Cacher l'icône de la barre des tâches",
            quit: "Quitter",
        },
        "de" => TrayStrings {
            discord_connected: "Discord: verbunden ✓",
            discord_disconnected: "Discord: getrennt",
            no_app: "Keine App erkannt",
            start_service: "▶ Dienst starten",
            stop_service: "■ Dienst stoppen",
            no_music: "Keine Musik",
            play_pause: "⏯ Wiedergabe / Pause",
            previous: "⏮ Zurück",
            next: "⏭ Weiter",
            music: "Musik",
            open: "Activity öffnen",
            settings: "Einstellungen",
            check_update: "Nach Updates suchen",
            hide_tray: "Taskleistensymbol ausblenden",
            quit: "Beenden",
        },
        "es-ES" | "es-419" => TrayStrings {
            discord_connected: "Discord: conectado ✓",
            discord_disconnected: "Discord: desconectado",
            no_app: "Ninguna app detectada",
            start_service: "▶ Iniciar servicio",
            stop_service: "■ Detener servicio",
            no_music: "Sin música",
            play_pause: "⏯ Reproducir / Pausa",
            previous: "⏮ Anterior",
            next: "⏭ Siguiente",
            music: "Música",
            open: "Abrir Activity",
            settings: "Configuración",
            check_update: "Buscar actualizaciones",
            hide_tray: "Ocultar icono de la barra de tareas",
            quit: "Salir",
        },
        "pt-BR" => TrayStrings {
            discord_connected: "Discord: conectado ✓",
            discord_disconnected: "Discord: desconectado",
            no_app: "Nenhum app detectado",
            start_service: "▶ Iniciar serviço",
            stop_service: "■ Parar serviço",
            no_music: "Nenhuma música",
            play_pause: "⏯ Reproduzir / Pausar",
            previous: "⏮ Anterior",
            next: "⏭ Próxima",
            music: "Música",
            open: "Abrir Activity",
            settings: "Configurações",
            check_update: "Verificar atualizações",
            hide_tray: "Ocultar ícone da barra de tarefas",
            quit: "Sair",
        },
        "zh-CN" => TrayStrings {
            discord_connected: "Discord：已连接 ✓",
            discord_disconnected: "Discord：未连接",
            no_app: "未检测到应用",
            start_service: "▶ 启动服务",
            stop_service: "■ 停止服务",
            no_music: "无音乐",
            play_pause: "⏯ 播放 / 暂停",
            previous: "⏮ 上一首",
            next: "⏭ 下一首",
            music: "音乐",
            open: "打开 Activity",
            settings: "设置",
            check_update: "检查更新",
            hide_tray: "隐藏任务栏图标",
            quit: "退出",
        },
        "zh-TW" => TrayStrings {
            discord_connected: "Discord：已連線 ✓",
            discord_disconnected: "Discord：未連線",
            no_app: "未偵測到應用",
            start_service: "▶ 啟動服務",
            stop_service: "■ 停止服務",
            no_music: "無音樂",
            play_pause: "⏯ 播放 / 暫停",
            previous: "⏮ 上一首",
            next: "⏭ 下一首",
            music: "音樂",
            open: "開啟 Activity",
            settings: "設定",
            check_update: "檢查更新",
            hide_tray: "隱藏工作列圖示",
            quit: "結束",
        },
        "ja" => TrayStrings {
            discord_connected: "Discord: 接続中 ✓",
            discord_disconnected: "Discord: 未接続",
            no_app: "アプリ未検出",
            start_service: "▶ サービス開始",
            stop_service: "■ サービス停止",
            no_music: "音楽なし",
            play_pause: "⏯ 再生 / 一時停止",
            previous: "⏮ 前へ",
            next: "⏭ 次へ",
            music: "音楽",
            open: "Activity を開く",
            settings: "設定",
            check_update: "アップデート確認",
            hide_tray: "タスクバーアイコンを非表示",
            quit: "終了",
        },
        "ko" => TrayStrings {
            discord_connected: "Discord: 연결됨 ✓",
            discord_disconnected: "Discord: 연결 끊김",
            no_app: "감지된 앱 없음",
            start_service: "▶ 서비스 시작",
            stop_service: "■ 서비스 중지",
            no_music: "음악 없음",
            play_pause: "⏯ 재생 / 일시정지",
            previous: "⏮ 이전",
            next: "⏭ 다음",
            music: "음악",
            open: "Activity 열기",
            settings: "설정",
            check_update: "업데이트 확인",
            hide_tray: "작업 표시줄 아이콘 숨기기",
            quit: "종료",
        },
        "ru" => TrayStrings {
            discord_connected: "Discord: подключён ✓",
            discord_disconnected: "Discord: отключён",
            no_app: "Приложение не найдено",
            start_service: "▶ Запустить сервис",
            stop_service: "■ Остановить сервис",
            no_music: "Нет музыки",
            play_pause: "⏯ Воспроизведение / Пауза",
            previous: "⏮ Назад",
            next: "⏭ Далее",
            music: "Музыка",
            open: "Открыть Activity",
            settings: "Настройки",
            check_update: "Проверить обновления",
            hide_tray: "Скрыть значок панели задач",
            quit: "Выход",
        },
        "it" => TrayStrings {
            discord_connected: "Discord: connesso ✓",
            discord_disconnected: "Discord: disconnesso",
            no_app: "Nessuna app rilevata",
            start_service: "▶ Avvia servizio",
            stop_service: "■ Ferma servizio",
            no_music: "Nessuna musica",
            play_pause: "⏯ Play / Pausa",
            previous: "⏮ Precedente",
            next: "⏭ Successivo",
            music: "Musica",
            open: "Apri Activity",
            settings: "Impostazioni",
            check_update: "Controlla aggiornamenti",
            hide_tray: "Nascondi icona barra delle applicazioni",
            quit: "Esci",
        },
        "tr" => TrayStrings {
            discord_connected: "Discord: bağlı ✓",
            discord_disconnected: "Discord: bağlı değil",
            no_app: "Uygulama algılanmadı",
            start_service: "▶ Servisi başlat",
            stop_service: "■ Servisi durdur",
            no_music: "Müzik yok",
            play_pause: "⏯ Oynat / Duraklat",
            previous: "⏮ Önceki",
            next: "⏭ Sonraki",
            music: "Müzik",
            open: "Activity'yi aç",
            settings: "Ayarlar",
            check_update: "Güncellemeleri kontrol et",
            hide_tray: "Görev çubuğu simgesini gizle",
            quit: "Çıkış",
        },
        "pl" => TrayStrings {
            discord_connected: "Discord: połączony ✓",
            discord_disconnected: "Discord: rozłączony",
            no_app: "Nie wykryto aplikacji",
            start_service: "▶ Uruchom usługę",
            stop_service: "■ Zatrzymaj usługę",
            no_music: "Brak muzyki",
            play_pause: "⏯ Odtwórz / Pauza",
            previous: "⏮ Poprzedni",
            next: "⏭ Następny",
            music: "Muzyka",
            open: "Otwórz Activity",
            settings: "Ustawienia",
            check_update: "Sprawdź aktualizacje",
            hide_tray: "Ukryj ikonę paska zadań",
            quit: "Wyjdź",
        },
        "nl" => TrayStrings {
            discord_connected: "Discord: verbonden ✓",
            discord_disconnected: "Discord: niet verbonden",
            no_app: "Geen app gedetecteerd",
            start_service: "▶ Service starten",
            stop_service: "■ Service stoppen",
            no_music: "Geen muziek",
            play_pause: "⏯ Afspelen / Pauzeren",
            previous: "⏮ Vorige",
            next: "⏭ Volgende",
            music: "Muziek",
            open: "Activity openen",
            settings: "Instellingen",
            check_update: "Controleren op updates",
            hide_tray: "Taakbalkpictogram verbergen",
            quit: "Afsluiten",
        },
        // English fallback for all other languages
        _ => TrayStrings {
            discord_connected: "Discord: connected ✓",
            discord_disconnected: "Discord: disconnected",
            no_app: "No app detected",
            start_service: "▶ Start service",
            stop_service: "■ Stop service",
            no_music: "No music",
            play_pause: "⏯ Play / Pause",
            previous: "⏮ Previous",
            next: "⏭ Next",
            music: "Music",
            open: "Open Activity",
            settings: "Settings",
            check_update: "Check for updates",
            hide_tray: "Hide taskbar icon",
            quit: "Quit",
        },
    }
}

/// Store tray menu item references for dynamic updates
struct TrayItems {
    status: MenuItem<tauri::Wry>,
    current_app: MenuItem<tauri::Wry>,
    now_playing: MenuItem<tauri::Wry>,
    start_service: MenuItem<tauri::Wry>,
    stop_service: MenuItem<tauri::Wry>,
    music_submenu: Submenu<tauri::Wry>,
    play_pause: MenuItem<tauri::Wry>,
    prev_track: MenuItem<tauri::Wry>,
    next_track: MenuItem<tauri::Wry>,
    show_item: MenuItem<tauri::Wry>,
    settings_item: MenuItem<tauri::Wry>,
    check_update: MenuItem<tauri::Wry>,
    hide_tray: MenuItem<tauri::Wry>,
    quit_item: MenuItem<tauri::Wry>,
}

static TRAY_ITEMS: OnceLock<Mutex<Option<TrayItems>>> = OnceLock::new();
static TRAY_LANG: OnceLock<Mutex<String>> = OnceLock::new();

/// Update tray menu items dynamically from the poller
pub fn update_tray_status(_app: &AppHandle, connected: bool, current_app: Option<&str>, now_playing: Option<&str>) {
    let lang_lock = TRAY_LANG.get_or_init(|| Mutex::new("en".to_string()));
    let lang = lang_lock.lock().map(|g| g.clone()).unwrap_or_else(|_| "en".to_string());
    let t = get_tray_strings(&lang);

    let lock = TRAY_ITEMS.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = lock.lock() {
        if let Some(items) = guard.as_ref() {
            let status_text = if connected { t.discord_connected } else { t.discord_disconnected };
            let _ = items.status.set_text(status_text);
            let _ = items.current_app.set_text(current_app.unwrap_or(t.no_app));
            let _ = items.now_playing.set_text(now_playing.unwrap_or(t.no_music));
        }
    }
}

/// Update all tray texts when language changes
pub fn update_tray_language(lang: &str) {
    // Update stored language
    let lang_lock = TRAY_LANG.get_or_init(|| Mutex::new("en".to_string()));
    if let Ok(mut guard) = lang_lock.lock() {
        *guard = lang.to_string();
    }

    let t = get_tray_strings(lang);
    let lock = TRAY_ITEMS.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = lock.lock() {
        if let Some(items) = guard.as_ref() {
            let _ = items.start_service.set_text(t.start_service);
            let _ = items.stop_service.set_text(t.stop_service);
            let _ = items.music_submenu.set_text(t.music);
            let _ = items.play_pause.set_text(t.play_pause);
            let _ = items.prev_track.set_text(t.previous);
            let _ = items.next_track.set_text(t.next);
            let _ = items.show_item.set_text(t.open);
            let _ = items.settings_item.set_text(t.settings);
            let _ = items.check_update.set_text(t.check_update);
            let _ = items.hide_tray.set_text(t.hide_tray);
            let _ = items.quit_item.set_text(t.quit);
            // Status texts will update on next poller tick
        }
    }
}

/// Update service toggle items
pub fn update_tray_service(running: bool) {
    let lock = TRAY_ITEMS.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = lock.lock() {
        if let Some(items) = guard.as_ref() {
            let _ = items.start_service.set_enabled(!running);
            let _ = items.stop_service.set_enabled(running);
        }
    }
}

pub fn setup_system_tray(app: &AppHandle) -> Result<(), String> {
    let config = super::config::load_config_internal();
    let t = get_tray_strings(&config.language);

    // Store language for dynamic updates
    let lang_lock = TRAY_LANG.get_or_init(|| Mutex::new("en".to_string()));
    if let Ok(mut guard) = lang_lock.lock() {
        *guard = config.language.clone();
    }

    // -- Status (read-only) --
    let status_item = MenuItem::with_id(app, "status", t.discord_disconnected, false, None::<&str>)
        .map_err(|e| e.to_string())?;
    let current_app_item = MenuItem::with_id(app, "current_app", t.no_app, false, None::<&str>)
        .map_err(|e| e.to_string())?;

    // -- Service --
    let start_service = MenuItem::with_id(app, "start_service", t.start_service, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let stop_service = MenuItem::with_id(app, "stop_service", t.stop_service, false, None::<&str>)
        .map_err(|e| e.to_string())?;

    // -- Music controls --
    let now_playing = MenuItem::with_id(app, "now_playing", t.no_music, false, None::<&str>)
        .map_err(|e| e.to_string())?;
    let play_pause = MenuItem::with_id(app, "play_pause", t.play_pause, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let prev_track = MenuItem::with_id(app, "prev_track", t.previous, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let next_track = MenuItem::with_id(app, "next_track", t.next, true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let sep_music1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let music_submenu = Submenu::with_items(
        app,
        t.music,
        true,
        &[&now_playing, &sep_music1, &play_pause, &prev_track, &next_track],
    )
    .map_err(|e| e.to_string())?;

    // -- Shortcuts --
    let show_item = MenuItem::with_id(app, "show", t.open, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings_item = MenuItem::with_id(app, "settings", t.settings, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let check_update = MenuItem::with_id(app, "check_update", t.check_update, true, None::<&str>)
        .map_err(|e| e.to_string())?;

    // -- Hide tray icon --
    let hide_tray = MenuItem::with_id(app, "hide_tray", t.hide_tray, true, None::<&str>)
        .map_err(|e| e.to_string())?;

    // -- Quit --
    let quit_item = MenuItem::with_id(app, "quit", t.quit, true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let sep1 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep2 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep3 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep4 = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

    let menu = MenuBuilder::new(app)
        .items(&[
            &status_item,
            &current_app_item,
            &sep1,
            &start_service,
            &stop_service,
            &sep2,
            &music_submenu,
            &sep3,
            &show_item,
            &settings_item,
            &check_update,
            &sep4,
            &hide_tray,
            &quit_item,
        ])
        .build()
        .map_err(|e| e.to_string())?;

    let default_icon = app
        .default_window_icon()
        .ok_or_else(|| "Could not get default window icon".to_string())?;

    let _tray = TrayIconBuilder::with_id("main_tray")
        .icon(default_icon.clone())
        .menu(&menu)
        .tooltip("Activity")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.unminimize();
                }
            }
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("navigate", "settings");
                }
            }
            "check_update" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.emit("check-update", ());
                }
            }
            "start_service" => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let poller = app_handle.state::<PollerState>();
                    if !poller.running.load(Ordering::SeqCst) {
                        let rpc = app_handle.state::<super::discord_rpc::DiscordRpcState>();
                        let _ = super::poller::start_poller(poller, rpc, app_handle.clone()).await;
                    }
                    update_tray_service(true);
                    // Notify frontend
                    let _ = app_handle.emit("service-changed", true);
                });
            }
            "stop_service" => {
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app_handle.state::<PollerState>();
                    let _ = super::poller::stop_poller(state).await;
                    update_tray_service(false);
                    // Notify frontend
                    let _ = app_handle.emit("service-changed", false);
                });
            }
            "play_pause" => {
                tauri::async_runtime::spawn(async { let _ = super::media_session::media_play_pause().await; });
            }
            "next_track" => {
                tauri::async_runtime::spawn(async { let _ = super::media_session::media_next().await; });
            }
            "prev_track" => {
                tauri::async_runtime::spawn(async { let _ = super::media_session::media_previous().await; });
            }
            "hide_tray" => {
                // Toggle hide_tray_icon in config and restart
                let mut config = super::config::load_config_internal();
                config.hide_tray_icon = !config.hide_tray_icon;
                let _ = super::config::save_config_internal(&config);
                app.restart();
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.unminimize();
                }
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    // Store references for dynamic updates
    let lock = TRAY_ITEMS.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = lock.lock() {
        *guard = Some(TrayItems {
            status: status_item,
            current_app: current_app_item,
            now_playing,
            start_service,
            stop_service,
            music_submenu,
            play_pause,
            prev_track,
            next_track,
            show_item,
            settings_item: settings_item,
            check_update,
            hide_tray,
            quit_item,
        });
    }

    Ok(())
}
