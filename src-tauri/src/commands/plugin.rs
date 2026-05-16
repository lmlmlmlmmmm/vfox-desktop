use crate::error::AppResult;
use crate::vfox::parser;
use crate::vfox::runner;
use crate::vfox::state;
use crate::vfox::types::{AvailablePlugin, PluginInfo, StreamDone, StreamLine};
use tauri::{AppHandle, Emitter};

/// 插件市场列表
///
/// 数据组合：
/// - `vfox available` 提供候选清单 + 官方/第三方标记（official）
/// - 扫 `~/.vfox/plugin/` 拿到真实已装集合，回填 `installed`
///
/// 之所以不直接信任 vfox available 的 ✓/✗，是因为那是"官方维护"标记，
/// 不是"本机已安装"。如果不交叉验证，所有 ✓ 插件都会被错误地标成已安装。
#[tauri::command]
pub async fn list_available_plugins() -> AppResult<Vec<AvailablePlugin>> {
    let out = runner::run_collect(&["available"]).await?;
    let mut plugins = parser::parse_available(&out.stdout);
    let installed = state::installed_plugin_names().await?;
    for p in plugins.iter_mut() {
        p.installed = installed.contains(&p.name);
    }
    // 已装的优先展示
    plugins.sort_by(|a, b| b.installed.cmp(&a.installed).then(a.name.cmp(&b.name)));
    Ok(plugins)
}

/// 单个插件详情
#[tauri::command]
pub async fn plugin_info(name: String) -> AppResult<PluginInfo> {
    let out = runner::run_collect(&["info", &name]).await?;
    parser::parse_info(&out.stdout)
}

/// 流式：添加插件
#[tauri::command]
pub async fn add_plugin(app: AppHandle, job_id: String, name: String) -> AppResult<()> {
    stream_op(app, job_id, &["add", &name]).await
}

/// 流式：移除插件
#[tauri::command]
pub async fn remove_plugin(app: AppHandle, job_id: String, name: String) -> AppResult<()> {
    stream_op(app, job_id, &["remove", &name]).await
}

/// 流式：更新单个插件
#[tauri::command]
pub async fn update_plugin(app: AppHandle, job_id: String, name: String) -> AppResult<()> {
    stream_op(app, job_id, &["update", &name]).await
}

/// 流式：更新全部插件
#[tauri::command]
pub async fn update_all_plugins(app: AppHandle, job_id: String) -> AppResult<()> {
    stream_op(app, job_id, &["update", "--all"]).await
}

/// 通用流式包装：转发输出到 Tauri 事件
async fn stream_op(app: AppHandle, job_id: String, args: &[&str]) -> AppResult<()> {
    let app_handle = app.clone();
    let jid = job_id.clone();
    let code = runner::run_stream(args, move |stream, line| {
        let _ = app_handle.emit(
            "vfox://stream-line",
            StreamLine {
                job_id: jid.clone(),
                stream,
                line,
            },
        );
    })
    .await?;

    let _ = app.emit(
        "vfox://stream-done",
        StreamDone {
            job_id,
            success: code == 0,
            code: Some(code),
        },
    );
    Ok(())
}
