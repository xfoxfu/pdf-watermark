use crate::settings::Settings;
use crate::{AppResult, DomainError};
use anyhow::{Context, Error};
use axum::extract::State;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::{AppendHeaders, IntoResponse};
use axum::{body::Bytes, extract::Query};
use serde::Deserialize;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tracing::info;

/// Modified from https://docs.rs/tokio/latest/tokio/process/struct.Child.html#method.wait_with_output
pub async fn wait_with_output(
    mut child: tokio::process::Child,
    timeout: Duration,
) -> tokio::io::Result<Option<std::process::Output>> {
    async fn read_to_end<A: tokio::io::AsyncRead + Unpin>(
        io: &mut Option<A>,
    ) -> tokio::io::Result<Vec<u8>> {
        let mut vec = Vec::new();
        if let Some(io) = io.as_mut() {
            io.read_to_end(&mut vec).await?;
        }
        Ok(vec)
    }

    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();

    let stdout_fut = read_to_end(&mut stdout_pipe);
    let stderr_fut = read_to_end(&mut stderr_pipe);

    let Ok(result) = tokio::time::timeout(
        timeout,
        futures::future::try_join3(child.wait(), stdout_fut, stderr_fut),
    )
    .await
    else {
        info!("timed out");
        child.kill().await?;
        return Ok(None);
    };
    let (status, stdout, stderr) = result?;

    // Drop happens after `try_join` due to <https://github.com/tokio-rs/tokio/issues/4309>
    drop(stdout_pipe);
    drop(stderr_pipe);

    Ok(Some(std::process::Output {
        status,
        stdout,
        stderr,
    }))
}

#[derive(Default, Deserialize)]
pub struct MarkQuery {
    text: String,
    font_size: f32,
    padding_w: f32,
    padding_h: f32,
    rot_deg: f32,
}

pub async fn mark(
    State(settings): State<Settings>,
    query: Query<MarkQuery>,
    pdf: Bytes,
) -> AppResult<impl IntoResponse> {
    info!("request received");

    let exe = std::env::current_exe()
        .context("Failed to get current exe path")?
        .parent()
        .ok_or(Error::msg("Current exe path does not have parent dir"))?
        .join("mark_pdf");
    let args = [
        "--text",
        &query.text,
        "--font-size",
        &query.font_size.to_string(),
        "--padding-w",
        &query.padding_w.to_string(),
        "--padding-h",
        &query.padding_h.to_string(),
        "--rot-deg",
        &query.rot_deg.to_string(),
    ];
    info!("executing {:?} {:?}", exe, args);

    let mut child = Command::new(exe)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn mark_pdf child process")?;

    let mut stdin = child.stdin.take().unwrap();
    tokio::spawn(async move {
        stdin
            .write_all(&pdf)
            .await
            .expect("Failed to write to stdin for child process");
        drop(stdin);
    });

    let Some(result) = wait_with_output(
        child,
        Duration::from_secs(settings.utils.mark_pdf_timeout_secs),
    )
    .await
    .context("Failed to get process output")?
    else {
        return Err(DomainError::PdfTimeout.into());
    };

    if result.status.success() {
        Ok((
            AppendHeaders([
                (CONTENT_TYPE, "application/pdf"),
                (CONTENT_DISPOSITION, "inline"),
            ]),
            result.stdout,
        ))
    } else {
        match result.status.code() {
            Some(1) => Err(DomainError::PdfInternalError {
                cause: String::from_utf8(result.stderr)?,
            }
            .into()),
            Some(2) => Err(DomainError::PdfFormatError.into()),
            _ => Err(DomainError::PdfInternalError {
                cause: format!("Unexpected exit code: {:?}", result.status.code()),
            }
            .into()),
        }
    }
}
