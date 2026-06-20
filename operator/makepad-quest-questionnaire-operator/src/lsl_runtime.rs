use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

pub const DEFAULT_RUNTIME_STATE_STREAM_NAME: &str = "peripersonal_runtime_state";
pub const DEFAULT_RUNTIME_STATE_STREAM_TYPE: &str = "peripersonal.runtime.state";
pub const DEFAULT_OPERATOR_COMMAND_STREAM_NAME: &str = "peripersonal_operator_command";
pub const DEFAULT_OPERATOR_COMMAND_STREAM_TYPE: &str = "peripersonal.operator.command";
pub const DEFAULT_OPERATOR_SIGNAL_STREAM_NAME: &str = "peripersonal_operator_signal";
pub const DEFAULT_OPERATOR_SIGNAL_STREAM_TYPE: &str = "peripersonal.operator.signal";
pub const DEFAULT_OPERATOR_COMMAND_ACK_STREAM_NAME: &str = "peripersonal_operator_command_ack";
pub const DEFAULT_OPERATOR_COMMAND_ACK_STREAM_TYPE: &str = "peripersonal.operator.command.ack";
pub const RUNTIME_STATE_LSL_PROTOCOL_VERSION: &str =
    "quest.questionnaire.operator.peripersonal_runtime_state_lsl_recording.v1";
pub const OPERATOR_COMMAND_LSL_PROTOCOL_VERSION: &str =
    "quest.questionnaire.operator.peripersonal_operator_command_lsl.v1";
pub const OPERATOR_SIGNAL_LSL_PROTOCOL_VERSION: &str =
    "quest.questionnaire.operator.peripersonal_operator_signal_lsl.v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeStateLslRecordingOptions {
    pub out: PathBuf,
    pub stream_name: String,
    pub stream_type: String,
    pub source_id_prefix: Option<String>,
    pub resolve_timeout_ms: u64,
    pub pull_timeout_ms: u64,
    pub idle_timeout_ms: u64,
    pub duration_ms: Option<u64>,
    pub max_samples: Option<usize>,
}

impl Default for RuntimeStateLslRecordingOptions {
    fn default() -> Self {
        Self {
            out: PathBuf::from("artifacts/operator-lsl/peripersonal_runtime_state_lsl.csv"),
            stream_name: DEFAULT_RUNTIME_STATE_STREAM_NAME.to_string(),
            stream_type: DEFAULT_RUNTIME_STATE_STREAM_TYPE.to_string(),
            source_id_prefix: None,
            resolve_timeout_ms: 30_000,
            pull_timeout_ms: 500,
            idle_timeout_ms: 60_000,
            duration_ms: None,
            max_samples: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RuntimeStateLslRecordingReport {
    pub protocol_version: String,
    pub accepted: bool,
    pub out: PathBuf,
    pub stream_name: String,
    pub stream_type: String,
    pub stream_source_id: String,
    pub stream_channel_count: usize,
    pub stream_sample_rate_hz: f64,
    pub sample_count: usize,
    pub started_unix_ms: u128,
    pub ended_unix_ms: u128,
    pub lsl_library_detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorCommandLslSendOptions {
    pub command_json: String,
    pub stream_name: String,
    pub stream_type: String,
    pub source_id: String,
    pub warmup_ms: u64,
    pub post_push_hold_ms: u64,
    pub wait_for_ack: bool,
    pub ack_stream_name: String,
    pub ack_stream_type: String,
    pub ack_timeout_ms: u64,
}

impl Default for OperatorCommandLslSendOptions {
    fn default() -> Self {
        Self {
            command_json: String::new(),
            stream_name: DEFAULT_OPERATOR_COMMAND_STREAM_NAME.to_string(),
            stream_type: DEFAULT_OPERATOR_COMMAND_STREAM_TYPE.to_string(),
            source_id: "windows.operator.peripersonal.command".to_string(),
            warmup_ms: 2_500,
            post_push_hold_ms: 500,
            wait_for_ack: false,
            ack_stream_name: DEFAULT_OPERATOR_COMMAND_ACK_STREAM_NAME.to_string(),
            ack_stream_type: DEFAULT_OPERATOR_COMMAND_ACK_STREAM_TYPE.to_string(),
            ack_timeout_ms: 5_000,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct OperatorCommandLslSendReport {
    pub protocol_version: String,
    pub accepted: bool,
    pub stream_name: String,
    pub stream_type: String,
    pub source_id: String,
    pub pushed_unix_ms: u128,
    pub completed_unix_ms: u128,
    pub ack_stream_name: String,
    pub ack_stream_type: String,
    pub ack_received: bool,
    pub ack_payload: Option<String>,
    pub lsl_library_detail: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OperatorSignalLslSendOptions {
    pub values: Vec<f32>,
    pub stream_name: String,
    pub stream_type: String,
    pub source_id: String,
    pub python_helper: bool,
    pub warmup_ms: u64,
    pub repeat_count: u32,
    pub repeat_interval_ms: u64,
    pub post_push_hold_ms: u64,
}

impl Default for OperatorSignalLslSendOptions {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            stream_name: DEFAULT_OPERATOR_SIGNAL_STREAM_NAME.to_string(),
            stream_type: DEFAULT_OPERATOR_SIGNAL_STREAM_TYPE.to_string(),
            source_id: "windows.operator.peripersonal.signal".to_string(),
            python_helper: false,
            warmup_ms: 3_000,
            repeat_count: 30,
            repeat_interval_ms: 250,
            post_push_hold_ms: 2_000,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct OperatorSignalLslSendReport {
    pub protocol_version: String,
    pub accepted: bool,
    pub sender_kind: String,
    pub stream_name: String,
    pub stream_type: String,
    pub source_id: String,
    pub channel_count: usize,
    pub sample_count: u32,
    pub pushed_unix_ms: u128,
    pub completed_unix_ms: u128,
    pub values: Vec<f32>,
    pub lsl_library_detail: String,
}

#[derive(Clone, Debug)]
pub struct RuntimeStateLslSample {
    pub source_lsl_timestamp_seconds: f64,
    pub operator_lsl_local_clock_seconds: f64,
    pub values: Vec<String>,
}

#[derive(Clone, Debug)]
struct ResolvedStringStream {
    name: String,
    stream_type: String,
    source_id: String,
    channel_count: usize,
    sample_rate_hz: f64,
}

pub fn record_runtime_state_lsl_csv(
    options: &RuntimeStateLslRecordingOptions,
    runtime_columns: &[&str],
) -> Result<RuntimeStateLslRecordingReport, String> {
    record_runtime_state_lsl_csv_impl(options, runtime_columns)
}

pub fn send_operator_command_lsl_json(
    options: &OperatorCommandLslSendOptions,
) -> Result<OperatorCommandLslSendReport, String> {
    send_operator_command_lsl_json_impl(options)
}

pub fn send_operator_signal_lsl_floats(
    options: &OperatorSignalLslSendOptions,
) -> Result<OperatorSignalLslSendReport, String> {
    if options.python_helper {
        return send_operator_signal_lsl_floats_with_python(options);
    }

    send_operator_signal_lsl_floats_impl(options)
}

#[cfg(windows)]
fn record_runtime_state_lsl_csv_impl(
    options: &RuntimeStateLslRecordingOptions,
    runtime_columns: &[&str],
) -> Result<RuntimeStateLslRecordingReport, String> {
    windows_lsl::record_runtime_state_lsl_csv(options, runtime_columns)
}

#[cfg(windows)]
fn send_operator_command_lsl_json_impl(
    options: &OperatorCommandLslSendOptions,
) -> Result<OperatorCommandLslSendReport, String> {
    windows_lsl::send_operator_command_lsl_json(options)
}

#[cfg(windows)]
fn send_operator_signal_lsl_floats_impl(
    options: &OperatorSignalLslSendOptions,
) -> Result<OperatorSignalLslSendReport, String> {
    windows_lsl::send_operator_signal_lsl_floats(options)
}

#[cfg(not(windows))]
fn record_runtime_state_lsl_csv_impl(
    _options: &RuntimeStateLslRecordingOptions,
    _runtime_columns: &[&str],
) -> Result<RuntimeStateLslRecordingReport, String> {
    Err(
        "LSL runtime recording is currently implemented for Windows operator builds only."
            .to_string(),
    )
}

#[cfg(not(windows))]
fn send_operator_command_lsl_json_impl(
    _options: &OperatorCommandLslSendOptions,
) -> Result<OperatorCommandLslSendReport, String> {
    Err(
        "LSL operator command sending is currently implemented for Windows operator builds only."
            .to_string(),
    )
}

#[cfg(not(windows))]
fn send_operator_signal_lsl_floats_impl(
    _options: &OperatorSignalLslSendOptions,
) -> Result<OperatorSignalLslSendReport, String> {
    Err(
        "LSL operator signal sending is currently implemented for Windows operator builds only."
            .to_string(),
    )
}

fn write_runtime_state_lsl_header(
    writer: &mut BufWriter<File>,
    runtime_columns: &[&str],
) -> Result<(), String> {
    writer
        .write_all(
            b"operator_received_unix_ms,operator_lsl_local_clock_seconds,source_lsl_timestamp_seconds,lsl_stream_name,lsl_stream_type,lsl_source_id",
        )
        .map_err(|err| err.to_string())?;
    for column in runtime_columns {
        writer.write_all(b",").map_err(|err| err.to_string())?;
        writer
            .write_all(column.as_bytes())
            .map_err(|err| err.to_string())?;
    }
    writer.write_all(b"\n").map_err(|err| err.to_string())
}

fn write_runtime_state_lsl_row(
    writer: &mut BufWriter<File>,
    stream: &ResolvedStringStream,
    sample: &RuntimeStateLslSample,
) -> Result<(), String> {
    write!(
        writer,
        "{},{},{},{},{},{}",
        unix_ms(),
        format_seconds(sample.operator_lsl_local_clock_seconds),
        format_seconds(sample.source_lsl_timestamp_seconds),
        csv(&stream.name),
        csv(&stream.stream_type),
        csv(&stream.source_id)
    )
    .map_err(|err| err.to_string())?;

    for value in &sample.values {
        writer.write_all(b",").map_err(|err| err.to_string())?;
        writer
            .write_all(value.as_bytes())
            .map_err(|err| err.to_string())?;
    }

    writer.write_all(b"\n").map_err(|err| err.to_string())
}

fn prepare_output(path: &PathBuf) -> Result<BufWriter<File>, String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "Could not create LSL recording output directory {}: {err}",
                    parent.display()
                )
            })?;
        }
    }

    let file = File::create(path).map_err(|err| {
        format!(
            "Could not create LSL recording CSV {}: {err}",
            path.display()
        )
    })?;
    Ok(BufWriter::new(file))
}

fn csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn format_seconds(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.9}")
    } else {
        String::new()
    }
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn normalize_text(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.trim().to_string()
    }
}

fn send_operator_signal_lsl_floats_with_python(
    options: &OperatorSignalLslSendOptions,
) -> Result<OperatorSignalLslSendReport, String> {
    if options.values.is_empty() {
        return Err("LSL operator signal requires at least one float value.".to_string());
    }

    let stream_name = normalize_text(&options.stream_name, DEFAULT_OPERATOR_SIGNAL_STREAM_NAME);
    let stream_type = normalize_text(&options.stream_type, DEFAULT_OPERATOR_SIGNAL_STREAM_TYPE);
    let source_id = normalize_text(&options.source_id, "windows.operator.peripersonal.signal");
    let python = std::env::var("VISCEREALITY_LSL_PYTHON").unwrap_or_else(|_| "python".to_string());
    let values_json = serde_json::to_string(&options.values).map_err(|err| err.to_string())?;
    let repeat_count = options.repeat_count.max(1);
    let code = r#"
import json
import sys
import time

import pylsl

stream_name = sys.argv[1]
stream_type = sys.argv[2]
source_id = sys.argv[3]
values = [float(value) for value in json.loads(sys.argv[4])]
warmup_ms = max(0, int(sys.argv[5]))
repeat_count = max(1, int(sys.argv[6]))
repeat_interval_ms = max(0, int(sys.argv[7]))
post_push_hold_ms = max(0, int(sys.argv[8]))

info = pylsl.StreamInfo(
    stream_name,
    stream_type,
    len(values),
    pylsl.IRREGULAR_RATE,
    "float32",
    source_id,
)
outlet = pylsl.StreamOutlet(info)
time.sleep(warmup_ms / 1000.0)

pushed_unix_ms = 0
for index in range(repeat_count):
    outlet.push_sample(values)
    if pushed_unix_ms == 0:
        pushed_unix_ms = int(time.time() * 1000)
    if index + 1 < repeat_count:
        time.sleep(repeat_interval_ms / 1000.0)

time.sleep(post_push_hold_ms / 1000.0)
try:
    library_info = pylsl.library_info()
except Exception as exc:
    library_info = "pylsl library_info unavailable: " + type(exc).__name__

print(json.dumps({
    "pushed_unix_ms": pushed_unix_ms,
    "completed_unix_ms": int(time.time() * 1000),
    "library_info": library_info,
}), flush=True)
"#;

    let output = Command::new(&python)
        .arg("-c")
        .arg(code)
        .arg(&stream_name)
        .arg(&stream_type)
        .arg(&source_id)
        .arg(&values_json)
        .arg(options.warmup_ms.to_string())
        .arg(repeat_count.to_string())
        .arg(options.repeat_interval_ms.to_string())
        .arg(options.post_push_hold_ms.to_string())
        .output()
        .map_err(|err| format!("Could not start Python pylsl helper {python}: {err}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(format!(
            "Python pylsl helper failed with exit code {:?}. stdout: {} stderr: {}",
            output.status.code(),
            stdout.trim(),
            stderr.trim()
        ));
    }

    let json_line = stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .ok_or_else(|| {
            format!(
                "Python pylsl helper did not print a JSON report. stdout: {} stderr: {}",
                stdout.trim(),
                stderr.trim()
            )
        })?;
    let value = serde_json::from_str::<serde_json::Value>(json_line.trim())
        .map_err(|err| format!("Could not parse Python pylsl helper report: {err}"))?;
    let pushed_unix_ms = value
        .get("pushed_unix_ms")
        .and_then(serde_json::Value::as_u64)
        .map(u128::from)
        .unwrap_or_else(unix_ms);
    let completed_unix_ms = value
        .get("completed_unix_ms")
        .and_then(serde_json::Value::as_u64)
        .map(u128::from)
        .unwrap_or_else(unix_ms);
    let library_info = value
        .get("library_info")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("pylsl library_info unavailable");

    Ok(OperatorSignalLslSendReport {
        protocol_version: OPERATOR_SIGNAL_LSL_PROTOCOL_VERSION.to_string(),
        accepted: true,
        sender_kind: "python_pylsl_helper".to_string(),
        stream_name,
        stream_type,
        source_id,
        channel_count: options.values.len(),
        sample_count: repeat_count,
        pushed_unix_ms,
        completed_unix_ms,
        values: options.values.clone(),
        lsl_library_detail: format!("{library_info} via {python}. stderr: {}", stderr.trim()),
    })
}

#[cfg(windows)]
mod windows_lsl {
    use super::*;
    use std::env;
    use std::ffi::{CStr, CString};
    use std::mem;
    use std::os::raw::{c_char, c_double, c_int, c_uint, c_void};
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use std::ptr;
    use std::slice;

    const RESOLVE_BUFFER_SIZE: usize = 16;
    const LSL_FLOAT_CHANNEL_FORMAT: i32 = 1;
    const LSL_STRING_CHANNEL_FORMAT: i32 = 3;
    const LSL_PROCESSING_ALL: u32 = 1 | 2 | 4 | 8;

    type LslResolveByProp = unsafe extern "C" fn(
        *mut *mut c_void,
        c_uint,
        *const c_char,
        *const c_char,
        c_int,
        c_double,
    ) -> c_int;
    type LslDestroyStreamInfo = unsafe extern "C" fn(*mut c_void);
    type LslGetString = unsafe extern "C" fn(*mut c_void) -> *const c_char;
    type LslGetDouble = unsafe extern "C" fn(*mut c_void) -> c_double;
    type LslGetInt = unsafe extern "C" fn(*mut c_void) -> c_int;
    type LslCreateInlet = unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int) -> *mut c_void;
    type LslDestroyInlet = unsafe extern "C" fn(*mut c_void);
    type LslCreateStreamInfo = unsafe extern "C" fn(
        *const c_char,
        *const c_char,
        c_int,
        c_double,
        c_int,
        *const c_char,
    ) -> *mut c_void;
    type LslCreateOutlet = unsafe extern "C" fn(*mut c_void, c_int, c_int) -> *mut c_void;
    type LslDestroyOutlet = unsafe extern "C" fn(*mut c_void);
    type LslPushFloatSample = unsafe extern "C" fn(*mut c_void, *const f32) -> c_int;
    type LslPushStringSample = unsafe extern "C" fn(*mut c_void, *const *const c_char) -> c_int;
    type LslOpenStream = unsafe extern "C" fn(*mut c_void, c_double, *mut c_int);
    type LslSetPostProcessing = unsafe extern "C" fn(*mut c_void, c_uint) -> c_int;
    type LslPullStringSample = unsafe extern "C" fn(
        *mut c_void,
        *mut *mut c_char,
        *mut c_uint,
        c_int,
        c_double,
        *mut c_int,
    ) -> c_double;
    type LslDestroyString = unsafe extern "C" fn(*mut c_char);
    type LslLastError = unsafe extern "C" fn() -> *const c_char;
    type LslLocalClock = unsafe extern "C" fn() -> c_double;
    type LslLibraryInfo = unsafe extern "C" fn() -> *const c_char;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LoadLibraryW(lp_lib_file_name: *const u16) -> *mut c_void;
        fn GetProcAddress(h_module: *mut c_void, lp_proc_name: *const c_char) -> *mut c_void;
    }

    struct LslLibrary {
        _handle: *mut c_void,
        detail: String,
        library_info: String,
        resolve_byprop: LslResolveByProp,
        destroy_streaminfo: LslDestroyStreamInfo,
        get_name: LslGetString,
        get_type: LslGetString,
        get_source_id: LslGetString,
        get_created_at: LslGetDouble,
        get_channel_count: LslGetInt,
        get_nominal_srate: LslGetDouble,
        get_channel_format: LslGetInt,
        create_inlet: LslCreateInlet,
        destroy_inlet: LslDestroyInlet,
        create_streaminfo: LslCreateStreamInfo,
        create_outlet: LslCreateOutlet,
        destroy_outlet: LslDestroyOutlet,
        push_float_sample: LslPushFloatSample,
        push_string_sample: LslPushStringSample,
        open_stream: LslOpenStream,
        set_postprocessing: LslSetPostProcessing,
        pull_string_sample: LslPullStringSample,
        destroy_string: LslDestroyString,
        last_error: LslLastError,
        local_clock: LslLocalClock,
        library_info_fn: LslLibraryInfo,
    }

    struct LslInlet<'a> {
        library: &'a LslLibrary,
        handle: *mut c_void,
        stream: ResolvedStringStream,
    }

    impl Drop for LslInlet<'_> {
        fn drop(&mut self) {
            if !self.handle.is_null() {
                unsafe {
                    (self.library.destroy_inlet)(self.handle);
                }
                self.handle = ptr::null_mut();
            }
        }
    }

    struct LslOwnedStreamInfo<'a> {
        library: &'a LslLibrary,
        handle: *mut c_void,
    }

    impl Drop for LslOwnedStreamInfo<'_> {
        fn drop(&mut self) {
            if !self.handle.is_null() {
                unsafe {
                    (self.library.destroy_streaminfo)(self.handle);
                }
                self.handle = ptr::null_mut();
            }
        }
    }

    struct LslOutlet<'a> {
        library: &'a LslLibrary,
        handle: *mut c_void,
    }

    impl Drop for LslOutlet<'_> {
        fn drop(&mut self) {
            if !self.handle.is_null() {
                unsafe {
                    (self.library.destroy_outlet)(self.handle);
                }
                self.handle = ptr::null_mut();
            }
        }
    }

    pub(super) fn record_runtime_state_lsl_csv(
        options: &RuntimeStateLslRecordingOptions,
        runtime_columns: &[&str],
    ) -> Result<RuntimeStateLslRecordingReport, String> {
        if runtime_columns.is_empty() {
            return Err(
                "runtime-state LSL recording requires a non-empty column contract.".to_string(),
            );
        }

        let library = LslLibrary::load()?;
        let started_unix_ms = unix_ms();
        let mut inlet = resolve_string_stream(&library, options)?;
        if inlet.stream.channel_count != runtime_columns.len() {
            return Err(format!(
                "Resolved LSL stream has {} channels, but the runtime-state contract expects {} columns.",
                inlet.stream.channel_count,
                runtime_columns.len()
            ));
        }

        let mut writer = prepare_output(&options.out)?;
        write_runtime_state_lsl_header(&mut writer, runtime_columns)?;

        let start = Instant::now();
        let mut last_sample_at = Instant::now();
        let mut sample_count = 0usize;

        loop {
            if let Some(max_samples) = options.max_samples {
                if sample_count >= max_samples {
                    break;
                }
            }
            if let Some(duration_ms) = options.duration_ms {
                if start.elapsed() >= Duration::from_millis(duration_ms) {
                    break;
                }
            }

            match pull_string_sample(&mut inlet, options.pull_timeout_ms)? {
                Some(sample) => {
                    if sample.values.len() != runtime_columns.len() {
                        return Err(format!(
                            "Received LSL sample with {} values, but expected {}.",
                            sample.values.len(),
                            runtime_columns.len()
                        ));
                    }

                    write_runtime_state_lsl_row(&mut writer, &inlet.stream, &sample)?;
                    writer.flush().map_err(|err| err.to_string())?;
                    sample_count += 1;
                    last_sample_at = Instant::now();
                }
                None => {
                    if last_sample_at.elapsed() >= Duration::from_millis(options.idle_timeout_ms) {
                        if sample_count == 0 {
                            return Err(format!(
                                "No runtime-state LSL samples arrived from {} / {} before idle timeout.",
                                inlet.stream.name, inlet.stream.stream_type
                            ));
                        }
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }

        let ended_unix_ms = unix_ms();
        Ok(RuntimeStateLslRecordingReport {
            protocol_version: RUNTIME_STATE_LSL_PROTOCOL_VERSION.to_string(),
            accepted: true,
            out: options.out.clone(),
            stream_name: inlet.stream.name.clone(),
            stream_type: inlet.stream.stream_type.clone(),
            stream_source_id: inlet.stream.source_id.clone(),
            stream_channel_count: runtime_columns.len(),
            stream_sample_rate_hz: inlet.stream.sample_rate_hz,
            sample_count,
            started_unix_ms,
            ended_unix_ms,
            lsl_library_detail: library.detail_with_info(),
        })
    }

    pub(super) fn send_operator_command_lsl_json(
        options: &OperatorCommandLslSendOptions,
    ) -> Result<OperatorCommandLslSendReport, String> {
        if options.command_json.trim().is_empty() {
            return Err("LSL operator command requires non-empty command JSON.".to_string());
        }

        let library = LslLibrary::load()?;
        let mut ack_inlet = if options.wait_for_ack {
            Some(resolve_ack_stream(&library, options)?)
        } else {
            None
        };

        let stream_name = non_empty(&options.stream_name, DEFAULT_OPERATOR_COMMAND_STREAM_NAME);
        let stream_type = non_empty(&options.stream_type, DEFAULT_OPERATOR_COMMAND_STREAM_TYPE);
        let source_id = non_empty(&options.source_id, "windows.operator.peripersonal.command");
        let stream_info =
            create_string_stream_info(&library, &stream_name, &stream_type, &source_id)?;
        let outlet_handle = unsafe { (library.create_outlet)(stream_info.handle, 1, 60) };
        if outlet_handle.is_null() {
            return Err(format!(
                "Could not create LSL operator command outlet. {}",
                library.last_error()
            )
            .trim()
            .to_string());
        }
        let outlet = LslOutlet {
            library: &library,
            handle: outlet_handle,
        };

        thread::sleep(Duration::from_millis(options.warmup_ms));
        let command = CString::new(options.command_json.as_str())
            .map_err(|_| "LSL operator command JSON must not contain NUL bytes.".to_string())?;
        let sample = [command.as_ptr()];
        let error_code = unsafe { (library.push_string_sample)(outlet.handle, sample.as_ptr()) };
        if error_code != 0 {
            return Err(library.error_message("Could not push LSL operator command", error_code));
        }
        let pushed_unix_ms = unix_ms();

        let mut ack_payload = None;
        if let Some(inlet) = ack_inlet.as_mut() {
            ack_payload = wait_for_ack(inlet, options.ack_timeout_ms)?;
            if ack_payload.is_none() {
                return Err(format!(
                    "No LSL operator command ack arrived from {} / {} before timeout.",
                    options.ack_stream_name, options.ack_stream_type
                ));
            }
        } else {
            thread::sleep(Duration::from_millis(options.post_push_hold_ms));
        }

        let completed_unix_ms = unix_ms();
        Ok(OperatorCommandLslSendReport {
            protocol_version: OPERATOR_COMMAND_LSL_PROTOCOL_VERSION.to_string(),
            accepted: ack_payload
                .as_deref()
                .map(ack_payload_accepted)
                .unwrap_or(true),
            stream_name,
            stream_type,
            source_id,
            pushed_unix_ms,
            completed_unix_ms,
            ack_stream_name: non_empty(
                &options.ack_stream_name,
                DEFAULT_OPERATOR_COMMAND_ACK_STREAM_NAME,
            ),
            ack_stream_type: non_empty(
                &options.ack_stream_type,
                DEFAULT_OPERATOR_COMMAND_ACK_STREAM_TYPE,
            ),
            ack_received: ack_payload.is_some(),
            ack_payload,
            lsl_library_detail: library.detail_with_info(),
        })
    }

    pub(super) fn send_operator_signal_lsl_floats(
        options: &OperatorSignalLslSendOptions,
    ) -> Result<OperatorSignalLslSendReport, String> {
        if options.values.is_empty() {
            return Err("LSL operator signal requires at least one float value.".to_string());
        }

        let library = LslLibrary::load()?;
        let stream_name = non_empty(&options.stream_name, DEFAULT_OPERATOR_SIGNAL_STREAM_NAME);
        let stream_type = non_empty(&options.stream_type, DEFAULT_OPERATOR_SIGNAL_STREAM_TYPE);
        let source_id = non_empty(&options.source_id, "windows.operator.peripersonal.signal");
        let stream_info = create_float_stream_info(
            &library,
            &stream_name,
            &stream_type,
            &source_id,
            options.values.len(),
        )?;
        let outlet_handle = unsafe { (library.create_outlet)(stream_info.handle, 1, 60) };
        if outlet_handle.is_null() {
            return Err(format!(
                "Could not create LSL operator signal outlet. {}",
                library.last_error()
            )
            .trim()
            .to_string());
        }
        let outlet = LslOutlet {
            library: &library,
            handle: outlet_handle,
        };

        thread::sleep(Duration::from_millis(options.warmup_ms));
        let repeat_count = options.repeat_count.max(1);
        let mut pushed_unix_ms = 0;
        for index in 0..repeat_count {
            let error_code =
                unsafe { (library.push_float_sample)(outlet.handle, options.values.as_ptr()) };
            if error_code != 0 {
                return Err(library.error_message("Could not push LSL operator signal", error_code));
            }
            if pushed_unix_ms == 0 {
                pushed_unix_ms = unix_ms();
            }
            if index + 1 < repeat_count {
                thread::sleep(Duration::from_millis(options.repeat_interval_ms));
            }
        }
        thread::sleep(Duration::from_millis(options.post_push_hold_ms));
        let completed_unix_ms = unix_ms();

        Ok(OperatorSignalLslSendReport {
            protocol_version: OPERATOR_SIGNAL_LSL_PROTOCOL_VERSION.to_string(),
            accepted: true,
            sender_kind: "native_liblsl".to_string(),
            stream_name,
            stream_type,
            source_id,
            channel_count: options.values.len(),
            sample_count: repeat_count,
            pushed_unix_ms,
            completed_unix_ms,
            values: options.values.clone(),
            lsl_library_detail: library.detail_with_info(),
        })
    }

    fn resolve_ack_stream<'a>(
        library: &'a LslLibrary,
        options: &OperatorCommandLslSendOptions,
    ) -> Result<LslInlet<'a>, String> {
        let recording_options = RuntimeStateLslRecordingOptions {
            out: PathBuf::new(),
            stream_name: non_empty(
                &options.ack_stream_name,
                DEFAULT_OPERATOR_COMMAND_ACK_STREAM_NAME,
            ),
            stream_type: non_empty(
                &options.ack_stream_type,
                DEFAULT_OPERATOR_COMMAND_ACK_STREAM_TYPE,
            ),
            source_id_prefix: None,
            resolve_timeout_ms: options.ack_timeout_ms,
            pull_timeout_ms: 100,
            idle_timeout_ms: options.ack_timeout_ms,
            duration_ms: None,
            max_samples: Some(1),
        };
        let inlet = resolve_string_stream(library, &recording_options)?;
        if inlet.stream.channel_count != 1 {
            return Err(format!(
                "Resolved LSL operator command ack stream has {} channels; expected 1.",
                inlet.stream.channel_count
            ));
        }
        Ok(inlet)
    }

    fn create_string_stream_info<'a>(
        library: &'a LslLibrary,
        name: &str,
        stream_type: &str,
        source_id: &str,
    ) -> Result<LslOwnedStreamInfo<'a>, String> {
        let name = CString::new(name).map_err(|err| err.to_string())?;
        let stream_type = CString::new(stream_type).map_err(|err| err.to_string())?;
        let source_id = CString::new(source_id).map_err(|err| err.to_string())?;
        let handle = unsafe {
            (library.create_streaminfo)(
                name.as_ptr(),
                stream_type.as_ptr(),
                1,
                0.0,
                LSL_STRING_CHANNEL_FORMAT,
                source_id.as_ptr(),
            )
        };
        if handle.is_null() {
            return Err(format!(
                "Could not create LSL operator command stream info. {}",
                library.last_error()
            )
            .trim()
            .to_string());
        }
        Ok(LslOwnedStreamInfo { library, handle })
    }

    fn create_float_stream_info<'a>(
        library: &'a LslLibrary,
        name: &str,
        stream_type: &str,
        source_id: &str,
        channel_count: usize,
    ) -> Result<LslOwnedStreamInfo<'a>, String> {
        let name = CString::new(name).map_err(|err| err.to_string())?;
        let stream_type = CString::new(stream_type).map_err(|err| err.to_string())?;
        let source_id = CString::new(source_id).map_err(|err| err.to_string())?;
        let handle = unsafe {
            (library.create_streaminfo)(
                name.as_ptr(),
                stream_type.as_ptr(),
                channel_count as i32,
                0.0,
                LSL_FLOAT_CHANNEL_FORMAT,
                source_id.as_ptr(),
            )
        };
        if handle.is_null() {
            return Err(format!(
                "Could not create LSL operator signal stream info. {}",
                library.last_error()
            )
            .trim()
            .to_string());
        }
        Ok(LslOwnedStreamInfo { library, handle })
    }

    fn wait_for_ack(
        inlet: &mut LslInlet<'_>,
        ack_timeout_ms: u64,
    ) -> Result<Option<String>, String> {
        let deadline = Instant::now() + Duration::from_millis(ack_timeout_ms);
        loop {
            match pull_string_sample(inlet, 100)? {
                Some(sample) => {
                    return Ok(sample.values.into_iter().next());
                }
                None => {
                    if Instant::now() >= deadline {
                        return Ok(None);
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }

    fn ack_payload_accepted(payload: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(payload)
            .ok()
            .and_then(|value| value.get("accepted").and_then(serde_json::Value::as_bool))
            .unwrap_or(false)
    }

    fn non_empty(value: &str, fallback: &str) -> String {
        if value.trim().is_empty() {
            fallback.to_string()
        } else {
            value.trim().to_string()
        }
    }

    fn resolve_string_stream<'a>(
        library: &'a LslLibrary,
        options: &RuntimeStateLslRecordingOptions,
    ) -> Result<LslInlet<'a>, String> {
        let deadline = Instant::now() + Duration::from_millis(options.resolve_timeout_ms);
        loop {
            if let Some(inlet) = try_resolve_string_stream(library, options)? {
                return Ok(inlet);
            }

            if Instant::now() >= deadline {
                return Err(format!(
                    "No visible LSL stream matched {} / {} before resolve timeout.",
                    options.stream_name, options.stream_type
                ));
            }

            thread::sleep(Duration::from_millis(250));
        }
    }

    fn try_resolve_string_stream<'a>(
        library: &'a LslLibrary,
        options: &RuntimeStateLslRecordingOptions,
    ) -> Result<Option<LslInlet<'a>>, String> {
        let stream_name = options.stream_name.trim();
        let stream_type = options.stream_type.trim();
        let (property, value) = if !stream_name.is_empty() {
            ("name", stream_name)
        } else if !stream_type.is_empty() {
            ("type", stream_type)
        } else {
            return Err("At least a stream name or stream type is required.".to_string());
        };

        let property = CString::new(property).map_err(|err| err.to_string())?;
        let value = CString::new(value).map_err(|err| err.to_string())?;
        let mut infos = [ptr::null_mut(); RESOLVE_BUFFER_SIZE];
        let count = unsafe {
            (library.resolve_byprop)(
                infos.as_mut_ptr(),
                infos.len() as u32,
                property.as_ptr(),
                value.as_ptr(),
                1,
                1.0,
            )
        };

        if count < 0 {
            destroy_stream_infos(library, &infos);
            return Err(library.error_message("Could not resolve LSL runtime-state stream", count));
        }
        if count == 0 {
            destroy_stream_infos(library, &infos);
            return Ok(None);
        }

        let mut selected: *mut c_void = ptr::null_mut();
        let mut selected_created_at = f64::NEG_INFINITY;
        for candidate in infos.iter().take(count as usize) {
            if candidate.is_null() || !matches_requested_stream(library, *candidate, options)? {
                continue;
            }

            let created_at = unsafe { (library.get_created_at)(*candidate) };
            if selected.is_null() || created_at > selected_created_at {
                selected = *candidate;
                selected_created_at = created_at;
            }
        }

        if selected.is_null() {
            destroy_stream_infos(library, &infos);
            return Ok(None);
        }

        let channel_count = unsafe { (library.get_channel_count)(selected) };
        let channel_format = unsafe { (library.get_channel_format)(selected) };
        if channel_count <= 0 {
            destroy_stream_infos(library, &infos);
            return Err(
                "Resolved runtime-state LSL stream did not report any channels.".to_string(),
            );
        }
        if channel_format != LSL_STRING_CHANNEL_FORMAT {
            destroy_stream_infos(library, &infos);
            return Err(format!(
                "Resolved runtime-state LSL stream uses channel format {channel_format}; expected cf_string."
            ));
        }

        let stream = ResolvedStringStream {
            name: unsafe { ptr_to_string((library.get_name)(selected)) },
            stream_type: unsafe { ptr_to_string((library.get_type)(selected)) },
            source_id: unsafe { ptr_to_string((library.get_source_id)(selected)) },
            channel_count: channel_count as usize,
            sample_rate_hz: unsafe { (library.get_nominal_srate)(selected) },
        };

        let inlet = unsafe { (library.create_inlet)(selected, 30, 1, 1) };
        destroy_stream_infos(library, &infos);
        if inlet.is_null() {
            return Err(
                format!("Could not create LSL inlet. {}", library.last_error())
                    .trim()
                    .to_string(),
            );
        }

        let mut error_code = 0;
        unsafe {
            (library.open_stream)(inlet, 2.0, &mut error_code);
        }
        if error_code != 0 {
            unsafe {
                (library.destroy_inlet)(inlet);
            }
            return Err(
                library.error_message("Could not open LSL runtime-state stream", error_code)
            );
        }

        let error_code = unsafe { (library.set_postprocessing)(inlet, LSL_PROCESSING_ALL) };
        if error_code != 0 {
            unsafe {
                (library.destroy_inlet)(inlet);
            }
            return Err(library.error_message(
                "Could not configure LSL runtime-state inlet post-processing",
                error_code,
            ));
        }

        Ok(Some(LslInlet {
            library,
            handle: inlet,
            stream,
        }))
    }

    fn pull_string_sample(
        inlet: &mut LslInlet<'_>,
        pull_timeout_ms: u64,
    ) -> Result<Option<RuntimeStateLslSample>, String> {
        let mut ptrs = vec![ptr::null_mut(); inlet.stream.channel_count];
        let mut lengths = vec![0u32; inlet.stream.channel_count];
        let mut error_code = 0;
        let timestamp = unsafe {
            (inlet.library.pull_string_sample)(
                inlet.handle,
                ptrs.as_mut_ptr(),
                lengths.as_mut_ptr(),
                inlet.stream.channel_count as i32,
                pull_timeout_ms as f64 / 1000.0,
                &mut error_code,
            )
        };

        if error_code != 0 {
            return Err(inlet
                .library
                .error_message("Could not pull runtime-state LSL sample", error_code));
        }
        if timestamp == 0.0 {
            return Ok(None);
        }

        let operator_lsl_local_clock_seconds = unsafe { (inlet.library.local_clock)() };
        let mut values = Vec::with_capacity(ptrs.len());
        for (index, ptr) in ptrs.into_iter().enumerate() {
            if ptr.is_null() {
                values.push(String::new());
                continue;
            }

            let length = lengths[index] as usize;
            let value = unsafe {
                let bytes = slice::from_raw_parts(ptr as *const u8, length);
                let nul_index = bytes
                    .iter()
                    .position(|byte| *byte == 0)
                    .unwrap_or(bytes.len());
                String::from_utf8_lossy(&bytes[..nul_index]).into_owned()
            };
            unsafe {
                (inlet.library.destroy_string)(ptr);
            }
            values.push(value);
        }

        Ok(Some(RuntimeStateLslSample {
            source_lsl_timestamp_seconds: timestamp,
            operator_lsl_local_clock_seconds,
            values,
        }))
    }

    fn matches_requested_stream(
        library: &LslLibrary,
        stream_info: *mut c_void,
        options: &RuntimeStateLslRecordingOptions,
    ) -> Result<bool, String> {
        let name = unsafe { ptr_to_string((library.get_name)(stream_info)) };
        let stream_type = unsafe { ptr_to_string((library.get_type)(stream_info)) };
        let source_id = unsafe { ptr_to_string((library.get_source_id)(stream_info)) };
        let name_matches =
            options.stream_name.trim().is_empty() || name == options.stream_name.trim();
        let type_matches =
            options.stream_type.trim().is_empty() || stream_type == options.stream_type.trim();
        let source_matches = options
            .source_id_prefix
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|prefix| source_id.starts_with(prefix.trim()))
            .unwrap_or(true);
        Ok(name_matches && type_matches && source_matches)
    }

    fn destroy_stream_infos(library: &LslLibrary, infos: &[*mut c_void]) {
        for info in infos {
            if !info.is_null() {
                unsafe {
                    (library.destroy_streaminfo)(*info);
                }
            }
        }
    }

    impl LslLibrary {
        fn load() -> Result<Self, String> {
            let candidates = candidate_library_paths();
            for candidate in &candidates {
                if !candidate.exists() {
                    continue;
                }

                let handle = unsafe { LoadLibraryW(wide_path(candidate).as_ptr()) };
                if handle.is_null() {
                    continue;
                }

                let mut library =
                    unsafe { Self::from_handle(handle, candidate.display().to_string())? };
                library.library_info = unsafe { ptr_to_string((library.library_info_fn)()) };
                return Ok(library);
            }

            Err(format!(
                "Could not locate or load lsl.dll. Set VISCEREALITY_LSL_DLL or place lsl.dll next to the operator executable. Searched: {}",
                candidates
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ))
        }

        unsafe fn from_handle(handle: *mut c_void, detail: String) -> Result<Self, String> {
            Ok(Self {
                _handle: handle,
                detail,
                library_info: String::new(),
                resolve_byprop: load_symbol(handle, "lsl_resolve_byprop")?,
                destroy_streaminfo: load_symbol(handle, "lsl_destroy_streaminfo")?,
                get_name: load_symbol(handle, "lsl_get_name")?,
                get_type: load_symbol(handle, "lsl_get_type")?,
                get_source_id: load_symbol(handle, "lsl_get_source_id")?,
                get_created_at: load_symbol(handle, "lsl_get_created_at")?,
                get_channel_count: load_symbol(handle, "lsl_get_channel_count")?,
                get_nominal_srate: load_symbol(handle, "lsl_get_nominal_srate")?,
                get_channel_format: load_symbol(handle, "lsl_get_channel_format")?,
                create_inlet: load_symbol(handle, "lsl_create_inlet")?,
                destroy_inlet: load_symbol(handle, "lsl_destroy_inlet")?,
                create_streaminfo: load_symbol(handle, "lsl_create_streaminfo")?,
                create_outlet: load_symbol(handle, "lsl_create_outlet")?,
                destroy_outlet: load_symbol(handle, "lsl_destroy_outlet")?,
                push_float_sample: load_symbol(handle, "lsl_push_sample_f")?,
                push_string_sample: load_symbol(handle, "lsl_push_sample_str")?,
                open_stream: load_symbol(handle, "lsl_open_stream")?,
                set_postprocessing: load_symbol(handle, "lsl_set_postprocessing")?,
                pull_string_sample: load_symbol(handle, "lsl_pull_sample_buf")?,
                destroy_string: load_symbol(handle, "lsl_destroy_string")?,
                last_error: load_symbol(handle, "lsl_last_error")?,
                local_clock: load_symbol(handle, "lsl_local_clock")?,
                library_info_fn: load_symbol(handle, "lsl_library_info")?,
            })
        }

        fn detail_with_info(&self) -> String {
            if self.library_info.trim().is_empty() {
                format!("Loaded lsl.dll from {}.", self.detail)
            } else {
                format!("{} Loaded from {}.", self.library_info, self.detail)
            }
        }

        fn last_error(&self) -> String {
            unsafe { ptr_to_string((self.last_error)()) }
        }

        fn error_message(&self, prefix: &str, code: i32) -> String {
            let last_error = self.last_error();
            let code_name = match code {
                0 => "no error",
                -1 => "timeout",
                -2 => "stream lost",
                -3 => "invalid argument",
                -4 => "internal error",
                _ => "unknown error",
            };
            if last_error.trim().is_empty() {
                format!("{prefix} ({code_name})")
            } else {
                format!("{prefix} ({code_name}): {last_error}")
            }
        }
    }

    unsafe fn load_symbol<T: Copy>(handle: *mut c_void, name: &str) -> Result<T, String> {
        let c_name = CString::new(name).map_err(|err| err.to_string())?;
        let ptr = GetProcAddress(handle, c_name.as_ptr());
        if ptr.is_null() {
            return Err(format!("lsl.dll is missing required symbol {name}."));
        }
        Ok(mem::transmute_copy(&ptr))
    }

    unsafe fn ptr_to_string(pointer: *const c_char) -> String {
        if pointer.is_null() {
            String::new()
        } else {
            CStr::from_ptr(pointer).to_string_lossy().into_owned()
        }
    }

    fn candidate_library_paths() -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        if let Ok(value) = env::var("VISCEREALITY_LSL_DLL") {
            push_non_empty_path(&mut candidates, value);
        }

        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                candidates.push(exe_dir.join("lsl.dll"));
                candidates.push(
                    exe_dir
                        .join("runtimes")
                        .join("win-x64")
                        .join("native")
                        .join("lsl.dll"),
                );
            }
        }
        if let Ok(current_dir) = env::current_dir() {
            candidates.push(current_dir.join("lsl.dll"));
            candidates.push(
                current_dir
                    .join("runtimes")
                    .join("win-x64")
                    .join("native")
                    .join("lsl.dll"),
            );
        }

        if let Ok(user_profile) = env::var("USERPROFILE") {
            add_user_tools_liblsl_candidates(&mut candidates, Path::new(&user_profile));
        }

        dedupe_paths(candidates)
    }

    fn add_user_tools_liblsl_candidates(candidates: &mut Vec<PathBuf>, user_profile: &Path) {
        let root = user_profile.join("Tools").join("liblsl");
        let Ok(entries) = fs::read_dir(root) else {
            return;
        };
        let mut version_dirs = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        version_dirs.sort_by(|left, right| right.file_name().cmp(&left.file_name()));
        for path in version_dirs {
            candidates.push(path.join("bin").join("lsl.dll"));
        }
    }

    fn push_non_empty_path(candidates: &mut Vec<PathBuf>, path: String) {
        if !path.trim().is_empty() {
            candidates.push(PathBuf::from(path));
        }
    }

    fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
        let mut deduped = Vec::new();
        for path in paths {
            if !deduped
                .iter()
                .any(|existing: &PathBuf| same_path_case_insensitive(existing, &path))
            {
                deduped.push(path);
            }
        }
        deduped
    }

    fn same_path_case_insensitive(left: &Path, right: &Path) -> bool {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    }

    fn wide_path(path: &Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }
}
