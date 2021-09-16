
use std::path::{PathBuf,Path};
use super::compat::canonicalize;

const HELP: &'static str = r#"
win_cp v1.0.0
cody <codylaeder@gmail.com>

Handles some of the deeply cursed pathing
cygwin/mingw64 environments create on
windows platforms.

USAGE:
    win_cp [FLAGS] [COPY_FROM] [COPY_TO]

FLAGS:
    -h, -H, --help     Displays this message
    -v, -V, --version  Displays version
    -f, -F, --force    Overwrites target file

ARGS:
    COPY_FROM          File to copy (can be multiple)
    COPY_TO            Folder/dir to copy to
                       Last (non-flag) arg is always
                       assumed to be destination

NOTES:
  Most of this application's code is targetted at
  handling paths with cygwin/mingw64 create.

  Input              | Output
  -------------------+------------------------------
  /c/Users/valarauca | C:\Users\Valarauca
  ~/Documents        | C:\Users\Valarauca\Documents
  C:\Users\Valarauca | C:\Users\Valarauca

  In short '/' and '\' are fixed by the application.
  While '.' and '..' are handled by Win32 API.
  UNC paths are accepted, they shouldn't be mangled.
"#;

pub struct Todo {
    pub overwrite: bool,
    pub copy_pairs: Vec<(PathBuf,PathBuf)>,
}

impl Todo {

    pub fn new() -> Result<Self,Box<dyn std::error::Error>> {
        let args = read_args();
        let (help, args) = bin_flag(args, |x| x == "-h" || x == "-H" || x == "--help");
        if help {
            print!("{}\n", HELP);
            std::process::exit(0);
        }
        let (vers, args) = bin_flag(args, |x| x == "-v" || x == "-V" || x == "--version");
        if vers {
            println!("{}", "1.0.0");
            std::process::exit(0);
        }
        let (forc, args) = bin_flag(args, |x| x == "-f" || x == "-F" || x == "--force");
        if args.len() < 2 {
            eprintln!("need 2 arguments to copy");
            std::process::exit(1);
        }

        Ok(Todo {
            overwrite: forc,
            copy_pairs: arg_logic(args)?,
        })
    }
}

fn arg_logic(args: Vec<String>) -> Result<Vec<(PathBuf,PathBuf)>, Box<dyn std::error::Error>> {

    let mut paths = Vec::with_capacity(args.len());
    for arg in args {
        let canonized = canonicalize(&arg)?;
        #[cfg(debug_assertions)]
        println!("{} -> {}", &arg, canonized);
        paths.push(canonized);
    }

    if paths.len() < 2 {
        panic!("checked before call")
    }
    let dest = paths.pop().unwrap();

    let mut sources = Vec::with_capacity(paths.len());
    for path in paths {
        let buf = PathBuf::from(path);
        if !buf.is_file() {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, format!("{:?}", &buf))));
        }
        sources.push(buf);
    }


    // precompute some shit
    let trailing_slash = dest.ends_with("\\");
    let dest = PathBuf::from(&dest);
    let dest_is_dir = dest.is_dir();
    return if sources.len() == 1 {
        let src = sources.pop().unwrap();
        debug_assert!(sources.is_empty());
        match modify_dest_path(&dest, trailing_slash, dest_is_dir, &src) {
            Option::Some(new_dest) => {
                Ok(vec![(new_dest,src)])
            },
            Option::None => {
                Ok(vec![(dest,src)])
            }
        }
    } else if !dest_is_dir && !trailing_slash {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, format!("destination dir:'{:?}' is invalid", &dest))))
    } else {
       Ok(sources.into_iter() 
           .map(|src| (modify_dest_path(&dest, trailing_slash, dest_is_dir, &src).unwrap(), src))
           .collect())
    };
}

fn modify_dest_path(dest_buf: &PathBuf, trailing_slash: bool, is_dir: bool, input: &Path) -> Option<PathBuf> {
   if trailing_slash || is_dir {
        let mut buf = dest_buf.clone();
        buf.push(input.file_name().expect("we already checked for filename"));
        Some(buf)
   } else {
       None
   }
}

fn read_args() -> Vec<String> {
    std::env::args().skip(1).collect()
}

fn bin_flag<F>(args: Vec<String>, lambda: F) -> (bool,Vec<String>)
where
    F: Fn(&str) -> bool,
{
    let mut flag = false;
    let mut v = Vec::with_capacity(args.len());
    for item in args {
        if lambda(&item) {
            flag = true
        } else {
            v.push(item);
        }
    }
    (flag, v)
}

