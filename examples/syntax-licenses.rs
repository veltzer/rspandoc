//! Print the licences of the syntax definitions rspandoc embeds (through
//! two-face, from bat), which `THIRD-PARTY-SYNTAXES.md` records:
//!
//! ```sh
//! cargo run -q --example syntax-licenses > THIRD-PARTY-SYNTAXES.md
//! ```

fn main() {
    print!("{}", two_face::acknowledgement::listing().to_md());
}
