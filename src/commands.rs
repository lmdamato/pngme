use std::path::PathBuf;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Encode a message into a PNG image file
    #[command()]
    Encode {
        /// The path of the image
        #[arg(required = true)]
        path: PathBuf,

        /// A 4-byte ASCII string which will be used to encode the message
        #[arg(required = true)]
        chunk_type: String,

        /// The message to be encoded
        #[arg(required = true)]
        message: String,
    },

    /// Decode a message contained in a PNG image file
    #[command()]
    Decode {
        /// The path of the image
        #[arg(required = true)]
        path: PathBuf,

        /// A 4-byte ASCII string which will be used to decode the message
        #[arg(required = true)]
        chunk_type: String,
    },

    /// Remove a message encoded in a PNG image file
    #[command()]
    Remove {
        /// The path of the image
        #[arg(required = true)]
        path: PathBuf,

        /// A 4-byte ASCII string which will be used to locate the message to remove
        #[arg(required = true)]
        chunk_type: String,
    },

    /// Prints a PNG image file to screen
    #[command()]
    Print {
        /// The path of the image
        #[arg(required = true)]
        path: PathBuf,
    },
}