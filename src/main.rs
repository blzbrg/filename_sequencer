static HEADFILE_NAME : &str = "fname_seq_head";
static BASE_NAME_ORIG_NAME_SEP : &str = "_";

fn find_headstate<T: AsRef<std::path::Path>>(base_path : T) -> std::path::PathBuf {
    base_path.as_ref().with_file_name(HEADFILE_NAME)
}

fn write_headstate(headstate : &std::path::Path, file : &std::path::Path)
                   -> Result<(), std::io::Error> {
    let output = file.to_str()
        .expect("Filepath cannot be converted to string for writing to headstate");
    std::fs::write(headstate, output)
        .map_err(std::io::Error::into)
}

#[derive(Debug)]
enum Error {
    Utf8Error(std::str::Utf8Error),
    FileError(std::io::Error),
    NoFilenameInPathError,
    OsStrUnicodeError // for OsStr to str
}

impl From<std::str::Utf8Error> for Error {
    fn from(e : std::str::Utf8Error) -> Error {
        Error::Utf8Error(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e : std::io::Error) -> Error {
        Error::FileError(e)
    }
}


/// Read the entire contents of `file` and attempt to convert them to a PathBuf.
fn read_path_from_file<T: AsRef<std::path::Path>>(file : T) -> Result<std::path::PathBuf, Error> {
    let bytes : std::vec::Vec<u8> = std::fs::read(file)?;
    let path_str : &str = std::str::from_utf8(bytes.as_slice())?;
    // Make and return an owned path
    Ok(std::path::PathBuf::from(std::path::Path::new(path_str)))
}

fn path_to_name<'a>(path : &'a std::path::Path) -> Result<&'a str, Error> {
    let os_name : &std::ffi::OsStr = path.file_name()
        .ok_or(Error::NoFilenameInPathError)?;
    os_name.to_str()
        .ok_or(Error::OsStrUnicodeError)
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
    let _ = std::io::stdin().read_line(&mut line_buf)
        .expect("Expects a file path on stdin");

    // Manipulate paths
    let input_file : &std::path::Path = std::path::Path::new(line_buf.trim());
    let headstate_path : std::path::PathBuf = find_headstate(input_file);
    let parent_path : &std::path::Path = input_file.parent().expect("No parent of file");

    // Choose action based on first argument
    let key : String = std::env::args().nth(1).expect("Expects a key code as argument");
    match key.as_ref() {
        // Rename the file based on the head
        "e" => {
            let end_name : String = new_name(&headstate_path, input_file)
                .expect("Could not create new name");
            let new_path : std::path::PathBuf = parent_path.with_file_name(end_name);
            std::fs::rename(input_file, new_path).expect("Failed to move");
        },
        // Set file as new head
        "r" => {
            write_headstate(headstate_path.as_ref(), input_file)
                .expect("Could not write head state");
        },
        // Reject any other key
        _   => panic!("Unknown key: {}", key)
    }
}
