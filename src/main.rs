use std::io;
use std::io::BufRead;
use std::io::Write;

/// Returns the number of bytes of escape code that start the slice.
fn skip_all_escape_code(buf: &[u8]) -> usize {
    // Skip one sequence
    fn skip_escape_code(buf: &[u8]) -> Option<usize> {
        if 2 <= buf.len() && &buf[..2] == b"\x1b[" {
            // "\x1b[" + sequence body + "m" => 3 additional bytes
            Some(index_of(&buf[2..], b'm')? + 3)
        } else {
            None
        }
    }
    let mut buf = buf;
    let mut sum = 0;
    while let Some(nbytes) = skip_escape_code(&buf) {
        buf = &buf[nbytes..];
        sum += nbytes
    }
    sum
}

fn process_line<W>(w: &mut W, line: &[u8]) -> std::io::Result<()>
where
    W: Write,
{
    let mut i = 0;
    let len = line.len();
    while i < len {
        i += skip_all_escape_code(&line[i..]);
        let tok_len = skip_token(&line[i..]);
        w.write(&line[i..i + tok_len])?;
        i += tok_len;
    }
    Ok(())
}

/// Scan the slice looking for the given byte, returning the index of
/// its first appearance.
fn index_of(buf: &[u8], target: u8) -> Option<usize> {
    let mut it = buf.iter().enumerate();
    loop {
        match it.next() {
            Some((index, c)) => {
                if *c == target {
                    return Some(index);
                }
            }
            None => return None,
        }
    }
}

/// Computes the number of bytes until either the next escape code, or
/// the end of buf.
fn skip_token(buf: &[u8]) -> usize {
    match buf.len() {
        0 => 0,
        len => {
            for i in 0..buf.len() - 1 {
                if &buf[i..i + 2] == b"\x1b[" {
                    return i;
                }
            }
            len
        }
    }
}

fn try_main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut buffer = vec![];
    let mut stdin = stdin.lock();
    let mut stdout = stdout.lock();

    loop {
        stdin.read_until(b'\n', &mut buffer)?;
        if buffer.is_empty() {
            break;
        }

        process_line(&mut stdout, &buffer)?;
        buffer.clear();
    }

    Ok(())
}

fn main() {
    match try_main() {
        Ok(()) => (),
        Err(ref err) if err.kind() == io::ErrorKind::BrokenPipe => (),
        Err(ref err) => {
            eprintln!("io error: {}", err);
            std::process::exit(-1)
        }
    }
}
