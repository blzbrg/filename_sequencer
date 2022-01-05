static HEADFILE_NAME : &str = "fname_seq_head";
static BASE_NAME_ORIG_NAME_SEP : &str = "_";

fn find_headstate<T: AsRef<std::path::Path>>(base_path : T) -> std::path::PathBuf {
    base_path.as_ref().with_file_name(HEADFILE_NAME)
}

fn write_headstate(headstate : &std::path::Path, file : &std::path::Path)
                   -> Result<(), Error> {
    let output = file.to_str()
        .ok_or(Error::PathUnicodeError(file.to_owned()))?;
    std::fs::write(headstate, output)
        .map_err(|e| Error::FileError(e, headstate.to_path_buf()))
}

#[derive(Debug)]
enum Error {
    BytesUnicodeError(std::str::Utf8Error), // for bytes to str
    FileError(std::io::Error, std::path::PathBuf), // path which operation was attempted on
    NoFilenameInPathError(std::path::PathBuf), // the path which has no filename
    OsStrUnicodeError(std::ffi::OsString), // for OsStr to str
    PathUnicodeError(std::path::PathBuf), // for Path to str
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::BytesUnicodeError(e) => write!(f, "Invalid unicode: {}", e),
            Error::FileError(e, p) => write!(f, "Error manipulating file {:?}: {}", p, e),
            Error::NoFilenameInPathError(p) => write!(f, "No filename in path {:?}", p),
            Error::OsStrUnicodeError(o) => write!(f, "Invalid unicode in {:?}", o),
            Error::PathUnicodeError(p) => write!(f, "Invalid unicode in {:?}", p),
        }
    }
}

/// Read the entire contents of `file` and attempt to convert them to a PathBuf.
fn read_path_from_file(file : &std::path::Path) -> Result<std::path::PathBuf, Error> {
    let bytes : std::vec::Vec<u8> = std::fs::read(file)
        .map_err(|e| Error::FileError(e, file.to_path_buf()))?;
    let path_str : &str = std::str::from_utf8(bytes.as_slice())
        .map_err(Error::BytesUnicodeError)?;
    // Make and return an owned path
    Ok(std::path::PathBuf::from(std::path::Path::new(path_str)))
}

/// Extract the filename from a path, converting any errors to our error format.
///
/// This will yield an error if the filename is not valid unicode, or if there is no filename in the
/// path.
fn path_to_name<'a>(path : &'a std::path::Path) -> Result<&'a str, Error> {
    let os_name : &std::ffi::OsStr = path.file_name()
        .ok_or(Error::NoFilenameInPathError(path.to_path_buf()))?;
    os_name.to_str()
        .ok_or(Error::OsStrUnicodeError(os_name.to_os_string()))
}

/// Given the `headstate` path, compute the new name for `file`.
fn new_name(headstate : &std::path::Path,
            file : &std::path::Path) -> Result<String, Error> {
    // Original name (the one that is being renamed)
    let orig_name : &str = path_to_name(file)?;

    // "Head" name
    let headfile = read_path_from_file(headstate)?;
    let headname : &str = path_to_name(&headfile)?;

    // Construct new name
    let base_name : &str = match headname.rsplit_once(".") {
        Some((a, _)) => a,
        None         => headname
    };
    Ok(String::from(base_name) + BASE_NAME_ORIG_NAME_SEP + orig_name)
}

fn main() {
    // Get first line from stdin
    let mut line_buf = String::new();
    match std::io::stdin().read_line(&mut line_buf) {
        Ok(_)  => (),
        Err(e) => panic!("Tried to read a line on stdin, but instead got \"{}\": {}",
                         line_buf, e)
    };

    // Manipulate paths
    let input_file : &std::path::Path = std::path::Path::new(line_buf.trim());
    let headstate_path : std::path::PathBuf = find_headstate(input_file);
    let parent_path : &std::path::Path = match input_file.parent() {
        Some(p) => p,
        None    => panic!("No parent dir of path {} received on stdin", headstate_path.display())
    };

    // Choose action based on first argument
    let key : String = std::env::args().nth(1).expect("Expects a key code as argument");
    match key.as_ref() {
        // Rename the file based on the head
        "e" => {
            let end_name : String = match new_name(&headstate_path, input_file) {
                Ok(n)  => n,
                Err(e) => panic!("Could not find new name for {:?}: {}", input_file, e)
            };
            let new_path : std::path::PathBuf = parent_path.with_file_name(end_name);
            match std::fs::rename(input_file, &new_path) {
                Ok(_)  => (),
                Err(e) => panic!("Could not rename {:?} to {:?}: {}", input_file, new_path, e)
            }
        },
        // Set file as new head
        "r" => {
            match write_headstate(headstate_path.as_ref(), input_file) {
                Ok(_) => (),
                Err(e) => panic!("Could not write {} into {:?}: {}", line_buf,
                                 headstate_path, e)
            };
        },
        // Bail-out key
        "q" => {},
        // Reject any other key
        _   => panic!("Unknown key: {}", key)
    }
}
