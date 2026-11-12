//! `ebc qr`: a QR code as an SVG image.

use ebc_qr::{EcLevel, QrCode};

use crate::args::{EcArg, QrArgs};
use crate::error::{CliError, Status};
use crate::util::{Streams, is_dash, write_output};

/// Runs `ebc qr`.
pub(crate) fn run(args: &QrArgs, streams: &mut Streams<'_>) -> Result<Status, CliError> {
    let level = match args.ec {
        EcArg::L => EcLevel::L,
        EcArg::M => EcLevel::M,
        EcArg::Q => EcLevel::Q,
        EcArg::H => EcLevel::H,
    };
    let encoded = if args.compact_url {
        QrCode::encode_url(&args.text, level)
    } else {
        QrCode::encode(&args.text, level)
    };
    let qr = encoded.map_err(|e| CliError::failure(format!("cannot make a QR code: {e}")))?;
    let mut svg = qr.to_svg(args.module_px, args.quiet);
    svg.push('\n');
    write_output(&args.output, svg.as_bytes(), streams)?;
    if args.quiet < ebc_qr::QUIET_ZONE_MODULES {
        streams.note(format!(
            "warning: a quiet zone of {} module(s) is below the {} the standard asks for",
            args.quiet,
            ebc_qr::QUIET_ZONE_MODULES
        ));
    }
    let side = qr.size() + 2 * args.quiet;
    let target = if is_dash(&args.output) {
        "standard output".to_owned()
    } else {
        args.output.display().to_string()
    };
    // The payload is not echoed: it may be a claim URL, which is a bearer secret.
    streams.note(format!(
        "wrote {target}: version {}, {} x {} modules, error correction {:?}, {} px wide",
        qr.version(),
        qr.size(),
        qr.size(),
        qr.ec_level(),
        u64::from(side) * u64::from(args.module_px)
    ));
    Ok(Status::Success)
}
