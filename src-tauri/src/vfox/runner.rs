//! vfox CLI 进程封装层
//!
//! 统一注入 `NO_COLOR=1` 以拿到干净文本输出。提供两种 API：
//! - [`run_collect`]：一次性收集 stdout/stderr，适用于读操作（list/current/...）
//! - [`run_stream`]：行级流式回调，适用于长耗时写操作（install/add）

use crate::error::{AppError, AppResult};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// vfox 可执行文件名（依赖 PATH 查找）
const VFOX_BIN: &str = "vfox";

/// 一次性收集模式的输出
pub struct CollectedOutput {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

/// 创建一个预先配置好环境的 vfox 命令
fn vfox_cmd() -> Command {
    let mut cmd = Command::new(VFOX_BIN);
    // 关键：禁用 ANSI 颜色，让输出可解析
    cmd.env("NO_COLOR", "1");
    // 显式禁用交互式行为（vfox 部分子命令可能读 stdin）
    cmd.stdin(Stdio::null());

    // 关键：伪装成"已 hook 的 shell"运行环境
    //
    // vfox 用 `__VFOX_SHELL` 是否非空来判定 IsHookEnv()。GUI 调用时若不设此变量，
    // `vfox use --global` 末尾会走 `shell.Open(os.Getppid())` 分支：
    // 它会读取父进程 cmdline 当成 shell 重新 exec ——
    // 而桌面端的父进程就是 vfox-desktop.exe 本身，于是每次切版本会多弹一个 GUI 实例。
    //
    // 注入任意非空值即可绕过该分支；选用 "pwsh" 是因为它是 Windows 上 vfox 推荐 shell，
    // 语义最贴近实际场景；vfox 在 use/install 路径不会读取此变量的具体值。
    cmd.env("__VFOX_SHELL", "pwsh");

    // Windows: 防止控制台子进程（vfox.exe）弹出黑色 cmd 窗口
    // 0x08000000 = CREATE_NO_WINDOW，详见 Win32 CreateProcess flags
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd
}

/// 检查 vfox 是否可用，返回版本字符串（如 "1.0.8"）
pub async fn detect_version() -> AppResult<String> {
    let out = match vfox_cmd()
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::VfoxNotFound);
        }
        Err(e) => return Err(e.into()),
    };

    if !out.status.success() {
        return Err(AppError::VfoxFailed {
            code: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }

    // 输出形如 "vfox version 1.0.8"
    let text = String::from_utf8_lossy(&out.stdout);
    let version = text
        .split_whitespace()
        .last()
        .ok_or_else(|| AppError::ParseError("空的 --version 输出".into()))?
        .trim()
        .to_string();
    Ok(version)
}

/// 一次性收集模式
///
/// 适用于 list/current/available/info 等读操作。退出码非 0 时返回
/// [`AppError::VfoxFailed`]，调用方拿到的 stdout 一定是成功执行的内容。
pub async fn run_collect(args: &[&str]) -> AppResult<CollectedOutput> {
    let out = vfox_cmd()
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::VfoxNotFound
            } else {
                AppError::Io(e)
            }
        })?;

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let code = out.status.code().unwrap_or(-1);

    if !out.status.success() {
        return Err(AppError::VfoxFailed { code, stderr });
    }

    Ok(CollectedOutput {
        stdout,
        stderr,
        code,
    })
}

/// 行级流式回调
///
/// `on_line(stream, line)` 中 `stream` 为 `"stdout"` 或 `"stderr"`。
/// 用于把 install/add/update 等长任务实时推送到前端。
pub async fn run_stream<F>(args: &[&str], mut on_line: F) -> AppResult<i32>
where
    F: FnMut(&'static str, String) + Send + 'static,
{
    let mut child = vfox_cmd()
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::VfoxNotFound
            } else {
                AppError::Io(e)
            }
        })?;

    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");

    // 在当前 task 内并发读 stdout/stderr，按行回调
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(&'static str, String)>();

    let tx1 = tx.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx1.send(("stdout", line));
        }
    });

    let tx2 = tx.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx2.send(("stderr", line));
        }
    });

    drop(tx);

    while let Some((stream, line)) = rx.recv().await {
        on_line(stream, line);
    }

    let status = child.wait().await?;
    Ok(status.code().unwrap_or(-1))
}
