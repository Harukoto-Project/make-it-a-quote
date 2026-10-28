use anyhow::{Context, Result};
use clap::Parser;
use miq_render::{encode_png, render_quote, set_assets_dir, FontCatalog, RenderOptions};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "miq", about = "Render a Make-it-a-Quote style PNG")]
struct Args {
    #[arg(long)]
    text: String,
    #[arg(long, default_value = "")]
    display_name: String,
    #[arg(long, default_value = "")]
    username: String,
    #[arg(long, default_value = "")]
    watermark: String,
    #[arg(long)]
    avatar: Option<PathBuf>,
    #[arg(long, short = 'o', default_value = "quote.png")]
    output: PathBuf,
    #[arg(long, env = "MIQ_ASSETS")]
    assets: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(dir) = args.assets {
        set_assets_dir(dir);
    }
    let catalog = FontCatalog::load_default().context("load fonts")?;
    let avatar = match &args.avatar {
        Some(p) => Some(std::fs::read(p).with_context(|| format!("read {}", p.display()))?),
        None => None,
    };
    let opts = RenderOptions {
        text: args.text,
        display_name: args.display_name,
        username: args.username,
        watermark: args.watermark,
    };
    let out = render_quote(&catalog, &opts, avatar.as_deref()).context("render")?;
    let png = encode_png(&out.image).context("encode png")?;
    std::fs::write(&args.output, png)
        .with_context(|| format!("write {}", args.output.display()))?;
    eprintln!(
        "wrote {} (font_size={}, lines={}, truncated={})",
        args.output.display(),
        out.layout.size,
        out.layout.lines.len(),
        out.layout.truncated
    );
    for (i, line) in out.layout.lines.iter().enumerate() {
        eprintln!("  [{i}] {line}");
    }
    Ok(())
}
