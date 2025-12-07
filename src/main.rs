//! open-tts-rs CLI entry point.

use std::fs;
use std::io::Write;

use anyhow::{Context, Result};
use clap::Parser;
use open_tts_rs::backend::create_backend;
use open_tts_rs::cli::{Args, Reference};
use open_tts_rs::engine::TTSEngine;
use open_tts_rs::voice::VoiceManager;

fn main() -> Result<()> {
    let args = Args::parse();

    // Create voice manager and backend
    let voice_manager = VoiceManager::new();
    let backend = create_backend(args.model, &args.host);
    let engine = TTSEngine::new(backend, voice_manager);

    // Handle utility commands first
    if args.list_voices {
        return list_voices(&engine);
    }

    if let Some(name) = &args.delete_voice {
        return delete_voice(&engine, name);
    }

    // Parse reference if provided (extract voice)
    if let Some(ref_str) = &args.reference {
        let reference = Reference::parse(ref_str)?;

        let voice_info = engine
            .extract_voice(
                &reference.audio_path,
                &reference.transcript,
                args.name.clone(),
            )
            .context("Failed to extract voice from reference audio")?;

        println!("Voice extracted: {}", voice_info.name);
        println!("  Transcript: {}", voice_info.transcript);
        println!("  Model: {}", voice_info.model);
        if let Some(duration) = voice_info.duration {
            println!("  Duration: {:.2}s", duration);
        }

        // If no generate flag, just extract and exit
        if args.generate.is_none() {
            return Ok(());
        }
    } else if let Some(name) = &args.name {
        // Load existing voice (just verify it exists)
        let manager = VoiceManager::new();
        manager
            .load_metadata(name)
            .with_context(|| format!("Voice '{}' not found", name))?;
        println!("Using voice: {name}");
    }

    // Generate speech if requested
    if let Some(text) = &args.generate {
        return generate_speech(&engine, text, args.name, args.speed, &args.output);
    }

    // No action specified
    if args.reference.is_none() && args.generate.is_none() {
        eprintln!("No action specified. Use -r to extract a voice or -g to generate speech.");
        eprintln!("Run with --help for usage information.");
    }

    Ok(())
}

fn list_voices<B: open_tts_rs::backend::Backend>(engine: &TTSEngine<B>) -> Result<()> {
    let voices = engine.list_voices().context("Failed to list voices")?;

    if voices.is_empty() {
        println!("No voices found.");
        return Ok(());
    }

    println!("Available voices:");
    for voice in voices {
        println!("  {} ({})", voice.name, voice.model);
        println!("    Transcript: {}", voice.transcript);
        if let Some(duration) = voice.duration {
            println!("    Duration: {:.2}s", duration);
        }
    }

    Ok(())
}

fn delete_voice<B: open_tts_rs::backend::Backend>(engine: &TTSEngine<B>, name: &str) -> Result<()> {
    engine
        .delete_voice(name)
        .with_context(|| format!("Failed to delete voice '{}'", name))?;

    println!("Voice '{}' deleted.", name);
    Ok(())
}

fn generate_speech<B: open_tts_rs::backend::Backend>(
    engine: &TTSEngine<B>,
    text: &str,
    voice_name: Option<String>,
    speed: f32,
    output: &std::path::Path,
) -> Result<()> {
    println!("Generating speech...");
    if let Some(ref name) = voice_name {
        println!("  Voice: {}", name);
    }
    println!("  Speed: {:.1}x", speed);

    let audio_data = engine
        .synthesize(text, voice_name, speed)
        .context("Failed to synthesize speech")?;

    // Write audio to file
    let mut file = fs::File::create(output)
        .with_context(|| format!("Failed to create output file: {}", output.display()))?;

    file.write_all(&audio_data)
        .with_context(|| format!("Failed to write audio to: {}", output.display()))?;

    println!("Audio saved to: {}", output.display());
    println!("  Size: {} bytes", audio_data.len());

    Ok(())
}
