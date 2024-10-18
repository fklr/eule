use eule::error::{create_report, EuleError};
use miette::Report;
use poise::serenity_prelude;
use std::io;

#[test]
fn test_all_error_variants() {
    let errors = vec![
        EuleError::DiscordApi(serenity_prelude::Error::Other("Discord API error")),
        EuleError::Database(sled::Error::CollectionNotFound("Database error".into())),
        EuleError::AuthenticationFailed("Failed to authenticate".into()),
        EuleError::Io(io::Error::new(io::ErrorKind::Other, "IO error")),
        EuleError::Serialization(serde_json::Error::io(io::Error::new(
            io::ErrorKind::Other,
            "Serialization error",
        ))),
        EuleError::LockError("Lock acquisition failed".into()),
        EuleError::NotInGuild,
        EuleError::InvalidTimeUnit,
        EuleError::TracingSetupFailed("Tracing setup error".into()),
        EuleError::Poise("Poise framework error".into()),
        EuleError::Miette(Report::msg("Miette error")),
    ];

    for error in errors {
        let report = create_report(error, Some("This is a help message"));
        print_report(report);
    }
}

fn print_report(report: Report) {
    let output = format!("{:?}", report);
    println!("{}", output);

    assert!(report.to_string().contains("This is a help message"));
}

#[test]
fn test_error_display() {
    let errors = vec![
        EuleError::DiscordApi(poise::serenity_prelude::Error::Other("Discord API error")),
        EuleError::Database(sled::Error::CollectionNotFound("Database error".into())),
        EuleError::AuthenticationFailed("Failed to authenticate".into()),
        EuleError::Io(io::Error::new(io::ErrorKind::Other, "IO error")),
        EuleError::Serialization(serde_json::Error::io(io::Error::new(
            io::ErrorKind::Other,
            "Serialization error",
        ))),
        EuleError::LockError("Lock acquisition failed".into()),
        EuleError::NotInGuild,
        EuleError::InvalidTimeUnit,
        EuleError::TracingSetupFailed("Tracing setup error".into()),
        EuleError::Poise("Poise framework error".into()),
        EuleError::Miette(Report::msg("Miette error")),
    ];

    for error in errors {
        let error_string = format!("{}", error);
        println!("Error Display: {}", error_string);

        // Add assertions to check if the error message contains expected content
        match error {
            EuleError::DiscordApi(_) => assert!(error_string.contains("Discord API error")),
            EuleError::Database(_) => assert!(error_string.contains("Database error")),
            EuleError::AuthenticationFailed(_) => {
                assert!(error_string.contains("Authentication failed"))
            }
            EuleError::Io(_) => assert!(error_string.contains("IO error")),
            EuleError::Serialization(_) => assert!(error_string.contains("Serialization error")),
            EuleError::LockError(_) => assert!(error_string.contains("Lock error")),
            EuleError::NotInGuild => assert!(error_string.contains("Not in a guild")),
            EuleError::InvalidTimeUnit => assert!(error_string.contains("Invalid time unit")),
            EuleError::TracingSetupFailed(_) => {
                assert!(error_string.contains("Tracing setup failed"))
            }
            EuleError::Poise(_) => assert!(error_string.contains("Poise framework error")),
            EuleError::Miette(_) => assert!(error_string.contains("Miette error")),
        }
    }
}
